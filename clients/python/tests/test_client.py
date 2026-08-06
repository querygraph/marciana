import json
import unittest
from pathlib import Path

from marciana_client import (
    ForgetRequest,
    ImproveRequest,
    MarcianaClient,
    MemoryReceipt,
    RecallRequest,
    RememberRequest,
)

FIXTURE = Path(__file__).resolve().parents[3] / "compat" / "fixtures" / "api_remember_v1.json"


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
            RememberRequest(space_id="tenant/coffee", text="price", purpose="research")
        )
        self.assertIsInstance(receipt, MemoryReceipt)
        client.recall(RecallRequest(space_id="tenant/coffee", query="price", purpose="research"))
        client.forget(
            ForgetRequest(space_id="tenant/coffee", memory_ids=["m1"], purpose="research")
        )
        self.assertEqual([call[0] for call in transport.calls], [
            "/v1/memory/remember", "/v1/memory/recall", "/v1/memory/forget"
        ])

    def test_requests_reject_unbounded_or_unknown_fields(self) -> None:
        with self.assertRaises(ValueError):
            RememberRequest(space_id="tenant/coffee", text="", purpose="research")
        with self.assertRaises(ValueError):
            RecallRequest(space_id="tenant coffee", query="price", purpose="research")
        with self.assertRaises(ValueError):
            RememberRequest(
                space_id="tenant/coffee", text="price", purpose="research", kind="semantic"
            )

    def test_forget_rejects_invalid_memory_id_items(self) -> None:
        with self.assertRaises(ValueError):
            ForgetRequest(space_id="tenant/coffee", memory_ids=["bad id"], purpose="research")

    def test_improve_validates_the_nested_replacement(self) -> None:
        with self.assertRaises(ValueError):
            ImproveRequest(
                space_id="tenant/coffee",
                memory_id="m1",
                replacement={"space_id": "tenant/coffee", "text": "", "purpose": "research"},
            )

    def test_remember_payload_round_trips_the_shared_wire_fixture(self) -> None:
        fixture = json.loads(FIXTURE.read_text())
        transport = Transport()
        MarcianaClient(transport).remember(RememberRequest.model_validate(fixture))
        self.assertEqual(transport.calls[0][1], fixture)

    def test_receipt_operation_mismatch_is_rejected(self) -> None:
        class MismatchTransport:
            def post(self, path, payload):
                return {"operation": "forget", "allowed": True, "memory_ids": []}

        client = MarcianaClient(MismatchTransport())
        with self.assertRaises(ValueError):
            client.remember(
                RememberRequest(space_id="tenant/coffee", text="price", purpose="research")
            )

    def test_forget_model_uses_memory_ids(self) -> None:
        request = ForgetRequest(space_id="tenant/coffee", memory_ids=["m1"], purpose="research")
        self.assertEqual(request.model_dump()["memory_ids"], ["m1"])
        self.assertNotIn("ids", request.model_dump())


if __name__ == "__main__":
    unittest.main()
