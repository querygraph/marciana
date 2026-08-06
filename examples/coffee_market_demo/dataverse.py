"""Dataverse native API loading with a local fixture fallback."""

from __future__ import annotations

import csv
import io
import json
import urllib.request
from pathlib import Path
from typing import Any

from .models import CoffeeRow


class DataverseClient:
    def __init__(self, dataset_url: str | None, fixture: Path) -> None:
        self.dataset_url = dataset_url or "urn:fixture:coffee-honduras"
        self.fixture = fixture

    def rows(self) -> list[CoffeeRow]:
        if self.dataset_url.startswith("urn:"):
            return self._parse_csv(self.fixture.read_text(encoding="utf-8"))
        with urllib.request.urlopen(self.dataset_url, timeout=30) as response:  # nosec B310
            payload = json.loads(response.read().decode("utf-8"))
        return self._rows_from_payload(payload)

    def _rows_from_payload(self, payload: dict[str, Any]) -> list[CoffeeRow]:
        data = payload.get("data", payload)
        version = data.get("latestVersion", data)
        files = version.get("files", [])
        csv_file = next(
            (entry.get("dataFile", entry) for entry in files if str(entry.get("dataFile", entry).get("filename", entry.get("label", ""))).lower().endswith(".csv")),
            None,
        )
        if not csv_file:
            raise ValueError("Dataverse dataset has no CSV data file")
        file_id = csv_file.get("id")
        url = csv_file.get("downloadUrl") or f"https://dataverse.harvard.edu/api/access/datafile/{file_id}"
        with urllib.request.urlopen(url, timeout=60) as response:  # nosec B310
            return self._parse_csv(response.read().decode("utf-8"))

    @staticmethod
    def _parse_csv(raw: str) -> list[CoffeeRow]:
        rows = []
        for row in csv.DictReader(io.StringIO(raw)):
            if row.get("price_usd_per_kg", "") == "":
                row["price_usd_per_kg"] = None
            rows.append(CoffeeRow.model_validate(row))
        return rows
