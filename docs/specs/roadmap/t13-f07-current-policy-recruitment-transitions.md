# T13.F07 — Current-Policy Recruitment Transitions

**Status**: Complete
**Last updated**: 2026-09-20
**Feature**: T13.F07
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

The recruitment assay records, for every new or copied mesh module, where it
stops on the ladder eligibility → local edit exposure → expression →
specialization → retention under the current production mutation policy, and
every lineage that produces no retained specialized recruit is classified as
one null: no eligibility, no edit, no expression, no benefit, or loss. A
recruit counts as specialized only if replacing its payload with its own birth
payload on the same route loses at least 1/8 on the task, so copied or
prepared computation alone never scores. The fixed T13.F06 legacy panel
(82,944 proposals) is re-read on current source inside the goal profile as a
separately versioned comparison; the S0 panel (27 arms × 64 lineages × 512
generations × 2 siblings = 1,769,472 proposals under production supply) is a
separate bounded observation with a compact raw record. Observation only:
nothing reaches creatures, no production trajectory changes.

## Non-Goals

- No founder, scheduler, cost, operator, weight, rate or selector change; no
  new production mutation operator, no learned-state fission, no F08
  candidate arm, no default switch.
- No S1 base-world qualification (T13.F10 owns it), no matched long legacy
  arm (a later prospective amendment), no new benchmark service or campaign
  runner.
- No positive recruitment floor: a trustworthy null closes the feature; an
  invalid observation (false specialization, ledger mismatch, RNG consumption
  by observation, cap breach) cannot.
- No change to T13.F06's committed summaries or readings.

## Inputs and Invariants

- Owning row: T13.F07; the track's **F07 scope** and **F07–F10
  continuation** notes and the S0 row of the
  [incremental recruitment report](../../strategy/incremental-recruitment-research-2026-09-19.md)
  ("Stages in dependency order", "Causal attribution and identity",
  "Recommended first feature") define acceptance; the
  [evidence audit](../../strategy/incremental-recruitment-research-2026-09-19.evidence.json)
  fixes the F06 numbers this feature is read against (27 arms, 82,944
  proposals, 21 zero-discovery arms, raw 6.17 KB per proposal).
  [T13.F06](t13-f06-recruitment-and-retention-qualification.md) supplies the
  selectors, starting forms, Task A/B, scenes, bypass usefulness, retention
  classes and pairing rule; [T11.F21](t11-f21-per-direction-motor-output.md)
  the direction bank the scenes' decode runs through (no form writes one).
- Code seams: `recruitment_paths::observe` (`experiment.rs`, legacy supply
  through `proposal_mutation_config`), `records::choose` (Drift always takes
  sibling 0), `RecruitmentTracker` (`neighborhood/recruitment.rs`, modules
  keyed `(lineage, node, created_depth)`, provenance by `PROVENANCE_RULE`),
  `mesh_execution::route_varies_with_input` (positions) beside
  `TaskReading.scenes[].routing` (`selected_target_id`),
  `timed_recruitment_paths` (`v3-cli/src/bench/schema.rs`) and
  `bench/artifacts.rs` for the committed summary.
- Production state at plan time: main `3806dc91`. Two default changes landed
  after the T17.F02 closure without a benchmark run — `8263a8ee` (large
  structural copy weight 25%) and `3806dc91` (age-cost grace 100, cap 200,
  max 10); both move draws and trajectories before this feature. T11.F20 is
  open, so `with_legacy_supply` exists.
- Entry point: one `v3-cli` subcommand over the extended `observe`, chosen
  over a fourth bench profile (unused world parameters) or a probe patch
  plus script (not replayable).

**Panels (fixed before measurement).**

| Panel | Where it runs | Supply | Sizes | Proposals |
| --- | --- | --- | --- | --- |
| Legacy | Goal profile, `recruitment_paths` block, unchanged position | `MutationConfig::default().with_legacy_supply()` | `Sizes::PRODUCTION` (4 × 8, 32 + 16) | 82,944 |
| S0 | `v3-cli recruitment --feature <id>` through `scripts/bench-wait`, alone on the host | `MutationConfig::default()` (per-unit draw on the child's own `genome_size()`) | 4 batches × 16 lineages, discovery 256, follow-up 256 | 1,769,472 |
| Pilot | Same command with `--pilot`, before the S0 run | as S0 | batch 0, lineages 0–1, all 27 arms, 512 generations | 55,296 |

Both panels use the nine F06 forms × Drift / Selection / CostSelection
(27 arms), the `proposal_seed` formula (collision-free to lineage 15,
generation 511), reachability and `ParentExecuted` recomputed per sibling
pair, and the same task config. Pilot lineages are a prefix of the panel and
reproduce byte-identically inside it. Shared seeds are dependent, not paired
evidence. `--threads` parallelizes arms; determinism is per lineage.

**Per-module facts added to the tracker and the assay.**

| Fact | Definition |
| --- | --- |
| Ancestry | `copy_source: Option<(NodeId, created_depth)>` (first pre-birth node whose `backend_def` matches, `ambiguous` flag when several match), `birth_payload` (the `backend_def` at creation; for a constructed start, its generation-0 payload — authored history is construction, not mutation; stored whole only for modules that reach Dispatch, as a hash otherwise), `later_copies`, `deleted_depth`. |
| Target-local exposure | Per module, counts (not first depth) of events that selected it: applied / discarded, by surface of the selecting operator — split (route target/gate/entry/splice; a new route entry naming an existing module from the birth diff is one applied split; wiring created in the module's own birth is provenance, not exposure), payload (Vm/Graph/SwapNodeBackend), other (InputRef/add/remove/copy) — plus the depth of the first of each; per lineage, `eligible_site_fraction` = cohort modules that reached `ApplicableSelection` ÷ cohort modules created (selection is the only observed applicability, so it under-reads; stated in the readings). |
| Expression | `scenes_dispatched` 0–8 per scene reading; contextual when 1–7, unconditional when 8. |
| Route variation | `route_destination_varies` beside the existing position reading, from `selected_target_id`; both in the assay checkpoints and in `mesh_execution` for the founder and evolved genomes. |
| Ancestral-payload counterfactual | `ancestral_payload_replacement(genome, module)`: the node keeps id, targets and position; only `backend_def` becomes `birth_payload`. `ancestral_loss` = score(current) − score(replaced) on the arm's task; `ending_energy_sum` of both reported beside it. |
| Destination kind (read-only) | Per cohort module at each checkpoint: `vm`, `graph_stateful` (self/forward compute references, stateful kinds, previous-output or graph-local state, plasticity or eligibility traces), `graph_pure_no_effect`, or `graph_pure_with_effect`; per lineage the count of each, so an ineligible canonical Graph→VM start is recorded explicitly. F08's predecessor, gate-slot and DAG conditions are not evaluated here. |

**Ladder and classification (per lineage, on the retained chain; proposal-level
counts reported beside it).**

| Stage / class | Rule |
| --- | --- |
| Eligibility | A cohort (New/Copy) module is statically reachable from the entry node. |
| Local edit | ≥ 1 applied event targeted a reachable cohort module. |
| Expression | A cohort module dispatched in ≥ 1 scene. |
| Specialized | Task-live; score ≥ starting score + 1; bypass loss ≥ 1; `ancestral_loss` ≥ 1; every scene correct at generation 0 is still correct. Each component is stored, so a reader sees which failed. Counts toward discovery only within the discovery horizon (256; legacy 32), the F02/report convention; a lineage that first specializes later keeps its null class and is counted `specialized_after_window` in the readings. |
| Retained | Specialized recruit present and still specialized at the first retained specialized discovery + 64 (primary); +16 and +256 reported; the legacy panel's F02 retention at bypass discovery + 16 is a separate, unchanged reading. |
| Null classes | Exclusive, first failing stage: `no_eligibility`, `no_edit`, `no_expression`, `no_benefit` (with sub-count `bypass_only`: bypass loss ≥ 1 but `ancestral_loss` 0), `loss` (sub-kinds `not_selected`, `deleted`, `despecialized`, `task_dead`, `no_longer_useful`). Legacy panel: +64 unobservable → `censored_at_64`, its +16 reading unchanged. |

**Compact raw record (S0 and pilot).** Per start, the initial genome once;
per proposal `generation, sibling, seed, chosen, live, score, discovery,
specialized, applied events (operator, target, outcome)`, `mutation_fingerprint`;
per chosen child the `GenomeDelta`; full genome, module table, battery
signature and cohort at checkpoints 0 / 256 / 512 and at the first retained
specialized discovery (tracker clone); no other full genome. The replay check
re-derives fingerprints from the initial genome, chosen chain and seeds.

**Invariants.** Mutation, runtime and simulation never depend on observation
types; every reading is computed from clones and consumes no mutation RNG
(fingerprints identical with the new readings on and off); the legacy panel
keeps its arm order, `arms/0` = `graph_blank`/Drift, `Sizes::proposals` =
82,944 = `total_proposals`, and two `observe` calls at `Sizes::TEST` are
equal; no production behavior change, so the pinned short-run identity hash
`14387572686062595774` (`baseline_worlds.rs`) stays; the S0 panel is
versioned `recruitment-transitions-s0-v1` with `supply_rule` and
`source_revision` strings; a verbatim copy always reads `ancestral_loss` 0;
the S0 command stops between lineages when `--wall-cap-secs` (7,200) or
`--byte-cap` (2 GiB on disk) is reached, writes what it has as
`incomplete: true` and exits 3, never dropping completed records.

## Implementation Tasks

- [x] Tracker: ancestry, exposure counts, `scenes_dispatched`, birth payload;
      `ancestral_payload_replacement` beside `static_successor_bypass`;
      `route_destination_varies` in `mesh_execution` and the assay.
- [x] Assay: parameterize `observe` by supply and sizes (lift the caps to
      16 / 256 / 256), multi-horizon retention (+16 / +64 / +256),
      specialization components, ladder classification, destination kind,
      per-arm and per-batch classification summaries; legacy panel
      unchanged in estimates.
- [x] Fixture readings: per qualified path and step, `ancestral_loss` and the
      specialization components; a verbatim-copy negative control per backend
      reading 0; at least one Graph and one VM constructed path whose useful
      step diverges its payload reading ≥ 1 with incumbent scenes preserved
      (F05's qualified paths where one exists, else a fixture-only path).
- [x] CLI: `v3-cli recruitment --feature <id> [--pilot] [--threads N]
      [--wall-cap-secs 7200] [--byte-cap] [--replay-check]`, raw to
      `.bench-artifacts/<id>/recruitment-s0.json` (`-pilot.json`), compact
      summary to `docs/progress/features/<id>-s0.json` (`-s0-pilot.json`);
      `--replay-check` reconstructs every pilot proposal from the record and
      compares fingerprints; one row in `docs/benchmark-artifacts.md`.
- [x] Goal summary: the legacy panel's ladder/classification block and the
      founder/evolved `route_destination_varies` reach the committed summary
      through `bench/artifacts.rs`.
- [x] Readings file `docs/progress/readings/<id>.md`: pilot projection, S0
      tables, the per-path/per-step `QualifiedPath::payload_readings()`
      table, legacy-panel comparison with F06's recorded numbers labelled
      "current-source remeasurement".

## Verification

- [x] `cargo test -p v3-core --test viability` first: 28 passed;
      `baseline_worlds` 19 passed, identity hash unchanged.
- [x] Focused tests: `cargo test -p v3-core recruitment` -> ancestry,
      exposure, `scenes_dispatched`, ancestral replacement (verbatim copy 0,
      diverged path ≥ 1), `recruitment_paths_known_specializing_lineage_classifies_retained_at_the_primary_horizon`
      (S0 arm 22 b1/l13 at 192 + 64), destination kind on canonical and
      prepared forms, RNG non-consumption with readings on/off,
      destination-versus-position route variation, and proptest invariants
      (classification exclusive and total; ancestral replacement of an
      unchanged payload is the identity).
- [x] `cargo test -p v3-cli` -> subcommand outputs, pilot prefix identity,
      cap stop marks `incomplete`, replay check, 2-thread parity.
- [x] Pilot: `scripts/bench-wait cargo run --release -p v3-cli -- recruitment
      --feature <id> --pilot --replay-check` -> per-arm wall and bytes,
      replay 100%, projection to 64 lineages in the readings file, summary at
      `docs/progress/features/<id>-s0-pilot.json`; launch only if the mean
      projection ≤ 1.6 h and 1.6 GiB (max-based bound beside it).
- [x] S0 panel: same command without `--pilot` -> summary at
      `docs/progress/features/<id>-s0.json`, raw hash/bytes/wall/thread count
      in the readings file, `incomplete` false.
- [x] `make check` -> exit 0 on tested commit `b6f8b455` (24 test binaries ok).
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.

      ```text
      387 mutants tested in 55m: 74 missed, 251 caught, 62 unviable
      run mode: fresh (diff against 3806dc91, at 922445d1 + the attempt-1 test fix)
      output: ~/.local/share/petri-tools/mutants/t13-f07/mutants.out
      missed.txt: 74 lines    timeout.txt: empty
      attempt 1 (no evidence): cargo mutants exit 4, baseline failed on
        v3-cli default_paths_follow_the_feature_and_pilot_naming (test read
        default paths from a cwd that is not a Git checkout under
        cargo-mutants); test-only fix, then the one fresh run above
      survivors: 72 killed (tests only), 2 equivalent
        (EditSurface::of deleted arm -> the wildcard maps the same operators
        to Other; lineage eligibility |= is_empty() -> every fixed start is
        eligible at generation 0), 0 deferred; full 74-row table with each
        test name in docs/progress/readings/t13-f07-current-policy-recruitment-transitions.md
        "Mutation gate"
      incremental passes 1-2 (MUTANTS_OUT=…/t13-f07-triage, MUTANTS_ITERATE=1,
        not closure evidence): 74 tested, 3 missed -> 3 tested, 2 missed (the
        two equivalents); see the readings "Mutation gate" run table
      no production, test-selection or tool-configuration change during
        triage, so no second fresh run
      ```
- [x] Benchmark summaries stored at `docs/progress/features/<id>.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation only: no natural
analog and no environmental pressure; nothing reaches a creature. Expected
simulation compute cost: none. Both profiles compare against the series
index at run time: gate epoch `remove-complementary-nutrition.json`, goal
epoch `t11-f17-executed-biased-mutation-targeting-goal.json`, goal-worlds
epoch and previous `t17-f02-unit-scale-introspection-goal.json`, all under
`docs/progress/features/`. The +10%/+50% work and +25%/+100% wall flags
stand. Trajectories: gate and goal diverge from T17.F02 at the first birth
that draws a large structural copy (`8263a8ee`) and, in the goal worlds, at
the first age-cost charge after tick 100 (`3806dc91`); that divergence is
attributed to those pre-existing default changes, not to this feature, which
cannot move a counter. A severe flag or extinction produced by that
divergence is a user decision (re-pin or not) that this feature cannot accept
on its own; the attribution check is the unchanged pinned short-run identity
hash and a diff confined to observation, CLI, tests and docs — no second goal
run. Caps: goal experiment (legacy panel with the new readings) under 120 s;
goal profile under 15 minutes; S0 panel under 2 host-hours wall and 2 GiB on
disk; pilot under 10 minutes (a pilot over 3.75 minutes already projects the
panel over its cap).

| Indicator | Predeclared direction |
| --- | --- |
| Six normalized counters, both profiles | Differ from T17.F02 by the two default changes only; the pinned identity hash is unchanged. |
| Founder neighborhood, drift walk, evolved trajectories, diversity and cognition indicators | Same attribution; no floor moved. Depth-2,000 drift is reported against the T11 track's 0.005 with no gate (withdrawn 2026-09-14). |
| Founder and evolved `mesh_execution` | Existing keys unchanged; `route_destination_varies` added and never `true` where the position reading is `false`; the founder has no duplicate-destination entries, so its two readings are equal. |
| Legacy panel, unprepared arms | Expected 0/32 proposal and retained discovery as at F06; no floor; a nonzero reading is reported with its path and classification. |
| Legacy panel, prepared arms | Discovery near F06's 17–18/32 (current-source remeasurement; the `8263a8ee` draw change means no byte-identity claim); every discovery reads `ancestral_loss` 0 (prepared payload is birth payload), so `bypass_only`, not specialized. |
| S0 unprepared arms (18 of 27) | No stochastic floor. Every lineage classified; the ladder stage counts and `eligible_site_fraction` are the result. Predeclared exposure reading: requested events per birth ≈ 0.005 × `genome_size` under the per-unit draw; applied and discarded reported beside it. |
| S0 prepared arms | Expression expected as in F06; specialization expected 0 unless a payload edit diverges the prepared module; reported. |
| S0 Drift arms | Genome size may grow without bound over 512 generations; p50/p90 `genome_size` per checkpoint reported; the pilot's projection is the cap check. |
| Negative controls | Verbatim copies 0 `ancestral_loss` in every fixture and every panel module whose payload never changed. |
| Wall/creature-tick, both profiles | No direction; recorded. |
| Committed goal summary bytes | Grows by the new keys on 27 arms and their lineage rows; recorded, no cap; shrinking stays with T15 summary-v2. S0 summary under 4 MB, per-lineage rows and no per-proposal rows. |

**Measured verdict.** Spec-owner rulings, 2026-09-20: (1) the goal deltas
(`plasticity_updates` −46.01%, stored level `ok`; Orchards minimum 149→30;
no extinction)
are the predeclared `8263a8ee`/`3806dc91` divergence under an unchanged
identity hash — recorded, not severe, no re-pin required; (2) legacy
prepared discovery 19–21/32 against F06's 17–18/32 is current-source
remeasurement under the remapped draw, not a deviation; (3) the two
`vm_copy` retained specialized lineages (arms 9 and 22, Task A, held at
+16/+64/+256) are an honest positive under the no-floor predeclaration,
1/64 retained per arm plus one lineage each specialized after the discovery
window, no F08 gate reached, paths in the readings file; the predeclaration's
"18 of 27" unprepared arms miscounted: 21 (7 forms × 3). Tables:

| Run (2026-09-20, `cdc4674b`, 8 threads, run once each, sequential, host idle) | Exit: observed outer / recorded CLI | `severe` | Verdict | Summary bytes | Raw bytes, sha256 (main `.bench-artifacts/`, local) |
| --- | --- | --- | --- | ---: | --- |
| Pilot `recruitment --pilot --replay-check` (no `--threads`) | 0 / n/a (CLI exits 3 incomplete, 1 error; neither) | n/a | replay 55,296/55,296 = 100%; `wall_secs` 6.784; mean projection 0.0603 h, 1.5045 GiB (inside 1.6 h / 1.6 GiB, launched); max-based 4.9428 GiB, wall not computable | 467,195 (`-s0-pilot.json`, by-product) | 50,483,845, `de977a6f…ffa9e3` |
| S0 `recruitment` (no `--pilot`) | 0 / n/a | n/a | `incomplete` false, 1,728 lineages, 1,769,472 proposals, `wall_secs` 162.537, 1.342 GiB of 2 GiB, summary < 4 MB | 2,405,314 | 1,441,006,562, `977ddfcc…be2e302` |
| Gate `make bench PROFILE=gate` | 0 / `{"code":0,"source":"v3-cli status on successful artifact-pair completion; output errors instead exit 1"}`, `outer_exit` null, `dirty` true (untracked summaries only) | false | all six `ok` vs epoch `remove-complementary-nutrition.json` and `t17-f02-unit-scale-introspection.json`; wall −3.35% / −17.68% `ok`; founder 42 ms of 10 s | 97,724 | 94,864, `763def13…b7244`, `verified_local` 2026-09-20T03:34:06Z |
| Goal `make bench PROFILE=goal` | 0 / same `cli_exit`, `outer_exit` null, `dirty` true | false | all six `ok` vs `t17-f02-unit-scale-introspection-goal.json` (goal-worlds epoch = latest closure; one comparison printed); `plasticity_updates` −46.01%, stored level `ok`; wall −6.68% `ok`; `wall_clock_ms_total` 460,424 of 900 s; evolved 572 ms of 180 s; founder 99 ms; recruitment 11.5 s of 120 s; no extinction | 7,632,294 | 591,945,210, `9e5c9180…f43f6`, `verified_local` 2026-09-20T03:42:22Z |

| Goal world (T17.F02 → F07) | `final_population` | `minimum_population` | `plateau_population` | `births` | `extinction_tick` | `drift_changed_per_all_births_at_2000` |
| --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland | 7,055 → 5,849 | 149 → 30 | 4,318.06 → 2,934.11 | 197,547 → 178,619 | null → null | 0.0015 → 0.001 |
| Canyon country | 3,741 → 3,506 | 806 → 1,282 | 4,177.84 → 4,670.50 | 175,258 → 188,444 | null → null | 0.0035 → 0.0005 |
| Confluence | 12,011 → 9,084 | 191 → 155 | 9,101.30 → 7,111.28 | 251,335 → 247,782 | null → null | 0.0015 → 0.001 |

| Predeclared row | Measured (readings file has the full tables) |
| --- | --- |
| Six counters | no severe flag in either profile; founder T11.F14 rows equal T17.F02 (87/123/0 gate; 96/114/0, 87/123/0, 96/114/0 goal) |
| `mesh_execution` | founder false/false everywhere; 0 of 39 raw goal blocks read destination true with position false |
| Legacy unprepared / prepared | 21 unprepared arms 0/32; prepared 19–21/32 discovery (above F06's 17–18/32), 229 discovery readings all `ancestral_loss` 0, `specialized` 0, `bypass_only` 15–21 |
| S0 unprepared / prepared / Drift | 19 of 21 unprepared arms `specialized` 0; arms 9 and 22 (`vm_copy` Selection / CostSelection) 1 `retained` each plus 1 `specialized_after_window`, arm 8 1 `loss_not_selected`; `specialized_after_window` 1 in arms 8, 9, 13, 16, 22 (0 elsewhere); prepared `specialized` 0, expression 64/64 under Drift/Selection, 43/64 and 45/64 under CostSelection (arms 24, 26); gen-512 p90 128–478 |
| Summary bytes | goal +759,239 vs T17.F02; S0 1,728 lineage rows, no proposal rows |

- Summaries: [gate](../../progress/features/t13-f07-current-policy-recruitment-transitions.json),
  [goal](../../progress/features/t13-f07-current-policy-recruitment-transitions-goal.json),
  S0 (local only; Evidence below).
- Full readings: [`docs/progress/readings/t13-f07-current-policy-recruitment-transitions.md`](../../progress/readings/t13-f07-current-policy-recruitment-transitions.md).

| Evidence (relocated 2026-09-24, local only) | SHA-256 | Bytes |
| --- | --- | ---: |
| S0 summary `.bench-artifacts/research/t13-f07-current-policy-recruitment-transitions/t13-f07-current-policy-recruitment-transitions-s0.json`; numbers in the readings | `1dab13f5d3e42d2d7e0d83d528bdc860dc0b7ee09326de353c8ac3dc41dda589` | 2,405,314 |

## Success Criteria

- [x] Every S0 and legacy-panel lineage carries exactly one ladder outcome
      (retained or one null class) with its stage facts, exposure counts,
      ancestry and route readings; counts reconcile (proposals = 1,769,472
      and 82,944; lineages × arms = classifications).
- [x] Observation consumes no mutation RNG and moves no production counter;
      the pinned short-run identity hash is unchanged.
- [x] Verbatim-copy controls read 0 `ancestral_loss`; the constructed diverged
      paths read ≥ 1 with incumbent scenes preserved; prepared fixtures
      express as expected.
- [x] Pilot replay check 100%, projection inside the caps, S0 run complete
      within 2 host-hours and 2 GiB, raw provenance recorded; or the cap
      breach is reported as incomplete and escalated, never trimmed.
- [x] Gate and goal summaries stored with the attribution recorded; the
      readings file separates historical F06 numbers from current-source
      remeasurement.

## Notes for AI Agents

- Decision: this run only, at the user's direction (2026-09-19): every role
  ran on Opus 5 instead of its Fable/Sonnet frontmatter and `/advisor` was
  skipped; not a precedent, nothing under `.claude/` changes.
- Decision: S1 base-world qualification and the matched long legacy arm are
  outside this feature; the feasibility pilot is inside it and runs before the
  S0 panel. A projection over either cap stops for a user decision.
- Decision: exposure surfaces and `eligible_site_fraction` are as the
  "Target-local exposure" row defines them (`EditSurface::of`).
- Decision: authored history is construction, not mutation: birth payloads
  are re-based on the start genome; preparation is never an ancestral loss.
- Decision: a specialized proposal never retained is `loss/not_selected`;
  `bypass_only` reads the retained chain; `ancestral_loss` is evaluated only
  past the bypass gate (`Some(0)` by identity when unchanged).
- Deferred: the goal drift walk reads identically for Orchards and
  Confluence (seed formula does not vary by world; already so at T17.F02);
  for the walk's owner, T11.F20.
- Decision: the S0 raw is single-line JSON streamed per lineage; `--byte-cap`
  sits beside `--wall-cap-secs`; an incomplete run exits 3 after writing both
  artifacts. `MeshObservation` (observation-only) carries the destination.
- Deferred: the final review's record-only findings (six-site chain facts,
  P3 cleanups, exit 3 untested, byte-cap overshoot, UTC dates) are in the
  readings file's "Deferred" section.
- Cost: `/usage` totals at closure not yet supplied (user command);
  implementer briefs 3 (advisor consults 2 / 2 / 2); spec-owner resumes after
  Plan 3; review 0 P1 / 6 P2 / 9 P3; one fresh mutation run, 72 killed,
  2 equivalent.
