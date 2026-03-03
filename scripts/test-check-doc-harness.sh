#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p "$TMP_DIR/scripts" "$TMP_DIR/docs/strategy"
cp "$ROOT_DIR/scripts/check-doc-harness.sh" "$TMP_DIR/scripts/check-doc-harness.sh"

cat > "$TMP_DIR/AGENTS.md" <<'EOF'
# AGENTS.md

## Mission
Keep repo rules canonical here.

## Repository Map
- `docs/`

## Non-Negotiable Invariants
- Keep docs aligned.

## Completion Gate
- Run harnesses.

## Doc Touch Policy
- Keep stubs minimal.
EOF

printf '@AGENTS.md\n' > "$TMP_DIR/CLAUDE.md"

cat > "$TMP_DIR/petri-roadmap.md" <<'EOF'
# Petri Roadmap
Canonical document: `docs/strategy/roadmap.md`.
See [docs/strategy/roadmap.md](docs/strategy/roadmap.md).
Compatibility stub only.
Keep edits in canonical docs.
EOF

cat > "$TMP_DIR/petri-architecture.md" <<'EOF'
# Petri Architecture
Canonical document: `docs/strategy/architecture.md`.
See [docs/strategy/architecture.md](docs/strategy/architecture.md).
Compatibility stub only.
Keep edits in canonical docs.
EOF

cat > "$TMP_DIR/petri-technology-review.md" <<'EOF'
# Petri Technology Review
Canonical document: `docs/strategy/roadmap.md`.
See [docs/strategy/roadmap.md](docs/strategy/roadmap.md).
Compatibility stub only.
Keep edits in canonical docs.
EOF

cat > "$TMP_DIR/docs/strategy/roadmap.md" <<'EOF'
# Roadmap

Short roadmap.
EOF

cat > "$TMP_DIR/docs/strategy/architecture.md" <<'EOF'
# Architecture

Short architecture.
EOF

if ! (cd "$TMP_DIR" && bash scripts/check-doc-harness.sh --mode strict > "$TMP_DIR/output.txt" 2>&1); then
  cat "$TMP_DIR/output.txt"
  exit 1
fi
