# T15.F02 — historical report migration and main-history rewrite

Preparation is in progress. No live ref has been rewritten and no cutover is
authorized. Frozen main is `84c9c7e2afad3f1621d1ada2988f9fe851af4cb9`;
the source feature branch starts with plan commit `bcbb248495e3f22f33e542d56b81b0b90b8c12dc`.
No historical simulation, assay, benchmark, or baseline was rerun for this migration.

## Inventory and preserved evidence

The [manifest](../historical-benchmark-manifest.json) records every report/path
association, measurement identity, version frontier, conversion input/result,
raw and summary hash, exclusion, and unsupported disposition. The original
raw/conversion/rewrite recovery package remains preserved at:

`/Users/istefanek/projects/petri/.bench-artifacts/historical/t15-f02-20260913-complete/`

The corrected reference census, current manifest and post-review receipts live at
`/Users/istefanek/projects/petri/.bench-artifacts/historical/t15-f02-20260913-remediation-v2/`.
This supplementary package reuses the original immutable raws, converter,
summaries and provenance; preserve both directories.

| Frozen-source census | Count |
| --- | ---: |
| Commits, including merged ancestry | 1,179 |
| Merge commits | 40 |
| Unique trees, including subtrees | 8,740 |
| Unique root trees | 1,166 |
| Unique blobs | 7,181 |
| Blob/path associations | 8,147 |
| Historical report/index references from 111 source blobs | 1,011 |
| Entries across unique root-tree listings | 809,883 |
| Entries across all commit-tree occurrences | 815,103 |
| Logical reports / unique raw report blobs | 97 / 97 |
| Historical-only logical reports | 19 |
| Converted summaries / supported historical-only summaries | 93 / 16 |
| Explicit unsupported raw-only reports / unresolved decisions | 4 / 0 |

The commit/tree census reconciles with the independently enumerated reachable
object set. Every blob was inspected without a path, extension, or size cutoff.
Historical comparison and series references are resolved against their source
introduction trees, preserving even unrecognizable referenced candidates while
leaving unrelated later path reuse intact. Two index references precede their
report's first commit; both later report blobs are inventoried, and the initial
absence is recorded rather than inferred as local availability.
Exclusions distinguish existing summaries, fixtures, configuration/other JSON,
text/binary files, and the two T03.F08 NDJSON simulation traces. The latter are
`v3-cli run` protocol events and tick samples, not benchmark report envelopes;
their configuration, runner, timing file, and bytes remain unchanged.

All 97 raw blobs occupy **85,992,446 bytes**, preserved byte for byte under
`/Users/istefanek/projects/petri/.bench-artifacts/historical/blobs/<git-blob-id>.json`.
Both the Git object identity and SHA-256/length verify. The 93 summaries occupy
**28,574,089 bytes**: 77 reuse current report paths and 16 use collision-free
historical paths. Three pre-existing summaries remain unchanged. The 54 current
series references all resolve to supported summaries; series ordering, epochs,
and omission meanings are unchanged.

Separate measurement timestamps/revisions/effective profiles identify separate
runs. Equal metadata at unrelated paths is insufficient to join lineages. The
T12.F04 first-pass rename is retained as an alias; versions are ordered by
ancestry, not measurement time. Fixtures cover ambiguous merges, later invalid
versions, and unrelated path reuse, which must not cause silent selection or
overbroad filtering.

## Unsupported historical shapes

These four blobs have null summary paths and no parity claim. Every original
byte, measured identity, source path, converter invocation, and exact rejection
is retained in the manifest. Advisor consultation 2 accepted these dispositions;
no production compatibility change is needed.

| Blob | Preserved reason |
| --- | --- |
| `c86114628181448c26c7f10b23beb56f8f58d2e4` | T11.F04's intentionally unreadable pre-schema-fix gate stores an integer-keyed requested-event histogram; the supported shape uses ordered buckets. The [original spec](../../specs/roadmap/t11-f04-mutation-supply-and-neutral-scaffold.md) expressly excludes it from series references. Its former tracked raw file is removed; its local raw remains recoverable. |
| `8fcae474fa29b6b049ddd87d95d4357550294d12` | Superseded T13.F01 goal at `ab00bb07`: per-lineage `selected_inapplicable` predates the corrected `discarded_selected_inapplicable` observation. |
| `e6d401d5ec40c56b091955853d357db1196a76aa` | Superseded T13.F01 goal at `afa6ae5c`, with the same retired field semantics. |
| `e2f57a9d7a418cdbb3720924f56e8d802c9fc2dc` | Superseded T13.F01 goal at `c1cfee5a`, with the same retired field semantics. |

The [T13.F01 readings](t13-f01-module-recruitment-observability.md) document
the superseded runs and the semantic correction. Aliasing the old field or
defaulting the new field would invent corrected observations. Immutable source
document blob IDs are included in each manifest disposition.

## Verification

Commands ran from `/Users/istefanek/projects/petri/.worktrees/t15-f02`.
Migration-specific tests exercised stored or synthetic bytes only; ordinary regression checks are
separate from historical measurement. The source remained uncommitted atop the
plan revision during this pass; final tested revision and candidate checks are
pending.

| Command | Observed result |
| --- | --- |
| `node --test scripts/historical-benchmarks.test.mjs` | Exit 0, initially nine fixtures; ten after the post-review root-level series regression. Missing-module/export and behavior assertions failed before their implementations, including incomplete-profile, unreachable-map, and historical-reference regressions. |
| `cargo test -p v3-cli --test bench_artifacts offline_pair_parity_preserves_compatible_incompatible_and_absent_references` | Exit 0, one test passed. |
| `cargo check --workspace --all-targets` | Exit 0. Production Rust is unchanged. |
| `cargo test -p v3-cli --lib historical_goal_` | Exit 0, two tests passed after replacing migrated full-report assumptions with the existing synthetic full fixture and exact retained-aggregate round trips. |
| `node scripts/historical-benchmarks.mjs inventory 84c9c7e2afad3f1621d1ada2988f9fe851af4cb9 <package>` | Complete census and object-set reconciliation passed. |
| `node scripts/historical-benchmarks.mjs convert <package> target/debug/v3-cli` | 93 selected conversions repeated byte-identically with saved fixed provenance; four exact unsupported rejections retained. |
| `node scripts/historical-benchmarks.mjs verify <package>` | Exit 0: all raws, exports, summary identities/provenance, and explicit dispositions verify. |
| `PETRI_HISTORICAL_MANIFEST=<package>/manifest.json cargo test -p v3-cli --test bench_artifacts historical_corpus_preserves_claims_identity_and_all_reference_comparisons -- --ignored --exact --nocapture --test-threads=1` | Exit 0: 93 selected reports, 2,349 compatible pairs, 6,300 incompatible pairs. Only reference paths were normalized; stored claims, comparisons, measurement identities, and unknown evidence matched. |
| `node scripts/historical-benchmarks.mjs export <package>` | Exit 0: 93 summaries exported, 54 series references verified, one unsupported tip raw removed after preservation. |
| `git bundle verify <package>/pre-rewrite.bundle` | Exit 0; complete frozen-main history, no prerequisites. |
| `git clone --no-local --single-branch --no-tags --branch main <package>/pre-rewrite.bundle <package>/backup-restore` then `git -C <package>/backup-restore fsck --full --strict` | Both exit 0. |
| Independent restored-clone identity, commit count, and batch object check | Exact frozen OID, 1,179 commits, all 97 raw objects with matching sizes; exit 0. |
| Main-checkout `git check-ignore --stdin -z` for every preserved raw path | All 97 raw paths ignored; exit 0. |
| `make roadmap-check` | Exit 0 after the source document edits. |
| First `make check` | Exit 2 at two schema unit tests that included migrated documentation files as full-report fixtures. Both test-only assumptions were corrected. |
| Second `make check` | Exit 2: seven server listener tests could not bind in the sandbox; no production defect. |
| Approved unrestricted `make check` | Exit 0: all gates passed, including 95 server tests, 61 frontend test files / 322 tests, and the frontend build. |
| `make check-docs` | Exit 0 after adding the check outcomes and ref/worktree audit evidence. |

`<package>` expands to the absolute package path above. Mutation is not
applicable: no mutation-testable production Rust changed. Benchmark runs are
not applicable and are prohibited by the spec.

## Recovery and remaining cutover preparation

The complete pre-rewrite bundle and its independent restored clone are outside
any rewrite candidate. The manifest names hashed census, occurrence, filter,
raw, provenance, converter executable, and summary exports. An earlier package
named `t15-f02-20260913` contains a preliminary root-tree census and is retained
as non-authoritative preparation evidence; use the `-complete` package above.

The pre-rewrite bundle is 10,477,647 bytes, SHA-256
`2fea87b5691d05414f435178d952d77b96cc2083a6d470e9d37e3e6c677a4646`.
To recover the frozen source independently, clone that bundle with the exact
single-branch command recorded above. The unmodified source feature branch and
all other original refs/worktrees are retained.

The generated `ref-worktree-audit.json` records 42 exact local refs and 13 linked
worktrees, with read-only status receipts: five worktrees are dirty, including
this feature. Forty-one local refs and all 13 worktree heads retain affected
blobs. All 27 Codex checkpoint refs target trees and were traversed directly;
their retained raw counts range from 36 to 78. No refs or worktrees were changed.
Its SHA-256 is `a62a7f07ae93fa627cd81f01ebf5af4dd4e5f8635dbc15864363d60fd0672f76`.

The orchestrator's direct remote advertisement on 2026-09-13 is preserved as
`remote-advertisement.json`, independently of cached remote-tracking refs.
Every advertised object is available locally for reachability proof: origin
main/HEAD at `a49478eb0172729aadbfe2a0182f0d325406c6b7` retain all 97 raws;
PR 12 retains 61; PRs 1–11 and `codex/streamline-lean-delivery-workflow` retain
none of this corpus. No tags were advertised. A main-only rewrite therefore
does not establish whole-host object erasure. Unadvertised refs, remote reflogs,
and hosting garbage collection remain outside this proof. This dated audit is
not a cutover lease; recapture main directly immediately before the approval package.

Self-review reused Git's batch object reader and the existing converter and
comparison API, with no dependencies or production abstraction. It added
explicit subtree enumeration and object-set reconciliation, rejected incomplete
effective profiles, required mapped commits to be reachable from the candidate,
checked author/committer metadata alongside messages and topology, and replaced
the ambiguous-merge fixture's edited manifest with an actual merged history.
It also follows historical references before classifying unrecognizable bytes;
an independently regenerated census produced identical report/exclusion sets
and enumeration hashes. The nine focused fixtures pass after those changes.
Advisor consultations: 4;
the accepted identity rule and four unsupported dispositions are recorded above.

## Disposable rewrite and review preparation

The official `git-filter-repo` v2.47.0 script (upstream revision `a40bce548d2c`)
is preserved in the package as `git-filter-repo-v2.47.0`, with source URL,
version command and SHA-256 in `filter-tool-provenance.json`. Its 210,911 bytes
hash to `67447413e273fc76809289111748870b6f6072f08b17efe94863a92d810b7d94`.
The initial missing executable is resolved without a project dependency change.

The independent `filter-candidate/` clone started at the exact frozen OID from
`pre-rewrite.bundle`. Filtering changed only its `refs/heads/main` to
`d7ae8a6b2bb39df39cd3d0ec7e8558b1fbdda36b`, with fresh-clone safety enabled.
The recorded invocation uses `--replace-refs delete-no-add`,
`--prune-empty never`, `--prune-degenerate never`, `--preserve-commit-hashes`,
`--preserve-commit-encoding` and an exact path/blob callback. Matching raw
versions become deletions, so a previous unrelated version cannot reappear.
An independent three-commit fixture verified both directions of path reuse,
an alias, executable-mode preservation, empty commits and raw absence.

Maps were copied immediately to `filter-maps/`, outside the disposable clone.
The [committed map](../historical-benchmark-commit-map.txt) has 1,179 entries:
757 unchanged IDs and 422 changed IDs. The final summary commit is separate.

| Export | SHA-256 |
| --- | --- |
| `commit-map` | `d285371678976801b000dcaa7ae914b31a3fd6524b3d166dd5775b1d10ec4253` |
| `ref-map` | `1fced551d6037d4e1e1bc70f284e497621d0d293acb7d8fb975920d0f5dd3613` |
| `changed-refs` | `599bbcd7b7f94b50d9b83318ba0dd4b8e1ba9e39d1d3ee73d1fbbd70496d0f93` |

Before any restoration, `verify-history` with that map passed for every original
commit: unrelated paths, bytes and modes; original messages, author/committer
identities and dates; and exact mapped parents. All 97 raw blobs are absent
from rewritten-main ancestry. All 40 merges and three newly empty single-parent
commits remain. The 405 original signed commits lose their signatures through
fast-export/import; signatures are not regenerated. `git fsck --full --strict`
passed. Detailed signature/empty-commit lists and check receipts remain local.

Advisor consultation 3 requires an In Progress review candidate first. Task 4
remains unchecked until fresh final review. Only afterward may the disposable
candidate stage Complete metadata and checked rollups, pass closure doc gates
and fresh-clone verification, and be frozen as the approval candidate. The
source worktree stays In Progress. At that checkpoint final review and closure
were pending; the later review/remediation is recorded below. Fresh lease,
expanded cutover commands and explicit authorization remain pending, with no
cutover-readiness claim.

The In Progress review snapshot is `74cf3cd50af5b127039db03da5ad1efdcbad8c6f`
in `<package>/filter-candidate`; its independent `review-fresh-clone` has the
same OID. This source-worktree follow-up records its results without changing
that immutable review snapshot. The 1,168-file tree equals the frozen tip minus
exact raw pairs plus 104 exported migration files and one preserved deletion.
Both clones pass all 93 summary hashes, manifest contents, committed map, all
54 series references, raw ancestry absence and no unexpected full report;
the existing synthetic test fixture is the sole allowed full envelope.
All 97 raw objects are also absent from the fresh clone's object database.
Candidate `make check-docs`, `make roadmap-check`, fresh-clone `fsck`, and
bundle verification each exit 0. Structural receipts are
`review-candidate-verification.json` and `review-fresh-clone-verification.json`;
they do not stand in for command transcripts. Command-specific closure receipts
will be created only for observed commands, with their exact revision and exit.

The single-main review bundle is 10,740,608 bytes, SHA-256
`9461ea932e066f6348e79ed790dcab9eebec2f5ad1a4311ee361b4897db517d8`.
It is 262,961 bytes larger than the pre-rewrite bundle; raw-unreachability and
uncompressed evidence-size reduction do not imply packed Git-size reduction.
No packing/GC optimization was attempted. The review commit is unsigned.

The operator verifier's initial byte-equality assertion exposed only indentation
differences in four manifest disposition entries. Parsed manifests match exactly;
the verifier now checks those contents and separately enforces the exact exported
tip bytes. No historical data, summary, or production code changed. Self-review
kept the callback bounded to saved pairs, copies restricted to explicit source
paths, and scans batched; it reuses the existing package verifier. Final-review
remediation and closure remain pending, with task 4 unchecked.

## Single post-review remediation

The fresh final review reported P1: 0, P2: 1, P3: 1. Its required P2 correction
adds the historical root-level `epoch_baseline`/`closed` index shape to the
existing series traversal, without new dependencies or a separate parser.
The focused deleted-baseline/closed-reference fixture failed with an empty
reference list before the three-line correction; it and all ten history
fixtures pass afterward. Self-review confirmed that nested pointers and
classification/grouping remain unchanged and that the same traversal handles
both formats.

Regeneration adds exactly 14 reference fields from four earlier index blobs:
1,011 references from 111 source blobs. All previous 997 references match
exactly. All 1,009 introduction-time target occurrences resolve to 54 supported,
preserved target blobs; the same two initially absent targets have their later
supported raw versions preserved. `reference-completeness.json` records those
proofs. The reference listing is 642,844 bytes, SHA-256
`be958b104d395883e076417107196a10155a53039789de01094f569fb7e457bf`.
Commit/tree/object/occurrence/filter-input hashes, classifications, 97 logical
reports, selected versions, conversion results, 93 summary byte streams and four
raw-only dispositions are unchanged. No conversion, comparison measurement,
benchmark or mutation run was repeated.

The P3 correction names the two existing structural receipts above instead of
the nonexistent consolidated filename. New command-specific receipts are
written only after their commands run and identify the exact candidate revision
and exit. The original reviewed candidate and bundles remain unchanged; the
versioned replacement candidate and evidence are under the supplementary
package. Source status remains In Progress. Final implementation commit, its
full `make check`, closure staging, fresh lease and authorization remain pending.
