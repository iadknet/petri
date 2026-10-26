# Learning as one type on edges and bundles

**Date**: 2026-10-22
**Question**: After T25.F11 makes the bundle the one concept for wired
sensor fields, do sensors, sinks, edges and bundles keep clean layers, and
who owns learning when a member may update itself or its bundle?
**Method**: the genome and runtime code read on `main` at 1967d95c
(`creature/genome/mod.rs` `PlasticityConfig`, `creature/genome/cgp/edge.rs`
`GraphEdge`, `Membership`, `Scope`, `creature/genome/cgp/group.rs` `Group`,
`runtime/plasticity/learned.rs`); the companion
[sensor wiring note](group-creation-unification-2026-10-22.md).

## Finding

Sensors (`contracts/inputs.rs`: fields, field sets, geometry) and sinks
(the vote catalog; a kind groups nine) carry no weights or learning and are
clean. Edges and, after T25.F11, bundles are clean for structure. Learning
is not:

1. **A bundle owns no learning configuration.** `PlasticityConfig` (rule,
   rate, clamp, `lamarckian`, modulation, retention) exists only on edges.
   A group's scale learns only when some member carries `scope = Group`,
   and then borrows from those members: its clamp is the smallest clamp
   among them (`learned.rs`, `from_seeds`), its rule and rate are each
   crediting member's own.
2. **One idea, two fields.** Passing learned state to a child is
   `lamarckian` on the edge's config and `inherit_scale` on the group;
   relaxing toward the birth value is `retention` on the edge's config and
   `retention` on the group.
3. **Uniform creation is a copy, not inheritance.** F07 uniform creation
   writes the same config onto every member; later edits make members
   diverge, and nothing expresses "follow the bundle unless overridden".
4. **The update target is decided in many places.** A member's stored
   `Scope` decides whether it updates itself or its group's scale, and
   `scope == Group` branches read it in `learned.rs` (learner layout,
   seeds, Lamarckian seeding, decay, initialization, the view) and the
   plasticity census. Separately, the mutation layer's scope coin
   (`hebbian.rs` `scope_targets`) decides whether an edit lands on one
   member or every member; it does not decide what learns.
5. **Learner discovery is edge-only.** Runtime construction, structural
   classification and the census find learning by asking which edges carry
   plasticity (`learned.rs`, `companions.rs`, `plasticity_census.rs`), so a
   scale cannot learn unless some edge is plastic.

This is the drift pattern of the sensor wiring note one layer down: one
concept (what learns, by which rule, and what it updates) decided by
several sites with their own rules.

## Decision (user, 2026-10-22)

The user's model, in their words (a side discussion pasted as the answer to
"what happens to a member with no learning settings of its own"):

- "edges without their own learning settings either: inherit their bundles
  weights, with their own evolvable static modifier to those weights. if
  the bundle is set to have learning edges, or an individual edge overrides
  its own learning settings, then the edges weight is a combination of the
  bundle's weight and the edges learned modifier."
- "a bundle that bundles would learn 'as a group' sometimes."
- A member may credit its bundle and learn itself at once ("Yes").
- "In general for questions like this, I want to bake in as few
  assumptions and constraints as possible and allow evolution maximum
  flexibility, while also maintaining clean abstraction layers."

## Model

Revised after an architecture challenge (Codex Astra, 2026-10-22) of the
domain diagrams: two learning slots on a bundle, a learning layout derived
from the genome rather than owned by runtime, and `Inherit` resolved once
at birth.

### Three meanings of "inherit", kept apart

| Word | Meaning | Where |
|---|---|---|
| **default settings** | a member with policy `Inherit` uses its bundle's member default `Learning` for its modifier | the bundle's `member_default` slot |
| **Lamarckian** | a learned value is a child's birth value | one `lamarckian` flag on every `Learning` (replaces `inherit_scale`) |
| **shared scale** | a member's weight moves when its bundle's scale moves | always: `weight = scale × pattern + modifier` |

### Objects (genome data)

**One `Learning` value type**, the same wherever it appears: rule, learning
rate, clamp, reward modulation (source, trace decay), retention, and the
`lamarckian` flag. It replaces `PlasticityConfig` on edges and the paired
`inherit_scale` / group `retention` fields on bundles.

| Object | Stores | Learnable number |
|---|---|---|
| Lone edge | `Option<Learning>` | its weight |
| Bundle | two slots of the same type: `scale_learning: Option<Learning>` ("learns as a group") and `member_default: Option<Learning>` ("learns per edge") | its scale |
| Member | its evolvable **modifier** (today's own correction); a **policy**: `Inherit`, `Static` or `Own(Learning)` (the override); a `credits` switch | its modifier |

Two slots rather than one shared `Learning` plus two switches (assistant
proposal, by the maximum-flexibility rule): the scale's learning and the
members' default evolve independently, and a slot's presence is its
switch, so no switch can be on with nothing to run. Every member's weight
is `scale × pattern + modifier`, in every case.

The graph remains the aggregate root that stores every edge; a member is
an edge whose membership links it to a bundle, identified by its
occurrence (surface and position), not stored by the bundle.

### Who learns what

A member's override always wins:

| Bundle slots | Member with policy `Inherit` | Member with `Own` (override) |
|---|---|---|
| neither (static bundle) | modifier stays as evolved | learns its modifier with its own `Learning` |
| `scale_learning` ("learns as a group") | modifier stays as evolved; the scale learns from crediting members | learns its modifier with its own `Learning`; the scale also learns |
| `member_default` ("learns per edge") | learns its modifier with `member_default` | learns its modifier with its own `Learning` |
| both (assistant proposal) | learns its modifier with `member_default`; the scale also learns | learns its modifier with its own `Learning`; the scale also learns |

A member with policy `Static` keeps its modifier as evolved under every
bundle setting (assistant proposal: an explicit static override, so "stop
this member learning" never means "fall back to the bundle"). `credits`
(per member, evolvable) says whether the member's activity feeds the scale
when `scale_learning` is present, with or without an override, so a member
may credit the bundle and learn itself at once.

### Learning layout (derived from the genome) and runtime (values)

Two owners, split by what they hold:

- **Learning layout**: a pure analysis of the finalized genome, computed
  once at birth after the child's mutation and before construction. It
  resolves each member's policy against its bundle (so `Inherit` is looked
  up once, not per visit; the genome still stores `Inherit`), and yields
  every learner with its identity and effective `Learning` (lone edge
  weights, member modifiers with an effective `Learning`, bundle scales
  with `scale_learning`) and, separately, each learner's contributing
  occurrences. It lives with the genome analysis, so structural
  classification (`creature/genome/companions.rs`), the plasticity census
  and runtime construction all read it without the genome depending on
  runtime.
- **Runtime** (`runtime/plasticity`): learned values, traces, birth values
  and seeds, and the update law below. It consumes the layout.

A scale's state does not depend on having contributors (assistant
proposal, by the maximum-flexibility rule): a scale whose last crediting
member was switched off, for example by a child's mutation after the
parent's seeds were captured (`simulation/actions/reproduction.rs`), keeps
its learned value, still decays by its retention and still passes to
descendants when `lamarckian`; it only stops receiving credit. Today's rule
allocates a scale slot only when a group-scope contributor exists
(`learned.rs` `from_seeds`). Every gate that asks "does an edge have
plasticity" is replaced by a layout read, named in the spec, beginning with
`learned.rs`'s empty-state return when no edge is plastic and its learner
lookup, `companions.rs`'s `plastic_edges()` use and
`plasticity_census.rs`'s carrier marking. The scattered `scope == Group`
branches go with `Scope`.

### Update law (today's scalar formulas, one owner; the configuration choice changes)

Two steps, kept distinct: **contribution aggregation** (per learner: one
edge's activity for a modifier or a lone edge; the pattern-weighted sum
over crediting members for a scale), then **scalar state update** (one
add, one clamp, or one trace accumulation per learner). Within one
committed visit, from the same pre-update state:

- **Modifier** and **lone-edge weight**: the effective `Learning`'s rule is
  evaluated on the edge's `pre`, `post` and its composed visit-start
  weight; the delta is added and clamped by that `Learning`'s clamp; a
  reward-modulated `Learning` adds the activity to that learner's trace
  instead.
- **Scale**: for each crediting member, `scale_learning`'s rule is
  evaluated on that member's `pre`, `post` and composed visit-start weight
  (the rule receives that weight, which Oja uses; covariance ignores it, as
  `rules.rs` does today); the deltas are multiplied by the member's pattern
  and summed, not normalized; the scale is written once after the loop,
  clamped by `scale_learning`'s clamp. A reward-modulated scale
  accumulates the pattern-weighted activities in one trace. This is a
  behavior change: today each crediting member evaluates its own rule,
  rate, reward source and trace, and the scale's clamp is the smallest
  among them.
- A member that learns its modifier and credits the scale contributes to
  both from the same samples.

### Detach and other membership transitions

When a member leaves its bundle (`DetachGraphEdge`, a source-changing
edit, a split), it becomes a lone edge:

| Before | Lone edge's weight | Lone edge's `Learning` |
|---|---|---|
| `Own` | composed weight | the override |
| `Inherit`, `member_default` present | composed weight | a copy of `member_default` |
| `Inherit`, no `member_default` | composed weight | none (static) |
| `Static` | composed weight | none (static) |

`credits` is dropped. The birth seed composes as today
(`scale × pattern + modifier` seed) and is consumed only when the resulting
`Learning` is `lamarckian`. The container keeps seed alignment; the runtime
interprets inheritance. Detaching the last member removes the bundle.

### Mutation

Two separate draws, never confused:

1. **Which `Learning` is edited**: a lone edge's, a member's override, a
   bundle's `scale_learning` or its `member_default`. Each existing
   learning operator (enable, disable, rule, rate, Lamarckian toggle,
   reward source, trace decay, retention) applies to any of the four. On a
   member, enable sets `Own` (drawing a `Learning`), disable sets `Static`,
   and a separate "clear override" sets `Inherit`; on a bundle slot,
   enable and disable add or remove that slot. Disabling one member changes
   neither its siblings nor the scale's credit.
2. **Broadcast** (the F07 scope coin, unchanged in meaning): an edit
   picked on a member lands on that member alone or on every member of its
   bundle that admits it (T25 audit row T16: "change this attribute for
   all edges in the group, or … for a single edge").

New toggles: `credits` and the member policy (member, with the broadcast
coin). `SetScope` is removed with `Scope`. Bundle creation draws both slots
and every member's policy and `credits` in the F07 uniform or F08
randomized form. Mutation makes these random choices; the graph domain
validates and applies them.

## Shape of the work

The final seams come first (assistant proposal, after the architecture
challenge): the learning layout and the update-law owner are defined in
their final places before any behavior moves, so the preservation stage
does not shape the abstraction. Two commits:
1. **Extraction, behavior preserving.** The layout and the update-law
   owner over today's representation and today's execution: each crediting
   member keeps its own configuration, trace, reward source and rate; a
   scale keeps the smallest-member clamp and allocates only when a
   group-scope contributor exists; a group-scope member's own correction
   stays unlearned. These old semantics live inside the layout's builder
   and are deleted in commit 2; no switch or config preserves them. Every
   pin holds. Preservation cases written first: a randomized group whose
   contributors differ in rule, rate, reward source and trace decay; mixed
   reward sources on one scale; group-scope members with nonzero own
   corrections.
2. **The new ownership model**: the two bundle slots, member policy and
   `credits`, `lamarckian` unification, scale state independent of
   contributors, the detach table, the mutation draws; with predeclared
   re-pins.

Acceptance cases for commit 2, written first: a bundle-only learner (every
member `Inherit`, `scale_learning` present) learns; an `Inherit` member
under a learning scale moves only through the scale; `member_default`
gives `Inherit` members its settings and leaves `Own` and `Static` members
alone; the two slots evolve independently (an edit to one leaves the other
unchanged); a member that credits and learns itself updates both from one
visit's samples; disabling one member leaves its siblings and the scale's
credit unchanged; a child whose last crediting member was switched off
keeps, decays and passes on the inherited scale; each row of the detach
table; Lamarckian seeding of a scale and of a modifier; the layout is
computed once per birth and `Inherit` is never resolved during a visit.

## Settled

- T25.F11 lands first: its rename gives this row the bundle object, and
  F11 keeps today's learning representation uniform across wirings.
- No backward compatibility: genomes, seeds and pins move, predeclared.
- Plasticity update cost stays 0 (user, 2026-10-13).

## Open for the spec owner

- Birth draws for the two bundle slots and `credits` (default: each a fair
  coin, by the maximum-flexibility rule).
- Verification: viability first, gate + goal; slow tests run (the mutation
  gate left the workflow on 2026-10-22); the acceptance cases above.
