#!/usr/bin/env bash
# Run a pinned Gitleaks scanner against Git history only. The working tree is
# deliberately out of scope, so untracked files cannot enter the scan.
set -euo pipefail
umask 077

readonly GITLEAKS_VERSION="8.30.1"
readonly RELEASE_BASE_URL="https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}"
readonly RELEASE_CHECKSUMS_FILENAME="gitleaks_${GITLEAKS_VERSION}_checksums.txt"
readonly RELEASE_CHECKSUMS_URL="${RELEASE_BASE_URL}/${RELEASE_CHECKSUMS_FILENAME}"
readonly RELEASE_CHECKSUMS_SHA256="061476c21adaf5441516f96f185c1a4706a83cd6329b9b38762271b3d4a52fae"
readonly CONFIG_SOURCE_URL="https://raw.githubusercontent.com/gitleaks/gitleaks/v${GITLEAKS_VERSION}/config/gitleaks.toml"
readonly CONFIG_UPSTREAM_SHA256="e163e53b9e7e8a8511e77271e2b323ed057759542a6d988258afe3a1fa329caf"
readonly CONFIG_SHA256="0ceeb4f9c567f9f80ee05e8e37eeba4646df809f69c736a64d5b8b1398eb3e4c"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly SCRIPT_DIR REPO_ROOT
readonly CHECKSUM_MANIFEST="${SCRIPT_DIR}/gitleaks-v${GITLEAKS_VERSION}.sha256"
readonly CONFIG_PATH="${SCRIPT_DIR}/gitleaks-v${GITLEAKS_VERSION}/default.gitleaks.toml"
readonly ARTIFACT_ROOT="${REPO_ROOT}/artifacts/security/gitleaks"
readonly CACHE_DIR="${ARTIFACT_ROOT}/cache/v${GITLEAKS_VERSION}"

MODE="scan"
WORK_DIR=""
MIRROR_DIR=""
TOOL_DIR=""
RUN_DIR=""
RAW_REPORT=""
MACHINE_MANIFEST=""
ASSET_NAME=""
ARCHIVE_SHA256=""
ARCHIVE_PATH=""
GITLEAKS_BIN=""
GITLEAKS_BINARY_SHA256=""
RUN_UTC=""
SCANNER_EXIT_CODE=""
REPORT_VALID=false
FINDING_COUNT_JSON="null"
FINDING_PATHS_JSON="[]"
SECRET_SCAN_PASSED=false

CAPTURED_INVENTORY_SHA256=""
CAPTURED_REF_COUNT=""
CAPTURED_OBJECT_COUNT=""
CAPTURED_HEAD_MODE=""
CAPTURED_HEAD_REF=""
CAPTURED_HEAD_OID=""
SOURCE_BEFORE_INVENTORY_FILE=""
SOURCE_BEFORE_INVENTORY_SHA256=""
SOURCE_BEFORE_REF_COUNT=""
SOURCE_BEFORE_OBJECT_COUNT=""
SOURCE_BEFORE_HEAD_MODE=""
SOURCE_BEFORE_HEAD_REF=""
SOURCE_BEFORE_HEAD_OID=""
SOURCE_AFTER_INVENTORY_FILE=""
SOURCE_AFTER_INVENTORY_SHA256=""
SOURCE_AFTER_REF_COUNT=""
SOURCE_AFTER_OBJECT_COUNT=""
SOURCE_AFTER_HEAD_MODE=""
SOURCE_AFTER_HEAD_REF=""
SOURCE_AFTER_HEAD_OID=""
SOURCE_INVENTORY_STABLE=false
MIRROR_INVENTORY_FILE=""
MIRROR_INVENTORY_SHA256=""
MIRROR_REF_COUNT=""
MIRROR_OBJECT_COUNT=""
MIRROR_HEAD_MODE=""
MIRROR_HEAD_REF=""
MIRROR_HEAD_OID=""

usage() {
  cat <<'USAGE'
Usage: scripts/security/gitleaks-scan.sh [--verify-config|--self-check|--help]

Without an option, scans all reachable Git history in a temporary bare
no-hardlinks mirror. Untracked working-tree files are never scanned.

  --verify-config  Verify the pinned effective-default configuration and archive
                   checksum manifest only.
  --self-check     Verify configuration, download and validate the pinned scanner,
                   and assert its version. Does not scan repository history.
  --help           Show this help text.
USAGE
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

cleanup() {
  if [[ -n "$WORK_DIR" && -d "$WORK_DIR" ]]; then
    rm -rf -- "$WORK_DIR"
  fi
}
trap cleanup EXIT HUP INT TERM

require_command() {
  local command_name="$1"
  command -v "$command_name" >/dev/null 2>&1 || die "required command is unavailable: ${command_name}"
}

sha256_file() {
  local file="$1"

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -- "$file" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum -- "$file" | awk '{print $1}'
  else
    die "neither shasum nor sha256sum is available"
  fi
}

sha256_stdin() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    die "neither shasum nor sha256sum is available"
  fi
}

expected_sha_for_asset() {
  case "$1" in
    "gitleaks_8.30.1_darwin_arm64.tar.gz")
      printf '%s\n' "b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5"
      ;;
    "gitleaks_8.30.1_darwin_x64.tar.gz")
      printf '%s\n' "dfe101a4db2255fc85120ac7f3d25e4342c3c20cf749f2c20a18081af1952709"
      ;;
    "gitleaks_8.30.1_linux_arm64.tar.gz")
      printf '%s\n' "e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080"
      ;;
    "gitleaks_8.30.1_linux_x64.tar.gz")
      printf '%s\n' "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb"
      ;;
    *)
      return 1
      ;;
  esac
}

manifest_sha_for_asset() {
  local asset="$1"
  local manifest_sha

  manifest_sha="$(awk -v asset="$asset" '
    NF && $1 !~ /^#/ && $2 == asset {
      count += 1
      value = $1
    }
    END {
      if (count != 1) exit 1
      print value
    }
  ' "$CHECKSUM_MANIFEST")" || die "checksum manifest must contain exactly one entry for ${asset}"

  [[ "$manifest_sha" =~ ^[0-9a-f]{64}$ ]] || die "checksum manifest has an invalid SHA-256 for ${asset}"
  printf '%s\n' "$manifest_sha"
}

validate_checksum_manifest() {
  [[ -f "$CHECKSUM_MANIFEST" ]] || die "missing checksum manifest: ${CHECKSUM_MANIFEST}"

  local non_comment_count
  non_comment_count="$(awk 'NF && $1 !~ /^#/ { count += 1 } END { print count + 0 }' "$CHECKSUM_MANIFEST")"
  [[ "$non_comment_count" == "4" ]] || die "checksum manifest must contain exactly four supported archives"

  local asset expected_sha manifest_sha
  for asset in \
    "gitleaks_8.30.1_darwin_arm64.tar.gz" \
    "gitleaks_8.30.1_darwin_x64.tar.gz" \
    "gitleaks_8.30.1_linux_arm64.tar.gz" \
    "gitleaks_8.30.1_linux_x64.tar.gz"; do
    expected_sha="$(expected_sha_for_asset "$asset")"
    manifest_sha="$(manifest_sha_for_asset "$asset")"
    [[ "$manifest_sha" == "$expected_sha" ]] || die "checksum manifest digest differs from the pinned upstream value for ${asset}"
  done
}

has_broad_project_maintenance_allowlist() {
  local config_path="$1"

  awk '
    /^[[:space:]]*#/ { next }
    /testdata/ || /test\\\.go/ { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "$config_path"
}

validate_config() {
  [[ -f "$CONFIG_PATH" ]] || die "missing pinned effective-default config snapshot: ${CONFIG_PATH}"

  local actual_sha
  actual_sha="$(sha256_file "$CONFIG_PATH")"
  [[ "$actual_sha" == "$CONFIG_SHA256" ]] || die "effective-default config snapshot digest does not match ${GITLEAKS_VERSION}"

  if has_broad_project_maintenance_allowlist "$CONFIG_PATH"; then
    die "effective-default config must not contain project-maintenance test allowlists"
  fi
}

detect_platform() {
  local os arch os_label arch_label
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      os_label="darwin"
      ;;
    Linux)
      os_label="linux"
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      die "Windows is out of scope for this gitleaks scanner wrapper"
      ;;
    *)
      die "unsupported operating system: ${os}"
      ;;
  esac

  case "$arch" in
    arm64|aarch64)
      arch_label="arm64"
      ;;
    x86_64|amd64)
      arch_label="x64"
      ;;
    *)
      die "unsupported architecture: ${arch}"
      ;;
  esac

  ASSET_NAME="gitleaks_${GITLEAKS_VERSION}_${os_label}_${arch_label}.tar.gz"
  ARCHIVE_SHA256="$(expected_sha_for_asset "$ASSET_NAME")" || die "no pinned archive exists for ${os_label}/${arch_label}"
}

make_work_dir() {
  WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/petri-gitleaks.XXXXXX")" || die "failed to create temporary workspace"
  chmod 700 "$WORK_DIR"
}

download_to() {
  local url="$1"
  local destination="$2"

  if ! curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
    --retry 3 --connect-timeout 20 --max-time 180 --output "$destination" "$url"; then
    die "failed to download a pinned gitleaks release asset from GitHub"
  fi
}

verify_upstream_release_manifest() {
  local upstream_manifest="${WORK_DIR}/${RELEASE_CHECKSUMS_FILENAME}"
  download_to "$RELEASE_CHECKSUMS_URL" "$upstream_manifest"

  local upstream_manifest_sha upstream_archive_sha
  upstream_manifest_sha="$(sha256_file "$upstream_manifest")"
  [[ "$upstream_manifest_sha" == "$RELEASE_CHECKSUMS_SHA256" ]] || die "official release checksum manifest digest did not match the pinned value"

  upstream_archive_sha="$(awk -v asset="$ASSET_NAME" '
    $2 == asset {
      count += 1
      value = $1
    }
    END {
      if (count != 1) exit 1
      print value
    }
  ' "$upstream_manifest")" || die "official release checksum manifest lacks a unique entry for ${ASSET_NAME}"

  [[ "$upstream_archive_sha" == "$ARCHIVE_SHA256" ]] || die "official release checksum for ${ASSET_NAME} differs from the pinned value"
}

assert_artifact_root_ignored() {
  if ! git -C "$REPO_ROOT" check-ignore -q -- "artifacts/security/gitleaks"; then
    die "artifacts/security/gitleaks must remain ignored so private reports cannot be staged"
  fi
}

ensure_verified_archive() {
  assert_artifact_root_ignored
  mkdir -p "$CACHE_DIR"
  chmod 700 "$CACHE_DIR"
  ARCHIVE_PATH="${CACHE_DIR}/${ASSET_NAME}"

  local actual_sha temporary_archive
  if [[ -f "$ARCHIVE_PATH" ]]; then
    actual_sha="$(sha256_file "$ARCHIVE_PATH")"
    if [[ "$actual_sha" != "$ARCHIVE_SHA256" ]]; then
      rm -f -- "$ARCHIVE_PATH"
    fi
  fi

  if [[ ! -f "$ARCHIVE_PATH" ]]; then
    temporary_archive="$(mktemp "${CACHE_DIR}/.${ASSET_NAME}.XXXXXX")" || die "failed to allocate archive download path"
    chmod 600 "$temporary_archive"
    download_to "${RELEASE_BASE_URL}/${ASSET_NAME}" "$temporary_archive"

    actual_sha="$(sha256_file "$temporary_archive")"
    [[ "$actual_sha" == "$ARCHIVE_SHA256" ]] || {
      rm -f -- "$temporary_archive"
      die "downloaded gitleaks archive checksum did not match the pinned value"
    }

    mv -f -- "$temporary_archive" "$ARCHIVE_PATH"
    chmod 600 "$ARCHIVE_PATH"
  fi

  actual_sha="$(sha256_file "$ARCHIVE_PATH")"
  [[ "$actual_sha" == "$ARCHIVE_SHA256" ]] || die "cached gitleaks archive checksum did not match the pinned value"
}

install_verified_gitleaks() {
  TOOL_DIR="${WORK_DIR}/tool"
  mkdir -p "$TOOL_DIR"
  chmod 700 "$TOOL_DIR"

  if ! tar -tzf "$ARCHIVE_PATH" | awk '$0 == "gitleaks" { found = 1 } END { exit(found ? 0 : 1) }'; then
    die "verified gitleaks archive does not contain the expected gitleaks binary"
  fi
  if ! tar -xzf "$ARCHIVE_PATH" -C "$TOOL_DIR" gitleaks; then
    die "failed to extract the verified gitleaks binary"
  fi

  GITLEAKS_BIN="${TOOL_DIR}/gitleaks"
  [[ -f "$GITLEAKS_BIN" ]] || die "verified gitleaks archive did not extract a binary"
  chmod 700 "$GITLEAKS_BIN"

  local reported_version
  reported_version="$("$GITLEAKS_BIN" version 2>/dev/null | tr -d '\r\n')" || die "pinned gitleaks binary could not report its version"
  case "$reported_version" in
    "$GITLEAKS_VERSION"|"v$GITLEAKS_VERSION")
      ;;
    *)
      die "pinned gitleaks binary reported an unexpected version"
      ;;
  esac

  GITLEAKS_BINARY_SHA256="$(sha256_file "$GITLEAKS_BIN")"
}

create_private_artifacts_at() {
  local artifact_root="$1"

  mkdir -p "$artifact_root"
  chmod 700 "$artifact_root"
  RUN_UTC="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
  RUN_DIR="${artifact_root}/${RUN_UTC}"
  if ! mkdir "$RUN_DIR"; then
    die "scan artifact directory already exists for ${RUN_UTC}; rerun after the next UTC second"
  fi
  chmod 700 "$RUN_DIR"

  RAW_REPORT="${RUN_DIR}/gitleaks.redacted.json"
  MACHINE_MANIFEST="${RUN_DIR}/manifest.json"
  : > "$RAW_REPORT"
  : > "$MACHINE_MANIFEST"
  chmod 600 "$RAW_REPORT"
  chmod 600 "$MACHINE_MANIFEST"
}

create_run_artifacts() {
  assert_artifact_root_ignored
  create_private_artifacts_at "$ARTIFACT_ROOT"
}

capture_inventory() {
  local repository="$1"
  local inventory_file="$2"
  local refs_file objects_file head_mode head_ref head_oid

  head_oid="$(git -C "$repository" rev-parse --verify 'HEAD^{commit}' 2>/dev/null)" || die "repository HEAD commit is unavailable for inventory"
  if head_ref="$(git -C "$repository" symbolic-ref -q HEAD 2>/dev/null)"; then
    head_mode="symbolic"
  else
    head_mode="detached"
    head_ref=""
  fi

  refs_file="${inventory_file}.refs"
  objects_file="${inventory_file}.objects"
  if ! git -C "$repository" for-each-ref --format='%(refname)%09%(objectname)%09%(objecttype)%09%(symref)' refs \
    | LC_ALL=C sort > "$refs_file"; then
    die "failed to enumerate repository refs for inventory"
  fi

  if ! {
    printf '%s\n' "$head_oid"
    awk -F '\t' '{print $2}' "$refs_file"
    git -C "$repository" rev-list --objects --all "$head_oid" | awk '{print $1}'
  } | LC_ALL=C sort -u > "$objects_file"; then
    die "failed to enumerate reachable repository objects for inventory"
  fi

  CAPTURED_REF_COUNT="$(wc -l < "$refs_file" | tr -d ' ')"
  CAPTURED_OBJECT_COUNT="$(wc -l < "$objects_file" | tr -d ' ')"
  {
    printf 'petri-gitleaks-inventory-v1\n'
    printf 'head-mode=%s\n' "$head_mode"
    printf 'head-ref=%s\n' "$head_ref"
    printf 'head-oid=%s\n' "$head_oid"
    printf 'ref-count=%s\n' "$CAPTURED_REF_COUNT"
    printf 'object-count=%s\n' "$CAPTURED_OBJECT_COUNT"
    printf 'refs\n'
    cat "$refs_file"
    printf 'objects\n'
    cat "$objects_file"
  } > "$inventory_file"
  rm -f -- "$refs_file" "$objects_file"

  CAPTURED_INVENTORY_SHA256="$(sha256_file "$inventory_file")"
  CAPTURED_HEAD_MODE="$head_mode"
  CAPTURED_HEAD_REF="$head_ref"
  CAPTURED_HEAD_OID="$head_oid"
}

snapshot_source_before() {
  SOURCE_BEFORE_INVENTORY_FILE="${WORK_DIR}/source-before.inventory"
  capture_inventory "$REPO_ROOT" "$SOURCE_BEFORE_INVENTORY_FILE"
  SOURCE_BEFORE_INVENTORY_SHA256="$CAPTURED_INVENTORY_SHA256"
  SOURCE_BEFORE_REF_COUNT="$CAPTURED_REF_COUNT"
  SOURCE_BEFORE_OBJECT_COUNT="$CAPTURED_OBJECT_COUNT"
  SOURCE_BEFORE_HEAD_MODE="$CAPTURED_HEAD_MODE"
  SOURCE_BEFORE_HEAD_REF="$CAPTURED_HEAD_REF"
  SOURCE_BEFORE_HEAD_OID="$CAPTURED_HEAD_OID"
}

snapshot_source_after() {
  SOURCE_AFTER_INVENTORY_FILE="${WORK_DIR}/source-after.inventory"
  capture_inventory "$REPO_ROOT" "$SOURCE_AFTER_INVENTORY_FILE"
  SOURCE_AFTER_INVENTORY_SHA256="$CAPTURED_INVENTORY_SHA256"
  SOURCE_AFTER_REF_COUNT="$CAPTURED_REF_COUNT"
  SOURCE_AFTER_OBJECT_COUNT="$CAPTURED_OBJECT_COUNT"
  SOURCE_AFTER_HEAD_MODE="$CAPTURED_HEAD_MODE"
  SOURCE_AFTER_HEAD_REF="$CAPTURED_HEAD_REF"
  SOURCE_AFTER_HEAD_OID="$CAPTURED_HEAD_OID"
}

snapshot_mirror_inventory() {
  MIRROR_INVENTORY_FILE="${WORK_DIR}/mirror.inventory"
  capture_inventory "$MIRROR_DIR" "$MIRROR_INVENTORY_FILE"
  MIRROR_INVENTORY_SHA256="$CAPTURED_INVENTORY_SHA256"
  MIRROR_REF_COUNT="$CAPTURED_REF_COUNT"
  MIRROR_OBJECT_COUNT="$CAPTURED_OBJECT_COUNT"
  MIRROR_HEAD_MODE="$CAPTURED_HEAD_MODE"
  MIRROR_HEAD_REF="$CAPTURED_HEAD_REF"
  MIRROR_HEAD_OID="$CAPTURED_HEAD_OID"
}

source_inventory_matches() {
  cmp -s "$SOURCE_BEFORE_INVENTORY_FILE" "$SOURCE_AFTER_INVENTORY_FILE"
}

create_no_hardlinks_mirror() {
  MIRROR_DIR="${WORK_DIR}/history.git"
  if ! git clone --mirror --no-hardlinks "$REPO_ROOT" "$MIRROR_DIR" >/dev/null 2>&1; then
    die "failed to create a temporary no-hardlinks Git mirror"
  fi
}

scan_history() {
  REPORT_VALID=false
  FINDING_COUNT_JSON="null"
  FINDING_PATHS_JSON="[]"

  set +e
  "$GITLEAKS_BIN" git \
    --config "$CONFIG_PATH" \
    --log-opts="--all --full-history" \
    --redact=100 \
    --report-format=json \
    --report-path "$RAW_REPORT" \
    "$MIRROR_DIR" >/dev/null 2>&1
  SCANNER_EXIT_CODE=$?
  set -e
  chmod 600 "$RAW_REPORT"

  if [[ "$SCANNER_EXIT_CODE" == "0" || "$SCANNER_EXIT_CODE" == "1" ]] \
    && jq -e 'type == "array"' "$RAW_REPORT" >/dev/null 2>&1; then
    REPORT_VALID=true
    FINDING_COUNT_JSON="$(jq -er 'length' "$RAW_REPORT")" || die "gitleaks report was not a valid findings array"
    FINDING_PATHS_JSON="$(jq -ce '[.[] | (.File // empty) | select(type == "string")] | unique | sort' "$RAW_REPORT")" || die "gitleaks report paths could not be parsed"
  fi
}

refresh_secret_scan_passed() {
  SECRET_SCAN_PASSED=false
  if [[ "$SOURCE_INVENTORY_STABLE" == "true" \
    && "$SCANNER_EXIT_CODE" == "0" \
    && "$REPORT_VALID" == "true" \
    && "$FINDING_COUNT_JSON" == "0" ]]; then
    SECRET_SCAN_PASSED=true
  fi
}

wrapper_exit_code() {
  if [[ "$SECRET_SCAN_PASSED" == "true" ]]; then
    printf '0\n'
  elif [[ "$SOURCE_INVENTORY_STABLE" != "true" ]]; then
    printf '2\n'
  elif [[ "$SCANNER_EXIT_CODE" == "1" ]]; then
    printf '1\n'
  elif [[ "$SCANNER_EXIT_CODE" == "0" && "$REPORT_VALID" == "true" ]]; then
    printf '1\n'
  else
    printf '2\n'
  fi
}

write_machine_manifest() {
  local relative_config_path relative_report_path
  relative_config_path="${CONFIG_PATH#"${REPO_ROOT}/"}"
  relative_report_path="${RAW_REPORT#"${REPO_ROOT}/"}"

  jq -n \
    --arg generated_at_utc "$RUN_UTC" \
    --arg scanner_name "gitleaks" \
    --arg scanner_version "$GITLEAKS_VERSION" \
    --arg scanner_binary_sha256 "$GITLEAKS_BINARY_SHA256" \
    --arg archive_filename "$ASSET_NAME" \
    --arg archive_sha256 "$ARCHIVE_SHA256" \
    --arg release_checksums_source "$RELEASE_CHECKSUMS_URL" \
    --arg release_checksums_sha256 "$RELEASE_CHECKSUMS_SHA256" \
    --arg config_path "$relative_config_path" \
    --arg config_sha256 "$CONFIG_SHA256" \
    --arg config_source "$CONFIG_SOURCE_URL" \
    --arg config_upstream_sha256 "$CONFIG_UPSTREAM_SHA256" \
    --arg source_before_head_mode "$SOURCE_BEFORE_HEAD_MODE" \
    --arg source_before_head_ref "$SOURCE_BEFORE_HEAD_REF" \
    --arg source_before_head_oid "$SOURCE_BEFORE_HEAD_OID" \
    --arg source_before_inventory_sha256 "$SOURCE_BEFORE_INVENTORY_SHA256" \
    --arg source_before_ref_count "$SOURCE_BEFORE_REF_COUNT" \
    --arg source_before_object_count "$SOURCE_BEFORE_OBJECT_COUNT" \
    --arg source_after_head_mode "$SOURCE_AFTER_HEAD_MODE" \
    --arg source_after_head_ref "$SOURCE_AFTER_HEAD_REF" \
    --arg source_after_head_oid "$SOURCE_AFTER_HEAD_OID" \
    --arg source_after_inventory_sha256 "$SOURCE_AFTER_INVENTORY_SHA256" \
    --arg source_after_ref_count "$SOURCE_AFTER_REF_COUNT" \
    --arg source_after_object_count "$SOURCE_AFTER_OBJECT_COUNT" \
    --arg mirror_head_mode "$MIRROR_HEAD_MODE" \
    --arg mirror_head_ref "$MIRROR_HEAD_REF" \
    --arg mirror_head_oid "$MIRROR_HEAD_OID" \
    --arg mirror_inventory_sha256 "$MIRROR_INVENTORY_SHA256" \
    --arg mirror_ref_count "$MIRROR_REF_COUNT" \
    --arg mirror_object_count "$MIRROR_OBJECT_COUNT" \
    --arg report_path "$relative_report_path" \
    --argjson scanner_exit_code "$SCANNER_EXIT_CODE" \
    --argjson report_valid "$REPORT_VALID" \
    --argjson finding_count "$FINDING_COUNT_JSON" \
    --argjson finding_paths "$FINDING_PATHS_JSON" \
    --argjson source_inventory_stable "$SOURCE_INVENTORY_STABLE" \
    --argjson secret_scan_passed "$SECRET_SCAN_PASSED" \
    '{
      schema_version: 2,
      generated_at_utc: $generated_at_utc,
      scanner: {
        name: $scanner_name,
        version: $scanner_version,
        binary_sha256: $scanner_binary_sha256,
        archive: {
          filename: $archive_filename,
          sha256: $archive_sha256,
          official_release_checksums: {
            source: $release_checksums_source,
            sha256: $release_checksums_sha256
          }
        }
      },
      config: {
        kind: "tagged-upstream-default-derived",
        derivation: "gitleaks v8.30.1 config/gitleaks.toml with only its final empty line removed; explicitly passed to the verified v8.30.1 binary",
        path: $config_path,
        sha256: $config_sha256,
        source: $config_source,
        source_sha256: $config_upstream_sha256
      },
      source: {
        inventory_stable: $source_inventory_stable,
        before_mirror: {
          head: {
            mode: $source_before_head_mode,
            ref: ($source_before_head_ref | if . == "" then null else . end),
            oid: $source_before_head_oid
          },
          sorted_ref_object_inventory_sha256: $source_before_inventory_sha256,
          ref_count: ($source_before_ref_count | tonumber),
          object_count: ($source_before_object_count | tonumber)
        },
        after_scan: {
          head: {
            mode: $source_after_head_mode,
            ref: ($source_after_head_ref | if . == "" then null else . end),
            oid: $source_after_head_oid
          },
          sorted_ref_object_inventory_sha256: $source_after_inventory_sha256,
          ref_count: ($source_after_ref_count | tonumber),
          object_count: ($source_after_object_count | tonumber)
        }
      },
      mirror: {
        head: {
          mode: $mirror_head_mode,
          ref: ($mirror_head_ref | if . == "" then null else . end),
          oid: $mirror_head_oid
        },
        sorted_ref_object_inventory_sha256: $mirror_inventory_sha256,
        ref_count: ($mirror_ref_count | tonumber),
        object_count: ($mirror_object_count | tonumber)
      },
      scope: {
        mirror: "temporary bare local clone created with git clone --mirror --no-hardlinks",
        command: [
          "gitleaks",
          "git",
          "--config",
          $config_path,
          "--log-opts=--all --full-history",
          "--redact=100",
          "--report-format=json",
          "--report-path=<private-redacted-report>",
          "<temporary-no-hardlinks-mirror>"
        ],
        history_selection: "--all --full-history",
        redaction_percent: 100,
        working_tree_scanned: false,
        report_path: $report_path
      },
      result: {
        scanner_exit_code: $scanner_exit_code,
        report_valid: $report_valid,
        finding_count: $finding_count,
        finding_paths: $finding_paths,
        secret_scan_passed: $secret_scan_passed
      }
    }' > "$MACHINE_MANIFEST"
  chmod 600 "$MACHINE_MANIFEST"
}

relative_to_repo() {
  local path="$1"
  printf '%s\n' "${path#"${REPO_ROOT}/"}"
}

emit_private_evidence_paths() {
  printf 'Private redacted report: %s\nPrivate machine manifest: %s\n' \
    "$(relative_to_repo "$RAW_REPORT")" \
    "$(relative_to_repo "$MACHINE_MANIFEST")"
}

emit_finding_summary_if_available() {
  if [[ "$REPORT_VALID" == "true" && "$FINDING_COUNT_JSON" != "null" ]]; then
    printf 'Finding count: %s\n' "$FINDING_COUNT_JSON" >&2
    printf 'Finding paths (JSON): %s\n' "$FINDING_PATHS_JSON" >&2
  fi
}

run_self_check() {
  validate_config
  validate_checksum_manifest
  detect_platform
  make_work_dir
  verify_upstream_release_manifest
  ensure_verified_archive
  install_verified_gitleaks
  printf 'Gitleaks %s installer/effective-default-config self-check passed for %s.\n' "$GITLEAKS_VERSION" "$ASSET_NAME"
}

run_scan() {
  validate_config
  validate_checksum_manifest
  detect_platform
  make_work_dir
  verify_upstream_release_manifest
  ensure_verified_archive
  install_verified_gitleaks
  create_run_artifacts
  snapshot_source_before
  create_no_hardlinks_mirror
  snapshot_mirror_inventory
  scan_history
  snapshot_source_after
  if source_inventory_matches; then
    SOURCE_INVENTORY_STABLE=true
  else
    SOURCE_INVENTORY_STABLE=false
  fi
  refresh_secret_scan_passed
  write_machine_manifest

  local exit_code
  exit_code="$(wrapper_exit_code)"
  if [[ "$SOURCE_INVENTORY_STABLE" != "true" ]]; then
    printf 'ERROR: source ref/object inventory changed during the scan; secret scan did not pass.\n' >&2
    emit_private_evidence_paths >&2
    return "$exit_code"
  fi

  if [[ "$SECRET_SCAN_PASSED" == "true" ]]; then
    printf 'PASS: gitleaks %s full-history mirror scan found 0 findings.\n' "$GITLEAKS_VERSION"
    emit_private_evidence_paths
    return 0
  fi

  if [[ "$SCANNER_EXIT_CODE" == "0" && "$REPORT_VALID" != "true" ]]; then
    printf 'ERROR: gitleaks returned success without a valid redacted JSON report; secret scan did not pass.\n' >&2
  elif [[ "$SCANNER_EXIT_CODE" != "0" ]]; then
    printf 'ERROR: gitleaks scanner exit code %s; secret scan did not pass.\n' "$SCANNER_EXIT_CODE" >&2
  else
    printf 'FINDINGS: secret scan did not pass.\n' >&2
  fi
  emit_finding_summary_if_available
  emit_private_evidence_paths >&2
  return "$exit_code"
}

main() {
  if [[ $# -gt 1 ]]; then
    usage >&2
    return 2
  fi

  if [[ $# -eq 1 ]]; then
    case "$1" in
      --verify-config)
        MODE="verify-config"
        ;;
      --self-check)
        MODE="self-check"
        ;;
      --help|-h)
        usage
        return 0
        ;;
      *)
        usage >&2
        return 2
        ;;
    esac
  fi

  case "$MODE" in
    verify-config)
      validate_config
      validate_checksum_manifest
      printf 'Pinned effective-default config and archive checksum manifest are valid.\n'
      ;;
    self-check)
      require_command curl
      require_command tar
      run_self_check
      ;;
    scan)
      require_command curl
      require_command tar
      require_command git
      require_command jq
      run_scan
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
