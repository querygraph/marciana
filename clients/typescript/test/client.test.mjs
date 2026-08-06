import assert from "node:assert/strict";
import test from "node:test";
import { MarcianaClient } from "../dist/index.js";

test("routes a typed remember request through injected transport", async () => {
  const calls = [];
  const client = new MarcianaClient({
    async post(path, payload) {
      calls.push([path, payload]);
      return { operation: "remember", allowed: true, memoryIds: ["m1"] };
    },
  });
  const receipt = await client.remember({ space: "tenant/coffee", text: "price", purpose: "research" });
  assert.deepEqual(receipt.memoryIds, ["m1"]);
  assert.equal(calls[0][0], "/v1/memory/remember");
});

test("rejects invalid identities before transport", async () => {
  const client = new MarcianaClient({ post: async () => { throw new Error("transport called"); } });
  await assert.rejects(() => client.recall({ space: "tenant coffee", query: "price", purpose: "research" }), /invalid memory identity/);
});
