#!/usr/bin/env python3
"""Validate the repository's inherited Cargo MSRV and lint policy."""

from __future__ import annotations

import argparse
import glob
from pathlib import Path
import re
import sys


EXPECTED_RUST_VERSION = "1.93"


def section(text: str, name: str) -> str | None:
    match = re.search(
        rf"(?ms)^\[{re.escape(name)}\][ \t]*\n(.*?)(?=^\[|\Z)",
        text,
    )
    return match.group(1) if match else None


def has_assignment(section_text: str | None, pattern: str) -> bool:
    return section_text is not None and re.search(
        rf"(?m)^[ \t]*{pattern}[ \t]*(?:#.*)?$",
        section_text,
    ) is not None


def workspace_member_patterns(workspace_section: str | None) -> list[str]:
    if workspace_section is None:
        return []
    match = re.search(r"(?s)\bmembers\s*=\s*\[(.*?)\]", workspace_section)
    if not match:
        return []
    return re.findall(r'"([^"]+)"', match.group(1))


def validate_policy(root: Path) -> list[str]:
    errors: list[str] = []
    root_manifest = root / "Cargo.toml"
    if not root_manifest.is_file():
        return [f"missing workspace manifest: {root_manifest}"]

    root_text = root_manifest.read_text(encoding="utf-8")
    workspace_section = section(root_text, "workspace")
    package_section = section(root_text, "workspace.package")
    if not has_assignment(
        package_section,
        rf'rust-version\s*=\s*"{re.escape(EXPECTED_RUST_VERSION)}"',
    ):
        errors.append(
            f"workspace.package.rust-version must be {EXPECTED_RUST_VERSION!r}"
        )

    rust_lints = section(root_text, "workspace.lints.rust")
    for name, expected in (("unsafe_code", "deny"), ("unexpected_cfgs", "warn")):
        if not has_assignment(rust_lints, rf'{name}\s*=\s*"{expected}"'):
            errors.append(f"workspace.lints.rust.{name} must be {expected!r}")

    clippy_lints = section(root_text, "workspace.lints.clippy")
    for name in ("correctness", "suspicious"):
        if not has_assignment(
            clippy_lints,
            rf'{name}\s*=\s*\{{\s*level\s*=\s*"deny"\s*,\s*priority\s*=\s*-1\s*\}}',
        ):
            errors.append(
                f"workspace.lints.clippy.{name} must be {{ level = 'deny', priority = -1 }}"
            )

    rustdoc_lints = section(root_text, "workspace.lints.rustdoc")
    if not has_assignment(
        rustdoc_lints,
        r'broken_intra_doc_links\s*=\s*"deny"',
    ):
        errors.append(
            "workspace.lints.rustdoc.broken_intra_doc_links must be 'deny'"
        )

    members = workspace_member_patterns(workspace_section)
    if not members:
        errors.append("workspace.members must be a non-empty list")
        return errors

    member_manifests: list[Path] = []
    for member_pattern in members:
        matches = sorted(Path(path) for path in glob.glob(str(root / member_pattern)))
        if not matches:
            errors.append(f"workspace member pattern matched nothing: {member_pattern}")
            continue
        member_manifests.extend(path / "Cargo.toml" for path in matches)

    for manifest in member_manifests:
        relative = manifest.relative_to(root)
        if not manifest.is_file():
            errors.append(f"missing member manifest: {relative}")
            continue
        member_text = manifest.read_text(encoding="utf-8")
        if not has_assignment(
            section(member_text, "package"),
            r"rust-version\.workspace\s*=\s*true",
        ):
            errors.append(f"{relative}: package rust-version must inherit from workspace")
        member_lints = section(member_text, "lints")
        lint_assignments = (
            []
            if member_lints is None
            else [
                line.strip()
                for line in member_lints.splitlines()
                if line.strip() and not line.lstrip().startswith("#")
            ]
        )
        if lint_assignments != ["workspace = true"]:
            errors.append(f"{relative}: lints must inherit from workspace without overrides")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="repository root containing Cargo.toml",
    )
    args = parser.parse_args()

    errors = validate_policy(args.root.resolve())
    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1

    print("Cargo policy check passed: MSRV and workspace lints are inherited")
    return 0


if __name__ == "__main__":
    sys.exit(main())
