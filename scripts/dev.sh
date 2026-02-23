#!/usr/bin/env zsh
set -euo pipefail

# Sandbox environments can reject zsh's default background niceness change.
unsetopt BG_NICE

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

if [[ ! -x "${ROOT_DIR}/frontend/node_modules/.bin/vite" ]]; then
  echo "Frontend dependencies are missing."
  echo "Run: cd \"${ROOT_DIR}/frontend\" && npm install"
  exit 1
fi

backend_pid=""
frontend_pid=""

cleanup() {
  if [[ -n "${backend_pid}" ]] && kill -0 "${backend_pid}" 2>/dev/null; then
    kill "${backend_pid}" 2>/dev/null || true
  fi
  if [[ -n "${frontend_pid}" ]] && kill -0 "${frontend_pid}" 2>/dev/null; then
    kill "${frontend_pid}" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

echo "Starting backend on http://localhost:3000 ..."
(
  cd "${ROOT_DIR}/v3"
  cargo run -p v3-server
) &
backend_pid=$!

echo "Starting frontend on http://localhost:5173 ..."
(
  cd "${ROOT_DIR}/frontend"
  npm run dev
) &
frontend_pid=$!

exit_status=0
while true; do
  if ! kill -0 "${backend_pid}" 2>/dev/null; then
    wait "${backend_pid}" || exit_status=$?
    break
  fi
  if ! kill -0 "${frontend_pid}" 2>/dev/null; then
    wait "${frontend_pid}" || exit_status=$?
    break
  fi
  sleep 1
done

exit "${exit_status}"
