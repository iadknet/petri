# Large file cleanup — Phase B readings (history rewrite)

Spec: [large-file-cleanup-2026-09-24](../../specs/large-file-cleanup-2026-09-24.md),
Phase B and Decision 2. This commit is the proposed post-cutover state. It does
not show that a cutover took place. Proof of cutover lives in an external
receipt under `.bench-artifacts/large-file-rewrite/`, not in Git. No simulation,
benchmark or mutation run belongs to Phase B.

| Identity | OID |
| --- | --- |
| Frozen source `F` (local `main` after Phase A) | `712695ac9c461bb7d8171efa844a854eb105857b` |
| Filtered tip `R` | `3802efe69537870be8da07d94c352cf1e265b059` |
| `F` and `R` tree | `2db5bb02a374bed79c08f7628cb54d1504f78af0` |

Package (ignored, local):
`/Users/istefanek/projects/petri/.bench-artifacts/large-file-rewrite/2026-09-25/`.
Blob copies:
`/Users/istefanek/projects/petri/.bench-artifacts/large-file-rewrite/blobs/<git-blob-id>`.
That directory sits beside the package, where `inventory-blobs` writes. The
spec's `historical/blobs/` holds only the T15.F02 corpus.

## Inventory

`large-blobs` finds blobs over 1 MiB at `docs/progress/features/` or
`docs/strategy/` in the ancestry of `F`, excluding tip blobs. `inventory-blobs`
records every path, commit, byte count and SHA-256 for each one. The
[manifest](../large-file-rewrite-manifest.json) holds the full list.

| Measure | Value |
| --- | ---: |
| Commits / merges in `F` ancestry | 1,490 / 40 |
| Unique root trees / blobs / blob–path associations | 1,474 / 9,225 / 10,191 |
| Inventoried blobs / bytes | 46 / 232,761,030 |
| `docs/progress/features/` (1 of them in `historical/`) | 40 / 215,967,161 |
| `docs/strategy/` (the six Phase A relocations) | 6 / 16,793,869 |
| Filter inputs (path/blob pairs; each blob has one path) | 46 |
| Largest / smallest inventoried blob | 8,087,659 / 1,178,226 |

For every copy, `git hash-object --no-filters` equals its file name, and
`historical-benchmarks.mjs verify` passes (46 raws).

## Commands

Run from the main checkout at `F`. `<pkg>` is the package path above. `<t15>`
is `.bench-artifacts/historical/t15-f02-20260913-complete`.

```sh
node scripts/historical-benchmarks.mjs large-blobs 712695ac9c461bb7d8171efa844a854eb105857b docs/progress/features/ docs/strategy/ > <pkg>/large-blobs.json
node scripts/historical-benchmarks.mjs inventory-blobs 712695ac9c461bb7d8171efa844a854eb105857b <pkg> <pkg>/large-blobs.json
git bundle create <pkg>/pre-rewrite-main.bundle refs/heads/main
git bundle create <pkg>/other-refs.bundle --stdin < <pkg>/other-reaching-refs.txt
git clone --no-local --single-branch --no-tags --branch main /Users/istefanek/projects/petri <pkg>/filter-candidate
cd <pkg>/filter-candidate && PETRI_FILTER_INPUTS=<pkg>/filter-inputs.json python3 <t15>/git-filter-repo-v2.47.0 \
  --refs refs/heads/main --replace-refs delete-no-add --prune-empty never --prune-degenerate never \
  --preserve-commit-hashes --preserve-commit-encoding --file-info-callback <pkg>/filter-callback.py
cp .git/filter-repo/commit-map .git/filter-repo/ref-map .git/filter-repo/changed-refs <pkg>/filter-maps/
git clone --no-local --single-branch --no-tags --branch main <pkg>/filter-candidate <pkg>/r-fresh-clone
node scripts/historical-benchmarks.mjs verify-history <pkg> <pkg>/r-fresh-clone main <pkg>/filter-maps/commit-map
```

The filter keeps fresh-clone safety (no `--force`). The callback is T15.F02's,
byte for byte. It turns each saved path/blob pair into a deletion, so an
unrelated version at a reused path survives.

| File | SHA-256 |
| --- | --- |
| `git-filter-repo-v2.47.0` (210,911 bytes, `--version` `a40bce548d2c`) | `67447413e273fc76809289111748870b6f6072f08b17efe94863a92d810b7d94` |
| `filter-callback.py` | `f0d284e490424219ac03cde29b89ed893a44ffd69da3fa55ffa42166f1546013` |
| `filter-inputs.json` (7,227 bytes) | `a573af2a97ce826c25cff5a79b027421b1e993a203a6fda9eacc8379a4ce1048` |
| `census.json` (249,900,054 bytes) | `bcef72b3d8bf5932041316a693d1ea87b5c6456e300c5d173647b09ffe539b4f` |
| `filter-maps/commit-map` (committed as [the map](../large-file-rewrite-commit-map.txt)) | `d9d044036c3e829532a1432b466816cd2fb52845145d46281f698cb78008f23a` |
| `filter-maps/ref-map` | `bdf8a1bb99787a73bd601998f6b88220cab8556be2080cdd3ef5bd24125b3ce2` |
| `filter-maps/changed-refs` (`refs/heads/main` only) | `599bbcd7b7f94b50d9b83318ba0dd4b8e1ba9e39d1d3ee73d1fbbd70496d0f93` |
| `pre-rewrite-main.bundle` (22,223,201 bytes) | `41462deeca58ee36db8145512584daaa0441b2a419573cd5cf79f99ccc817f6b` |
| `other-refs.bundle` (23,525,956 bytes, 31 refs) | `7b43d4da9d0634ebaa60dfc5ebbc0083362c99e7c7006b2163056209de3f5752` |

## Checks

| Check | Result |
| --- | --- |
| `git bundle verify` on both bundles | exit 0, complete history, no prerequisites |
| Restoring both bundles into a scratch bare repo | every bundled ref's objects are present, and each reaches the same number of inventoried blobs as the audit |
| Clone of `pre-rewrite-main.bundle` (`backup-restore/`) | HEAD `F`, 1,490 commits, `fsck --full --strict` exit 0 |
| Filter | exit 0; only `refs/heads/main` changed. Of 1,490 mapped commits, 323 IDs changed and 1,167 did not. 40 merges kept, 0 newly empty commits |
| Signatures | all 308 signed commits in `F` are among the changed IDs; fast-export/import strips their signatures, and none are regenerated |
| Fresh clone of `R`: tree | `R^{tree}` = `F^{tree}` |
| Fresh clone of `R`: absence | `cat-file -e` finds none of the 46 IDs in the object database |
| `verify-history` with the commit map | exit 0. For every original commit, it checks non-inventoried paths (bytes and modes), parents, message, and author/committer fields |
| Negative control: `verify-history` on `F` | exit 1, `raw blob remains` |
| `fsck --full --strict` on the filter clone and the fresh clone | exit 0 |

Packed size, from fresh `--no-local` single-branch clones, measured with
`git count-objects -v` before and after `git gc --aggressive --prune=now`:

| Clone | As cloned | After aggressive gc (pack bytes) |
| --- | ---: | ---: |
| `F` | 21.71 MiB | 20.26 MiB (20,624,657) |
| `R` | 14.22 MiB | 13.34 MiB (13,366,442) |

## Ref audit

`ref-audit.py` checks each ref with `git rev-list --objects`. It covers 39 local
refs (21 of them `refs/codex/turn-diffs/*` tree refs, plus `refs/stash`), 16
worktree HEADs, and 15 refs advertised by `git ls-remote origin`, including PRs
1–12. No worktree has per-worktree refs. Output: `ref-audit-b1.json`
(`0e8e7322…`).

- **Advertised.** `main` (and `HEAD`) at `4b88eee713da7bec04d62d5e65129d53e304c87c`
  reaches 40 of the 46 blobs. This ref is the one being replaced. The
  `codex/streamline-lean-delivery-workflow` ref and PR heads 1–12 reach none,
  so no advertised ref blocks cutover.
- **Local.** Besides `main`, 31 refs reach inventoried blobs and are in
  `other-refs.bundle`:
  - the 21 turn-diff trees (27–43 each) and `refs/stash` (28)
  - `large-file-cleanup` (46)
  - `claude/agitated-merkle-6703f8` (46)
  - `claude/quizzical-wing-016880` (16)
  - both `scratch/*` refs (24)
  - `codex/t15-f02` (12) and `codex/t16-f02` (27)
  - `origin/HEAD` and `origin/main` (40)

  The rewrite changes none of them.
- **Detached worktree HEADs.** `3806dc91` (28) is an ancestor of `F`, so the
  main bundle holds it. `1c09c26d` (16) is the same commit as the bundled
  `claude/quizzical-wing-016880`. `c1e3b406` and `43d966c1` reach none.
- **Unpushed commits.** The remote is 55 commits behind `F`, and all 55 are
  among the changed IDs. Remote `main` maps to
  `0b1e62792c1dc3a9c3713efb017ec8cf12eec8ba`. 268 of its 1,435 commits change
  ID, starting with `67b8630b` (2026-09-13).

## Recovery

- **Old `main`.** Clone it into a new directory:
  `git clone --no-local --single-branch --no-tags --branch main <pkg>/pre-rewrite-main.bundle <dir>`
  (expect `F`). To restore other refs, use `git bundle unbundle` on
  `other-refs.bundle` in a scratch repo. Inventoried contents are in
  `blobs/<id>`.
- **Collaborators.** After a cutover, keep each old clone as evidence and start
  a fresh clone of `main`. Rebase or cherry-pick only reviewed work onto the
  new IDs, and use the commit map to translate old IDs. Never merge an old
  `main` or old feature branch into the rewritten history, because it brings
  the blobs back.
- **Remaining copies.** The rewrite leaves old objects in local refs, reflogs,
  linked worktrees, other clones, forks, and GitHub caches. It proves only that
  `main` no longer reaches the blobs. Pruning is the separately authorized B5.
