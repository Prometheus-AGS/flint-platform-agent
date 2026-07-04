import { test, expect, APIRequestContext } from "@playwright/test";
import jwt from "jsonwebtoken";

// HS256 shared secret — MUST match compose.smoke.yml's FPA_GATE_JWT_KEY.
const SECRET = process.env.FPA_GATE_JWT_KEY ?? "smoke-hs256-secret-not-a-real-credential";

// Mint a bearer the agent's HS256 verify path accepts. GateClaims = { sub, roles, exp }.
// project.create needs "operator"; project.inspect/list need "viewer" — include both.
function bearer(roles: string[] = ["operator", "viewer"]): string {
  return jwt.sign(
    { sub: "smoke-operator", roles, exp: 4_102_444_800 /* year 2100 */ },
    SECRET,
    { algorithm: "HS256" },
  );
}

const auth = (roles?: string[]) => ({ authorization: `Bearer ${bearer(roles)}` });

// A2A submit: POST /a2a/tasks {kind, input} -> TaskEvent (serde tag "type").
async function a2a(req: APIRequestContext, kind: string, input: unknown) {
  return req.post("/a2a/tasks", {
    headers: auth(),
    data: { kind, input },
  });
}

test("healthz is up", async ({ request }) => {
  const r = await request.get("/healthz");
  expect(r.status()).toBe(200);
});

test("unauthenticated requests are rejected", async ({ request }) => {
  // No bearer on protected surfaces -> 401.
  const stream = await request.get("/agui/stream");
  expect(stream.status()).toBe(401);
  const submit = await request.post("/a2a/tasks", { data: { kind: "fabric.health", input: {} } });
  expect(submit.status()).toBe(401);
});

test("project CRUD round-trips through the live agent's store", async ({ request }) => {
  const projectId = "00000000-0000-0000-0000-000000009001";

  // create
  const created = await a2a(request, "project.create", { name: "smoke-project", project_id: projectId });
  expect(created.status(), await created.text()).toBe(200);
  const createdBody = await created.json();
  expect(createdBody.type).toBe("completed");

  // inspect -> returns the stored aggregate
  const inspected = await a2a(request, "project.inspect", { project_id: projectId });
  expect(inspected.status(), await inspected.text()).toBe(200);
  expect((await inspected.json()).type).toBe("completed");

  // list -> contains it
  const listed = await a2a(request, "project.list", {});
  expect(listed.status()).toBe(200);
  expect((await listed.json()).type).toBe("completed");
});

test("fabric.health flows through to the stub", async ({ request }) => {
  const r = await a2a(request, "fabric.health", {});
  expect(r.status(), await r.text()).toBe(200);
  expect((await r.json()).type).toBe("completed");
});

test("MCP tools/list and tools/call dispatch", async ({ request }) => {
  const list = await request.post("/mcp", {
    headers: auth(),
    data: { jsonrpc: "2.0", id: 1, method: "tools/list" },
  });
  expect(list.status(), await list.text()).toBe(200);
  expect((await list.json()).jsonrpc).toBe("2.0");

  const call = await request.post("/mcp", {
    headers: auth(),
    data: { jsonrpc: "2.0", id: 2, method: "tools/call", params: { name: "fabric.health", arguments: {} } },
  });
  expect(call.status(), await call.text()).toBe(200);
});

test("authenticated agui stream opens", async ({ request }) => {
  const r = await request.get("/agui/stream", { headers: auth() });
  expect(r.status()).toBe(200);
});
