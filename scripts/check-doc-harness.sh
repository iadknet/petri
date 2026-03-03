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
}

check_claude_adapter() {
  if [[ ! -f "CLAUDE.md" ]]; then
    report_violation "missing required file: CLAUDE.md"
    return
  fi

  local content
  content="$(tr -d '\r' < "CLAUDE.md")"
  if [[ "$content" != "@AGENTS.md" ]]; then
    report_violation "CLAUDE.md must contain only @AGENTS.md"
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
  check_root_stub "petri-technology-review.md" "docs/strategy/roadmap.md"
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

}

check_strategy_redundancy() {
  local strategy_files=()
  while IFS= read -r file; do
    strategy_files+=("$file")
  done < <(find docs/strategy -maxdepth 1 -type f -name '*.md' | sort)

  if (( ${#strategy_files[@]} < 2 )); then
    return
  fi

  local paragraphs_file
  paragraphs_file="$(mktemp)"

  local strategy_file
  for strategy_file in "${strategy_files[@]}"; do
    awk -v file="$strategy_file" '
      function flush_paragraph() {
        gsub(/[[:space:]]+/, " ", paragraph)
        sub(/^ /, "", paragraph)
        sub(/ $/, "", paragraph)
        normalized = tolower(paragraph)
        if (length(normalized) >= 220) {
          print normalized "\t" file
        }
        paragraph = ""
      }
      /^```/ {
        in_code = !in_code
        next
      }
      {
        if (in_code) next

        line = $0
        if (line == "" || line ~ /^#/ || line ~ /^- / || line ~ /^\|/ || line ~ /^```/) {
          flush_paragraph()
          next
        }

        paragraph = paragraph " " line
      }
      END {
        flush_paragraph()
      }
    ' "$strategy_file" >> "$paragraphs_file"
  done

  local duplicates_file
  duplicates_file="$(mktemp)"

  awk -F '\t' '
    {
      paragraph = $1
      file = $2
      count[paragraph]++
      if (index(files[paragraph], "|" file "|") == 0) {
        files[paragraph] = files[paragraph] file "|"
      }
    }
    END {
      for (paragraph in count) {
        n = split(files[paragraph], arr, /\|/)
        unique_files = 0
        file_list = ""
        for (i = 1; i <= n; i++) {
          if (arr[i] == "") continue
          unique_files++
          if (file_list == "") file_list = arr[i]
          else file_list = file_list ", " arr[i]
        }
        if (unique_files > 1) {
          snippet = substr(paragraph, 1, 120)
          print snippet "\t" file_list
        }
      }
    }
  ' "$paragraphs_file" > "$duplicates_file"

  while IFS=$'\t' read -r snippet file_list; do
    [[ -z "${snippet:-}" ]] && continue
    report_violation "repeated long documentation block across strategy docs (${file_list}). Prefer references over repetition. Snippet: ${snippet}..."
  done < "$duplicates_file"

  rm -f "$paragraphs_file" "$duplicates_file"
}

check_conflicting_dependency_direction_statements() {
  local matches_file
  matches_file="$(mktemp)"

  rg -n '(dependency direction|crate direction).*petri-.*->.*petri-.*->.*petri-' AGENTS.md docs 2>/dev/null \
    | while IFS= read -r line; do
        local chain
        chain="$(printf '%s' "$line" \
          | grep -Eo 'petri-[a-z-]+[[:space:]]*->[[:space:]]*petri-[a-z-]+[[:space:]]*->[[:space:]]*petri-[a-z-]+(/[[:space:]]*petri-[a-z-]+)?' \
          | sed -E 's/[[:space:]]*->[[:space:]]*/ -> /g; s/[[:space:]]*\/[[:space:]]*/\//g' \
          || true)"
        if [[ -n "$chain" ]]; then
          printf '%s\n' "$chain" >> "$matches_file"
        fi
      done || true

  if [[ -s "$matches_file" ]]; then
    local unique_chains
    unique_chains="$(sort -u "$matches_file")"
    local unique_count
    unique_count="$(printf '%s\n' "$unique_chains" | sed '/^$/d' | wc -l | tr -d ' ')"
    if (( unique_count > 1 )); then
      report_violation "conflicting crate dependency direction statements found: $(printf '%s' "$unique_chains" | paste -sd '; ' -)"
    fi
  fi

  rm -f "$matches_file"
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
    -print0)
}

check_required_agents
check_claude_adapter
check_root_strategy_stubs
check_root_agents_shape
check_strategy_redundancy
check_conflicting_dependency_direction_statements
check_markdown_links

echo ""
echo "Doc harness summary: mode=$MODE violations=$violations"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
