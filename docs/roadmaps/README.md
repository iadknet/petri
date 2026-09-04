# Roadmaps and Feature Specs

Roadmaps are the durable execution queue for multi-track work. `docs/roadmap.md`
is an optional program rollup; each live track is a separate `tNN-<kebab-slug>.md`
file under this directory. Executable features belong to exactly one track and
use `TNN.FNN` IDs. Create a flat feature spec just in time during feature
planning at `docs/specs/roadmap/tNN-fNN-<kebab-slug>.md`.

The files beginning with `_` are templates and never represent live execution
state. Copy a template, replace its placeholders, and keep IDs, links,
dependencies, statuses, and checkbox state truthful. Feature dependencies live
in the owning track roadmap rather than being duplicated in the feature spec.

Run `make roadmap-check` while editing. It checks ownership, links, dependency
existence and cycles, feature/spec completion agreement, and completion rollups.
It intentionally does not police prose, review procedure, dates, or agent
orchestration.

Roadmap documents describe intent and state. They do not encode agent
execution policy or integration authority. Those belong in
[`docs/workflow.md`](../workflow.md), the one live execution workflow. It runs
one feature per goal, in dependency order: the target feature must have all
dependencies checked, and a goal never implements a prerequisite or an unrelated
ready feature implicitly.
