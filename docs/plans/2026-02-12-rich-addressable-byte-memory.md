# Rich Addressable Byte Memory Implementation Plan

## Goal

Replace bit-based creature memory with addressable byte memory and expose address/head operations so evolved controllers can read/write richer internal state.

## Scope

- Addressable memory slots are bytes (`u8`) instead of bits (`bool`).
- Controller memory I/O moves from a single write scalar to explicit address + write value + write enable outputs.
- Tick evaluation becomes two-stage for address-first semantics.
- Fresh-world rollout: old snapshots/controllers are not guaranteed compatible.

## Boundary Impact

- Crate dependency direction remains unchanged: `petri-graph -> petri-core -> petri-server/petri-cli`.
- Public API/wire format changes are expected for memory input/output fields in Rust + TypeScript contracts.
- Test migration approach:
  - Unit-level graph node/eval tests in `crates/petri-graph/tests/*`.
  - World behavior and snapshot regression tests in `crates/petri-core/src/world/tests.rs`.
  - Web protocol contract tests in `web/src/protocol.test.ts`.

## Decisions

- Memory store type: `Vec<u8>`.
- Address encoding: one selector scalar mapped from `[-1, 1]` to `[0, len-1]`.
- Shared read/write address per tick.
- Two-stage evaluation:
  - Stage A picks address.
  - Stage B receives `memory_read` + normalized resolved address and emits write signals.
- Compute cost remains charged once per tick.
- Offspring inherit memory contents.
- Memory size remains evolvable with existing bounds logic.

## Tasks

1. **TDD: graph contracts (RED)**
   - Add failing tests for:
     - New memory node kinds (`InputMemoryAddressNorm`, `OutputMemoryAddressSelect`, `OutputMemoryWriteValue`, `OutputMemoryWriteEnable`).
     - Output clamping and deterministic address selector behavior.
   - Run targeted `petri-graph` tests and confirm failure reason is missing new fields/nodes.

2. **Implement graph contracts (GREEN)**
   - Update `petri-graph` types, evaluator, node classification, mutation helpers, and founder presets to include new memory nodes/outputs.
   - Update graph tests to pass.

3. **TDD: world memory behavior (RED)**
   - Add failing `petri-core` tests for:
     - Address mapping for selector edge values.
     - Two-stage read/write correctness.
     - Byte quantization behavior for writes.
     - Single compute-cost charge despite two evaluations.
     - Offspring byte memory content inheritance.
     - Snapshot round-trip for byte memory and new memory head detail.
   - Run targeted `petri-core` tests and confirm expected failures.

4. **Implement world behavior (GREEN)**
   - Update creature memory storage to `Vec<u8>`.
   - Add two-stage memory logic in tick loop.
   - Add memory-head debug struct to runtime state and creature detail.
   - Keep size mutation/bounds behavior.
   - Update snapshot serialization/deserialization for new types.

5. **TDD + implement web/server contract updates**
   - Update TS protocol and tests for new memory input/output/head fields.
   - Update server-facing Rust contracts if required by `CreatureDetail` changes.
   - Run targeted web tests and ensure inspector remains stable with new fields.

6. **Docs update**
   - Update `docs/reference/creature-controller-reference.md` memory node/type tables.
   - Update README only if user-visible behavior/terminology changed materially.

## Verification

Run in this order before completion:

1. `cargo fmt --all --check`
2. `cargo test --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cd web && npm run build`

During iteration use targeted runs:

- `cargo test -p petri-graph`
- `cargo test -p petri-core memory`
- `cd web && npm test -- --run protocol`
