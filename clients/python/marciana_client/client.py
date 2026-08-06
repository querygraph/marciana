"""Transport-neutral typed client for Marciana memory routes."""

from __future__ import annotations

from typing import Any, Protocol

from .models import (
    ForgetRequest,
    ImproveRequest,
    MemoryReceipt,
    RecallRequest,
    RememberRequest,
)


class MemoryTransport(Protocol):
    """Authenticated transport owned by the embedding application."""

    def post(self, path: str, payload: dict[str, Any]) -> dict[str, Any]: ...


class MarcianaClient:
    """Route typed requests without implementing authorization or storage."""

    def __init__(self, transport: MemoryTransport) -> None:
        self._transport = transport

    def remember(self, request: RememberRequest) -> MemoryReceipt:
        return self._call("/v1/memory/remember", request, "remember")

    def recall(self, request: RecallRequest) -> MemoryReceipt:
        return self._call("/v1/memory/recall", request, "recall")

    def improve(self, request: ImproveRequest) -> MemoryReceipt:
        return self._call("/v1/memory/improve", request, "improve")

    def forget(self, request: ForgetRequest) -> MemoryReceipt:
        return self._call("/v1/memory/forget", request, "forget")

    def _call(self, path: str, request: Any, operation: str) -> MemoryReceipt:
        body = self._transport.post(path, request.model_dump(mode="json"))
        if body.get("operation", operation) != operation:
            raise ValueError("memory receipt operation mismatch")
        return MemoryReceipt.model_validate({**body, "operation": operation})
