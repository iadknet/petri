# T27 design review

Codex plan-design-review of T27 roadmap changes made after the
[T26–T28 design review](t26-t28-design-review.md). These are not feature
readings: each feature's spec is written later by the workflow's Plan step.

## T27.F03 redesign (2026-10-21)

The user rejected the first F03 implementation's fixed `exchange.bite`
(closed at d36f7be6, reverted at 58121149) as a global amount no creature
shapes. The redesigned row makes the amount a heritable per-kind share
(`steal_share`, `give_share` in `HeritableBody`) × a decoded effort field
that is neutral when unwired × the source's current energy, with graded
effort for both acts and a zero transfer as a failure with no kill credit.

- **Job.** One fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) applying `$plan-design-review` with all six checks and coverage
  to the uncommitted row diff, with the live-run steal telemetry summary as
  evidence.
- **Inputs held fixed.** The user's 2026-10-21 decisions above: per-kind
  share present from the founder, neutral unwired effort, graded give
  effort, zero transfer as failure.
- **Evidence.** Prometheus counters from the petri-telemetry stack: 21 of
  247 live runs moved real energy by steal, all at least 2,566 ticks long,
  with late takeoff bursts; kills outnumber energy taken in several runs
  because zero-amount steals on already-exhausted victims count as kills.

### Round 1

Verdict: `ready with should-fix`. No blocking findings, no `step-back:`
findings; four premise findings (3 should-fix, 1 advisory).

| Check | Result |
| --- | --- |
| C1 Code decomposition | Clear |
| C2 Task decomposition | Clear |
| C3 Domain boundaries | Clear |
| C4 Premises | Finding (1–4) |
| C5 Larger design flaw | Clear |
| C6 Workarounds versus refactors | Clear |

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | should-fix, premise | Old-decoder run lengths do not establish zero use at 2,000 ticks under F03; neutral effort removes one barrier, and no giving was observed | Fixed in the row: the telemetry Note says the observations concern the old decoder and do not establish onset; the 2,000-tick reading reports giving and stealing, including zero, and the goal-long run gives the longer horizon |
| 2 | should-fix, premise | T11.F23's `max(|c|, 1)` floor makes steps absolute across (0, 1], so it is not a relative-step rule for the shares | Fixed in the row: the spec defines the bounded-share mutation and its boundary handling; T11.F23 is named as background only |
| 3 | should-fix, premise | Population mean energy times hit rate does not show aimed stealing pays; the multiplier and caps also apply | Fixed in the row: the 0.2–0.6 figures are illustrative gross returns, profitability is unestablished, and the spec predeclares the payoff under its charge base, the multiplier and applied-transfer limits |
| 4 | advisory, premise | "Every recorded steal requested nothing" contradicts the 21 runs with positive transfers | Fixed in the row: wiring a steal sink alone requests nothing, because the old amount reads 0 unwired |

The assumption ledger also marked the Polyworld give formula as unverified,
because the prior-art note covers only Polyworld's birth transfer. The
formula was checked against Polyworld's source
(`TSimulation::GetGiveStatus`, `src/library/sim/Simulation.cc`: energy ×
give activation × `GiveFraction`), and the row now cites that function. The
workaround ledger confirmed that the reverted catalog refactor (R1–R3, R8)
stays required inside F03 and that the reusable `627bd02e` also replaces
`tick.rs`'s read-back of steal results through predation events (R4).
