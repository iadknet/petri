# Capability Lab Contract

The standing rules for `crates/v3-lab`, the capability assay and evolvability
lab. They were the notes of the program 1 track that built the lab,
[T22](archive/program-1/roadmaps/t22-capability-assays-and-evolvability-lab.md),
which also records the closed features' implementation contracts. The design
evidence is the
[capability assay research note](strategy/capability-assay-research-2026-09-28.md).

## Purpose

The lab answers "can creatures evolve this capability under Petri's own
variation?" in seconds to minutes: small fixed arenas, real Petri genomes, the
production mutation engine, lab selection, and one capability scored at a
time. The lab measures reachability. The living world stays natural and
open-ended and never receives a lab task, reward or solution.

Lab runs are expensive in practice. Run one only when a decision hinges on it.

## Boundary

- `v3-lab` depends on `v3-core`. Neither `v3-core`, `v3-cli` nor `v3-server`
  depends on `v3-lab`, and only the `v3-lab` binary consumes it.
- No lab arena, scorer, selected genome or calibration constant is imported
  into a production default, founder or recipe. Lab selection replaces births,
  and that is the part that never enters production.
- A knob the lab needs and core lacks is a recorded finding for the owning
  roadmap track, never a core patch made from inside lab work.
- Capability, discovery and reachability questions run as assays, arenas or
  arms in `crates/v3-lab`, not as one-off assay modules in `v3-core` or
  scratch crates.

## What a lab verdict may decide

- A lab verdict may block or unblock roadmap rows, including mechanism rows
  that stay opt-in until qualified, and may retire opt-in, unqualified
  candidate code.
- It never sets a production default, founder, mutation policy or closure
  threshold. Production availability and any rate or targeting default
  require goal-profile ecological evidence recorded in the changing feature's
  own spec. A lab result can suggest that change, never justify it alone.
- A lab result may motivate a feature only through a mechanism that passes the
  master roadmap's natural-analog rule. A task reward, curriculum,
  scene-specific wiring or authored solution never does.
- "No feature may optimize against an indicator" applies to every assay.
- A lab pass shows a capability is reachable by native variation under lab
  selection. It does not show ecological retention or transfer, which remain
  with the goal profile. Necessary, not sufficient.

## Exploration runs

Lab experiments are not roadmap features and do not use
[the feature workflow](workflow.md).

- An exploration or diagnosis session runs existing assays with new
  parameters, arms (config overlays), genome files or arena files. Its
  deliverable is a dated research note under `docs/strategy/`: options tried,
  what was measured, compact results within the tracked-size limit, and a
  recommendation among a roadmap feature, no action, or more exploration.
- A new arena, arm, reading or diagnostic worth keeping lands inside
  `crates/v3-lab` through an ordinary commit gated by `make check`, with no
  spec, Codex challenge, mutation gate or benchmark.
- An exploration commit touches only `crates/v3-lab/src`,
  `crates/v3-lab/tests` and `docs/strategy/`. Its message lists the paths
  touched. A change to any `Cargo.toml` or `Cargo.lock`, a build script, the
  `Makefile`, a shared build setting, a production test, the harness core
  (loop, formats, calibration gate, controls) or a production crate is a
  roadmap feature.
- A track or feature is created only when a note recommends one and cites its
  readings.

**Prototype branches** (user decision, 2026-10-01). An exploration worktree
may carry, on its own branch, production-crate mechanism prototypes as
default-off arms classified `policy-deviation`, new instruments, and the
harness-core changes either needs, including a lab-authored comparator that is
labelled as such and passes the unchanged calibration gate. The lab and
production boundary holds on that branch too. That branch never merges into
`main`. The unchanged-policy reference arm must match `main`'s output before a
prototype arm is read, and only commits inside the exploration scope above
reach `main`. A prototype or instrument that proves out becomes a roadmap
feature.

## Every run

- Every run carries the unchanged-policy reference arm: the production
  `MutationConfig` with no overlay on supply, operator weights, targeting or
  operators. Any arm whose overlay changes mutation policy is classified
  `policy-deviation` in every verdict and summary and is never reported as
  native reachability.
- Every run carries its controls: founder-only, mutation-off, shuffled-score
  selection, authored comparator (labelled, never a start) and random-walk
  floor.
- A calibration gate checks the instrument before any campaign. An
  uncalibrated arena's readings are instrument readings, not reachability
  evidence.
- Improvement across generations is inherited change. It is never described
  as learning unless an arm enables lifetime learning and attributes it
  separately.

## Why-not ladder

"X is not evolving" decomposes into five rungs, each an existing reading. A
why-not run reports the first failing rung for a capability and start, with
the counts behind it, and routes it.

| Rung | Question | Routes to |
| --- | --- | --- |
| Exposure | Does the arena present the opportunity, and does the scorer respond to the authored comparator? | the assay (an instrument problem) |
| Supply | Does the native engine propose changes touching the relevant nodes or families at all? | mutation supply and recruitment |
| Viability | Are those proposals silent, changed or dead? | the genotype-phenotype map and boundary scale |
| Benefit | Does a viable change move the score? | the scorer, or in the world the pressure owners |
| Retention | Does a selected change persist under continued mutation and selection? | recruitment and retention observations |

Retention is read over identified descendant lineages, never over an elite
carried unchanged, and distinguishes deletion, lineage loss and censoring.

## Telemetry

Every lab command commits nothing by default. Its compact summary is a
versioned keep-list keyed to its named consumers and carries provenance:
source revision and dirty state, resolved config digest, overlay contents and
order, genome and arena content hashes, and seeds. Raw NDJSON stays under the
ignored `.bench-artifacts/lab/` beneath a byte cap enforced before every
write. A run that would exceed the cap stops and marks itself incomplete.
Per-tick traces and exhaustive per-proposal records are never written. The
general rule is in [benchmark artifacts](benchmark-artifacts.md).

Features that build the lab itself skip the gate and goal profiles under the
[workflow's lab exemption](workflow.md#benchmark-gate).
