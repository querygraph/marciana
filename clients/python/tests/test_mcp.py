import unittest

from marciana_client import MarcianaClient, McpMemoryAdapter


class Transport:
    def post(self, path, payload):
        return {"allowed": True, "memory_ids": ["mcp-1"]}


class McpTests(unittest.TestCase):
    def test_tools_are_typed_and_dispatch_through_client(self) -> None:
        adapter = McpMemoryAdapter(MarcianaClient(Transport()))
        self.assertEqual(
            {tool.name for tool in adapter.tools()},
            {"marciana_remember", "marciana_recall", "marciana_improve", "marciana_forget"},
        )
        receipt = adapter.call(
            "marciana_remember",
            {"space_id": "tenant/coffee", "text": "price", "purpose": "research"},
        )
        self.assertEqual(receipt.memory_ids, ["mcp-1"])

    def test_unknown_tool_fails_closed(self) -> None:
        adapter = McpMemoryAdapter(MarcianaClient(Transport()))
        with self.assertRaises(ValueError):
            adapter.call("marciana_search", {})


if __name__ == "__main__":
    unittest.main()
