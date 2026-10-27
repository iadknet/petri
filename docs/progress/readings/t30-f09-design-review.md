# T30.F09 design review

Codex plan-design-review of adding T30.F09 (retire the pre-lab experiments
in core), its dependency edges on T27.F02 and T28.F01, and the sampler
cleanup added to T30.F05 (2026-10-27, user direction at T25.F11's closure).
Not a feature reading.

- **Job.** Four fresh read-only `codex exec` jobs (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, with all six checks,
  coverage and the Intent section per round. Round 1: the T30 track and
  `docs/roadmap.md`. Rounds 2–4 add the T27 and T28 tracks. Each round's
  intent export carried the session's turns up to that point.
- **Input held fixed.** The user's direction: "Propose retiring the
  recruitment-paths slow tier instead of moving it", then "Add the T30.F09
  retirement row as drafted".

## Round 1

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | F09's core module, CLI command, benchmark schema, runner, artifact projection | — |
| C2 Task decomposition | finding | priority list, T27/T28 dependencies, concurrent-start rule, Makefile targets | 1, 2 |
| C3 Domain boundaries | clear | `docs/architecture.md` §§2–5, `docs/lab.md`; retirement implements the lab boundary | — |
| C4 Premises | finding | intent export, debt-first decision, assay consumers, lab supply and retention | 1 |
| C5 Larger design flaw | clear | recruitment introduction `712951212`, extensions, slow-tier extraction `2c9e77c04` | — |
| C6 Workarounds | clear | `WORKAROUND(T25.F11)` marker, goal-profile embedding, slow pins | — |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Priority position alone does not enforce the recorded debt-first requirement: T27.F02 and T28.F01 can start while F09 is active | Fixed (user, "Add T30.F09 as a dependency of T27.F02 and T28.F01, like T29.F03 and T30.F05"): both rows carry the edge; T28.F02 follows through F01. An earlier answer ("no dependency edges") rested on the assistant's wrong claim that the other debt payers were not edges, and was superseded |
| 2 | advisory | The inventory misses the `recruitment_paths` invocation in `make rust-test-temporal-fixtures` | Fixed: F09's notes name it, and `bench/schema.rs` |

## Round 2

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | finding | deletion scope, exports, benchmark producers, CLI command and surviving consumers | 4 |
| C2 Task decomposition | clear | F09's `None`, priority, new edges, T28.F02 inheritance; acyclic | — |
| C3 Domain boundaries | finding | architecture §§2–5, `docs/lab.md`, the interaction note | 3 |
| C4 Premises | finding | intent export, decision records, assay routes, introduction history | 3 |
| C5 Larger design flaw | clear | `712951212`, `2c9e77c04`, T25.F11 adaptation | — |
| C6 Workarounds | finding | recruitment marker, discovery experiment, CLI helper coupling | 3, 4 |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 3 | blocking | "Core holds no capability assay" is false: T20.F01's discovery assay (`neighborhood/opportunity`, `a0939c050`, 2026-09-25) remains in core with no payer | Goal corrected; F09 widened (user, "Widen T30.F09 to retire the T20.F01 discovery experiment and the input-opportunity command too"): the opportunity assay and command retire, the lab's controllers and `Family` move to `v3-lab` |
| 4 | advisory | `recruitment.rs` owns `HashingWriter`, `thread_pool`, `io_error` used by `input-opportunity` | Resolved by the widening: both consumers retire, so the helpers are deleted |

## Round 3

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | finding | deletion list, controllers, `Family`, CLI helpers, lab consumers | 5 |
| C2 Task decomposition | clear | four document diffs, T25.F12 → T29, T30.F05, the F09 edges | — |
| C3 Domain boundaries | clear | architecture §§2–5, `docs/lab.md` against removals and the controller move | — |
| C4 Premises | clear | `712951212`, `a0939c050`, `496245d24`, goal execution, CLI consumers | — |
| C5 Larger design flaw | clear | requalification history, `2c9e77c04`, comparator `ffd2dd490` | — |
| C6 Workarounds | finding | recruitment and opportunity assays, CLI helpers, comparator, unpaid accepts | 5, 6 |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 5 | blocking | Moving `controller()` beside `barrier_comparator` keeps two Ring constructions | Adopted (refactor removing a duplicate): one shared Ring-wiring operation, genomes unchanged |
| 6 | advisory | The VM/Topology test-only samplers (`WORKAROUND(T25.F08)`) have no payer | Adopted (user, "Add the VM/Topology test-only sampler cleanup to T30.F05's scope") |

## Round 4

Verdict: no findings, no `step-back:`. All six checks clear: F05 and F09
land independently, both downstream rows wait for F09, every section 5 debt
has a payer, and the shared Ring operation and sampler deletion remove the
workarounds rounds 3 named.

## Intent

| Source (session a5b1a622) | Reconciliation |
| --- | --- |
| User: "Propose retiring the recruitment-paths slow tier instead of moving it" | F09 retires rather than relocates |
| User: "Add the T30.F09 retirement row as drafted" | Full retirement: code, goal indicator, CLI, slow-tier line, workflow row |
| User: "Add T30.F09 as a dependency of T27.F02 and T28.F01, like T29.F03 and T30.F05" | Both rows carry the edge; supersedes the earlier "no dependency edges" |
| User: "Widen T30.F09 to retire the T20.F01 discovery experiment and the input-opportunity command too" | F09's goal and notes cover both experiments |
| User: "Add the VM/Topology test-only sampler cleanup to T30.F05's scope" | F05's notes; T25.F11's deferred note names F05 |
| Assistant choices | T30 placement, the F09 number, priority after F06, controllers moving to the lab |
