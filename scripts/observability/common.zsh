#!/usr/bin/env zsh

set -euo pipefail

: "${API_BASE:=http://localhost:3000}"

CURL_BIN="$(whence -p curl || true)"
JQ_BIN="$(whence -p jq || true)"
AWK_BIN="$(whence -p awk || true)"
SORT_BIN="$(whence -p sort || true)"
HEAD_BIN="$(whence -p head || true)"
SLEEP_BIN="$(whence -p sleep || true)"
if [[ -z "${SLEEP_BIN}" && -x "/bin/sleep" ]]; then
	SLEEP_BIN="/bin/sleep"
fi

if [[ -z "${CURL_BIN}" ]]; then
	echo "Missing required command: curl" >&2
	exit 1
fi

if [[ -z "${JQ_BIN}" ]]; then
	echo "Missing required command: jq" >&2
	exit 1
fi

if [[ -z "${AWK_BIN}" || -z "${SORT_BIN}" || -z "${HEAD_BIN}" ]]; then
	echo "Missing required text-processing commands (awk, sort, head)" >&2
	exit 1
fi

fetch_json() {
	local path="${1:?path is required}"
	local attempts="${API_RETRIES:-5}"
	local delay_seconds="${API_RETRY_DELAY_SECONDS:-0.2}"
	local response=""
	local attempt=1

	while [[ "${attempt}" -le "${attempts}" ]]; do
		if response="$("${CURL_BIN}" -sS "${API_BASE}${path}")"; then
			echo "${response}"
			return 0
		fi
		if [[ "${attempt}" -ge "${attempts}" ]]; then
			return 1
		fi
		if [[ -n "${SLEEP_BIN}" ]]; then
			"${SLEEP_BIN}" "${delay_seconds}"
		fi
		attempt=$((attempt + 1))
	done
}

sample_creature_ids() {
	local sample_size="${1:?sample size is required}"
	local snapshot_json="${2:?snapshot json is required}"

	echo "${snapshot_json}" \
		| "${AWK_BIN}" '
			{
				line = $0
				while (match(line, /"id"[[:space:]]*:[[:space:]]*[0-9]+/)) {
					token = substr(line, RSTART, RLENGTH)
					gsub(/[^0-9]/, "", token)
					if (length(token) > 0) {
						print token
					}
					line = substr(line, RSTART + RLENGTH)
				}
			}
		' \
		| "${AWK_BIN}" 'BEGIN { srand(); } { print rand(), $0; }' \
		| "${SORT_BIN}" -k1,1n \
		| "${AWK_BIN}" -v max="${sample_size}" 'NR <= max { print $2; }'
}
