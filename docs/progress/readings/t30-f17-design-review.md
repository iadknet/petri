# T30.F17 design review

Codex plan-design-review of adding T30.F17 (the genome owns its nodes) to
T30, debt G28 to `docs/architecture.md`, and T30.F17 to the master priority
order and to T27.F02's and T28.F01's dependencies (2026-10-29, user
direction at T29.F02's closure). Not a feature reading.

- **Job.** One fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, all six checks,
  coverage and Intent, over `docs/roadmaps/t30-core-boundaries.md` and
  `docs/roadmap.md`, with an intent export of the T29.F02 session.
- **Source.** T29.F02's `Deferred: unpaid accept` for `CreatureGenome.nodes`
  (`WORKAROUND(T29.F01)`, T29.F01 audit row A11); user, 2026-10-29: "We
  should do these too" on the item "A future row should make it private".

## Coverage

| Check | Verdict | What was examined | Findings or why clear |
| --- | --- | --- | --- |
| C1 Code decomposition | clear | `CreatureGenome`, `NodeGenome`, genome mesh, node and learning edits, mutation callers, reader probes, lab writers | Node storage and edits go to the existing owner; no new responsibility |
| C2 Task decomposition | finding | Dependencies, master order, concurrent-start rule, F05/F11 overlap, unchanged-pin verification | F1 |
| C3 Domain boundaries | finding | `docs/architecture.md` §§1–5, heredity note ownership | F2 |
| C4 Premises | finding | User request, A11, constructor migration, positional inheritance, debt-first ordering | F1; constructor and `eval.rs` qualifications |
| C5 Larger design flaw | clear | `11d39f426`, `082ff0c67`, `4746b32c8`, `c12a547c5`, `52fe27e78` | F17 completes the existing ownership refactor |
| C6 Workarounds | clear | Public mesh storage, reader rewiring, lab reconstruction and ablation, G23 edits, F11 snapshots | Other mechanisms have distinct purposes or existing payers (F05, F11) |

## Findings and dispositions

| ID | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| F1 | P1 (C2, C4) | Debt-first ordering is not enforced: T27.F02 and T28.F01 do not depend on F17, so either can start while F17 is open, against the 2026-10-27 direction "so all architecture debt is paid first" | User, 2026-10-29: "Yes, add dependency". T27.F02 and T28.F01 depend on T30.F17; T28.F02 follows through T28.F01; master note updated |
| F2 | P2 (C3) | Architecture §5 omits the debt F17 pays | Adopted: G28 added with payer T30.F17; T30's debt-per-row note names it |
| Q1 | qualification (C4) | "Every genome is built through `CreatureGenome::new`" misses deserialization (`GenomeFile::load`) | Corrected in the F17 note |
| Q2 | qualification (C4) | The cited `eval.rs` plasticity writer is test-only; core census and input-use probes also write nodes | Corrected in the F17 note; the writer list is marked non-exhaustive |

## Intent

| Source quote | Reconciliation |
| --- | --- |
| User: "We should do these too" | Authorizes removing the public-node workaround; F17 owns it |
| Accepted item: "A future row should make it private" | F17's goal and note require private storage, read accessors and genome-owned edits |
| A11: "The mapping's node indices hold only while node count and order change through genome operations" | Verified by the reviewer against `LearnableId`, `InheritanceMapping`, engine composition and birth finalization |
| Planner: T30 placement, behavior-preserving, `Depends on: None`, after T30.F16 | Planner choices, not user decisions; the reviewer raised no finding against them |
| User: "Yes, add dependency" | Settles F1 |
