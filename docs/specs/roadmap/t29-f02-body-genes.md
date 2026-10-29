# T29.F02 — Body Genes

**Status**: Complete
**Last updated**: 2026-10-29
**Feature**: T29.F02
**Track**: [T29 — Heredity](../../roadmaps/t29-heredity.md)

## Goal

One genome encodes both the body plan and the nervous system (the row's
natural analog): colour, kin tag, capacity, steal share and give share are
five body genes in a body chromosome of `CreatureGenome`, one genome unit
each, mutated by the body's own supply at the brain's per-unit rate with no
gate on brain mutation, and paid for like any unit (carrying, replication,
gestation). `lineage_id` becomes observation-only provenance. The founder
grows from 80 to 85 units, `per_unit_rate` is re-pinned to `0.485 / 85` so
the founder's supply stays 0.485 events per birth, and the replication and
gestation anchors move to 85. Pays G5.

## Non-Goals

- Memory provisioning and uptake masks (T29.F03, G6).
- Retuning body step sizes (`log_step`, the phenotype step); a later sweep may.
- Changing a body rule: move factor, upkeep, sensor scale, energy cap,
  exchange intensity and T26.F02's capacity law stay as they are.
- Body genes in functional `complexity()`, in the brain battery rows
  (`neighborhood/operators.rs` `ROW_DOMAINS`) or in the one-event
  mutation-effect screen: these are brain instruments.
- Making `CreatureGenome.nodes` and `entry_node_id` private (T29.F01 audit A11),
  the L7 node edit transaction, signals in the payload (T27).

## Inputs and Invariants

| Input | Where | What F02 takes from it |
| --- | --- | --- |
| Track row, criterion 2, Notes | `docs/roadmaps/t29-heredity.md` (F02 row; second criterion; "Decisions (user, 2026-10-22)"; "Supersedes"; "F02 predeclares its re-pins"; "Verification") | Five body genes, one law, no gate, `lineage_id` provenance, founder 85, rate re-pin, anchors, per-chromosome supply, uniform gene pick, today's per-trait step, predeclared re-pins, `KIN_AFFINITY` reading, gate + goal + goal-long |
| Design note | `docs/strategy/heredity-architecture-2026-10-22.md` ("Objects and owners", "Mutation supply: one law", "Other moves", "Consequences to accept") | `BodyGenome` owned by its own edit API, a body policy that samples and calls it, colour one gene, carrying cost accepted, equality and serialization include the body |
| T29.F01 | invariants 1, 3, 8, 9, 10 | `Zygote`, `Provenance`, the body policy's draws and arithmetic, conception order, finalization |
| T26.F02, T27.F03 | `creature/body.rs:59-118` | Body rules, the capacity range `[energy.capacity.min, lifecycle.max_energy]`, the share range `[exchange.share_min, 1]`, the log-space step and its clamp |
| T25.F02 | `config/simulation.rs:983-988` | The rate re-pin precedent: `0.485 / founder units` |
| Audit | [readings](../../progress/readings/t29-f02.md), rows B1–B11 | Every row is in-feature |
| Code | `creature/{body,identity,phenotype,birth,state,founder,colour}.rs`, `mutation/{body,phenotype}.rs`, `mutation/engine/mod.rs:22-200`, `mutation/types/mod.rs:15-330`, `genome/mod.rs:236-310`, `config/simulation.rs:620-731,940-1016,1330-1360`, `simulation/{seeding.rs:90-130,actions/reproduction.rs:10-47,actions/{predation,giving,mod}.rs,tick.rs:180-200}`, `neighborhood/{drift,births,mutation_effects/mod,distributions,population,population_sample,clades}.rs`, `v3-cli/src/bench/{tracking,capacity,run,census_pins,mutation_effects}.rs`, `v3-telemetry/src/{trace.rs:426-450,windows.rs:750-770,metrics.rs}`, `v3-server/src/{http/creature.rs,state.rs,types.rs:5}`, `v3-lab/src/eval.rs:111-131`, `frontend/src/components/config-panel/runtime/{MutationSection.tsx,groups.ts}` | What changes |
| Reference specs | `docs/reference/v3-{genome,mutation,phenotype,creature-identity,reproduction,runtime-config,startup-seeding,creature-lifecycle,sensor,server-api-protocol}-spec.md` | Sentences on the body record, per-birth chances, the gate, `lineage_id`, 80 units, `0.0060625` |

Research: three supply shapes give a founder 0.485 events per birth. Per-trait
chances (today) are the second law the user removed. One
`Binomial(genome_size, rate)` with each event landing on a uniformly drawn unit
has the same distribution, but every brain event would first draw its
chromosome, so the brain's stream would depend on body size and F03's masks
would move it again. Per-chromosome draws (the note's proposal) sum to the same
binomial, keep brain dispatch unchanged and give F03 its units without touching
the brain: chosen. Independent per-site mutation at one rate is the standard
per-gene model; no library applies to this internal ownership move.

| # | Invariant |
| --- | --- |
| 1 | **Body chromosome.** `genome/body.rs` owns `BodyGenome` (imports only `contracts`): five genes in `BodyGene` order `Colour`, `KinTag`, `Capacity`, `StealShare`, `GiveShare`, a closed enum with `ALL`, and `BodyGenome::UNITS = BodyGene::ALL.len()`. Colour is one gene: today's six channels, active channel and six polarities (the type moves from `creature::phenotype` with its `FOUNDER` value; its fields private behind read accessors; an active channel outside `0..6` is rejected at construction and at decode). The kin tag is a `u32`; capacity and both shares are `f32`. Gene operations, applied deterministically: the colour step (today's `Phenotype::mutated` arithmetic; its step type moves to `genome/body.rs`), the kin-tag flip of one bit in `0..32`, a scalar's `(value × e^δ).clamp(min, max)` over the gene's range in a given `BodyRanges` (capacity `(min, max)`, share `(min, max)`, plain numbers; today's `inherit_scalar` arithmetic, so every step lands in range and a later step never multiplies a zero), and a clamp of the three scalars to a `BodyRanges`. No `Default`: a body's values come from config (founders) or a parent. |
| 2 | **Genome.** `CreatureGenome` holds `entry_node_id`, `nodes` and a private `body`; `CreatureGenome::new(entry_node_id, nodes, body)` replaces every struct literal; `body()` reads it and `with_body(body)` returns the genome with another body (founders, the lab). `PartialEq`, `Serialize`/`Deserialize` and `Debug` include the body. `genome_size()` is `brain_units() + BodyGenome::UNITS`, `brain_units()` being today's count; `complexity()` is unchanged. |
| 3 | **Supply.** `MutationEngine::apply_mutations_in` draws `Binomial(brain_units, rate)` and runs today's brain loop on that count (pressure draw, two-layer dispatch, gate, mappings), then draws `Binomial(BodyGenome::UNITS, rate)` body events. Each body event draws its gene uniformly over `BodyGene::ALL`, then that gene's draws: colour, the switch draw, the index among the other five channels when it passes, the polarity draw (step `phenotype.channel_step.max(1)`); kin tag, a bit in `0..32`; capacity, a step uniform in `[-capacity.log_step, capacity.log_step]`; each share, the same with `share.log_step`. A birth makes no other body draw: `sample_body_draws` and the `applied_events > 0` gate are deleted. Every engine entry takes the resolved `BodyRanges`, built from config by one function in `creature::body` (`[energy.capacity.min, lifecycle.max_energy]`, `[exchange.share_min, 1]`); every caller passes it (conception, the drift walk, the births and mutation-effect readers, the battery, the lab campaign and readings), so an instrument's mutated body equals the one conception would build from the same draws (B11). `apply_mutations_on_units` takes per-chromosome counts (brain, body): production passes the genome's own, the drift walk the founder's (`brain_units()` and `BodyGenome::UNITS`: 80 and 5), the one-event screen (1, 0). A body event is never rejected and needs no parseability or graph gate (it cannot touch the brain), ignores the size-pressure restriction (it changes no size) and maps every learnable to itself. |
| 4 | **Reporting.** `MutationDomain::Body` and `MutationOperator::Body(BodyOperator)`, one operator per gene, keys `Body.Colour`, `Body.KinTag`, `Body.Capacity`, `Body.StealShare`, `Body.GiveShare`, complexity effect neutral. Each body event is recorded as attempted and applied with no target, no discards and `TargetReachability::NotApplicable`, so `applied_events`, the by-domain and by-operator counts, funnels, value totals by operator, `birth_mutation_operators` and their telemetry families carry body events with no new counter. Facts only the brain has (reachable bias, weights, two-layer dispatch, dispatch to a target) are defined over the brain domains, with no value for `Body` (B10): `dispatch` and `apply_event` take a brain operator type (the four brain variants, convertible into `MutationOperator`, with the brain catalogs and operator weights over it), so no arm or result exists for a body operator. `birth_edge_class` keeps its definition: exactly one applied event. |
| 5 | **Conception.** F01 invariant 9 with the body inside the genome: clone the parent's genome; capture the payload; mutate both chromosomes under the config's `BodyRanges`; clamp the child's three scalars to the same ranges on every birth (today's `inherit_scalar` clamp of an unmutated trait, so a live patch of a bound reaches the next generation without a draw; idempotent when no bound changed, so with one step on a gene the value is today's `(parent × e^δ).clamp(min, max)`, bitwise); derive provenance. |
| 6 | **Provenance.** `lineage_id: u32` moves into `Provenance` (a founder's is its founder index, a child's its parent's) and onto the creature record beside `generation`, written by finalization. No rule, sensor or draw reads it. `CreatureIdentityState`, `HeritableBody` and `Zygote`'s body field are deleted; `Zygote::founder(genome, lineage_id)`. |
| 7 | **Founders.** `founder_genome` and `founder_genome_with_age_gate` take the body; `creature::founder::founder_body(founder_index, startup_seed, config)` gives today's values: `FOUNDER` colour, kin tag `splitmix64(startup_seed ^ founder_index) as u32`, `energy.capacity.founder`, `exchange.founder_share` for both shares. A reader or instrument that builds a founder genome gives it founder 0's body under its own seed and config, or under startup seed 0 when it has no seed (the body is inert there and only fixes its genome's identity); the default-config founder genome carries founder 0's body at seed 0 under the default config. The V3Alpha1 founder is 80 brain + 5 body = 85 units, the ForageFirst profiles 88. `FOUNDER_GENOME_SIZE_UNITS = 85`, pinned to the founder's `genome_size()`, is the replication anchor (founder multiplier exactly 1.0) and the gestation anchor (the founder gestates one tick at rate 1.0). The founder's carrying charge grows by 5 units. |
| 8 | **Rate.** `default_per_unit_rate()` is `0.485 / 85`; the config test pins rate × founder size = 0.485. The frontend's reset value (`MutationSection.tsx:11`) and the founder-size tooltips (`MutationSection.tsx:13`, `EnergyLifecycleSection.tsx:51`) follow. Capacity and each share mutate about 0.0057 times per birth instead of 0.1, colour and kin tag about 0.0057 instead of about 0.39 (founder size). |
| 9 | **Config.** `LogStepMutationConfig` keeps `log_step` and its normalization bound; `chance` is deleted from the schema, generated types, the frontend config panel and its fixtures, `v3-telemetry/src/trace.rs`'s config attributes and the CLI test. The server's `PROTOCOL_VERSION` becomes `v3alpha19` (the config schema on the wire changes). |
| 10 | **Readers of the body.** The body rules (`move_factor`, `upkeep`, `energy_scale`, the energy cap) stay in `creature::body`, reading `&BodyGenome`; sensors get colour and kin tag through `BodyView` as today; predation, giving, the tick's upkeep, distributions, the server, telemetry and the bench read the genome's body and `lineage_id`; every wire field keeps its name and value. A creature's body cannot change after birth: a fixture that needs a trait builds the creature from a genome carrying it (no writer, test-only included). |
| 11 | **Lab.** The lab evaluates every genome with the config founder body (`with_body(founder_body(..))`), as its eval does today, so assays keep a fixed body; campaign genome files carry the body, and the lab's `GENOME_FORMAT` (`v3-lab/src/summary.rs:34`) becomes 12. |
| 12 | **Readings.** A body census in `neighborhood/` over living creatures, formatted by a `body` block in the bench goal sample blocks (`Retention::Kept`, `null` at extinction): `distinct_colours` (distinct channel arrays), `founder_colour_fraction`, `distinct_kin_tags`, and `kin_hamming_within_clade` and `kin_hamming_across_clades`, the mean Hamming distance between kin tags over unordered pairs of distinct living creatures with equal or different `lineage_id` (`null` without such a pair), computed exactly from per-bit counts; a pair's `KIN_AFFINITY` is `1 − h/32`. Capacity and share distributions are unchanged. |
| 13 | **Re-pins**, one cause, predeclared by the row: genome JSON and `Debug` digests (`neighborhood/mutation_effects/mod.rs:57`), every draw stream (engine draw-stream tests; the body draw pin becomes a body-supply pin), `tests/applied_trajectory.rs`, `tests/baseline_worlds.rs`, `tests/viability.rs` digests (founder units and carrying; both the default and the GNU/Linux x86_64 pins, the latter re-measured at the same births with a binary cross-built for `x86_64-unknown-linux-gnu` under emulated Ubuntu 24.04, as T26.F02 and T27.F03 did, since CI runs `rust-viability` on `ubuntu-24.04`), mutation-off runs (rate 0 still runs per-unit trials, no chance draws), census pins, `bench_artifacts` digests, lab fixtures, the slow tier, generated types, `PROTOCOL_VERSION`, the lab's `GENOME_FORMAT` pin, reference specs, and the gate and goal epochs. A re-chosen fixture seed keeps its precondition and assertions. |

## Architecture

Owners touched ([`docs/architecture.md`](../../architecture.md) §3): the genome
gets the body chromosome and its gene operations ("chromosomes", "edits that
validate"); mutation keeps supply, dispatch and every birth draw, now per
chromosome with body operators, and only samples (rule 4); the creature keeps
the body rules, founder body values, the conception clamp (the range in force
is a body rule over config) and provenance; readers add the body census and
outer crates format it. No boundary changes: §3 and §4 already draw the body
chromosome, so the plan does not edit the file; G5 is deleted at closure.
Shared rule "a body trait's value": owner `BodyGenome`; applied today by
`HeritableBody::inherit` (`creature/body.rs:59-80`) with draws in
`mutation/body.rs`, read by `creature/state.rs:122-160`,
`simulation/actions/mod.rs:209`, `predation.rs:95`, `giving.rs:74`,
`tick.rs:188`, `neighborhood/distributions.rs:61-95`, `v3-server/src/http/creature.rs:325-391`,
`v3-server/src/state.rs:402` and `v3-telemetry/src/windows.rs:756-764`; all read the genome's
body. Shared rule "mutation supply": owner `requested_event_count`
(`mutation/engine/mod.rs:27`), applied per chromosome; the instruments
(`drift.rs`, `births.rs:320`, `mutation_effects/mod.rs:459`) pass per-chromosome
counts (B6). Shared rule "a body trait's range": owner the body rules in `creature::body`, which resolve `BodyRanges` from config once; the gene operation applies it and every engine caller passes it, as `HeritableBody::inherit` (`creature/body.rs:59-80,120`) does today for its one path (B11). Shared rule "founder size anchor": `FOUNDER_GENOME_SIZE_UNITS`
(`simulation/actions/reproduction.rs:16-47`), now 85 (B7). Retained: the public
`nodes` and `entry_node_id` (F01's A11, unpaid accept): F02 adds no node write
and moves every literal to the constructor, which A11's payer would need
anyway; G2, G12, G15, G23 are untouched.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_attempted` / `_applied`, `petri.domain=Body` | counter | applied body events | `mutation.per_unit_rate` (`run.started` body) |
| the same by operator, `petri.operator=Body.*`, and `petri.run.mutation_operator_funnel` | counter | applied body events by gene | `mutation.capacity.log_step`, `mutation.share.log_step`, `mutation.phenotype.*` |
| bench `mutation_supply.operator_funnel_by_operator` `Body.*` rows | summary field | the same counters | the bench config |
| bench sample `body` block (invariant 12) | summary block | the body census of living creatures | the bench config |
| bench `capacity`, `shares`, `petri.tick.mean_capacity` | summary block, gauge | the genome's body | unchanged |
| `petri.config.mutation.capacity.log_step`, `.share.log_step` | span attribute | config (`trace.rs`) | the two `chance` keys are deleted |

## Verification Plan

**Question.** Body genes under the one per-unit law keep the founder's supply
at 0.485 events per birth and the three goal worlds persisting while every
body gene keeps varying; a viability failure, goal persistence outside the
epoch's ordinary variation, or a body gene with no applied event or a
collapsed distribution in the goal-long run shows it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal + goal-long` | A founder, default and supply change read at 2,000 ticks against the goal epoch; body genes move about 0.0057 times per birth, so only the 40,000-tick run shows their distributions, and the track names F02 its goal-long carrier. |
| Slow tests (`make rust-test-slow`) | `run` | The diff touches `crates/v3-lab` (eval's founder body, genome files). |

## Implementation Tasks

- [x] B8, B9: the colour gene and its step type in `genome/body.rs`, fields private, active channel validated.
- [x] B1, B2: `BodyGenome` (invariant 1) and `CreatureGenome::new` with the private body; every literal migrated; `genome_size()` and `brain_units()` (invariant 2).
- [x] B5: `lineage_id` into `Provenance` and the creature record; `CreatureIdentityState`, `HeritableBody` and `Zygote`'s body deleted (invariant 6).
- [x] B7 and the rate: founder bodies, `FOUNDER_GENOME_SIZE_UNITS = 85`, `0.485 / 85` (invariants 7, 8); run `cargo test -p v3-core --test viability` first.
- [x] B3, B6, B10, B11: per-chromosome supply, bounded body operators with `BodyRanges` at every engine caller, reporting, gate deleted (invariants 3, 4).
- [x] Conception's clamp (invariant 5); body readers and fixtures (invariant 10); lab (invariant 11).
- [x] B4: `chance` deleted across config, frontend, telemetry, CLI; `PROTOCOL_VERSION` (invariant 9).
- [x] Body census and bench `body` block (invariant 12).
- [x] Reference specs updated; re-pins with their cause (invariant 13).

## Verification

- [x] Viability first: `cargo test -p v3-core --test viability` -> 30 passed on macOS aarch64 after re-pinning the three founder-only digests and raising the colour-divergence fixture's rate to 0.2. GNU/Linux x86_64: the same test binary cross-built for `x86_64-unknown-linux-gnu` and run under emulated Ubuntu 24.04 (glibc 2.39) -> 30 passed with the re-measured pins at the same births (98, 94, 89); the method reproduces the three pins of the commit before T29.F02 (transcript in the readings).
- [x] Focused tests: brain operators are the catalog's leading run, carry the weights and are all `dispatch` takes, body operators outside them; founder supply 0.485 and size 85; body-supply draw pin; gene operations and the conception clamp against today's arithmetic; repeated scalar events at extreme normalized `share_min` and `log_step` stay finite and in range; an instrument's mutated genome equals conception's for the same draws; genome serde with the body and a rejected out-of-range active channel; body events recorded as applied `NotApplicable`; a zero-brain-event birth still draws body supply; the one-event screen draws no body event; size pressure leaves body events; `lineage_id` provenance; body census against brute-force pairs.
- [x] Build: `cargo check --workspace --all-targets`, `cargo fmt --all --check`, `make rust-clippy`, `node scripts/architecture-check.mjs` -> exit 0; `cargo test -p v3-core` (2,256 unit tests and every integration target), `-p v3-cli -p v3-lab`, `-p v3-telemetry -p v3-server` -> all pass; frontend `npm test` -> 680 passed, `npm run lint` -> no new finding.
- [x] `make check` -> exit 0; `make rust-test-slow` -> exit 0 (6 passed), on tested commit 3ec6b1156.
- [x] Gate, goal and goal-long run once each (`make bench PROFILE=<p> FEATURE=t29-f02-body-genes`); summaries at `docs/progress/features/t29-f02-body-genes.json`, `-goal.json`, `-goal-long.json`; outer exits 0, 2, 2 and CLI exits 0, 3, 3 (3 is the severe exit); raw hashes, byte counts and times in the readings; series `closed` entries point to them; no full report staged. Goal and goal-long are `severe` (below).

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: one genome
encodes both the body plan and the nervous system; it reaches creatures
through every birth (body genes now vary only by mutation events) and through
the body rules and carrying cost every tick.

Compute: a birth draws five more Bernoulli trials and rarely a body event; the
genome clones five small genes more; the body census runs at sample
checkpoints in `O(32·N)`. Gate `wall_clock_ms_per_creature_tick` is predeclared
within the 25 % flag threshold against the gate epoch, ±5 % as a hypothesis;
a `severe` blocks closure until remediation removes it or the user accepts it.

Trajectories move with one cause (invariant 13): the gate's deterministic
block and the goal and goal-long series differ from their epochs, and the
gate and goal epochs re-pin in the closing commit. The goal-long reading
compares against its epoch and last reading (both
`t26-f02-evolvable-energy-reserves-goal-long.json`), naming the features
closed in between; its epoch is not re-pinned.

| Indicator | Expected | Shown wrong by |
| --- | --- | --- |
| Founder supply | exactly 0.485 events per birth (test) | any other value |
| 2,000-tick persistence, three worlds | within the goal epoch's ordinary variation: the founder pays 5 more carrying units (+6.25 % of carrying, under 0.1 % of its per-tick drain) and 5.9 % fewer brain events per birth | an extinction the epoch does not show |
| Mean genome size | +5 by construction; growth otherwise without direction | — |
| Body events by gene | each gene about a fifth of body events; total about 0.0285 per founder-size birth | a gene with zero applied events in goal-long |
| Capacity and share distributions | narrower than the same horizon's earlier reading (about 18 times fewer steps per lineage): capacity at 40,000 ticks against `t26-f02-evolvable-energy-reserves-goal-long.json`, both at 2,000 ticks against `t27-f03-giving-energy-goal.json` | a wider p10–p90 band at the same horizon |
| Kin tags (goal-long end) | `kin_hamming_within_clade` well below `kin_hamming_across_clades` (about 16 bits); at about 700 generations a lineage path carries about 4 flips, so within-clade at most 8 bits as a hypothesis | within-clade near across-clade |
| Colour | `founder_colour_fraction` high and `distinct_colours` low at 2,000 ticks; no baseline exists | — |
| Neighborhood and drift readings | move with the rate (brain supply 80/85 of before at founder size); no direction claimed | — |

**Measured verdict.** Final code `3dc216baf`. Gate: `severe=false`, CLI exit 0,
deterministic block differs from the epoch as predeclared, wall -0.58 %. Goal:
`severe=true` (CLI exit 3), `decided_passes` +86 % (0.0038 to 0.0070); all three
worlds persist, with final, minimum and plateau population below the epoch's (Canyon
final 1,624 against 2,784). Goal-long: `severe=true` (CLI exit 3),
`plasticity_updates` +203 % and `decided_passes` +233 %, five more counters flagged;
all three worlds persist; every body gene has 11,500 to 29,400 applied events per
world, 0.0286 to 0.0289 body events per birth; capacity p10-p90 narrower in two worlds
and wider in Confluence; within-clade kin bits 3.2 to 4.5. Several features closed
between the epoch and this run, so the goal-long shift has more than one cause. The
severe flags are unresolved and go to the orchestrator; the gate and goal epoch
re-pins are not made.

- Summaries: [gate](../../progress/features/t29-f02-body-genes.json),
  [goal](../../progress/features/t29-f02-body-genes-goal.json),
  [goal-long](../../progress/features/t29-f02-body-genes-goal-long.json).
- Full readings: [`docs/progress/readings/t29-f02.md`](../../progress/readings/t29-f02.md).

## Success Criteria

- [x] Colour, kin tag, capacity, steal share and give share are body genes in `CreatureGenome`, one unit each, mutated at the brain's per-unit rate with no gate on brain mutation.
- [x] `lineage_id` is observation-only provenance.
- [x] The founder is 85 units, its supply is 0.485 events per birth, and the replication and gestation anchors are 85.
- [x] The declared profiles and slow tests ran and their readings are recorded; G5 is deleted from `docs/architecture.md`.

## Notes for AI Agents

- Decision: audit rows B1–B11 are in-feature refactors of this feature ([readings](../../progress/readings/t29-f02.md)).
- Decision: body traits are genes under the one per-unit law, accepting that capacity and the shares mutate about 16 to 18 times less often than 0.1 per birth (user, 2026-10-22, track Notes).
- Deferred: unpaid accept, `CreatureGenome.nodes` (`crates/v3-core/src/genome/mod.rs:247`, `WORKAROUND(T29.F01)`); payer proposed to the user at closure.
- Deferred: the engine's whole-genome clone taken before an event and diffed after it (`crates/v3-core/src/mutation/engine/mod.rs:62`, `WORKAROUND(T30.F11)`) is paid by T30.F11 (Operators Report Their Own Edits).
- Deferred: review P3, `undo_t29_f02` in `crates/v3-cli/tests/bench_artifacts.rs:216` extends the historical schema replay by restoring the retired `chance` fields and the old rate; removing the reverse-schema chain while keeping the current-digest and repeated-resolution assertions is left as advisory test cruft.
- Decision: the frontend Vite upgrade in `frontend/package-lock.json` (vite 6.4.4, GHSA-8g5g-mxqh-grgj) lands with this feature as its own `chore(deps)` commit `22bbe5b1c` (lockfile only): the pre-commit OSV hook blocks every commit while main carries 6.4.4; reported to the user at closure.
- Exception: user, 2026-10-29: accepted the goal and goal-long severes (`decided_passes`, `plasticity_updates`) and the goal persistence deviation as consequences of the one-law body-gene change; gate and goal epochs re-pinned to `t29-f02-body-genes.json` and `t29-f02-body-genes-goal.json`.
- Cost: `/usage` not recorded; implementer passes 3 (build, remediation and simplify, rebase) with advisor consults 3, 2, 1; spec-owner resumes after Plan 4; Codex challenge rounds 2, final verdict ready; review findings: lens 1 one P1 (accepted at triage), lens 2 none, lens 3 one P2 and one P3; run under the user's 2026-10-29 no-Fable override (spec owner Opus 5.5 high).
