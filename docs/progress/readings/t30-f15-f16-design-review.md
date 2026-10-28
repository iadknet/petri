# T30.F15–F16 design review

Codex plan-design-review of adding T30.F15 (one definition per population
statistic) and T30.F16 (world-input keys name their food type) to T30 and
the master priority order (2026-10-28, user direction at T30.F13's closure).
Not a feature reading.

- **Job.** One fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, all six checks,
  coverage and Intent, over `docs/roadmaps/t30-core-boundaries.md`,
  `docs/roadmap.md` and `docs/progress/readings/t30-f13.md`. A first attempt
  exited before running (`--worktree` omitted) and was rerun.
- **Input held fixed.** "Plan the two payer rows (paired definitions R4–R6,
  and the food-typed world-input wire keys R11 with its sibling) with the
  track-design review."
- **Source.** T30.F13's unpaid accepts R4–R6 and R11
  ([readings](t30-f13.md)); the planner added telemetry's lossy
  `petri.world_input` label on the mutation counter as a third R11 site.

## Coverage

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | F15's `PopulationCensus`, `CladeCensus`, trait distributions; F16's `WorldInputKey` and label consumers | Rows consolidate existing concepts without a second responsibility |
| C2 Task decomposition | clear | Dependencies, master order change, verification notes, F11 overlap | Each row delivers independently after the closed F13 |
| C3 Domain boundaries | clear | Owners against `docs/architecture.md` §§2–5 | Arithmetic stays in readers; input identity in contracts; transport outside core; no architecture change or unpaid debt |
| C4 Premises | finding | Intent export, F13 readings, arithmetic, server projections, telemetry aggregation, generated map types | F1 |
| C5 Larger design flaw | clear | `09d70fda9`, `83ac4d5ac`, `a9f3deac6`, `4eabe8afc`, `f3df12b0d` | Rows remove duplicate arithmetic and lossy conversion directly |
| C6 Workarounds | finding | Paired-arithmetic markers, both server conversions, telemetry conversion, the reader label function, against R4–R6 and R11 | F2 |

## Findings and dispositions

| Round | # | Severity | Finding | Disposition |
| --- | --- | --- | --- | --- |
| 1 | F1 | advisory (P2) | F16's note said all three outputs overwrite colliding counts; telemetry's `Families::nested` sums them (`metrics/tests.rs` asserts it) | Fixed: the note separates the server's overwrite from telemetry's sum, which splits into food-typed series |
| 1 | F2 | advisory (P3) | F15 said "the four markers"; the paired definitions carry six | Fixed: "all six paired-definition markers" |

No blocking finding, no `step-back:` and no user-owned premise finding.

## Intent

| Source and short quote | Owning requirement | Reconciliation |
| --- | --- | --- |
| USER: "Plan the two payer rows … with the track-design review" | F15, F16 and their source note | Both payers present |
| SPEC-OWNER: "three quantities keep two definitions each" | F15; R4–R6 | Verified against the readers and consumers; an assistant proposal |
| SPEC-OWNER: "Same defect next door" | F16's two server maps | Verified |
| Planner: "F15 keeps the benchmark definition" | F15 note | Planner choice, not a user answer; compatible with the request |
| Planner: food-typed label as `WorldInputKey`'s transport key | F16 note | Fits contracts vocabulary; the helper's consumers follow the move |
| Planner: third lossy site in telemetry | F16 scope | Verified, with F1's correction |
| Planner: ordered after T30.F14 | Master priority order | A preference, not a dependency |
| Planner: no compatibility aliases | F16 note | Consistent with the repository rule; its source conversation is not in the export |

Reports: `round1.md` and `intent-export.md` in the session scratchpad
(`scratchpad/t30-f15-f16/`), not committed.
