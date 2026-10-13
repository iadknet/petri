#!/bin/sh
# Writes petri-viz.min.js (the copy pasted into widgets) from petri-viz.js,
# using the frontend's esbuild. Prints the minified size in characters.
# The bundle is replaced only after esbuild succeeds.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$dir/../../../.." && pwd)
esbuild="$root/frontend/node_modules/.bin/esbuild"
if [ ! -x "$esbuild" ]; then
	echo "esbuild not found at $esbuild; run npm ci in frontend/" >&2
	exit 1
fi
tmp=$(mktemp "${TMPDIR:-/tmp}/petri-viz.XXXXXX")
trap 'rm -f "$tmp"' EXIT INT TERM
"$esbuild" "$dir/petri-viz.js" --minify --target=es2017 --log-level=warning >"$tmp"
chmod 644 "$tmp"
mv "$tmp" "$dir/petri-viz.min.js"
wc -c <"$dir/petri-viz.min.js" | tr -d ' '
