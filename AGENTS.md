# Petri repository instructions

- Active PRDs live in `docs/prds/active/`; implement only a `Ready` or `In Progress` PRD with `Review Status: APPROVED`.
- Use the repository `$project-workflow` skill for PRD work. Lean mode is the default; Deep mode requires explicit user authorization recorded in the master PRD.
- In Lean mode, use one bounded planning pass, one persistent implementer, one diff-scoped final review, and at most one P1/P2 fix pass. Pass paths and compact evidence between agents; do not fork full conversation history.
- Stop and revise scope or request Deep-mode authorization when a Lean scope or remediation limit is exceeded. Do not continue autonomous review/fix recursion.
- Keep PRD dependencies, task checkboxes, documentation state, and verification evidence truthful. `make check` is the completion gate.
- Superpowers and the retired feature lifecycle are prohibited. Historical workflow material and Git history are archive material, not current guidance.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Use `$rust-skills` for every Rust change, loading only the rule files relevant to the affected code.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Shell automation must be POSIX `sh` compatible; do not add Bash or Zsh runtime dependencies.
- Do not create commits, pull requests, remotes, or external state without explicit user authorization.
