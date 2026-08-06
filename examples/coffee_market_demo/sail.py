"""Sail Spark Connect adapter used by the demo."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Any

from .models import CoffeeRow

SAFE_TABLE = re.compile(r"^[a-z][a-z0-9_]{0,62}$")


@dataclass
class SailWarehouse:
    remote_url: str | None
    table_name: str = "coffee_honduras_market"
    _spark: Any = None

    @property
    def connected(self) -> bool:
        return self._spark is not None

    def load(self, rows: list[CoffeeRow]) -> bool:
        self._check_table()
        if not self.remote_url:
            return False
        from pyspark.sql import SparkSession  # type: ignore[import-not-found]

        self._spark = SparkSession.builder.remote(self.remote_url).getOrCreate()
        self._spark.createDataFrame([row.model_dump() for row in rows]).write.mode("overwrite").saveAsTable(self.table_name)
        return True

    def query(self, sql: str) -> list[dict[str, Any]]:
        if not self._spark:
            raise RuntimeError("Sail is not connected; set SAIL_SPARK_CONNECT_URL")
        return [row.asDict(recursive=True) for row in self._spark.sql(sql).collect()]

    def _check_table(self) -> None:
        if not SAFE_TABLE.fullmatch(self.table_name):
            raise ValueError("unsafe Sail table name")
