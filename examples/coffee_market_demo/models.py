"""Typed domain objects for the Honduras coffee-market demonstration."""

from __future__ import annotations

from datetime import date
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class CoffeeRow(BaseModel):
    model_config = ConfigDict(extra="ignore")

    observation_id: str
    country: str = "Honduras"
    commodity: str = "coffee"
    market: str
    price_usd_per_kg: float | None = Field(default=None, ge=0)
    observed_on: date
    source: str
    note: str = ""


class DataverseLoadReport(BaseModel):
    dataset_url: str
    row_count: int = Field(ge=0)
    source_file: str
    table_name: str
    sail_loaded: bool


class MemoryFact(BaseModel):
    model_config = ConfigDict(extra="forbid")

    memory_id: str
    text: str
    source: str
    observed_on: date
    state: Literal["current", "superseded", "forgotten"] = "current"
    confidence_basis_points: int = Field(ge=0, le=10_000)


class MemoryReceipt(BaseModel):
    operation: Literal["remember", "recall", "improve", "forget"]
    allowed: bool
    memory_ids: list[str] = Field(default_factory=list)
    detail: str = ""


class AgentDecision(BaseModel):
    """Structured Pydantic AI v2 output for each agent turn."""

    agent: str
    action: Literal["learn", "recall", "improve", "forget", "report"]
    claim: str
    confidence_basis_points: int = Field(ge=0, le=10_000)
    memory_ids: list[str] = Field(default_factory=list)
    citations: list[str] = Field(default_factory=list)
