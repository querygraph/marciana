import assert from "node:assert/strict";
import test from "node:test";
import { readFile } from "node:fs/promises";
import { MarcianaClient } from "../dist/index.js";

const FIXTURE_URL = new URL("../../../compat/fixtures/api_remember_v1.json", import.meta.url);

test("routes a typed remember request through injected transport", async () => {
  const calls = [];
  const client = new MarcianaClient({
    async post(path, payload) {
      calls.push([path, payload]);
      return { operation: "remember", allowed: true, memory_ids: ["m1"] };
    },
  });
  const receipt = await client.remember({ space_id: "tenant/coffee", text: "price", purpose: "research" });
  assert.deepEqual(receipt.memory_ids, ["m1"]);
  assert.equal(calls[0][0], "/v1/memory/remember");
});

test("rejects invalid identities before transport", async () => {
  const client = new MarcianaClient({ post: async () => { throw new Error("transport called"); } });
  await assert.rejects(() => client.recall({ space_id: "tenant coffee", query: "price", purpose: "research" }), /invalid memory identity/);
});

test("rejects invalid nested improve replacements before transport", async () => {
  const client = new MarcianaClient({ post: async () => { throw new Error("transport called"); } });
  await assert.rejects(
    () => client.improve({
      space_id: "tenant/coffee",
      memory_id: "m1",
      replacement: { space_id: "tenant/coffee", text: "", purpose: "research" },
    }),
    /invalid memory text/,
  );
});

test("rejects invalid forget memory ids before transport", async () => {
  const client = new MarcianaClient({ post: async () => { throw new Error("transport called"); } });
  await assert.rejects(
    () => client.forget({ space_id: "tenant/coffee", memory_ids: ["bad id"], purpose: "research" }),
    /invalid memory identity/,
  );
});

test("remember payload round-trips the shared wire fixture", async () => {
  const fixture = JSON.parse(await readFile(FIXTURE_URL, "utf8"));
  let payload;
  const client = new MarcianaClient({
    async post(_path, body) {
      payload = body;
      return { operation: "remember", allowed: true, memory_ids: ["m1"] };
    },
  });
  await client.remember(fixture);
  assert.deepEqual(payload, fixture);
  assert.equal(payload.spaceId, undefined);
});

test("forget sends memory_ids on the shared wire", async () => {
  let payload;
  const client = new MarcianaClient({
    async post(_path, body) {
      payload = body;
      return { operation: "forget", allowed: true, memory_ids: ["m1"] };
    },
  });
  await client.forget({ space_id: "tenant/coffee", memory_ids: ["m1"], purpose: "research" });
  assert.deepEqual(payload.memory_ids, ["m1"]);
  assert.equal(payload.ids, undefined);
});
