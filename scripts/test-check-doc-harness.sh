#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

mkdir -p \
  "$TMP_DIR/scripts" \
  "$TMP_DIR/docs/strategy" \
  "$TMP_DIR/.claude/skills/rust-skills"
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

[Architecture (canonical)](architecture.md)

Short roadmap.
EOF

cat > "$TMP_DIR/docs/strategy/architecture.md" <<'EOF'
# Architecture

Short architecture.
EOF

cat > "$TMP_DIR/.claude/skills/rust-skills/vendor.md" <<'EOF'
# Vendored documentation

[Vendored relative link](missing-vendor.md)
EOF

cat > "$TMP_DIR/docs/code-example.md" <<'EOF'
# Code examples

[External protocol link](custom-scheme:resource)

[Nested destination](guide(1).md)

[Angle destination with title](<guide with space.md> "reference")

`[Inline example](missing-inline.md)`

``[Double-delimited example](missing-double.md) with `inner backtick` ``

Text: ```[Long-delimited example](missing-long.md) with ``inner pair`` ```

[First-party guide after code spans](guide(1).md)

```markdown
[Fenced example](missing-fenced.md)
```
EOF

cat > "$TMP_DIR/docs/guide(1).md" <<'EOF'
# Nested destination guide
EOF

cat > "$TMP_DIR/docs/guide with space.md" <<'EOF'
# Angle destination guide
EOF

if ! (cd "$TMP_DIR" && bash scripts/check-doc-harness.sh --mode strict > "$TMP_DIR/output.txt" 2>&1); then
  cat "$TMP_DIR/output.txt"
  exit 1
fi

cat > "$TMP_DIR/docs/project.md" <<'EOF'
# Project documentation

``[Ignored double-span link](ignored-double.md)`` [Missing first-party document](missing.md)

Text: ```[Ignored long-span link](ignored-long.md) with ``inner pair`` ``` [Another missing first-party document](missing-after-long.md)
EOF

if (cd "$TMP_DIR" && bash scripts/check-doc-harness.sh --mode strict > "$TMP_DIR/output.txt" 2>&1); then
  echo "expected strict doc harness to reject a first-party broken link"
  exit 1
fi

grep -q 'broken markdown link in docs/project.md: missing.md' "$TMP_DIR/output.txt"
grep -q 'broken markdown link in docs/project.md: missing-after-long.md' "$TMP_DIR/output.txt"
if grep -Eq 'ignored-double\.md|ignored-long\.md' "$TMP_DIR/output.txt"; then
  echo "markdown code spans must not be treated as links"
  exit 1
fi

echo "Doc harness tests passed"
