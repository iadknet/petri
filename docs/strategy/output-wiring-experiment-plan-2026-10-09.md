# Output and memory wiring: research and experiment plan (2026-10-09)

Date: 2026-10-09. Status: plan, not yet run. Question raised by the user after
T20.F16 closed: the born edge wires a new sensor into an action vote only; do
the other output surfaces (shared-memory writes, custom outputs, router gates,
action parameters) suffer the same under-use the vote surface did before
T20.F16, and should a born edge be allowed to land on them? The user proposes
an experiment. This note inspects the local evidence first, then the external
precedent, then lays out the options and one predeclared run under the
[T22 exploration contract](../archive/program-1/roadmaps/t22-capability-assays-and-evolvability-lab.md),
in the form of the [born-connected run](born-connected-run-2026-10.md).

## Problem and constraints

- **Problem.** A Graph node's 94 sinks split into 27 `ActionVote` sinks (the
  only born-edge destination since T20.F16) and 67 others: 24 `CustomOutput`
  (the bus the next mesh node reads as `UpstreamSlot`), 8 `RouterGate` (which
  successor runs), 16 `WriteSlot` and 16 `ClearSlot` (shared memory), 3
  `ActionParam`. If those surfaces are reachable only by the old two-step path,
  memory, routing and internal signalling may stay under-used for the same
  reason sensors did.
- **Constraints.** The not-forcing checks (general rule, natural analog, no
  task-solving correspondence, nothing assay-dependent, what variation still
  discovers); one row per run with matched controls; no production default
  change outside a feature; the T20.F02 neutral-connection contract, which
  qualified zero-weight edges onto vote sinks only.
- **Decision criteria.** Is there evidence the other surfaces are under-wired
  (the same defect) or wired-but-inert (a different defect)? Which surfaces
  can carry a sensor's value at all? What would a wider destination class cost
  the vote-side gain T20.F16 bought? What can be measured with instruments
  that exist?

## Local evidence (inspected 2026-10-09)

**What each sink does** ([graph backend spec](../reference/v3-graph-backend-spec.md),
the sink semantics table): `CustomOutput(slot)` sets `output_slots[slot] =
wsum` (slot under 12; an unwired slot keeps the incoming bus value);
`WriteSlot(slot)` sets `shared_memory[slot] = sanitize(wsum)`; `ClearSlot(slot)`
sets the slot to 0.0 **and ignores `wsum`**; `RouterGate(slot)` sets a gate
score; `ActionParam(field)` overwrites a decoded parameter. Votes add; every
other sink **overwrites**. That is why T20.F02 kept them out of the neutral
contract ("an empty clear/store/output/parameter sink can clear or overwrite
nonzero incumbent data when wired"). A `ClearSlot` sink cannot carry a sensor's
value at all: whatever is wired to it, the slot is zeroed when the node runs.

**Memory is already wired, by one-event operators.** `AddGraphEdge` draws its
surface uniformly over compute inputs and all 94 sinks, and its source over
compute nodes (50 %), input leaves (30 %) and shared memory (20 %), so a memory
write (any source onto a `WriteSlot`) and a memory read (a `SharedMemory`
source onto any surface) are each one event today. The two-step defect of
sensors (declare, then connect) does not apply to memory or outputs: a memory
loop needs two events, but neither is a silent intermediate that a janitor
operator deletes at 1.9 × 10⁻² per birth (E4).

**Memory is written and read, and behaviorally inert at 2,000 ticks.** The
T20.F16 goal summary (`docs/progress/features/t20-f16-born-connected-input-declaration-goal.json`,
tick 2,000, Orchards 11 / Canyon 22 / Confluence 33):

| Reading | Orchards | Canyon | Confluence |
| --- | ---: | ---: | ---: |
| Genomes with a reachable memory write (`structural_companions.writes_shared_memory`) | 3,890 of 4,423 (88 %) | 3,088 of 7,582 (41 %) | 836 of 6,048 (14 %) |
| Genomes with a reachable memory read | 1,777 (40 %) | 3,025 (40 %) | 983 (16 %) |
| Creatures whose live walk reads shared memory at the last sample | 1,498 | 2,919 | 2,266 |
| Memory writes that changed a slot's value over the run | 2,423,584 | 382,833 | 55,534 |
| **Creatures whose final action differs with memory zeroed or scrambled** (`memory_sensitivity`) | **0 of 4,423** | **0 of 7,582** | **0 of 6,048** |
| Temporal sensitivity, Orchards: operator state / persisted outputs / previous slots | 0.34 % / 1.33 % / 0 | | |
| Offspring per creature by cognitive class, Orchards: none / shared memory / plasticity / stateful | 0.965 / 1.006 / 0.957 / 0.917 | | |

So the other surfaces do not have the sensors' defect (unwired); they have a
different one: **wired but inert**. Memory writes are everywhere, memory reads
are common, and no creature's action depends on what memory holds.

**Horizon matters.** The [live survey](live-survey-2026-09-16.md) of a user
world at 281,405 ticks (an older substrate) found memory zeroed changed the
action in 117 of 400 creatures (29 %) and scrambled in 100 (25 %), and routes
varying with input in 20 to 40 % of creatures while only 2 nodes contributed to
behavior and route variation selected among detours producing the same
actions. At 2,000 ticks the sensitivity is zero; at 281,000 it was a quarter.
Whether memory use appears by 10,000 or 20,000 ticks is unmeasured.

**No reading covers custom outputs, router gates or action parameters.** No
neighborhood or bench reading names `RouterGate` or `CustomOutput`; the input
funnel (T20.F01) and the vote-delta reading (follow-up run, row N3) read input
references only. Whether a sensor value ever reaches the bus or a gate, and
whether that changes an action, has never been measured.

**What T20.F16 established on the vote surface** (its spec and the born-connected
run): a born vote edge makes target reads causal at the goal checkpoints
(Orchards 4/4, Canyon 3/3 against 0/0) and decided passes rise 78 ×; the
born edge's useful share is 2/189 on the founder; the destination class was
the user's B1 contract and was not compared against any other class.

**The memory-motif capstone exists and is next by the priority rule.** T11.F10
("Can memory be found? In small temporal tasks, measure whether founders reach
and retain useful remembered decisions through viable mutations") is the first
unchecked ID in the order of new starts whose dependencies are all checked; it
runs on the T22 lab as a memory-motif assay. The lab has no temporal assay yet
(its assays are food seeking, wall and ring, all reactive).

## External evidence (searched 2026-10-09)

- **Neuroevolution adds connections to any surface.** NEAT's add-connection
  mutation joins any two previously unconnected nodes, hidden or output, with
  no destination preference
  ([Stanley and Miikkulainen 2002, summarized](https://en.wikipedia.org/wiki/Neuroevolution_of_augmenting_topologies);
  [the complexification paper](https://arxiv.org/pdf/1107.0037)). Precedent
  for a uniform destination class over every surface, not votes only; NEAT
  has no overwrite sinks, so its precedent says nothing about clears.
- **Memory gets used when a task requires it.** Markov Brains wire inputs,
  hidden (memory) states and outputs uniformly, and evolve representations in
  the hidden states when the task needs memory (Marstaller, Hintze and Adami
  2013, [arXiv 1206.5771](https://arxiv.org/pdf/1206.5771); the
  [Markov Brains introduction](https://arxiv.org/pdf/1709.05601)). The
  mechanism to reach memory is the same as to reach outputs; what makes memory
  load-bearing is selection on a temporal task, which Petri's goal worlds may
  or may not supply at 2,000 ticks.
- **The analog allows any existing cascade.** Duplicated opsins came to couple
  to alternative G-protein pathways (ciliary, rhabdomeric, Go-coupled); a
  receptor's downstream pathway is itself re-coupled over evolution
  ([PMC3528684](https://pmc.ncbi.nlm.nih.gov/articles/PMC3528684);
  [PMC2013938](https://pmc.ncbi.nlm.nih.gov/articles/PMC2013938)). "Expressed
  into an existing cascade" does not single out the motor end; a memory or
  bus surface is as legitimate a cascade as a vote.

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| **A. Measure first** (`instr:`) | A **sink-use funnel**, the output analog of the input funnel, on the census probe's selected cohort: per sink kind, the parents with a wired sink on a reachable node (declared), with that node executing on the panels (executed), and whose committed action queue changes when the sink's edges are ablated (causal); a **memory-loop census** (a write to slot k and a read of slot k on the same genome, each stage), and `memory_sensitivity` at the 10,000 and 20,000 checkpoints. | Answers the actual question (wired-but-inert, or unreachable, and whether horizon fixes it) with the instruments that exist; reuses the vote-delta machinery (ablation, action-queue comparison); cheap to run. | Measures only; no mechanism tested unless an arm is added. |
| **B. Widen the born edge's destination** (`proto:`) | `mutation.born_connected_destinations`: `votes` (production) or `non_clear` (uniform by index over the 78 sinks that use their input: 27 votes, 24 outputs, 8 gates, 16 writes, 3 parameters; `ClearSlot` excluded because it ignores its input, not for safety). The user's proposal. | One general rule, same analog, NEAT precedent; tests whether sensors born into memory, the bus or a gate become causal there. | Dilutes the vote-side gain (27 of 78 born edges reach a vote, so T20.F16's causal counts should fall by about two thirds); an overwrite sink replaces incumbent data with the sensor's value, so harm per fired birth may rise (E5 found whole-family wiring harmful 4 of 5); memory inertness looks like a loop-closure and selection problem, which a wider destination does not touch. |
| C. Born-closed memory loop (`proto:`) | When a write edge lands on slot k, the same event adds a read of slot k somewhere on the node (or the reverse). | Attacks the pairing step directly (16 slots: a random read matches a random write 1 in 16). | Pairs a source and a sink by index: borderline against not-forcing check 3 even though it names no meaning; no evidence yet that pairing is the block rather than selection; wait for A. |
| D. Wait for T11.F10 | The capstone's lab memory-motif assay. | The right instrument for "can memory be found" under selection that needs memory. | Needs a new temporal assay in the lab; says nothing about outputs or gates in the natural world. |

## Recommendation: A with one B arm, as one predeclared row

Measure first, because the evidence says the other surfaces are not under-wired
but under-used, and no reading can yet say why; add the one mechanism arm the
user asked for in the same run, since the instrument is the expensive part and
the arm costs two world runs. C waits on A's loop census. D stays the roadmap's
next feature; this run informs it.

**Row W1 (draft).** Design as B1: Canyon goal recipe, seeds 1022 and 2022,
20,000 ticks, the census probe with checkpoints 10,000 and 20,000, the input
funnel, the vote-delta reading, the births block, and the new sink-use funnel
and memory-loop census at both; `memory_sensitivity` and the three temporal
sensitivities at both checkpoints; 3 h kill, one job at a time. Arms:
`default` (production: T20.F16 votes only), `wide` (`non_clear` destinations),
and the two mutation-off pairs (identical by construction, verified). Gate W0
on the founder through the E4 probe: `default` byte-identical to T20.F16's
engine; under `wide` every single-event target declaration arrives connected
(1,393 of 1,393 at 10⁷ as B0), destinations spread over the 78 sinks by index
with the vote share's CP interval containing 27/78, requested supply and
`InputRefAdd` applied rate unchanged within 5 %. Gate W0.2 as B0.2: the
census probe on the prototype binary with the flag at `votes` against the
instrument built without the prototype, samples and timing-free blocks
hash-equal. Codex reviews (a) of the predeclaration and (b) of the readings;
the advisor at every rule 8 point.

**Readings and predictions.**

| Reading | `default` prediction | `wide` prediction |
| --- | --- | --- |
| Sink-use funnel: memory writes, outputs, gates, parameters wired / executed / causal on 20 parents | wired on most parents, causal on few or none at 10,000; whether causal rises by 20,000 is the horizon question | causal incidences on `WriteSlot` and `CustomOutput` where `default` has none |
| Memory-loop census: slot-matched write and read, executed, causal | loops present, causal 0 at 10,000 | same or higher |
| `memory_sensitivity` at 10,000 and 20,000 | above 0 by 20,000 if the live survey's 29 % is horizon; 0 if it needs an older substrate or a temporal task | not predicted |
| Target-family rows (declared / connected / executed / causal) | T20.F16's levels | connected still tracks declared; causal on vote sinks about one third of `default`'s |
| Vote-delta block | as B1 | fewer action-changing target channels |
| Births block dead / silent / changed | as B1 | dead not reliably different (underpowered, as B1) |
| Persistence, population, genome size | as B1 | population not under 0.5 × `default`'s; a trough recorded as in B1 |

**Validation rule (draft, for the user to fix in the goal command).** `wide`
is **validated as a lead** if on both seeds at 20,000: `wide` persists; its
window-mean population is at least 0.5 × `default`'s; the sink-use funnel
shows at least one causal incidence on a non-vote sink where `default` has
none; and the vote-side target causal count is not below one third of
`default`'s. **The horizon question is answered apart**, on `default` alone:
`memory_sensitivity` above 0 at 20,000 on either seed means memory use is a
matter of horizon and T11.F10 should read later checkpoints; 0 on both seeds
with loops present means the block is selection or closure, and option C or a
temporal assay comes next. A 20-parent cohort is a lead-selection screen; one
lineage can carry a count.

**What a positive W1 would and would not say.** It would say sensors can be
born into memory and the bus and become causal there in a natural world; it
would not say the widened class is better than votes only (that needs the
usefulness readings, the lab reach and the twin screen the born-connected
run's plan forward named), and the vote-side dilution is the cost to read
against it.

**Cost.** The sink-use funnel and loop census: one session of `instr:` work
on the census probe, reusing `vote_delta`'s ablation and action-queue
machinery (the vote-delta reading took one session in the follow-up run). The
prototype: a destination enum on the born edge, one afternoon with its tests.
Runs: eight census runs of about 11 min (`default`, `wide`) and 2 min (pairs)
each, under 1.5 h; W0 minutes. One goal of about 150 turns, as the
born-connected run.

## Remaining uncertainty

- Whether a sensor value overwriting a memory slot or a bus slot is harmful
  more often than a vote edge: E5 measured whole-family wiring onto votes,
  never overwrite sinks; the births block in W1 reads it descriptively only.
- Whether `memory_sensitivity` at the terminal tick is the right stage reading
  for memory: it compares one final action with memory zeroed or scrambled; a
  memory loop whose effect shows up a tick later is missed. The temporal
  sensitivities cover persisted outputs and operator state, not that case.
- Whether Confluence, where T20.F16's selected cohort carried no target
  declaration at all, would read differently from Canyon; W1 stays on Canyon
  for comparability with B1.

## Goal command (draft)

Open a new session in the main checkout on a clean `main`, model Fable 5.1,
the Fable advisor enabled, Codex for reviews. Paste:

```
/goal An output-wiring experiment run is complete on main. Read docs/strategy/output-wiring-experiment-plan-2026-10-09.md in full (its options, row W1, gates W0 and W0.2, readings and validation rule are the contract's form), then docs/strategy/born-connected-run-2026-10.md (its gates, Advice table and reviews are the precedent), then docs/strategy/evolvability-diagnosis-run-2026-10-followup.md row N3 (the vote-delta instrument the sink-use funnel extends). Confirm Fable 5.1 with the Fable advisor enabled and Codex for reviews, a clean main, and create the worktree with EnterWorktree named output-wiring. Take the census probe, the vote-delta reading, the births block and the E4 probe as file state from branch worktree-born-connected (instr and lab commits, no proto). Build the sink-use funnel and the memory-loop census as instr on the census probe, reading the prototype flag through the serialized config so the instrument builds without the prototype. Predeclare and commit row W1 before it runs: the proto field mutation.born_connected_destinations with votes (production) and non_clear (uniform by index over the 78 sinks that use their input, ClearSlot excluded because it ignores its input), the gates W0 and W0.2 as the plan states them, the arms default and wide with their mutation-off pairs, Canyon seeds 1022 and 2022, 20,000 ticks, checkpoints 10,000 and 20,000, every reading on at both, 3 h kill per run, one job at a time. The validation rule is the plan's draft rule as written; the horizon question is read on default alone and recorded apart. Consult the advisor at every rule 8 point and record it; send the predeclaration to Codex before the row runs and the note's reading before closing. Write docs/strategy/output-wiring-run-2026-10.md with the row, the gates, the Advice table and a plan forward that says whether the widened destination class becomes a roadmap feature or a T20.F16 spec amendment, what the horizon reading means for T11.F10, and whether option C is next. Land docs only; keep proto, instr and lab on the branch. Done means the note is on main with W1 completed, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 150 turns.
```
