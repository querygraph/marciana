import unittest

from metrics import CaseResult, percentile, summarize


class MetricsTests(unittest.TestCase):
    def test_summary_has_accuracy_latency_and_safety_gates(self) -> None:
        rows = [
            CaseResult("ok", ("a",), ("a",), (), 10.0, 4),
            CaseResult("redacted", (), (), ("secret",), 20.0, 0),
        ]
        report = summarize(rows)
        self.assertEqual(report["accuracy"], 1.0)
        self.assertEqual(report["redaction_leaks"], 0)
        self.assertEqual(report["p95_latency_us"], 20.0)

    def test_percentile_empty_is_safe(self) -> None:
        self.assertEqual(percentile([], 95), 0.0)


if __name__ == "__main__":
    unittest.main()
