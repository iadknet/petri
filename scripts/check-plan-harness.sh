#!/usr/bin/env bash
set -euo pipefail

MODE="warn"
if [[ $# -gt 0 ]]; then
  case "$1" in
    --mode)
      if [[ $# -lt 2 ]]; then
        echo "usage: scripts/check-plan-harness.sh --mode <warn|strict>"
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
      echo "usage: scripts/check-plan-harness.sh --mode <warn|strict>"
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

GOALS_FILE="docs/strategy/goals.md"
violations=0
warnings=0
files_checked=0

GOAL_IDS_TMP=""
FILES_TMP=""

cleanup() {
  [[ -n "$GOAL_IDS_TMP" && -f "$GOAL_IDS_TMP" ]] && rm -f "$GOAL_IDS_TMP"
  [[ -n "$FILES_TMP" && -f "$FILES_TMP" ]] && rm -f "$FILES_TMP"
}
trap cleanup EXIT

report_violation() {
  local message="$1"
  if [[ "$MODE" == "strict" ]]; then
    echo "FAIL: $message"
  else
    echo "WARN: $message"
  fi
  violations=$((violations + 1))
}

report_warning() {
  local message="$1"
  echo "WARN: $message"
  warnings=$((warnings + 1))
}

extract_section() {
  local file="$1"
  local heading="$2"
  awk -v heading="$heading" '
    $0 == heading { in_section=1; next }
    in_section && /^## / { exit }
    in_section { print }
  ' "$file"
}

has_unresolved_marker() {
  local text="$1"
  printf '%s\n' "$text" | grep -Eiq '(^|[^A-Za-z])(TBD|TODO)([^A-Za-z]|$)|\?\?\?'
}

goal_id_exists() {
  local goal_id="$1"
  grep -Fxq "$goal_id" "$GOAL_IDS_TMP"
}

parse_goals_catalog() {
  if [[ ! -f "$GOALS_FILE" ]]; then
    report_violation "missing goals catalog: $GOALS_FILE"
    return
  fi

  GOAL_IDS_TMP="$(mktemp)"
  sed -nE 's/^### ((GP|GO)-[0-9]{2}).*/\1/p' "$GOALS_FILE" | sort -u > "$GOAL_IDS_TMP"

  if [[ ! -s "$GOAL_IDS_TMP" ]]; then
    report_violation "no goal IDs parsed from $GOALS_FILE (expected headings like '### GP-01 ...')"
  fi
}

check_required_metadata_line() {
  local file="$1"
  local label="$2"
  local pattern="$3"
  if ! grep -Eq "$pattern" "$file"; then
    report_violation "$file missing required metadata line: $label"
    return 1
  fi
  return 0
}

check_required_section() {
  local file="$1"
  local section="$2"
  if ! grep -Fq "$section" "$file"; then
    report_violation "$file missing required section: $section"
    return 1
  fi
  return 0
}

parse_goal_ids_from_file() {
  local file="$1"
  grep -E '^\*\*Goal IDs:\*\*' "$file" | grep -Eo '(GP|GO)-[0-9]{2}' | sort -u || true
}

check_goal_ids_and_alignment() {
  local file="$1"

  local goal_ids
  goal_ids="$(parse_goal_ids_from_file "$file")"
  if [[ -z "$goal_ids" ]]; then
    report_violation "$file must declare at least one goal ID in '**Goal IDs:**'"
    return
  fi

  while IFS= read -r goal_id; do
    [[ -z "$goal_id" ]] && continue
    if ! goal_id_exists "$goal_id"; then
      report_violation "$file references unknown goal ID: $goal_id"
    fi
  done <<< "$goal_ids"

  local goal_alignment
  goal_alignment="$(extract_section "$file" "## Goal Alignment")"
  if [[ -z "$goal_alignment" ]]; then
    report_violation "$file missing Goal Alignment section content"
    return
  fi

  while IFS= read -r goal_id; do
    [[ -z "$goal_id" ]] && continue
    if ! printf '%s\n' "$goal_alignment" | grep -Fq "$goal_id"; then
      report_violation "$file Goal Alignment must reference declared goal ID: $goal_id"
    fi
  done <<< "$goal_ids"

  if has_unresolved_marker "$goal_alignment"; then
    if [[ "$MODE" == "strict" ]]; then
      report_violation "$file Goal Alignment contains unresolved marker (TBD/TODO/???)"
    else
      report_warning "$file Goal Alignment contains unresolved marker (TBD/TODO/???)"
    fi
  fi
}

check_existing_boundary_recheck() {
  local file="$1"
  local section
  section="$(extract_section "$file" "## Existing Boundary Recheck")"
  if [[ -z "$section" ]]; then
    report_violation "$file missing Existing Boundary Recheck section content"
    return
  fi

  if ! printf '%s\n' "$section" | grep -Eiq '\|\s*area\s*\|\s*decision\s*\|\s*rationale\s*\|'; then
    report_violation "$file Existing Boundary Recheck must include table header: | area | decision | rationale |"
    return
  fi

  local rows
  rows="$(printf '%s\n' "$section" | awk -F'|' '
    function trim(s){gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s}
    /^\|/ {
      area=trim($2); decision=trim($3); rationale=trim($4)
      lower_area=tolower(area); lower_decision=tolower(decision); lower_rationale=tolower(rationale)
      if (lower_area=="area" && lower_decision=="decision" && lower_rationale=="rationale") next
      if (area ~ /^-+$/ && decision ~ /^-+$/ && rationale ~ /^-+$/) next
      if (area=="" && decision=="" && rationale=="") next
      print area "\t" decision "\t" rationale
    }
  ')"

  local reviewed_count=0
  while IFS=$'\t' read -r area decision rationale; do
    [[ -z "${area:-}" && -z "${decision:-}" && -z "${rationale:-}" ]] && continue
    reviewed_count=$((reviewed_count + 1))
    local decision_norm
    decision_norm="${decision//\`/}"
    decision_norm="$(printf '%s' "$decision_norm" | tr '[:upper:]' '[:lower:]')"
    if [[ "$decision_norm" != "keep" && "$decision_norm" != "change" ]]; then
      report_violation "$file Existing Boundary Recheck decision must be 'keep' or 'change' (area: $area)"
    fi
    if [[ -z "$rationale" ]]; then
      report_violation "$file Existing Boundary Recheck row missing rationale (area: $area)"
    fi
  done <<< "$rows"

  if (( reviewed_count < 2 )); then
    report_violation "$file Existing Boundary Recheck must review at least 2 existing areas (found $reviewed_count)"
  fi

  if has_unresolved_marker "$rows"; then
    if [[ "$MODE" == "strict" ]]; then
      report_violation "$file Existing Boundary Recheck contains unresolved marker (TBD/TODO/???)"
    else
      report_warning "$file Existing Boundary Recheck contains unresolved marker (TBD/TODO/???)"
    fi
  fi
}

check_open_questions() {
  local file="$1"
  local section
  section="$(extract_section "$file" "## Open Questions")"
  if [[ -z "$section" ]]; then
    report_violation "$file missing Open Questions section content"
    return
  fi

  if ! printf '%s\n' "$section" | grep -Eiq '\|\s*question\s*\|\s*decision\s*\|\s*owner\s*\|\s*status\s*\|'; then
    report_violation "$file Open Questions must include table header: | question | decision | owner | status |"
    return
  fi

  local rows
  rows="$(printf '%s\n' "$section" | awk -F'|' '
    function trim(s){gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s}
    /^\|/ {
      question=trim($2); decision=trim($3); owner=trim($4); status=trim($5)
      lq=tolower(question); ld=tolower(decision); lo=tolower(owner); ls=tolower(status)
      if (lq=="question" && ld=="decision" && lo=="owner" && ls=="status") next
      if (question ~ /^-+$/ && decision ~ /^-+$/ && owner ~ /^-+$/ && status ~ /^-+$/) next
      if (question=="" && decision=="" && owner=="" && status=="") next
      print question "\t" decision "\t" owner "\t" status
    }
  ')"

  local row_count=0
  while IFS=$'\t' read -r question decision owner status; do
    [[ -z "${question:-}" && -z "${decision:-}" && -z "${owner:-}" && -z "${status:-}" ]] && continue
    row_count=$((row_count + 1))

    if [[ -z "$decision" || -z "$owner" || -z "$status" ]]; then
      report_violation "$file Open Questions row must include decision, owner, and status"
    fi

    local status_lower
    status_lower="${status//\`/}"
    status_lower="$(printf '%s' "$status_lower" | tr '[:upper:]' '[:lower:]')"
    if [[ "$status_lower" != "resolved" ]]; then
      if [[ "$MODE" == "strict" ]]; then
        report_violation "$file Open Questions status must be resolved in strict mode (question: $question, status: $status)"
      else
        report_warning "$file Open Questions status not resolved (question: $question, status: $status)"
      fi
    fi

    local critical_fields
    critical_fields="${decision} ${status}"
    if has_unresolved_marker "$critical_fields"; then
      if [[ "$MODE" == "strict" ]]; then
        report_violation "$file Open Questions decision/status contains unresolved marker (question: $question)"
      else
        report_warning "$file Open Questions decision/status contains unresolved marker (question: $question)"
      fi
    fi
  done <<< "$rows"

  if (( row_count == 0 )); then
    report_violation "$file Open Questions must include at least one data row"
  fi
}

check_metadata_markers() {
  local file="$1"
  local line
  line="$(grep -E '^\*\*(Goal|Goal IDs|Scope|Docs Impact|Supersedes|Superseded-By):\*\*' "$file" || true)"
  if has_unresolved_marker "$line"; then
    if [[ "$MODE" == "strict" ]]; then
      report_violation "$file metadata contains unresolved marker (TBD/TODO/???)"
    else
      report_warning "$file metadata contains unresolved marker (TBD/TODO/???)"
    fi
  fi
}

check_plan_file_requirements() {
  local file="$1"
  check_required_metadata_line "$file" "**Goal:**" '^\*\*Goal:\*\*'
  check_required_metadata_line "$file" "**Goal IDs:**" '^\*\*Goal IDs:\*\*'
  check_required_metadata_line "$file" "**Scope:**" '^\*\*Scope:\*\*'
  check_required_metadata_line "$file" "**Docs Impact:**" '^\*\*Docs Impact:\*\*'
  check_required_metadata_line "$file" "**Supersedes:**" '^\*\*Supersedes:\*\*'
  check_required_metadata_line "$file" "**Superseded-By:**" '^\*\*Superseded-By:\*\*'
}

check_required_sections_all_targets() {
  local file="$1"
  check_required_section "$file" "## Goal Alignment"
  check_required_section "$file" "## Boundary Impact"
  check_required_section "$file" "## Existing Boundary Recheck"
  check_required_section "$file" "## Open Questions"
}

check_file() {
  local file="$1"
  files_checked=$((files_checked + 1))

  if [[ "$file" == docs/features/ready_to_implement/*/master_plan.md || "$file" == docs/features/in_progress/*/master_plan.md ]]; then
    check_plan_file_requirements "$file"
  fi

  check_required_sections_all_targets "$file"
  check_goal_ids_and_alignment "$file"
  check_existing_boundary_recheck "$file"
  check_open_questions "$file"
  check_metadata_markers "$file"
}

collect_target_files() {
  FILES_TMP="$(mktemp)"

  if [[ -f "docs/strategy/architecture.md" ]]; then
    echo "docs/strategy/architecture.md" >> "$FILES_TMP"
  else
    report_violation "missing required architecture doc target: docs/strategy/architecture.md"
  fi

  # Scan master_plan.md files in docs/features/ready_to_implement/ and docs/features/in_progress/
  for stage_dir in docs/features/ready_to_implement docs/features/in_progress; do
    if [[ -d "$stage_dir" ]]; then
      find "$stage_dir" -mindepth 2 -maxdepth 2 -type f -name 'master_plan.md' | sort >> "$FILES_TMP"
    fi
  done
}

parse_goals_catalog
collect_target_files

if [[ -f "$FILES_TMP" ]]; then
  while IFS= read -r target; do
    [[ -z "$target" ]] && continue
    if [[ ! -f "$target" ]]; then
      report_violation "target file not found: $target"
      continue
    fi
    check_file "$target"
  done < "$FILES_TMP"
fi

echo ""
echo "Plan harness summary: mode=$MODE violations=$violations warnings=$warnings files_checked=$files_checked"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
