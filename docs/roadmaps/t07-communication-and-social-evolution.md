# T07 — Communication and Social Evolution

**Status**: Planned
**Last updated**: 2026-09-02
**Master**: [Program Roadmap](../roadmap.md)

## Goal

Provide evolvable, costly signaling and reciprocal interaction primitives from
which communication, cooperation, deception, partner choice, and stable social
strategies can emerge and be tested causally.

## Track Success Criteria

- [ ] Signal production, propagation, perception, and cost are applied simulation behavior with no assigned semantic meaning.
- [ ] Live-versus-replay and ghost-agent assays distinguish reciprocal coordination from reactions to static cues.
- [ ] Cooperative benefits remain vulnerable to exploitation so signaling and social behavior face frequency-dependent tradeoffs.
- [ ] Independent long-running populations sustain more than one causally distinct social strategy under predeclared criteria.

## Executable Features

- [ ] **T07.F01 — Costly Evolvable Signal Emission** — Depends on: T01.F01, T03.F01
- [ ] **T07.F02 — Signal Perception and Identity Context** — Depends on: T07.F01
- [ ] **T07.F03 — Spatial and Temporal Signal Medium** — Depends on: T07.F01
- [ ] **T07.F04 — Live, Replay, and Ghost Social Assays** — Depends on: T07.F02, T07.F03, T10.F05
- [ ] **T07.F05 — Resource Transfer and Social Conflict Actions** — Depends on: T03.F05, T05.F01, T07.F02
- [ ] **T07.F06 — Reciprocity and Partner-Choice Opportunities** — Depends on: T07.F04, T07.F05, T09.F01
- [ ] **T07.F07 — Social Strategy Coexistence Confirmatory Campaign** — Depends on: T01.F09, T04.F06, T05.F04, T07.F06, T10.F08

## Notes for AI Agents

- Do not encode meanings such as food, danger, kin, or cooperation into signal channels. Meaning must arise from sender-receiver effects.
- Existing identity and kin-affinity inputs may provide context, but lineage labels must not become automatic cooperation bonuses.
- A social-cognition claim requires loss under a nonresponsive or temporally mismatched partner, not merely activity near another creature.
- T07.F01 through T07.F03 must retain signal channel count, energetic cost, diffusion, decay, and noise as treatment parameters. Their feature specs must include a bounded fixture or short-evolution comparison of at least two viable parameterizations; no cited study selects Petri's numerical values in advance.
- Research basis reviewed 2026-09-02: [Evolutionary Conditions for the Emergence of Communication in Robots](https://doi.org/10.1016/j.cub.2007.01.058), [Partner choice promotes cooperation](https://doi.org/10.1016/j.jtbi.2013.11.019), and [Embodied Dyadic Interaction Increases Complexity of Neural Dynamics](https://doi.org/10.3389/fpsyg.2019.00540).

| Option | Petri fit and evidence | Decision / exit criterion |
| --- | --- | --- |
| Fixed semantic messages | Easy to assay, but assigns meaning and removes sender-receiver coevolution. | Reject for evolutionary runs; allow only fixture controllers. |
| Cost-free global broadcast | Minimal implementation, but removes spatial history and permits consequence-free traffic. | Reject as the default; retain only as an ablation. |
| Costly uncommitted spatial-temporal medium | Supports evolved honest, deceptive, and context-dependent use while preserving local interaction. | Adopt provisionally. If bounded PoCs cannot produce detectable sender effects without saturating energy or bandwidth, revise cost/propagation in the owning feature before proceeding. |
