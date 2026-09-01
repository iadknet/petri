# Petri repository instructions

- Active PRDs live in `docs/prds/active/`; implement only a `Ready` or `In Progress` PRD with `Review Status: APPROVED`.
- Keep PRD dependencies, task checkboxes, documentation state, and verification evidence truthful. `make check` is the completion gate.
- Superpowers and the retired feature lifecycle are prohibited. Historical workflow material and Git history are archive material, not current guidance.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Shell automation must be POSIX `sh` compatible; do not add Bash or Zsh runtime dependencies.
- Do not create commits, pull requests, remotes, or external state without explicit user authorization.

## Lean delivery workflow

- Lean mode is the default. Deep mode requires explicit user opt-in recorded in
  the PRD before planning begins.
- Keep Lean work to at most two stages and a budget of roughly 20–25 affected
  files. Stop and request direction when scope or remediation limits are
  exceeded; do not create recursive review cycles.
- Use compact artifact and path handoffs between agents. Never use full-history
  forks. Preserve the same implementer through its single P1/P2 fix pass.
- Run focused checks while implementing, one `make check` before final review,
  then affected checks plus a final `make check` after any P1/P2 fixes.
