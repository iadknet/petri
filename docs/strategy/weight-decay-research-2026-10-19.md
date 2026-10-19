# Learned-weight decay: prior art and a probe

**Date**: 2026-10-19
**Question**: A learned edge weight in Petri persists for the creature's whole
life unless activity or reward moves it; nothing returns it toward the weight
it was born with. Should Petri add passive forgetting, toward what target, at
what grain, and is the rate heritable?
**Method**: the T25.F06 plasticity runtime read in code
(`crates/v3-core/src/runtime/plasticity/{rules.rs,learned.rs}`); the T25.F06
goal census re-read; three parallel literature passes (artificial life and
evolved plasticity, neuroscience, machine learning) reading 47 primary sources
in full text, with verbatim quotes, recorded in
[the source appendices](weight-decay-research-2026-10-19/); the decisive quotes
re-checked against the extracted text or the publisher page; a scratch probe on
the production update law (Section 3, not committed).

## Summary

Every model in all three literatures that returns a learned weight "to zero"
does so for a deviation from a baseline: the effective weight relaxes toward
the synapse's own pre-learning value, never toward zero. That is the design
fast weights have used since Hinton and Plaut (1987), the one the
tag-and-capture and consolidation models use (Clopath et al. 2008; Ziegler et
al. 2015; Leimer et al. 2019), and the one the in vivo data show: LTP "eventually
returned to baseline even if it took many weeks to do so" (Abraham 2003, p. 736).
Decay rates differ by pathway and memory type (Abraham 2003: families at 2.1 h,
3.5 d and 20.3 d; Davis & Zhong 2017: Rac1 speeds object-recognition forgetting
about threefold and leaves fear memory unchanged), and the normative theory
says the rate should match how fast the environment changes and can be
"genetically encoded" (Brea et al. 2014). The one evolutionary study that
evolved a decay rate found evolution kept a consistent nonzero decay of the
learned component (δ = 0.0011 ± 0.0002 over five runs) and drove ordinary
decay toward zero to "values so low that they have no significant effect"
(Bullinaria 2009, pp. 5, 7).

The case for adding it in Petri is the random walk. A single-component
reward-modulated weight receiving zero-mean coincidental updates "loses its
original value … The weight undergoes a random walk, or diffusion, that leads
to information loss" (Soltoggio 2015, p. 8). Petri's probe reproduces this on
the production law: at learning rate 0.005 the mean distance from the birth
weight grows from 0.19 at 100 ticks to 1.38 at 20,000 with no decay, and holds
at 0.58 with a per-tick retention of 0.999, which keeps about 90% of a
reward-correlated learned shift. A retention of 0.99 holds the walk at 0.14
but erases most of the real learning too. No single rate fits, which is the
argument for a heritable one.

The strongest case against: Petri already clamps every learned weight, and
Backpropamine found a hard clip gave "equal or superior performance" to decay
(Miconi et al. 2019, p. 3); Yaman et al. (2021) found evolution prefers rules
that stop changing once the task is learned. Both come from lifetimes that
reset each episode. Petri's creatures live thousands of ticks without a reset,
the regime Miconi (2023, Appendix G.4) calls "the missing loop", and the clamp
bounds the walk without returning anything to the evolved prior.

**Recommendation** (Section 5): add a heritable per-edge retention
`w ← b + ρ·(w − b)` applied once per world tick to every plastic edge, where
`b` is the slot's birth value (the clamped Lamarckian seed when seeded, else
the clamped genomic weight) and `ρ ∈ [0, 1]`, with the founder at exactly
`ρ = 1` so the feature is founder-neutral. Named analog: early LTP decays back
to baseline; intrinsic forgetting returns the synapse to its birth state. Place
it after T25.F08, so the group scale F07 makes learnable decays by the same
rule. The user placed it as
[T25.F09](../roadmaps/t25-edge-groups-and-learning.md) on 2026-10-19.
Nothing in production code is changed by this note.

## 1. What Petri does today

- Each plastic edge carries a `PlasticityConfig`: rule (Classic
  `η·pre·post`, Oja `η·post·(pre − w·post)`, AntiHebb, Covariance), learning
  rate, `weight_clamp` in `[0.01, 10]`, a Lamarckian flag and optional reward
  modulation (`reward_source`, `trace_decay`).
- `trace_decay` multiplies the eligibility trace once per world tick
  (`learned.rs`, `decay`). It fades which synapse was recently active, not the
  weight (Gerstner et al. 2018 keep the two separate: the flag decays over
  seconds, the tag over about an hour).
- Oja's rule carries an activity-gated pull toward zero (`−η·w·post²`); it
  does nothing while the edge is idle.
- A Darwinian edge restarts from the genomic weight at birth; a Lamarckian
  edge starts from the parent's learned weight, clamped by the child's bound.
- **The genomic weight is never clamped**: `from_seeds` clamps a Lamarckian
  seed but builds an unseeded slot from `genomic_weight` as is, and
  `perturb_weight` (`mutation/graph/operators.rs`) has no bound. An edge whose
  genomic weight exceeds its clamp snaps inside on its first update. A decay
  target outside the clamp would fight the clamp every tick.

The T25.F06 goal census at its last sample
([readings](../progress/readings/t25-f06.md)): about 6% of creatures carry a
plastic edge (364 of 5,582 and 116 of 2,044 in the two larger worlds); in seed
11, 18 to 21% of plastic edges sit at their clamp, and sink edges reached
50.8% transiently at sample 7; four reward-modulated edges exist in total.
Plasticity is a minority trait, and a clamped weight stays clamped until
activity drives it back.

## 2. Evidence

Full per-source sections, equations and quotes are in the appendices:
[artificial life](weight-decay-research-2026-10-19/alife-sources.md),
[neuroscience](weight-decay-research-2026-10-19/neuroscience-sources.md),
[machine learning](weight-decay-research-2026-10-19/ml-sources.md).

### 2.1 What the weight decays toward

| Source | Form | Target in absolute weight |
| --- | --- | --- |
| Hinton & Plaut 1987 | fast weight "decays towards zero by 1% after each weight update" on top of a slow weight | the slow weight |
| Ba et al. 2016 | `A(t) = λA(t−1) + η·h·hᵀ` on top of fixed weights | the fixed weights |
| Miconi et al. 2018 | `w + α·Hebb`, `Hebb ← η·x_i·x_j + (1−η)·Hebb` | `w` |
| Soltoggio 2015 (HTP) | `W = w_st + w_lt`, `ẇ_st = −w_st/τ_st + m·E` | the long-term part |
| Clopath et al. 2008 | `w_i = w̄(1 + h_i − α·l_i + β·z_i)`, tags reset stochastically | the consolidated baseline |
| Zenke et al. 2015 | each synapse "'remembers' its baseline value" through a reference weight | the reference weight |
| Leimer et al. 2019 | `w = w_f + w_s`; the fast part "converges to baseline value" | the slow part |
| Kumar et al. 2023 (L2 Init) | L2 penalty toward the initial parameters | the initial weights |
| Loewenstein et al. 2011 | log spine size as two Ornstein–Uhlenbeck processes (τ 2.9 d and 212 d) | a population mean, not the spine's own size |

Every row but the last is decay toward the synapse's own prior value; the last
is the one counter-example and concerns spontaneous spine volatility, not the
loss of a learned change. Decay of the whole weight toward zero corresponds in
biology to synapse elimination (Richards & Frankland 2017's mapping of "weight
elimination" to Rac-mediated spine shrinkage), which removes innate wiring
rather than forgetting a learned change. Kumar et al. tested the target
directly: regularizing "towards the initial parameters rather than sampling a
new set of parameters at each time step performs much better" (§ ablations).

### 2.2 Whether the rate is heritable and per synapse

- Brea et al. 2014: "optimal behavior in a stochastically changing environment
  requires a forgetting rate that is adapted to the time constant of the
  changes"; approach and avoid synapses decay at different rates, and "the
  forgetting rates can be considered to be genetically encoded."
- Abraham 2003: three decay families; "the persistence of LTP will vary across
  brain regions, perhaps according to the information processing requirements
  of those regions" (p. 737).
- Bullinaria 2009: the only evolved decay rate found converged consistently
  (global, supervised task).
- FADE (Ramesh, Lewandowski & Schmidhuber, 2026 preprint): learned per-weight
  decay rates separated irrelevant weights (λ ≈ 0.9) from relevant ones
  (λ ≈ 0.02); decay toward zero, output layer only.
- Grain caveat: in flies and mice the rate is set per neuron or compartment
  (Rac1 level, one dopamine neuron per mushroom-body zone), and Clopath et al.
  argue "it is the neuron as a whole … which 'decides'". Per edge is finer than
  the biology. It is chosen here because T25 made the edge the domain object
  that carries every other learning attribute, and F07's scope coin already
  lets a group change an attribute on all its members at once.
- No source found evolves a per-synapse decay rate; that is an absence in the
  search, not proof.

### 2.3 Consolidation

Consolidation slows forgetting by moving the change into a slow component; it
does not make it immune. "Establishment of the late-phase does not fully
protect the LTP against subsequent reversal" (Abraham 2003), and Davis
& Zhong (2017) describe consolidation as "in constant competition with a
'homeostatic' signal from intrinsic forgetting." Leimer et al. (2019) give the
cheapest form: the decay flux of the fast part moves into the slow part only
while a gate is open, so consolidation adds one state per edge and no new
rate. The one-component decay toward a fixed `b` recommended here is that
model with the gate closed. Its own control without decay (Petri today)
suffered interference: performance in an earlier context "decayed roughly to a
third of the original one."

### 2.4 The case against, stated at full strength

- The clamp already bounds runaway, and Backpropamine's clip matched or beat
  decay (Miconi et al. 2019, p. 3). But its plastic state reset every episode;
  the clamp bounds Petri's walk without returning to the prior.
- Yaman et al. (2021) found evolution prefers rules that stop changing weights
  once the task is learned, because continued change causes forgetting. Decay
  at `ρ = 1` is that option, and the recommended draw keeps it reachable.
- In continual deep learning the evidence is mixed: L2 Init forgets more than
  EWC in Kumar et al.'s own appendix (backward transfer −63.4% vs −39.0%);
  Elsayed et al. (2024) found pulling toward any point overwrites useful
  weights, and Lyle et al. (2023, 2024) found weight decay alone did not
  restore plasticity. These are large gradient-trained networks; their failure
  modes (dead units, rank collapse) transfer weakly to small evolved Hebbian
  graphs.
- Evolved neuromodulated agents (Soltoggio et al. 2008) handle reversal by
  gated relearning, not passive decay. Petri's reward-modulated edges are
  almost absent today (four edges), so the reward-gated route has little to
  act on yet.

## 3. Probe: the random walk on Petri's update law

A scratch `#[ignore]` test in `runtime/plasticity/tests.rs` (reverted, not
committed) ran one reward-modulated Classic edge (genomic weight 0.5, clamp 2,
`trace_decay` 0.9) for 20,000 ticks, 20 seeds per arm: each tick a visit with a
uniform input, a reward pass, then `w ← b + ρ·(w − b)` through the fixture
`set_slot`. Arm A gives a zero-mean ±1 reward independent of the input (pure
coincidence); arm B makes the reward follow the input 30% of the time (a real
contingency). Arm A reports the mean `|w − b|`, arm B the mean signed `w − b`.

Learning rate 0.005:

| ρ | A at 100 | A at 1,000 | A at 20,000 | B at 20,000 (kept vs ρ = 1) |
| --- | ---: | ---: | ---: | ---: |
| 1 (today) | 0.19 | 0.61 | 1.38 | +0.55 |
| 0.999 | 0.19 | 0.39 | 0.58 | +0.49 (90%) |
| 0.99 | 0.11 | 0.09 | 0.14 | +0.10 (18%) |
| 0.9 | 0.03 | 0.05 | 0.03 | +0.01 |

At learning rate 0.05 the walk reaches the clamp scale within 100 ticks in
every arm but `ρ = 0.9` (mean 0.38 to 0.51), and arm B's signed shift is
indistinguishable from noise: large steps defeat any retention slow enough to
keep learning.

Readings: without decay the coincidental walk grows roughly as `√t` and
reaches the clamp scale within a lifetime, which is HTP's information loss in
Petri's own law. The stationary spread under decay scales as
`step / √(2(1 − ρ))`, so a useful retention depends on the learning rate and on
how long a real contingency lasts, which is Brea's rate-matching argument in
numbers. Both rates are already heritable per edge; the retention belongs
beside them. The probe measures one edge with a synthetic reward and says
nothing about selection; the feature's goal run is that reading.

## 4. Options

| Option | Natural analog | Evidence | Verdict |
| --- | --- | --- | --- |
| A. Status quo: clamp only | stable LTP lasting months (Abraham) | Backpropamine clip ≥ decay; Yaman | Rejected as the only mechanism: the walk is unbounded below the clamp and nothing returns to the prior (Section 3). Kept reachable as `ρ = 1`. |
| B. Per-edge decay toward the slot's birth value | early LTP returns to baseline; intrinsic forgetting returns the brain "to its basal state" (Davis & Zhong 2017); the *C. elegans* AVA synapse reverts to its naive state (Richards & Frankland 2017) | Section 2.1 rows 1–8; Brea; Bullinaria; probe | **Recommended.** |
| C. Per-edge decay toward the genomic weight | as B | as B | Rejected: for a Lamarckian edge it erases the inheritance at rate `ρ`, making two mechanisms for one concept (the Lamarckian flag and `ρ`). B and C are identical on Darwinian edges. |
| D. Decay of the whole weight toward zero | synapse elimination | none for forgetting a learned change; Bullinaria's evolved λ went to zero | Rejected: erases innate wiring, which the biology does not do. |
| E. Neuron-wide multiplicative scaling | Turrigiano synaptic scaling; sleep downscaling (Tononi & Cirelli 2014) | strong, but it "normalize[s] firing without changing the relative strength of synaptic inputs" (Turrigiano 2008) | Out of scope: activity homeostasis, a separate feature; it preserves what was learned. |
| F. Energy cost on learned deviation | "stronger synapses consume more energy" (Tononi & Cirelli 2014) | not tested | Rejected: the plasticity cost is 0 and its machinery was deleted by user decision (T25 D9, 2026-10-13). |
| G. Two-component weight with gated consolidation | tag-and-capture; Leimer's fast and slow parts | Clopath 2008; Ziegler 2015; Leimer 2019 | Later extension of B: one extra state per edge, gated on an outcome signal, no new rate. Not now: reward-modulated edges barely exist. |

## 5. Recommended design

- **Attribute**: a heritable retention `ρ` in `PlasticityConfig`, range
  `[0, 1]`, `1.0` meaning no decay. The spec names it. It applies to every
  plastic edge, Hebbian and reward-modulated alike.
- **Law**: once per world tick, beside the trace decay in `begin_tick`,
  `w ← b + ρ·(w − b)`. The learning update acts on top, as in Leimer's rule.
- **Anchor `b`**: the slot's birth value, computed once at construction: the
  clamped seed for a seeded Lamarckian slot, else the genomic weight clamped to
  `±weight_clamp`. Clamping the genomic anchor at construction removes the
  decay-versus-clamp fight noted in Section 1.
- **Founder-neutral**: the founder carries no plastic edge, and every
  plasticity-enabling draw starts at `ρ = 1` or draws `ρ` with most mass at or
  near 1. A retention operator mutates `1 − ρ` in log space (FADE's
  parameterization; Abraham's decay families span four orders of magnitude).
  The F07 scope coin applies to it like every other learning attribute.
- **Group scale**: after F07 the learned scale of a group decays toward its
  birth scale by one group retention under the same law, under the group's
  own `inherit_scale` rule. This is why the feature follows F08: it edits the
  same catalog and plasticity layout, and the T25 chain is linear by design
  (D10).
- **Numerics**: with `w` stored absolute, the decay stalls once
  `|w − b|·(1 − ρ)` falls under half an ulp of `w`, a residual of at most
  `ulp(w) / (2(1 − ρ))` (about `6e−5` at `|w| ≈ 1`, `ρ = 0.999`). The spec
  decides whether that residual is acceptable or the slot stores the deviation.
  An eager per-tick multiply keeps determinism; a lazy `ρ^Δt` form would need
  integer exponentiation to stay bit-identical across platforms.
- **Tests first**: decay converges to `b` and never crosses it; `ρ = 1` is
  byte-identical to today; a Lamarckian slot decays toward its seed, not the
  genomic weight; a genomic weight above the clamp yields a clamped anchor;
  Oja plus decay stays bounded under both pulls (two attractors, one toward a
  normalized weight and one toward `b`).
- **Readings**: the distribution of `ρ` among plastic edges at goal close;
  at-clamp share and drift beside the F06 census; plastic-carrier share.
  Expected from the Baldwin effect: in stable worlds selection may favor
  `ρ` near 1 or encode the learned value genetically (Floreano & Urzelai 2000;
  Soltoggio et al. 2018); that is a reading, not a failure.

## 6. Remaining uncertainty

- Whether selection moves `ρ` off 1 at all when plastic edges are 6% of
  creatures and reward-modulated edges number four. The feature can be
  correct and inert; its goal run decides, and a later consolidation feature
  (option G) is pointless until reward-modulated edges exist.
- Per-edge against per-node grain (Section 2.2).
- Lamarckian drift: decay toward the seed has no path back to the genomic
  weight, so learned values can accumulate across generations; selection on
  the seed is the only check. Sasaki & Tokoro (1999, abstract only) found
  Darwinian inheritance best in dynamic worlds; Luo et al. (2024, full text)
  found Lamarckian better under between-generation change.
- Gated decay (Zenke et al.'s burst-gated return; Turrigiano's "useful if it
  could be selectively turned on or off") is not covered by a constant `ρ`.

## Sources and how they were read

Forty-seven sources were read in full text by three research passes on
2026-10-19 (arXiv and PMC PDFs extracted with PyMuPDF; publisher pages and
PDFs behind bot checks read in the browser). Decisive quotes in this note were
re-checked: Bullinaria 2009, Soltoggio 2015, Miconi et al. 2019, Kumar et al.
2023, Brea et al. 2014, Davis & Zhong 2017, Turrigiano 2008 and Zenke et al.
2015 against the extracted text; Leimer et al. 2019 on the bioRxiv full-text
page and Abraham 2003 on the PMC PDF, both in the browser. Citations, URLs and
the per-source quotes are in the three appendices.

Not verified in full text and labeled as such where used: Frey & Morris 1997,
Fusi, Drew & Abbott 2005, Hadziselimovic et al. 2014, Sasaki & Tokoro 1999,
Kraemer & Golding 1997, Redondo & Morris 2011 (abstracts or second-hand);
Richards & Frankland 2017 was read in full in the browser by the research pass
and not re-checked here; Dohare et al. was read as the arXiv v3 preprint, not
the *Nature* version; the FADE result is a 2026 preprint.
