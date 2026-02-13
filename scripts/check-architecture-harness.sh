#!/usr/bin/env bash
set -euo pipefail

MODE="warn"
if [[ $# -gt 0 ]]; then
  case "$1" in
    --mode)
      if [[ $# -lt 2 ]]; then
        echo "usage: scripts/check-architecture-harness.sh --mode <warn|strict>"
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
      echo "usage: scripts/check-architecture-harness.sh --mode <warn|strict>"
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

BASELINE_FILE="docs/standards/architecture-size-baseline.tsv"
violations=0
warnings=0

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

is_allowed_workspace_dep() {
  local from_crate="$1"
  local to_crate="$2"

  case "$from_crate:$to_crate" in
    petri-core:petri-graph) return 0 ;;
    petri-server:petri-core) return 0 ;;
    petri-cli:petri-core) return 0 ;;
  esac

  case "$from_crate" in
    petri-core|petri-graph|petri-server|petri-cli) return 1 ;;
    *) return 0 ;;
  esac
}

check_workspace_dependency_direction() {
  local manifest
  for manifest in crates/*/Cargo.toml; do
    local from_crate
    from_crate="$(basename "$(dirname "$manifest")")"

    while IFS= read -r dep_name; do
      [[ -z "$dep_name" ]] && continue
      [[ "$dep_name" != petri-* ]] && continue

      if ! is_allowed_workspace_dep "$from_crate" "$dep_name"; then
        report_violation "disallowed workspace dependency: ${from_crate} -> ${dep_name} (${manifest})"
      fi
    done < <(
      grep -E '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=[[:space:]]*\{[^}]*path[[:space:]]*=[[:space:]]*"\.\./[^"[:space:]]+"' "$manifest" \
        | sed -E 's/^[[:space:]]*([A-Za-z0-9_-]+)[[:space:]]*=.*/\1/' \
        || true
    )
  done
}

check_forbidden_runtime_deps_manifest() {
  local crate
  for crate in petri-core petri-graph; do
    local manifest="crates/${crate}/Cargo.toml"
    [[ -f "$manifest" ]] || continue

    local dep
    for dep in axum tokio tower-http hyper; do
      if grep -Eq "^[[:space:]]*${dep}([[:space:]]*=|[[:space:]]*\.)" "$manifest"; then
        report_violation "forbidden runtime dependency '${dep}' found in ${manifest}"
      fi
    done
  done
}

check_forbidden_runtime_imports_source() {
  while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    report_violation "forbidden runtime import in pure crate source: ${hit}"
  done < <(
    rg -n --glob '*.rs' '\b(axum|tokio|hyper|tower_http)::' crates/petri-core/src crates/petri-graph/src 2>/dev/null || true
  )
}

check_lib_rs_export_focus() {
  local lib_file
  for lib_file in crates/*/src/lib.rs; do
    [[ -f "$lib_file" ]] || continue

    while IFS= read -r hit; do
      [[ -z "$hit" ]] && continue
      report_violation "lib.rs must not contain test modules/attributes: ${hit}"
    done < <(rg -n '^\s*#\[cfg\(test\)\]|^\s*mod\s+tests\b' "$lib_file" || true)

    while IFS= read -r hit; do
      [[ -z "$hit" ]] && continue
      report_violation "lib.rs must remain export-focused (implementation item found): ${hit}"
    done < <(rg -n '^\s*(pub\s+)?(fn|struct|enum)\b|^\s*impl\b' "$lib_file" || true)
  done
}

baseline_line_for_path() {
  local target_path="$1"
  awk -F '\t' -v target="$target_path" '
    $0 ~ /^[[:space:]]*#/ { next }
    $1 == "path" { next }
    NF >= 2 && $1 == target { print $2; exit }
  ' "$BASELINE_FILE"
}

check_baseline_entries_reference_existing_files() {
  if [[ ! -f "$BASELINE_FILE" ]]; then
    report_violation "missing baseline file: ${BASELINE_FILE}"
    return
  fi

  while IFS=$'\t' read -r path _rest; do
    [[ -z "$path" ]] && continue
    [[ "$path" == "path" ]] && continue
    [[ "$path" =~ ^# ]] && continue

    if [[ ! -f "$path" ]]; then
      report_warning "baseline entry references missing file: ${path}"
    fi
  done < "$BASELINE_FILE"
}

check_production_file_sizes() {
  if [[ ! -f "$BASELINE_FILE" ]]; then
    report_violation "missing baseline file: ${BASELINE_FILE}"
    return
  fi

  local source_file
  while IFS= read -r source_file; do
    local base_name
    base_name="$(basename "$source_file")"

    if [[ "$base_name" == "tests.rs" || "$base_name" == test*.rs ]]; then
      continue
    fi

    local line_count
    line_count="$(wc -l < "$source_file" | tr -d ' ')"

    local baseline_count
    baseline_count="$(baseline_line_for_path "$source_file")"

    if [[ -n "$baseline_count" ]]; then
      if (( line_count > baseline_count )); then
        report_warning "baseline allowlisted file grew: ${source_file} (${baseline_count} -> ${line_count} lines)"
      fi

      if (( line_count > 400 )); then
        report_warning "baseline allowlisted oversize production file: ${source_file} (${line_count} lines > 400)"
      fi
      continue
    fi

    if (( line_count > 600 )); then
      report_violation "production file exceeds 600 lines and is not baselined: ${source_file} (${line_count} lines)"
    elif (( line_count > 400 )); then
      report_warning "production file exceeds 400 lines: ${source_file} (${line_count} lines)"
    fi
  done < <(find crates -type f -path '*/src/*' -name '*.rs' | sort)
}

check_workspace_dependency_direction
check_forbidden_runtime_deps_manifest
check_forbidden_runtime_imports_source
check_lib_rs_export_focus
check_baseline_entries_reference_existing_files
check_production_file_sizes

echo ""
echo "Architecture harness summary: mode=$MODE violations=$violations warnings=$warnings"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
