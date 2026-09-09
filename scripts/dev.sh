#!/bin/sh
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_PORT="${BACKEND_PORT:-3000}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"
FRONTEND_API_URL="${FRONTEND_API_URL:-}"
FRONTEND_PROXY_TARGET="${FRONTEND_PROXY_TARGET:-http://localhost:${BACKEND_PORT}}"
MODE="${MODE:-dev}"

if [ "${MODE}" = "prod" ]; then
  MODE="release"
fi

if [ "${MODE}" != "dev" ] && [ "${MODE}" != "release" ]; then
  echo "Unsupported MODE='${MODE}'. Use MODE=dev or MODE=release."
  exit 1
fi

if [ ! -x "${ROOT_DIR}/frontend/node_modules/.bin/vite" ]; then
  echo "Frontend dependencies are missing."
  echo "Run make setup to provision Petri's pinned frontend toolchain and dependencies."
  exit 1
fi

backend_pid=""
frontend_pid=""

# shellcheck disable=SC2329 # Registered with trap below.
cleanup() {
  for pid in "${backend_pid}" "${frontend_pid}"; do
    if [ -n "${pid}" ] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
    fi
  done
  # Reap the children before returning so the ports are free for the next run.
  for pid in "${backend_pid}" "${frontend_pid}"; do
    if [ -n "${pid}" ]; then
      wait "${pid}" 2>/dev/null || true
    fi
  done
}

# Exit on a signal instead of resuming the watch loop below: cleanup has
# already reaped both children, so a resumed `wait` would report a bogus
# status. The EXIT trap then runs cleanup once more, harmlessly.
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Every background process must be the real server or bundler rather than a
# launcher that spawns it: a launcher dies on shutdown and leaves the process
# holding the port orphaned. So build first and start the binary directly, and
# reach vite without going through npm.
if [ "${MODE}" = "release" ]; then
  cargo_profile_args="--release"
  target_profile="release"
else
  cargo_profile_args=""
  target_profile="debug"
fi

echo "Building backend ..."
# shellcheck disable=SC2086 # Deliberate word splitting of the profile flag.
(cd "${ROOT_DIR}" && cargo build ${cargo_profile_args} -p v3-server)

echo "Starting backend on http://localhost:${BACKEND_PORT} ..."
V3_SERVER_BIND_ADDR="0.0.0.0:${BACKEND_PORT}" \
  "${ROOT_DIR}/target/${target_profile}/v3-server" &
backend_pid=$!

vite="${ROOT_DIR}/frontend/node_modules/.bin/vite"
if [ "${MODE}" = "release" ]; then
  echo "Building frontend ..."
  (cd "${ROOT_DIR}/frontend" && VITE_API_URL="${FRONTEND_API_URL}" npm run build)
  vite_command="preview"
else
  vite_command="dev"
fi

echo "Starting frontend on http://localhost:${FRONTEND_PORT} ..."
(
  cd "${ROOT_DIR}/frontend"
  VITE_PROXY_TARGET="${FRONTEND_PROXY_TARGET}" VITE_API_URL="${FRONTEND_API_URL}" \
    exec "${vite}" "${vite_command}" --port "${FRONTEND_PORT}"
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
