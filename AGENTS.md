# Petri repository instructions

- Use verification proportionate to the changed files. Run `make check` before completing application or runtime source-code or build-configuration changes; documentation-only work uses its relevant focused checks.
- Commits, remotes, pull requests, and other external state require explicit user authorization.
- Roadmap features are executed through the workflow in `docs/workflow.md`. Do not add parallel workflow machinery.
- When asked for the next roadmap goal command, produce it from `docs/workflow.md` and do not implement the feature in that session.
- `docs/archive/` and `docs/prds/archive/` are historical and non-executable.
- Preserve existing user changes and work carefully in dirty worktrees.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Use `$rust-skills` for every Rust change, loading only the rule files relevant to the affected code.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Shell automation must be POSIX `sh` compatible; do not add Bash or Zsh runtime dependencies.
