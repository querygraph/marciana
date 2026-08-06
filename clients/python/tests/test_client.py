import unittest

from marciana_client import (
    ForgetRequest,
    MarcianaClient,
    MemoryReceipt,
    RecallRequest,
    RememberRequest,
)


class Transport:
    def __init__(self) -> None:
        self.calls = []

    def post(self, path, payload):
        self.calls.append((path, payload))
        return {"allowed": True, "memory_ids": ["m1"]}


class ClientTests(unittest.TestCase):
    def test_routes_typed_requests_and_returns_receipts(self) -> None:
        transport = Transport()
        client = MarcianaClient(transport)
        receipt = client.remember(
            RememberRequest(space="tenant/coffee", text="price", purpose="research")
        )
        self.assertIsInstance(receipt, MemoryReceipt)
        self.assertEqual(transport.calls[0][0], "/v1/memory/remember")
        self.assertEqual(transport.calls[0][1]["kind"], "semantic")
        client.recall(RecallRequest(space="tenant/coffee", query="price", purpose="research"))
        client.forget(
            ForgetRequest(space="tenant/coffee", ids=["m1"], purpose="research")
        )
        self.assertEqual([call[0] for call in transport.calls], [
            "/v1/memory/remember", "/v1/memory/recall", "/v1/memory/forget"
        ])

    def test_requests_reject_unbounded_or_unknown_fields(self) -> None:
        with self.assertRaises(ValueError):
            RememberRequest(space="tenant/coffee", text="", purpose="research")
        with self.assertRaises(ValueError):
            RecallRequest(space="tenant coffee", query="price", purpose="research")


if __name__ == "__main__":
    unittest.main()
