#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/security/gitleaks-scan.sh"
CHECKSUM_MANIFEST="$ROOT_DIR/scripts/security/gitleaks-v8.30.1.sha256"
CONFIG_PATH="$ROOT_DIR/scripts/security/gitleaks-v8.30.1/default.gitleaks.toml"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/petri-gitleaks-test.XXXXXX")"
trap 'rm -rf -- "$TMP_DIR"' EXIT

fail() {
  printf 'TEST FAILURE: %s\n' "$*" >&2
  exit 1
}

assert_equal() {
  local expected="$1"
  local actual="$2"
  local message="$3"
  [[ "$actual" == "$expected" ]] || fail "${message}: expected=${expected} actual=${actual}"
}

assert_file_mode() {
  local path="$1"
  local expected_mode="$2"
  local actual_mode

  if actual_mode="$(stat -f '%Lp' "$path" 2>/dev/null)"; then
    :
  else
    actual_mode="$(stat -c '%a' "$path")"
  fi
  assert_equal "$expected_mode" "$actual_mode" "unexpected mode for ${path}"
}

portable_sha256_file() {
  local path="$1"

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -- "$path" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum -- "$path" | awk '{print $1}'
  else
    fail "no SHA-256 command is available"
  fi
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "expected file is missing: ${path}"
}

require_file "$SCRIPT"
require_file "$CHECKSUM_MANIFEST"
require_file "$CONFIG_PATH"
[[ ! -e "$ROOT_DIR/scripts/security/gitleaks-v8.30.1/.gitleaks.toml" ]] \
  || fail "development .gitleaks.toml snapshot must not remain alongside the effective default"
bash -n "$SCRIPT"

assert_equal \
  "0ceeb4f9c567f9f80ee05e8e37eeba4646df809f69c736a64d5b8b1398eb3e4c" \
  "$(portable_sha256_file "$CONFIG_PATH")" \
  "tagged upstream effective-default config digest"

expected_manifest="$(printf '%s\n' \
  'b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5  gitleaks_8.30.1_darwin_arm64.tar.gz' \
  'dfe101a4db2255fc85120ac7f3d25e4342c3c20cf749f2c20a18081af1952709  gitleaks_8.30.1_darwin_x64.tar.gz' \
  'e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080  gitleaks_8.30.1_linux_arm64.tar.gz' \
  '551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb  gitleaks_8.30.1_linux_x64.tar.gz')"
actual_manifest="$(awk 'NF && $1 !~ /^#/ {print $1 "  " $2}' "$CHECKSUM_MANIFEST")"
assert_equal "$expected_manifest" "$actual_manifest" "supported v8.30.1 archive checksum set"

"$SCRIPT" --verify-config >/dev/null

run_library_tests() (
  source "$SCRIPT"

  WORK_DIR="$TMP_DIR/work"
  mkdir -p "$WORK_DIR"
  chmod 700 "$WORK_DIR"

  validate_config
  assert_equal \
    "e163e53b9e7e8a8511e77271e2b323ed057759542a6d988258afe3a1fa329caf" \
    "$CONFIG_UPSTREAM_SHA256" \
    "tagged upstream source config digest"
  if has_broad_project_maintenance_allowlist "$CONFIG_PATH"; then
    fail "effective-default config unexpectedly contains a project-maintenance allowlist"
  fi
  bad_config="$TMP_DIR/broad-allowlist.toml"
  printf '%s\n' '[allowlist]' "paths = [ '''testdata''' ]" > "$bad_config"
  if ! has_broad_project_maintenance_allowlist "$bad_config"; then
    fail "maintenance allowlist detector did not reject testdata"
  fi

  fake_gitleaks() {
    local report_path=""
    local saw_config=false
    local saw_history=false
    local saw_redaction=false
    local saw_report_format=false
    local saw_mirror=false

    while [[ $# -gt 0 ]]; do
      case "$1" in
        git)
          ;;
        --config)
          [[ $# -ge 2 && "$2" == "$CONFIG_PATH" ]] || return 90
          saw_config=true
          shift
          ;;
        --log-opts=--all\ --full-history)
          saw_history=true
          ;;
        --redact=100)
          saw_redaction=true
          ;;
        --report-format=json)
          saw_report_format=true
          ;;
        --report-path)
          [[ $# -ge 2 ]] || return 91
          report_path="$2"
          shift
          ;;
        "$MIRROR_DIR")
          saw_mirror=true
          ;;
        *)
          return 92
          ;;
      esac
      shift
    done

    [[ "$saw_config" == true && "$saw_history" == true && "$saw_redaction" == true \
      && "$saw_report_format" == true && "$saw_mirror" == true && -n "$report_path" ]] || return 93
    printf '%s\n' "$FAKE_REPORT_JSON" > "$report_path"
    return "$FAKE_EXIT_CODE"
  }

  run_scan_case() {
    local name="$1"
    local scanner_exit="$2"
    local report_json="$3"
    local expected_valid="$4"
    local expected_count="$5"
    local expected_passed="$6"
    local expected_wrapper_exit="$7"

    FAKE_EXIT_CODE="$scanner_exit"
    FAKE_REPORT_JSON="$report_json"
    GITLEAKS_BIN=fake_gitleaks
    MIRROR_DIR="$TMP_DIR/fake-mirror-${name}"
    RAW_REPORT="$TMP_DIR/${name}.redacted.json"
    : > "$RAW_REPORT"
    chmod 600 "$RAW_REPORT"
    SOURCE_INVENTORY_STABLE=true
    scan_history
    refresh_secret_scan_passed

    assert_equal "$expected_valid" "$REPORT_VALID" "report validity for ${name}"
    assert_equal "$expected_count" "$FINDING_COUNT_JSON" "finding count for ${name}"
    assert_equal "$expected_passed" "$SECRET_SCAN_PASSED" "secret scan status for ${name}"
    assert_equal "$expected_wrapper_exit" "$(wrapper_exit_code)" "wrapper exit for ${name}"
    assert_file_mode "$RAW_REPORT" 600
  }

  run_scan_case clean 0 '[]' true 0 true 0
  run_scan_case scanner_nonzero_empty_report 1 '[]' true 0 false 1
  run_scan_case findings_with_zero_scanner_exit 0 '[{"File":"fixture/path.txt"}]' true 1 false 1
  run_scan_case malformed_report 0 'not-json' false null false 2
  run_scan_case scanner_error 2 '[]' false null false 2

  evidence_root="$TMP_DIR/evidence"
  create_private_artifacts_at "$evidence_root"
  assert_file_mode "$RUN_DIR" 700
  assert_file_mode "$RAW_REPORT" 600
  assert_file_mode "$MACHINE_MANIFEST" 600

  SOURCE_INVENTORY_STABLE=true
  SCANNER_EXIT_CODE=0
  REPORT_VALID=true
  FINDING_COUNT_JSON=0
  FINDING_PATHS_JSON='[]'
  SECRET_SCAN_PASSED=true
  GITLEAKS_BINARY_SHA256="$(printf 'a%.0s' {1..64})"
  ASSET_NAME="gitleaks_8.30.1_linux_x64.tar.gz"
  ARCHIVE_SHA256="$(printf 'b%.0s' {1..64})"
  SOURCE_BEFORE_HEAD_MODE=symbolic
  SOURCE_BEFORE_HEAD_REF=refs/heads/main
  SOURCE_BEFORE_HEAD_OID="$(printf 'c%.0s' {1..40})"
  SOURCE_BEFORE_INVENTORY_SHA256="$(printf 'd%.0s' {1..64})"
  SOURCE_BEFORE_REF_COUNT=4
  SOURCE_BEFORE_OBJECT_COUNT=9
  SOURCE_AFTER_HEAD_MODE=symbolic
  SOURCE_AFTER_HEAD_REF=refs/heads/main
  SOURCE_AFTER_HEAD_OID="$SOURCE_BEFORE_HEAD_OID"
  SOURCE_AFTER_INVENTORY_SHA256="$SOURCE_BEFORE_INVENTORY_SHA256"
  SOURCE_AFTER_REF_COUNT=4
  SOURCE_AFTER_OBJECT_COUNT=9
  MIRROR_HEAD_MODE=symbolic
  MIRROR_HEAD_REF=refs/heads/main
  MIRROR_HEAD_OID="$SOURCE_BEFORE_HEAD_OID"
  MIRROR_INVENTORY_SHA256="$(printf 'e%.0s' {1..64})"
  MIRROR_REF_COUNT=4
  MIRROR_OBJECT_COUNT=9
  write_machine_manifest
  assert_file_mode "$MACHINE_MANIFEST" 600
  jq -e '
    .schema_version == 2
    and .config.kind == "tagged-upstream-default-derived"
    and (.config.derivation | contains("config/gitleaks.toml"))
    and .config.source_sha256 == "e163e53b9e7e8a8511e77271e2b323ed057759542a6d988258afe3a1fa329caf"
    and .source.inventory_stable == true
    and .source.before_mirror.ref_count == 4
    and .source.after_scan.object_count == 9
    and .mirror.ref_count == 4
    and .result.scanner_exit_code == 0
    and .result.finding_count == 0
    and .result.secret_scan_passed == true
    and ([.. | objects | select(has("publication" + "_allowed"))] | length == 0)
  ' "$MACHINE_MANIFEST" >/dev/null || fail "machine manifest semantics are incorrect"

  SCANNER_EXIT_CODE=1
  SECRET_SCAN_PASSED=false
  write_machine_manifest
  jq -e '
    .result.scanner_exit_code == 1
    and .result.report_valid == true
    and .result.finding_count == 0
    and .result.secret_scan_passed == false
    and ([.. | objects | select(has("publication" + "_allowed"))] | length == 0)
  ' "$MACHINE_MANIFEST" >/dev/null || fail "nonzero scanner exit must not pass the machine manifest"

  inventory_repo="$TMP_DIR/inventory-repo"
  git init -q "$inventory_repo"
  git -C "$inventory_repo" config user.email gitleaks-test@example.invalid
  git -C "$inventory_repo" config user.name gitleaks-test
  printf 'inventory fixture\n' > "$inventory_repo/tracked.txt"
  git -C "$inventory_repo" add tracked.txt
  git -C "$inventory_repo" commit -q -m inventory-fixture
  inventory_head="$(git -C "$inventory_repo" rev-parse HEAD)"
  git -C "$inventory_repo" tag test-tag "$inventory_head"
  git -C "$inventory_repo" update-ref refs/remotes/origin/main "$inventory_head"
  git -C "$inventory_repo" update-ref refs/stash "$inventory_head"
  inventory_head_ref="$(git -C "$inventory_repo" symbolic-ref -q HEAD)"

  inventory_before="$TMP_DIR/inventory-before"
  inventory_after="$TMP_DIR/inventory-after"
  capture_inventory "$inventory_repo" "$inventory_before"
  assert_equal symbolic "$CAPTURED_HEAD_MODE" "symbolic source HEAD handling"
  assert_equal "$inventory_head" "$CAPTURED_HEAD_OID" "source HEAD object inventory"
  [[ "$CAPTURED_REF_COUNT" -ge 4 ]] || fail "inventory did not include heads/tags/remotes/stash"
  for required_ref in "$inventory_head_ref" refs/tags/test-tag refs/remotes/origin/main refs/stash; do
    awk -v ref="$required_ref" '
      /^refs$/ { in_refs = 1; next }
      /^objects$/ { in_refs = 0 }
      in_refs && $1 == ref { found = 1 }
      END { exit(found ? 0 : 1) }
    ' "$inventory_before" || fail "inventory omitted required ref: ${required_ref}"
  done
  git -C "$inventory_repo" update-ref refs/tags/mutation "$inventory_head"
  capture_inventory "$inventory_repo" "$inventory_after"
  SOURCE_BEFORE_INVENTORY_FILE="$inventory_before"
  SOURCE_AFTER_INVENTORY_FILE="$inventory_after"
  if source_inventory_matches; then
    fail "source ref mutation was not detected"
  fi
  SOURCE_INVENTORY_STABLE=false
  SCANNER_EXIT_CODE=0
  REPORT_VALID=true
  FINDING_COUNT_JSON=0
  refresh_secret_scan_passed
  assert_equal false "$SECRET_SCAN_PASSED" "source mutation must invalidate a clean scanner result"
  assert_equal 2 "$(wrapper_exit_code)" "source mutation wrapper exit"
  git -C "$inventory_repo" checkout -q --detach "$inventory_head"
  capture_inventory "$inventory_repo" "$TMP_DIR/inventory-detached"
  assert_equal detached "$CAPTURED_HEAD_MODE" "detached source HEAD handling"
 hash_fixture="$TMP_DIR/hash-fixture"
  printf 'portable hash fixture\n' > "$hash_fixture"
  expected_file_hash="$(portable_sha256_file "$hash_fixture")"
  expected_stream_hash="$expected_file_hash"
  # The fallback is emulated locally; stock macOS supplies shasum but not GNU
  # sha256sum, so this test must not require a host sha256sum binary.
  if test_hash_backend_path="$(command -v shasum)"; then
    test_hash_backend_kind=shasum
  elif test_hash_backend_path="$(command -v sha256sum)"; then
    test_hash_backend_kind=sha256sum
  else
    fail "no SHA-256 command is available for the test-local sha256sum emulator"
  fi
  hash_emulator_trace="$TMP_DIR/hash-emulator-trace"
  : > "$hash_emulator_trace"
  (
    command() {
      if [[ "${1:-}" == "-v" && "${2:-}" == "shasum" ]]; then
        return 1
      fi
      builtin command "$@"
    }
    sha256sum() {
      printf 'sha256sum\n' >> "$hash_emulator_trace"
      case "$test_hash_backend_kind" in
        shasum)
          "$test_hash_backend_path" -a 256 "$@"
          ;;
        sha256sum)
          "$test_hash_backend_path" "$@"
          ;;
        *)
          fail "unknown SHA-256 emulator backend: ${test_hash_backend_kind}"
          ;;
      esac
    }
    shasum() {
      fail "sha256sum fallback attempted to use shasum"
    }
    assert_equal "$expected_file_hash" "$(sha256_file "$hash_fixture")" "sha256sum file fallback"
    assert_equal "$expected_stream_hash" "$(printf 'portable hash fixture\n' | sha256_stdin)" "sha256sum stdin fallback"
  )
  assert_equal 2 "$(wc -l < "$hash_emulator_trace" | tr -d '[:space:]')" "test-local sha256sum emulator call count"
)
run_library_tests

assert_platform_failure() {
  local fixture_dir="$1"
  local expected_message="$2"
  local output

  if output="$(PATH="$fixture_dir:$PATH" "$SCRIPT" --self-check 2>&1)"; then
    fail "expected the scanner wrapper to fail closed for fixture: ${fixture_dir}"
  fi
  [[ "$output" == *"$expected_message"* ]] || fail "unexpected unsupported-platform error: ${output}"
}

assert_platform_failure \
  "$ROOT_DIR/scripts/security/test-fixtures/gitleaks-uname-windows" \
  "Windows is out of scope"
assert_platform_failure \
  "$ROOT_DIR/scripts/security/test-fixtures/gitleaks-uname-unsupported-arch" \
  "unsupported architecture: riscv64"

echo "Gitleaks scan tooling tests passed"
