# Gitleaks Scan Attestation

**Status: zero findings — pre-sanitation secret-scan evidence only.**

On 2026-08-29T21:58:07Z, the repository's reachable Git history at source HEAD
`8554a3e6ec4937ee7f5252798d319147f34032c2` was scanned with pinned Gitleaks
v8.30.1. The scan explicitly used the pinned derived tagged-upstream default
configuration at `scripts/security/gitleaks-v8.30.1/default.gitleaks.toml`
(SHA-256 `0ceeb4f9c567f9f80ee05e8e37eeba4646df809f69c736a64d5b8b1398eb3e4c`).
It is reproducibly derived by removing only the final empty line from the tagged
upstream default (source SHA-256
`e163e53b9e7e8a8511e77271e2b323ed057759542a6d988258afe3a1fa329caf`) at
`https://raw.githubusercontent.com/gitleaks/gitleaks/v8.30.1/config/gitleaks.toml`.
It scanned a temporary bare `--no-hardlinks` mirror with
`--log-opts="--all --full-history"` and `--redact=100`.

The source inventory was unchanged before and after the scan: 50 refs, 12,311
objects, and SHA-256
`2d362190aa28a40d53a80b7d04bcf2faa460ea10d52cbb8f32979bd6fb8a9991`.
The scan result was `secret_scan_passed=true` with zero findings.

The raw redacted JSON report and machine manifest remain private under ignored
`artifacts/security/gitleaks/`; they are not a public artifact.

This does not authorize publication. Before publishing any sanitized publication
ref, create the sanitized ref set and rerun
`scripts/security/gitleaks-scan.sh` against it. A finding, scanner error, or
missing rerun means the secret scan has not passed for that publication ref.
