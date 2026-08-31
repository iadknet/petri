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

BASELINE_FILE="docs/standards/architecture-baseline.tsv"
CURRENT_DEBT_FILE="$(mktemp)"
SORTED_BASELINE_FILE="$(mktemp)"
SORTED_CURRENT_FILE="$(mktemp)"
TAB=$'\t'

cleanup() {
  rm -f "$CURRENT_DEBT_FILE" "$SORTED_BASELINE_FILE" "$SORTED_CURRENT_FILE"
}
trap cleanup EXIT

violations=0
warnings=0
baseline_oversize_count=0
current_oversize_count=0

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

fingerprint_declaration() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    return 1
  fi
}

is_allowed_workspace_dep() {
  local from_crate="$1"
  local to_crate="$2"

  case "$from_crate:$to_crate" in
    v3-server:v3-core) return 0 ;;
    v3-cli:v3-core) return 0 ;;
  esac

  case "$from_crate" in
    v3-core|v3-server|v3-cli) return 1 ;;
    *) return 0 ;;
  esac
}

check_workspace_dependency_direction() {
  local manifest
  for manifest in crates/*/Cargo.toml; do
    [[ -f "$manifest" ]] || continue

    local from_crate
    from_crate="$(basename "$(dirname "$manifest")")"

    while IFS= read -r dep_name; do
      [[ -z "$dep_name" ]] && continue
      [[ "$dep_name" != v3-* ]] && continue

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
  local manifest="crates/v3-core/Cargo.toml"
  [[ -f "$manifest" ]] || return

  local dep
  for dep in axum tokio tower-http hyper; do
    if grep -Eq "^[[:space:]]*${dep}([[:space:]]*=|[[:space:]]*\.)" "$manifest"; then
      report_violation "forbidden runtime dependency '${dep}' found in ${manifest}"
    fi
  done
}

check_forbidden_runtime_imports_source() {
  while IFS= read -r hit; do
    [[ -z "$hit" ]] && continue
    report_violation "forbidden runtime import in pure crate source: ${hit}"
  done < <(
    find crates/v3-core/src -type f -name '*.rs' -exec \
      grep -En '(^|[^[:alnum:]_])(axum|tokio|hyper|tower_http)::' {} + 2>/dev/null || true
  )
}

check_lib_rs_export_focus() {
  if ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
    report_violation "missing SHA-256 tool for lib.rs declaration fingerprints (need shasum or sha256sum)"
    return
  fi

  local lib_file
  for lib_file in crates/*/src/lib.rs; do
    [[ -f "$lib_file" ]] || continue

    while IFS= read -r hit; do
      [[ -z "$hit" ]] && continue
      report_violation "lib.rs must not contain test modules/attributes: ${lib_file}:${hit}"
    done < <(
      grep -En '^[[:space:]]*#\[cfg\(test\)\]|^[[:space:]]*mod[[:space:]]+tests([[:space:]]|$)' \
        "$lib_file" || true
    )

    while IFS= read -r hit; do
      [[ -z "$hit" ]] && continue
      local declaration
      declaration="$(printf '%s' "$hit" \
        | sed -E 's/^[0-9]+://' \
        | tr -s '[:space:]' ' ' \
        | sed -E 's/^ //; s/ $//')"
      local fingerprint
      if ! fingerprint="$(printf '%s' "$declaration" | fingerprint_declaration)"; then
        report_violation "failed to fingerprint lib.rs implementation declaration: $lib_file"
        continue
      fi
      printf 'declaration\t%s\t%s\n' "$lib_file" "$fingerprint" >> "$CURRENT_DEBT_FILE"
    done < <(
      grep -En '^[[:space:]]*(pub[[:space:]]+)?(fn|struct|enum)([[:space:]]|$)|^[[:space:]]*impl([[:space:]]|<|$)' \
        "$lib_file" || true
    )
  done
}

check_production_file_sizes() {
  local source_file
  while IFS= read -r source_file; do
    local base_name
    base_name="$(basename "$source_file")"

    if [[ "$source_file" == */tests/* \
      || "$base_name" == "test.rs" \
      || "$base_name" == "tests.rs" \
      || "$base_name" == test_*.rs \
      || "$base_name" == *_test.rs \
      || "$base_name" == *_tests.rs ]]; then
      continue
    fi

    local line_count
    line_count="$(wc -l < "$source_file" | tr -d ' ')"

    if (( line_count > 400 )); then
      printf 'oversize\t%s\t%s\n' "$source_file" "$line_count" >> "$CURRENT_DEBT_FILE"
    fi
  done < <(find crates -type f -path '*/src/*' -name '*.rs' | sort)
}

check_architecture_debt_baseline() {
  if [[ ! -f "$BASELINE_FILE" ]]; then
    report_violation "missing architecture debt baseline: $BASELINE_FILE"
    return
  fi

  if ! awk -F "$TAB" '
    NF != 3 ||
    ($1 != "declaration" && $1 != "oversize") ||
    $2 == "" ||
    $3 == "" ||
    $2 ~ /[[:space:]]/ ||
    $3 ~ /[[:space:]]/ {
      exit 1
    }
  ' "$BASELINE_FILE"; then
    report_violation "invalid architecture debt baseline row in $BASELINE_FILE"
    return
  fi

  LC_ALL=C sort "$BASELINE_FILE" > "$SORTED_BASELINE_FILE"
  LC_ALL=C sort "$CURRENT_DEBT_FILE" > "$SORTED_CURRENT_FILE"
  baseline_oversize_count="$(awk -F "$TAB" '$1 == "oversize" { count++ } END { print count + 0 }' "$SORTED_BASELINE_FILE")"
  current_oversize_count="$(awk -F "$TAB" '$1 == "oversize" { count++ } END { print count + 0 }' "$SORTED_CURRENT_FILE")"

  local duplicate_oversize_paths
  duplicate_oversize_paths="$(awk -F "$TAB" '$1 == "oversize" && ++seen[$2] == 2 { print $2 }' "$SORTED_BASELINE_FILE")"
  if [[ -n "$duplicate_oversize_paths" ]]; then
    local duplicate_oversize_path
    while IFS= read -r duplicate_oversize_path; do
      [[ -z "$duplicate_oversize_path" ]] && continue
      report_violation "duplicate oversized production baseline entry: $duplicate_oversize_path"
    done <<< "$duplicate_oversize_paths"
    return
  fi

  local baseline_declarations
  local current_declarations
  baseline_declarations="$(grep -F "declaration${TAB}" "$SORTED_BASELINE_FILE" || true)"
  current_declarations="$(grep -F "declaration${TAB}" "$SORTED_CURRENT_FILE" || true)"
  if [[ "$baseline_declarations" != "$current_declarations" ]]; then
    report_violation "lib.rs declaration fingerprint multiset differs from $BASELINE_FILE"
  fi

  while IFS="$TAB" read -r kind path fingerprint; do
    [[ "$kind" == "declaration" ]] || continue
    if grep -Fxq "$kind"$'\t'"$path"$'\t'"$fingerprint" "$SORTED_CURRENT_FILE"; then
      report_warning "known lib.rs implementation declaration: $path ($fingerprint)"
    fi
  done < "$SORTED_BASELINE_FILE"

  while IFS="$TAB" read -r kind path max_lines; do
    [[ "$kind" == "oversize" ]] || continue
    local current_lines
    current_lines="$(awk -F "$TAB" -v path="$path" \
      '$1 == "oversize" && $2 == path { print $3 }' "$SORTED_CURRENT_FILE")"
    if [[ -z "$current_lines" ]]; then
      report_violation "stale architecture baseline entry: oversize $path $max_lines"
    elif [[ "$current_lines" != "$max_lines" ]]; then
      report_violation "oversize baseline mismatch for $path: expected $max_lines lines, found $current_lines"
    else
      report_warning "ratcheted known oversized production file: $path ($current_lines lines)"
    fi
  done < "$SORTED_BASELINE_FILE"

  while IFS="$TAB" read -r kind path current_lines; do
    [[ "$kind" == "oversize" ]] || continue
    if ! awk -F "$TAB" -v path="$path" \
      '$1 == "oversize" && $2 == path { found = 1 } END { exit !found }' \
      "$SORTED_BASELINE_FILE"; then
      report_violation "new oversized production file: $path ($current_lines lines)"
    fi
  done < "$SORTED_CURRENT_FILE"
}

check_cargo_policy() {
  local output
  if output="$(python3 scripts/check_cargo_policy.py 2>&1)"; then
    return
  fi

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    report_violation "Cargo policy: ${line}"
  done <<< "$output"
}

check_cargo_policy
check_workspace_dependency_direction
check_forbidden_runtime_deps_manifest
check_forbidden_runtime_imports_source
check_lib_rs_export_focus
check_production_file_sizes
check_architecture_debt_baseline

echo ""
echo "Architecture harness summary: mode=$MODE violations=$violations warnings=$warnings ratcheted_debt_baseline_oversize=$baseline_oversize_count current_oversize=$current_oversize_count (no-growth; reductions require baseline update)"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
