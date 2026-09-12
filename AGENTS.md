# Petri repository instructions

- Use verification proportionate to the changed files. Run `make check` before completing application or runtime source-code or build-configuration changes; documentation-only work uses `make check-docs`.
- Commits, remotes, pull requests, and other external state require explicit user authorization.
- Roadmap features are executed through the workflow in `docs/workflow.md`. Do not add parallel workflow machinery. That contract is split across `docs/workflow.md` (live contract), `docs/workflow-codex.md` (Codex substitutions), and `docs/workflow-history.md` (rationale and dated changes); the split is one workflow, not several.
- For every Codex roadmap feature, follow `docs/workflow-codex.md`: Sol at `medium` effort orchestrates with one separate persistent Astra `xhigh` spec owner and advisor for spec writing, readiness review, implementation advice, and escalation decisions. Delegate implementation and remediation to one persistent Astra `xhigh` subagent and use a fresh Astra `xhigh` subagent for final review; do not spawn a separate advisor. This delegation applies to feature execution, not requests to generate a goal prompt or edit the workflow.
- When asked for the next roadmap goal command, produce it from `docs/workflow.md` and do not implement the feature in that session.
- `docs/archive/` and `docs/prds/archive/` are historical and non-executable.
- Preserve existing user changes and work carefully in dirty worktrees.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Pure invariants get property tests (proptest in v3-core); assertions must not depend on which cases were drawn, and `proptest-regressions/` files are committed when they appear.
- Use `$rust-skills` for every Rust change, loading only the rule files relevant to the affected code.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Shell automation must be POSIX `sh` compatible; do not add Bash or Zsh runtime dependencies.
