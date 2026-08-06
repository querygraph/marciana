"""Minimal MCP-facing tool adapter over the typed Marciana client."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

from .client import MarcianaClient
from .models import ForgetRequest, ImproveRequest, RecallRequest, RememberRequest


@dataclass(frozen=True)
class McpTool:
    name: str
    description: str
    input_schema: dict[str, Any]


TOOLS = (
    McpTool("marciana_remember", "Store one governed memory proposal.", RememberRequest.model_json_schema()),
    McpTool("marciana_recall", "Recall through the governed vault.", RecallRequest.model_json_schema()),
    McpTool("marciana_improve", "Submit one governed improvement proposal.", ImproveRequest.model_json_schema()),
    McpTool("marciana_forget", "Request scoped audited forgetting.", ForgetRequest.model_json_schema()),
)


class McpMemoryAdapter:
    """Translate MCP tool calls into the existing client contract."""

    def __init__(self, client: MarcianaClient) -> None:
        self._client = client
        self._handlers: dict[str, Callable[[dict[str, Any]], Any]] = {
            "marciana_remember": lambda payload: client.remember(RememberRequest.model_validate(payload)),
            "marciana_recall": lambda payload: client.recall(RecallRequest.model_validate(payload)),
            "marciana_improve": lambda payload: client.improve(ImproveRequest.model_validate(payload)),
            "marciana_forget": lambda payload: client.forget(ForgetRequest.model_validate(payload)),
        }

    @staticmethod
    def tools() -> tuple[McpTool, ...]:
        return TOOLS

    def call(self, name: str, payload: dict[str, Any]) -> Any:
        try:
            handler = self._handlers[name]
        except KeyError as error:
            raise ValueError("unknown Marciana MCP tool") from error
        return handler(payload)
