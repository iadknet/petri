# Petri repository instructions

- Active PRDs live in `docs/prds/active/`; implement only a `Ready` or `In Progress` PRD with `Review Status: APPROVED`.
- Keep PRD dependencies, task checkboxes, documentation state, and verification evidence truthful. `make check` is the completion gate.
- Superpowers and the retired feature lifecycle are prohibited. Historical workflow material and Git history are archive material, not current guidance.
- Runtime-facing telemetry and state must derive from applied simulation behavior; backward compatibility is not a default goal.
- Use TDD for behavior changes and bug fixes. Test determinism is required only where assertions depend on reproducibility.
- Run `cargo test -p v3-core --test viability` first when production defaults, founder behavior, or tick-loop mechanics change.
- Do not create commits, pull requests, remotes, or external state without explicit user authorization.
