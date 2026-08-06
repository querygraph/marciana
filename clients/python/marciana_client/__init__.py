"""Small typed client boundary for Marciana's four memory verbs.

Authentication and authorization stay in the injected transport; this package
only validates stable request/receipt shapes and routes them.
"""

from .client import MarcianaClient, MemoryTransport
from .mcp import McpMemoryAdapter, McpTool
from .models import (
    ForgetRequest,
    ImproveRequest,
    MemoryReceipt,
    RecallRequest,
    RememberRequest,
)

__all__ = [
    "ForgetRequest",
    "ImproveRequest",
    "MarcianaClient",
    "McpMemoryAdapter",
    "McpTool",
    "MemoryReceipt",
    "MemoryTransport",
    "RecallRequest",
    "RememberRequest",
]
