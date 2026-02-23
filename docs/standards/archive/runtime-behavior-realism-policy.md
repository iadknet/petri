# Runtime Behavior Realism Policy

This policy defines generalized, open-ended requirements that prevent simulation surfaces from passing with synthetic behavior.

This policy is a domain-specific extension of `docs/standards/intent-verification-policy.md`.

## Scope

Applies to all current and future checkpoints whenever a runtime surface exposes simulation state or telemetry (server, CLI, web fixtures, and related tests).

## Core Requirements

1. Runtime truthfulness is mandatory.
   - Any exposed creature/world/telemetry value must come from applied simulation state transitions.
   - Placeholder formulas or synthetic counters are not acceptable substitutes for runtime behavior.
2. Boundary ownership is mandatory.
   - Simulation policy (tick semantics, energy, actions, death/reproduction) belongs in core runtime crates.
   - Transport layers may validate/map/serialize, but may not fabricate simulation outcomes.
3. Behavior visibility is mandatory.
   - If a surface claims ticks are running, tests must prove world behavior changes beyond tick counters alone.
   - If a surface exposes action counters or energy/population metrics, tests must prove they reflect applied behavior.
4. Lifecycle realism is mandatory.
   - Test suites must cover both positive dynamics (actions/state change) and negative dynamics (decay/exhaustion/death or equivalent removal path), where the domain includes those concepts.

## Generalized Verification Rule

For every runtime feature area (including future ones), required tests must include:

- one regression showing runtime state actually evolves during ticks/steps
- one regression showing reported telemetry matches applied behavior, not synthetic derivation
- one regression for a depletion/failure path relevant to that feature area

Equivalent tests are allowed by domain; exact file names may evolve.

## Change Control

- Any exception must be explicit in a plan `Open Questions` table, with owner and resolution status.
- Any new runtime surface must update checkpoint matrix requirements to include behavior-realism coverage.
