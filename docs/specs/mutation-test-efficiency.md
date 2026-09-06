# Mutation test efficiency — Technical Spec

**Status**: Complete
**Last updated**: 2026-09-05

## Overview

Mutation testing repeatedly builds and tests each changed-code mutant. The
existing wrapper limits mutant generation to the feature diff but runs all tests
in each affected package. Optimize that repeated work without narrowing coverage.

## Goal

Reduce complete diff-scoped mutation run time by at least 20% on a fixed workload,
preserving mutant detection, test inventory, and test assertions. Keep final
feature closure based on a fresh run; speed up intermediate remediation with
explicit reuse of earlier results.

## Non-Goals

- No production code or test-assertion changes, test filtering, reduced property
  case counts, or additional mutant exclusions.
- No custom mutation scheduler or coverage-to-test mapping.
- No merge until the user reviews the validation evidence and confirms it.

## Inputs

- Worktree `.worktrees/mutation-test-efficiency`, branch
  `codex/mutation-test-efficiency`, starting at
  `eb3c56987905483b82ff53e8e285423171823c68`.
- Fixed T11.F01 workload: `git diff 573c0336 eb3c5698`, 187 mutants, no source
  changes between compared runs. Explicit diff selection avoids benchmarking
  an empty tooling-only diff.
- cargo-mutants 27.1.0, two workers, fresh scratch builds (`--copy-target false`),
  unchanged timeout settings. Store temporary raw results under
  `/private/tmp/petri-mutation-efficiency`; retain a portable evidence summary
  with this spec when validation finishes.
- [Profile tuning](https://mutants.rs/performance.html) trades compilation time
  for repeated test runtime. Compare optimization levels 1 and 2 with the
  current test profile; preserve debug assertions and overflow checks.
- [Nextest](https://mutants.rs/nextest.html) can stop earlier after a failure,
  but process overhead and stragglers can make it slower. Trial version 0.9.143
  separately; adopt only if faster with the required test inventory preserved.
- [Iteration](https://mutants.rs/iterate.html) reuses caught/unviable outcomes
  heuristically. Limit it to intermediate, additive test remediation on the
  same production content; final closure always runs fresh.

## Implementation Tasks

- [x] Create the isolated worktree and freeze the 187-candidate workload.
- [x] Add wrapper regression tests, observe failure, and implement explicit
  `MUTANTS_ITERATE=0|1`, mode reporting, and preservation of prior evidence when
  a concurrent run blocks execution.
- [x] Compare build profiles and select the measured winner: optimization
  level 1 (177.97 seconds on the pilot), versus 282.34 seconds unoptimized and
  206.40 seconds at level 2. All 13 pilot mutant outcomes are identical.
- [x] Evaluate nextest separately: 184.13 seconds versus 177.97 with cargo at
  level 1, with identical pilot outcomes. Retain cargo: no demonstrated speed
  gain justifies another dependency or loss of automatic doctest coverage.
- [x] Update the shared workflow and command help for the selected defaults
  and intermediate remediation mode.

## Verification

- [x] Wrapper tests failed before implementation (`fresh mode not identified`),
  then passed. Existing benchmark-wait tests and repository quality checks pass.
- [x] Compare a fixed sample of caught, missed, and unviable mutants across
  candidates, then corroborate the chosen settings on all 187 mutants.
- [x] Compare test inventories and every mutant outcome: all 1,173 test names
  and all 187 mutant outcomes match, with no lost kills, new timeouts, new
  unviable mutants, or classification changes. Production and Rust test sources
  are unchanged.
- [x] Exercise incremental reuse with the real tool: it skipped 11 prior
  caught/unviable mutants and tested two survivors. The subsequent fresh run
  tested all 13 pilot candidates and reproduced their outcomes.
- [x] Run `make check` for the candidate on pinned base `eb3c5698`: exit 0 in 169.07 seconds,
  including the shell regression suites, Rust gates, 272 frontend tests,
  frontend build, dependency audit, and skill checks. Raw log:
  `/private/tmp/petri-mutation-efficiency/make-check.log`.

## Success Criteria

- [x] Complete-run elapsed time improves by at least 20%: 23.78% by UTC timestamps
  (55.22% by the execution stopwatch; discrepancy documented below).
- [x] All required functionality, selected tests, and mutant detection are
  preserved, with exact evidence recorded below.
- [x] Implementation and evidence are ready for integration.
- [x] User authorized merge and explicitly waived further test runs after
  integration with newer main.

## Notes for AI Agents

Integration update: main advanced to
`9c9942b774fbaa9b7fc2039eb6b3db7b824301c4` during validation. The tooling
worktree fast-forwarded to it without overlap or changes to the implementation.
The repeated viability check passed all 25 tests. The user then authorized
merging without any further tests. The repeat 129-mutant run was stopped and
is incomplete; the repeat `make check` was not run. Commit hooks are skipped
for this commit only to honor that waiver. The controlled T11.F01 comparison below
remains tied to its original pinned source revision.

Measured results (same source, tests, diff, workers, and timeout configuration;
stopwatch time includes scratch builds and the unmutated baseline):

| Run | Candidates | Stopwatch |
| --- | ---: | ---: |
| Full diff, original test profile | 187 | 1,860.61 s (31m 01s) |
| Full diff, selected level-1 profile | 187 | 833.23 s (13m 53s) |
| Pilot, original test profile | 13 | 282.34 s |
| Pilot, optimization level 1 | 13 | 177.97 s |
| Pilot, optimization level 2 | 13 | 206.40 s |
| Pilot, level 1 with nextest | 13 | 184.13 s |
| Incremental remediation, level 1 | 2 | 78.00 s |
| Fresh pilot after remediation, level 1 | 13 | 179.33 s |

The full-run UTC spans were 1860.14 seconds and
1417.77 seconds, a 23.78% reduction. The candidate UTC span
contains approximately 585 seconds absent from the stopwatch; the first level-1
pilot has a similar discrepancy of approximately 590 seconds. The cause is
unverified. Both clocks meet the full-run 20% target; the conservative UTC
comparison is the acceptance measure. Pilot comparisons above use stopwatch
time, with both clocks retained in the portable evidence.

Both full runs produced 146 caught, two missed, 39 unviable, and zero timed-out
mutants. Their native exit code is 2 because the two pre-existing survivors
remain; the wrapper treats survivor reporting as success, as before. The
unoptimized result also exactly matches the earlier T11.F01 outcome record.
The repeated level-1 pilot stopwatch time differed by less than 1%. This is evidence for one
feature diff on one Apple Silicon host, not a universal speed guarantee.

The [portable evidence](mutation-test-efficiency-results.json) includes exact
commands, input and configuration hashes, test inventory checks, timings, and
every full-run mutant classification. Raw logs remain under
`/private/tmp/petri-mutation-efficiency`; they are temporary and may be cleaned
by the host. Reproduction requires the pinned source revision with this tooling
change applied, not a later main containing different production code.

This is tooling/workflow work, not roadmap feature execution. The user authorized local integration from this worktree into main. The T11.F02 checkout and its artifacts belong to another task.
Benchmark runs are sequential and use the existing host-process guard. The
temporary nextest binary was verified against its official release SHA-256:
`4830d430411148d17602a75cc880bfb4dc8dac153dea59a48a2ef4cc93577f07`.
