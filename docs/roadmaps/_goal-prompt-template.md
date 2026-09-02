# Single-Feature Roadmap Goal

Use this complete prompt for one roadmap feature. Replace the values in the
first section, preserve the contract below, and keep every document and status
truthful.

## Goal inputs

- Repository: `<local repository path>`
- Source ref: `<branch or commit containing the current roadmap state>`
- Roadmap: `docs/roadmap.md`
- Target feature: `<exactly one TNN.FNN ID>`
- Requested outcome: `<concise restatement of the feature outcome>`
- Durable experiment artifact root: `<absolute repository-external path, or N/A when no experiment artifacts are produced>`
- Campaign resource envelope: `<wall-clock, concurrency/CPU, and storage caps, or N/A when no experiments run>`

## Scope and dependency gate

Load the master roadmap and the owning track, then resolve the exact target
feature, title, and dependencies. This goal may implement exactly one unchecked
`TNN.FNN` feature. Do not implement another roadmap feature, an unchecked
prerequisite, or a downstream feature.

Resolve the source ref to a commit SHA and record it before creating a branch.
Read roadmap state from that commit, not from uncommitted files. If the supplied
repository checkout has uncommitted changes to the master roadmap, owning track,
or target spec, stop and ask the user to commit them or choose another source
ref.

Verify that every declared dependency exists and is checked at the resolved
source commit. If any dependency is unchecked, stop and report its feature ID as
the next prerequisite. If the target is already checked, verify its completed
spec and report that no work is needed. Stop for user direction if the requested
outcome conflicts with the roadmap feature or materially expands it.

## Roles and bounded review

- Use one bounded planning pass with a `gpt-5.6-sol` xhigh planner.
- Use an independent `gpt-5.6-sol` high reviewer for readiness and an
  independent `gpt-5.6-sol` high reviewer for the final diff review.
- Keep one persistent `gpt-5.6-luna` high implementer through implementation
  and remediation.
- Only a P1 finding blocks implementation or completion. Record P2 and P3
  findings without expanding an executable feature.
- Allow one readiness revision and one post-review remediation pass. Return
  failed required verification to the same implementer until corrected or
  genuinely blocked. If the bounded readiness revision cannot make the feature
  executable, mark its spec `Blocked` with the concrete reason and stop.

## Branch and worktree contract

Create or resume one branch named `roadmap/tNN-fNN-<slug>` and one
dedicated worktree for the target feature, based on the exact source ref. Do not
modify the source checkout or combine another feature into the branch. Resume
existing state only when it clearly belongs to the same target feature.

This prompt explicitly authorizes local branch, worktree, and commit creation
or modification for the target feature. It does not authorize pushing, opening
or updating a pull request, merging into the source ref or the user's `main`,
or deleting the feature worktree. Never delete or garbage-collect a declared
durable experiment artifact root.

## Planning and implementation contract

Create or update the target's flat feature spec from the canonical template.
Put decision-relevant source evidence, dependency outputs, observable behavior,
non-goals, implementation tasks, and focused verification in the spec. Complete
the bounded planning and readiness review before implementation; do not add
approval metadata merely to restate that process.

Implement only the target feature. Use TDD for behavior changes and preserve
repository boundaries and user changes. Keep the spec, feature checkbox, owning
track, master rollup, dates, and commits atomic and truthful. An incomplete or
blocked feature remains unchecked. Do not change unrelated feature status.

After readiness review and immediately before implementation, set the feature
spec to `In Progress`, promote a `Planned` owning track to `Active`, and promote
a `Planning` master roadmap to `Active`. On success, set the spec to `Complete`
and check the target feature. Mark the track `Complete` only when all its
features and success criteria are checked, and mark the master `Complete` only
when all tracks and final success criteria are checked. If implementation is
blocked, set the spec to `Blocked`, leave the feature unchecked, and keep any
already-active rollups active.

## Verification and experiments

Run the focused checks named by the feature spec and record their exact commands
and results. Run viability-first testing when production defaults, founders, or
tick-loop mechanics change. Run the full `make check` only when the feature
changes application or runtime source code or build configuration; use relevant
focused checks for documentation-only work. After every roadmap or feature-spec
status update, run `make roadmap-check` and `git diff --check`.

Only run an experiment when the target feature requires it. Classify its
manifest as fixture, characterization, or confirmatory. Before any experiment,
require non-`N/A` goal inputs for a concrete repository-external artifact root
and wall-clock, concurrency/CPU, and storage caps, then validate them before
launch. Fixture and characterization results are not confirmatory evidence. A
confirmatory campaign additionally requires checked dependency `T01.F09` and
verification of its protocol and analysis hashes without inspecting outcomes.
Stop for renewed user authorization before exceeding any declared cap.

## Closure and final report

Close the goal when the one target feature is either complete or genuinely
blocked. Mark it complete only after implementation, focused verification,
final review, documentation, and commit gates pass. Update its track rollup or
the master roadmap only if their own completion criteria are now satisfied; do
not imply that the larger roadmap is complete.

Report the target feature, changed files, verification results, review findings
and remediation, blocker if any, branch/worktree/commit state, and the exact
status changes. Leave the feature branch and worktree available for user
inspection.
