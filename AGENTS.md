# Petri repository instructions

Repo-wide rules, layout and pointers only; detail lives in the linked documents.
Before editing this file, follow `.agents/skills/agents-md/SKILL.md`.

## Layout

- `crates/`: `v3-core` (simulation), `v3-cli` (commands, benchmarks), `v3-server` (API server),
  `v3-lab` (capability assay lab, T22), `v3-telemetry` (run telemetry, T21).
- `frontend/`: web UI, with its own `frontend/AGENTS.md`. `scripts/`: POSIX checks and tooling.
  `telemetry/`: local observability stack. `experiments/worlds/`: world files.
- `docs/README.md` indexes the documentation: roadmap tracks in `docs/roadmaps/`, feature specs in
  `docs/specs/roadmap/`, research notes in `docs/strategy/`, progress records in `docs/progress/`.
- `CONTRIBUTING.md` covers roadmap editing and commits; `SECURITY.md` covers dependencies and reporting.

## Rules

- Verify in proportion to the changed files: run `make check` before completing source, runtime
  or build-configuration changes; documentation-only work uses `make check-docs`.
- Commits, remotes, pull requests and other external state require explicit user authorization.
- Preserve existing user changes and work carefully in dirty worktrees.
- Roadmap features run through `docs/workflow.md` (live contract, Claude roles, goal command);
  Codex runs apply `docs/workflow-codex.md`; rationale is in `docs/workflow-history.md`.
  These are one workflow: add no parallel workflow machinery.
- Asked for the next roadmap goal command, produce it from `docs/workflow.md` and do not
  implement the feature in that session. Generating a goal command or editing the workflow
  documents is not feature execution and uses none of the workflow's delegation.
- `docs/archive/` and `docs/prds/archive/` are historical and non-executable.
- Runtime-facing telemetry and state derive from applied simulation behavior; backward
  compatibility is not a default goal.
- Telemetry and experiment output follows the telemetry commit rule in `docs/workflow.md`:
  commit only the minimum summary; richer data stays byte-capped under ignored `.bench-artifacts/`.
- Capability, discovery and reachability questions run on the T22 lab (`crates/v3-lab`), not as
  one-off assay code in `v3-core` or scratch crates; lab code never enters production. Lab
  experiments are not roadmap features; see the exploration contract in
  `docs/roadmaps/t22-capability-assays-and-evolvability-lab.md`.
- Use TDD for behavior changes and bug fixes. Require determinism only where an assertion depends
  on reproducibility.
- Pure invariants get proptest property tests in v3-core whose assertions do not depend on which
  cases were drawn; commit `proptest-regressions/` files when they appear.
- Use `$rust-skills` for every Rust change, loading only the rule files relevant to the code.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior or
  tick-loop mechanics change.
- Shell automation is POSIX `sh`; add no Bash or Zsh runtime dependencies.
- Codex: every `wait_agent` call uses a timeout of at least 10 minutes (it returns early when a
  subagent responds or the user writes).
