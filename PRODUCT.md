# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

One user: the project's owner, who builds Petri alone with AI coding agents
(Claude and Codex) and reads every surface themselves. Their recurring jobs:

- After each roadmap feature closes, find out what that feature did to the
  simulation: did cognition or evolvability move, did anything break, what did
  it cost. Cognition and evolvability matter most (owner, 2026-10-10);
  diversity is watched as context.
- Watch a live run to see how creatures actually live.
- Decide what to build next from that evidence.

No collaborators or public readers are expected for the progress page
(confirmed 2026-10-10).

## Product Purpose

Petri is an artificial-life simulation written in Rust. Creatures with evolvable
brains forage, reproduce and evolve in seeded worlds. It succeeds when a world
feels alive: many ways of making a living last side by side, and lineages keep
evolving behaviour that depends on remembering, learning and predicting, not
only reacting (`docs/roadmap.md`, Success Definition).

Progress is tracked, not proved. A small set of diversity and cognition
indicators, read at the close of every feature, should rise across closures,
with each rise attributed to the feature that moved it.

Petri is also the owner's test bed for agentic coding workflows.

## Positioning

A personal research simulation, not a product for others. What it does that
neighbouring ALife projects do not claim:

- Every closed feature carries a stored, deterministic benchmark run on three
  fixed worlds, so its effect is attributable.
- Every mechanism must have a natural analog and reach creatures through the
  world and body, never through a sensor invented for the feature.

## Operating Context

- **Roadmap workflow:** features close through `docs/workflow.md`. Each closure
  stores a benchmark summary under `docs/progress/features/`, listed in
  `docs/progress/benchmark-series.json`.
- **Progress page** (`docs/progress/index.html`): reads those summaries in the
  browser.
  - Viewed in a full desktop browser (1200px or wider), served locally or from
    the telemetry stack at `127.0.0.1:3300`.
  - Phones and narrow panes are not a priority (confirmed 2026-10-10).
- **Live web UI** (`frontend/`, React and Vite at `localhost:5173`): watches a
  running simulation through the typed v3 server API.
- **Telemetry stack** (`telemetry/`, Grafana on `127.0.0.1:3300`): run
  dashboards and the creature-windows report.
- **Goal profile:** three saved worlds, run for 2,000 ticks from 10,000
  founders when a feature's Verification Plan declares it; the long goal
  profile runs the same worlds to 40,000 ticks:
  - Orchards in grassland, seed 11;
  - Canyon country, seed 22;
  - Confluence, seed 33.

  They are different environments, not replicates.

## Capabilities and Constraints

- **Runs are deterministic.** A closure that changes no behaviour reproduces
  the previous closure's readings exactly.
- **Each world is one seeded run.** Small changes between closures can be
  chance; there are no seed replicates.
- **Cost levels are stored, not recomputed.** Comparisons carry `ok`, `flag`
  and `severe` levels. Work per creature-tick flags at +10% and is severe at
  +50%; wall clock uses +25% and +100%.
- **Measured readings:**
  - Cognition: memory sensitivity.
  - Evolvability: mutations that change behaviour (the T14.F12 neighbourhood
    read, with its floor) and drift depth (behaviour change after 2,000
    mutation-only generations).
  - Diversity: founder clades alive, lineage entropy.
  - Guardrails: population, compute counters.
- **Not measured yet**, so never to be drawn as zero: learning, prediction,
  strategy count and adaptive novelty.
- **Progress page delivery:** one static HTML file with no build step.
  - Charts come from a vendored chart library committed beside the page,
    pinned to an exact version with a recorded checksum (user decision,
    2026-10-10).
  - The repository's dependency policy applies: exact versions, a seven-day
    minimum package age, checksum verification.
  - Tracked files stay under 1 MiB unless allowlisted.
- **Page test contract:** `scripts/benchmark-artifacts.test.mjs` evaluates the
  page script up to its `// ── Boot` marker without a DOM. It needs `loadAll`,
  `meshSums`, `artifactNotice`, `num`, `get` and `mutationEffectsView`.
- **Changing the progress page** follows a short loop: edit, Codex review,
  commit to main. It needs no roadmap feature (user instruction, 2026-10-10).

## Brand Commitments

- Keep the dark mode. The owner likes the current dark styling (stated
  2026-10-10).

## Evidence on Hand

- 44 three-world closure summaries (`goal-worlds-v1`, from T12.F04) and older
  single-world and gate summaries, in `docs/progress/features/`.
- The series index, report omissions and counter-definition changes, in
  `docs/progress/benchmark-series.json`.
- The per-closure table in `docs/progress.md`.
- The design rationale and its sources, in
  `docs/strategy/progress-page-design-2026-10-10.md`.
- Screenshots: `docs/assets/petri-creatures.gif` and
  `docs/assets/petri-v3-ui.png`.
- No user research, external readers, testimonials or published results exist;
  none are to be invented.

## Product Principles

1. Track progress, do not prove it. Read cheap indicators at every closure;
   skip confirmatory machinery.
2. Attribute every move to the closure that caused it, and say plainly when a
   closure changed nothing.
3. Never present an unmeasured reading as a measured one, and never average
   different worlds into one number.
4. Evolvability, not target abilities: features should help creatures evolve
   useful behaviour on their own.
5. Keep it simple. Prefer a small, clear surface to verification machinery.

## Accessibility & Inclusion

- The owner has ADHD. Every surface leads with the answer, keeps one idea per
  line, keeps working sets small (about five items per group), and uses plain
  words beside any term of art.
- Colour never carries meaning alone: pair it with an arrow, a label or
  position.
