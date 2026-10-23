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

Output: `/private/tmp/claude-501/-Users-istefanek-projects-petri/dfc7ccd8-7685-41ad-a73b-a4c0e7045a0a/scratchpad/t30-review/codex-out.md`
(outside the repository).
