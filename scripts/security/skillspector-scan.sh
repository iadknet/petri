#!/usr/bin/env bash
# Scan only the repository-local skill tree with a pinned, isolated SkillSpector
# container. Bootstrap is intentionally the sole networked operation.
set -euo pipefail
umask 077

readonly SKILLSPECTOR_VERSION="2.11.0"
readonly SKILLSPECTOR_COMMIT="b7241089d7ec15d8b30df980dacbb428214732b9"
readonly SKILLSPECTOR_ARCHIVE_FILENAME="SkillSpector-b7241089d7ec15d8b30df980dacbb428214732b9.tar.gz"
readonly SKILLSPECTOR_ARCHIVE_URL="https://github.com/NVIDIA/SkillSpector/archive/${SKILLSPECTOR_COMMIT}.tar.gz"
readonly SKILLSPECTOR_ARCHIVE_SHA256="1370abe7a76dbe90d6bddf7d1e0353cc58ae9442eece3b03aa7ba75830210dfc"
readonly PYTHON_BASE_IMAGE="python:3.12-slim-bookworm@sha256:0f5b26b9518d002b6173fd61daad821fa340635ebfec5bba471013f9ca114579"
readonly IMAGE_REF="petri-skillspector:2.11.0-b7241089"
readonly CONTAINER_RUNTIME_SECONDS=300
readonly CONTAINER_MEMORY="1g"
readonly CONTAINER_CPUS="1.0"
readonly CONTAINER_PIDS=128

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly SCRIPT_DIR REPO_ROOT
readonly SOURCE_MANIFEST="${SCRIPT_DIR}/skillspector/source-v${SKILLSPECTOR_VERSION}.sha256"
readonly DOCKERFILE="${SCRIPT_DIR}/skillspector/Dockerfile"
readonly SKILLS_ROOT="${REPO_ROOT}/.claude/skills"
readonly ARTIFACT_ROOT="${REPO_ROOT}/artifacts/security/skillspector"

MODE="scan"
WORK_DIR=""
STAGED_ROOT=""
RUN_DIR=""
RAW_REPORT=""
SCANNER_LOG=""
MACHINE_MANIFEST=""
RUN_UTC=""
IMAGE_ID=""
INPUT_SHA256=""
EXPECTED_ROOTS_FILE=""
SYMLINK_TRANSFORMATIONS_FILE=""
SCANNED_ROOTS_JSON="[]"
REPORT_VALID=false
REPORT_POLICY_REASON="report has not been assessed"
CAUTION_PRESENT=false
SCANNER_EXIT_CODE=2
SKILL_SCAN_PASSED=false
PARTIAL_CLASSIFICATION=""
PARTIAL_CLASSIFICATION_COUNT=0

usage() {
  cat <<'USAGE'
Usage: scripts/security/skillspector-scan.sh [--bootstrap|--self-check|--help]

Without an option, scans only a staged private copy of .claude/skills using a
previously bootstrapped local SkillSpector image. It never fetches or builds.

  --bootstrap   Download and verify the pinned upstream source archive, then
                build and validate the local image. This is the sole networked
                operation. It does not scan repository skills.
  --self-check  Validate the pinned metadata, local image, version, and CLI
                contract without downloading, building, or scanning.
  --help        Show this help text.
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

require_command() {
  local command_name="$1"
  command -v "$command_name" >/dev/null 2>&1 \
    || die "required command is unavailable: ${command_name}"
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

validate_source_manifest() {
  [[ -f "$SOURCE_MANIFEST" ]] || die "missing SkillSpector source checksum manifest"

  local entries expected actual
  entries="$(awk 'NF && $1 !~ /^#/ { count += 1 } END { print count + 0 }' "$SOURCE_MANIFEST")"
  [[ "$entries" == "1" ]] || die "SkillSpector source checksum manifest must contain exactly one archive"

  expected="${SKILLSPECTOR_ARCHIVE_SHA256}  ${SKILLSPECTOR_ARCHIVE_FILENAME}"
  actual="$(awk 'NF && $1 !~ /^#/ { print $1 "  " $2 }' "$SOURCE_MANIFEST")"
  [[ "$actual" == "$expected" ]] \
    || die "SkillSpector source checksum manifest differs from the pinned upstream archive"
}

validate_dockerfile() {
  [[ -f "$DOCKERFILE" ]] || die "missing SkillSpector Dockerfile"

  local from_line
  from_line="$(awk '$1 == "FROM" { print; exit }' "$DOCKERFILE")"
  [[ "$from_line" == "FROM ${PYTHON_BASE_IMAGE}" ]] \
    || die "SkillSpector Dockerfile must use the pinned official Python 3.12 image digest"

  awk '/uv sync --locked --no-dev/ { found = 1 } END { exit(found ? 0 : 1) }' "$DOCKERFILE" \
    || die "SkillSpector Dockerfile must resolve the upstream lockfile with uv sync --locked --no-dev"
  awk '/USER 65532:65532/ { found = 1 } END { exit(found ? 0 : 1) }' "$DOCKERFILE" \
    || die "SkillSpector Dockerfile must set a non-root default user"
  awk '/^[[:space:]]*ENTRYPOINT[[:space:]]/ { found = 1 } END { exit(found ? 1 : 0) }' "$DOCKERFILE" \
    || die "SkillSpector Dockerfile must not prepend an entrypoint to the verified CLI invocation"
}

make_work_dir() {
  WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/petri-skillspector.XXXXXX")" \
    || die "failed to create private SkillSpector workspace"
  chmod 700 "$WORK_DIR"
}

download_to() {
  local url="$1"
  local destination="$2"

  curl --fail --location --silent --show-error --proto '=https' --tlsv1.2 \
    --retry 3 --connect-timeout 20 --max-time 300 --output "$destination" "$url" \
    || die "failed to download the pinned NVIDIA SkillSpector source archive"
}

verify_upstream_source_contract() {
  local source_dir="$1"
  local pyproject="${source_dir}/pyproject.toml"
  local cli_source="${source_dir}/src/skillspector/cli.py"
  local reported_version

  [[ -f "$pyproject" && -f "${source_dir}/uv.lock" && -f "$cli_source" ]] \
    || die "verified SkillSpector archive lacks pyproject.toml, uv.lock, or the CLI source"

  reported_version="$(awk -F '"' '$1 ~ /^version = / { print $2; exit }' "$pyproject")"
  [[ "$reported_version" == "$SKILLSPECTOR_VERSION" ]] \
    || die "verified SkillSpector source reports an unexpected package version"

  local required_flag
  for required_flag in --recursive --no-llm --fail-on-incomplete --format --output; do
    grep -F -- "\"${required_flag}\"" "$cli_source" >/dev/null \
      || die "verified SkillSpector v${SKILLSPECTOR_VERSION} source lacks required CLI flag: ${required_flag}"
  done
}

resolve_existing_path() {
  local candidate="$1"
  local link_target=""
  local hop_count=0
  local canonical_parent

  while [[ -L "$candidate" ]]; do
    hop_count=$((hop_count + 1))
    [[ "$hop_count" -le 40 ]] || die "symlink resolution exceeded the maximum depth"
    link_target="$(readlink "$candidate")" || die "failed to read skill-tree symlink"
    if [[ "$link_target" == /* ]]; then
      candidate="$link_target"
    else
      candidate="$(dirname "$candidate")/${link_target}"
    fi
  done

  [[ -e "$candidate" ]] || die "skill-tree symlink target does not exist"
  canonical_parent="$(cd -P "$(dirname "$candidate")" && pwd -P)" \
    || die "failed to canonicalize a skill-tree path"
  printf '%s/%s\n' "$canonical_parent" "$(basename "$candidate")"
}

path_is_within() {
  local root="$1"
  local candidate="$2"

  case "$candidate" in
    "$root"|"$root"/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

validate_relative_path() {
  local relative="$1"

  [[ -n "$relative" && "$relative" != /* && "$relative" != *$'\n'* \
    && "$relative" != *$'\r'* && "$relative" != *$'\t'* ]] \
    || die "skill-tree paths containing absolute, newline, carriage-return, or tab characters are unsupported"
}

copy_staged_file() {
  local source_path="$1"
  local destination_path="$2"

  mkdir -p -- "$(dirname "$destination_path")"
  chmod 700 "$(dirname "$destination_path")"
  cat -- "$source_path" > "$destination_path"
  chmod 600 "$destination_path"
}

enumerate_skill_roots() {
  local staged_root="$1"
  local roots_file="$2"
  local roots_list="${roots_file}.list"
  local skill_file relative parent

  : > "$roots_list"
  while IFS= read -r -d '' skill_file; do
    relative="${skill_file#"${staged_root}/"}"
    validate_relative_path "$relative"
    parent="$(dirname "$relative")"
    if [[ "$parent" != "." && "$parent" == */* ]]; then
      die "nested skill roots are unsupported by SkillSpector's required --recursive mode"
    fi
    printf '%s\n' "$parent" >> "$roots_list"
  done < <(find "$staged_root" -type f -name SKILL.md -print0)

  LC_ALL=C sort -u "$roots_list" > "${roots_list}.sorted"
  mv -f -- "${roots_list}.sorted" "$roots_list"
  [[ -s "$roots_list" ]] || die "no SKILL.md roots were found in .claude/skills"

  jq -Rn '[inputs]' < "$roots_list" > "$roots_file"
  chmod 600 "$roots_file"
  rm -f -- "$roots_list"
}

staged_input_sha256() {
  local staged_root="$1"
  local metadata_dir="$(dirname "$staged_root")"
  local list_file
  local entry relative kind

  list_file="$(mktemp "${metadata_dir}/.skillspector-input.XXXXXX")" \
    || die "failed to create staged-input inventory"
  : > "$list_file"

  while IFS= read -r -d '' entry; do
    relative="${entry#"${staged_root}/"}"
    validate_relative_path "$relative"
    if [[ -d "$entry" ]]; then
      kind=D
    elif [[ -f "$entry" && ! -L "$entry" ]]; then
      kind=F
    else
      rm -f -- "$list_file"
      die "staged input contains an unsupported non-regular entry"
    fi
    printf '%s\t%s\n' "$kind" "$relative" >> "$list_file"
  done < <(find "$staged_root" -mindepth 1 -print0)

  LC_ALL=C sort "$list_file" > "${list_file}.sorted"
  mv -f -- "${list_file}.sorted" "$list_file"
  {
    printf 'petri-skillspector-input-v1\0'
    while IFS=$'\t' read -r kind relative; do
      printf '%s\0%s\0' "$kind" "$relative"
      if [[ "$kind" == F ]]; then
        cat -- "${staged_root}/${relative}"
        printf '\0'
      fi
    done < "$list_file"
  } | sha256_stdin
  rm -f -- "$list_file"
}

stage_skill_tree() {
  local source_root="$1"
  local staged_root="$2"
  local canonical_root metadata_dir staged_name transformations_jsonl
  local entry relative destination canonical_target target_relative

  [[ -d "$source_root" && ! -L "$source_root" ]] \
    || die ".claude/skills must be a real directory, not a symlink"
  canonical_root="$(cd -P "$source_root" && pwd -P)" \
    || die "failed to canonicalize .claude/skills"

  mkdir -p -- "$staged_root"
  chmod 700 "$staged_root"
  metadata_dir="$(dirname "$staged_root")"
  staged_name="$(basename "$staged_root")"
  EXPECTED_ROOTS_FILE="${metadata_dir}/${staged_name}.expected-roots.json"
  SYMLINK_TRANSFORMATIONS_FILE="${metadata_dir}/${staged_name}.symlink-transformations.json"
  transformations_jsonl="${metadata_dir}/${staged_name}.symlink-transformations.jsonl"
  : > "$transformations_jsonl"

  while IFS= read -r -d '' entry; do
    relative="${entry#"${canonical_root}/"}"
    validate_relative_path "$relative"
    destination="${staged_root}/${relative}"

    if [[ -L "$entry" ]]; then
      # Resolve and contain the link before reading its target. This ordering is
      # intentional: an external target is never copied or otherwise read.
      canonical_target="$(resolve_existing_path "$entry")"
      path_is_within "$canonical_root" "$canonical_target" \
        || die "skill-tree symlink resolves outside canonical skills root"
      [[ -f "$canonical_target" && ! -L "$canonical_target" ]] \
        || die "skill-tree symlink must resolve to a regular file"
      target_relative="${canonical_target#"${canonical_root}/"}"
      validate_relative_path "$target_relative"
      copy_staged_file "$canonical_target" "$destination"
      jq -cn --arg path "$relative" --arg target "$target_relative" \
        '{path: $path, kind: "internal-file-symlink", target: $target}' >> "$transformations_jsonl"
    elif [[ -d "$entry" ]]; then
      mkdir -p -- "$destination"
      chmod 700 "$destination"
    elif [[ -f "$entry" ]]; then
      copy_staged_file "$entry" "$destination"
    else
      die "skill tree contains an unsupported entry type"
    fi
  done < <(find "$canonical_root" -mindepth 1 -print0)

  jq -s 'sort_by(.path)' "$transformations_jsonl" > "$SYMLINK_TRANSFORMATIONS_FILE"
  chmod 600 "$SYMLINK_TRANSFORMATIONS_FILE"
  rm -f -- "$transformations_jsonl"

  if find "$staged_root" -type l -print -quit | grep -q .; then
    die "staged skill input retained a symlink"
  fi
  enumerate_skill_roots "$staged_root" "$EXPECTED_ROOTS_FILE"
  INPUT_SHA256="$(staged_input_sha256 "$staged_root")"
}

assert_artifact_root_ignored() {
  git -C "$REPO_ROOT" check-ignore -q -- "artifacts/security/skillspector" \
    || die "artifacts/security/skillspector must remain ignored"
}

create_private_artifacts() {
  assert_artifact_root_ignored
  mkdir -p -- "$ARTIFACT_ROOT"
  chmod 700 "$ARTIFACT_ROOT"
  RUN_UTC="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
  RUN_DIR="${ARTIFACT_ROOT}/${RUN_UTC}"
  mkdir "$RUN_DIR" \
    || die "private artifact directory already exists for ${RUN_UTC}; rerun after the next UTC second"
  chmod 700 "$RUN_DIR"

  RAW_REPORT="${RUN_DIR}/report.json"
  SCANNER_LOG="${RUN_DIR}/scanner.log"
  MACHINE_MANIFEST="${RUN_DIR}/manifest.json"
  : > "$RAW_REPORT"
  : > "$SCANNER_LOG"
  : > "$MACHINE_MANIFEST"
  chmod 600 "$RAW_REPORT" "$SCANNER_LOG" "$MACHINE_MANIFEST"
}

container_base_args() {
  local host_uid host_gid

  host_uid="$(id -u)"
  host_gid="$(id -g)"
  [[ "$host_uid" != 0 ]] \
    || die "SkillSpector refuses to run a container as root"

  CONTAINER_BASE_ARGS=(
    --rm
    --network none
    --read-only
    --cap-drop ALL
    --security-opt no-new-privileges
    --user "${host_uid}:${host_gid}"
    --pids-limit "$CONTAINER_PIDS"
    --memory "$CONTAINER_MEMORY"
    --cpus "$CONTAINER_CPUS"
    --tmpfs /tmp:rw,noexec,nosuid,nodev,size=64m
  )
}

run_with_timeout() {
  local timeout_seconds="$1"
  shift

  # `timeout` is not available on stock macOS. Perl is part of the supported
  # host toolchain on macOS/Linux and preserves an alarm across exec so the
  # Docker client has a hard wall-clock bound without leaving watchdog jobs.
  perl -e 'my $seconds = shift @ARGV; alarm $seconds; exec { $ARGV[0] } @ARGV or die "exec failed: $!\n"' \
    "$timeout_seconds" "$@"
}

image_label() {
  local label_name="$1"
  docker image inspect --format "{{ index .Config.Labels \"${label_name}\" }}" "$IMAGE_REF"
}

ensure_local_image() {
  require_command docker

  IMAGE_ID="$(docker image inspect --format '{{.Id}}' "$IMAGE_REF" 2>/dev/null)" \
    || die "pinned SkillSpector image is missing; run scripts/security/skillspector-scan.sh --bootstrap first"
  [[ "$IMAGE_ID" =~ ^sha256:[0-9A-Za-z._:-]+$ ]] \
    || die "pinned SkillSpector image did not provide a content-addressed image ID"

  [[ "$(image_label org.opencontainers.image.version 2>/dev/null)" == "$SKILLSPECTOR_VERSION" ]] \
    || die "local SkillSpector image version label does not match the pin"
  [[ "$(image_label org.opencontainers.image.revision 2>/dev/null)" == "$SKILLSPECTOR_COMMIT" ]] \
    || die "local SkillSpector image commit label does not match the pin"
  [[ "$(image_label org.opencontainers.image.source-archive-sha256 2>/dev/null)" == "$SKILLSPECTOR_ARCHIVE_SHA256" ]] \
    || die "local SkillSpector image archive label does not match the pin"
}

run_isolated_container() {
  container_base_args
  run_with_timeout "$CONTAINER_RUNTIME_SECONDS" \
    docker run "${CONTAINER_BASE_ARGS[@]}" "$IMAGE_REF" "$@"
}

verify_local_cli_contract() {
  local version_output help_output required_flag

  version_output="$(run_isolated_container skillspector --version 2>/dev/null)" \
    || die "local SkillSpector image could not report its version under scan isolation"
  [[ "$version_output" == *"SkillSpector v${SKILLSPECTOR_VERSION}"* ]] \
    || die "local SkillSpector image reported an unexpected version"

  help_output="$(run_isolated_container skillspector scan --help 2>/dev/null)" \
    || die "local SkillSpector image could not show scan help under isolation"
  for required_flag in --recursive --no-llm --fail-on-incomplete --format --output; do
    [[ "$help_output" == *"${required_flag}"* ]] \
      || die "local SkillSpector image lacks required scan flag: ${required_flag}"
  done
}

assess_report() {
  local report_path="$1"
  local expected_roots_file="$2"
  local expected_roots_json

  REPORT_VALID=false
  REPORT_POLICY_REASON="report is missing or malformed"
  CAUTION_PRESENT=false
  SCANNED_ROOTS_JSON="[]"

  [[ -s "$report_path" ]] || return 1
  expected_roots_json="$(jq -ce '
    . as $roots
    | if (
      type == "array"
      and length > 0
      and all(.[]; type == "string" and length > 0)
      and (($roots | unique | length) == ($roots | length))
    ) then . else error("invalid expected roots") end
  ' "$expected_roots_file" 2>/dev/null)" || {
    REPORT_POLICY_REASON="expected skill-root inventory is malformed"
    return 1
  }

  # Preserve the scanner's reported root inventory whenever its JSON is
  # structurally readable, even when another policy condition fails. That
  # keeps the private manifest useful for a nonzero scanner exit without
  # turning that exit into a pass.
  if ! SCANNED_ROOTS_JSON="$(jq -ce '
    if (
      type == "object"
      and (.skills | type == "array")
      and all(.skills[]; (.path | type == "string" and length > 0))
    ) then [.skills[].path] | sort else error("invalid scanned roots") end
  ' "$report_path" 2>/dev/null)"; then
    SCANNED_ROOTS_JSON="[]"
  fi

  if ! jq -e --arg version "$SKILLSPECTOR_VERSION" --argjson expected "$expected_roots_json" '
    def normalized: tostring | ascii_upcase;
    def not_severe:
      ([.risk_severity // "", .risk_assessment.severity // "", .risk_assessment.max_issue_severity // ""]
        | map(normalized)
        | all(. != "HIGH" and . != "CRITICAL"));
    def not_do_not_install:
      ((.risk_recommendation // "") | normalized) != "DO_NOT_INSTALL"
      and ((.risk_assessment.recommendation // "") | normalized) != "DO_NOT_INSTALL";
    def complete_successful:
      (.execution_successful == true)
      and (.analysis_completeness | type == "object")
      and (.analysis_completeness.is_complete == true)
      and (.analysis_completeness.execution_successful == true);
    def static_only:
      (.metadata | type == "object")
      and (.metadata.skillspector_version == $version)
      and (.metadata.llm_requested == false)
      and (.metadata.meta_analysis_applied == false)
      and (.metadata.inference_usage | type == "array" and length == 0)
      and ((.metadata.llm_calls_attempted? // 0) == 0)
      and ((.metadata.llm_calls_succeeded? // 0) == 0)
      and ((.metadata.llm_degraded? // false) == false);
    type == "object"
    and (.multi_skill == true)
    and (.skill_count | type == "number")
    and (.skill_count == ($expected | length))
    and (.skills_scanned | type == "number")
    and (.skills_scanned == ($expected | length))
    and (.skills_omitted == 0)
    and (.risk_recommendation | normalized) != "DO_NOT_INSTALL"
    and complete_successful
    and (.skills | type == "array")
    and (.skills | length == ($expected | length))
    and ([.skills[] | .path] | all(type == "string" and length > 0) and sort == $expected)
    and (all(.skills[];
      type == "object"
      and (.path | type == "string" and length > 0)
      and (.risk_assessment | type == "object")
      and (.issues | type == "array")
      and complete_successful
      and static_only
      and not_severe
      and not_do_not_install
      and ([.issues[] | ((.severity // "") | normalized)] | all(. != "HIGH" and . != "CRITICAL"))
    ))
  ' "$report_path" >/dev/null 2>&1; then
    REPORT_POLICY_REASON="report violated the complete static-only root-coverage policy"
    return 1
  fi

  if jq -e '
    ((.risk_recommendation // "") | tostring | ascii_upcase) == "CAUTION"
    or any(.skills[];
      ((.risk_recommendation // "") | tostring | ascii_upcase) == "CAUTION"
      or ((.risk_assessment.recommendation // "") | tostring | ascii_upcase) == "CAUTION"
    )
  ' "$report_path" >/dev/null 2>&1; then
    CAUTION_PRESENT=true
  fi
  REPORT_VALID=true
  REPORT_POLICY_REASON="complete static-only scan with exact root coverage"
  return 0
}

policy_incomplete() {
  local report_path="$1"

  jq -e '
    type != "object"
    or (.execution_successful != true)
    or (.analysis_completeness | type != "object" or .is_complete != true or .execution_successful != true)
    or any(.skills[]?;
      .execution_successful != true
      or (.analysis_completeness | type != "object" or .is_complete != true or .execution_successful != true)
    )
  ' "$report_path" >/dev/null 2>&1
}

policy_llm_or_inference_detected() {
  local report_path="$1"

  jq -e 'any(.skills[]?;
    (.metadata.llm_requested? == true)
    or (.metadata.meta_analysis_applied? == true)
    or ((.metadata.inference_usage? // []) | type == "array" and length > 0)
    or ((.metadata.llm_calls_attempted? // 0) != 0)
    or ((.metadata.llm_calls_succeeded? // 0) != 0)
    or (.metadata.llm_degraded? == true)
  )' "$report_path" >/dev/null 2>&1
}

# This is intentionally diagnostic-only. A report with this classification is
# still incomplete and therefore cannot be valid or pass the scan. It exists
# only to distinguish the narrow, fully-accounted reference-resolution case
# from generic incomplete analysis without recording report paths or messages.
classify_nonfatal_unresolved_references_only() {
  local report_path="$1"
  local expected_roots_file="$2"
  local expected_roots_json exception_count

  PARTIAL_CLASSIFICATION=""
  PARTIAL_CLASSIFICATION_COUNT=0

  [[ -s "$report_path" ]] || return 0
  expected_roots_json="$(jq -ce '
    . as $roots
    | if (
      type == "array"
      and length == 11
      and all(.[]; type == "string" and length > 0)
      and (($roots | unique | length) == ($roots | length))
    ) then . else error("expected exactly eleven unique roots") end
  ' "$expected_roots_file" 2>/dev/null)" || return 0

  if ! exception_count="$(jq -er --arg version "$SKILLSPECTOR_VERSION" --argjson expected "$expected_roots_json" '
    def nonnegative_integer:
      if type == "number" then . >= 0 and floor == . else false end;
    def static_metadata:
      type == "object"
      and (.skillspector_version == $version)
      and (.llm_requested == false)
      and (.meta_analysis_applied == false)
      and (.inference_usage | type == "array" and length == 0)
      and ((.llm_calls_attempted? // 0) == 0)
      and ((.llm_calls_succeeded? // 0) == 0)
      and ((.llm_degraded? // false) == false);
    def optional_static_metadata:
      if . == null then true else static_metadata end;
    def clean_analyzer:
      if type != "object" then false
      elif (.status | type) != "string" then false
      elif ([.planned_work, .completed, .partial, .skipped, .failed, .unaccounted]
            | all(.[]; nonnegative_integer)) | not then false
      else
        (.status == "completed" or .status == "disabled" or .status == "not_applicable")
        and (.completed == .planned_work)
        and (.partial == 0)
        and (.skipped == 0)
        and (.failed == 0)
        and (.unaccounted == 0)
      end;
    def qualifying_exception:
      type == "object"
      and (.fatal == false)
      and (.phase == "reference_resolution")
      and (.reason_code == "reference_unresolved")
      and (.outcome == "partial");
    def empty_array:
      type == "array" and length == 0;
    def qualified_skill:
      if type != "object" then false
      elif (.analysis_completeness | type) != "object" then false
      elif (.execution_successful != true) then false
      elif (.metadata | static_metadata | not) then false
      else .analysis_completeness as $completeness
      | if ($completeness.execution_successful != true) then false
        elif ($completeness.status != "partial" and $completeness.status != "complete") then false
        elif ([
          $completeness.total_components,
          $completeness.scanned_components,
          $completeness.fully_inspected_files,
          $completeness.partially_inspected_files,
          $completeness.entirely_uninspected_files
        ] | all(.[]; nonnegative_integer)) | not then false
        elif ($completeness.scanned_components != $completeness.total_components) then false
        elif (($completeness.fully_inspected_files + $completeness.partially_inspected_files)
              != $completeness.total_components) then false
        elif ($completeness.entirely_uninspected_files != 0) then false
        elif ($completeness.scope_exclusions | empty_array | not) then false
        elif ($completeness.limitations | empty_array | not) then false
        elif ($completeness.analyzer_statuses | type != "array") then false
        elif (all($completeness.analyzer_statuses[]; clean_analyzer) | not) then false
        elif ($completeness.ledger_exceptions | type != "array") then false
        elif ($completeness.is_complete == true) then
          $completeness.status == "complete"
          and ($completeness.ledger_exceptions | length == 0)
        elif ($completeness.is_complete == false) then
          $completeness.status == "partial"
          and ($completeness.ledger_exceptions | length > 0)
          and all($completeness.ledger_exceptions[]; qualifying_exception)
        else false
        end
      end;
    if type != "object" then error("report is not an object")
    elif (.multi_skill != true or .execution_successful != true) then error("global execution failed")
    elif (.metadata? | optional_static_metadata | not) then error("global LLM metadata is not static-only")
    elif (.analysis_completeness | type) != "object" then error("global completeness is absent")
    elif (.skills | type) != "array" then error("skills inventory is absent")
    elif (.skill_count != 11 or .skills_scanned != 11 or .skills_omitted != 0) then error("root counts differ")
    elif (.skills | length != 11) then error("skill inventory count differs")
    elif ([.skills[].path] | all(type == "string" and length > 0) and sort == ($expected | sort)) | not then error("root inventory differs")
    else .analysis_completeness as $global
    | if ($global.is_complete != false or $global.execution_successful != true or $global.status != "partial") then error("global report is not partial")
      elif ($global.scope != "recursive_skills") then error("global scope differs")
      elif ([
        $global.fully_inspected_files,
        $global.partially_inspected_files,
        $global.entirely_uninspected_files,
        $global.total_files
      ] | all(.[]; nonnegative_integer)) | not then error("global file counts are malformed")
      elif (($global.fully_inspected_files + $global.partially_inspected_files) != $global.total_files) then error("global file counts are incoherent")
      elif ($global.entirely_uninspected_files != 0) then error("global files are uninspected")
      elif ($global.limitations | empty_array | not) then error("global resources are incomplete")
      elif (($global.scope_exclusions? // []) | empty_array | not) then error("global scope is incomplete")
      elif (($global.ledger_exceptions? // []) | type != "array") then error("global exceptions are malformed")
      elif (all(($global.ledger_exceptions? // [])[]; qualifying_exception) | not) then error("global exception is not qualifying")
      elif (all(.skills[]; qualified_skill) | not) then error("per-skill prerequisites differ")
      else [
        ($global.ledger_exceptions? // [])[],
        .skills[].analysis_completeness.ledger_exceptions[]
      ] as $exceptions
      | if ($exceptions | length) == 0 then error("no partial exception")
        elif all($exceptions[]; qualifying_exception) then ($exceptions | length)
        else error("exception is not qualifying")
        end
      end
    end
  ' "$report_path" 2>/dev/null)"; then
    return 0
  fi

  [[ "$exception_count" =~ ^[1-9][0-9]*$ ]] || return 0
  PARTIAL_CLASSIFICATION="nonfatal_unresolved_references_only"
  PARTIAL_CLASSIFICATION_COUNT="$exception_count"
}

write_machine_manifest() {
  local expected_roots transformations report_sha policy_is_incomplete policy_has_llm

  expected_roots="$(jq -ce . "$EXPECTED_ROOTS_FILE")"
  transformations="$(jq -ce . "$SYMLINK_TRANSFORMATIONS_FILE")"
  report_sha="$(sha256_file "$RAW_REPORT")"
  policy_is_incomplete=false
  policy_has_llm=false
  if policy_incomplete "$RAW_REPORT"; then
    policy_is_incomplete=true
  fi
  if policy_llm_or_inference_detected "$RAW_REPORT"; then
    policy_has_llm=true
  fi

  jq -n \
    --arg generated_at_utc "$RUN_UTC" \
    --arg version "$SKILLSPECTOR_VERSION" \
    --arg commit "$SKILLSPECTOR_COMMIT" \
    --arg archive_url "$SKILLSPECTOR_ARCHIVE_URL" \
    --arg archive_sha256 "$SKILLSPECTOR_ARCHIVE_SHA256" \
    --arg base_image "$PYTHON_BASE_IMAGE" \
    --arg image_id "$IMAGE_ID" \
    --arg input_sha256 "$INPUT_SHA256" \
    --arg policy_reason "$REPORT_POLICY_REASON" \
    --arg report_sha256 "$report_sha" \
    --arg partial_classification "$PARTIAL_CLASSIFICATION" \
    --argjson expected_roots "$expected_roots" \
    --argjson scanned_roots "$SCANNED_ROOTS_JSON" \
    --argjson transformations "$transformations" \
    --argjson runtime_limit_seconds "$CONTAINER_RUNTIME_SECONDS" \
    --argjson scanner_exit_code "$SCANNER_EXIT_CODE" \
    --argjson report_valid "$REPORT_VALID" \
    --argjson incomplete "$policy_is_incomplete" \
    --argjson llm_or_inference_detected "$policy_has_llm" \
    --argjson caution_present "$CAUTION_PRESENT" \
    --argjson partial_classification_count "$PARTIAL_CLASSIFICATION_COUNT" \
    --argjson skill_scan_passed "$SKILL_SCAN_PASSED" \
    '{
      schema_version: 1,
      generated_at_utc: $generated_at_utc,
      scanner: {
        name: "NVIDIA SkillSpector",
        version: $version,
        commit: $commit,
        source_archive_url: $archive_url,
        archive_sha256: $archive_sha256,
        base_image: $base_image,
        image_id: $image_id
      },
      input: {
        sha256: $input_sha256,
        expected_roots: $expected_roots,
        scanned_roots: $scanned_roots,
        symlink_transformations: $transformations
      },
      policy: {
        static_only_required: true,
        complete_required: true,
        exact_root_coverage_required: true,
        block_recommendations: ["DO_NOT_INSTALL"],
        block_severities: ["HIGH", "CRITICAL"],
        caution_nonblocking: true,
        runtime_limit_seconds: $runtime_limit_seconds,
        report_valid: $report_valid,
        incomplete: $incomplete,
        llm_or_inference_detected: $llm_or_inference_detected,
        caution_present: $caution_present,
        partial_classification: (if $partial_classification == "" then null else $partial_classification end),
        partial_classification_count: (if $partial_classification == "" then null else $partial_classification_count end),
        reason: $policy_reason
      },
      raw: {
        scanner_exit_code: $scanner_exit_code,
        report_sha256: $report_sha256
      },
      result: {
        skill_scan_passed: $skill_scan_passed
      }
    }' > "$MACHINE_MANIFEST"
  chmod 600 "$MACHINE_MANIFEST"
}

relative_evidence_path() {
  local path="$1"
  printf '%s\n' "${path#"${REPO_ROOT}/"}"
}

run_bootstrap() {
  local archive_path source_dir actual_sha

  require_command curl
  require_command tar
  require_command docker
  require_command perl
  validate_source_manifest
  validate_dockerfile
  make_work_dir
  archive_path="${WORK_DIR}/${SKILLSPECTOR_ARCHIVE_FILENAME}"
  download_to "$SKILLSPECTOR_ARCHIVE_URL" "$archive_path"
  actual_sha="$(sha256_file "$archive_path")"
  [[ "$actual_sha" == "$SKILLSPECTOR_ARCHIVE_SHA256" ]] \
    || die "downloaded SkillSpector source archive checksum did not match the pin"

  source_dir="${WORK_DIR}/source"
  mkdir -p -- "$source_dir"
  chmod 700 "$source_dir"
  tar -xzf "$archive_path" -C "$source_dir" --strip-components=1 \
    || die "failed to extract the verified SkillSpector source archive"
  verify_upstream_source_contract "$source_dir"

  docker build --pull=false --no-cache --tag "$IMAGE_REF" \
    --build-arg "SKILLSPECTOR_VERSION=${SKILLSPECTOR_VERSION}" \
    --build-arg "SKILLSPECTOR_COMMIT=${SKILLSPECTOR_COMMIT}" \
    --build-arg "SKILLSPECTOR_ARCHIVE_SHA256=${SKILLSPECTOR_ARCHIVE_SHA256}" \
    --file "$DOCKERFILE" "$source_dir" \
    || die "failed to build the verified local SkillSpector image"
  ensure_local_image
  verify_local_cli_contract
  printf 'SkillSpector %s bootstrap completed with local image %s.\n' \
    "$SKILLSPECTOR_VERSION" "$IMAGE_ID"
}

run_self_check() {
  require_command perl
  validate_source_manifest
  validate_dockerfile
  ensure_local_image
  verify_local_cli_contract
  printf 'SkillSpector %s local image and CLI contract self-check passed.\n' "$SKILLSPECTOR_VERSION"
}

run_scan() {
  validate_source_manifest
  validate_dockerfile
  require_command docker
  require_command git
  require_command jq
  require_command perl
  assert_artifact_root_ignored
  ensure_local_image
  make_work_dir
  STAGED_ROOT="${WORK_DIR}/scan"
  stage_skill_tree "$SKILLS_ROOT" "$STAGED_ROOT"
  create_private_artifacts
  verify_local_cli_contract

  container_base_args
  SCANNER_EXIT_CODE=0
  run_with_timeout "$CONTAINER_RUNTIME_SECONDS" \
    docker run "${CONTAINER_BASE_ARGS[@]}" \
      --mount "type=bind,src=${STAGED_ROOT},dst=/scan,readonly" \
      --mount "type=bind,src=${RUN_DIR},dst=/out" \
      "$IMAGE_REF" \
      skillspector scan /scan --recursive --no-llm --fail-on-incomplete \
      --format json --output /out/report.json \
      > "$SCANNER_LOG" 2>&1 || SCANNER_EXIT_CODE=$?
  chmod 600 "$RAW_REPORT" "$SCANNER_LOG"

  # Assess a readable report even when the scanner itself exits nonzero so the
  # manifest retains its safe root inventory and policy diagnosis. The final
  # pass still requires both a zero scanner exit and a complete policy pass.
  assess_report "$RAW_REPORT" "$EXPECTED_ROOTS_FILE" || true
  classify_nonfatal_unresolved_references_only "$RAW_REPORT" "$EXPECTED_ROOTS_FILE"
  if [[ "$SCANNER_EXIT_CODE" == 0 && "$REPORT_VALID" == true ]]; then
    SKILL_SCAN_PASSED=true
  else
    SKILL_SCAN_PASSED=false
  fi
  write_machine_manifest

  if [[ "$SKILL_SCAN_PASSED" == true ]]; then
    if [[ "$CAUTION_PRESENT" == true ]]; then
      printf 'CAUTION: SkillSpector reported CAUTION-level static findings; the initial policy permits CAUTION but requires review.\n' >&2
    fi
    printf 'PASS: SkillSpector %s scanned all staged skill roots without blocking findings.\n' "$SKILLSPECTOR_VERSION"
    printf 'Private evidence: %s and %s\n' \
      "$(relative_evidence_path "$RAW_REPORT")" \
      "$(relative_evidence_path "$MACHINE_MANIFEST")"
    return 0
  fi

  printf 'ERROR: SkillSpector scan did not pass; raw findings remain private.\n' >&2
  printf 'Private evidence: %s and %s\n' \
    "$(relative_evidence_path "$RAW_REPORT")" \
    "$(relative_evidence_path "$MACHINE_MANIFEST")" >&2
  return 1
}

main() {
  if [[ $# -gt 1 ]]; then
    usage >&2
    return 2
  fi
  if [[ $# -eq 1 ]]; then
    case "$1" in
      --bootstrap)
        MODE="bootstrap"
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

  trap cleanup EXIT HUP INT TERM
  case "$MODE" in
    bootstrap)
      run_bootstrap
      ;;
    self-check)
      run_self_check
      ;;
    scan)
      run_scan
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
