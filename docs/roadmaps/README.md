# Roadmaps and Feature Specs

Roadmaps are the durable execution queue for multi-track work. `docs/roadmap.md`
is an optional program rollup; each live track is a separate `tNN-<kebab-slug>.md`
file under this directory. Executable features belong to exactly one track and
use `TNN.FNN` IDs. Create a flat feature spec just in time during feature
planning at `docs/specs/roadmap/tNN-fNN-<kebab-slug>.md`.

The files beginning with `_` are templates and never represent live execution
state. Copy a template, replace every placeholder, and preserve the canonical
metadata, headings, links, IDs, and checkbox state. Keep dependencies explicit,
acyclic, and limited to work that is actually required.

Run `make roadmap-check` while editing. A checked feature requires one complete
feature spec, and a complete spec requires every implementation, verification,
and success item to be checked. Use `Blocked` only with a concrete blocker;
blocked work remains unchecked. A complete track and master rollup must agree
with their checked criteria.

Roadmap documents describe intent and state. They do not encode model choice,
review recursion, severity policy, worktree orchestration, or integration
authority. Those decisions belong in the reusable goal prompt template.
