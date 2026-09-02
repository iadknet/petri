# Contributing

Start with `AGENTS.md`, then read the applicable roadmap track and flat feature
spec. Roadmap execution state lives in `docs/roadmap.md` when a live master is
needed, track roadmaps under `docs/roadmaps/`, and feature specs under
`docs/specs/roadmap/`. Templates begin with `_` and are never live state.

Run `make roadmap-check` while editing roadmap documents and record focused
verification in the owning feature spec. Keep dependencies, checkboxes, links,
statuses, and completion rollups truthful. Run `make check` before declaring a
feature or integration complete.

Review security-sensitive reports privately as described in `SECURITY.md`.
When adding or changing dependencies, follow the age-gate, lockfile, registry,
and exception rules in `SECURITY.md`; run `make dependency-audit` after setup.

Use concise commit summaries. The release changelog groups summaries beginning
with `feat` under Added and `fix` under Fixed; all other non-merge commits are
listed under Changed.
