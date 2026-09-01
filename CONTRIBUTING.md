# Contributing

Start with `AGENTS.md` and the applicable active PRD. Keep a change scoped to one
stage when practical, update its checkboxes only after the work is observable,
and run the stage's verification plus `make check`.

For new work, create a PRD set with:

```sh
scripts/prd-new <kebab-slug> <stage-slug> [stage-slug ...]
```

Review security-sensitive reports privately as described in `SECURITY.md`.
When adding or changing dependencies, follow the age-gate, lockfile, registry,
and exception rules in `SECURITY.md`; run `make dependency-audit` after setup.

Use concise commit summaries. The release changelog groups summaries beginning
with `feat` under Added and `fix` under Fixed; all other non-merge commits are
listed under Changed.
