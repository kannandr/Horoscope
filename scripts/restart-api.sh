#!/usr/bin/env bash
# Rebuild and restart the local panchang-api so the Next.js web/MCP UIs
# always talk to the latest /v1/panchang/day route.
#
# Usage:  scripts/restart-api.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/rust"

if ! command -v cargo >/dev/null 2>&1; then
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
fi

PORT="${PANCHANG_API_PORT:-8080}"

echo "[restart-api] killing any process listening on :$PORT"
PIDS="$(lsof -nP -iTCP:"$PORT" -sTCP:LISTEN -t 2>/dev/null || true)"
if [[ -n "$PIDS" ]]; then
  # Kill the listener and its parents (e.g. cargo run wrapper), then wait briefly.
  for PID in $PIDS; do
    PARENT="$(ps -p "$PID" -o ppid= 2>/dev/null | tr -d ' ' || true)"
    [[ -n "$PARENT" && "$PARENT" != "1" ]] && kill "$PARENT" 2>/dev/null || true
    kill "$PID" 2>/dev/null || true
  done
  sleep 1
fi

echo "[restart-api] building panchang-api (release)"
cargo build -p panchang-api --release

BIN="$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['target_directory'])")/release/panchang-api"

if [[ ! -x "$BIN" ]]; then
  echo "[restart-api] ERROR: built binary not found at $BIN" >&2
  exit 1
fi

LOG="${PANCHANG_API_LOG:-/tmp/panchang-api.log}"
echo "[restart-api] starting $BIN (logs: $LOG)"
nohup "$BIN" > "$LOG" 2>&1 &
NEW_PID=$!
disown "$NEW_PID" 2>/dev/null || true

# Wait for /healthz so callers know it's actually ready.
for i in 1 2 3 4 5 6 7 8 9 10; do
  sleep 0.5
  if curl -fsS "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1; then
    echo "[restart-api] ready on http://127.0.0.1:$PORT (pid $NEW_PID)"
    exit 0
  fi
done

echo "[restart-api] ERROR: API did not become ready in 5s; tail $LOG" >&2
tail -n 40 "$LOG" >&2 || true
exit 1
