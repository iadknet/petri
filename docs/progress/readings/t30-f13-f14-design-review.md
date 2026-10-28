# T30.F13–F14 design review

Codex plan-design-review of adding T30.F13 (censuses live in core,
summaries follow their types) and T30.F14 (the inspector reads resolved
views), with a note on T30.F11 (2026-10-27/28, user direction after a
second post-closure refactor audit of T25.F11's outer areas). Not a feature
reading.

- **Job.** Four fresh read-only `codex exec` jobs (`gpt-6.1-sol`, effort
  `high`) through `scripts/codex-review track-design`, all six checks,
  coverage and Intent per round, over the T30 track and `docs/roadmap.md`.
  One round-3 attempt failed at model capacity and was rerun unchanged.
- **Input held fixed.** "Run the fix pass (items 1-7) through the
  implementer with make check and merge, then draft the roadmap rows and
  notes and run their track-design review, committing if it clears."
- **Source.** The second refactor audit (a fresh read-only agent applying
  `$refactor-audit` to `v3-cli/src/bench/`, the server transport, the
  frontend inspector and telemetry): candidates 1–2 → F13; 3, 4, 7 → F14;
  5 → T30.F11's note; 6 is T25.F12's.

## Findings and dispositions

| Round | # | Severity | Finding | Disposition |
| --- | --- | --- | --- | --- |
| 1 | 1 | blocking | Summary keep-lists deliberately exclude fields (`rows` at the last checkpoint only, `81ab1cf7d`); "keep what typed records hold" would break the minimum-summary rule | Fixed: blocks declare a retention policy; keep-lists derive from it with the exclusions unchanged |
| 1 | 2 | advisory | Removing the lists alone still leaves sample-block plumbing in `tracking.rs` | Fixed: one composed sample record through observation and accumulation |
| 1 | 3 | advisory | "Every population census" exceeds the bundle-only move | Fixed: F13 names the other CLI calculations (assistant choice) |
| 1 | 4 | advisory | `graphFullModel.ts` does not repeat the coordinate join | Fixed: F14 names `EdgeBundleList.tsx` only |
| 2 | 5 | advisory | More CLI census calculations (`clade_diversity`, `structural_companions_census`, `bucket_surviving_clades`) | Fixed: named in F13 |
| 2 | 6 | advisory | F14's slot rule left liveness open (whole-program scan vs annotation liveness) | Fixed: slot sets follow the annotations' liveness rule for both bus references, a diagnostic correction recorded in F14's spec (assistant choice) |
| 3 | 7 | advisory | Further routes (`GestationTracking::observe`, `generation_distribution`, `structure_size_distribution`, `structure_means`, the kernel occupancy census) | Fixed: F13 states a rule and its spec inventories at Plan time; known routes named |
| 4 | 8 | advisory | The rule stopped at benchmark routes; telemetry's `Census::of` and the server's `PopulationCensus` also compute censuses | Adopted (reviewer's smallest fix, a wording change): the rule covers every outer crate, numerical definitions preserved |

Round 4 had no blocking finding, so the review clears. Every round's C2
and C5 were clear; no `step-back:` finding was raised. Every debt listed in
`docs/architecture.md` §5 has a payer.

## Intent

| Source (session a5b1a622) | Reconciliation |
| --- | --- |
| User: "having a fresh review/audit could still be useful, if anything for peace of mind" | The second audit whose candidates these rows carry |
| User: "draft the roadmap rows and notes and run their track-design review" | F13, F14 and the F11 note |
| Assistant proposal accepted: "New row: benchmark summary and bundle census" | F13 |
| Assistant proposal accepted: "New row: the inspector reads genome-resolved views" | F14 |
| Assistant choices | Row numbers and placement after F12; widening F13 to every outer-crate census; F14's liveness rule |
