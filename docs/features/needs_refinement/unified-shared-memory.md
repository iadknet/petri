---
title: Unified Shared Memory and VM Memory Evolvability
tags: [core, simulation, genome, frontend]
size: XL
depends-on: []
status: needs-review
---

## Problem Statement

The simulation has two separate memory systems that serve overlapping purposes but with very different evolvability characteristics:

1. **VM creature memory** (`[u8; 1024]`): A 1024-byte array of `u8` values on `CreatureState`, accessed via `LoadMem8`/`StoreMem8` opcodes. Inherited on reproduction. The `u8` representation forces lossy `f32 -> u8 -> f32` quantization, and the 1024-slot address space is far too large for random mutation to discover useful addressing patterns.

2. **Graph runtime state** (`GraphRuntimeState.node_state`): Per-node `f32` state used by stateful operators (DecayIntegrator, Momentum, Oscillator, AdaptiveGain). NOT inherited on reproduction. Indexed by graph topology, so meaning shifts when nodes are inserted/deleted/reordered.

This split creates several problems:
- **Asymmetric evolvability**: Graph backends get temporal behavior "for free" through stateful operators, while VM backends must evolve complex multi-instruction memory circuits through a hostile address space. This makes graph inherently advantaged for any task requiring memory.
- **No shared surface**: VM and graph nodes within the same creature mesh cannot communicate through a common memory bank. Cross-backend memory sharing is impossible.
- **Poor discoverability**: With 1024 `u8` slots, the probability that random mutation produces a useful load/store pair addressing the same slot is ~1/1024. Even when it does, the `u8` quantization loses precision.
- **Weak observability**: "Does this creature use memory?" means different things for VM (byte array access) vs graph (stateful operator state), making behavioral analysis inconsistent.

## User Stories / Acceptance Criteria

### Core memory surface
- As a simulation operator, I want a single shared memory bank of `f32` slots (target: 16 slots) on each creature, so that both VM and graph backends read/write the same values.
- As a simulation operator, I want shared memory to be inherited on reproduction, so offspring can build on learned state.
- As a simulation operator, I want the old `[u8; 1024]` memory to be completely removed and replaced by the new `[f32; N]` shared slots, with no backward-compatibility shim.

### VM access
- As a VM genome, I can read a shared slot into a register (`LoadSlot { dst, slot }`) and write a register to a shared slot (`StoreSlot { slot, src }`), both using native `f32` values with no quantization.
- As a VM genome, I can use both immediate-addressed and register-addressed variants for slot access.

### Graph access
- As a graph node, I can read committed shared slot values during evaluation via new `ReadSlot(slot_idx)` graph node kind(s).
- As a graph node, I can stage writes to shared slots via new `WriteSlot(slot_idx)` graph node kind(s), with writes committed after the relaxation loop converges (not during iteration), so recurrence and memory writes don't interfere.

### Temporal primitives
- As a creature genome, I have access to temporal memory primitives that make memory useful without requiring complex multi-instruction circuits:
  - **One-tick delay**: A slot or operator that outputs the previous tick's value of a signal.
  - **Latch**: A slot that holds its value until explicitly cleared or overwritten, useful for "remember this happened" patterns.
  - **Clear/reset**: An operation that zeros a slot or set of slots.
  - **Decay**: Optional per-slot exponential decay toward zero, configurable via a decay rate, so unused memory naturally fades rather than persisting forever.

### Mutation motifs
- As an evolving population, mutation can insert structured memory-using motifs rather than relying only on random independent opcode/node sampling:
  - **Read-store motif**: Insert a paired `ReadInput -> StoreSlot` sequence that captures a sensor value into a slot.
  - **Load-compare motif**: Insert a paired `LoadSlot -> CmpGt/CmpLt` sequence that uses a stored value in a decision.
  - **Paired address mutation**: When mutating a slot address, optionally co-mutate a paired load/store that references the same slot.
  - **Memory slice duplication**: Duplicate a small memory-using code slice with remapped slot addresses.
- As a simulation operator, I can observe via mutation telemetry whether memory-capable motifs are entering the population.

### Reproduction semantics
- Shared creature memory (`[f32; N]`) is inherited (copied to child).
- Graph-local runtime state (`node_state`, `eligibility_traces`) continues to NOT be inherited (reset on reproduction).
- Plasticity weights continue to follow existing Lamarckian/Darwinian inheritance flags.

### Observability
- As a simulation operator, I can see in the creature inspector which shared slots are non-zero, recently read, or recently written.
- As a simulation operator, I can filter/search for "creatures that use memory" with a single consistent definition across both backends.

## Out of Scope

- **Multiple memory classes** (short-term vs long-term): The brainstorm notes this should only be explored if a single shared bank proves insufficient. Start with one bank.
- **Inter-creature memory sharing / communication**: Separate feature (Communication idea in brainstorms).
- **Graph `node_state` as shared memory**: Explicitly excluded per brainstorm — `node_state` is indexed by graph topology and would change meaning on structural mutations.
- **Repurposing plasticity weights as memory**: Plasticity is a learning mechanism, not a general-purpose memory surface.
- **Memory protection / access control**: All slots are freely readable/writable by any node in the mesh.
- **Frontend memory visualization beyond basic inspector**: A dedicated memory debugger/timeline is a follow-on.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Exact slot count: 8, 16, or 32? 16 is the current best guess — small enough for easy addressing, large enough for non-trivial state. | Leaning 16 | - | open |
| Should temporal primitives (delay, latch, decay) be shared-memory-level behaviors (per-slot config) or separate opcode/node-kind mechanisms that happen to use shared slots? | - | - | open |
| Should decay rates be per-slot (genome-configured) or global (simulation config)? Per-slot is more expressive but increases genome complexity. | - | - | open |
| What energy cost model for shared memory ops? Current `LoadMem8` costs 0.16, `StoreMem8` costs 0.18. New f32 ops should be similar or cheaper to encourage use. | - | - | open |
| Should graph `WriteSlot` commits happen after each graph node's relaxation loop, or after the entire mesh evaluation? After each node's loop is simpler and matches the current graph commit model. | - | - | open |
| Wire format: How are shared slots serialized to the API? Likely a simple `shared_memory: [f32; N]` field on the creature detail endpoint. | - | - | open |
| Should mutation motifs be weighted by reachability if the reachability-aware mutation feature lands first? | - | - | open |
