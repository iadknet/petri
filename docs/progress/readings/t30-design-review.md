# T30 design review

Codex plan-design-review of the T30 Core Boundaries track, the T27.F02
dependency change (`T25.F10` → `T25.F11`) and the master priority order
(2026-10-22). These are not feature readings: each feature's spec is written
later by the workflow's Plan step.

- **Job.** One fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) applying `$plan-design-review` with all six checks and coverage to
  the uncommitted track, row diff and [`docs/architecture.md`](../../architecture.md).
- **Inputs held fixed.** The user's 2026-10-22 directions: every A8 gap
  without an owning row gets a row; T27.F02 depends on T25.F11.

## Round 1

Verdict: `not-ready`. Two blocking findings, six advisory, no `step-back:`.

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | finding | core modules, server projections and frontend genome consumers against A3–A5 | 3, 5 |
| C2 Task decomposition | finding | all T30 rows, T25.F11–F12, T29.F01–F03, T27.F02, T28 and the next-feature rule; the graph is acyclic, T27.F02 → T25.F11 and the last row's two dependencies are sound | 4, 5, 6, 8 |
| C3 Domain boundaries | finding | contracts, genome, mutation, sensors, runtime, creature, simulation and server ownership against A2–A7 | 1, 2, 3 |
| C4 Premises | finding | current representations, draw sites, constructors, fixtures and the cited notes, with blame on `e5b41ddc`, `adb6a6d3`, `e33b4861`, `5329bd53`, `163a68b0` | 1, 2, 3, 7 |
| C5 Larger design flaw | clear | container and seed extraction, group-learning extension, sensor caches and the keyed food stream; the rows address those leaks | — |
| C6 Workarounds | finding | raw instruction serialization, frontend rule copies, census-backed construction, seed payloads, dispatch-record coupling | 3, 6 |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | F01 cannot present the desired learning and fact semantics as today's: one scale today takes contributors with different rules and traces | Fixed: F01 projects applied semantics; heterogeneous contributors and fact members are its preservation cases; F11, F12 and T29.F02 change the view's schema with their semantics |
| 2 | blocking | F11's descriptor needs vote and outcome counts and `OUTPUT_SLOT_COUNT`, which G2 left to F11 only optionally and F04 postponed until after F11 | Fixed: that vocabulary is T25.F11's unconditionally (T25 Notes, A8 G2); the rest of G2 (route resolution, jump targets) is the genome-analysis row |
| 3 | advisory | `vote-sinks.json` is catalog data, not type generation; the sampler aliases VM instructions; the frontend recomputes member patterns and learner initialization | Adopted: generated types or an exhaustive parity test; the sampler route is in scope; the view supplies resolved facts and the frontend copies are deleted (F01 Notes, A2) |
| 4 | advisory | Priority position does not enforce completion; "F01 before F11" and "T28 builds on F02" read as prerequisites | Adopted: stated as scheduling preferences in the master note and T30 Notes; no dependency edges added |
| 5 | advisory | The contracts-and-genome row bundles independent owners | Adopted: split into F04 Config and World Without Cycles (G10, G11) and F05 Genome Owns Its Analysis (G2 rest, G12, G15); F05 needs no F11 dependency once the descriptor vocabulary is F11's |
| 6 | advisory | F11 already needs a declaration-plus-backend transaction for `AddSensor*`; deferring all of it builds a second coordinator | Adopted: F11 builds the minimum transaction (T25 Notes, A8 G13); F06 moves the other edit paths onto it, keeping unused declarations and soft defaults |
| 7 | advisory | The wiring note's upstream bus is 8 slots; `OUTPUT_SLOT_COUNT` is 24 | Adopted: the note says 24; the count moves to contracts with F11 |
| 8 | advisory | Refactor goal lines describe modules, not the world | Adopted: goal lines restated around preserved spending, sensing, mutation and inheritance, each with "with no behavior change" |

Output: `/tmp/claude-501/-Users-<user>-projects-petri/dfc7ccd8-7685-41ad-a73b-a4c0e7045a0a/scratchpad/t30-review/codex-out.md`
(outside the repository).

## Architecture-review follow-up (2026-10-22)

A fresh Astra (`gpt-6-astra`, `high`) adversarial review of
`docs/architecture.md` at `bf876ae7` returned `sound with changes`. The user
chose, for its four design questions: Graph edits own inheritance
correspondence; simulation splits into scheduling and mechanics with one
applied result per action; imports follow direct arrows only; memory
inheritance uses a parent provisioning mask and a child uptake mask; and
both new gaps get rows ("Yes, add them": T30.F08, F09). One fresh Codex Sol
`high` plan-design-review job then covered the resulting changes: T29's
re-scoped F01 criterion and F03, T30.F03's added G20, T30.F08 and F09, T28's
note, the master order and the doc.

Verdict: `not-ready`. Six blocking findings, two advisory, no `step-back:`.

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | finding | A3–A5, T29.F01, T30.F03, F08, F09, action executors, aggregate and sensor types | 1, 2, 7 |
| C2 Task decomposition | finding | every G1–G20 owner and the relevant rows; F09 after F08, both before T28 as preference | 2, 4, 8 |
| C3 Domain boundaries | finding | the direct arrows against cognition, construction, perception, action resolution and removal | 1, 2, 3 |
| C4 Premises | finding | current code, the cited notes, blame for seed alignment, RNG derivation, gestation and sensor caches | 3, 4, 5, 6, 7 |
| C5 Larger design flaw | clear | `e33b4861`, `5329bd53`, `7e01c9b4`, `24c7aca3`: the proposed changes address the repeated pressure points | — |
| C6 Workarounds | finding | seed transport, mutation rollback, reproduction reseeding, census-backed caches, the `WORKAROUND(T25.F08)` group-edit tallies | 4, 7 |

| # | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Direct arrows forbade work owners do: scheduling calling cognition, holding aggregate and world state, seeding through birth; the aggregate's key-set and mutation summary imports | Fixed: arrows scheduling → runtime, aggregate, kernel, birth and aggregate → mutation (summary types); sensor key sets go to contracts (G9); G8's wording matches the graph |
| 2 | blocking | Moving executors alone leaves mechanics taking the whole `Simulation`, removals and flooring in the scheduler, and move effort composed on the aggregate | Fixed: A5 mechanics inputs and cost-law rows; G19 widened with evidence; F08 Notes cover narrow inputs, removal settlement and cost composition; T28's surcharge goes through the mechanics' cost law |
| 3 | blocking | "Rejection leaves every party unchanged" contradicts give and steal charging attempt effort before rejection | Fixed: the transfer is atomic, the attempt effort is charged and carried in the applied result (A5; F08, F09 Notes) |
| 4 | blocking | Run-start independent streams are not today's behavior (reproduction reseeded from the queue RNG each tick) and no row changed it | Fixed: gap G21 and behavior-changing row T30.F10; F08 keeps the reseeding marked `WORKAROUND(T30.F10)`; T30's criteria name F10 as the one row that moves draws |
| 5 | blocking | The heredity note's model still had six body genes, a child-only mask, founder 86 and stable ids alone; T29's Decisions still said one mask | Fixed: note rewritten to seven genes, both masks, founder 87, graph-returned mappings; the one-mask decision kept as superseded history; T29's Decisions updated |
| 6 | blocking | The phase table omitted cognition's energy debits and shared-memory writes, the world-update memory snapshot, and delivery in the parent's action slot | Fixed: A6 phase table rows |
| 7 | advisory | Mappings must cover seed invalidation on source, weight and scale writes, detach fallbacks, whole-backend clones and rollback | Adopted: A5 inheritance row and T29.F01 Notes, with the `WORKAROUND(T25.F08)` tallies as a retirement candidate |
| 8 | advisory | F08 and F09 overlap T27.F02, T28.F01, T29.F01 and T30.F02; lifecycle events are not action results | Adopted: T30 integration note; A5 applied-result row |

Output: `/tmp/claude-501/-Users-<user>-projects-petri/dfc7ccd8-7685-41ad-a73b-a4c0e7045a0a/scratchpad/t30-review-2/codex-out.md`
(outside the repository). No confirming round was run.

## Simplicity review (2026-10-22)

A fresh Astra (`gpt-6-astra`, `high`) review of the desired architecture,
the T30 rows and the hooks, asked only what could be cut, merged or stated
more simply while still preventing spaghetti, returned `trim`. The user took
all four proposals:

| Proposal | Disposition |
| --- | --- |
| Slim `docs/architecture.md`: five sections, about twelve owners, six rules; creature and simulation one domain each; feature detail (learning variants, the two-mask rule, address lifetimes, birth API names, phase cells, signal timing, material stocks) to the implementing rows | Done; detail lives in T25, T27, T28, T29 and T30 Notes |
| Replace the 26-arrow allowlist with a dependency order, no cycles and five named prohibitions (reverses the morning's "direct arrows only") | Done (section 3) |
| T30 from ten rows to six: F07 into F02 (no T29.F01 dependency; reproduction already passes the dispatch record), F09 into F08 (now F06, with one removal-settlement path), the old F06's edits folded into T25.F11, F10 and the independent-streams rule cut (G21 dropped) | Done; rows renumbered F01–F06 |
| Lighter process: no roadmap-check citation rule or track Architecture notes; a one-paragraph spec Architecture section; the design lens reports only coupling a diff introduces or widens; known debts instead of an exhaustive gap ledger; the reviewer agent points to the workflow's rule | Done; checker reverted to its state before `bf876ae7` |

Output: `/tmp/claude-501/-Users-<user>-projects-petri/dfc7ccd8-7685-41ad-a73b-a4c0e7045a0a/scratchpad/arch-review-3/codex-out.md`
(outside the repository). Row IDs in the sections above are the pre-trim
numbering.
