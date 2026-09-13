# T15.F02 — Historical Report Migration and Main-History Rewrite

**Status**: In Progress
**Last updated**: 2026-09-13
**Feature**: T15.F02
**Track**: [T15 — Benchmark Evidence Storage](../../roadmaps/t15-benchmark-evidence-storage.md)

## Goal

Every historical benchmark report reachable from the exact starting `main` has
its raw versions preserved locally and its final valid version represented by
committed concise evidence. Inventoried full reports are unreachable from the
rewritten `main`, which replaces `origin/main` only after explicit cutover
authorization and an exact remote lease check.

## Non-Goals

- New measurements, simulation or assay reruns, recomputed historical verdicts,
  threshold changes, baseline repins, or a new summary format.
- Rewriting other branches, tags or pull-request refs; changing protection
  settings; remote artifact hosting; physical erasure from other clones,
  forks, hosting caches, reflogs or retained refs.
- A reusable migration framework or changes to simulation behavior.

## Inputs and Invariants

**Frozen source.** Inventory the complete ancestry of
`84c9c7e2afad3f1621d1ada2988f9fe851af4cb9`, the recorded pre-rewrite `main`.
Work in `/Users/istefanek/projects/petri/.worktrees/t15-f02` on
`codex/t15-f02`; store raws in main's ignored
`/Users/istefanek/projects/petri/.bench-artifacts/historical/`. Transport later
feature commits separately. Record local and directly advertised remote OIDs;
a moved main blocks cutover, never silently refreshes the inventory or lease.

[T15.F01](t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.md)
provides the version-1 summary, ignored raw root and `bench-summarize`.
Reuse `crates/v3-cli/src/bench/artifacts.rs::convert`, `ComparisonInputs` and
`compare_against_path` in `crates/v3-cli/src/bench/comparison.rs`, and their
tests in `crates/v3-cli/tests/bench_artifacts.rs`, following
[the artifact contract](../../benchmark-artifacts.md). Preserve
`docs/progress/benchmark-series.json` reference meanings and the progress page's
ability to consume summaries without fetching raw files.

**Inventory and logical identity.** Enumerate every reachable commit/tree and
blob/path association, including merged ancestry, deleted paths and renames.
Inspect content, locations and report references; current files, extensions,
size cutoffs and `git rev-list --objects`' single name per object cannot prove
completeness. Record enumeration digests/counts and classification evidence for
gate/goal/sweep reports, earlier versions and malformed candidates. Distinguish
summaries, configuration, fixtures and unrelated JSON with explicit exclusion
reasons. Unknown candidates remain unresolved.

Group versions by report lineage, paths and measured identity, keeping separate
sweeps/reruns distinct. Retain every alias/rename and unique blob version.
Select the valid tip version, otherwise the final valid historical version;
record ancestry-based selection and later invalid versions. Resolve incomparable
histories/path reuse before filtering. Duplicate bytes may share a raw file,
with every report/path association retained.

| Artifact | Required content and location |
| --- | --- |
| Raw corpus | One byte-identical file for every inventoried unique full-report/candidate blob at `<main>/.bench-artifacts/historical/blobs/<git-blob-id>.json`; preserve invalid bytes too. No tracked raw copies. |
| Committed summaries | One T15.F01 summary for each logical report with a valid version. Reuse its current report path when safe; historical-only reports use collision-free paths under `docs/progress/features/historical/`. Existing summaries remain unchanged. |
| Manifest | `docs/progress/historical-benchmark-manifest.json`: format version, frozen source, enumeration evidence, logical IDs, all original paths and version blob IDs, occurrence/ancestry evidence, selected version and reason, profile/measurement identity when known, raw SHA-256/bytes/path, summary path/hash/bytes, explicit conversion result/error, availability state and verification time, historical-only flags, exclusions and unresolved counts. |
| Conversion inputs | Fixed, saved provenance per selected report: converter executable/arguments/working directory and verification time. Original measured revision/time and unknown evidence remain untouched. |
| Recovery/export package | A named directory outside the disposable clone, under main's ignored historical root or another verified persistent local location: all raw versions, summaries, manifest, provenance, filter inputs, pre-rewrite Git bundle, ref/worktree audit, candidate bundle, maps and receipts. Record absolute paths and hashes. |
| Commit mapping | Export `.git/filter-repo/commit-map`, `ref-map` and `changed-refs` immediately after filtering; commit the old-to-new map at `docs/progress/historical-benchmark-commit-map.txt`. Record the later summary/closure commit separately. |
| Readings and handoff | `docs/progress/readings/t15-f02-historical-report-migration-and-main-history-rewrite.md`: inventory/size totals, conversion and parity results, check commands/exits, tested revision, filter version/command, map/export hashes, ref audit, approval package and recovery instructions. Bulky listings/transcripts remain in the local package. |

A report with no valid version is explicitly `unparsable`/`unsupported`, with
preserved raw, exact error and null summary path. Resolve possible converter
defects with the spec owner; unexplained gaps block cutover. Never synthesize
measurements. Historical-only valid reports receive summaries; later invalid
versions do not erase earlier valid evidence. Preserve series ordering and
baseline choices, updating paths only where the summary mapping requires it.

Repeat each T15.F01 conversion with identical raw path and explicit provenance;
require identical output bytes. Match all raw copies to Git bytes, SHA-256 and
length. Preserve comparisons, measured identities and undefined/missing/unknown
evidence. Offline parity exercises the existing comparison API for every selected
full/summary pair and available compatible reference pairs; normalize only the
reference-path change. Incompatible/absent references retain their behavior.

**Rewrite boundary.** Before filtering, export/verify raws, summaries,
provenance, manifest, filter inputs and a bundle that independently restores
the frozen source and inventoried blobs. The commit map cannot exist before
rewriting: reserve its export destination and copy generated maps out immediately
after filtering, before discarding the clone.

Use `git clone --no-local --single-branch --no-tags --branch main` from the
frozen source/backup and verify its OID. Execute the actual rewrite in this
disposable fresh clone as the live repository's dry run; `--dry-run` alone
does not establish an object graph. Keep filter-repo's fresh-clone safety check.
Rewrite only `refs/heads/main`, create no replacement refs, preserve messages
and topology with supported options, and record command/version. Filter exact
inventoried paths/aliases and blob IDs, never broad directories or sizes.
Unrelated versions at a reused path must survive; unresolved collisions block
path-wide filtering.

Before restoration, prove raw paths/blobs absent from rewritten-main ancestry
and mapped unrelated paths identical in bytes/modes; account for changed IDs,
signatures and preserved empty commits. Then restore exported summaries and
scoped migration/closure files. A reused raw path may contain its new summary
only after filtering, never an earlier full-report version. Compare the final
tree with the frozen tree plus explicit migration changes.

**Research decision (2026-09-13).** Use mandated `git-filter-repo`, T15.F01 and
a small Node helper with built-in filesystem/JSON/hash support. Manual commands
fit cutover; reproducible corpus inventory/export/verification needs the helper.
A separate projection duplicates T15.F01. The official
[filter-repo manual](https://github.com/newren/git-filter-repo/blob/main/Documentation/git-filter-repo.txt)
documents blob/path filtering, renames, safety, maps and `--dry-run` limits.
`--refs` implies partial rewriting, so test main reachability and a separate
clone, not global object deletion. Git's
[clone documentation](https://git-scm.com/docs/git-clone) supports `--no-local`
for an independent local clone. Git's
[push documentation](https://git-scm.com/docs/git-push)
specifies the exact expected-value lease; a remote-tracking ref alone is
insufficient. GitHub's
[rewrite guidance](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)
identifies signature, PR and recontamination effects. The dry run resolves
historical classification/grouping and ref retention.

**Exceptional integration and authorization.** Audit linked worktrees, local
branches/tags/other refs, advertised remote refs and available PR refs. Record
OIDs, dirty work and old-history retention/reintroduction risks; state unavailable
PR visibility. Preserve other refs. Contacting collaborators or changing branch
protection needs separate authorization.

Ordinary close's fast-forward, automatic rebase and branch deletion do not apply.
Keep the original feature worktree/branch as recovery evidence; never merge its
old ancestry into the candidate. Source status stays `In Progress` pending
permission. The verified disposable candidate may stage `Complete` metadata and
the checked row as proposed release state. Its checkboxes attest completed
preparation/checks, not an unobserved push. This staging exception allows approval
of the exact tip before the sole force-push. Actual cutover proof stays in an
external receipt, avoiding self-referential OIDs or another push. Overall
completion requires that receipt to pass.

Before any destructive local-main replacement or remote update, present the
directly captured remote main OID, exact old local main and proposed new main
OIDs, inventory totals/bytes and omissions, exact changed-ref set, all checks,
backup/restore locations, repository effects, collaborator recovery procedure,
and the fully expanded command below. Present the local replacement command as
well. Pause for explicit user authorization of this concrete package.

```sh
git push --force-with-lease=refs/heads/main:<captured-remote-main-oid> origin <approved-new-main-oid>:refs/heads/main
```

After authorization, recheck remote main directly and abort on mismatch; retain
the same exact lease for the push race. Recheck local main identity/cleanliness,
candidate OID and package hashes. Execute only the approved local replacement
and single-ref push. No mirror/all/tags push, unconditional force, old-history
merge, ref cleanup or destructive GC. On failure preserve both states and record
the blocker. Verify remote OID, advertised-ref delta and a fresh single-branch
clone. Updating tracking `origin/main` is observational; unrelated refs/worktrees
retain their audited state.

## Implementation Tasks

- [x] Create the bounded inventory/export/verification tooling and fixtures;
  record the complete frozen-source census and all classification decisions.
- [x] Preserve every raw version, convert each selected valid report twice,
  verify identity and offline comparison parity, and assemble the manifest and
  independently recoverable export package.
- [ ] Filter a disposable fresh clone, export its maps, restore summaries and
  scoped feature files only afterward, and prove history/tree preservation.
- [ ] Complete final review and required checks; assemble the immutable cutover
  candidate, staged closure metadata, exact ref audit and approval package.

## Verification

Results live in the readings and hashed local receipts named above.

- [x] Focused inventory/export/verifier fixtures and stored-artifact parity
  checks pass; record exact commands. New behavior uses TDD, including deleted,
  renamed, repeated and invalid versions, aliases and unrelated path reuse.
- [x] Frozen-source census covers every reachable tree/blob/path association;
  all candidates have dispositions, every logical report has its selected valid
  summary or documented raw-only disposition, and totals reconcile.
- [x] Every raw version matches Git bytes/hash/length and is ignored; each
  selected conversion is byte-identical across repeats, with correct provenance
  and unchanged stored comparisons. Series references resolve to summaries.
- [ ] Pre-rewrite backup restores independently; external exports and map
  hashes verify; only inventoried full-report history is removed, mapped
  unrelated trees match, and candidate contains no tracked raw artifacts.
- [ ] A separate fresh candidate clone passes raw-blob/path absence,
  manifest/summary completeness and tree checks. Record raw absence scoped to
  rewritten main, not global object deletion.
- [ ] `make check` passes on the final source/script content; record its tested
  revision. After staged closure edits, `make check-docs` and
  `make roadmap-check` pass on the exact cutover candidate.
- [x] Mutation: not applicable while the diff has no mutation-testable Rust
  production code. If that changes, the separate mutation specialist runs fresh
  `MUTANTS_ITERATE=0 make rust-mutants` on final code; record its summary, path
  and every survivor disposition here before cutover.
- [x] Benchmark runs: not applicable. Only historical stored artifacts are
  converted; no `make bench`, historical simulation rerun or new baseline is
  permitted. Existing normal regression tests remain required.
- [ ] The approval package names the verified candidate, exact lease and
  commands, ref/worktree audit, backups and recovery instructions. It requires
  an external receipt of authorization, remote recheck, push result, main/origin
  equality, postpush fresh-clone proof, unchanged other refs and clean main.

## Performance and Goal Impact

**Predeclaration — written before the run.** This is a stored-evidence and Git
history migration. No simulation/assay, production defaults, world recipes,
environmental pressures, work counters or indicator definitions change. No
natural analog applies and no indicator or compute direction is predicted.
Expected changes are migration documentation/data, a small Node helper and its
fixtures, and if needed a test-only Rust harness for offline corpus comparison.
No production Rust, application or build-configuration change is expected.
Any necessary behavioral fix returns to the spec owner and the persistent
implementer, uses TDD and the Rust skill where applicable, and receives the
required checks/review. `make check` applies to the final implementation;
mutation applies only if mutation-testable code enters the diff.

No benchmark specialist or benchmark run is permitted: recomputation would
replace historical evidence and cannot establish a storage migration's
correctness. Preserve original thresholds, flags, measured identities and
comparison verdicts; do not re-pin an epoch. Conversion and Git storage sizes
are migration evidence only, recorded in the readings.

**Measured verdict.** No new measurement. All 97 raw versions (85,992,446
bytes) are preserved; 93 summaries (28,574,089 bytes) convert identically on
repeat. Four unsupported historical shapes have explicit raw-only dispositions.
Offline parity passes for 2,349 compatible and 6,300 incompatible pairs,
preserving stored verdicts and measurement identities. See the
[readings](../../progress/readings/t15-f02-historical-report-migration-and-main-history-rewrite.md)
for census evidence, exact dispositions, checks, and pending cutover work.

## Success Criteria

- [ ] Every inventoried raw version is preserved, every valid logical report
  has its concise committed summary, and the manifest accounts for all paths,
  identities, availability evidence and raw-only dispositions.
- [ ] The exact proposed rewritten main preserves unrelated history/content,
  excludes inventoried raw blobs, and passes separate fresh-clone verification.
- [ ] The staged Complete spec, checked row, applicable rollups and immutable
  approval package agree on the candidate and required cutover; overall delivery
  requires the external receipt proving approved local/remote replacement,
  matching `origin/main`, unchanged unauthorized refs and clean main.

## Notes for AI Agents

- Decision: The only history-rewrite target is `main`; explicit user approval
  of the exact dry-run package is required before local replacement and remote
  force-push. Preserved refs/clones may still retain old history.
- Exception: Complete metadata in the disposable candidate is staged release
  state, not a preauthorization assertion that cutover happened. The original
  worktree remains In Progress until live cutover passes the external receipt.
- Exception: No new benchmark is permitted for this stored-artifact migration;
  verification reuses T15.F01 conversion/comparison and existing measurements.
- Cost: Closure usage and role/review/advice counts are pending; planning and
  readiness self-review are not advisor consultations.
