# Roadmap Execution Goal

Use this complete prompt for a roadmap-owned Codex goal. Replace the values in
the first section, preserve the contract below, and keep every document and
status truthful.

## Goal inputs

- Repository: `<local repository path>`
- Roadmap: `docs/roadmap.md`
- Requested outcome: `<outcome>`
- Durable experiment artifact root: `<absolute repository-external path>`
- Campaign resource envelope: `<per-run and aggregate wall-clock, concurrency/CPU, and storage caps>`

## Outcome and traversal

Load every track roadmap linked from `docs/roadmap.md`, construct one global
feature DAG, and repeatedly select a ready node. Track order and track-level
dependencies are organizational summaries, never phase boundaries. Execute
only unchecked `TNN.FNN` features whose exact feature dependencies are checked.
Do not invent ecosystem content or bypass a dependency. Work sequentially by
default; parallel work requires an explicit reason and remains isolated per
feature. Continue until the roadmap reaches `roadmap/complete`, or stop with a
concrete blocker and request user direction.

## Roles and bounded review budget

- Use a bounded planning pass with a `gpt-5.6-sol` xhigh planner.
- Use an independent `gpt-5.6-sol` high reviewer for readiness and an
  independent `gpt-5.6-sol` high reviewer for the final diff review.
- Keep one persistent `gpt-5.6-luna` high implementer through implementation
  and any remediation; do not replace the implementer between passes.
- Only a P1 finding blocks progress. P2 and P3 findings are advisory and must
  be recorded without expanding the scope after a feature is executable. During
  readiness, classify P2 findings as `non-executable` when a missing dependency,
  decomposition boundary, acceptance criterion, or verification method prevents
  safe implementation; all other P2 findings are `deferrable`. A non-executable
  P2 yields `Revision Required`, not approval, but does not stop unrelated ready
  nodes or become a P1 merely to bypass this contract.
- The budget is per feature. Allow one readiness revision and one post-review
  remediation pass for each feature. Return any failed required verification
  to the same implementer until it is corrected or the implementer is genuinely
  blocked; do not silently waive a required check. If the bounded readiness
  revision cannot make a feature executable, mark it `Blocked` with the exact
  unresolved condition and continue only with unrelated ready nodes.

## Branch, worktree, and integration contract

Create or resume the integration branch `roadmap/complete` and a dedicated
integration worktree as the durable roadmap execution goal. For each feature,
create or resume exactly one feature branch and one feature worktree based on
the latest commit of the integration branch. Keep feature work isolated and
integrate completed feature commits into the integration branch in dependency
order, sequentially by default. Verify the feature worktree is clean after
integration, then clean and remove it. Feature-worktree cleanup must never
remove or garbage-collect the declared durable experiment artifact root.
This prompt explicitly authorizes local
branch, worktree, and commit creation or modification within the repository
scope. Do not push, open or update a pull request, or merge into the user's `main` branch.

## Implementation and verification contract

Read the owning track and create or update its flat feature spec from the
canonical template before implementation. Copy the roadmap row's exact
dependencies into spec metadata, put decision-relevant source evidence in
`Inputs and Invariants`, and record the independent reviewer and review date.
Implementation may begin only when the spec says `Execution approval: Approved`; a
`Revision Required` spec remains `Planned`. Make atomic, truthful
spec/feature/track/master status/date/commit updates. Update the spec and
roadmap atomically and truthfully: feature status, spec status, track rollup,
master rollup, dates, and commit references must describe what actually exists.
Create a flat spec just in time when a feature is planned, and keep an unchecked
feature unchecked until its complete spec exists.

Run focused spec verification while implementing, including the viability-first
check where defaults, founders, or tick-loop mechanics are applicable. Record
the exact commands and results in the spec. Run the independent readiness and
final diff reviews within the stated budgets, then run post-review `make check`
before closure. A failed required verification goes back to the same persistent
implementer until corrected or genuinely blocked.

Classify every experiment manifest as fixture, characterization, or
confirmatory. Do not use fixture or characterization results as confirmatory
evidence. Before any confirmatory campaign, require checked dependency
`T01.F09`, verify its protocol and analysis hashes without inspecting
confirmatory outcomes, and validate the durable artifact root and resource
envelope. Stop and request user authorization before exceeding a declared
wall-clock, concurrency/CPU, or storage cap.

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
unauthorized local mutation, a missing or unsafe durable artifact root, a
resource-envelope increase, or a material scope change.
