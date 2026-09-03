# Security Policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Contact the repository
maintainers through an already-established private channel.

This repository does not yet make claims about supported release versions. That
policy is established after the product and release model are selected.

## Local checks

`make precommit` runs the standard pre-commit framework, validates repository
quality and dependency policy, scans staged secrets, checks project skills for
HIGH or CRITICAL security findings, and scans dependency lockfiles for known
vulnerabilities. `make audit` scans repository history. Run `make setup` once
to create project-local Aqua proxy links for pinned Node/npm, uv, Gitleaks,
ShellCheck, actionlint, and OSV-Scanner; Aqua downloads each tool on first use
after verifying committed release checksums. Setup also installs pre-commit and
Cisco AI Skill Scanner and installs the Git hook. CI runs the same
deterministic checks and offline zizmor analysis over GitHub Actions definitions.
LLM skill analysis is intentionally excluded from blocking checks.

## Dependency acquisition policy

The repository currently has a Cargo workspace and a frontend npm application.
`make check` enforces committed lockfiles, the frontend's exact npm version, a
seven-day minimum package age, and matching seven-day Dependabot cooldowns for
GitHub Actions, Cargo, and the frontend npm application. The installed npm binary
must exactly match the frontend's `packageManager` value, and npm's project
configuration must report `min-release-age=7`. The age gate applies to new
dependency resolution, not to reinstalling the committed lockfile.

- Use HTTPS registries and one authoritative source for each package namespace.
  Do not use an additional public index as a fallback for private names.
- Pin direct Git or URL dependencies to immutable commits and verify their
  content with a checksum or signature where the ecosystem supports it.
- CI must use the declared package manager's immutable or frozen-lockfile
  installation mode. Keep dependency lifecycle scripts disabled or allowlisted
  where the selected manager supports that control.
- The frontend requires `package-lock.json` and `min-release-age=7`.
- An urgent security fix may bypass the age gate only for a specific reviewed
  package and version through an explicit, reviewed policy change. Record the
  reason in the pull request and restore the gate after the package ages in.

`make dependency-audit` uses OSV-Scanner to block known vulnerabilities in
supported lockfiles. OSV's online service receives package metadata and file
hashes, not project source. Any vulnerability exception requires explicit review.

## Deferred release controls

After the release artifact is selected, add an SBOM and build provenance
attestation, then decide whether GitHub dependency
review, CodeQL, OpenSSF Scorecard, and repository rulesets apply. These controls
depend on the selected release model and GitHub plan, so they are intentionally
not enabled until the release model is established.
