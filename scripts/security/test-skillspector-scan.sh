#!/usr/bin/env bash
# Behavioral tests for the pinned, container-only SkillSpector wrapper.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/security/skillspector-scan.sh"
DOCKERFILE="$ROOT_DIR/scripts/security/skillspector/Dockerfile"
SOURCE_MANIFEST="$ROOT_DIR/scripts/security/skillspector/source-v2.11.0.sha256"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/petri-skillspector-test.XXXXXX")"
ARTIFACT_ROOT="$ROOT_DIR/artifacts/security/skillspector"
ARTIFACT_RUN=""
ARTIFACT_RUNS=()

cleanup() {
  local artifact
  for artifact in "${ARTIFACT_RUNS[@]}"; do
    [[ -d "$artifact" ]] && rm -rf -- "$artifact"
  done
  rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT HUP INT TERM

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

source_tree_hash() {
  local root="$1"
  local list="$TMP_DIR/tree-hash-list"

  : > "$list"
  while IFS= read -r -d '' path; do
    local relative="${path#"${root}/"}"
    [[ "$relative" != *$'\n'* && "$relative" != *$'\r'* ]] \
      || fail "test fixture tree uses an unsupported newline path"
    if [[ -L "$path" ]]; then
      printf 'L %s %s\n' "$relative" "$(readlink "$path")" >> "$list"
    elif [[ -f "$path" ]]; then
      printf 'F %s %s\n' "$relative" "$(portable_sha256_file "$path")" >> "$list"
    elif [[ -d "$path" ]]; then
      printf 'D %s\n' "$relative" >> "$list"
    else
      printf 'X %s\n' "$relative" >> "$list"
    fi
  done < <(find "$root" -mindepth 1 -print0)
  LC_ALL=C sort "$list" | {
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 | awk '{print $1}'
    else
      sha256sum | awk '{print $1}'
    fi
  }
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "expected file is missing: ${path}"
}

require_file "$SCRIPT"
require_file "$DOCKERFILE"
require_file "$SOURCE_MANIFEST"
bash -n "$SCRIPT"

source "$SCRIPT"

validate_source_manifest
validate_dockerfile

assert_equal \
  '1370abe7a76dbe90d6bddf7d1e0353cc58ae9442eece3b03aa7ba75830210dfc' \
  "$(awk 'NF && $1 !~ /^#/ { print $1 }' "$SOURCE_MANIFEST")" \
  'pinned SkillSpector source archive digest'

fixture_root="$TMP_DIR/source-skills"
mkdir -p "$fixture_root/alpha" "$fixture_root/beta"
printf '%s\n' '# Alpha skill' > "$fixture_root/alpha/SKILL.md"
printf '%s\n' '# Beta skill' > "$fixture_root/beta/SKILL.md"
printf '%s\n' 'shared internal fixture' > "$fixture_root/alpha/shared.md"
ln -s ../alpha/shared.md "$fixture_root/beta/internal-alias.md"
fixture_before="$(source_tree_hash "$fixture_root")"

stage_root="$TMP_DIR/staged-skills"
stage_skill_tree "$fixture_root" "$stage_root"
assert_file_mode "$stage_root" 700
[[ -f "$stage_root/beta/internal-alias.md" && ! -L "$stage_root/beta/internal-alias.md" ]] \
  || fail 'validated internal symlink was not materialized as a regular file'
assert_equal \
  "$(portable_sha256_file "$fixture_root/alpha/shared.md")" \
  "$(portable_sha256_file "$stage_root/beta/internal-alias.md")" \
  'materialized internal alias content'
assert_equal "$fixture_before" "$(source_tree_hash "$fixture_root")" 'staging changed source input'
jq -e '
  . == [
    {path: "beta/internal-alias.md", kind: "internal-file-symlink", target: "alpha/shared.md"}
  ]
' "$SYMLINK_TRANSFORMATIONS_FILE" >/dev/null \
  || fail 'internal symlink transformation manifest is incorrect'
jq -e '. == ["alpha", "beta"]' "$EXPECTED_ROOTS_FILE" >/dev/null \
  || fail 'actual skill roots were not fully inventoried'

external_root="$TMP_DIR/external-skills"
external_target="$TMP_DIR/external-target"
mkdir -p "$external_root/escape"
printf '%s\n' '# Escape skill' > "$external_root/escape/SKILL.md"
printf '%s\n' 'this file must not be read' > "$external_target"
ln -s "$external_target" "$external_root/escape/outside.md"
if external_output="$(stage_skill_tree "$external_root" "$TMP_DIR/external-stage" 2>&1)"; then
  fail 'external symlink unexpectedly staged'
fi
[[ "$external_output" == *'resolves outside canonical skills root'* ]] \
  || fail "external symlink rejection did not occur before target copy: ${external_output}"
[[ ! -e "$TMP_DIR/external-stage/escape/outside.md" ]] \
  || fail 'external symlink target was materialized'

roots_file="$TMP_DIR/expected-roots.json"
printf '%s\n' '["alpha","beta"]' > "$roots_file"

write_report() {
  local name="$1"
  local jq_filter="$2"
  local report="$TMP_DIR/${name}.json"

  jq -n '
    {
      multi_skill: true,
      skill_count: 2,
      skills_scanned: 2,
      skills_omitted: 0,
      execution_successful: true,
      risk_recommendation: "SAFE",
      analysis_completeness: {is_complete: true, execution_successful: true},
      skills: [
        {
          name: "alpha", path: "alpha", risk_score: 0, risk_severity: "LOW",
          risk_recommendation: "SAFE",
          risk_assessment: {score: 0, severity: "LOW", recommendation: "SAFE", max_issue_severity: null},
          issues: [], execution_successful: true,
          analysis_completeness: {is_complete: true, execution_successful: true},
          metadata: {skillspector_version: "2.11.0", llm_requested: false, meta_analysis_applied: false, inference_usage: []}
        },
        {
          name: "beta", path: "beta", risk_score: 0, risk_severity: "LOW",
          risk_recommendation: "SAFE",
          risk_assessment: {score: 0, severity: "LOW", recommendation: "SAFE", max_issue_severity: null},
          issues: [], execution_successful: true,
          analysis_completeness: {is_complete: true, execution_successful: true},
          metadata: {skillspector_version: "2.11.0", llm_requested: false, meta_analysis_applied: false, inference_usage: []}
        }
      ]
    }
  ' | jq "$jq_filter" > "$report"
  printf '%s\n' "$report"
}

assert_report_passes() {
  local name="$1"
  local report="$2"

  if ! assess_report "$report" "$roots_file"; then
    fail "expected report to pass policy (${name}): ${REPORT_POLICY_REASON}"
  fi
}

assert_report_fails() {
  local name="$1"
  local report="$2"

  if assess_report "$report" "$roots_file"; then
    fail "expected report to fail policy: ${name}"
  fi
}

clean_report="$(write_report clean '.')"
assert_report_passes clean "$clean_report"
assert_equal false "$CAUTION_PRESENT" 'clean report caution status'

caution_report="$(write_report caution '.skills[0].risk_severity = "MEDIUM" | .skills[0].risk_assessment.recommendation = "CAUTION"')"
assert_report_passes caution "$caution_report"
assert_equal true "$CAUTION_PRESENT" 'caution report must remain visible in policy state'

assert_report_fails malformed "$(write_report malformed '{not: "a valid recursive report"}')"
assert_report_fails incomplete "$(write_report incomplete '.analysis_completeness.is_complete = false')"
assert_report_fails missing-root "$(write_report missing-root '.skills = [.skills[0]] | .skill_count = 1 | .skills_scanned = 1')"
assert_report_fails duplicate-root "$(write_report duplicate-root '.skills[1].path = "alpha"')"
assert_report_fails wrong-version "$(write_report wrong-version '.skills[0].metadata.skillspector_version = "2.11.1"')"
assert_report_fails do-not-install "$(write_report do-not-install '.risk_recommendation = "DO_NOT_INSTALL"')"
assert_report_fails high "$(write_report high '.skills[0].risk_severity = "HIGH"')"
assert_report_fails critical "$(write_report critical '.skills[0].issues = [{severity: "CRITICAL"}]')"
assert_report_fails llm "$(write_report llm '.skills[0].metadata.llm_requested = true')"
assert_report_fails inference "$(write_report inference '.skills[0].metadata.inference_usage = [{provider: "fixture"}]')"

classification_roots_file="$TMP_DIR/classification-roots.json"
jq -n '[range(1; 12) | "skill-\(.)"]' > "$classification_roots_file"

write_partial_classification_report() {
  local name="$1"
  local report_roots_file="$2"
  local jq_filter="$3"
  local report="$TMP_DIR/${name}.json"

  jq -n --argjson roots "$(jq -c . "$report_roots_file")" '
    def analyzer: {
      analyzer_id: "static",
      status: "completed",
      planned_work: 1,
      completed: 1,
      partial: 0,
      skipped: 0,
      failed: 0,
      unaccounted: 0
    };
    def exception: {
      fatal: false,
      phase: "reference_resolution",
      reason_code: "reference_unresolved",
      outcome: "partial"
    };
    {
      multi_skill: true,
      skill_count: ($roots | length),
      skills_scanned: ($roots | length),
      skills_omitted: 0,
      execution_successful: true,
      risk_recommendation: "SAFE",
      analysis_completeness: {
        is_complete: false,
        execution_successful: true,
        status: "partial",
        coverage_percent: 100,
        fully_inspected_files: 0,
        partially_inspected_files: ($roots | length),
        entirely_uninspected_files: 0,
        total_files: ($roots | length),
        limitations: [],
        scope: "recursive_skills"
      },
      skills: ($roots | map({
        name: ., path: ., risk_score: 0, risk_severity: "LOW",
        risk_recommendation: "SAFE",
        risk_assessment: {score: 0, severity: "LOW", recommendation: "SAFE", max_issue_severity: null},
        issues: [], execution_successful: true,
        analysis_completeness: {
          total_components: 1,
          scanned_components: 1,
          coverage_percent: 100,
          is_complete: false,
          status: "partial",
          execution_successful: true,
          fully_inspected_files: 1,
          partially_inspected_files: 0,
          entirely_uninspected_files: 0,
          ledger_exceptions: [exception],
          scope_exclusions: [],
          analyzer_statuses: [analyzer],
          references: [],
          limitations: [],
          findings_before_filtering: 0,
          findings_after_filtering: 0
        },
        metadata: {skillspector_version: "2.11.0", llm_requested: false, meta_analysis_applied: false, inference_usage: []}
      }))
    }
  ' | jq "$jq_filter" > "$report"
  printf '%s\n' "$report"
}

assert_partial_classification() {
  local name="$1"
  local report="$2"
  local report_roots_file="$3"

  if assess_report "$report" "$report_roots_file"; then
    fail "partial diagnostic unexpectedly made report pass: ${name}"
  fi
  assert_equal false "$REPORT_VALID" "partial diagnostic report validity (${name})"
  classify_nonfatal_unresolved_references_only "$report" "$report_roots_file"
  assert_equal nonfatal_unresolved_references_only "$PARTIAL_CLASSIFICATION" "partial classification (${name})"
  assert_equal 11 "$PARTIAL_CLASSIFICATION_COUNT" "partial classification count (${name})"
}

assert_no_partial_classification() {
  local name="$1"
  local report="$2"
  local report_roots_file="$3"

  classify_nonfatal_unresolved_references_only "$report" "$report_roots_file"
  assert_equal "" "$PARTIAL_CLASSIFICATION" "unexpected partial classification (${name})"
  assert_equal 0 "$PARTIAL_CLASSIFICATION_COUNT" "unexpected partial classification count (${name})"
}

qualifying_partial_report="$(write_partial_classification_report qualifying-partial "$classification_roots_file" '.')"
assert_partial_classification qualifying "$qualifying_partial_report" "$classification_roots_file"
assert_no_partial_classification fatal-exception "$(write_partial_classification_report fatal-exception "$classification_roots_file" '.skills[0].analysis_completeness.ledger_exceptions[0].fatal = true')" "$classification_roots_file"
assert_no_partial_classification different-reason "$(write_partial_classification_report different-reason "$classification_roots_file" '.skills[0].analysis_completeness.ledger_exceptions[0].reason_code = "reference_extraction_limit"')" "$classification_roots_file"
assert_no_partial_classification analyzer-failed "$(write_partial_classification_report analyzer-failed "$classification_roots_file" '.skills[0].analysis_completeness.analyzer_statuses[0].failed = 1 | .skills[0].analysis_completeness.analyzer_statuses[0].status = "failed"')" "$classification_roots_file"
assert_no_partial_classification analyzer-skipped "$(write_partial_classification_report analyzer-skipped "$classification_roots_file" '.skills[0].analysis_completeness.analyzer_statuses[0].skipped = 1 | .skills[0].analysis_completeness.analyzer_statuses[0].status = "skipped"')" "$classification_roots_file"
assert_no_partial_classification resource-limit "$(write_partial_classification_report resource-limit "$classification_roots_file" '.skills[0].analysis_completeness.limitations = [{kind: "resource_limit"}]')" "$classification_roots_file"
assert_no_partial_classification uninspected-file "$(write_partial_classification_report uninspected-file "$classification_roots_file" '.skills[0].analysis_completeness.entirely_uninspected_files = 1')" "$classification_roots_file"
assert_no_partial_classification llm-enabled "$(write_partial_classification_report llm-enabled "$classification_roots_file" '.skills[0].metadata.llm_requested = true')" "$classification_roots_file"

actual_roots_file="$TMP_DIR/actual-roots.json"
enumerate_skill_roots "$ROOT_DIR/.claude/skills" "$actual_roots_file"
actual_clean_report="$TMP_DIR/actual-clean.json"
jq -n --argjson roots "$(jq -c . "$actual_roots_file")" '
  {
    multi_skill: true,
    skill_count: ($roots | length),
    skills_scanned: ($roots | length),
    skills_omitted: 0,
    execution_successful: true,
    risk_recommendation: "SAFE",
    analysis_completeness: {is_complete: true, execution_successful: true},
    skills: ($roots | map({
      name: ., path: ., risk_score: 0, risk_severity: "LOW",
      risk_recommendation: "SAFE",
      risk_assessment: {score: 0, severity: "LOW", recommendation: "SAFE", max_issue_severity: null},
      issues: [], execution_successful: true,
      analysis_completeness: {is_complete: true, execution_successful: true},
      metadata: {skillspector_version: "2.11.0", llm_requested: false, meta_analysis_applied: false, inference_usage: []}
    }))
  }
' > "$actual_clean_report"

actual_partial_report="$(write_partial_classification_report actual-partial "$actual_roots_file" '.')"

fake_bin="$TMP_DIR/fake-bin"
fake_log="$TMP_DIR/fake-docker.log"
mkdir -p "$fake_bin"

cat > "$fake_bin/docker" <<'FAKE_DOCKER'
#!/usr/bin/env bash
set -euo pipefail
jq -cn '$ARGS.positional' --args -- "$@" >> "$FAKE_DOCKER_LOG"

if [[ "${1:-}" == image && "${2:-}" == inspect ]]; then
  if [[ "${FAKE_DOCKER_IMAGE_PRESENT:-true}" != true ]]; then
    exit 1
  fi
  case "${3:-}" in
    --format)
      case "${4:-}" in
        '{{.Id}}') printf '%s\n' 'sha256:fixture-image-id' ;;
        *org.opencontainers.image.version*) printf '%s\n' '2.11.0' ;;
        *org.opencontainers.image.revision*) printf '%s\n' 'b7241089d7ec15d8b30df980dacbb428214732b9' ;;
        *org.opencontainers.image.source-archive-sha256*) printf '%s\n' '1370abe7a76dbe90d6bddf7d1e0353cc58ae9442eece3b03aa7ba75830210dfc' ;;
        *) exit 64 ;;
      esac
      ;;
    *) exit 64 ;;
  esac
  exit 0
fi

if [[ "${1:-}" == run ]]; then
  if [[ " $* " == *' skillspector --version '* ]]; then
    printf '%s\n' 'SkillSpector v2.11.0'
    exit 0
  fi
  if [[ " $* " == *' skillspector scan --help '* ]]; then
    printf '%s\n' '--recursive --no-llm --fail-on-incomplete --format --output'
    exit 0
  fi
  scan_mount=""
  out_mount=""
  previous=""
  for arg in "$@"; do
    if [[ "$previous" == --mount ]]; then
      case "$arg" in
        type=bind,src=*,dst=/scan,readonly)
          scan_mount="${arg#type=bind,src=}"
          scan_mount="${scan_mount%,dst=/scan,readonly}"
          ;;
        type=bind,src=*,dst=/out)
          out_mount="${arg#type=bind,src=}"
          out_mount="${out_mount%,dst=/out}"
          ;;
      esac
    fi
    previous="$arg"
  done
  [[ -n "$scan_mount" && -n "$out_mount" ]] || exit 65
  [[ -f "$FAKE_DOCKER_REPORT" ]] || exit 66
  cp "$FAKE_DOCKER_REPORT" "$out_mount/report.json"
  exit "${FAKE_DOCKER_SCAN_EXIT:-0}"
fi

if [[ "${1:-}" == build ]]; then
  exit 0
fi

exit 64
FAKE_DOCKER
chmod 700 "$fake_bin/docker"

cat > "$fake_bin/curl" <<'FAKE_CURL'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl %q\n' "$@" >> "$FAKE_CURL_LOG"
exit 97
FAKE_CURL
chmod 700 "$fake_bin/curl"

: > "$fake_log"
: > "$TMP_DIR/fake-curl.log"
if missing_output="$(
  PATH="$fake_bin:$PATH" \
  FAKE_DOCKER_LOG="$fake_log" \
  FAKE_CURL_LOG="$TMP_DIR/fake-curl.log" \
  FAKE_DOCKER_IMAGE_PRESENT=false \
  "$SCRIPT" 2>&1
)"; then
  fail 'default scan unexpectedly passed without a local image'
fi
[[ "$missing_output" == *'--bootstrap first'* ]] || fail 'missing-image failure was not explicit'
[[ ! -s "$TMP_DIR/fake-curl.log" ]] || fail 'default scan fetched unexpectedly'
if jq -e 'select(.[0] == "build")' "$fake_log" >/dev/null; then
  fail 'default scan built an image unexpectedly'
fi

source_before="$(source_tree_hash "$ROOT_DIR/.claude/skills")"
: > "$fake_log"
FAKE_DOCKER_REPORT="$actual_clean_report" \
FAKE_DOCKER_LOG="$fake_log" \
FAKE_CURL_LOG="$TMP_DIR/fake-curl.log" \
FAKE_DOCKER_IMAGE_PRESENT=true \
NVIDIA_INFERENCE_KEY='must-not-be-forwarded' \
OPENAI_API_KEY='must-not-be-forwarded' \
PATH="$fake_bin:$PATH" \
"$SCRIPT" >/dev/null
source_after="$(source_tree_hash "$ROOT_DIR/.claude/skills")"
assert_equal "$source_before" "$source_after" 'default scan changed the source skills tree'

run_args="$(tail -n 1 "$fake_log")"
jq -e '
  def has_pair($option; $value):
    . as $argv
    | any(range(0; ($argv | length) - 1);
        . as $index | $argv[$index] == $option and $argv[$index + 1] == $value);
  def mount_values:
    . as $argv
    | [range(0; ($argv | length) - 1) as $index
       | select($argv[$index] == "--mount") | $argv[$index + 1]];
  .[0] == "run"
  and has_pair("--network"; "none")
  and index("--read-only") != null
  and has_pair("--cap-drop"; "ALL")
  and has_pair("--security-opt"; "no-new-privileges")
  and has_pair("--pids-limit"; "128")
  and has_pair("--memory"; "1g")
  and has_pair("--cpus"; "1.0")
  and has_pair("--tmpfs"; "/tmp:rw,noexec,nosuid,nodev,size=64m")
  and (mount_values | length == 2)
  and (mount_values | any(test("^type=bind,src=.+,dst=/scan,readonly$")))
  and (mount_values | any(test("^type=bind,src=.+,dst=/out$")))
  and (index("-v") | not)
  and (. as $argv | any(range(0; ($argv | length) - 1);
    . as $index | $argv[$index] == "--user" and ($argv[$index + 1] | test("^[1-9][0-9]*:[0-9]+$"))))
  and all(.[]; . != "--env" and . != "--env-file" and . != "-e")
  and .[-10:] == [
    "skillspector", "scan", "/scan", "--recursive", "--no-llm", "--fail-on-incomplete",
    "--format", "json", "--output", "/out/report.json"
  ]
' <<< "$run_args" >/dev/null || fail 'fake Docker did not receive the exact isolated static-only scan invocation'

ARTIFACT_RUN="$(find "$ARTIFACT_ROOT" -mindepth 1 -maxdepth 1 -type d -print | LC_ALL=C sort | tail -n 1)"
[[ -n "$ARTIFACT_RUN" ]] || fail 'default scan did not create private evidence'
ARTIFACT_RUNS+=("$ARTIFACT_RUN")
assert_file_mode "$ARTIFACT_RUN" 700
assert_file_mode "$ARTIFACT_RUN/report.json" 600
assert_file_mode "$ARTIFACT_RUN/manifest.json" 600
git -C "$ROOT_DIR" check-ignore -q -- "${ARTIFACT_RUN#"$ROOT_DIR/"}" \
  || fail 'SkillSpector artifact directory is not ignored'
jq -e --arg root "$ROOT_DIR" '
  .scanner.version == "2.11.0"
  and .scanner.commit == "b7241089d7ec15d8b30df980dacbb428214732b9"
  and .scanner.archive_sha256 == "1370abe7a76dbe90d6bddf7d1e0353cc58ae9442eece3b03aa7ba75830210dfc"
  and .scanner.image_id == "sha256:fixture-image-id"
  and .input.expected_roots == ["feature-1-capture", "feature-2-refine", "feature-3-plan", "feature-4-implement", "feature-5-resume", "feature-6-complete", "feature-7-cancel", "frontend-design", "rust-skills", "vercel-composition-patterns", "vercel-react-best-practices"]
  and .input.scanned_roots == .input.expected_roots
  and (.input.sha256 | test("^[0-9a-f]{64}$"))
  and (.input.symlink_transformations | type == "array")
  and .policy.llm_or_inference_detected == false
  and .policy.incomplete == false
  and .policy.partial_classification == null
  and .policy.partial_classification_count == null
  and .raw.scanner_exit_code == 0
  and (.raw.report_sha256 | test("^[0-9a-f]{64}$"))
  and .result.skill_scan_passed == true
  and (tostring | contains($root) | not)
  and (tostring | contains("must-not-be-forwarded") | not)
' "$ARTIFACT_RUN/manifest.json" >/dev/null || fail 'manifest semantics or privacy boundary is incorrect'

# A report that otherwise satisfies policy still cannot turn a nonzero scanner
# process into a pass. Wait one second so the test-only evidence directory name
# cannot collide with the preceding UTC-second run.
sleep 1
if nonzero_output="$(
  FAKE_DOCKER_REPORT="$actual_clean_report" \
  FAKE_DOCKER_LOG="$fake_log" \
  FAKE_CURL_LOG="$TMP_DIR/fake-curl.log" \
  FAKE_DOCKER_IMAGE_PRESENT=true \
  FAKE_DOCKER_SCAN_EXIT=1 \
  PATH="$fake_bin:$PATH" \
  "$SCRIPT" 2>&1
)"; then
  fail 'default scan unexpectedly passed when the scanner exited nonzero'
fi
[[ "$nonzero_output" == *'scan did not pass'* ]] \
  || fail 'nonzero scanner exit did not produce the fail-closed result'
failed_artifact="$(find "$ARTIFACT_ROOT" -mindepth 1 -maxdepth 1 -type d -print | LC_ALL=C sort | tail -n 1)"
[[ "$failed_artifact" != "$ARTIFACT_RUN" ]] \
  || fail 'nonzero scanner test reused the preceding evidence directory'
ARTIFACT_RUNS+=("$failed_artifact")
jq -e '
  .raw.scanner_exit_code == 1
  and .input.scanned_roots == .input.expected_roots
  and .policy.report_valid == true
  and .policy.reason == "complete static-only scan with exact root coverage"
  and .policy.partial_classification == null
  and .policy.partial_classification_count == null
  and .result.skill_scan_passed == false
' "$failed_artifact/manifest.json" >/dev/null \
  || fail 'nonzero scanner result was not recorded as a failed scan'

# Even a fully accounted, nonfatal reference-resolution partial report is not
# a passing scan. The diagnostic manifest fields must remain path-free and
# cannot override report_valid or skill_scan_passed.
sleep 1
if partial_output="$(
  FAKE_DOCKER_REPORT="$actual_partial_report" \
  FAKE_DOCKER_LOG="$fake_log" \
  FAKE_CURL_LOG="$TMP_DIR/fake-curl.log" \
  FAKE_DOCKER_IMAGE_PRESENT=true \
  FAKE_DOCKER_SCAN_EXIT=0 \
  NVIDIA_INFERENCE_KEY='must-not-be-forwarded' \
  PATH="$fake_bin:$PATH" \
  "$SCRIPT" 2>&1
)"; then
  fail 'qualifying partial report unexpectedly passed the wrapper'
fi
[[ "$partial_output" == *'scan did not pass'* ]] \
  || fail 'qualifying partial report did not retain the fail-closed result'
partial_artifact="$(find "$ARTIFACT_ROOT" -mindepth 1 -maxdepth 1 -type d -print | LC_ALL=C sort | tail -n 1)"
[[ "$partial_artifact" != "$failed_artifact" ]] \
  || fail 'qualifying partial report reused the preceding evidence directory'
ARTIFACT_RUNS+=("$partial_artifact")
jq -e --arg root "$ROOT_DIR" '
  .raw.scanner_exit_code == 0
  and .input.scanned_roots == .input.expected_roots
  and .policy.incomplete == true
  and .policy.report_valid == false
  and .policy.partial_classification == "nonfatal_unresolved_references_only"
  and .policy.partial_classification_count == 11
  and .result.skill_scan_passed == false
  and (tostring | contains($root) | not)
  and (tostring | contains("must-not-be-forwarded") | not)
' "$partial_artifact/manifest.json" >/dev/null \
  || fail 'partial diagnostic semantics, fail-closed state, or privacy boundary is incorrect'

echo 'SkillSpector scan tooling tests passed'
