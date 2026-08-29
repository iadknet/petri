---
title: Public Repository Readiness
tags: [architecture, documentation, repository, portfolio, ci]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

Petri has a substantial Rust simulation, server, CLI, React visualization, test suite, and agent-oriented development process, but its public entry point does not yet explain that work to a time-constrained employer. The active Cargo workspace is also nested under the transitional `v3/` directory even though the repository's root-level frontend, documentation, scripts, and experiments all target that implementation; this makes the superseded layout look current. The tree lacks a root license file and public CI evidence, while generated experiment output obscures the authored source. Before changing GitHub visibility, the repository needs a focused structural, presentation, and hygiene pass plus a safety checklist for decisions that should not be automated, such as publishing historical author metadata.

## User Stories / Acceptance Criteria

- As a hiring manager, I can understand Petri's purpose, technical depth, current status, and major capabilities from the first screen of the root README.
- As a technical reviewer, I can see a real application screenshot, inspect the architecture, and find the active source and canonical documentation without navigating internal history first.
- As a developer, I can install pinned toolchains, start the full application, run a CLI example, and execute the documented verification commands from a fresh clone.
- As a technical reviewer, I find the active Cargo workspace at the repository root (`Cargo.toml`, `Cargo.lock`, and `crates/`) rather than behind a superseded `v3/` wrapper.
- As a repository visitor, I can identify the license, contribution expectations, security reporting path, and continuous-integration status.
- As the repository owner, I have a concise pre-publication checklist covering visibility consequences, secret scanning, repository metadata, and exposed commit-author emails.
- Generated experiment output is absent from the current source tree and remains ignored; curated experiment definitions and reproducible collection scripts remain tracked.
- Generated `artifacts/` paths and the audited historical work-domain address are absent from the rewritten publication branch's author, committer, and annotated-tag metadata, with a pre-rewrite Git bundle retained for recovery.
- All repository completion gates pass after the cleanup, and no crate names, `/v3/*` HTTP routes, wire formats, simulation behavior, or frontend behavior change.

## Out of Scope

- Changing the GitHub repository from private to public.
- Pushing local commits or changing GitHub repository settings, topics, description, social preview, or branch protection.
- Deploying a hosted demo or adding deployment infrastructure.
- Redesigning the frontend, changing simulation behavior, renaming crates or versioned HTTP routes, or changing Rust dependency directions and architecture boundaries.
- Adding broad community governance beyond concise contribution and security guidance appropriate to a portfolio project.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Which audience should the public entry point optimize for? | Employers first, with enough setup detail for technical reviewers and contributors. | user+agent | resolved |
| Which license should the repository publish? | Add the MIT license already declared by the Rust workspace manifests. | user+agent | resolved |
| Should historical work-domain author email addresses and generated artifacts be rewritten? | Yes. Preserve names, timestamps, messages, and topology where possible; map the audited work-domain address to `isaac@iadk.net`, purge `artifacts/`, and create a recovery bundle first. | user | resolved |
| Should this cleanup publish or deploy the project? | No; stop at a verified, locally committed publication-ready branch and checklist. | user+agent | resolved |
| Does "make `v3/` the base" mean replacing all root content? | No. Promote the active Cargo workspace files and `crates/` directory into the existing repository root alongside the already-active root `frontend/`, `docs/`, `scripts/`, and `experiments/`; remove only the empty transitional wrapper and its duplicate metadata. | user+agent | resolved |
