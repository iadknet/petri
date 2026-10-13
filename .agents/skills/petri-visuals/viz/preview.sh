#!/bin/sh
# Prints a standalone HTML page that renders a spec (a JSON file) with
# petri-viz.min.js, for checking a visual in the browser pane before pasting it
# into show_widget. Usage: sh preview.sh spec.json > page.html
# Every < in the spec is written as \u003c so text cannot close the script.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
spec=$1
printf '%s' '<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>petri-viz preview</title><style>:root{--text-primary:#e8e6dc;--text-secondary:#b4b2a9;--text-accent:#9ecbff;--bg-accent:#1f3a5c;--border-accent:#3d6a99;--surface-1:#262624;--radius:8px}body{background:#1f1e1d;color:#e8e6dc;font-family:system-ui,sans-serif;max-width:680px;margin:16px auto;padding:0 16px}button{background:transparent;color:inherit;border:0.5px solid #5f5e5a;border-radius:8px;padding:6px 10px;cursor:pointer}</style></head><body><div id="pv"></div><script>'
cat "$dir/petri-viz.min.js"
printf '%s' '</script><script>PetriViz.mount("#pv",'
sed 's/</\\u003c/g' "$spec"
printf '%s\n' ');</script></body></html>'
