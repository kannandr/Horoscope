#!/usr/bin/env bash
# Rebuild and restart a local Rust HTTP service so the Next.js UI always
# talks to the latest binary. Defaults to panchang-api on :8080; pass
# muhurta-api / a different port via env or argument.
#
# Usage examples:
#   scripts/restart-api.sh                       # panchang-api on :8080
#   scripts/restart-api.sh muhurta-api           # muhurta-api on :8090
#   PORT=9000 scripts/restart-api.sh panchang-api
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT/rust"

CRATE="${1:-${RESTART_CRATE:-panchang-api}}"
case "$CRATE" in
  panchang-api) DEFAULT_PORT=8080 ;;
  muhurta-api)  DEFAULT_PORT=8090 ;;
  horoscope-mcp) DEFAULT_PORT=8790 ;;
  *) echo "[restart-api] unknown crate: $CRATE (expected panchang-api, muhurta-api, or horoscope-mcp)" >&2; exit 2 ;;
esac
PORT="${PORT:-${PANCHANG_API_PORT:-$DEFAULT_PORT}}"

if ! command -v cargo >/dev/null 2>&1; then
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi
fi

echo "[restart-api] killing any process listening on :$PORT"
PIDS="$(lsof -nP -iTCP:"$PORT" -sTCP:LISTEN -t 2>/dev/null || true)"
if [[ -n "$PIDS" ]]; then
  for PID in $PIDS; do
    PARENT="$(ps -p "$PID" -o ppid= 2>/dev/null | tr -d ' ' || true)"
    [[ -n "$PARENT" && "$PARENT" != "1" ]] && kill "$PARENT" 2>/dev/null || true
    kill "$PID" 2>/dev/null || true
  done
  sleep 1
fi

echo "[restart-api] building $CRATE (release)"
cargo build -p "$CRATE" --release

BIN="$(cargo metadata --format-version=1 --no-deps 2>/dev/null \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['target_directory'])")/release/$CRATE"

if [[ ! -x "$BIN" ]]; then
  echo "[restart-api] ERROR: built binary not found at $BIN" >&2
  exit 1
fi

LOG="${RESTART_LOG:-/tmp/$CRATE.log}"
echo "[restart-api] starting $BIN (logs: $LOG)"
BIND_ADDR="0.0.0.0:$PORT" nohup "$BIN" > "$LOG" 2>&1 &
NEW_PID=$!
disown "$NEW_PID" 2>/dev/null || true

for i in 1 2 3 4 5 6 7 8 9 10; do
  sleep 0.5
  if curl -fsS "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1; then
    echo "[restart-api] $CRATE ready on http://127.0.0.1:$PORT (pid $NEW_PID)"
    exit 0
  fi
done

echo "[restart-api] ERROR: $CRATE did not become ready in 5s; tail $LOG" >&2
tail -n 40 "$LOG" >&2 || true
exit 1
