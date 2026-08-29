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
    rg -n --glob '*.rs' '\b(axum|tokio|hyper|tower_http)::' crates/v3-core/src 2>/dev/null || true
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

check_production_file_sizes() {
  local source_file
  while IFS= read -r source_file; do
    local base_name
    base_name="$(basename "$source_file")"

    if [[ "$base_name" == "tests.rs" || "$base_name" == test*.rs ]]; then
      continue
    fi

    local line_count
    line_count="$(wc -l < "$source_file" | tr -d ' ')"

    if (( line_count > 600 )); then
      report_violation "production file exceeds 600 lines: ${source_file} (${line_count} lines)"
    elif (( line_count > 400 )); then
      report_warning "production file exceeds 400 lines: ${source_file} (${line_count} lines)"
    fi
  done < <(find crates -type f -path '*/src/*' -name '*.rs' | sort)
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

echo ""
echo "Architecture harness summary: mode=$MODE violations=$violations warnings=$warnings"

if [[ "$MODE" == "strict" && $violations -gt 0 ]]; then
  exit 1
fi

exit 0
