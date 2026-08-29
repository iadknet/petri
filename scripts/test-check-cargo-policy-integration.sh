#!/usr/bin/env bash
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/petri-cargo-policy-integration.XXXXXX")"
trap 'rm -rf "$TEST_ROOT"' EXIT

mkdir -p "$TEST_ROOT/scripts" "$TEST_ROOT/crates/v3-core/src"
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

printf '%s\n' \
  '[package]' \
  'name = "v3-core"' \
  'version = "0.1.0"' \
  'edition = "2021"' \
  'rust-version.workspace = true' \
  '' \
  '[lints]' \
  'workspace = true' \
  > "$TEST_ROOT/crates/v3-core/Cargo.toml"

printf '%s\n' 'pub mod domain;' > "$TEST_ROOT/crates/v3-core/src/lib.rs"
printf '%s\n' '// Domain fixture.' > "$TEST_ROOT/crates/v3-core/src/domain.rs"

"$TEST_ROOT/scripts/check-architecture-harness.sh" --mode strict >/dev/null

sed -i.bak '/^\[lints\]/,$d' "$TEST_ROOT/crates/v3-core/Cargo.toml"
if "$TEST_ROOT/scripts/check-architecture-harness.sh" --mode strict \
  > "$TEST_ROOT/failure.txt" 2>&1; then
  echo "expected strict architecture harness to reject missing Cargo policy inheritance"
  exit 1
fi

grep -q 'Cargo policy: FAIL: crates/v3-core/Cargo.toml: lints must inherit' \
  "$TEST_ROOT/failure.txt"

echo "Cargo policy architecture-harness integration test passed"
