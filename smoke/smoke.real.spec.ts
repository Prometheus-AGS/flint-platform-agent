import { test, expect, APIRequestContext } from "@playwright/test";
import jwt from "jsonwebtoken";
import { randomUUID } from "node:crypto";

// HS256 shared secret — MUST match compose.real.yml's FPA_GATE_JWT_KEY.
const SECRET = process.env.FPA_GATE_JWT_KEY ?? "smoke-hs256-secret-not-a-real-credential";
const AGENT = process.env.SMOKE_BASE_URL ?? "http://localhost:8088";
const FABRIC = process.env.FABRIC_BASE_URL ?? "http://localhost:28080";

function bearer(roles: string[] = ["operator", "viewer"]): string {
  return jwt.sign(
    { sub: "smoke-operator", roles, exp: 4_102_444_800 },
    SECRET,
    { algorithm: "HS256" },
  );
}
const auth = (roles?: string[]) => ({ authorization: `Bearer ${bearer(roles)}` });

async function a2a(req: APIRequestContext, kind: string, input: unknown) {
  return req.post(`${AGENT}/a2a/tasks`, { headers: auth(), data: { kind, input } });
}

// ─────────────────────────── HTTP hops (real stack) ───────────────────────────

test("agent healthz is up", async ({ request }) => {
  expect((await request.get(`${AGENT}/healthz`)).status()).toBe(200);
});

test("unauthenticated protected surfaces are rejected", async ({ request }) => {
  expect((await request.get(`${AGENT}/agui/stream`)).status()).toBe(401);
});

test("fabric.health flows through to the REAL fabric gateway", async ({ request }) => {
  const r = await a2a(request, "fabric.health", {});
  expect(r.status(), await r.text()).toBe(200);
  expect((await r.json()).type).toBe("completed");
});

// NOTE: the gate plane is proven standalone (c001: agent's GET /routes hop → 200 against
// the real gate admin). It is NOT exercised via A2A here because the task catalog has no
// read-only gate kind — every TargetPort::Gate entry (application.deploy) is a WRITE that
// dispatch_gate refuses until route-writes are implemented. A read-only gate catalog kind
// is a follow-up; the real gate still runs in this stack (fabric points at its JWKS).

test("project CRUD round-trips through the live agent store (real Postgres)", async ({ request }) => {
  const projectId = "00000000-0000-0000-0000-000000009101";
  const created = await a2a(request, "project.create", { name: "real-smoke", project_id: projectId });
  expect(created.status(), await created.text()).toBe(200);
  expect((await created.json()).type).toBe("completed");
  const listed = await a2a(request, "project.list", {});
  expect((await listed.json()).type).toBe("completed");
});

// ─────────────────────────── realtime event (the proof) ───────────────────────────
//
// Agent subscribes to fabric channel X (via the agent's /fabric/subscribe SSE bridge)
// -> we POST an EventEnvelope to fabric's /v1/publish on the SAME channel X
// -> assert the agent's SSE stream emits that envelope.
//
// This is the deterministic path fabric's own subscribe_mux.rs uses (publish to the
// subscribed channel), NOT the dev-inject routes (which use random channels).

test("realtime: agent receives a fabric event over the /fabric/subscribe SSE bridge", async () => {
  test.setTimeout(30_000);
  const channel = randomUUID();
  const tenant = "00000000-0000-0000-0000-000000000001";

  // 1. Open the agent's SSE subscription to channel X.
  const ac = new AbortController();
  const sseResp = await fetch(`${AGENT}/fabric/subscribe?channel=${channel}`, {
    headers: { authorization: `Bearer ${bearer()}`, accept: "text/event-stream" },
    signal: ac.signal,
  });
  expect(sseResp.status, `subscribe open: ${sseResp.status}`).toBe(200);
  const reader = sseResp.body!.getReader();
  const decoder = new TextDecoder();

  // Give the WS subscription a beat to establish on the fabric side.
  await new Promise((r) => setTimeout(r, 1_000));

  // 2. Publish an EventEnvelope to fabric on the SAME channel.
  const eventId = randomUUID();
  const envelope = {
    id: eventId,
    channel: { id: channel, tenant_id: tenant, path: "entity/smoke/updates" },
    offset: 1,
    kind: "entity_change",
    payload: { op: "insert", smoke: true },
    timestamp: new Date(0).toISOString(),
    correlation_id: null,
  };
  const pub = await fetch(`${FABRIC}/v1/publish`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(envelope),
  });
  expect([200, 202]).toContain(pub.status);

  // 3. Read the agent SSE stream until we see our event id (or time out).
  let received = "";
  const deadline = Date.now() + 15_000;
  try {
    while (Date.now() < deadline) {
      const { value, done } = await reader.read();
      if (done) break;
      received += decoder.decode(value, { stream: true });
      if (received.includes(eventId)) break;
    }
  } finally {
    ac.abort();
  }

  expect(received, `agent SSE did not carry the published event within 15s. Got: ${received.slice(0, 500)}`)
    .toContain(eventId);
});
