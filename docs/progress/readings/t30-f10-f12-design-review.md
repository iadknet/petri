# T30.F10–F12 design review

Codex plan-design-review of adding T30.F10 (one operator catalog), T30.F11
(operators report their own edits) and T30.F12 (the sink catalog is
whole), with note lines on T30.F05, T30.F09 and T29.F01 (2026-10-27, user
direction after a post-closure refactor audit of T25.F11's areas). Not a
feature reading.

- **Job.** Three fresh read-only `codex exec` jobs (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, with all six checks,
  coverage and the Intent section per round, over the T30, T25 and T29
  tracks and `docs/roadmap.md`.
- **Input held fixed.** The user's direction: "Draft rows A, B and C in
  T30, the T30.F05/T29.F01 note lines and the F09 shim note, then run the
  track-design review and commit if it clears."
- **Source.** The refactor audit's ten candidates (a fresh read-only
  agent applying `$refactor-audit` in area mode to `genome/cgp` and
  `mutation/graph`): rows 1 → F10; 2, 6, 7 and the quadratic
  `insert_member` scan → F11; 3 → F12; 5 → T30.F05; 4 → T29.F01; 8–10 left
  for when that code is next touched.

## Round 1

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | F10's nested catalog and dispatch, F11's reports and container operations, F12's sink ownership, T29.F01's node encapsulation | — |
| C2 Task decomposition | clear | master order, F10 → T25.F12 and F09 → F12, the F05/F09/T29.F01 additions | — |
| C3 Domain boundaries | clear | `docs/architecture.md` §§1–5, the sensor-wiring, bundle-learning and heredity notes | — |
| C4 Premises | finding | catalog, reporting, rollback, sink and declaration-edit premises | 1 |
| C5 Larger design flaw | clear | `20ece7990`, `97e22ce6d`, `b14f0e6c01`, `c2631fcb97`, T25.F11 to `4b7c1bc46` | — |
| C6 Workarounds | clear | duplicate catalogs and dispatch, snapshots, declaration clones, detach, member scans, partial catalogs, default context, test samplers | — |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | T29.F01's note had seeding use `swap`, which rejects `FoodHere` (width 1) → `AreaFoodSummary` (width 7) | Fixed: seeding builds the node with its declarations through the node's construction API; `swap` keeps its rule |

## Round 2

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | F05, F09–F12, T29.F01, engine, domain enums, node edits, container | — |
| C2 Task decomposition | clear | master order, T25.F12 → T29 chain, F09 → F12 | — |
| C3 Domain boundaries | finding | owner operations against the architecture and notes | 2 |
| C4 Premises | finding | export, notes, validators, T25.F11 history | 3, 4 |
| C5 Larger design flaw | clear | `b14f0e6c01`, `c2631fcb9`, `9b6d452e6`, `cdf277859`, `c4a471e2e`, `4b7c1bc46` | — |
| C6 Workarounds | finding | snapshots, clones, detach, partial catalogs, shim, samplers | 2 |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 2 | blocking | F11's self-detaching retarget misses `split_edge`, a third caller-managed detach | Fixed: F11 makes `split_edge` owner-complete, behavior unchanged |
| 3 | advisory | T30.F05's note deleted `apply_gated`'s graph validation on an unshown equivalence | Fixed: the genome-owned gate carries the graph-validation and rollback contract; only checks an owner edit provably enforces are removed |
| 4 | advisory | T29.F01 still assigned the group-edit tallies `9b6d452e6` already deleted | Fixed: the note points at `BundleEdits` and T30.F11 |

## Round 3

Verdict: no findings, no `step-back:`. All six checks clear; every listed
architecture debt has a paying row, and F11/F12's priority positions are
planner preferences.

## Intent

| Source (session a5b1a622) | Reconciliation |
| --- | --- |
| User: "Should we do another independent review ... signs of other refactors we should do that were never in scope?" | The audit whose candidates these rows carry |
| User: "Draft rows A, B and C in T30, the T30.F05/T29.F01 note lines and the F09 shim note" | F10–F12, the F05 and T29.F01 notes, the F09 shim note |
| Assistant proposal accepted with row A: F10 before T25.F12 | T25.F12 depends on T30.F10 |
| Assistant choices | Row numbers and titles, which medium candidate joins which row, F11/F12 after F09; no edges from T27.F02/T28.F01 |
