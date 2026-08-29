#!/usr/bin/env python3
"""Regression tests for check_cargo_policy.py."""

from __future__ import annotations

from pathlib import Path
import tempfile
import unittest

from check_cargo_policy import validate_policy


VALID_ROOT = """\
[workspace]
members = ["crates/example"]

[workspace.package]
rust-version = "1.93"

[workspace.lints.rust]
unsafe_code = "deny"
unexpected_cfgs = "warn"

[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"
"""

VALID_MEMBER = """\
[package]
name = "example"
version = "0.1.0"
edition = "2021"
rust-version.workspace = true

[lints]
workspace = true
"""


class CargoPolicyTests(unittest.TestCase):
    def make_workspace(
        self,
        root_manifest: str = VALID_ROOT,
        member_manifest: str = VALID_MEMBER,
    ) -> Path:
        temp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(temp_dir.cleanup)
        root = Path(temp_dir.name)
        member_dir = root / "crates" / "example"
        member_dir.mkdir(parents=True)
        (root / "Cargo.toml").write_text(root_manifest, encoding="utf-8")
        (member_dir / "Cargo.toml").write_text(member_manifest, encoding="utf-8")
        return root

    def test_accepts_exact_inherited_policy(self) -> None:
        self.assertEqual(validate_policy(self.make_workspace()), [])

    def test_rejects_missing_workspace_policy(self) -> None:
        root = self.make_workspace(
            "[workspace]\nmembers = [\"crates/example\"]\n",
        )
        errors = validate_policy(root)
        self.assertTrue(any("rust-version" in error for error in errors))
        self.assertTrue(any("unsafe_code" in error for error in errors))
        self.assertTrue(any("broken_intra_doc_links" in error for error in errors))

    def test_rejects_member_without_inheritance(self) -> None:
        root = self.make_workspace(
            member_manifest="[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        errors = validate_policy(root)
        self.assertTrue(any("rust-version must inherit" in error for error in errors))
        self.assertTrue(any("lints must inherit" in error for error in errors))

    def test_rejects_each_weakened_lint(self) -> None:
        mutations = (
            ('unsafe_code = "deny"', 'unsafe_code = "warn"', "rust.unsafe_code"),
            (
                'unexpected_cfgs = "warn"',
                'unexpected_cfgs = "allow"',
                "rust.unexpected_cfgs",
            ),
            (
                'correctness = { level = "deny", priority = -1 }',
                'correctness = "warn"',
                "clippy.correctness",
            ),
            (
                'suspicious = { level = "deny", priority = -1 }',
                'suspicious = "warn"',
                "clippy.suspicious",
            ),
            (
                'broken_intra_doc_links = "deny"',
                'broken_intra_doc_links = "warn"',
                "rustdoc.broken_intra_doc_links",
            ),
        )

        for configured, weakened, expected_error in mutations:
            with self.subTest(lint=expected_error):
                root = self.make_workspace(VALID_ROOT.replace(configured, weakened))
                errors = validate_policy(root)
                self.assertTrue(
                    any(expected_error in error for error in errors),
                    errors,
                )


if __name__ == "__main__":
    unittest.main()
