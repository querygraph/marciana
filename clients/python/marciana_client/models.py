"""Versioned, strict request and receipt shapes for the four verbs.

The wire matches ``crates/marciana-memory/src/api.rs`` exactly: the server
denies unknown fields, so these models carry no client-only extras.
"""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field

Text = Field(min_length=1, max_length=16_384)
Identity = Field(min_length=1, max_length=256, pattern=r"^[A-Za-z0-9_:/.-]+$")
IdentityStr = Annotated[str, Identity]


class WireModel(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)


class RememberRequest(WireModel):
    space_id: str = Identity
    text: str = Text
    purpose: str = Identity


class RecallRequest(WireModel):
    space_id: str = Identity
    query: str = Text
    purpose: str = Identity


class ImproveRequest(WireModel):
    space_id: str = Identity
    memory_id: str = Identity
    replacement: RememberRequest


class ForgetRequest(WireModel):
    space_id: str = Identity
    memory_ids: list[IdentityStr] = Field(min_length=1, max_length=256)
    purpose: str = Identity


class MemoryReceipt(WireModel):
    operation: Literal["remember", "recall", "improve", "forget"]
    allowed: bool
    memory_ids: list[str] = Field(default_factory=list, max_length=256)
    detail: str | None = Field(default=None, max_length=512)
