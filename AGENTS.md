# Petri repository instructions

- `make check` is the project completion gate.
- Commits, remotes, pull requests, and other external state require explicit user authorization.
- Superpowers and the retired feature lifecycle are prohibited. Historical material remains non-executable.
- Preserve existing user changes and work carefully in dirty worktrees.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Use `$rust-skills` for every Rust change, loading only the rule files relevant to the affected code.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Shell automation must be POSIX `sh` compatible; do not add Bash or Zsh runtime dependencies.
