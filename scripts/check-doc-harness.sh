#!/usr/bin/env bash
set -euo pipefail

MODE="warn"
if [[ $# -gt 0 ]]; then
  case "$1" in
    --mode)
      if [[ $# -lt 2 ]]; then
        echo "usage: scripts/check-doc-harness.sh --mode <warn|strict>"
        exit 2
      fi
      MODE="$2"
      shift 2
      ;;
    --mode=*)
      MODE="${1#--mode=}"
      shift
      ;;
    *)
      echo "usage: scripts/check-doc-harness.sh --mode <warn|strict>"
      exit 2
      ;;
  esac
fi

if [[ "$MODE" != "warn" && "$MODE" != "strict" ]]; then
  echo "invalid mode: $MODE (expected warn or strict)"
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

violations=0

report_violation() {
  local message="$1"
  if [[ "$MODE" == "strict" ]]; then
    echo "FAIL: $message"
  else
    echo "WARN: $message"
  fi
  violations=$((violations + 1))
}

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    report_violation "missing required file: $path"
  fi
}

check_required_agents() {
  require_file "AGENTS.md"
  require_file "web/AGENTS.md"
  require_file "crates/petri-core/AGENTS.md"
  require_file "crates/petri-graph/AGENTS.md"
  require_file "crates/petri-server/AGENTS.md"
  require_file "crates/petri-cli/AGENTS.md"
}

check_claude_adapter() {
  if [[ ! -f "CLAUDE.md" ]]; then
    report_violation "missing required file: CLAUDE.md"
    return
  fi

  local required_pointer='Read `AGENTS.md` first.'
  local required_minimal="This file is intentionally minimal."

  if ! grep -Fq "$required_pointer" "CLAUDE.md"; then
    report_violation "CLAUDE.md must include required pointer text: $required_pointer"
  fi

  if ! grep -Fq "$required_minimal" "CLAUDE.md"; then
    report_violation "CLAUDE.md must include required minimal-adapter text"
  fi

  local line_count
  line_count="$(wc -l < "CLAUDE.md" | tr -d ' ')"
  local max_lines=40
  if (( line_count > max_lines )); then
    report_violation "CLAUDE.md should stay minimal (<= ${max_lines} lines, found ${line_count})"
  fi
}

check_root_stub() {
  local path="$1"
  local target="$2"

  if [[ ! -f "$path" ]]; then
    report_violation "missing root compatibility stub: $path"
    return
  fi

  if ! grep -Fq "Canonical document: \`$target\`." "$path"; then
    report_violation "$path must declare canonical document path: $target"
  fi

  if ! grep -Fq "$target" "$path"; then
    report_violation "$path must link to canonical document: $target"
  fi

  local line_count
  line_count="$(wc -l < "$path" | tr -d ' ')"
  if (( line_count < 5 || line_count > 15 )); then
    report_violation "$path should remain a short compatibility stub (5-15 lines; found $line_count)"
  fi
}

check_root_strategy_stubs() {
  check_root_stub "petri-roadmap.md" "docs/strategy/roadmap.md"
  check_root_stub "petri-architecture.md" "docs/strategy/architecture.md"
  check_root_stub "petri-technology-review.md" "docs/strategy/technology-review.md"
}

check_root_agents_shape() {
  if [[ ! -f "AGENTS.md" ]]; then
    report_violation "missing root AGENTS.md"
    return
  fi

  local required_headings=(
    "## Mission"
    "## Repository Map"
    "## Non-Negotiable Invariants"
    "## Completion Gate"
    "## Doc Touch Policy"
  )

  local heading
  for heading in "${required_headings[@]}"; do
    if ! grep -Fq "$heading" "AGENTS.md"; then
      report_violation "AGENTS.md missing required heading: $heading"
    fi
  done

  local required_refs=(
    "web/AGENTS.md"
    "crates/petri-core/AGENTS.md"
    "crates/petri-graph/AGENTS.md"
    "crates/petri-server/AGENTS.md"
    "crates/petri-cli/AGENTS.md"
  )

  local ref
  for ref in "${required_refs[@]}"; do
    if ! grep -Fq "$ref" "AGENTS.md"; then
      report_violation "AGENTS.md must reference local instruction file: $ref"
    fi
  done
}

is_ignored_link() {
  local link="$1"
  [[ -z "$link" ]] && return 0
  [[ "$link" == \#* ]] && return 0
  [[ "$link" == http://* ]] && return 0
  [[ "$link" == https://* ]] && return 0
  [[ "$link" == mailto:* ]] && return 0
  [[ "$link" == javascript:* ]] && return 0
  return 1
}

normalize_link_target() {
  local raw="$1"

  raw="${raw#<}"
  raw="${raw%>}"
  raw="${raw%%\"*}"
  raw="${raw%%\'*}"
  raw="${raw%% *}"

  printf '%s' "$raw"
}

check_markdown_links() {
  while IFS= read -r -d '' md_file; do
    local md_dir
    md_dir="$(dirname "$md_file")"

    while IFS= read -r raw_link; do
      local link
      link="$(normalize_link_target "$raw_link")"

      if is_ignored_link "$link"; then
        continue
      fi

      local target_no_anchor
      target_no_anchor="${link%%#*}"
      if [[ -z "$target_no_anchor" ]]; then
        continue
      fi

      local target_path
      if [[ "$target_no_anchor" == /* ]]; then
        target_path="$target_no_anchor"
      else
        target_path="$md_dir/$target_no_anchor"
      fi

      if [[ ! -e "$target_path" ]]; then
        report_violation "broken markdown link in ${md_file#./}: ${link}"
      fi
    done < <(grep -oE '\[[^][]+\]\(([^)]+)\)' "$md_file" | sed -E 's/^[^\(]*\(([^)]+)\)$/\1/' || true)
  done < <(find . -type f -name '*.md' \
    -not -path './.git/*' \
    -not -path './.worktrees/*' \
    -not -path './target/*' \
    -not -path './node_modules/*' \
    -not -path './web/node_modules/*' \
    -print0)
}

check_required_agents
check_claude_adapter
check_root_strategy_stubs
check_root_agents_shape
check_markdown_links

echo ""
echo "Doc harness summary: mode=$MODE violations=$violations"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
