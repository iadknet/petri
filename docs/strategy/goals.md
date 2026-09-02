# Petri High-Level Goals

This file is intentionally minimal and reflects only the current top-priority goals.

Use these goal IDs when connecting roadmap work to strategy and architecture.

## Goals

### GP-01 Evolve Richer Creature Decision-Making

- Why this matters: The project is centered on improving creature cognition and behavior quality over time.
- Non-goals:
  - Preserving transitional multi-action tick semantics as the long-term model.
  - Deferring cognition-focused semantics until after downstream feature work.

### GP-02 Maintain Clean Architecture Boundaries

- Why this matters: Clear separation of concerns keeps the system easier to evolve without cross-layer regressions.
- Non-goals:
  - Mixing transport concerns into simulation policy.
  - Breaking the established crate ownership model to land short-term changes faster.

### GP-03 Keep Iteration High-Confidence

- Why this matters: Fast, reliable tests and safe refactoring are required for sustained progress.
- Determinism scope reference: root `AGENTS.md`.
- Non-goals:
  - Treating determinism as a product goal by itself.
  - Accepting changes that reduce test confidence.

### GP-04 Keep Behavior Observable

- Why this matters: Evolution work requires visibility into runtime behavior, decisions, and outcomes.
- Non-goals:
  - Shipping major behavior changes with weak diagnostics.
  - Hiding arbitration or controller behavior that blocks debugging.
