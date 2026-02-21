# petri-cli AGENTS.md

Local instructions for `crates/petri-cli`.

## Scope

`petri-cli` is a legacy maintenance surface.
For net-new architecture/contracts, treat V3 docs under `docs/strategy/` and
`docs/reference/` as canonical.

When touching this crate, keep headless simulation workflows here (run command,
ablation command, benchmark entrypoints).

## Boundary Rules

- Reuse `petri-core` policy; avoid duplicating simulation rules in CLI-specific code.
- Keep CLI argument parsing, reporting, and orchestration concerns localized.
- Do not introduce server/web transport behavior into this crate.

## Benchmark and Ablation Expectations

- Keep reproducible benchmark presets clearly named and documented.
- Treat benchmark thresholds as project policy, not ad hoc local overrides.
- If behavior changes affect benchmark interpretation, document expectations in plans.

## Local Test Strategy

- Run targeted CLI tests where present.
- For CLI behavior touching core semantics, run relevant `petri-core` tests and workspace gate.

## Related Canonical Docs

- Root policy: `AGENTS.md`
- Docs entrypoint: `docs/`
- Active strategy docs: `docs/strategy/`
- Active reference specs: `docs/reference/`
- Archived reference specs: `docs/reference/archive/`
