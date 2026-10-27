# T27.F02 reorder design review

Codex plan-design-review of moving T27.F02 after T29.F03 and T30.F05 in the
master priority order, then making that a dependency of T27.F02 and T28.F01
(2026-10-27, user direction). Not a feature reading.

- **Job.** One fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, with all six checks,
  coverage and the Intent section per round. Round 1: `docs/roadmap.md`
  (changed) with the T27, T29 and T30 tracks for context. Round 2 adds the
  T28 track and the heredity note.
- **Input held fixed.** The user's direction: "Move T27.F02 after T29.F03
  and T30.F05 so all architecture debt is paid first".

## Round 1

Verdict: two findings, no `step-back:`.

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | T27.F02 writes, held state and sensor views; T29 birth coordinator; T30.F05 analysis ownership | — |
| C2 Task decomposition | finding | priority list, dependencies, concurrent-start rule, verification plans; the graph is acyclic | 1 |
| C3 Domain boundaries | clear | rows against `docs/architecture.md`; every section 5 debt has a payer (T25.F11–F12, T29.F01–F03, T30.F05) | — |
| C4 Premises | finding | the debt-first claim, recorded user choices, construction behavior, cited preservation cases | 1, 2 |
| C5 Larger design flaw | clear | body, seed, group-edit and action-record history; the planned refactors address it | — |
| C6 Workarounds | clear | sensor-wiring duplication, seed transport, group-edit tallies, birth access, program repair; each has an owning row | — |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Priority position does not enforce debt-first: the concurrent-start rule lets T27.F02 (after T25.F11) and T28.F01 (now) start while T25.F12, T29 or T30.F05 is unfinished | Fixed (user, "Yes: add T29.F03 and T30.F05 as dependencies of T27.F02 and T28.F01"): both rows gain the edges; the master T27 and T28 rows list T29 and T30 |
| 2 | advisory | T29.F01's notes and the heredity note keep a birth cache-reuse preservation case that T30.F08 (`7fc60133f`) removed | Fixed (user, "fix finding 2 ... in the same change"): T29.F01's note and the heredity note's builder step 2 and L3 row name the one constructor |

## Round 2

Verdict: one advisory finding, no `step-back:`. C1, C2, C3, C5 and C6 clear:
both dependency chains are acyclic and cover every section 5 debt payer;
T30.F02 and F06, still preferences in the notes, are already complete.

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | T27.F02 runtime writes and views; T28 material transactions; T29 coordinator; T30.F05 edit APIs | — |
| C2 Task decomposition | clear | priority order, new edges, concurrent-start rule, verification plans | — |
| C3 Domain boundaries | clear | rows and cited notes against `docs/architecture.md` sections 1–5 | — |
| C4 Premises | finding | debt-first claim, decision records, constructor, body mutation laws, seed correspondence | 3 |
| C5 Larger design flaw | clear | seed, body, mutation-repair and construction history | — |
| C6 Workarounds | clear | seed transport, edit tallies, birth access, sensor wiring, structural edits; each has a payer before T27.F02 and T28.F01 | — |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 3 | advisory | T29.F02's re-pin list and the heredity note's consequences still name the no-mutation fast path that `7fc60133f` deleted | Fixed: both now list only mutation-effect screens and digests |

## Intent

| Source (session 3816c64f) | Reconciliation |
| --- | --- |
| Turn 2 user: "finish our architecture refactor before starting any new functionality like carrying and food caching" | Recorded in the master note; enforcement is finding 1 |
| Turn 3 user: "Move T27.F02 after T29.F03 and T30.F05" | Implemented in the priority order |
| Turn 3 user: "so all architecture debt is paid first" | Round 1: ordered but not enforced (finding 1); round 2: enforced by the T27.F02 and T28.F01 dependencies, confirmed against every section 5 debt |
| Turn 3 user: "run the track-design review for the reorder" | This file |
| Turn 4 user: "add T29.F03 and T30.F05 as dependencies of T27.F02 and T28.F01" | Both rows carry exactly those edges; T28.F02 follows through T28.F01; the master track-row dependencies are the assistant's summary of them |
| Turn 4 user: "fix finding 2 (stale cache-reuse premise in T29) in the same change" | Findings 2 and 3 |

Assistant claims checked: the turn-1 "disjoint surfaces" claim for running
T25.F11 beside T28.F01 was unsupported and was withdrawn in turn 2; the
debts and payers listed in turn 2 are confirmed; "run strictly sequentially"
and the round-1 "no dependency edges" are assistant proposals, not user
decisions; the user's turn-4 answer replaced the latter.

Output: both Codex reports kept in the session scratchpad, outside the repository.
