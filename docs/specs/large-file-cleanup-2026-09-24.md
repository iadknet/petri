# Large File Cleanup (2026-09-24)

**Status**: Complete
**Last updated**: 2026-09-25
**Scope**: Maintenance; no roadmap feature ID or benchmark run. Phase A is an
ordinary commit to `main`. Phase B rewrites `main` history and needs its own
explicit authorization. Measured at `a9f59893` (census counts are dated
observations; Phase A enumerates by artifact kind at its own starting revision).

## Goal

Committed benchmark summaries keep only what reporting reads. Reporting means
the progress page and the closure comparison. Full payloads live in the
ignored `.bench-artifacts/` tree, and no tracked file exceeds 1 MiB. After
Phase A, the superseded large blobs are removed from `main` history, and the
rewritten `main` replaces `origin/main` through an exact-lease force push, as
[T15.F02](roadmap/t15-f02-historical-report-migration-and-main-history-rewrite.md)
did.

## Non-Goals

- New measurements, reruns, threshold changes or epoch re-pins. The paths in
  `docs/progress/benchmark-series.json` do not change.
- Changing the recruitment S0 format or its writer
  (`crates/v3-cli/src/recruitment.rs`).
- Images, source files, and the T15.F02 census
  `docs/progress/historical-benchmark-manifest.json`.
- Converting historical summary versions. Old commits lose their large
  summary blobs (Decision 2).
- Rewriting any ref other than `refs/heads/main`, changing branch protection,
  or erasing data from other clones, forks or GitHub caches.

## Evidence (base `a9f59893`)

| Measure | Value |
| --- | --- |
| HEAD tracked | 1,380 files, 246.6 MB |
| Goal-profile summaries | 54 non-historical files; 36 files over 1 MiB (193.4 MB). The largest is 7.8 MB pretty-printed, 3.25 MB compact |
| `main` ancestry blobs | 377.6 MB raw, **19.9 MB packed**; GitHub `diskUsage` 20,313 KB |
| Local `.git` | 1.3 GB, almost all in one 1.33 GB pack from 2026-09-13. All refs, worktrees and reflogs together reach only 23.3 MB packed; the rest is unreachable objects that were never pruned |

Phase A takes the checkout from about 247 MB to about 25 MB. Phase B saves
only about 10 MB on the remote. B5's prune shrinks the local `.git`, not the
force push.

**What reporting reads.**

- **Progress page.** A tracer wrapped each summary in a JSON `Proxy` and
  logged every path the page read. It covered the whole page and every table
  toggle, served from `docs/progress/`. The page read 370 paths, all under:
  - `kind`, `summary_version`, `feature`, `raw.verified_at` and
    `omitted_details`
  - `environment.{generated_at, git_revision, host.cpu_model,
    wall_clock_ms_per_creature_tick, wall_clock_ms_per_seed}`
  - `comparison.references[]`
  - `deterministic.{per_seed, per_creature_tick}`
  - `goal_indicators.{lineage_diversity, memory_sensitivity,
    temporal_memory_sensitivity, structural_companions,
    mutational_neighborhood}.per_seed` scalars
  - `population_persistence.per_seed[]`: scalars and `samples[]`, including
    occupancy grids and the sensor census
  - `cases[]` scalar blocks: case identity, cognition, energy flows,
    mortality, predation, moves, mutation supply and outcome, clade profiles,
    and typed eats
  - `drift_depth.readings[].{depth, changed_per_all_births,
    births.{births_total, any_events.changed}}`
  - `neighborhood_read`: its rates, its birth counts, and only the count of
    `genomes`
  - `mutational_neighborhood.founder.births`, `evolved.per_seed[].{pooled_births,
    mesh_summary}` (report and case level), and the case
    `reachable_structure_size_distribution` quantiles

  This list summarizes the trace. The trace output itself is the
  authoritative page read set.

  The page also rejects any `summary_version` other than 1
  (`index.html:301`).
- **Comparison.** The summary loader validates `deterministic.profile`
  (`artifacts.rs:555`). `crates/v3-cli/src/bench/comparison.rs` loads
  `comparison_inputs`, the normalized counters and `per_seed`. `case_readings`
  reads persistence scalars, neighborhood and memory rates, move and fraction
  tracking, and drift `changed_per_all_births` at depth 2,000.
- **Nothing reads** `recruitment_paths`, the neighborhood genome rows,
  `mutation_value_totals_by_operator`, or the drift `opportunities` and
  `recruitment` detail. Together these are about 90% of a goal summary.

Projecting every series summary onto the page and comparison read set plus
the provenance header gives 100 files and 6.75 MB total, down from 74.6 MB
compact. The largest is 246,654 bytes. Replacing `neighborhood_read.genomes`
with its count removes about 12 KB more per case.

**Other large tracked files** move to the ignored tree. Their notes and
readings already carry the numbers they cite:

| File | Bytes |
| --- | --- |
| `docs/strategy/ring-adaptation-experiment-2026-09-23.results.json` | 7,758,320 |
| `docs/strategy/ring-learning-experiment-2026-09-23.results.json` | 2,470,094 |
| `docs/strategy/population-decline-mechanisms-research-2026-09-19.results.json` | 2,418,174 |
| `docs/progress/features/t13-f07-current-policy-recruitment-transitions-s0.json` | 2,405,314 |
| `docs/strategy/drift-silence-followup-2026-09-24.replay.tar.gz` | 1,786,584 |
| `docs/strategy/ring-group-experiments-2026-09-23.results.json` | 1,182,471 |
| `docs/strategy/ring-group-experiments-2026-09-23.observations.json.gz` | 1,178,226 |

`incremental-recruitment-research-2026-09-19.evidence.json` names one of these
files in its historical `status_at_audit` text, which is not a link.

**Refs** (`git for-each-ref` at the base):

- Local branches: `main`, 4 `claude/*`, 4 `codex/*`, 2 `scratch/*`, 3
  `worktree-*`
- 21 `refs/codex/turn-diffs/*` refs and `refs/stash`
- Advertised on origin: `main` and `codex/streamline-lean-delivery-workflow`

## Decisions

1. **Keep-list, not cut-list.** A v2 summary contains only:
   - the provenance header: `kind`, `summary_version`,
     `source_schema_version`, `feature`, `raw`, `conversion`, `claims` and
     `measurement_evidence`
   - `environment`, `comparison` and `comparison_inputs`
   - the fields the page or comparison reads
   - `omitted_details`, which lists what was dropped

   This amends T15: summaries no longer carry experiment proposal totals,
   outcomes, denominators, or batch and lineage uncertainty. That detail stays
   in the raw reports and the closure readings. A6 rewrites the affected T15
   success criterion itself and the `docs/benchmark-artifacts.md` contract,
   not just a note.
2. **Strip, don't convert, history.** Phase B removes historical blobs over
   1 MiB at `docs/progress/features/` and `docs/strategy/` paths. Smaller
   historical summaries stay. The rewritten tip tree is identical to the
   pre-rewrite tip tree.
3. **Ceiling: 1 MiB per tracked file.** The allowlist is
   `docs/assets/petri-creatures.gif` and
   `docs/progress/historical-benchmark-manifest.json`. A new reporting need is
   met by adding its field to the keep-list, never by an allowlist entry.
4. **Ad-hoc, not T15.F03.** Phase A changes production Rust in `v3-cli`, so
   TDD, `$rust-skills`, `make check`, and one scoped `make rust-mutants` over
   the changed files apply. Survivors get test-only remediation.

## Phase A — Condense and Relocate (ordinary commit)

Work in `.worktrees/large-file-cleanup` from `main`. Run `npm ci` in
`frontend/` first. Check `df` before copying anything into `.bench-artifacts/`.

- [x] **A1. Keep-list.** Put the v2 keep-list in `artifacts.rs` as one
  explicit field projection over a v1 summary, and set `SUMMARY_VERSION` to 2.
  - Its source is the union of four sets: the full page trace (re-run and
    saved with the change), every field the summary loader validates, every
    field comparison reads (derived from the types that `comparison.rs`
    touches), and the provenance header.
  - `neighborhood_read.genomes` becomes `genome_count`, and the page reads
    that instead of `.genomes.length`.
  - Output is compact JSON. Kept values are byte-identical to v1, and the
    undefined, missing and zero meanings are preserved.
  - Write the tests first:
    - a full report and its v1 summary give identical v2 content, except for
      `conversion.from_summary_v1`; each route repeated gives identical bytes
    - the loader rejects v1
    - every dropped top-level block appears in `omitted_details`
    - a v2 summary loads into comparison
- [x] **A2. Entry points.** `bench-summarize` and `make bench` produce v2 by
  running the existing full-report-to-v1 projection and then A1. A new
  `bench-summarize --from-summary-v1 <in> --out <out>` converts committed
  files.
  - It adds `conversion.from_summary_v1` with the input's path, SHA-256 and
    bytes.
  - `raw` still describes the original full report.
  - The loader accepts full reports and v2 summaries.
  - Migrate the existing tests that read retained detail from committed
    summaries: `bench/schema/tests.rs:160` (T12.F04 structure distributions)
    and `tests/bench_artifacts.rs:985` (recruitment detail). Historical
    deserialization coverage moves to a dedicated fixture, and obsolete
    retention assertions are replaced. The keep-list is never enlarged to
    satisfy a test.
- [x] **A3. Convert every committed `petri-benchmark-summary` at once**, at
  unchanged paths. Enumerate by `kind` at the starting revision, across
  `docs/progress/features/`, `features/historical/` and `docs/progress/sweeps/`.
  At `a9f59893` there were 147: 115 feature, 16 historical and 16 sweep
  summaries. Every converted file must load through the new loader, including
  files outside the series parity pairs.
  - Both parity environments (A3 comparison and A4 page) use one frozen
    copy of Phase A's starting `benchmark-series.json`, stored beside the
    oracle. Neither checkout's own index is used.
  - First copy each v1 file byte-identically to
    `<main>/.bench-artifacts/summary-v1/<same path>` as the parity oracle.
  - Re-converting the oracle gives identical bytes.
  - Comparison parity. There is no offline CLI comparison, and
    `compare_against_path` needs a full current `Report`. The summary
    loader (`artifacts.rs:528`) and `compare_inputs` (`comparison.rs:515`)
    are crate-private at both revisions. So add one identical
    ignored-by-default `#[cfg(test)]` harness inside the comparison module in
    both checkouts. It loads two stored artifacts through the lossless
    `ComparisonInputs` path, keeps the profile compatibility checks, prints
    the comparison as JSON, and runs no simulation.
    - Run it unchanged in a scratch worktree at the base revision over the v1
      oracle, and in the feature worktree over v2. Use identical logical
      reference paths.
    - Cover every pair that `benchmark-series.json` implies: each closed entry
      against its epoch baseline and against its predecessor.
    - The outputs must be identical: normalized readings, deltas and verdicts.
    - Record the pair count and the exact commands.
- [x] **A4. Progress page parity.**
  - `validateArtifact` in `docs/progress/index.html` accepts
    `summary_version` 2 and rejects 1. Full reports still pass.
  - Serve the base revision's `index.html` over the v1 oracle, and the new
    `index.html` over v2. The rendered visible content must be identical
    (text plus chart SVG, excluding `<script>` and the artifact notice).
  - Re-run the Proxy trace on both. Every path that resolved on v1 must also
    resolve on v2, with one declared exception: the reads of
    `neighborhood_read.genomes` and `neighborhood_read.genomes.length` map to
    `neighborhood_read.genome_count`, and the v1 array length must equal
    `genome_count`. The page keeps its `.genomes.length`
    fallback for full reports. Reads that are already absent in v1, such as T12.F04 case
    `mortality`, stay absent.
- [x] **A5. Relocate the other large files.** Move each file in the table,
  byte-identically, to `<main>/.bench-artifacts/research/<note-or-feature>/`,
  then `git rm` it.
  - The SHA-256 of each moved file equals the SHA-256 of its Git blob's
    contents.
  - Each note, reading or spec that links to a moved file gets one Evidence
    line with the local path, SHA-256 and bytes. That includes
    `docs/specs/roadmap/t13-f07-current-policy-recruitment-transitions.md:279`.
  - Historical provenance text, such as `status_at_audit`, stays unchanged.
- [x] **A6. Prevention.**
  - Add a POSIX `sh` tracked-size check under `scripts/`, called from
    `policy-check`. That target is shared by `make check`, `make check-docs`
    and CI (`.github/workflows/ci.yml:29`). It fails on any tracked file over
    1 MiB that is not on the allowlist.
  - Its test builds a temporary Git repo with a planted 1.1 MiB tracked file,
    and expects failure there and success on the real tree.
  - Replace the "Summary version 1" section of `docs/benchmark-artifacts.md`
    with v2 and the Decision 1 amendment. Also update its opening paragraph
    and its S0 row: an S0 summary over 1 MiB stays local-only evidence beside
    its raw file, and the committed reading carries its numbers. The S0
    writer is unchanged; its full-run summary is 1.33 MB compact.
  - Rewrite the affected T15 success criterion, the evidence criterion at
    `t15-benchmark-evidence-storage.md:24`, to the reporting-only rule, and
    add a dated note.
  - Land Phase B's tooling now so that `F` contains it. Extend
    `scripts/historical-benchmarks.mjs` to take an explicit blob-ID
    inventory; today it skips summaries and requires v1. Add fixtures for
    path reuse in both directions, aliases, modes, merges and empty commits.
- [x] **A6b. Absorb T11.F26 (user direction 2026-09-24).** T11.F26 is in
  progress. It edits `bench/artifacts.rs`, `docs/progress/index.html` (a new
  `mutation_effects` block the page reads) and the bench tests, and its
  closure commits v1 summaries. Once it is on `main`:
  - Rebase this branch.
  - Re-run the page trace so `mutation_effects` joins the keep-list.
  - Convert T11.F26's summaries.
  - Rerun the A3 and A4 parity with a refreshed oracle and a refreshed frozen
    series index.

  A7 waits for this step.
- [x] **A7. Review and land.**
  - Codex implementation review, recorded under Review.
  - `make check` exits 0.
  - The scoped mutation run.
  - Fast-forwarding `main` needs your authorization.

## Phase B — History Rewrite (separate authorization)

Follow the T15.F02 rewrite boundary and authorization procedure, including its
filter invocation:

- `--replace-refs delete-no-add`
- `--prune-empty never`
- `--prune-degenerate never`
- `--preserve-commit-hashes`
- `--preserve-commit-encoding`
- fresh-clone safety

Matching blobs become explicit deletions, so an earlier unrelated version at a
reused path cannot reappear.

- [x] **B1. Freeze and inventory.** The frozen source `F` is `main` after
  Phase A. Inventory every blob/path association in `F`'s ancestry where the
  blob is over 1 MiB and the path is under `docs/progress/features/` or
  `docs/strategy/`, excluding `F`'s own tip blobs.
  - For each, record the blob ID, every path and commit where it occurs, its
    bytes, and the SHA-256 of its contents.
  - Save byte-identical copies in `<main>/.bench-artifacts/historical/blobs/`,
    using the A6 tooling.
  - Make and verify `git bundle` backups of `F` and of every other local ref
    that reaches an inventoried blob.
- [x] **B2. Rewrite a disposable clone.** Clone
  `git clone --no-local --single-branch --no-tags --branch main` at `F`, filter
  only `refs/heads/main`, and copy `commit-map`, `ref-map` and `changed-refs`
  out immediately.
- [x] **B3. Prove it** in a separate fresh clone of the rewritten tip `R`:
  - `R`'s tree equals `F`'s tree.
  - No inventoried blob is reachable from `R`.
  - Every non-inventoried path matches in bytes and mode, commit by commit.
  - Record the packed size before and after.
- [x] **B4. Candidate, audit, approval.**
  - The candidate `C` is `R` plus one commit that adds three files:
    - `docs/progress/large-file-rewrite-manifest.json` (compact, under 1 MiB)
    - `docs/progress/large-file-rewrite-commit-map.txt`
    - `docs/progress/readings/large-file-cleanup-2026-09-24.md`

    `make check-docs` passes on `C`.
  - Enumerate refs dynamically: every local ref, including `refs/stash`, the
    turn-diff refs and the worktree HEADs, plus every ref origin advertises,
    including `refs/pull/*` from `git ls-remote origin`. T15.F02 found PR 12
    retaining 61 affected blobs. Mark a ref unobservable only if it genuinely
    cannot be listed. An advertised ref other than the
    `main` being replaced, including a PR ref, that reaches an inventoried
    blob blocks cutover until you decide what to do with it.
  - The approval package gives:
    - the directly captured remote `main` OID
    - `F`, `R` and `C`
    - inventory totals and the before and after packed sizes
    - the check results and backup paths
    - collaborator recovery steps
    - the expanded push command:

    ```sh
    git push --force-with-lease=refs/heads/main:<captured-remote-main-oid> origin <C>:refs/heads/main
    ```

  - After you approve: recheck the remote, then push only this ref. The
    receipt stays outside Git under `.bench-artifacts/large-file-rewrite/`. It
    records the push result, `main`/`origin/main` equality, a fresh-clone
    proof, and that other refs are unchanged.
- [x] **B5. Local prune (optional, separately authorized).** Run this after
  the receipt passes and the bundles are verified.
  - Stop writers in every linked worktree first.
  - Run
    `git -c gc.reflogExpire=never -c gc.reflogExpireUnreachable=never gc --prune=now`.
    This keeps every reflog entry, including the old `main`, until you
    separately approve `git reflog expire`.
  - Record the `.git` size before and after.

## Verification

- A1–A2: focused `cargo test -p v3-cli` and `node --test scripts/*.test.mjs`.
- A3: re-conversion is byte-identical, and the parity test passes with a
  recorded pair count.
- A4: the rendered page is identical, and the v2 trace finds no missing reads.
- A3 evidence (2026-09-24, refreshed after A6b): 151 summaries (119 feature,
  16 historical, 16 sweep), 223,471,710 → 9,290,520 bytes; largest v2 file
  269,652 bytes (`t11-f26-…-goal.json`). Oracle and frozen index (from
  `02d34d75`; the earlier copy is kept as `benchmark-series.a9f.json`) under
  `<main>/.bench-artifacts/summary-v1/`. All 151 were re-converted from the
  oracle twice (worktree and temp dir): byte-identical, each re-loaded; only
  the two T11.F26 files differ from the pass-3 conversion. The harness
  `bench/comparison/parity.rs` covers 202 pairs (gate 110, goal 30,
  goal-worlds 62) plus the `ComparisonInputs` of all 151 files, 0 load errors
  or profile mismatches; outputs are byte-identical (3,016,811 bytes, sha256
  `7cf9eceb…`), saved as `summary-v1/parity/parity-v{1,2}-a6b.json`:

  ```sh
  PETRI_PARITY_ROOT=<root> PETRI_PARITY_SERIES=<main>/.bench-artifacts/summary-v1/benchmark-series.json \
  PETRI_PARITY_OUT=<out> cargo test -p v3-cli --lib stored_artifact_comparison_parity -- --ignored
  ```

  run at `02d34d75` plus the same patch (own `CARGO_TARGET_DIR`) with root
  `summary-v1/`, and here with the repo root.
- A7 remediation evidence (2026-09-24): the v1 route now writes
  `comparison_inputs` as stored JSON (the loader validates a typed copy) and
  refuses a conversion whose typed model would change any whole-kept block;
  pass 5 had added absent pass counters as `null` in 135 files. All 151 were
  re-converted from the oracle twice: byte-identical, 223,471,710 → 9,282,909
  bytes (largest still 269,652), and every whole-kept block is JSON-equal to
  its v1 source (`conversion` less `from_summary_v1`); nothing outside
  `comparison_inputs` changed from pass 5. Parity rerun on both sides (base in
  a scratch worktree at `02d34d75` plus `summary-v1/parity/base-harness.patch`):
  202 pairs over 151 files, 0 load errors, profile mismatches or panics;
  `parity-v{1,2}-a7.json` byte-identical (3,016,811 bytes, sha256
  `7cf9eceb…`, same as A6b). `historical-benchmarks.mjs` verify and export now
  require summary version 2, and `inventory-blobs` reads the JSON array
  `large-blobs` prints.
- Rebase evidence (2026-09-24): rebased onto `28439051` without conflicts and
  squashed to one commit (tree unchanged). All 151 re-converted from the oracle
  twice are byte-identical to the committed files, so none were rewritten.
  Parity at `28439051` plus `summary-v1/parity/base-harness-rebase.patch`
  (the same harness, new hunk offset): 202 pairs, 151 files, 0 errors;
  `parity-v{1,2}-rebase.json` byte-identical (sha256 `7cf9eceb…`, as A7).
  The ignored `historical_corpus_preserves_claims_identity_and_all_reference_comparisons`
  passes over the committed v2 summaries with
  `summary-v1/parity/historical-manifest-v2.json` (the T15.F02 `final-v3`
  manifest with summary paths moved to this worktree): 93 reports, 2,349
  compatible and 6,300 incompatible pairs, as in T15.F02.
- A4 evidence (2026-09-24, refreshed after A6b): `summary-v1/parity/parity-server.js`
  served `02d34d75`'s page over v1 and this page over v2 with the frozen index
  at 1280×900 (a `ResizeObserver` shim draws charts in the hidden pane).
  Visible text before and after all 42 table toggles and 3 details, and 113
  chart SVGs (2,317,679 bytes), are identical outside the artifact notice,
  including the six T11.F26 `mutation_effects` tables (measured values, e.g.
  Orchards drift@0 0.493 requested per birth). Traces: 11,174 v1 paths; the
  only differences are T11.F26 goal `neighborhood_read.genomes{,.length}` →
  `genome_count`, with equal counts in all 69 cases (23 files) that store
  genomes. The v1 trace re-derived the read-set fixture: 76 `mutation_effects`
  lines added, none removed (446 lines).
- A5: the SHA-256 values match, and `git grep` finds no live link to a moved
  filename.
- A5 evidence (2026-09-24): all seven files copied to
  `<main>/.bench-artifacts/research/<note-or-feature>/`; for each, `git cat-file
  -p <blob> | shasum -a 256` and `git cat-file -s` equal the copy's SHA-256 and
  bytes (1,786,584 to 7,758,320 bytes; S0 `1dab13f5…` matches the T13.F07
  reading). Evidence lines added to the five strategy notes,
  `incremental-recruitment-research-2026-09-19.md`, the T13.F07 spec and its
  reading; their numbers were already in those files. Residual `git grep` hits
  are only this spec's census table and `status_at_audit`. Nothing else reads
  the S0 summary (the parity harness filters on `petri-benchmark-summary`), and
  the 467,195-byte `-s0-pilot.json` stays.
- A6 evidence (2026-09-24): `scripts/tracked-size-check` (index-based, allowlist
  by exact path) runs inside `scripts/policy-check`; `scripts/tracked-size-check-test`
  (`make tracked-size-check-test`, part of `make policy-check`) plants a
  1,153,024-byte tracked file in a temporary repo (fails), covers exactly 1 MiB,
  allowlisted paths, untracked files and an allowlisted name at another path,
  and passes on the real tree. `docs/benchmark-artifacts.md` and the T15
  evidence criterion and note are rewritten. `scripts/historical-benchmarks.mjs`
  adds `large-blobs SOURCE PREFIX...` (blob IDs over 1 MiB at the prefixes in
  the ancestry, tip blobs excluded, printed as a JSON array) and
  `inventory-blobs SOURCE PACKAGE BLOB_IDS` (reads that array; every path and commit, bytes, SHA-256, a copy in the sibling
  `blobs/`, and one `filter-inputs.json` pair per path for the T15.F02
  deletion callback; its manifest feeds `verify-history`). Two fixtures in
  `historical-benchmarks.test.mjs` replay the history with deletions: path
  reuse before and after, an alias, a merge, an executable mode, a commit left
  empty, and a restored earlier version failing `verify-history`.
- A6b evidence (2026-09-24): rebased onto `02d34d75`; the one conflict (both
  sides' `bench_artifacts.rs` tests) kept both. The keep-list gains the page's
  `mutation_effects` reads (A3, A4 above).
- A6: the size-check test passes through `make policy-check`.
- `make check` exits 0 on the final Phase A code.
- Mutation gate evidence (Decision 4, 2026-09-24): the fresh
  `MUTANTS_ITERATE=0 make rust-mutants` ran on `8a3d2afa` against `28439051`.
  The in-diff scope produced mutants only in `bench/artifacts.rs` (39) and
  `main.rs` (4). `comparison.rs` adds only module declarations, and
  `comparison/parity.rs` is `#[cfg(test)]`. Summary: `43 mutants tested in 22m:
  3 missed, 33 caught, 7 unviable`, 0 timeouts. Output is in
  `~/.local/share/petri-tools/mutants/large-file-cleanup/mutants.out`; the
  incremental pass below later overwrote it. Each survivor was **killed** by a
  test in `crates/v3-cli/tests/bench_artifacts.rs`:
  - `main.rs:214:17` deletes the `(None, Some(input), Some(provenance))` arm:
    killed by `bench_summarize_converts_a_full_report_with_provenance`.
  - `artifacts.rs:891:32` changes `||` to `&&` in `project_v2`: killed by
    `project_v2_rejects_a_version_1_summary_of_another_kind`.
  - `artifacts.rs:1199:9` changes `||` to `&&` in `convert_summary_v1`: killed
    by `from_summary_v1_rejects_a_version_1_header_of_another_kind_before_parsing`.
  The `MUTANTS_ITERATE=1` feedback pass reported `3 mutants tested in 5m: 3
  caught`. No second fresh run was needed because production code, test
  selection, and configuration were unchanged.
- B1–B4: the helper fixtures pass, the B3 proofs pass, and the receipt passes.
- Cutover 2026-09-25 (user approved in chat): Phase A landed as `712695ac`
  (fast-forward). Phase B replaced `main` with `C` = `8b03f5d2` (46 blobs,
  232,761,030 bytes stripped; packed 20.3 → 13.3 MiB) by a leased push from
  `4b88eee7`. The receipt passes: only `main`/`HEAD` changed on origin, a fresh
  GitHub clone has HEAD `C`, `fsck` clean, 0 of 46 blobs, and `verify-history`
  exit 0; locally only `main` and `origin/main` moved. Package, bundles and
  receipt are in `.bench-artifacts/large-file-rewrite/2026-09-25/`.
- B5 2026-09-25: `.git` was already 30 MB (the 1.33 GB pack had been pruned
  by automatic gc). The prune ran as `gc --prune=1.hour.ago` with reflog and
  worktree expiry disabled, which is safe against concurrent sessions: 30 MB →
  28 MB, `fsck` connectivity clean. Pre-rewrite refs such as the
  `large-file-cleanup` branch, `refs/stash` and the `claude/`, `codex/` and
  `scratch/` branches still hold the old blobs; rebase or cherry-pick from
  them, never merge.

## Success Criteria

- [x] Every committed benchmark summary is v2 and holds only the keep-list.
  Comparison output and the rendered progress page match v1 on every series
  entry, and each v1 file is preserved locally.
- [x] No tracked file on `main` exceeds 1 MiB outside the allowlist, and
  `make check-docs` enforces this.
- [x] Relocated files are local, hash-verified and cited.
- [x] After approval, `origin/main` equals `C`, no inventoried blob is
  reachable from it, and `R`'s tree equals `F`'s tree.

## Notes for AI Agents

- Hard stops that need your authorization: committing to `main`, the force
  push, any ref deletion, and pruning or reflog expiry.
- The page trace used a local static server that injected a `Response.json`
  Proxy wrapper. Re-create it for A4 rather than relying on a static grep of
  `index.html`.

## Review

| Round | Codex job | Verdict |
| --- | --- | --- |
| PRD 1 | `task-mug0d9pz-sxta4k` | not-ready: 7 blocking, 4 should-fix; applied in PRD 2 (history stripped not converted, keep-list from traced consumers, page validator and render parity, v1 parity oracle, deletion semantics, candidate ordering, dynamic ref audit, reflog-safe prune, census fixes) |
| PRD 2 | `task-mug2tbz7-91t31z` (after `task-mug12dhl-tdh44w` stalled and was cancelled) | not-ready: 2 blocking, 5 should-fix, 1 advisory; applied in PRD 3 (full trace as read set, loader reads, base-revision oracle reader and page, test migration, provenance-aware byte parity, T15 criterion rewrite, Phase B tooling moved into Phase A, kind-based enumeration, `main` lease exemption) |
| PRD 3 | `task-mug484hy-kaql0i` (after `task-mug34zy2-q6qxa2` stalled and was cancelled) | not-ready: 2 blocking, 3 should-fix; applied in PRD 4 (offline two-revision comparison harness, `genome_count` trace mapping, size check in `policy-check`, PR-ref audit, sweep summaries included: 147 at base) |
| PRD 4 | `task-mug5533a-rc7rur` (after `task-mug4j1up-pxu5le` stalled and was cancelled) | not-ready: 1 blocking, 2 should-fix; applied in PRD 5 (in-module `#[cfg(test)]` parity harness at both revisions, `genomes` container exemption, S0 local-only contract and T13.F07 spec link) |
| PRD 5 | `task-mug5hfvp-oolo8a` | ready; one advisory applied (one frozen `benchmark-series.json` shared by both parity environments) |
| Implementation 1 | `task-mugg7lrc-wquymp` | changes-requested: 3 should-fix, 1 advisory; fixed in `92b3d8d8` (whole-kept blocks written as stored, v2 accepted by historical verify/export, `large-blobs` JSON fed to `inventory-blobs`, short artifact notice) |
| Implementation 2 | `task-mugi1zpl-n6frrn` | approve; two advisories applied (loader-vs-page rejection wording in `docs/benchmark-artifacts.md`, parity harness now asserts full coverage and zero load errors) |
