# History Sanitation and Publication Handoff — Companion Implementation Plan

**Parent plan:** [Public Repository Readiness](master_plan.md)
**Goal:** Produce a recoverable, exhaustively verified publication history and a safe GitHub cutover checklist without mutating the remote repository.
**Goal IDs:** GP-02, GP-03
**Scope:** Preserve all legacy refs in a private recovery bundle; rewrite exactly one local source branch into the allowlisted publication ref `refs/heads/main`; remove historical `artifacts/` paths and map the audited historical work-domain address; verify every surviving commit and the complete publication refset; remove the legacy checkout's remote; and generate an unexecuted remote ref/Actions retirement manifest. No push, remote deletion, visibility change, or GitHub-setting mutation is authorized.
**Docs Impact:** Add history and remote-cutover gates to `docs/PUBLICATION_CHECKLIST.md`; record only redacted counts, hashes, versions, and outcomes in committed documentation. Store address maps, full ref inventories, bundles, and unsanitized evidence only in an approved private directory outside the repository.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: publish only the intended project history and source boundary, excluding generated artifacts and all legacy remote refs rather than presenting stale implementation branches as current architecture.
- `GP-03`: make the destructive rewrite recoverable and mechanically auditable with pinned tools, explicit refspecs, exhaustive commit mapping, a restored-bundle drill, and blocking remote-cutover checks.

## Approach Decision

Use a separate sanitized publication workspace and an allowlist, not an in-place rewrite or mirror push. The source is the final verified local `refs/heads/codex/public-repository-readiness`; the sole output ref is `refs/heads/main`; the publication refset contains no tags. Local tag inventory is currently empty. Before execution, retry the authenticated remote inventory because the planning environment could not authenticate to GitHub; any additional remote branch, tag, pull-request ref, or other advertised/hidden legacy ref is retirement work, not an input to the publication rewrite.

Create the sanitized workspace with an explicit single-ref fetch and `--no-tags`, then detach it from the legacy repository before filtering. Run checksum-verified `git-filter-repo` v2.47.0 with two reviewed inputs: an inverted `artifacts/` path rule and a private address mapping covering author, committer, and annotated-tag tagger metadata. Because the publication refset has no tags, any tag appearing afterward is a hard failure. Use checksum-verified gitleaks 8.30.1 with a hashed effective configuration for every pre/post scan.

The remote cutover is a separate future operation requiring explicit authorization. The handoff manifest must describe how to force-update `main`, retire every non-allowlisted remote branch/tag/ref, remove or audit Actions runs/logs/artifacts, inspect pull-request/hidden refs, and verify the final GitHub inventory. If GitHub retains a legacy ref that the owner cannot delete, visibility remains blocked pending GitHub Support or publication through a fresh repository.

## Boundary Impact

- **Git object boundary:** rescue every object reachable only through pseudorefs, reflogs, detached heads, or unreachable-object inventory before bundling. Legacy objects then remain only in the private recovery bundle and legacy checkout. The sanitized workspace contains symbolic `HEAD` pointing to one branch ref and no tags, stash refs, replace refs, notes, remote-tracking refs, auxiliary pseudorefs, reflog-only objects, or unreachable objects.
- **Working-tree content:** the final sanitized `main` tree must equal the already-clean source HEAD tree exactly. Earlier expected trees equal their source trees with only `artifacts/` entries removed.
- **Metadata:** author, committer, and annotated-tag tagger email mapping is the only permitted identity change; names, author/committer timestamps, messages, and translated parent topology remain exact for surviving commits.
- **Remote state:** unchanged by implementation. The plan produces a checked retirement manifest and blocks visibility until a separately authorized cutover proves the GitHub-side ref and Actions inventory clean.
- **Sensitive evidence:** recovery material and literal address maps remain outside the repository. Committed docs contain no legacy address or unsanitized ref contents.

## Existing Boundary Recheck

| Area | Decision | Rationale |
| --- | --- | --- |
| final source branch | keep | Treat the fully verified `codex/public-repository-readiness` tip as the sole rewrite input. |
| sanitized `refs/heads/main` | change | Create as the only publication ref and future default branch. |
| local/remote tags | change | Publication allowlist contains zero tags; inventory and retire any remote tags rather than fetching or rewriting them. |
| legacy branches, stashes, detached heads, pseudorefs, and reflogs | keep | Inventory all uniquely reachable objects, create explicit rescue refs, and preserve them through a verified/restored private bundle; exclude them from publication. |
| historical `artifacts/` trees | change | Remove only generated `artifacts/` entries and enumerate any commits pruned because they become empty. |
| audited historical address metadata | change | Map across author, committer, and tagger fields using a private reviewed input. |
| GitHub refs and Actions history | keep | Do not mutate in this feature; inventory them and generate a blocking, separately authorized retirement manifest. |
| unrelated untracked draft | keep | Hash and back up privately before history work, then prove the source file remains unchanged. |

## Open Questions

| Question | Decision | Owner | Status |
| --- | --- | --- | --- |
| Which refs are allowed in the publication workspace? | Exactly `refs/heads/main`; no tags or auxiliary refs. | user+agent | resolved |
| Should legacy local branches and stashes be rewritten? | No. Preserve them in the private recovery bundle and exclude them from publication. | user+agent | resolved |
| Should the sanitized history be imported back into the legacy checkout? | No. The sanitized sibling workspace is the only future publication checkout. | user+agent | resolved |
| Does this plan push or delete remote refs? | No. It creates an exact manifest; remote mutation needs separate explicit authorization. | user+agent | resolved |
| What if GitHub retains a non-deletable hidden or pull-request ref? | Block visibility and use GitHub Support, or publish through a fresh repository. | user+agent | resolved |
| Are any tags preserved? | No. Local inventory is empty and the publication allowlist has zero tags; any desired tag requires explicit owner approval and a plan revision. | user+agent | resolved |

## Required Skills

- Use `research-first-planning` for current GitHub and tool-specific rewrite/cutover procedures.
- Use Sol Medium for history architecture, risk review, and integrity review; use Terra High for implementation and implementation-focused verification scripts.
- Use the repository's feature workflow and recursive reviewer gates. Gemini is prohibited.

## TDD Policy

Build the audit as deterministic scripts/fixtures before executing the rewrite:

1. Create a disposable synthetic Git repository containing normal commits, an `artifacts/`-only commit, mixed author/committer metadata, a tag, a stash, divergent refs, reflog-only commits, unreachable objects, `FETCH_HEAD`, and `ORIG_HEAD`.
2. First prove the verification harness fails for an unexpected ref/tag/pseudoref, retained reflog or unreachable object, retained artifact path, unmapped identity field, altered message/time/name, incorrect translated parent, wrong expected tree, or missing pruned-commit explanation.
3. Implement the rewrite/audit workflow until the fixture yields exactly one `main` ref, zero tags, exhaustive source-to-output mapping, and a successful recovery-bundle restore.
4. Run the same scripts against the real repositories with literal sensitive inputs supplied only from the approved private directory.

The production source checkout and remote are read-only until the final local rewrite step; the GitHub remote remains read-only throughout this feature.

## Code Review Policy

Review rewrite commands, explicit refspecs, private-input handling, comparison schema, recovery drill, and remote-retirement manifest as security-sensitive code. Fix every finding, re-run the synthetic fixture and all affected real audits, and repeat until clean. A sampled commit comparison is never sufficient.

## Commit Policy

- Commit rewrite/audit scripts and redacted documentation before running them against real history.
- Never commit bundles, literal legacy addresses, private ref inventories, unsanitized logs, or generated address-map files.
- Do not commit from the legacy checkout after its remote is removed. The final sanitized workspace becomes the continuation point.
- No force push, ref deletion, Actions deletion, or visibility mutation is part of these commits.

## Implementation Steps

- [ ] Step 1: Freeze and inventory inputs. Require a clean reviewed source branch; record its tip/tree/count and every local ref, pseudoref (`HEAD`, `FETCH_HEAD`, `ORIG_HEAD`, merge/cherry-pick/rebase heads when present), reflog entry, worktree, detached head, stash, replace/note ref, local tag, `git fsck --unreachable` object, remote-advertised ref, GitHub Actions run/artifact, tool version, and relevant configuration. Record the authenticated remote-inventory failure as a publication blocker if access is still unavailable. Hash and privately back up the unrelated draft without adding it to Git.
- [ ] Step 2: Make recovery concrete. Create collision-safe rescue refs for each detached head, stash/reflog entry, pseudoref target, and otherwise uniquely reachable object that must be retained; create a timestamped `git bundle --all` in an approved private directory; run `git bundle verify`; restore it into a temporary clone; and prove every recorded rescue/local object is recoverable. Record only redacted hashes/counts in project docs.
- [ ] Step 3: Build and test the rewrite/audit harness and remote-retirement manifest. Pin git-filter-repo 2.47.0 and gitleaks 8.30.1, verify official release checksums, hash the effective gitleaks configuration, and make all synthetic failure fixtures pass. The audit schema must map every surviving source commit to one output commit; compare filtered expected trees, names, timestamps, messages, and translated parents; enumerate pruned empty commits; and assert the exact ref allowlist. Generate the exact but unexecuted commands/API checks for the separately authorized remote cutover: force-update sanitized `main`; delete or sanitize every non-allowlisted branch/tag/ref; audit/remove Actions runs/logs/artifacts; inspect advertised plus pull-request/hidden refs; require GitHub Support or a fresh public repository for retained legacy refs; and re-run remote gitleaks/inventory checks before visibility.
- [ ] Review Gate: Interim security/code review — dispatch a Terra High reviewer on Steps 1–3 scripts and fixtures. Fix all findings, re-run synthetic recovery/rewrite/audit tests, and repeat until clean.
- [ ] Review Gate: Interim history architecture review — dispatch a Sol Medium reviewer on recovery completeness, ref isolation, sensitive-data handling, tool provenance, exhaustive comparison design, and the remote-retirement manifest. Fix findings and repeat until clean.
- [ ] Step 4: Commit the reviewed rewrite/audit scripts, synthetic fixtures, redacted documentation, and unexecuted remote-retirement manifest before running any command against real history. Re-run their tests from the committed tree and require an unchanged diff.
- [ ] Step 5: Create the isolated publication workspace. In an approved sibling directory, initialize a fresh repository; explicitly set symbolic `HEAD` to `refs/heads/main`; fetch `refs/heads/codex/public-repository-readiness` through an explicit `--no-tags` single-ref refspec directly to local `refs/heads/main`; remove the source remote; remove fetch/filter-created auxiliary pseudorefs such as `FETCH_HEAD` and `ORIG_HEAD`; expire every reflog; prune unreachable objects; and run the checksum-verified filter with private path/address inputs. Repeat pseudoref/reflog expiration and pruning after filtering.
- [ ] Step 6: Verify history exhaustively. Apply the commit-map schema to every source/output commit; require symbolic `HEAD` to target `refs/heads/main`, the same final HEAD tree, expected artifact-filtered earlier trees, exact allowed metadata/topology, an explicit empty-prune list, exactly one branch and zero tags/auxiliary refs/pseudorefs, empty reflogs, no `git fsck --unreachable` output, and an all-reachable-object audit using Git object plumbing. Require no historical `artifacts/` path, no audited address in any reachable object/metadata field, a clean pinned gitleaks scan over `--all`, and all repository completion gates.
- [ ] Step 7: Separate legacy from publication. Remove or rename `origin` in the legacy repository, prove it has no push target, configure the GitHub origin only in the sanitized workspace, and verify no sanitized object/ref was imported into the legacy checkout. Do not push.
- [ ] Review Gate: Code review — dispatch a Terra High reviewer on the complete implementation/evidence diff and sanitized workspace state. Fix all findings, re-run every affected synthetic and production verification, and repeat until clean.
- [ ] Review Gate: Architecture & integrity review — dispatch a Sol Medium reviewer to independently verify recovery of pseudoref/reflog/unreachable source objects, bundle restore, symbolic HEAD, exact ref and object allowlists, empty sanitized reflogs/unreachable set, exhaustive commit/tree/metadata/topology map, pruned list, secret scan, legacy isolation, and source-draft hash. Fix findings and repeat until clean.
- [ ] Completion gate — prove synthetic harnesses, recovery of uniquely reachable legacy objects, bundle restore, symbolic HEAD, exact publication refset, empty reflog/unreachable object set, all-reachable-object audit, exhaustive history comparison, pinned/configured gitleaks, full repository gates, legacy no-push state, redacted evidence, and a complete but unexecuted remote-retirement manifest all pass.

**Review cycles:** 12
