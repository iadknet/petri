# Roadmap Execution Goal

Use this complete prompt for a roadmap-owned Codex goal. Replace the values in
the first section, preserve the contract below, and keep every document and
status truthful.

## Goal inputs

- Repository: `<local repository path>`
- Roadmap: `docs/roadmap.md`
- Requested outcome: `<outcome>`

## Outcome and traversal

Traverse the track roadmaps linked from `docs/roadmap.md`, in dependency order.
Execute only unchecked `TNN.FNN` features whose feature dependencies are
checked. Do not invent ecosystem content or bypass a dependency. Work
sequentially by default; parallel work requires an explicit reason and remains
isolated per feature. Continue until the roadmap reaches `roadmap/complete`, or
stop with a concrete blocker and request user direction.

## Roles and bounded review budget

- Use a bounded planning pass with a `gpt-5.6-sol` xhigh planner.
- Use an independent `gpt-5.6-sol` high reviewer for readiness and an
  independent `gpt-5.6-sol` high reviewer for the final diff review.
- Keep one persistent `gpt-5.6-luna` high implementer through implementation
  and any remediation; do not replace the implementer between passes.
- Only a P1 finding blocks progress. P2 and P3 findings are advisory and must
  be recorded without expanding the scope.
- Allow one readiness revision and one post-review remediation pass. Return any
  failed required verification to the same implementer until it is corrected or
  the implementer is genuinely blocked; do not silently waive a required
  check.

## Branch, worktree, and integration contract

Create or resume the integration branch `roadmap/complete` and a dedicated
integration worktree as the durable roadmap execution goal. For each feature,
create or resume exactly one feature branch and one feature worktree based on
the latest commit of the integration branch. Keep feature work isolated and
integrate completed feature commits into the integration branch in dependency
order, sequentially by default. Verify the feature worktree is clean after
integration, then clean and remove it. This prompt explicitly authorizes local
branch, worktree, and commit creation or modification within the repository
scope. Do not push, open or update a pull request, or merge into the user's `main` branch.

## Implementation and verification contract

Read the owning track and flat feature spec before implementation. Make atomic,
truthful spec/feature/track/master status/date/commit updates. Update the spec
and roadmap atomically and truthfully: feature status, spec status, track
rollup, master rollup, dates, and commit references must describe what actually
exists. Create a flat spec just in time when a feature is planned, and keep an
unchecked feature unchecked until its complete spec exists.

Run focused spec verification while implementing, including the viability-first
check where defaults, founders, or tick-loop mechanics are applicable. Record
the exact commands and results in the spec. Run the independent readiness and
final diff reviews within the stated budgets, then run post-review `make check`
before closure. A failed required verification goes back to the same persistent
implementer until corrected or genuinely blocked.

## Closure and final report

Close a feature only when its implementation, focused verification, review,
documentation, dependency, and checkbox gates pass. Update the owning track and
master only in the same truthful state transition; do not mark a rollup
complete early. On the clean integration branch, run the final `make check`.
Close the goal as `roadmap/complete` only when all tracks, features, final
success criteria, and post-review checks are complete.

Report the changed files, exact verification commands and results, review
findings and remediation, blockers, branch/worktree/commit state, integration
state, and whether `roadmap/complete` was reached. Preserve user changes and
stop for user direction on unresolved P1 findings, missing dependencies, an
unauthorized local mutation, or a material scope change.
