//! MCP dispatch tests (p13-c003 `mcp.tool.list`, p14-c001 `mcp.tool.call`), split
//! from `tests/mod.rs` to keep every file under the 500-line limit. `use super::*;`
//! resolves the shared fakes/helpers in `mod.rs` — but the *recording* MCP fake
//! (which captures the forwarded `name`+`arguments`) is local to these tests, since
//! `mod.rs`'s `FakeMcp` is deliberately non-recording.

use super::*;
use std::io;
use std::sync::Mutex as StdMutex;
use tracing_subscriber::fmt::MakeWriter;

/// A recording MCP fake: captures the exact `name` + `arguments` forwarded to
/// `call_tool`, so tests can assert the runner threads them through verbatim.
#[derive(Default)]
struct RecordingMcp {
    last_name: Mutex<Option<String>>,
    last_arguments: Mutex<Option<serde_json::Value>>,
    call_tool_hits: AtomicBool,
    list_tools_hits: AtomicBool,
}
#[async_trait]
impl McpClient for RecordingMcp {
    async fn list_tools(&self) -> Result<serde_json::Value, PortError> {
        self.list_tools_hits.store(true, Ordering::SeqCst);
        Ok(serde_json::json!({"tools": [{"name": "echo"}]}))
    }
    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, PortError> {
        self.call_tool_hits.store(true, Ordering::SeqCst);
        *self.last_name.lock().expect("name lock") = Some(name.to_owned());
        *self.last_arguments.lock().expect("args lock") = Some(arguments.clone());
        // Echo the arguments back so the caller sees a real, non-null result.
        Ok(serde_json::json!({ "echoed": arguments }))
    }
}

/// A runner wired with a specific (recording) MCP client; forge/gate/store are
/// default fakes.
fn runner_with_mcp(mcp: Arc<RecordingMcp>) -> TaskRunner {
    TaskRunner::new(
        Arc::new(FakeForge::default()),
        Arc::new(FakeFabric),
        Arc::new(FakeGate::default()),
        mcp,
        Arc::new(crate::project_store::InMemoryProjectStore::new()),
    )
}

// ---- p13-c003 (moved): mcp.tool.list ----

#[tokio::test]
async fn mcp_tool_list_runs_as_viewer_and_lists_tools() {
    // mcp.tool.list is a benign discovery read — viewer reaches list_tools.
    let mcp = Arc::new(RecordingMcp::default());
    let r = runner_with_mcp(mcp.clone());
    let out = r
        .run(&task("mcp.tool.list"), &auth(&["viewer"]))
        .await
        .expect("viewer lists tools");
    assert!(
        mcp.list_tools_hits.load(Ordering::SeqCst),
        "mcp.tool.list must call list_tools"
    );
    assert!(
        !mcp.call_tool_hits.load(Ordering::SeqCst),
        "a listing must never invoke a tool"
    );
    assert!(out.get("tools").is_some(), "returns the tools envelope");
}

#[test]
fn every_mcp_catalog_kind_is_dispatched() {
    // Guard against a future TargetPort::Mcp kind being catalogued without a
    // `dispatch_mcp` arm — it would hit the clean catch-all ("mcp kind not
    // implemented") rather than its intended handler. Extended to both kinds in
    // p14-c001 (list + call).
    const MCP_KINDS: &[&str] = &["mcp.tool.list", "mcp.tool.call"];
    for entry in catalog::CATALOG
        .iter()
        .filter(|e| e.target == TargetPort::Mcp)
    {
        assert!(
            MCP_KINDS.contains(&entry.kind),
            "Mcp catalog kind '{}' has no dispatch_mcp arm — add it or update MCP_KINDS",
            entry.kind
        );
    }
}

// ---- p14-c001: mcp.tool.call (invoke) ----

#[tokio::test]
async fn mcp_tool_call_forwards_name_and_arguments_verbatim() {
    // GW1 (real invoke): operator invokes a downstream tool; the runner threads the
    // exact `name` + `arguments` through to call_tool and returns its result.
    let mcp = Arc::new(RecordingMcp::default());
    let r = runner_with_mcp(mcp.clone());
    let mut t = task("mcp.tool.call");
    t.input = serde_json::json!({
        "name": "search",
        "arguments": { "query": "widgets", "limit": 5 }
    });
    let out = r.run(&t, &auth(&["operator"])).await.expect("invoke");

    assert!(
        mcp.call_tool_hits.load(Ordering::SeqCst),
        "mcp.tool.call must call call_tool"
    );
    assert!(
        !mcp.list_tools_hits.load(Ordering::SeqCst),
        "an invoke must never fall through to list_tools"
    );
    assert_eq!(
        mcp.last_name.lock().expect("name lock").as_deref(),
        Some("search"),
        "runner must forward the tool name verbatim"
    );
    assert_eq!(
        *mcp.last_arguments.lock().expect("args lock"),
        Some(serde_json::json!({ "query": "widgets", "limit": 5 })),
        "runner must forward the arguments verbatim"
    );
    assert_eq!(out["echoed"]["query"], "widgets", "returns the port result");
}

#[tokio::test]
async fn mcp_tool_call_denies_sub_operator_before_any_port_call() {
    // mcp.tool.call floors at operator; a viewer is rejected by the permission gate
    // BEFORE any port call (Base Rule 33).
    let mcp = Arc::new(RecordingMcp::default());
    let r = runner_with_mcp(mcp.clone());
    let mut t = task("mcp.tool.call");
    t.input = serde_json::json!({ "name": "search", "arguments": {} });
    let err = r.run(&t, &auth(&["viewer"])).await.unwrap_err();
    assert!(matches!(err, AppError::Unauthorized(_)));
    assert!(
        !mcp.call_tool_hits.load(Ordering::SeqCst),
        "denied invoke must not reach call_tool"
    );
}

#[tokio::test]
async fn mcp_tool_call_missing_arguments_is_rejected() {
    // Schema requires `arguments`; its absence fails validation before any port call.
    let mcp = Arc::new(RecordingMcp::default());
    let r = runner_with_mcp(mcp.clone());
    let mut t = task("mcp.tool.call");
    t.input = serde_json::json!({ "name": "search" });
    let err = r.run(&t, &auth(&["operator"])).await.unwrap_err();
    assert!(matches!(err, AppError::InvalidInput(_)));
    assert!(
        !mcp.call_tool_hits.load(Ordering::SeqCst),
        "malformed invoke must not reach call_tool"
    );
}

/// A `MakeWriter` that appends every formatted tracing line into a shared buffer,
/// so a test can inspect exactly what the runner emitted.
#[derive(Clone)]
struct BufWriter(Arc<StdMutex<Vec<u8>>>);
impl io::Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("log buffer lock")
            .extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl<'a> MakeWriter<'a> for BufWriter {
    type Writer = BufWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[tokio::test]
async fn mcp_tool_call_never_logs_arguments() {
    // GW: the runner must NEVER log MCP tool-call `arguments` — they may carry
    // operator secrets/PII. Capture ALL tracing output for a real invoke and assert
    // a sentinel argument value never appears in it.
    const SENTINEL: &str = "s3cr3t-argument-value";
    let buf = Arc::new(StdMutex::new(Vec::<u8>::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(BufWriter(buf.clone()))
        .with_max_level(tracing::Level::TRACE)
        .without_time()
        .finish();

    let mcp = Arc::new(RecordingMcp::default());
    let r = runner_with_mcp(mcp.clone());
    let mut t = task("mcp.tool.call");
    t.input = serde_json::json!({
        "name": "search",
        "arguments": { "token": SENTINEL }
    });

    tracing::subscriber::with_default(subscriber, || {
        // Drive the async run to completion inside the capturing scope on this
        // (current-thread) runtime, so all spans/events route to our subscriber.
        futures::executor::block_on(async {
            r.run(&t, &auth(&["operator"])).await.expect("invoke");
        });
    });

    // Sanity: the invoke actually forwarded the sentinel to the port…
    assert_eq!(
        mcp.last_arguments.lock().expect("args lock").as_ref(),
        Some(&serde_json::json!({ "token": SENTINEL })),
        "precondition: the sentinel must have reached the port"
    );
    // …yet it must appear NOWHERE in the emitted logs.
    let logged = String::from_utf8(buf.lock().expect("buf lock").clone()).expect("utf8 logs");
    assert!(
        !logged.contains(SENTINEL),
        "MCP tool-call arguments must never be logged; found sentinel in:\n{logged}"
    );
    // The audit trail must still record the kind (so the invoke is auditable).
    assert!(
        logged.contains("mcp.tool.call"),
        "the invoke must still be audited by kind:\n{logged}"
    );
}
