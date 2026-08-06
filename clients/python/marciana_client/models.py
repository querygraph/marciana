"""Versioned, strict request and receipt shapes for the four verbs."""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

Text = Field(min_length=1, max_length=16_384)
Identity = Field(min_length=1, max_length=256, pattern=r"^[A-Za-z0-9_:/.-]+$")


class WireModel(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True)


class RememberRequest(WireModel):
    space: str = Identity
    text: str = Text
    purpose: str = Identity
    kind: Literal["semantic", "episodic", "profile", "event"] = "semantic"


class RecallRequest(WireModel):
    space: str = Identity
    query: str = Text
    purpose: str = Identity
    clearance: Literal["public", "internal", "sensitive", "secret"] = "internal"


class ImproveRequest(WireModel):
    space: str = Identity
    memory_id: str = Identity
    replacement: RememberRequest


class ForgetRequest(WireModel):
    space: str = Identity
    memory_ids: list[str] = Field(min_length=1, max_length=256)
    purpose: str = Identity
    mode: Literal["erase", "retract"] = "retract"


class MemoryReceipt(WireModel):
    operation: Literal["remember", "recall", "improve", "forget"]
    allowed: bool
    memory_ids: list[str] = Field(default_factory=list, max_length=256)
    detail: str | None = Field(default=None, max_length=512)
