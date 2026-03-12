#!/usr/bin/env zsh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

: "${API_BASE:=http://localhost:3000}"

CURL_BIN="$(whence -p curl || true)"
JQ_BIN="$(whence -p jq || true)"
DATE_BIN="$(whence -p date || true)"

if [[ -z "${CURL_BIN}" ]]; then
	echo "Missing required command: curl" >&2
	exit 1
fi

if [[ -z "${JQ_BIN}" ]]; then
	echo "Missing required command: jq" >&2
	exit 1
fi

if [[ -z "${DATE_BIN}" ]]; then
	echo "Missing required command: date" >&2
	exit 1
fi

api_get_json() {
	local path="${1:?path is required}"
	"${CURL_BIN}" -sS "${API_BASE}${path}"
}

api_post_json() {
	local path="${1:?path is required}"
	local payload="${2:?payload is required}"
	"${CURL_BIN}" -sS \
		-X POST \
		-H "content-type: application/json" \
		--data "${payload}" \
		"${API_BASE}${path}"
}

timestamp_utc() {
	"${DATE_BIN}" -u +"%Y%m%dT%H%M%SZ"
}
