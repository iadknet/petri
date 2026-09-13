# T13.F02 — Recruitment paths and replicated baseline readings

## Implementation verification

The observation uses eight fresh production ticks per subject, default runtime,
mutation and lifecycle charges, and only the specified fixture-world changes.
The complete historical drift walk and the three ecological profiles are
unchanged. The report-level experiment has its own configuration digest and
wall-clock field; gate and older reports remain explicitly unmeasured.

TDD evidence:

- `cargo test -p v3-core --test recruitment_paths` first failed because the new
  observation module was absent. After implementation, both applied-scene and
  constructed-path tests passed.
- `cargo test -p v3-cli recruitment_paths` first failed because the report and
  timing fields were absent. After wiring, its goal-only and historical-absence
  test passed.
- `cargo test -p v3-core recruitment_paths_retention --lib` first failed because
  the retention classifier was absent; the final verification below covers its
  implementation.
- The first expanded core pass, `cargo test -p v3-core recruitment_paths --lib`,
  passed eight tests: selection and Wilson invariants, complete-delta replay,
  matched starts/history, exact-copy activation, stationary wrong actions,
  complete reduced supply, and independent production-engine sibling replay.

Final implementation checks, 2026-09-12:

| Command | Result |
| --- | --- |
| `cargo check --workspace --all-targets` | Exit 0, all workspace targets checked. |
| `cargo test -p v3-core recruitment_paths` | Exit 0, 12 unit and 2 integration tests passed; unrelated tests filtered. |
| `cargo test -p v3-core --test recruitment_paths` | Exit 0, 2 tests passed. |
| `cargo test -p v3-cli recruitment_paths` | Exit 0, 2 tests passed, including once-per-world-set wiring and identical reduced results. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0, no warnings. |
| `make roadmap-check` | Exit 0, validation passed; repeated after the verification record update. |

No production defaults, founder behavior or tick-loop mechanics changed, so
the implementer's viability-first rule did not apply. The orchestrator owns
`make check`; the separate specialists own the final mutation and benchmark
gates. No new proptest regression file was generated.

The explicit reuse/simplification/efficiency review retained existing mutation,
trace, bypass, tracker, battery, config-digest and serialization seams. It
removed task-dead subjects from useful-module labels while retaining their
outcomes; represented removed-subject ending energy as absent; exposed separate
genotype/RNG divergence and backend-discard resolution; translated tracker-local
lineage IDs at the report boundary; corrected the F03/F04/F05 ownership labels;
and added useful-incumbent exact-copy activation coverage. A bounded sibling
helper satisfies the repository's function-length limit. The delta property
test now constructs its tiny genome directly, avoiding repeated task/battery
fixture execution inside a pure invariant. Affected focused tests and the
workspace compiler/lint checks were rerun successfully.

Advisor consultations: three complete — initial approach, repeated diagnostic,
and before-done review. Accepted guidance kept
constructed creation out of production proposal totals, registered copies
before preparation, and retained complete whole-birth replay including node
order and unselected terminal siblings. The repeated compiler diagnostic was
resolved through the existing public config-digest re-export; integer benefit
checks use strict comparison while the permitted one-scene loss remains intact.
The before-done review identified one blocker: unavailable memory after subject
removal must not become an observed zero. The focused removed-subject test first
failed against the former array representation. The observation now distinguishes
`Some([0; 16])` from unavailable memory and returns an unmeasured paired memory
effect if any required observation is missing. Living/dead controls and a pure
property test cover that distinction, including a measured difference followed
by an unavailable scene. The remediation self-review confirmed that the optional
state stays confined to observation records and consumes no mutation RNG. The
compiler, all focused tests, lint checks and roadmap validation passed again.
All three consultations' recommendations were accepted because they resolve
concrete reporting or verification requirements; none was rejected. No optional
architecture or requirement changes were introduced.

## Observation interpretation

The five Task A starting forms have identical always-NoOp battery actions and
4/8 task correctness. Their sizes, carrying charges and mutable sites differ.
Graph and VM constructed blank/copy paths reach 8/8 Task A correctness. The
changed-task arms share an explicitly recorded Task A-correct history; the
prepared module changes only its dormant cue and eastward action material.
Prepared-module activation reaches 8/8 Task B and 4/8 Task A in the fixture.
These are authored possibility controls, not production discovery counts.

Every complete production mutation call supplies one sibling record, including
zero-event calls and selected-but-inapplicable operator discards. Construction
registration is kept separate from those proposal totals. The tracker retains
the incumbent separately from the added scaffold and later modules. Full
battery and module facts are recorded at depths 0/32/48; task survival and the
battery's dead/silent/changed classification have separate meanings.

Mutation replay stores exact whole-birth before/after node values, insertion,
deletion, order and entry-node changes alongside the ordered event records and
seed. The first observed successful path includes its unselected terminal
sibling when applicable. This resolution does not attribute a multiply changed
field to an individual event inside the same birth.

The conditional VM sham is not applicable: this feature introduces no
mutation-RNG draw or constructor intervention. Pairing records the first
genotype/transition/RNG divergence; common seeds alone do not establish common
mutations after eligible sites diverge. Any unresolved transient target backend
is reported rather than inferred without evidence.

## Closure measurements

Pending the final-code gate and single standard goal run. The goal experiment
must contain 18 arms, four batches of eight lineages, 48 generations and two
siblings: 55,296 proposals. Its complete observation wall time must be at most
120 seconds. No discovery or retention value is claimed before that run.

The closure record will include per-arm and per-batch denominators, Wilson
intervals, paired lineage differences, observed discovery paths and retention,
null/low-exposure outcomes, applicable F03/F04/F05 gaps, and comparison verdicts.
