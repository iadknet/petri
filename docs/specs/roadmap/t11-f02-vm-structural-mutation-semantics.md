# T11.F02 — VM Structural Mutation Semantics

**Status**: Blocked
**Last updated**: 2026-09-05
**Feature**: T11.F02
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

VM instruction edits preserve surviving control-flow references, operand
mutations make one small change without rewriting the opcode, and register
capacity can change without aliasing existing registers. Founders carry spare
registers on which later mutations can operate without overwriting live values.

## Non-Goals

- No labels, new controller family, graph-growth or clock repair, mutation
  probability/weight/reachability changes, or learned-state inheritance changes.
- No claim that arbitrary live insertion, replacement, deletion, or copying is
  behaviorally neutral. T11.F08 owns neutral production duplication and its
  activation; this feature repairs reference integrity for every existing VM
  splice and proves neutral insertion/copying under the conditions below.
- No benchmark threshold changes, baseline replacement, or second goal run.

## Inputs and Invariants

- Sources: the owning track's T11.F02 and node-type-contract notes,
  [T11.F01](t11-f01-mutational-neighborhood-indicator.md), the
  [brain evolvability audit](../../strategy/brain-evolvability-audit-2026-09-04.md),
  and the [VM ISA](../../reference/v3-vm-isa-spec.md) and
  [mutation](../../reference/v3-mutation-spec.md) reference specs.
- Existing seams: `mutation/vm/operators.rs`, `mutation/input_ref/mod.rs`
  (which inserts VM `ReadInput` instructions when an input reference is added),
  `runtime/vm.rs::jump_target` (also used by traced execution), the genome
  slice analyses, founder builder, and `neighborhood` battery. Extend these
  using `std` and existing dependencies; do not introduce an alternate mutation
  or execution path.
- Research checked 2026-09-05: repair positional references in place, or adopt
  label/tag addressing. [SignalGP](https://arxiv.org/abs/1804.05445) associates
  events and functions through evolvable tags; [Avida's instruction set](https://github.com/devosoft/avida/wiki/Instruction-Set)
  includes relative head jumps and template-based flow control. Those are
  credible representation alternatives, but require ISA and analysis changes
  beyond this repair. Retain the roadmap's in-place repair; T11.F11 remains the
  conditional label alternative. The local audit directly identifies the
  missing splice repair, so no additional library or architecture is needed.
- Jump policy: resolve each old jump's absolute target as
  `(old_pc + 1 + offset).rem_euclid(old_len)` using signed wide arithmetic.
  After an edit, encode `new_target - (new_pc + 1)`. This applies to wrapped,
  forward, backward, conditional, self, and extreme-i32 offsets alike. Reuse
  the runtime's target semantics instead of maintaining a second definition.
- Insertion preserves the identities of all old instructions, including the
  one at the insertion boundary. A surviving old jump follows that instruction
  past the inserted span. Replacement retains its position as the target of
  incoming jumps; the replacement's own operands are newly authored. Deletion
  redirects a jump into the deleted span to the first surviving instruction
  after that span, wrapping to the first survivor when deleting the tail.
  Existing nonempty programs are never emptied by a deletion operator.
- Copy policy: map each selected source instruction to its copied position.
  A copied jump to another selected instruction targets that copy; a copied
  jump to an unselected instruction targets the surviving original. This map
  covers contiguous blocks and noncontiguous forward/backward slices. Old
  jumps keep targeting originals, even when the insertion splits the source
  span. Both block-copy variants obey this rule; the remapped variant retains
  its explicit register-renaming behavior, independently of jump repair.
  Newly authored motif-internal offsets are interpreted in the new program.
- Every VM splice uses provenance-aware old instruction indices, never
  instruction equality: copied items retain their source index while newly
  authored instructions have none. This includes automatic `ReadInput`
  insertion during input-reference addition. Repairing that existing operator
  changes control flow around the inserted span, including old jumps that now
  skip it while following their original target; its input-reference
  growth/wiring policy remains owned by its existing feature area.
- Operand steps: `VmInstructionRawFieldMutation` keeps its operator identity
  and weight but selects one operand-bearing instruction and changes exactly
  one encoded field by a one-unit step (inward at a numeric boundary; no large
  wraparound). Jump offsets use the same bounded plus/minus-one step, with no
  overflow. This is field mutation, not opcode replacement. `Noop`, `Halt`,
  `ExecuteActionQueue`, and `PopAction` have no operands and are ineligible;
  an empty/no-operand program reports `NoApplicableTarget`. Other operands and
  the opcode are unchanged. The standalone `VmMutateSlotAddress` also moves
  its one address field by one unit. The existing paired-slot operation is
  explicitly a linked-address macro, not a single-field event; its coordinated
  changes and the macro instruction-replacement operator stay available.
- Register-capacity policy: grow by one up to 32; shrink by one only if the
  highest effective register is unused by every register-typed operand.
  Resolve/canonicalize existing raw register operands under the old runtime
  width before changing the count, preserving each effective register's
  identity; otherwise skip. Never fold a referenced removed register onto 0.
  Widths outside 1 through 32 skip without altering the genome, preserving
  the runtime's immediate halt at zero and its existing larger-width behavior.
  Add four spare registers to the 16-register founder (20 total), retaining
  every instruction, constant, action, and existing register reference.
- Neutrality means identical action/output/shared-memory behavior when both
  executions have enough energy and steps. A Noop inserted anywhere into an
  in-range-target program preserves that behavior. A block copied to a proven
  unreachable suffix does likewise; a terminal alone does not prove a suffix
  unreachable if a jump reaches it. Executed Noops still consume their defined
  energy and steps. Do not hide those costs or claim neutrality at exhaustion.
- Record the four-part node-type contract in `v3-mutation-spec.md`, with links
  from the VM and graph backend specs: stable/remapped references, neutral
  growth, one world-tick persistent-state clock, and small mutation steps.
  Name the remaining T11 owners rather than claiming these are already all
  satisfied. Update owning reference documents for every changed semantic rule.

## Implementation Tasks

- [x] Add failing example/property tests for splice references, copied-target
      mapping, one-field steps, terminal stability, register identity, and
      neutral Noop/unreachable-copy behavior before implementation.
- [x] Route every VM insertion/deletion/replacement, motif, block, and slice
      copy through the shared reference-remapping rule; preserve deterministic
      seeded behavior and the production/traced execution agreement.
- [x] Implement one-field operand steps and safe register-capacity changes;
      add founder register slack and verify its original behavior is preserved.
- [x] Update the mutation/VM references and graph contract pointer; update any
      founder contract documenting its register count.
- [x] Self-review the diff for reuse, simplification, and efficiency; mutation
      evidence is recorded below.
- [ ] Store gate and single-run goal reports at
      `docs/progress/features/t11-f02-vm-structural-mutation-semantics.json`
      and `...-goal.json`; append both series entries and update
      `docs/progress.md` with measured comparisons and every indicator reading.

## Verification

- [x] TDD evidence: record initial failing commands and the regression they
      exposed. Property tests cover old-to-new jump targets for all edit kinds,
      copied internal/external targets, modulo-wrapped references, one-field
      bounded changes for all operand-bearing opcodes, and register identity.
      Commit any generated `proptest-regressions/` files.
- [x] Behavioral property tests cover Noop insertion and unreachable block
      copying with enough energy/steps; explicit fixtures distinguish energy
      exhaustion and step caps, deleted targets, terminal replacement versus
      field mutation, and noncontiguous slice copies.
- [x] `cargo test -p v3-core --test viability` ran first for the founder
      change; `cargo check --workspace --all-targets` follows coherent Rust
      edits. Focused VM/founder/neighborhood tests and cross-process
      reproducibility pass; exact evidence is recorded below.
- [ ] `make rust-mutants` after self-review: record summary, output directory,
      and full missed/timeout list, each killed by a strengthened test and rerun,
      equivalent with reason, or deferred in Notes. Do not alter production
      code to kill mutants or add unjustified exclusions.
- [ ] `make bench PROFILE=gate FEATURE=t11-f02-vm-structural-mutation-semantics`
      and one `PROFILE=goal` run pass and their reports are stored. Measurement
      uses the existing host-contention preflight and unchanged profile sizes.
- [x] Second goal-profile determinism run: Not applicable per the 2026-09-05
      workflow decision; cross-process reproducibility remains in `make check`.
- [ ] `make roadmap-check` on document edits, independently by the orchestrator
      before accepting implementation; final `make check` exits 0 for the
      content committed and integrated into `main`.

## Performance and Goal Impact

Natural analog: gene insertion, deletion, and duplication. Reproduction applies
small changes to inherited VM programs; stable references allow descendants to
retain behavior while acquiring structure.

Predeclared cost: remapping scans the edited program at mutation time, and four
spare founder registers enlarge dispatch initialization slightly; no per-opcode
cost or work-counter definition changes. No severe compute regression is
budgeted, and no baseline re-pin is authorized. Gate simulation counters should
stay close to T11.F01 and the T10.F10 epoch, but changed offspring behavior can
change them. Record all six normalized work and wall-clock deltas against both
references and investigate any threshold crossing without weakening a gate.

Neighborhood expectations before implementation: founder VM field-mutation and
register-capacity silence should increase; repaired insertion/motif silence
should generally increase and dead fractions should decrease. Reference repair
does not promise neutrality for live macro copies/replacements: their silent
fractions may fall when correctly retained control flow makes a formerly inert
edit execute. Such movement is predeclared only for the affected VM structural
operators and must be attributed to their corrected targets. Founder slack
changes which registers VM operand/motif/copy mutations sample, so those rows
may move in either direction from register interference; report that separately.
Founder non-VM operator rows should remain unchanged because founder execution
is preserved and their mutation rules are untouched, except input-reference
addition on VM nodes, whose auto-wired insertion now has repaired jump
targeting. Mutated-birth silence is expected to rise and dead fractions fall;
report every bucket and investigate any reversal, mindful of T11.F01's
four-sample single-event bucket.

The evolved sample, lineage diversity, persistence, and memory sensitivity may
shift in either direction because the inherited mutation map changes the
population, not merely the measurement. Record the actual shifts against
T11.F01 and T01.F12, distinguishing changed subjects from operator regressions;
do not assert a cognition gain. All T11.F01 floors remain fixed and are due by
T11.F10. No operator family is disabled/down-weighted to improve a reading.
Closure readings and comparison conclusions remain to be recorded here.
Record that input-reference-addition effect separately from its unchanged
input-reference policy.

## Success Criteria

- [ ] Every existing VM splice preserves surviving references under the
      stated target policy, including copied internal/external references.
- [ ] Operand events change one field by one unit, keep terminals unchanged,
      and register-count events preserve effective register identities.
- [ ] Founder slack preserves unmutated behavior; neutral edit properties,
      applied-cost fixtures, viability, and reproducibility pass.
- [ ] Reference contracts, mutation triage, benchmark/progress evidence, and
      final review are complete; the checked feature and Complete spec land on
      clean `main` at the tested commit, with worktree and branch removed.

## Notes for AI Agents

- Start: clean main `c1e3b406ddd82b6bc42b431411011326d634af5e`;
  worktree `/Users/istefanek/projects/petri/.worktrees/t11-f02`, branch
  `codex/t11-f02`. T11.F01 is checked; no other session owns T11.F02.
- The user explicitly selected Astra `xhigh` for this task after the initial
  launch check. Terra, Sol, and the fresh Astra reviewer use `high`.
- Planning readiness review (Astra, 2026-09-05): Ready after one revision;
  clarified the standalone slot-address step and out-of-range register-count
  policy. Requirements cover each roadmap obligation without extending into
  neutral production duplication or the remaining T11 repairs. Runtime behavior
  and performance remain implementation verification, not planning claims.
- Closure cost and review records pending.
- Sol consultation 1 (2026-09-05, accepted): route every VM splice, including
  input-reference auto-wiring, through a provenance-aware old-index map using
  runtime jump-target semantics; use exhaustive register-field canonicalization
  and remove positional jump-offset adjustment. Consultation 2 (accepted after
  the register-count smoke-test failure recurred): use a width-2, r0-only
  fixture to demonstrate permitted grow/shrink, retain the founder as a
  required shrink skip, and cover width-4 raw-register canonicalization and
  atomic skips separately. No guidance was rejected.
- TDD and coverage evidence (2026-09-05): initial
  `cargo test -p v3-core mutation::vm::tests` recorded 51 passing and five
  intentional red tests (terminal/no-operand raw-field skips, one-field step,
  out-of-range width atomicity, and insertion target identity). Founder slack first recorded
  16-versus-required-20 in `/private/tmp/t11-f02-founder-slack-red.log`, then
  `cargo test -p v3-core --test viability` passed 25/25 before the founder
  change's subsequent all-target compile. Final focused library coverage
  passed 1,087/1,087 in `/private/tmp/t11-f02-v3-core-lib-full-pass-2.log`;
  it includes property coverage for all splice kinds and wrapped offsets,
  all operand opcode fields, every register-typed read/write field, actual
  noncontiguous conditional forward slices, Noop insertion at every boundary,
  and unreachable copied suffixes. The Noop fixtures separately show applied
  step-cap and energy-exhaustion differences. `cargo check --workspace
  --all-targets` passed in `/private/tmp/t11-f02-cargo-check-final-coverage.log`.
  The generated `proptest-regressions/runtime/tests/vm_execution.txt` seed is
  retained for the prior zero-cost-default counterexample.
- Release-only regression evidence (2026-09-05):
  `cargo test -p v3-core --release
  mutation::vm::tests::raw_field_mutation_changes_exactly_one_encoded_field
  -- --exact` initially failed because `debug_assert!` compiled out the actual
  field mutation; `/private/tmp/t11-f02-raw-field-release-red.log` records the
  0-versus-1 changed-field assertion. The unconditional mutation path then
  passed the same exact release test in
  `/private/tmp/t11-f02-raw-field-release-pass.log`. The full release VM
  filter then passed 70/70 in `/private/tmp/t11-f02-vm-release-final.log`.
- Self-review (2026-09-05): inspected every production VM program structural
  edit with `rg`, leaving no direct `insert`, `remove`, or `splice` outside
  tests; routed the empty-program mutation through the same splice seam as all
  other insertions. Reviewed provenance handling, signed offset encoding,
  register mapping, input-reference insertion, and all changed reference text.
  Reused the exhaustive register mapper for canonicalization and remapped
  copies, removed the obsolete raw-field redraw path, and found/fixed the
  release-only `debug_assert!` mutation omission above. Focused integration
  (`viability` 25/25, neighborhood 4/4, reproducibility 1/1, VM E2E 1/1)
  passed in `/private/tmp/t11-f02-focused-integration-final.log`; no remaining
  simplification or efficiency finding requires a production change. A second
  review after survivor remediation found one composition bug introduced by the
  expanded encoded-slot domain: paired slot-address mutation added `u8` values
  before reducing modulo 16. The maximum-slot red regression in
  `/private/tmp/t11-f02-paired-slot-raw255-red.log` panicked in debug. The
  production repair widens that existing arithmetic to `u16` before reduction;
  it preserves the macro's modulo-16 policy. A proptest now covers every raw
  slot and seed, requiring co-addressed immediate load/store operands below 16
  and a changed effective address.
- Remediation verification (2026-09-05): after the paired-slot repair and
  survivor-driven test strengthening, `cargo check --workspace --all-targets`
  passed (`/private/tmp/t11-f02-cargo-check-remediation-final.log`) and
  `cargo test -p v3-core --lib` passed 1,095/1,095
  (`/private/tmp/t11-f02-v3-core-lib-remediation-final.log`). The exact
  remapped-copy regression passed 1/1 in
  `/private/tmp/t11-f02-remapped-all-fields-green-exact.log`; the VM module
  passed 78/78 in `/private/tmp/t11-f02-vm-module-remediation.log`; and its
  release filter passed 78/78 with the expected zero-test integration binaries
  in `/private/tmp/t11-f02-vm-release-remediation-final.log`. The two earlier
  attempted multi-filter cargo invocations are retained as command-usage
  failures in `/private/tmp/t11-f02-survivor-coverage-red.log`; they executed
  no tests and are not validation evidence.
- Sol consultations (2026-09-05): consultation 3, accepted, required existing
  bounded seeded calibration rather than a custom RNG for register direction
  and numeric-boundary tests. Consultation 4, accepted after a remapped-copy
  red failure, replaced whole-instruction equality with register-field identity
  because a surviving conditional jump's offset may be reencoded; it also
  required one Cargo positional filter per command. Six consultations total;
  no guidance rejected.
- Original mutation run (2026-09-05): `make rust-mutants` generated
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f02/mutants.out` from
  the captured feature diff. Its terminal `cargo-mutants` summary was 122
  tested, 96 caught, 24 missed, 2 unviable, and 0 timeouts. The retained
  process chain was make 53683, script 53685, cargo-mutants 53705. The outer
  execution wrapper did not retain a terminal session id, so its direct make
  exit status is not asserted; the terminal summary is preserved in
  `/private/tmp/t11-f02-rust-mutants.log`. Missed locations: register-count
  negation (operators.rs:55); u8 boundary arms/guards (252-253); u16 boundary
  arms/guards (260-262); i32 boundary arms/guards (269-271); instruction
  mutation branch ranges (440,456); register remapping body/operators (532,
  534-536); splice position/deletion mapping (634,648,678); read/store and
  load/compare motif adjacency (883,972); and indirect slot-field matching
  (1078). Exact first-run survivors are preserved in
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f02/mutants.out/missed.txt`.
  The second target `/Users/istefanek/.local/share/petri-tools/mutants/t11-f02-remediation/mutants.out`
  completed 129 tested: 123 caught, 4 missed, 2 unviable, 0 timeout. Its
  `nudge_u16` zero arm and splice `||` guard are killed by the final test-only
  batch. The two `new_len > i32::MAX` replacements (`==`, `>=`) are deferred:
  distinguishing them requires allocating and cloning vectors at least about
  2.1 billion elements; they are not equivalent. A third final target rerun is
  verified by the final target. The two unviable mutants were founder default and input-reference
  `Ok(Default)` changes. Survivor-driven test strengthening and the separate
  paired-slot overflow repair are ready for the required distinct-output rerun;
  no production edit was made merely to kill a mutant.
- Integration blocker (2026-09-05): while this feature was at implementation
  commit `e63248591faa7eb51f79014068a6920cc305ecb1`, `main` at
  `/Users/istefanek/projects/petri` advanced from the recorded base
  `c1e3b406ddd82b6bc42b431411011326d634af5e` to
  `eddfacae43cc5e32a3748b9e2a589e14acba1172` (`test: fix rust-mutants guard
  case against a shallow checkout`). The main change is limited to
  `scripts/bench-wait-test`, which does not overlap this feature's paths, but
  workflow requires explicit reconciliation authority before rebasing or
  merging. No reconciliation has been performed. Completed validation is the
  debug/release VM, viability, neighborhood, reproducibility, VM E2E, docs,
  and gate evidence above; the gate report is present at
  `docs/progress/features/t11-f02-vm-structural-mutation-semantics.json` but
  has not been appended to the series because closure-series append remains held for reconciliation authorization.
  The original mutation run has since reached the terminal summary recorded
  above. Outstanding feature-branch work is the required distinct-output
  survivor rerun, the replacement gate and one goal benchmark run, and progress
  evidence. Reconciliation to the approved main revision, final review, and
  integration remain blocked on explicit authority; no rebase or merge has been
  performed.

- Final mutation evidence (2026-09-05): original wrapper exit was not retained: original 122 tested/96 caught/24 missed/2 unviable/0 timeout; remediation 129/123/4/2/0; final 129/125/2/2/0. Outputs: `t11-f02`, `t11-f02-remediation`, and `t11-f02-final` under `/Users/istefanek/.local/share/petri-tools/mutants/`.
  Original survivors and final dispositions:
  - `crates/v3-core/src/mutation/vm/operators.rs:55:42: delete ! in apply_register_count_mutation` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:252:9: delete match arm u8::MAX in nudge_u8` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:253:14: replace match guard rng.gen_bool(0.5) with true in nudge_u8` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:253:14: replace match guard rng.gen_bool(0.5) with false in nudge_u8` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:260:9: delete match arm 0 in nudge_u16` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:262:14: replace match guard rng.gen_bool(0.5) with true in nudge_u16` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:262:14: replace match guard rng.gen_bool(0.5) with false in nudge_u16` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:269:9: delete match arm i32::MIN in nudge_i32` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:271:14: replace match guard rng.gen_bool(0.5) with true in nudge_i32` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:271:14: replace match guard rng.gen_bool(0.5) with false in nudge_i32` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:440:30: replace + with * in apply_instruction_mutation` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:456:84: replace + with * in apply_instruction_mutation` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:532:5: replace remap_register_refs with ()` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:534:30: replace % with / in remap_register_refs` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:534:30: replace % with + in remap_register_refs` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:535:44: replace + with * in remap_register_refs` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:536:37: replace % with / in remap_register_refs` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:634:38: replace || with && in splice_program_with_reference_repair` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with == in splice_program_with_reference_repair` — deferred: distinguishing requires vectors of about 2.1 billion elements.
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with >= in splice_program_with_reference_repair` — deferred: distinguishing requires vectors of about 2.1 billion elements.
  - `crates/v3-core/src/mutation/vm/operators.rs:678:43: replace && with || in splice_program_with_reference_repair` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:883:21: replace + with * in apply_insert_read_store_motif` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:972:21: replace + with * in apply_insert_load_compare_motif` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:1078:9: delete match arm VmInstruction::LoadSlot{slot_reg, ..} | VmInstruction::StoreSlot{slot_reg, ..} in mutate_slot_idx_field` — caught in final rerun.
  Second-run survivors:
  - `crates/v3-core/src/mutation/vm/operators.rs:260:9: delete match arm 0 in nudge_u16` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:634:38: replace || with && in splice_program_with_reference_repair` — caught in final rerun.
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with == in splice_program_with_reference_repair` — deferred: requires vectors of about 2.1 billion elements to distinguish this huge-allocation guard.
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with >= in splice_program_with_reference_repair` — deferred: requires vectors of about 2.1 billion elements to distinguish this huge-allocation guard.
  Final misses:
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with == in splice_program_with_reference_repair` — deferred: requires vectors of about 2.1 billion elements to distinguish this huge-allocation guard.
  - `crates/v3-core/src/mutation/vm/operators.rs:648:16: replace > with >= in splice_program_with_reference_repair` — deferred: requires vectors of about 2.1 billion elements to distinguish this huge-allocation guard.
  Consultations 1–6 were accepted; none rejected. Only reconciliation/integration remains approval-blocked.

- Final benchmarks: replacement gate and the single goal run both exited 0. Goal report has no severe regression; plasticity updates are +23.452613% (flagged) while VM steps are -51.407250%. This follows altered mutation application/neighborhood behavior, not a work-counter or per-opcode runtime change; source review found only mutation-time splice repair and paired-slot arithmetic changes.
