# Roadmaps and Feature Specs

Roadmaps are the durable execution queue for multi-track work. `docs/roadmap.md`
is an optional program rollup; each live track is a separate `tNN-<kebab-slug>.md`
file under this directory. Executable features belong to exactly one track and
use `TNN.FNN` IDs. Create a flat feature spec just in time during feature
planning at `docs/specs/roadmap/tNN-fNN-<kebab-slug>.md`.

That flat feature template is the active PRD equivalent. Its Telemetry section
requires new or materially revised specs to identify needed applied observations
and their collection cost, or explain why no telemetry change is needed. Reuse
existing sources and adopt T21 delivery when available; this adds no dependency
on unfinished observability work and requires no completed-spec backfill.

The files beginning with `_` are templates and never represent live execution
state. Copy a template, replace its placeholders, and keep IDs, links,
dependencies, statuses, and checkbox state truthful. Feature dependencies live
in the owning track roadmap rather than being duplicated in the feature spec.

Every feature row carries one indented `Goal:` line stated in world terms, what
the world or a creature can do afterward. A mechanism feature's goal line names
its natural analog first; the master roadmap's natural-analog rule is what the
reviewer checks it against. Anything longer than a sentence belongs in the
feature spec, written just in time.

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

Feature specs that introduce environmental pressures must include their
integration into all three standard goal environments and applied-behavior
verification under the [shared baseline contract](../workflow.md#environmental-pressures-in-the-standard-baseline).
This remains part of the owning feature, not a separate roadmap item.
