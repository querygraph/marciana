#!/usr/bin/env bash
set -euo pipefail

# Run Marciana's ignored live tests against the exact Sail binary built by CI.
# Callers may provide SAIL_ENDPOINT when a server is already managed outside
# this script; otherwise the pinned binary is started for the duration of the
# test. No source checkout or PATH installation is accepted as the binary.

binary="${SAIL_TEST_BIN:-}"
if [[ -z "$binary" || ! -x "$binary" ]]; then
  echo "SAIL_TEST_BIN must name an executable Sail binary built from the pinned source" >&2
  exit 2
fi

endpoint="${SAIL_ENDPOINT:-}"
child=""
cleanup() {
  if [[ -n "$child" ]]; then
    kill "$child" 2>/dev/null || true
    wait "$child" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

if [[ -z "$endpoint" ]]; then
  port="${SAIL_PORT:-50051}"
  endpoint="http://${SAIL_HOST:-127.0.0.1}:${port}"
  "$binary" spark server --ip "${SAIL_HOST:-127.0.0.1}" --port "$port" \
    >"${TMPDIR:-/tmp}/marciana-sail.log" 2>&1 &
  child=$!
  # Spark Connect has no stable cross-version health endpoint. Give the
  # server a bounded startup window; the Rust test reports connection errors.
  for _ in {1..30}; do
    if ! kill -0 "$child" 2>/dev/null; then
      echo "Sail exited before the live gate; log follows:" >&2
      sed -n '1,160p' "${TMPDIR:-/tmp}/marciana-sail.log" >&2 || true
      exit 1
    fi
    sleep 1
  done
fi

SAIL_ENDPOINT="$endpoint" cargo test -p querygraph-memory --features sail \
  --test sail_cognition_live -- --ignored
