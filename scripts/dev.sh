#!/usr/bin/env zsh
set -euo pipefail

# Sandbox environments can reject zsh's default background niceness change.
unsetopt BG_NICE

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_PORT="${BACKEND_PORT:-3000}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"
FRONTEND_API_URL="${FRONTEND_API_URL:-}"
FRONTEND_PROXY_TARGET="${FRONTEND_PROXY_TARGET:-http://localhost:${BACKEND_PORT}}"
MODE="${MODE:-dev}"

if [[ "${MODE}" == "prod" ]]; then
  MODE="release"
fi

if [[ "${MODE}" != "dev" && "${MODE}" != "release" ]]; then
  echo "Unsupported MODE='${MODE}'. Use MODE=dev or MODE=release."
  exit 1
fi

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

echo "Starting backend on http://localhost:${BACKEND_PORT} ..."
(
  cd "${ROOT_DIR}/v3"
  if [[ "${MODE}" == "release" ]]; then
    V3_SERVER_BIND_ADDR="0.0.0.0:${BACKEND_PORT}" cargo run --release -p v3-server
  else
    V3_SERVER_BIND_ADDR="0.0.0.0:${BACKEND_PORT}" cargo run -p v3-server
  fi
) &
backend_pid=$!

echo "Starting frontend on http://localhost:${FRONTEND_PORT} ..."
(
  cd "${ROOT_DIR}/frontend"
  if [[ "${MODE}" == "release" ]]; then
    VITE_API_URL="${FRONTEND_API_URL}" npm run build
    VITE_PROXY_TARGET="${FRONTEND_PROXY_TARGET}" VITE_API_URL="${FRONTEND_API_URL}" npm run preview -- --port "${FRONTEND_PORT}"
  else
    VITE_PROXY_TARGET="${FRONTEND_PROXY_TARGET}" VITE_API_URL="${FRONTEND_API_URL}" npm run dev -- --port "${FRONTEND_PORT}"
  fi
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
