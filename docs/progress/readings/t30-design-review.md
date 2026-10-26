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

## Architecture reviews (2026-10-25)

Three fresh read-only jobs from the fixed templates in
[`docs/review-prompts/`](../../review-prompts/): `adhoc-plan` on
`docs/architecture.md`, the T30 track and `scripts/architecture-debts.txt`
(base `d5361607`, before the document existed), run once by Codex Astra
(`gpt-6-astra`, `high`) and once by a Fable subagent; then `track-design`
(`gpt-6.1-sol`, `high`) on the resulting planning changes, with an intent
export of the session's turns. Debt IDs below are the committed ones (the
track-design job saw G21–G24 for today's G22–G25; G21 stays retired).

| # | Source | Finding | Disposition |
| --- | --- | --- | --- |
| 1 | Astra P2 | VM splice and insertion with jump repair have no owner | G23 added; T30.F05 makes them genome-owned edits |
| 2 | Astra P2 | The import checker missed multi-line and nested `use` groups and `super::`/`self::` paths | Fixed in `scripts/architecture-check.mjs` with tests; the code still passes |
| 3 | Astra P3 | F06's per-tick reproduction RNG lost its workaround treatment | F06 note requires a `WORKAROUND(T30.F06)` comment |
| 4 | Fable P2 | Action log and paint structs on the wire, copied by the frontend | G22; action log to F06, paint to F01 (user) |
| 5 | Fable P2 | Master T30 "Depends on: T25, T29" stale since the trim | Now None |
| 6 | Fable P2 | Readers and ledger had no paths in section 3 | Paths named |
| 7 | Fable P2 | F02 and F03 move types across a prohibition without naming their homes | F02: a contracts- or mutation-owned view; F03: contracts keys or `WorldKeySet` to contracts |
| 8 | Fable P3 | F02 overlaps with F03 and F06 unstated | Overlap notes added |
| 9 | Fable P3 | F06 is the track's largest row | One row in two halves (user) |
| 10 | Fable P3 | `contracts` has no admission rule | Rule added to its owner cell |
| 11 | Fable P3 | The genome sits under `creature/`, special-cased by the checker | G24; new row T30.F07 (user); checker case marked `WORKAROUND(G24)` |
| 12 | Fable P3 | Section 5's `file:line` cites duplicate the debts file | Rows cite modules and symbols |
| 13 | Fable P3 | `v3-core/src/bin/profile_ticks.rs` does I/O in the core crate | G25, unpaid; proposed to the user |
| 14 | track-design P2 | Register-count resize also left in mutation | Added to G23 and F05 |
| 15 | track-design P2 | G25 has no payer | Awaiting the user's disposition |
| 16 | track-design P2 | The sensor-wiring note still let F11 defer the vote vocabulary | Note corrected to the T25 assignment |

track-design coverage:

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | F01–F07, the three design notes, server projections, runtime state, sensor caches, VM edit helpers | units separate transport, execution, analysis and scheduling |
| C2 Task decomposition | finding | every debt payer, row dependencies, priority order, F06's halves | 15 |
| C3 Domain boundaries | finding | serialization, paint, dispatch history, sensor requirements, energy primitives, VM edits | 14 |
| C4 Premises | finding | intent export, design notes, roadmap records, blame and history | 16 |
| C5 Larger design flaw | clear | `d5f94720`, `729681fb`, `e33b4861`, `163a68b0`, `7e01c9b4`, `24c7aca3`, `6a1987d0` | the rows address the repeated leaks; no step-back |
| C6 Workarounds | clear | wire serialization, frontend rule copies, census caches, dispatch coupling, seed transport, tallies, reseeding, checker case | each has an owner and reason |

Intent: the job reconciled every user decision in the export (the review
request, waiting before more templates, fixing the agreed findings, F06's two
halves, F07, paint to F01) with the rows and found each recorded. It marked
T30.F07's priority position as an assistant proposal, which the master note
states, and the 2026-10-22 requirements as unverified because their source
conversations were not in the export.

### Planned-work and proportion review (2026-10-25)

`scripts/codex-review arch-scenarios`, run by Codex Astra and by a Fable
subagent from the same brief, walked all 15 open rows through the document and
checked every rule, prohibition, owner and debt against current code and the
open rows. Both found no larger design flaw.

| # | Source | Finding | Disposition |
| --- | --- | --- | --- |
| 17 | Astra P2, Fable P2 | Rows ordered before a debt's payer must build on it: T27.F02 and T29.F02 add sensed state before T30.F03's views (G9); T25.F11 and F12 extend seed alignment before T29.F01 (G4) | T30.F03 moved ahead of T27.F02 and T29.F02; rule 3 of "Use this file" lets an earlier row extend a debt in its present shape, inside the debt's modules, marked `WORKAROUND(<debt id>)` (user) |
| 18 | Astra P2, Fable P2 | G25 has no payer | T30.F07 moves the binary to `v3-cli` and deletes the checker's `bin/` skip (user) |
| 19 | Fable P2 | The one applied result has no named home | A contracts type (outcome kind, attempt effort); F06 note and contracts cell |
| 20 | Astra P3 | Rule 5's "once per birth" is broader than its purpose | Rule 5 keeps the separation; when derived data is computed is the owning row's choice |
| 21 | Fable P3 | Three prohibitions repeat the order | Folded into one sentence; the two the order does not say stay, the sensors one now covering field access |
| 22 | Fable P3 | T28.F01's material records have no owner | The ledger cell names energy and material accounting |
| 23 | Fable P3 | Put runtime below mutation and drop the view type F02 needs | Kept as siblings (user) |
| 24 | tooling research | A clippy `disallowed_fields` guard on sensors' reads until G9 and G20 are paid | Dropped; T30.F03 moved to the front of the priority order to pay them instead (user) |

A second `track-design` job (`gpt-6.1-sol`, `high`) reviewed findings 17–24's
planning changes, including T30.F03 at the front of the order, after the
first attempt stopped at Codex's usage limit. All six checks were clear but
C6, with no `step-back:`.

| # | Source | Finding | Disposition |
| --- | --- | --- | --- |
| 25 | track-design P2 | `group_edits`' `WORKAROUND(T25.F08)` tallies have no firm payer: T29.F01 "can also retire" them | T29.F01's note now requires the retirement |

Markers name their id as `AGENTS.md` and the workflow use it: a feature's own
workaround carries its `TNN.FNN`, and a debt extended under rule 3 carries the
debt ID (`WORKAROUND(G24)` on the checker's genome case).

Tooling research (2026-10-25, a read-only subagent): keep and harden
`scripts/architecture-check.mjs` rather than adopt cargo-modules, cargo-pup,
dylint or Semgrep (each misses function-body paths, needs nightly, or adds
nothing); generate frontend wire types with `ts-rs` (typeshare cannot emit
64-bit integers or internally tagged enums; specta is a release candidate);
no frontend boundary or ADR tooling. The user took the checker hardening (an
import of a module no domain owns, non-`mod` lines in `lib.rs` and any
dependency on `v3-lab` now fail) and `ts-rs` for T30.F01, and dropped the clippy
guard (row 24).

## Third round (2026-10-26)

Four fresh jobs from the fixed briefs: `adhoc-plan` on the document, the T30
track and the debts file (base `b86e075a`) and `arch-scenarios` over the 15
open rows, each run by Codex Astra and a Fable subagent. No `step-back:`.

| # | Source | Finding | Disposition |
| --- | --- | --- | --- |
| 26 | Fable plan P2, Fable scenarios P2 | Rule 3 keyed on priority position, while rows run by dependency readiness and in concurrent sessions | Rule 3 now applies to a row that lands before the payer |
| 27 | Fable plan P2, Fable scenarios P2 | The `WORKAROUND(T30.F06)` on the reproduction reseed names no intended design or payer, since independent streams were cut | The reseed is the tick's design (section 4); no marker (user). Reverses row 3 |
| 28 | Fable plan P2, Fable scenarios P2 | The census readers under `creature/` have G24's shape with no debt | G26, paid by T30.F07 moving them into `neighborhood/` (user); checker list marked `WORKAROUND(G26)` |
| 29 | Fable plan P2 | An extension under rule 3 also needs its import in the debts file | Rule 3 says so |
| 30 | Astra plan P2 | T30.F05 misses the dormant-tail copy operation | Added to G23 and F05 |
| 31 | Astra scenarios P2 | Compound mesh copy and removal stay caller-assembled while T29.F01 removes seeds from cloned backends | G4 and T29.F01 now make them genome edits returning mappings |
| 32 | Fable plan P3 | F01's goal line omitted paint | Goal names the inspector and the paint tools |
| 33 | Fable plan P3 | F07's binary move omits its feature and dependency | Note names `dhat-heap`, `dhat` and the telemetry flag |
| 34 | Fable plan P3 | F06's action-log wire type needs F01's generator | F06 ships a parity test if it lands first |
| 35 | Fable plan P3 | "Two halves" has no workflow meaning | Two implementation passes under one spec, one closure |
| 36 | Fable plan P3 | "Neither can own it" does not follow from the order; G22 listed `ActionType` | Reason is rule 1; `ActionType` noted as already in contracts |
| 37 | Fable plan P3 | Field access through another type is review-only but unlisted; "line-level" list | Section 5 lists it; "file-level" |
| 38 | Fable scenarios P3 | No owner for the sensor views | `sensors` defines them; the creature record and scheduler build them |
| 39 | Fable scenarios P3 | Ledger row carried T28 detail | Shortened |
| 40 | Fable scenarios P3 | Section 5 restated the checker | One sentence pointing at its header |
| 41 | Fable scenarios P3 | The checker hand-kept section 3's order | The checker now fails when section 3's list and its levels differ |
| 42 | Fable plan P3 | The `bin/` skip was unmarked | Marked `WORKAROUND(G25)` |

A confirming `track-design` job (`gpt-6.1-sol`, `high`) on the round's changes:

| # | Source | Finding | Disposition |
| --- | --- | --- | --- |
| 43 | track-design P1 | Row 31 gave T29.F01 route composition, which calls the VM repair T30.F05 moves later | Route composition moved to G23 and T30.F05; T29.F01 keeps the backend copies in `mutation::topology::structural`, which do not call that repair |
| 44 | track-design P2 | The heredity and bundle-learning notes still required the mutation gate | Removed from both notes |
