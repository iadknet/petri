# T15 — Benchmark Evidence Storage

**Status**: Planned
**Last updated**: 2026-09-12
**Master**: [Program Roadmap](../roadmap.md)

## Goal

The world's measured outcomes and closure verdicts remain auditable and
comparable from concise evidence in Git without committing full experiment
payloads. Detailed raw measurements remain separate local artifacts with
truthful provenance and availability.

## Track Success Criteria

- [ ] All future full gate, goal and sweep artifacts default to the main
  checkout's ignored `.bench-artifacts/<feature>/`, including calls made from
  feature worktrees; `OUT` remains an explicit raw-output override. No new full
  artifact is committed, including small gate reports.
- [ ] Committed, versioned summaries occupy the existing
  `docs/progress/features/` paths, with concise readings and valid series-index
  references. They retain comparison inputs and headline world/indicator
  readings without full genomes, per-proposal rows or traces.
- [ ] Each summary identifies the measured revision, exact command and exit
  evidence, profile/config/recipe identities and seeds, reference identities,
  thresholds, severe/inherited flags and wall caps. Experiment totals preserve
  proposals, outcomes, denominators, batch/lineage uncertainty and measured-zero
  versus unmeasured distinctions; selected claim extracts permit spot-checks.
- [ ] Raw provenance records SHA-256, byte count, local path and when
  availability was verified, without promising durable or portable storage.
  Summaries can be generated deterministically from existing artifacts without
  rerunning observations or relabeling old measurements as new runs.
- [ ] Comparison loading accepts both historical full reports and new summaries
  with identical normalized counters, per-case readings and verdicts from the
  same source. Profile/recipe compatibility, absent-reading semantics and
  missing-reference errors are preserved; no thresholds or epochs are changed
  to accommodate storage. Existing history is untouched and current report
  consumers remain functional, with omitted detail explicitly unavailable.
- [ ] The shared workflow, Codex adapter, feature template, output conventions
  and ignore rules describe this one contract; closure review verifies summary
  provenance and that no new raw artifact is staged.

## Executable Features

- [ ] **T15.F01 — Local Raw Artifacts and Committed Benchmark Summaries** — Depends on: T14.F01
  - Goal: The world's measured outcomes remain readable and comparable across closures from concise committed summaries, while full experiment records stay local and their availability is stated honestly.

## Notes for AI Agents

- Decision: Added at the user's direction on 2026-09-12 after T13.F02's single
  goal run produced a 356,934,683-byte report; its [readings](../progress/readings/t13-f02-recruitment-paths-and-replicated-baseline.md)
  record the narrow current exception. This track is Planned and unscheduled;
  it changes no live workflow or benchmark behavior until its feature executes.
  Write the flat spec just in time. T13.F02 is evidence, not a prerequisite.
- Decision: Local inspection on 2026-09-12 found existing output overrides in
  `Makefile` and full `Report` deserialization in
  `crates/v3-cli/src/bench.rs::compare_against_path`. Extend those producers and
  consumers together, preserving T14.F01's comparison-integrity work. Merely
  removing fields cannot supply a compatible reference. Compacting full JSON
  does not meet the user's concise-summary requirement; new artifact hosting
  would add an unrequested service. No new storage framework is prescribed.
- Decision: T15 owns evidence representation, storage and provenance, not new
  telemetry or indicators. T14 retains applied recording and comparison
  correctness; T13 retains recruitment science. Reuse existing reports,
  dependencies and the static progress page; no runtime/default/RNG change,
  experiment reduction, dashboard redesign or new campaign is in scope.
- Deferred: Historical migration, Git-history rewriting and durable external
  hosting are optional later decisions, not executable work or prerequisites
  in this track. Historical full reports stay readable and unchanged.
