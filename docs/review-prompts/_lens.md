## Lens rules

- Two other fresh contexts review the same diff through the other lenses;
  report only what belongs to yours.
- Read each document once and work from what you read. When a command's output
  is too long to read whole, rerun it narrowed (`grep`, a line range, `--stat`)
  rather than reading it again.

## Severity

- P1: wrong behavior, untruthful spec state, a waived required check, scope
  beyond the feature, or a blocking regression. Blocks closure.
- Telemetry P1: a counter, per-tick value, config value or mechanism switch
  the diff adds without its exported signal, because the T21.F07 row makes
  the signal part of the feature; a policy-sampled observation (span, event,
  window or body, not a cadence-bounded gauge) the diff adds without its
  policy, per-run cap and projected size, the `AGENTS.md` trace rule; a
  departure that unbounds memory, series or storage or can collide trace IDs,
  which is wrong behavior.
- Telemetry P2: a departure that only causes later rework (a name, a
  description, an attribute set). A spec planned before 2026-09-29 is not
  required to have a Telemetry section.
- Lenses 2 and 3 raise P1 only as their Method says. Scope and invariant
  findings belong to lens 1.
- P2: likely to cause rework or mislead a later feature; for lenses 2 and 3,
  code a later roadmap feature is expected to extend. Advisory.
- P3: wording, organization, or small cleanups. Advisory.
