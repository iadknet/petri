# Benchmark artifacts

`make bench` and direct `v3-cli bench` write a full local report and a separate
versioned summary. Commit the summary and concise readings. Full reports,
including small gate reports, stay out of Git. Existing historical reports
remain valid comparison inputs; historical migration belongs to T15.F02.
Committed summaries are the durable comparison inputs: a full report embeds
typed config, so one written before a config key was retired no longer loads
and is regenerated rather than migrated.

| Profile | Full report in the main checkout | Summary in the calling checkout |
| --- | --- | --- |
| Gate | `.bench-artifacts/<feature>/gate.json` | `docs/progress/features/<feature>.json` |
| Goal | `.bench-artifacts/<feature>/goal.json` | `docs/progress/features/<feature>-goal.json` |
| Sweep | `.bench-artifacts/<feature>/sweep.json` | `docs/progress/features/<feature>-sweep.json` |
| Recruitment S0 (`v3-cli recruitment`, `--pilot` adds `-pilot`) | `.bench-artifacts/<feature>/recruitment-s0.json` (`recruitment-s0-pilot.json`): compact per-lineage records streamed as they complete, `incomplete: true` when a cap stopped the run; exit status 3. The byte cap is checked after each record lands, so lineages already in flight on other threads still append: the raw file may exceed `--byte-cap` by up to `threads − 1` lineage records. | `docs/progress/features/<feature>-s0.json` (`-s0-pilot.json`): `petri-recruitment-s0-summary`, per-arm estimates, ladder and classification counts, raw bytes/sha256, thread count, optional replay check |

The first entry from `git worktree list --porcelain -z` identifies the main
checkout, including when called from a linked worktree or a path containing
spaces. The tracked `/.bench-artifacts/` ignore rule protects that root after
integration. A feature introducing the rule must also verify a temporary local
ignore in main before its own baseline runs. An unavailable main checkout or
missing Git context requires explicit output paths.

```sh
make bench PROFILE=gate FEATURE=tNN-fNN-slug
make bench PROFILE=goal FEATURE=tNN-fNN-slug
make bench PROFILE=sweep FEATURE=tNN-fNN-slug \
  BENCH_ARGS="--width 128 --height 128 --founders 256 --seeds 11,22,33 --ticks 300"
make bench PROFILE=sweep OUT="/tmp/ad hoc/full.json" \
  SUMMARY_OUT="/tmp/ad hoc/summary.json" \
  BENCH_ARGS="--width 8 --height 8 --founders 2 --seeds 11 --ticks 1"
```

`OUT` / `--out` changes only the full-report path. `SUMMARY_OUT` /
`--summary-out` changes only the summary path. A labelled sweep uses the table
defaults; an unlabelled sweep needs `--out` and defaults its summary to
`<raw-stem>.summary.json` beside that output. Direct CLI profile arguments are
otherwise unchanged. Measured runs use `make bench` and its existing process
preflight; direct CLI runs do not acquire that guard.

Both outputs are checked for normalized, existing symlink and hardlink aliases
before a run. Resolution errors fail rather than bypassing this check. Explicit
raw output cannot overwrite a comparison reference. Both output
identities are excluded from reference selection, with the existing explicit
absence cause when no reference remains. A write error fails the command and
does not announce a completed pair. A completed severe run writes both artifacts
and returns CLI status 3. An outer `make` or wrapper can return a different
status, which must be recorded with its own source.

## Summary version 1

Summaries carry `kind: "petri-benchmark-summary"` and `summary_version: 1`.
They retain the existing `feature`, `deterministic`, `environment`, and
`comparison` presentation paths. The original measured revision, generation
time, counters and indicator-definition tokens are preserved. Unknown summary
versions, unsupported source versions and inconsistent duplicated comparison
metadata are errors. Neither comparison nor the
progress page opens `raw.path`.

The projection retains per-seed counters, totals, six normalized counters,
world/config/recipe identities, host/thread/timing evidence, complete stored
comparisons and available reference identities. Goal readings include
persistence, structure, lineage, memory and temporal memory, exposure, applied
world totals/fractions, founder/evolved pooled neighborhood readings and drift
checkpoint aggregates. Existing `Undefined`, missing fields and measured zero
keep their meanings. Omitted detail is listed in `omitted_details`.

Persistence retains at most 21 source checkpoints per seed: all when there are
21 or fewer, otherwise source index `floor(i * (n - 1) / 20)` for `i=0..20`.
The standard 2,000-tick profile retains every checkpoint, including the first
and last. Evolved sampled-genome rows are replaced by per-seed `mesh_summary`
totals; a missing mesh observation remains `null`, not zero.

Recruitment retains task/config/size/seed-formula identities and limitations,
actual proposal totals, outcome and opportunity counts, arm and batch estimates
including their original numerators, denominators and Wilson intervals,
compact lineage discovery/retention outcomes and the original paired-lineage
contrasts/divergence evidence. Constructed stages and starts retain task scalar
summaries and identities. Each arm also pools selected-inapplicable counts by
the recorded backend/operator and retains the unresolved-backend count; these
cross-tabs do not add to the existing opportunity totals. Full genomes, deltas, per-proposal rows, scenes,
signatures, routing/state traces and replay paths stay raw. Detail scales with
cases, fixed checkpoints, arms, batches and lineages, not proposal or genome size.

`comparison_inputs` stores the existing comparison projection separately from
presentation data. Per-case numeric inputs and wall time use round-trip decimal
strings; they are not rounded to six decimals before delta calculation.
The CLI enables Serde JSON's `float_roundtrip` support to preserve retained
floating-point readings as well as integer counters and fractional strings.
Historical full reports and summaries use the same profile, case, severity,
value-only-reading and host-matching rules. No synthetic full report is rebuilt.

`claims` selects up to 16 fixed JSON pointers, omitting unavailable locations:
creature-tick and birth totals; normalized VM work; severe verdict; the first
seed's final/minimum population; the first world's first drift checkpoint
lineage count; recruitment total/attempted proposals; the first arm's retained
discovery numerator/denominator; its first proposal's Task A score and chosen
flag; the first pair's matched-proposal count; the first constructed stage's
useful flag; and total simulation wall time. Each extract is a scalar or count
whose pointer resolves in the hashed raw JSON. No per-proposal trace is retained.

## Convert an existing full report

Create an explicit provenance JSON file. Use the converter invocation and the
time at which local raw availability is being verified. Reuse these inputs to
repeat a conversion byte for byte; they do not replace the original measurement
identity. For example:

```json
{
  "verified_at": "2026-09-13T10:00:00Z",
  "converter": {
    "executable": "/absolute/path/to/v3-cli",
    "arguments": ["bench-summarize", "--input", "/tmp/full.json", "--out", "/tmp/summary.json", "--provenance", "/tmp/provenance.json"],
    "working_directory": "/absolute/path/to/checkout"
  },
  "supplied_evidence": null
}
```

```sh
cargo run -p v3-cli -- bench-summarize --input /tmp/full.json \
  --out /tmp/summary.json --provenance /tmp/provenance.json
```

Conversion reads the stored bytes, validates the full report, projects existing
observations, hashes the exact stored bytes including whitespace, and writes the
summary. It never reruns simulation, assays, comparisons, or confidence-interval
calculations. Input/output aliases are rejected.

Writers serialize borrowed data directly to buffered files, including a final
newline and checked flush. The live run report is released before conversion;
raw-byte verification compares and hashes a fixed-size buffer rather than
allocating a second complete raw byte array. Conversion still reads the original
JSON and typed report so historical absence is not replaced by Serde defaults.

`raw` records SHA-256, byte count, resolved local path, `verified_local`
availability and verification time. This means verified on this machine at
that time; it is neither durable storage nor a download URL. Missing raw files
later do not invalidate committed measurements. Record raw and summary sizes
in the feature's readings.

New CLI runs capture executable/arguments/working directory, available dirty
state, CLI completion status, and threshold/cap provenance in raw
`measurement_evidence`, then retain it in the summary. Historical evidence
absent from raw is explicitly unknown. Inheritance is unknown unless recorded
or supplied; absence of a severe flag does not prove acceptance or inheritance.
Optional `supplied_evidence` has a required `source` and nullable `command`
(same shape as `converter`), `cli_exit`, `outer_exit`, `thresholds`, `wall_caps`,
and `inheritance`. This separately attributed evidence can record an observed
outer process status or documented inherited flag without rewriting raw facts.

Before closure, verify summary hashes against available local raw bytes, inspect
original measurement and converter identities, preserve comparison verdicts and
unknown evidence, check series paths point to summaries, and confirm no new
full report is staged.
