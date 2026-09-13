# T15 — Benchmark Evidence Storage

**Status**: In Progress
**Last updated**: 2026-09-13
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
  to accommodate storage. Current report consumers remain functional, with
  omitted detail explicitly unavailable.
- [ ] Every full benchmark report reachable from the pre-migration `main` is
  inventoried. For each logical report, its final valid version has a generated
  committed summary and a byte-identical copy under the main checkout's ignored
  `.bench-artifacts/historical/`; the inventory records original path, blob and
  file hashes, byte count, summary path and local raw path. Historical-only or
  unparsable reports are retained locally and called out rather than silently
  dropped.
- [ ] Full benchmark report paths and blobs are absent from rewritten `main`
  history, while the post-rewrite tip contains the generated summaries and no
  tracked raw report. The remote update is limited to `main`, uses an explicit
  expected-old-tip force-with-lease, and is verified from a fresh clone. Any
  other advertised branch or tag retaining an affected object blocks cutover
  until its disposition is explicitly authorized; cached pull-request views,
  forks and existing clones are reported as outside what a `main` force-push
  can erase.
- [ ] The shared workflow, Codex adapter, feature template, output conventions
  and ignore rules describe this one contract; closure review verifies summary
  provenance and that no new raw artifact is staged.

## Executable Features

- [x] **T15.F01 — Local Raw Artifacts and Committed Benchmark Summaries** — Depends on: T14.F01
  - Goal: The world's measured outcomes remain readable and comparable across closures from concise committed summaries, while full experiment records stay local and their availability is stated honestly.
- [ ] **T15.F02 — Historical Report Migration and Main-History Rewrite** — Depends on: T15.F01
  - Goal: Every earlier benchmark report is reduced to concise committed evidence and preserved as a local ignored raw artifact, then the full reports are removed from `main` history and the rewritten branch replaces `origin/main` without overwriting an unseen remote update.

## Notes for AI Agents

- Decision: Added at the user's direction on 2026-09-12 after T13.F02's single
  goal run produced a 356,934,683-byte report; its [readings](../progress/readings/t13-f02-recruitment-paths-and-replicated-baseline.md)
  record the narrow current exception. T15.F01 is Complete under its flat spec;
  T15.F02 is In Progress under its
  [flat spec](../specs/roadmap/t15-f02-historical-report-migration-and-main-history-rewrite.md).
  Storage and workflow changes landed through T15.F01. T13.F02 is evidence,
  not a prerequisite.
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
- Decision: T15.F02 is the one-time retroactive migration requested by the user
  on 2026-09-13. T15.F01 lands the parser, summary format, comparison support,
  ignored artifact root and deterministic conversion first; F02 reuses that
  implementation and does not rerun any benchmark. The migration inventories
  reports from the exact pre-rewrite `main`, exports raw copies and generated
  summaries outside the rewrite clone, rewrites a disposable fresh clone, then
  adds summaries to the rewritten tip. This ordering prevents a path filter
  from deleting summaries restored at the same paths.
- Decision: Use `git-filter-repo` for the rewrite. Options reviewed on
  2026-09-13 were `git-filter-repo`, BFG Repo-Cleaner and Git's legacy
  `filter-branch`; the [Git project recommends `git-filter-repo`](https://github.com/newren/git-filter-repo/blob/main/README.md#why-filter-repo-instead-of-other-alternatives),
  whose fresh-clone safety and path filtering fit this migration, while BFG is
  narrower and `filter-branch` is documented as slow and hazardous. Perform
  the remote cutover as a single-ref force-push guarded by the explicitly
  captured old `origin/main` object ID; Git documents that exact-value
  [`--force-with-lease`](https://git-scm.com/docs/git-push#Documentation/git-push.txt---force-with-leaseltrefnamegtltexpectgt)
  as protection against overwriting an unseen update.
- Decision: F02 is an exceptional cutover, not an ordinary feature merge. Its
  dry run must prove the inventory, byte-identical raw copies, deterministic
  summaries, changed-ref set, rewritten-tree checks and a recoverable local
  backup before asking for separate explicit authorization to rewrite local
  `main` and force-push `origin/main`. No commit, remote or branch-protection
  change is implied by adding this roadmap row.
- Decision: A force-push makes the reports unreachable from rewritten
  `origin/main`; it does not promise physical erasure from forks, old clones,
  pull-request refs or hosting caches. GitHub's
  [history-rewrite guidance](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository#side-effects-of-rewriting-history)
  requires collaborator coordination, warns that commit IDs and signatures
  change, and notes that old clones can recontaminate the repository. F02
  records the old-to-new commit map and clone recovery instructions, pauses
  writes during cutover, and verifies all advertised refs it is authorized to
  change; durable external raw-artifact hosting remains out of scope.
