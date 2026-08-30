# SkillSpector skill scan

`scripts/security/skillspector-scan.sh` scans the repository-local
`.claude/skills` tree with NVIDIA SkillSpector v2.11.0. It is deliberately a
local, explicit security-evidence operation. Its behavioral test harness is
safe for ordinary CI, but the full scanner is deliberately opt-in.

## Current repository status (2026-08-30 UTC)

The current repository-local SkillSpector scan is expected to return nonzero.
That is a fail-closed result, not a clean attestation or publication approval.
Its raw JSON report and machine manifest remain private, ignored evidence under
`artifacts/security/skillspector/<UTC>/`.

The manifest may record
`partial_classification: "nonfatal_unresolved_references_only"` only for the
narrow case where every completeness exception is a nonfatal unresolved
reference and every other execution, analyzer, scope, resource, root-coverage,
inspection, and no-LLM prerequisite is satisfied. This is diagnostic only: it
still records `report_valid: false`, `skill_scan_passed: false`, and a nonzero
wrapper exit. All other incomplete cases remain generic.

Use these four commands from the repository root:

```bash
scripts/security/test-skillspector-scan.sh
scripts/security/skillspector-scan.sh --bootstrap
scripts/security/skillspector-scan.sh --self-check
scripts/security/skillspector-scan.sh
```

## Trust and bootstrap boundary

The only networked operation is:

```bash
scripts/security/skillspector-scan.sh --bootstrap
```

Bootstrap downloads the exact GitHub source archive for NVIDIA SkillSpector
commit `b7241089d7ec15d8b30df980dacbb428214732b9` (tag `v2.11.0`), checks it
against the committed SHA-256 in
`scripts/security/skillspector/source-v2.11.0.sha256`, verifies the upstream
CLI contract and lockfile, then builds a local image. The image starts from
the digest-pinned official `python:3.12-slim-bookworm` image and resolves the
upstream committed `uv.lock` with `uv sync --locked --no-dev`.

The normal command never downloads or builds anything:

```bash
scripts/security/skillspector-scan.sh
```

It fails closed until the verified local image exists. No global Python
installation, repository venv, or vendored upstream source is used.

## Scan boundary and policy

Before scanning, the wrapper makes a private mode-0700 staged copy of
`.claude/skills`. It inventories every direct skill root, rejects unsafe or
external symlinks before reading their targets, and materializes validated
internal file aliases. The scanner receives only that staged copy as a
read-only `/scan` mount.

The container runs non-root with no network, a read-only root filesystem, all
Linux capabilities dropped, `no-new-privileges`, bounded CPU/memory/PIDs/runtime,
and a noexec/nosuid/nodev `/tmp`. It receives no forwarded environment or
credential variables. The exact upstream v2.11.0 command is:

```text
skillspector scan /scan --recursive --no-llm --fail-on-incomplete --format json --output /out/report.json
```

The wrapper fails closed on a missing/malformed report, version mismatch,
incomplete analysis, LLM/inference activity, missing or duplicate skill roots,
`DO_NOT_INSTALL`, or HIGH/CRITICAL risk. CAUTION remains visible in the result
and manifest, but does not by itself block this initial static-only policy.

## Private evidence

Raw reports, scanner output, and a machine-readable manifest are written below
`artifacts/security/skillspector/<UTC>/`. The existing `artifacts/` ignore rule
keeps them out of Git; directories are mode 0700 and files mode 0600. The
manifest contains only relative skill paths and scanner metadata—never host
paths or environment values—and records the staged input hash, root inventory,
symlink transformations, scanner status, report hash, and `skill_scan_passed`.
