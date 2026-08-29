#!/usr/bin/env bash
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/petri-architecture-harness.XXXXXX")"
trap 'rm -rf "$TEST_ROOT"' EXIT

mkdir -p \
  "$TEST_ROOT/scripts" \
  "$TEST_ROOT/docs/standards" \
  "$TEST_ROOT/crates/v3-cli/src" \
  "$TEST_ROOT/crates/v3-core/src/domain/tests"
cp "$SOURCE_ROOT/scripts/check-architecture-harness.sh" "$TEST_ROOT/scripts/"
cp "$SOURCE_ROOT/scripts/check_cargo_policy.py" "$TEST_ROOT/scripts/"

printf '%s\n' \
  '[workspace]' \
  'members = ["crates/*"]' \
  'resolver = "2"' \
  '' \
  '[workspace.package]' \
  'rust-version = "1.93"' \
  '' \
  '[workspace.lints.rust]' \
  'unsafe_code = "deny"' \
  'unexpected_cfgs = "warn"' \
  '' \
  '[workspace.lints.clippy]' \
  'correctness = { level = "deny", priority = -1 }' \
  'suspicious = { level = "deny", priority = -1 }' \
  '' \
  '[workspace.lints.rustdoc]' \
  'broken_intra_doc_links = "deny"' \
  > "$TEST_ROOT/Cargo.toml"

for crate_name in v3-cli v3-core; do
  printf '%s\n' \
    '[package]' \
    "name = \"$crate_name\"" \
    'version = "0.1.0"' \
    'edition = "2021"' \
    'rust-version.workspace = true' \
    '' \
    '[lints]' \
    'workspace = true' \
    > "$TEST_ROOT/crates/$crate_name/Cargo.toml"
done

printf '%s\n' 'pub struct Legacy;' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
printf '%s\n' 'pub mod domain;' > "$TEST_ROOT/crates/v3-core/src/lib.rs"
yes '// production baseline' | head -n 401 \
  > "$TEST_ROOT/crates/v3-core/src/domain/large.rs" || true
yes '// nested test fixture' | head -n 650 \
  > "$TEST_ROOT/crates/v3-core/src/domain/tests/large_test.rs" || true
yes '// conventional test fixture' | head -n 650 \
  > "$TEST_ROOT/crates/v3-core/src/domain/test_helper.rs" || true

fingerprint() {
  if command -v shasum >/dev/null 2>&1; then
    printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha256sum | awk '{print $1}'
  else
    echo "missing SHA-256 tool for architecture harness test" >&2
    return 1
  fi
}

declaration_hash="$(fingerprint 'pub struct Legacy;')"
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$declaration_hash" \
  > "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'oversize\t%s\t%s\n' \
  'crates/v3-core/src/domain/large.rs' '401' \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"

run_strict() {
  (cd "$TEST_ROOT" && bash scripts/check-architecture-harness.sh --mode strict)
}

expect_failure() {
  local label="$1"
  if run_strict > "$TEST_ROOT/failure.txt" 2>&1; then
    echo "expected architecture harness failure: $label"
    exit 1
  fi
}

# Exact known debt passes, and nested/conventional test files are not production debt.
run_strict >/dev/null

# A production module merely beginning with "test" remains ratcheted as new debt.
yes '// testing production fixture' | head -n 401 \
  > "$TEST_ROOT/crates/v3-core/src/domain/testing.rs" || true
expect_failure 'testing.rs production debt'
grep -q 'new oversized production file: crates/v3-core/src/domain/testing.rs (401 lines)' \
  "$TEST_ROOT/failure.txt"
rm -f "$TEST_ROOT/crates/v3-core/src/domain/testing.rs"

# The established *_tests.rs convention remains excluded without using a broad test*.rs glob.
yes '// suffix test fixture' | head -n 650 \
  > "$TEST_ROOT/crates/v3-core/src/domain/unit_tests.rs" || true
run_strict >/dev/null
rm -f "$TEST_ROOT/crates/v3-core/src/domain/unit_tests.rs"

# Whitespace-only declaration changes do not churn the normalized fingerprint.
printf '%s\n' '  pub   struct   Legacy;' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
run_strict >/dev/null

# Moving a declaration without changing it does not churn the fingerprint.
printf '\n%s\n' 'pub struct Legacy;' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
run_strict >/dev/null

# Declaration entries are an exact multiset, including legitimate duplicate impl blocks.
impl_hash="$(fingerprint 'impl Legacy {}')"
printf '%s\n' 'pub struct Legacy;' 'impl Legacy {}' 'impl Legacy {}' \
  > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$declaration_hash" \
  > "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$impl_hash" \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$impl_hash" \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'oversize\t%s\t%s\n' \
  'crates/v3-core/src/domain/large.rs' '401' \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
run_strict >/dev/null

printf '%s\n' 'pub struct Legacy;' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$declaration_hash" \
  > "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'oversize\t%s\t%s\n' \
  'crates/v3-core/src/domain/large.rs' '401' \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"

# A duplicated oversize row is invalid: one production file has one size record.
printf 'oversize\t%s\t%s\n' \
  'crates/v3-core/src/domain/large.rs' '401' \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
expect_failure 'duplicate oversized baseline entry'
grep -q 'duplicate oversized production baseline entry' "$TEST_ROOT/failure.txt"

printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$declaration_hash" \
  > "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
printf 'oversize\t%s\t%s\n' \
  'crates/v3-core/src/domain/large.rs' '401' \
  >> "$TEST_ROOT/docs/standards/architecture-baseline.tsv"

# A same-count declaration substitution must not evade the ratchet.
printf '%s\n' 'pub enum Replacement { Value }' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
expect_failure 'declaration substitution'

# An added declaration must fail even when existing debt remains unchanged.
printf '%s\n' 'pub struct Legacy;' 'pub fn added() {}' \
  > "$TEST_ROOT/crates/v3-cli/src/lib.rs"
expect_failure 'declaration addition'

printf '%s\n' 'pub struct Legacy;' > "$TEST_ROOT/crates/v3-cli/src/lib.rs"

# Oversized production files cannot grow beyond the recorded maximum.
printf '%s\n' '// growth' >> "$TEST_ROOT/crates/v3-core/src/domain/large.rs"
expect_failure 'oversize growth'

# Improvements require lowering the baseline rather than leaving stale debt entries.
sed -i.bak '$d' "$TEST_ROOT/crates/v3-core/src/domain/large.rs"
sed -i.bak '$d' "$TEST_ROOT/crates/v3-core/src/domain/large.rs"
expect_failure 'stale oversize maximum after reduction'

# Removing the resolved debt from the baseline restores a clean strict result.
printf 'declaration\t%s\t%s\n' \
  'crates/v3-cli/src/lib.rs' "$declaration_hash" \
  > "$TEST_ROOT/docs/standards/architecture-baseline.tsv"
run_strict >/dev/null

echo "Architecture harness tests passed"
