# Final diff review, lens 1 (correctness): {{FEATURE}}

{{RULES}}

## Target

- Feature: {{FEATURE}}, row in `{{TRACK}}`
- Worktree: `{{WORKTREE}}`
- Spec: `{{SPEC}}`
- Diff: `git diff {{BASE}}` ({{BASE}} is the merge base with `main`) and the
  untracked files from `git status --porcelain`

## Method

Apply `$adversarial-code-review`. The spec owner wrote the spec, so check the
diff and the spec against the original roadmap row, not only against each
other.

Read:

- `docs/roadmaps/README.md` (the roadmap contract); from `{{TRACK}}` the
  {{FEATURE}} row, its dependency rows, and the "Notes for AI Agents" entries
  that name them, not the whole track.
- From the spec: Goal, Inputs and Invariants, Architecture, Telemetry,
  Verification, Verification Plan, and the Performance predeclaration when the
  feature is subject to it.
- The Telemetry section of `docs/specs/roadmap/_feature-template.md`: the
  conventions table the telemetry checks compare the diff against.
- From `docs/workflow.md`: `### Review`, the `#### Verification plan`
  subsection of `### Benchmark gate` with its paragraphs, and `### Environmental
  pressures in the standard baseline`. Apply every check those sections assign
  to the reviewer: among them the Telemetry section check, the spec claim
  spot-check, the Verification Plan fit, and the lab and telemetry-feature
  scope claims. The benchmark profiles run after this review; do not audit
  their reports, but check that a feature subject to the Performance section
  predeclares its expected cost.

Check:

- Scope: the diff implements exactly the assigned feature and the `in-feature`
  preparatory refactors its spec declares in `Decision:` notes. Adjacent
  features, speculative work, or prerequisite work are findings, declared or
  not: a prerequisite lands as its own feature. So is a declared preparatory
  refactor the diff skips or exceeds.
- Architecture: the spec's Architecture paragraph says truthfully what the
  diff does to `docs/architecture.md`'s owners and boundaries; a diff that
  changes the desired architecture without editing that file is P1.
- Spec truthfulness: every checked box in the spec corresponds to an artifact
  in the diff; status, dates, and commit references describe what exists.
- Verification record: every command and result recorded in the spec's
  Verification section is consistent with the diff. Flag a recorded result the
  code makes implausible, for example a test file the diff does not add, or a
  viability gate not recorded when defaults, founders, or tick-loop mechanics
  changed.
- Repository rules from `AGENTS.md`: TDD for behavior changes, property tests
  for pure invariants with assertions that do not depend on which cases were
  drawn, POSIX `sh` in shell automation, telemetry derived from applied
  simulation behavior, and no parallel workflow machinery: everything the
  workflow uses is named from `docs/workflow.md`.
- Telemetry, against the template's conventions table:
  - each numeric cumulative counter or numeric `last_tick_*` value the diff
    adds has its family in `crates/v3-telemetry/src/metrics.rs`, its expected
    row in `crates/v3-telemetry/src/metrics/tests.rs` and its Telemetry row; a
    per-event record or seam is trace material and a measurement record is not
    a metric;
  - each config value or mechanism switch the diff adds is listed as recorded
    configuration (config body or `run.started` attribute) and, when it
    governs a traced phase, sits in that phase's group in
    `crates/v3-telemetry/src/trace.rs`;
  - no metric attribute carries a tick, creature, lineage, genome, seed or
    config value;
  - a policy-sampled observation the diff adds names its policy, per-run cap
    and projected size, while a gauge is bounded by the snapshot cadence; a
    trace kind it adds has an entropy-free ID that collides with neither
    delivered kind; an unsampled record at a region boundary needs neither;
  - a value not captured is absent, never zero;
  - a test the diff adds that spawns a binary passes `--telemetry off`, or is a
    telemetry fixture that passes `on` and scrubs `PETRI_TELEMETRY*` and the
    endpoint;
  - telemetry capture, export and seams the diff adds sit behind `telemetry`
    and `telemetry-seams`.

{{LENS}}

## Output

One finding per line, most severe first:
`P1: path:line — issue — the input or code path that triggers it, and why it
matters — smallest fix` (or `P2:`, `P3:`).
Then a `Could not verify:` list. Nothing else.
