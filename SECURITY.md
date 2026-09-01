# Security Policy

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Until the project
foundation PRD records an approved private reporting channel, contact the
repository maintainers through an already-established private channel.

This repository does not yet make claims about supported release versions. That
policy is established after the product and release model are selected.

## Local checks

`make precommit` runs the standard pre-commit framework, validates repository
quality and dependency policy, scans staged secrets, checks project skills for
HIGH or CRITICAL security findings, and scans dependency lockfiles for known
vulnerabilities. `make audit` scans repository history. Run `make setup` once
to install pinned Gitleaks, ShellCheck, actionlint, and OSV-Scanner through
Aqua, plus pre-commit and Cisco AI Skill Scanner, and install the Git hook. Aqua
enforces the committed release checksums. CI runs the same
deterministic checks and offline zizmor analysis over GitHub Actions definitions.
LLM skill analysis is intentionally excluded from blocking checks.

## Dependency acquisition policy

The project remains manifest-free until its stack is selected. Once a Node or
Python application manifest is added, `make check` enforces a committed lockfile,
an exact package-manager version, a seven-day minimum package age, and a matching
seven-day Dependabot cooldown. It checks each independent dependency root. A
workspace may share its root lockfile, configuration, and Dependabot entry with
its declared members; a nested manifest outside a declared workspace needs its
own lockfile and matching Dependabot `directory` entry. For Node projects, the
installed package-manager binary must exactly match the declared `packageManager`
value and its native configuration query must confirm the age gate. This applies
to new dependency resolution, not to reinstalling a committed lockfile.

- Use HTTPS registries and one authoritative source for each package namespace.
  Do not use an additional public index as a fallback for private names.
- Pin direct Git or URL dependencies to immutable commits and verify their
  content with a checksum or signature where the ecosystem supports it.
- CI must use the selected package manager's immutable or frozen-lockfile
  installation mode. Keep dependency lifecycle scripts disabled or allowlisted
  where the selected manager supports that control.
- npm requires `package-lock.json` and `min-release-age=7`; `npm-shrinkwrap.json`
  is not accepted because current npm no longer reads or writes it. pnpm requires
  `minimumReleaseAge: 10080`, strict age enforcement, and timestamps from
  registries; Yarn requires `npmMinimalAgeGate: 7d`; Python projects use uv with
  `exclude-newer = "7 days"`.
- An urgent security fix may bypass the age gate only for a specific reviewed
  package and version. Record the reason in the pull request and remove the
  exception after the package ages in.

The selected-stack PRD should use the package manager's current documentation:
[npm](https://docs.npmjs.com/using-npm/config/#min-release-age),
[pnpm](https://pnpm.io/settings/dependency-resolution#minimumreleaseage),
[Yarn](https://yarnpkg.com/configuration/yarnrc/#npmMinimalAgeGate), and
[uv](https://docs.astral.sh/uv/concepts/resolution/#dependency-cooldowns).
For workspace ownership and configuration precedence, also consult
[npm workspaces](https://docs.npmjs.com/cli/using-npm/workspaces/),
[pnpm workspaces](https://pnpm.io/workspaces),
[Yarn workspaces](https://yarnpkg.com/features/workspaces), and
[uv configuration files](https://docs.astral.sh/uv/configuration/files/).

`make dependency-audit` uses OSV-Scanner to block known vulnerabilities in
supported lockfiles. OSV's online service receives package metadata and file
hashes, not project source. If an exception is necessary, put an `IgnoredVulns`
entry in the adjacent `osv-scanner.toml` with its vulnerability ID, reason, and
`ignoreUntil` date; broad package overrides are prohibited.

## Deferred release controls

After the stack, repository visibility, and release artifact are selected, add
an SBOM and build provenance attestation, then decide whether GitHub dependency
review, CodeQL, OpenSSF Scorecard, and repository rulesets apply. These controls
depend on the selected release model and GitHub plan, so they are intentionally
not enabled in the stack-neutral foundation.
