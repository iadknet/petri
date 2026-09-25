# T13.F07 — Current-Policy Recruitment Transitions readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f07-current-policy-recruitment-transitions.md`](../../specs/roadmap/t13-f07-current-policy-recruitment-transitions.md).

## Benchmark (roadmap-benchmark-specialist, 2026-09-20)

Commands, sequentially from the worktree at `cdc4674b` (final feature code,
tracked files clean; the four summaries below were the only untracked files
at any run's start), nothing else running (`pgrep -fl 'v3-|cargo'` before
the first run listed no process; `scripts/bench-wait` printed no
blocking-PID line in any log). Every run was wrapped by a POSIX `sh` runner
in the session scratchpad that logs the command, UTC start and end, the
observed outer exit status (`outer exit: <status>`) and the outer wall in
whole seconds; logs `pilot.log`, `s0.log`, `gate.log`, `goal.log`. No
`--threads` flag was passed (the spec's Performance section names no thread
count); the rayon global pool reported `threads` 8 in every summary. No
run was repeated.

| # | Command | Observed outer exit | Outer wall (s) | CLI-measured wall (s) | Start (UTC) |
| ---: | --- | ---: | ---: | ---: | --- |
| 1 | `scripts/bench-wait cargo run --release -p v3-cli -- recruitment --feature t13-f07-current-policy-recruitment-transitions --pilot --replay-check` | 0 | 140 (includes a 2 m 10 s release build) | 6.784 (`wall_secs`) | 2026-09-20T03:28:06Z |
| 2 | `scripts/bench-wait cargo run --release -p v3-cli -- recruitment --feature t13-f07-current-policy-recruitment-transitions` | 0 | 164 | 162.537 (`wall_secs`) | 2026-09-20T03:31:03Z |
| 3 | `make bench PROFILE=gate FEATURE=t13-f07-current-policy-recruitment-transitions` | 0 | 1 | 0.411 (`wall_clock_ms_total` 411.288) | 2026-09-20T03:34:05Z |
| 4 | `make bench PROFILE=goal FEATURE=t13-f07-current-policy-recruitment-transitions` | 0 | 494 | 460.424 (`wall_clock_ms_total` 460,423.787) | 2026-09-20T03:34:13Z |

Recorded CLI exit for runs 3 and 4 (`measurement_evidence.cli_exit`):
`{"code":0,"source":"v3-cli status on successful artifact-pair completion;
output errors instead exit 1"}`; `outer_exit` null in both summaries;
`dirty` true in both (untracked summary files from runs 1–3, tracked files
clean). Runs 1 and 2 have no `measurement_evidence` block; their exit
statuses are the observed outer statuses (the CLI exits 3 on an incomplete
run and 1 on an output error; neither occurred).

### Artifacts

| Run | Summary path (worktree) | Summary bytes | Summary sha256 | Raw path (main checkout `.bench-artifacts/`, local, not staged) | Raw bytes | Raw sha256 |
| --- | --- | ---: | --- | --- | ---: | --- |
| pilot | `docs/progress/features/t13-f07-current-policy-recruitment-transitions-s0-pilot.json` | 467,195 | `e01251f7ff6f512d63f91f03e0c67fbed3955648c8b3a107eeb806e37aa1c23c` | `.bench-artifacts/t13-f07-current-policy-recruitment-transitions/recruitment-s0-pilot.json` | 50,483,845 | `de977a6f3e612df54311f6bfb58630cc7f74732b64ff33224a8b78aa52ffa9e3` |
| S0 | `.bench-artifacts/research/t13-f07-current-policy-recruitment-transitions/t13-f07-current-policy-recruitment-transitions-s0.json` (relocated 2026-09-24, local only; see the Evidence line below) | 2,405,314 | `1dab13f5d3e42d2d7e0d83d528bdc860dc0b7ee09326de353c8ac3dc41dda589` | `.bench-artifacts/t13-f07-current-policy-recruitment-transitions/recruitment-s0.json` | 1,441,006,562 | `977ddfccc252a22a1fb7c12327b5600b2aca69bc9302f3bb4ff7c1746be2e302` |
| gate | `docs/progress/features/t13-f07-current-policy-recruitment-transitions.json` | 97,724 | `3fd71b043e465244db945379a98bce5f81780800d5ca0858d6c2cfb9212109ac` | `.bench-artifacts/t13-f07-current-policy-recruitment-transitions/gate.json` | 94,864 | `763def13518997d1454c95df43f43fa75d3c64cbf44e2a8e73bda9a2c16b7244` |
| goal | `docs/progress/features/t13-f07-current-policy-recruitment-transitions-goal.json` | 7,632,294 | `a825b33223bdb72044ddf60bda74969d1998c54d2e5a3084e1739e0f7ac22eb1` | `.bench-artifacts/t13-f07-current-policy-recruitment-transitions/goal.json` | 591,945,210 | `9e5c9180cc8d9c8d4e263c9fc3ed7a6f2cf055cc821f3cfe6ca2fadc370f43f6` |

Evidence (relocated 2026-09-24, local only): the S0 summary moved out of Git to
`.bench-artifacts/research/t13-f07-current-policy-recruitment-transitions/t13-f07-current-policy-recruitment-transitions-s0.json`,
sha256 `1dab13f5d3e42d2d7e0d83d528bdc860dc0b7ee09326de353c8ac3dc41dda589`,
2,405,314 bytes (byte-identical to the committed blob). This file's S0
sections carry its numbers.

Raw sha256 and byte counts re-checked with `shasum -a 256` and `ls -l`
after all four runs; each matches its summary's `raw` block. Gate and goal
raw `availability` `verified_local` (`verified_at` 2026-09-20T03:34:06Z gate,
2026-09-20T03:42:22Z goal; `generated_at` 2026-09-20T03:34:06Z /
2026-09-20T03:42:20Z; `git_revision`
`cdc4674b4da32563b45347df56d8bc412a182559`; host Apple M1 Pro, macOS,
8 logical cores). Pilot and S0 summaries: `source_revision` the same
commit, `config_digest`
`sha256:ac1bf26562f153ae554f871dfd815f3bdd06e9949a8caf0a3da0f3f9102f7a8d`
(equal to the goal legacy panel's `recruitment_paths.config_digest`),
`supply_rule` "production per-unit supply on the child's own
genome_size()", `version` `recruitment-transitions-s0-v1`. The pilot
summary (`-s0-pilot.json`) is a tool-written by-product; the spec names only
the `-s0.json` summary. Thresholds recorded in `measurement_evidence`: work
flag 10% / severe 50%, wall flag 25% / severe 100%, wall not fatal; caps
founder 10 s per world, evolved 180 s, drift 30 s per world, recruitment
120 s per report, goal investigation 900 s.

## Pilot (run 1)

`panel.sizes` `{batches 1, lineages 2, discovery 256, followup 256}`, 27
arms, 54 lineages, 55,296 proposals (`expected_proposals` 55,296),
`incomplete` false, `stop_reason` null, `threads` 8, `wall_secs` 6.784.
Replay check: `{"proposals":55296,"matched":55296,"first_mismatch":null}`
= 100%.

Per-arm bytes are the byte lengths of the arm's two single-line lineage
records in the raw file (56 lines: header, 54 lineage lines, footer; 549
non-lineage bytes). The tool records one wall reading for the whole run
(arms and lineages run in parallel on the pool); per-arm wall seconds are
not recorded and are not estimated here.

| Arm | Start | Policy | Task | Lineages | Bytes | Max lineage bytes |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 0 | graph_blank | Drift | A | 2 | 1,188,970 | 621,211 |
| 1 | graph_blank | Selection | A | 2 | 1,188,380 | 620,617 |
| 2 | graph_copy | Drift | A | 2 | 1,668,355 | 1,067,252 |
| 3 | graph_copy | Selection | A | 2 | 1,668,363 | 1,067,256 |
| 4 | graph_split | Drift | A | 2 | 2,543,947 | 1,675,905 |
| 5 | graph_split | Selection | A | 2 | 2,543,955 | 1,675,909 |
| 6 | vm_blank | Drift | A | 2 | 2,169,000 | 1,600,216 |
| 7 | vm_blank | Selection | A | 2 | 2,169,008 | 1,600,220 |
| 8 | vm_copy | Drift | A | 2 | 2,428,233 | 1,315,595 |
| 9 | vm_copy | Selection | A | 2 | 2,428,241 | 1,315,599 |
| 10 | graph_unprepared | Drift | B | 2 | 2,991,621 | 1,800,198 |
| 11 | graph_unprepared | Selection | B | 2 | 4,422,021 | 3,071,347 |
| 12 | graph_prepared | Drift | B | 2 | 2,994,298 | 1,802,251 |
| 13 | graph_prepared | Selection | B | 2 | 2,702,114 | 1,897,764 |
| 14 | vm_unprepared | Drift | B | 2 | 2,565,068 | 1,950,245 |
| 15 | vm_unprepared | Selection | B | 2 | 2,622,761 | 1,950,249 |
| 16 | vm_prepared | Drift | B | 2 | 2,565,256 | 1,950,379 |
| 17 | vm_prepared | Selection | B | 2 | 2,562,682 | 1,950,383 |
| 18 | graph_blank | CostSelection | A | 2 | 839,401 | 432,251 |
| 19 | graph_copy | CostSelection | A | 2 | 848,413 | 433,024 |
| 20 | graph_split | CostSelection | A | 2 | 855,124 | 436,167 |
| 21 | vm_blank | CostSelection | A | 2 | 701,753 | 351,891 |
| 22 | vm_copy | CostSelection | A | 2 | 725,557 | 364,972 |
| 23 | graph_unprepared | CostSelection | B | 2 | 836,873 | 450,996 |
| 24 | graph_prepared | CostSelection | B | 2 | 840,558 | 446,302 |
| 25 | vm_unprepared | CostSelection | B | 2 | 706,654 | 354,838 |
| 26 | vm_prepared | CostSelection | B | 2 | 706,690 | 354,860 |

Projection to the S0 panel (64 lineages per arm = 32 × the pilot's 2):

| Projection | Wall | Bytes | Launch margin (≤ 1.6 h, ≤ 1.6 GiB) | Cap (2 h, 2 GiB) |
| --- | ---: | ---: | --- | --- |
| Mean-based (pilot × 32) | 6.784 × 32 = 217.1 s = 0.0603 h | 50,483,296 × 32 + 549 = 1,615,466,021 = 1.5045 GiB | inside | inside |
| Max-based (largest lineage line 3,071,347 × 64 × 27) | not computable (no per-lineage wall reading) | 5,307,288,165 = 4.9428 GiB | over (reported beside, not the launch rule) | over |

Launch decision: the mean-based projection is inside both margins; the S0
panel was launched. Observed S0 outcome against the projection: wall
162.537 s (mean projection 217.1 s), bytes 1,441,006,562 = 1.3421 GiB
(mean projection 1.5045 GiB, max-based 4.9428 GiB).

## S0 panel (run 2)

`panel.sizes` `{batches 4, lineages 16, discovery 256, followup 256}`, 27
arms, `lineage_count` 1,728, `proposal_count` 1,769,472
(`expected_proposals` 1,769,472), `incomplete` false, `stop_reason` null,
`threads` 8, `wall_secs` 162.537, `replay_check` null (pilot-scale only).
Raw 1,441,006,562 bytes (1.342 GiB) against the 2 GiB byte cap and the
7,200 s wall cap; summary 2,405,314 bytes against the spec's 4 MB note.
Reconciliation: 27 arms × 64 lineages = 1,728 rows = 1,728 class
assignments (every lineage has exactly one class); `opportunities.births`
1,769,472 = proposals. The raw record carries 1,728
`route_destination_varies` keys (one per lineage record).

Pooled `opportunities`: births 1,769,472; zero_event_births 1,403,681;
attempted 489,969; applied 472,048; skipped 17,921; reachable / unreachable
/ executed target events 359,055 / 57,008 / 340,010;
`attempted_by_domain` Topology 98,799, Vm 130,445, Graph 129,604, InputRef
131,121; `applied_by_domain` Topology 98,799, Vm 130,391, Graph 111,737,
InputRef 131,121; `skipped_by_operator_reason` {}; `selected_inapplicable_by_domain`
{}; `no_eligible_node_by_domain` Vm 54, Graph 17,867.

### S0 ladder per arm

| Arm | Start | Policy | Task | Lineages | eligibility | local_edit | expression | specialized | proposal_specialized (lineages) | specialized_proposals | specialized_after_window | bypass_only | Classes | eligible_site_fraction |
| ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| 0 | graph_blank | Drift | A | 64 | 64 | 61 | 61 | 0 | 0 | 0 | 0 | 0 | no_benefit 60, no_edit 3, no_expression 1 | 348/498 = 0.6988 |
| 1 | graph_blank | Selection | A | 64 | 64 | 61 | 61 | 0 | 0 | 0 | 0 | 0 | no_benefit 60, no_edit 3, no_expression 1 | 348/498 = 0.6988 |
| 2 | graph_copy | Drift | A | 64 | 64 | 62 | 63 | 0 | 0 | 0 | 0 | 0 | no_benefit 61, no_edit 2, no_expression 1 | 583/856 = 0.6811 |
| 3 | graph_copy | Selection | A | 64 | 64 | 62 | 63 | 0 | 0 | 0 | 0 | 0 | no_benefit 61, no_edit 2, no_expression 1 | 581/856 = 0.6787 |
| 4 | graph_split | Drift | A | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 0 | no_benefit 64 | 755/1107 = 0.6820 |
| 5 | graph_split | Selection | A | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 0 | no_benefit 64 | 738/1066 = 0.6923 |
| 6 | vm_blank | Drift | A | 64 | 64 | 63 | 64 | 0 | 0 | 0 | 0 | 0 | no_benefit 63, no_edit 1 | 542/737 = 0.7354 |
| 7 | vm_blank | Selection | A | 64 | 64 | 63 | 64 | 0 | 0 | 0 | 0 | 0 | no_benefit 63, no_edit 1 | 556/770 = 0.7221 |
| 8 | vm_copy | Drift | A | 64 | 64 | 64 | 63 | 0 | 1 | 2 | 1 | 0 | loss_not_selected 1, no_benefit 62, no_expression 1 | 840/1178 = 0.7131 |
| 9 | vm_copy | Selection | A | 64 | 64 | 64 | 63 | 1 | 1 | 1056 | 1 | 1 | no_benefit 62, no_expression 1, retained 1 | 834/1193 = 0.6991 |
| 10 | graph_unprepared | Drift | B | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 2 | no_benefit 64 | 691/1017 = 0.6794 |
| 11 | graph_unprepared | Selection | B | 64 | 64 | 62 | 62 | 0 | 0 | 0 | 0 | 1 | no_benefit 61, no_edit 2, no_expression 1 | 741/1093 = 0.6780 |
| 12 | graph_prepared | Drift | B | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 33 | no_benefit 64 | 718/1038 = 0.6917 |
| 13 | graph_prepared | Selection | B | 64 | 64 | 64 | 64 | 0 | 0 | 2 | 1 | 44 | no_benefit 64 | 640/958 = 0.6681 |
| 14 | vm_unprepared | Drift | B | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 3 | no_benefit 64 | 775/1076 = 0.7203 |
| 15 | vm_unprepared | Selection | B | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 0 | no_benefit 64 | 802/1187 = 0.6757 |
| 16 | vm_prepared | Drift | B | 64 | 64 | 64 | 64 | 0 | 0 | 13 | 1 | 40 | no_benefit 64 | 767/1061 = 0.7229 |
| 17 | vm_prepared | Selection | B | 64 | 64 | 64 | 64 | 0 | 0 | 0 | 0 | 50 | no_benefit 64 | 837/1193 = 0.7016 |
| 18 | graph_blank | CostSelection | A | 64 | 64 | 20 | 43 | 0 | 0 | 0 | 0 | 0 | no_benefit 19, no_edit 44, no_expression 1 | 34/64 = 0.5312 |
| 19 | graph_copy | CostSelection | A | 64 | 64 | 33 | 32 | 0 | 0 | 0 | 0 | 0 | no_benefit 26, no_edit 31, no_expression 7 | 62/65 = 0.9538 |
| 20 | graph_split | CostSelection | A | 64 | 64 | 15 | 1 | 0 | 0 | 0 | 0 | 0 | no_edit 49, no_expression 15 | 61/65 = 0.9385 |
| 21 | vm_blank | CostSelection | A | 64 | 64 | 13 | 34 | 0 | 0 | 0 | 0 | 0 | no_benefit 13, no_edit 51 | 49/64 = 0.7656 |
| 22 | vm_copy | CostSelection | A | 64 | 64 | 35 | 39 | 1 | 1 | 1095 | 1 | 0 | no_benefit 32, no_edit 29, no_expression 2, retained 1 | 63/64 = 0.9844 |
| 23 | graph_unprepared | CostSelection | B | 64 | 64 | 13 | 2 | 0 | 0 | 0 | 0 | 0 | no_benefit 2, no_edit 51, no_expression 11 | 56/64 = 0.8750 |
| 24 | graph_prepared | CostSelection | B | 64 | 64 | 43 | 43 | 0 | 0 | 0 | 0 | 42 | no_benefit 43, no_edit 21 | 64/67 = 0.9552 |
| 25 | vm_unprepared | CostSelection | B | 64 | 64 | 9 | 4 | 0 | 0 | 0 | 0 | 0 | no_benefit 3, no_edit 55, no_expression 6 | 59/66 = 0.8939 |
| 26 | vm_prepared | CostSelection | B | 64 | 64 | 45 | 45 | 0 | 0 | 0 | 0 | 44 | no_benefit 43, no_edit 19, no_expression 2 | 62/64 = 0.9688 |

Unprepared arms (21 of 27 = 7 forms × 3 selectors: 0–11, 14, 15, 18–23,
25): `specialized` 0 in 19 of the 21, except arm 9 (`vm_copy`/Selection, 1
lineage retained at horizons 16, 64 and 256, `specialized_proposals` 1,056)
and arm 22 (`vm_copy`/CostSelection, 1 lineage retained at 16/64/256,
`specialized_proposals` 1,095); arm 8 (`vm_copy`/Drift) has one
`loss_not_selected` lineage (1 specialized proposal in the window, never
retained). Prepared arms (12, 13, 16, 17, 24, 26): `specialized` 0,
`bypass_only` 33 / 44 / 40 / 50 / 42 / 44; expression 64/64 under Drift and
Selection (12, 13, 16, 17), 43/64 and 45/64 under CostSelection (24, 26).

`specialized_after_window` counts lineages with `specialized_proposals` > 0
whose `proposal_specialized` is false: specialization first appeared after
the 256-generation discovery window, so the lineage keeps its null class
(the F02/report convention, spec Inputs and Invariants). Derived from the S0
summary's per-lineage rows: five lineages in five arms, 0 in the other 22 —
arm 8 b3/l0 (1 proposal), arm 9 b3/l0 (460), arm 13 b0/l7 (2), arm 16 b3/l9
(13), arm 22 b2/l15 (461); the in-window rows are arm 8 b1/l15 (1 proposal,
`loss_not_selected`), arm 9 b1/l15 (596, discovery at generation 214,
retained) and arm 22 b1/l13 (634, discovery at 192, retained).

### S0 discovery, retention and destination kinds per arm

| Arm | Start | Policy | proposal_discovery | retained_discovery | viable_retained | retained_useful | retained_at | retention_outcomes | destination_kinds (vm / graph_stateful / graph_pure_no_effect / graph_pure_with_effect) |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | graph_blank | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 227 / 39 / 134 / 45 |
| 1 | graph_blank | Selection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 227 / 39 / 134 / 45 |
| 2 | graph_copy | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 355 / 95 / 208 / 95 |
| 3 | graph_copy | Selection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 353 / 95 / 210 / 96 |
| 4 | graph_split | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 483 / 150 / 240 / 90 |
| 5 | graph_split | Selection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 470 / 144 / 230 / 82 |
| 6 | vm_blank | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 356 / 56 / 161 / 70 |
| 7 | vm_blank | Selection | 1/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 370 / 60 / 175 / 71 |
| 8 | vm_copy | Drift | 1/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 549 / 81 / 301 / 100 |
| 9 | vm_copy | Selection | 1/64 | 1/64 | 1/64 | 1/64 | {"16": 1, "64": 1, "256": 1} | useful 1, no_longer_useful 0, deleted 0, task_dead 0 | 568 / 97 / 287 / 100 |
| 10 | graph_unprepared | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 456 / 116 / 219 / 106 |
| 11 | graph_unprepared | Selection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 472 / 102 / 268 / 113 |
| 12 | graph_prepared | Drift | 43/64 | 31/64 | 29/64 | 3/64 | {} | useful 3, no_longer_useful 24, deleted 4, task_dead 0 | 465 / 124 / 213 / 109 |
| 13 | graph_prepared | Selection | 43/64 | 43/64 | 43/64 | 40/64 | {} | useful 40, no_longer_useful 2, deleted 1, task_dead 0 | 404 / 114 / 225 / 90 |
| 14 | vm_unprepared | Drift | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 523 / 79 / 242 / 79 |
| 15 | vm_unprepared | Selection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 569 / 112 / 280 / 85 |
| 16 | vm_prepared | Drift | 49/64 | 39/64 | 30/64 | 5/64 | {} | useful 5, no_longer_useful 30, deleted 4, task_dead 0 | 511 / 83 / 237 / 78 |
| 17 | vm_prepared | Selection | 50/64 | 50/64 | 50/64 | 43/64 | {} | useful 43, no_longer_useful 7, deleted 0, task_dead 0 | 574 / 91 / 314 / 88 |
| 18 | graph_blank | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 0 / 0 / 50 / 0 |
| 19 | graph_copy | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 1 / 8 / 7 / 19 |
| 20 | graph_split | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 0 / 6 / 1 / 11 |
| 21 | vm_blank | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 28 / 0 / 0 / 0 |
| 22 | vm_copy | CostSelection | 1/64 | 1/64 | 1/64 | 1/64 | {"16": 1, "64": 1, "256": 1} | useful 1, no_longer_useful 0, deleted 0, task_dead 0 | 30 / 0 / 1 / 0 |
| 23 | graph_unprepared | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 0 / 4 / 0 / 16 |
| 24 | graph_prepared | CostSelection | 42/64 | 42/64 | 42/64 | 41/64 | {} | useful 41, no_longer_useful 0, deleted 1, task_dead 0 | 1 / 17 / 0 / 30 |
| 25 | vm_unprepared | CostSelection | 0/64 | 0/64 | 0/64 | 0/64 | {} | useful 0, no_longer_useful 0, deleted 0, task_dead 0 | 12 / 0 / 1 / 0 |
| 26 | vm_prepared | CostSelection | 44/64 | 44/64 | 44/64 | 44/64 | {} | useful 44, no_longer_useful 0, deleted 0, task_dead 0 | 45 / 0 / 0 / 0 |

### S0 exposure per arm (`opportunities` pooled over the arm's 64 lineages)

The summary carries attempted (requested), applied and skipped event counts
per arm; `attempted / births` is the requested-events-per-birth reading the
predeclaration names, with the arm's genome-size trajectory in the next
table beside it (the summary holds no per-birth genome size, so the
0.005 × `genome_size` comparison is left as the two readings side by side).

| Arm | Start | Policy | births | zero_event_births | attempted | applied | skipped | attempted per birth | applied per birth | reachable / unreachable / executed target events | task_dead | task_live_loss |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| 0 | graph_blank | Drift | 65,536 | 55,523 | 12,383 | 12,383 | 0 | 0.1889 | 0.1889 | 9,951 / 1,107 / 9,140 | 0/65536 | 7/65536 |
| 1 | graph_blank | Selection | 65,536 | 55,523 | 12,383 | 12,383 | 0 | 0.1889 | 0.1889 | 9,951 / 1,107 / 9,140 | 0/65536 | 8/65536 |
| 2 | graph_copy | Drift | 65,536 | 50,124 | 21,233 | 21,228 | 5 | 0.3240 | 0.3239 | 16,752 / 2,116 / 16,024 | 0/65536 | 5/65536 |
| 3 | graph_copy | Selection | 65,536 | 50,131 | 21,218 | 21,213 | 5 | 0.3238 | 0.3237 | 16,734 / 2,117 / 16,004 | 0/65536 | 5/65536 |
| 4 | graph_split | Drift | 65,536 | 47,765 | 25,972 | 25,965 | 7 | 0.3963 | 0.3962 | 20,623 / 2,616 / 19,957 | 876/65536 | 9/64660 |
| 5 | graph_split | Selection | 65,536 | 48,122 | 25,208 | 25,201 | 7 | 0.3846 | 0.3845 | 20,003 / 2,538 / 19,405 | 2/65536 | 10/65534 |
| 6 | vm_blank | Drift | 65,536 | 50,751 | 19,001 | 17,804 | 1,197 | 0.2899 | 0.2717 | 12,858 / 2,845 / 11,951 | 785/65536 | 6/64751 |
| 7 | vm_blank | Selection | 65,536 | 50,630 | 19,366 | 18,179 | 1,187 | 0.2955 | 0.2774 | 13,264 / 2,785 / 12,263 | 4/65536 | 27/65532 |
| 8 | vm_copy | Drift | 65,536 | 43,794 | 30,556 | 29,427 | 1,129 | 0.4662 | 0.4490 | 21,569 / 4,585 / 19,890 | 97/65536 | 14/65439 |
| 9 | vm_copy | Selection | 65,536 | 43,874 | 30,534 | 29,380 | 1,154 | 0.4659 | 0.4483 | 21,511 / 4,591 / 19,862 | 6/65536 | 20/65530 |
| 10 | graph_unprepared | Drift | 65,536 | 48,157 | 24,266 | 24,253 | 13 | 0.3703 | 0.3701 | 18,693 / 3,070 / 17,919 | 305/65536 | 168/65231 |
| 11 | graph_unprepared | Selection | 65,536 | 47,870 | 25,983 | 25,972 | 11 | 0.3965 | 0.3963 | 20,211 / 3,112 / 19,343 | 1/65536 | 357/65535 |
| 12 | graph_prepared | Drift | 65,536 | 48,120 | 24,625 | 24,612 | 13 | 0.3757 | 0.3755 | 18,429 / 3,609 / 17,676 | 305/65536 | 175/65231 |
| 13 | graph_prepared | Selection | 65,536 | 47,920 | 23,577 | 23,572 | 5 | 0.3598 | 0.3597 | 18,810 / 2,178 / 18,226 | 1/65536 | 807/65535 |
| 14 | vm_unprepared | Drift | 65,536 | 44,916 | 28,716 | 27,296 | 1,420 | 0.4382 | 0.4165 | 20,331 / 3,929 / 18,934 | 1027/65536 | 242/64509 |
| 15 | vm_unprepared | Selection | 65,536 | 44,605 | 29,923 | 28,587 | 1,336 | 0.4566 | 0.4362 | 21,208 / 4,158 / 19,788 | 18/65536 | 368/65518 |
| 16 | vm_prepared | Drift | 65,536 | 44,961 | 28,437 | 27,021 | 1,416 | 0.4339 | 0.4123 | 20,156 / 3,853 / 18,763 | 1027/65536 | 234/64509 |
| 17 | vm_prepared | Selection | 65,536 | 44,848 | 29,668 | 28,586 | 1,082 | 0.4527 | 0.4362 | 20,452 / 4,879 / 18,889 | 13/65536 | 665/65523 |
| 18 | graph_blank | CostSelection | 65,536 | 62,051 | 3,592 | 3,592 | 0 | 0.0548 | 0.0548 | 2,872 / 199 / 2,607 | 0/65536 | 1/65536 |
| 19 | graph_copy | CostSelection | 65,536 | 60,567 | 5,199 | 5,199 | 0 | 0.0793 | 0.0793 | 4,157 / 213 / 4,085 | 0/65536 | 5/65536 |
| 20 | graph_split | CostSelection | 65,536 | 60,159 | 5,666 | 5,661 | 5 | 0.0865 | 0.0864 | 4,435 / 282 / 4,339 | 0/65536 | 1/65536 |
| 21 | vm_blank | CostSelection | 65,536 | 59,923 | 5,889 | 4,277 | 1,612 | 0.0899 | 0.0653 | 3,113 / 210 / 3,046 | 0/65536 | 10/65536 |
| 22 | vm_copy | CostSelection | 65,536 | 58,033 | 8,055 | 5,916 | 2,139 | 0.1229 | 0.0903 | 4,393 / 135 / 4,343 | 0/65536 | 8/65536 |
| 23 | graph_unprepared | CostSelection | 65,536 | 59,822 | 6,011 | 6,004 | 7 | 0.0917 | 0.0916 | 4,819 / 174 / 4,759 | 0/65536 | 255/65536 |
| 24 | graph_prepared | CostSelection | 65,536 | 59,510 | 6,356 | 6,356 | 0 | 0.0970 | 0.0970 | 5,046 / 196 / 5,014 | 0/65536 | 453/65536 |
| 25 | vm_unprepared | CostSelection | 65,536 | 57,970 | 8,105 | 6,012 | 2,093 | 0.1237 | 0.0917 | 4,392 / 217 / 4,350 | 0/65536 | 267/65536 |
| 26 | vm_prepared | CostSelection | 65,536 | 58,012 | 8,047 | 5,969 | 2,078 | 0.1228 | 0.0911 | 4,322 / 187 / 4,293 | 0/65536 | 432/65536 |

### S0 genome size per arm at checkpoints

`checkpoint_cost` min / median / max from the arm summary; p50 / p90 computed
over the 64 lineage rows' `final_checkpoint.genome_size` (generation 512;
the summary has no per-checkpoint p90, only min/median/max).

| Arm | Start | Policy | gen 0 genome_size | gen 256 min/median/max | gen 512 min/median/max | gen 512 p50 | gen 512 p90 | gen 512 modules median | ending_energy_sum gen 512 min/median |
| ---: | --- | --- | ---: | --- | --- | ---: | ---: | ---: | --- |
| 0 | graph_blank | Drift | 14 | 8 / 24 / 92 | 20 / 52.5 / 997 | 52.5 | 128.1 | 7 | 393.522 / 395.558 |
| 1 | graph_blank | Selection | 14 | 8 / 24 / 92 | 20 / 52.5 / 997 | 52.5 | 128.1 | 7 | 393.522 / 395.558 |
| 2 | graph_copy | Drift | 22 | 15 / 43.5 / 172 | 35 / 93.5 / 1145 | 93.5 | 317.4 | 10 | 3.452 / 395.512 |
| 3 | graph_copy | Selection | 22 | 15 / 43.5 / 172 | 35 / 93.5 / 1145 | 93.5 | 317.4 | 10 | 3.452 / 395.507 |
| 4 | graph_split | Drift | 24 | 16 / 51 / 390 | 29 / 119.5 / 1553 | 119.5 | 353.9 | 12 | -0.000 / 395.496 |
| 5 | graph_split | Selection | 24 | 16 / 50 / 390 | 29 / 113 / 1553 | 113 | 332 | 11.5 | 1.477 / 395.492 |
| 6 | vm_blank | Drift | 21 | 8 / 42.5 / 106 | 13 / 90 / 821 | 90 | 262.7 | 9 | -0.000 / 395.513 |
| 7 | vm_blank | Selection | 21 | 8 / 42 / 106 | 13 / 92 / 821 | 92 | 320.8 | 10 | 3.414 / 395.502 |
| 8 | vm_copy | Drift | 34 | 28 / 66.5 / 203 | 43 / 187 / 1044 | 187 | 478.2 | 17 | 3.107 / 395.431 |
| 9 | vm_copy | Selection | 34 | 28 / 65.5 / 203 | 43 / 189.5 / 1044 | 189.5 | 478.2 | 16 | 3.107 / 395.401 |
| 10 | graph_unprepared | Drift | 25 | 15 / 52 / 304 | 29 / 123.5 / 1787 | 123.5 | 462.2 | 12 | 1.706 / 395.457 |
| 11 | graph_unprepared | Selection | 25 | 15 / 49.5 / 313 | 18 / 111 / 1654 | 111 | 461.3 | 12 | 3.438 / 395.471 |
| 12 | graph_prepared | Drift | 25 | 15 / 52 / 434 | 29 / 123.5 / 1787 | 123.5 | 462.2 | 12 | 1.706 / 395.457 |
| 13 | graph_prepared | Selection | 25 | 18 / 52 / 202 | 32 / 133 / 1373 | 133 | 369.3 | 12.5 | 2.746 / 394.893 |
| 14 | vm_unprepared | Drift | 33 | 25 / 66 / 171 | 36 / 169 / 1238 | 169 | 437.2 | 13.5 | -0.000 / 394.267 |
| 15 | vm_unprepared | Selection | 33 | 30 / 65 / 225 | 23 / 158.5 / 1706 | 158.5 | 431.4 | 14 | 2.383 / 395.326 |
| 16 | vm_prepared | Drift | 33 | 25 / 66 / 171 | 36 / 169 / 1238 | 169 | 392.1 | 13.5 | -0.000 / 394.302 |
| 17 | vm_prepared | Selection | 33 | 23 / 64 / 171 | 30 / 156.5 / 4658 | 156.5 | 380.4 | 15 | 2.383 / 392.913 |
| 18 | graph_blank | CostSelection | 14 | 4 / 11 / 14 | 4 / 8 / 14 | 8 | 12 | 2.5 | 395.589 / 395.594 |
| 19 | graph_copy | CostSelection | 22 | 6 / 16.5 / 22 | 4 / 10 / 21 | 10 | 17 | 2 | 395.583 / 395.592 |
| 20 | graph_split | CostSelection | 24 | 8 / 19 / 24 | 3 / 9.5 / 20 | 9.5 | 17 | 2 | 395.584 / 395.592 |
| 21 | vm_blank | CostSelection | 21 | 5 / 19 / 21 | 4 / 17 / 20 | 17 | 19 | 2 | 394.986 / 395.586 |
| 22 | vm_copy | CostSelection | 34 | 13 / 18 / 33 | 10 / 17 / 32 | 17 | 19.4 | 2 | 394.987 / 395.586 |
| 23 | graph_unprepared | CostSelection | 25 | 5 / 18.5 / 25 | 1 / 12.5 / 25 | 12.5 | 21.7 | 2 | 394.980 / 395.589 |
| 24 | graph_prepared | CostSelection | 25 | 5 / 20 / 25 | 5 / 14 / 25 | 14 | 22.7 | 2 | 394.980 / 394.989 |
| 25 | vm_unprepared | CostSelection | 33 | 15 / 20 / 33 | 11 / 17 / 32 | 17 | 29 | 2 | 394.974 / 395.586 |
| 26 | vm_prepared | CostSelection | 33 | 14 / 18 / 33 | 11 / 17 / 32 | 17 | 18 | 2 | 394.974 / 394.986 |

## Gate (run 3)

`comparison.severe` false against both references; every counter and both
wall readings `ok`. `wall_clock_ms_total` 411.288;
`neighborhood_founder_wall_clock_ms` 42.257 against the 10,000 ms cap;
`recruitment_paths` "unmeasured; recruitment-paths-v1 is goal-only";
`drift_depth` "Undefined". Totals: creature_ticks 272,826, births 7,081,
plasticity_updates 1,895, vm_steps 6,147,387.

| Counter | Current | vs `remove-complementary-nutrition.json` (epoch, `a2917972`) | vs `t17-f02-unit-scale-introspection.json` (latest closure, `ae5eef80`) |
| --- | --- | --- | --- |
| mesh_hops | 2.040249 | +0.640667% ok | +0.461871% ok |
| vm_steps | 22.532262 | −0.962819% ok | +0.316211% ok |
| graph_relax_iters | 0.999842 | +0.463007% ok | −0.023898% ok |
| plasticity_updates | 0.006946 | −37.821144% ok | +1.967117% ok |
| actions_applied | 1.352060 | +6.000274% ok | +0.084091% ok |
| births | 0.025954 | −3.084391% ok | −0.169244% ok |
| wall ms per creature-tick | 0.001508 | −3.354218% ok (ref 0.001560) | −17.678957% ok (ref 0.001831) |

Gate per seed (75 ticks; before = T17.F02 gate summary; no
`extinction_tick`):

| Seed | births (F07) | births (before) | final_population (F07) | final_population (before) | creature_ticks (F07) | creature_ticks (before) | plasticity_updates (F07) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 11 | 2,348 | 2,352 | 2,137 | 2,109 | 90,810 | 90,778 | 954 |
| 22 | 2,390 | 2,349 | 2,183 | 2,117 | 90,876 | 90,261 | 216 |
| 33 | 2,343 | 2,382 | 2,153 | 2,162 | 91,140 | 91,402 | 725 |

Gate founder T11.F14 rows
(`mutational_neighborhood.founder.births.any_events`, 210 applied): changed
87 (0.414286), silent 123 (0.585714), dead 0 — identical to T17.F02's
gate row (87/123/0). Gate founder `mesh_execution`: backends graph 1/1/1
and vm 1/1/1 (contributing/executed/total), `executed_node_count` 2,
`reachable_node_count` 2, `total_node_count` 2, `knockout_count` 0,
`route_varies_with_input` false, `route_destination_varies` false,
`snapshot_route_probes` 48, `version` `mesh-execution-v1`. The gate raw
report holds one `route_destination_varies` key (the founder block).

## Goal (run 4)

`comparison.severe` false; one comparison printed because the goal-worlds-v1
epoch and the latest closure are the same file
(`t17-f02-unit-scale-introspection-goal.json`, identity
`ae5eef809af00ef41d4dd0d9d7be8ce6b6e99cf4`, 2026-09-19T06:41:38Z). The
series index's `goal` (goal-v1) epoch
`t11-f17-executed-biased-mutation-targeting-goal.json` is not a reference
the CLI printed for this run. Run once; the second determinism run is
closed by the 2026-09-05 user decision.

| Counter | Current | Reference (T17.F02 goal) | Delta | Level |
| --- | --- | --- | --- | --- |
| mesh_hops | 2.284326 | 2.220186 | +2.888947% | ok |
| vm_steps | 23.014350 | 23.114328 | −0.432537% | ok |
| graph_relax_iters | 1.020063 | 1.041916 | −2.097386% | ok |
| plasticity_updates | 0.047907 | 0.088735 | −46.011157% | ok (above the 10% flag, below the 50% severe line) |
| actions_applied | 1.328119 | 1.358706 | −2.251186% | ok |
| births | 0.015790 | 0.015684 | +0.675848% | ok |
| wall ms per creature-tick | 0.011824 | 0.012671 | −6.677920% | ok |

Goal `plasticity_updates` totals 1,865,405 over 38,938,170 creature-ticks
(T17.F02: 3,531,132 over 39,794,254). Goal wall budgets (closure checks):
`wall_clock_ms_total` 460,423.787 (seeds 11/22/33: 182,106.8 / 113,147.5
/ 165,169.5) against the 900 s investigation threshold;
`neighborhood_evolved_wall_clock_ms_total` 571.637 (197.936 / 186.651 /
187.051 per seed) against the 180,000 ms summed cap;
`neighborhood_founder_wall_clock_ms` 99.085 against the 10,000 ms cap;
`neighborhood_read_wall_clock_ms_total` 237.406; `drift_depth_wall_clock_ms`
13,194.576 (30 s per world cap); `recruitment_paths_wall_clock_ms` 11,517.792
against the 120,000 ms cap (the spec's "goal experiment under 120 s"). No
`extinction_tick` in any seed or world.

### Goal per seed (`deterministic.per_seed`, 2,000 ticks; T17.F02 in parentheses)

| Seed | births | final_population | creature_ticks | plasticity_updates | extinction_tick |
| ---: | ---: | ---: | ---: | ---: | --- |
| 11 | 178,619 (197,547) | 5,849 (7,055) | 11,410,064 (12,857,652) | 238,293 (1,304,158) | null (null) |
| 22 | 188,444 (175,258) | 3,506 (3,741) | 11,858,968 (11,172,435) | 886,699 (1,107,756) | null (null) |
| 33 | 247,782 (251,335) | 9,084 (12,011) | 15,669,138 (15,764,167) | 740,413 (1,119,218) | null (null) |

### Goal per world versus T17.F02 (`comparison_inputs.case_readings`)

| World | Indicator | T17.F02 | T13.F07 |
| --- | --- | --- | --- |
| Orchards in grassland | `final_population` | 7,055 | 5,849 |
| Orchards in grassland | `minimum_population` | 149 | 30 |
| Orchards in grassland | `peak_population` | 100,000 | 100,000 |
| Orchards in grassland | `plateau_population` | 4,318.060 | 2,934.114 |
| Orchards in grassland | `births` | 197,547 | 178,619 |
| Orchards in grassland | `mean_energy` | 41.698212 | 32.594680 |
| Orchards in grassland | `extinction_tick` | null | null |
| Orchards in grassland | `births_per_creature_tick` | 0.015364 | 0.015655 |
| Orchards in grassland | `vm_steps_per_creature_tick` | 22.912170 | 22.836424 |
| Orchards in grassland | `mesh_hops_per_creature_tick` | 2.148887 | 2.131845 |
| Orchards in grassland | `graph_relax_iters_per_creature_tick` | 1.033170 | 1.003663 |
| Orchards in grassland | `plasticity_updates_per_creature_tick` | 0.101430 | 0.020884 |
| Orchards in grassland | `actions_applied_per_creature_tick` | 1.358125 | 1.350282 |
| Orchards in grassland | `founder_changed_per_all_births` | 0.457143 | 0.457143 |
| Orchards in grassland | `founder_dead_per_all_births` | 0 | 0 |
| Orchards in grassland | `evolved_changed_per_mutated_births` | 0.394668 | 0.347578 |
| Orchards in grassland | `evolved_dead_per_mutated_births` | 0.022757 | 0.007123 |
| Orchards in grassland | `neighborhood_read_changed_per_all_births` | 0.2156 | 0.2118 |
| Orchards in grassland | `neighborhood_read_dead_per_all_births` | 0.0068 | 0.0024 |
| Orchards in grassland | `neighborhood_read_silent_per_all_births` | 0.3714 | 0.355 |
| Orchards in grassland | `drift_changed_per_all_births_at_2000` | 0.0015 | 0.001 |
| Orchards in grassland | `lineage_shannon_entropy_nats` | 1.429177 | 0.872903 |
| Orchards in grassland | `surviving_founder_clade_count` | 10 | 4 |
| Orchards in grassland | `reachable_structure_size_median` | 87 | 72 |
| Canyon country | `final_population` | 3,741 | 3,506 |
| Canyon country | `minimum_population` | 806 | 1,282 |
| Canyon country | `peak_population` | 79,825 | 80,476 |
| Canyon country | `plateau_population` | 4,177.842 | 4,670.504 |
| Canyon country | `births` | 175,258 | 188,444 |
| Canyon country | `mean_energy` | 30.704367 | 31.747004 |
| Canyon country | `extinction_tick` | null | null |
| Canyon country | `births_per_creature_tick` | 0.015687 | 0.015890 |
| Canyon country | `vm_steps_per_creature_tick` | 23.097642 | 23.245295 |
| Canyon country | `mesh_hops_per_creature_tick` | 2.195508 | 2.418215 |
| Canyon country | `graph_relax_iters_per_creature_tick` | 1.010572 | 1.049494 |
| Canyon country | `plasticity_updates_per_creature_tick` | 0.099151 | 0.074770 |
| Canyon country | `actions_applied_per_creature_tick` | 1.347814 | 1.325933 |
| Canyon country | `founder_changed_per_all_births` | 0.414286 | 0.414286 |
| Canyon country | `founder_dead_per_all_births` | 0 | 0 |
| Canyon country | `evolved_changed_per_mutated_births` | 0.308634 | 0.295593 |
| Canyon country | `evolved_dead_per_mutated_births` | 0.016315 | 0 |
| Canyon country | `neighborhood_read_changed_per_all_births` | 0.2048 | 0.142 |
| Canyon country | `neighborhood_read_dead_per_all_births` | 0.0094 | 0.0072 |
| Canyon country | `neighborhood_read_silent_per_all_births` | 0.357 | 0.3704 |
| Canyon country | `drift_changed_per_all_births_at_2000` | 0.0035 | 0.0005 |
| Canyon country | `lineage_shannon_entropy_nats` | 1.432460 | 2.076143 |
| Canyon country | `surviving_founder_clade_count` | 9 | 10 |
| Canyon country | `reachable_structure_size_median` | 75 | 79 |
| Confluence | `final_population` | 12,011 | 9,084 |
| Confluence | `minimum_population` | 191 | 155 |
| Confluence | `peak_population` | 100,000 | 100,000 |
| Confluence | `plateau_population` | 9,101.302 | 7,111.280 |
| Confluence | `births` | 251,335 | 247,782 |
| Confluence | `mean_energy` | 36.881574 | 33.559567 |
| Confluence | `extinction_tick` | null | null |
| Confluence | `births_per_creature_tick` | 0.015943 | 0.015813 |
| Confluence | `vm_steps_per_creature_tick` | 23.291039 | 22.969126 |
| Confluence | `mesh_hops_per_creature_tick` | 2.295830 | 2.294029 |
| Confluence | `graph_relax_iters_per_creature_tick` | 1.071262 | 1.009731 |
| Confluence | `plasticity_updates_per_creature_tick` | 0.070998 | 0.047253 |
| Confluence | `actions_applied_per_creature_tick` | 1.366899 | 1.313635 |
| Confluence | `founder_changed_per_all_births` | 0.457143 | 0.457143 |
| Confluence | `founder_dead_per_all_births` | 0 | 0 |
| Confluence | `evolved_changed_per_mutated_births` | 0.356962 | 0.304762 |
| Confluence | `evolved_dead_per_mutated_births` | 0 | 0.000794 |
| Confluence | `neighborhood_read_changed_per_all_births` | 0.2442 | 0.1788 |
| Confluence | `neighborhood_read_dead_per_all_births` | 0.0008 | 0.0038 |
| Confluence | `neighborhood_read_silent_per_all_births` | 0.4732 | 0.3238 |
| Confluence | `drift_changed_per_all_births_at_2000` | 0.0015 | 0.001 |
| Confluence | `lineage_shannon_entropy_nats` | 1.027931 | 1.251369 |
| Confluence | `surviving_founder_clade_count` | 4 | 5 |
| Confluence | `reachable_structure_size_median` | 90 | 75 |

### Goal T11.F14 rows per world (changed / silent / dead; T17.F02 in parentheses)

Founder: `cases[].mutational_neighborhood.founder.births.any_events`;
evolved: `cases[].mutational_neighborhood.evolved.per_seed[0].pooled_births.any_events`
(2,400 births per world).

| World | Founder applied | Founder changed / silent / dead | Evolved applied | Evolved changed / silent / dead | Evolved fractions changed / silent / dead |
| --- | ---: | --- | ---: | --- | --- |
| Orchards in grassland | 210 (210) | 96/114/0 (96/114/0) | 1,404 (1,538) | 488/906/10 (607/896/35) | 0.347578 / 0.645299 / 0.007123 (0.394668 / 0.582575 / 0.022757) |
| Canyon country | 210 (210) | 87/123/0 (87/123/0) | 1,316 (1,471) | 389/927/0 (454/993/24) | 0.295593 / 0.704407 / 0.000000 (0.308634 / 0.675051 / 0.016315) |
| Confluence | 210 (210) | 96/114/0 (96/114/0) | 1,260 (1,580) | 384/875/1 (564/1016/0) | 0.304762 / 0.694444 / 0.000794 (0.356962 / 0.643038 / 0.000000) |

### Goal `mesh_execution` per world

Founder `mesh_execution` in every world: graph 1/1/1, vm 1/1/1,
`executed_node_count` 2, `reachable_node_count` 2, `total_node_count` 2,
`knockout_count` 0, `route_varies_with_input` false,
`route_destination_varies` false (the two founder readings are equal).
Evolved `mesh_summary` (executed / reachable / total / routeVaries /
routeDestinationVaries / knockout; T17.F02 executed/reachable/total/routeVaries):
Orchards 31/39/50/1/1/7 (30/41/58/1), Canyon 35/39/50/5/4/11 (39/47/58/6),
Confluence 41/48/53/1/1/17 (30/37/59/0). Raw goal report: 39
`mesh_execution` blocks carry both route keys — 32 read false/false, 1
reads `route_varies_with_input` true with `route_destination_varies` false,
6 read true/true, 0 read destination true with position false (`grep -A1`
tally over the raw). The raw holds 2,631 `route_destination_varies` keys in
all; the other 2,592 sit in per-module `MeshObservation` blocks that carry no
`route_varies_with_input` neighbour.

### Goal drift depth per world (`drift_depth.readings`, changed / silent / dead of applied)

Orchards and Confluence read identically at every checkpoint (the walk seed
formula does not vary with the world; the T17.F02 summary shows the same
equality at 2,000: 0.0015 for both). Reported against the T11 track's
0.005 with no gate.

| World | Checkpoint | applied | changed | silent | dead | changed_fraction | mesh graph executed/contributing/total, vm executed/contributing/total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Orchards in grassland | 0 | 872 | 415 | 457 | 0 | 0.475917 | g 50/50/50, vm 50/50/50 |
| Orchards in grassland | 22 | 865 | 226 | 639 | 0 | 0.261272 | g 67/49/84, vm 64/51/79 |
| Orchards in grassland | 250 | 897 | 45 | 847 | 5 | 0.050167 | g 99/20/291, vm 88/44/254 |
| Orchards in grassland | 1000 | 868 | 9 | 858 | 1 | 0.010369 | g 174/10/1045, vm 167/38/925 |
| Orchards in grassland | 2000 | 865 | 2 | 863 | 0 | 0.002312 | g 220/4/2040, vm 208/36/1897 |
| Canyon country | 0 | 872 | 405 | 467 | 0 | 0.464450 | g 50/50/50, vm 50/50/50 |
| Canyon country | 22 | 865 | 216 | 649 | 0 | 0.249711 | g 63/47/82, vm 64/50/81 |
| Canyon country | 250 | 897 | 28 | 862 | 7 | 0.031215 | g 93/22/285, vm 87/41/289 |
| Canyon country | 1000 | 868 | 17 | 851 | 0 | 0.019585 | g 219/8/1034, vm 195/29/1017 |
| Canyon country | 2000 | 865 | 1 | 864 | 0 | 0.001156 | g 249/2/2077, vm 234/25/1998 |
| Confluence | 0 | 872 | 415 | 457 | 0 | 0.475917 | g 50/50/50, vm 50/50/50 |
| Confluence | 22 | 865 | 226 | 639 | 0 | 0.261272 | g 67/49/84, vm 64/51/79 |
| Confluence | 250 | 897 | 45 | 847 | 5 | 0.050167 | g 99/20/291, vm 88/44/254 |
| Confluence | 1000 | 868 | 9 | 858 | 1 | 0.010369 | g 174/10/1045, vm 167/38/925 |
| Confluence | 2000 | 865 | 2 | 863 | 0 | 0.002312 | g 220/4/2040, vm 208/36/1897 |

Goal `cognition` per world: Orchards plasticity_updates_total 238,293
(hebbian 236,650, reward-modulated 1,643); Canyon 886,699 (865,372 /
21,327); Confluence 740,413 (735,353 / 5,060). `mutation_supply` events
attempted = applied (skipped 0): Orchards 108,030, Canyon 117,188,
Confluence 155,645.

### Legacy panel (goal `recruitment_paths`, current-source remeasurement)

`version` `recruitment-paths-v1`, `supply` `legacy` ("legacy per-birth
supply (per_unit_supply_enabled forced false)"), `sizes` {batches 4,
lineages 8, discovery 32, followup 16}, `total_proposals` 82,944 (27 arms ×
3,072; T17.F02 82,944), 864 lineages = 864 class assignments, 171 pairs.
These are current-source numbers at `cdc4674b`; the historical F06 numbers
(prepared discovery 17–18/32, unprepared 0/32) live in
[`t13-f06-recruitment-and-retention-qualification.md`](t13-f06-recruitment-and-retention-qualification.md)
and are not byte-comparable after `8263a8ee`. `ancestral_loss` column: every
`proposal_discovery` and `retained_discovery` module reading in the arm
(229 discovery readings over the panel, all `ancestral_loss` 0,
`specialization.ancestral_loss` false, `specialization.bypass_loss` true).

| # | Form | Policy | Task | Proposal disc. | Retained disc. | Retained useful | eligibility / local_edit / expression / specialized | bypass_only | Classes | eligible_site_fraction | ancestral_loss on discoveries | destination_kinds vm / graph_stateful / graph_pure_no_effect / graph_pure_with_effect |
| ---: | --- | --- | --- | --- | --- | --- | --- | ---: | --- | --- | --- | --- |
| 0 | graph_blank | Drift | A | 0/32 | 0/32 | 0/32 | 32 / 21 / 26 / 0 | 0 | no_benefit 20, no_edit 11, no_expression 1 | 46/98 = 0.4694 | — | 37 / 3 / 42 / 12 |
| 1 | graph_blank | Selection | A | 0/32 | 0/32 | 0/32 | 32 / 21 / 26 / 0 | 0 | no_benefit 20, no_edit 11, no_expression 1 | 46/98 = 0.4694 | — | 37 / 3 / 42 / 12 |
| 2 | graph_copy | Drift | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 56/101 = 0.5545 | — | 37 / 6 / 17 / 36 |
| 3 | graph_copy | Selection | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 56/101 = 0.5545 | — | 37 / 6 / 17 / 36 |
| 4 | graph_split | Drift | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 56/101 = 0.5545 | — | 37 / 9 / 17 / 33 |
| 5 | graph_split | Selection | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 56/101 = 0.5545 | — | 37 / 9 / 17 / 33 |
| 6 | vm_blank | Drift | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 54/97 = 0.5567 | — | 58 / 4 / 26 / 5 |
| 7 | vm_blank | Selection | A | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 54/97 = 0.5567 | — | 58 / 4 / 26 / 5 |
| 8 | vm_copy | Drift | A | 0/32 | 0/32 | 0/32 | 32 / 25 / 27 / 0 | 0 | no_benefit 25, no_edit 7 | 58/100 = 0.5800 | — | 63 / 3 / 21 / 8 |
| 9 | vm_copy | Selection | A | 0/32 | 0/32 | 0/32 | 32 / 25 / 27 / 0 | 0 | no_benefit 25, no_edit 7 | 58/100 = 0.5800 | — | 63 / 3 / 21 / 8 |
| 10 | graph_unprepared | Drift | B | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 55/100 = 0.5500 | — | 35 / 7 / 17 / 36 |
| 11 | graph_unprepared | Selection | B | 0/32 | 0/32 | 0/32 | 32 / 23 / 19 / 0 | 0 | no_benefit 18, no_edit 9, no_expression 5 | 49/103 = 0.4757 | — | 34 / 6 / 17 / 40 |
| 12 | graph_prepared | Drift | B | 19/32 | 15/32 | 7/32 | 32 / 24 / 27 / 0 | 15 | no_benefit 24, no_edit 8 | 55/100 = 0.5500 | 34 readings, all 0 | 35 / 7 / 17 / 36 |
| 13 | graph_prepared | Selection | B | 20/32 | 20/32 | 20/32 | 32 / 28 / 29 / 0 | 20 | no_benefit 28, no_edit 4 | 60/101 = 0.5941 | 40 readings, all 0 | 35 / 9 / 14 / 39 |
| 14 | vm_unprepared | Drift | B | 0/32 | 0/32 | 0/32 | 32 / 24 / 27 / 0 | 0 | no_benefit 24, no_edit 8 | 61/101 = 0.6040 | — | 64 / 4 / 21 / 6 |
| 15 | vm_unprepared | Selection | B | 0/32 | 0/32 | 0/32 | 32 / 21 / 17 / 0 | 0 | no_benefit 16, no_edit 11, no_expression 5 | 53/101 = 0.5248 | — | 59 / 4 / 20 / 9 |
| 16 | vm_prepared | Drift | B | 19/32 | 16/32 | 9/32 | 32 / 24 / 27 / 0 | 16 | no_benefit 24, no_edit 8 | 61/101 = 0.6040 | 35 readings, all 0 | 64 / 4 / 21 / 6 |
| 17 | vm_prepared | Selection | B | 19/32 | 19/32 | 18/32 | 32 / 26 / 28 / 0 | 20 | no_benefit 26, no_edit 6 | 65/100 = 0.6500 | 38 readings, all 0 | 63 / 5 / 20 / 6 |
| 18 | graph_blank | CostSelection | A | 0/32 | 0/32 | 0/32 | 32 / 8 / 22 / 0 | 0 | no_benefit 8, no_edit 24 | 14/32 = 0.4375 | — | 0 / 0 / 25 / 0 |
| 19 | graph_copy | CostSelection | A | 0/32 | 0/32 | 0/32 | 32 / 18 / 17 / 0 | 0 | no_benefit 16, no_edit 14, no_expression 2 | 26/32 = 0.8125 | — | 0 / 6 / 0 / 20 |
| 20 | graph_split | CostSelection | A | 0/32 | 0/32 | 0/32 | 32 / 9 / 2 / 0 | 0 | no_benefit 2, no_edit 23, no_expression 7 | 19/32 = 0.5938 | — | 0 / 2 / 1 / 22 |
| 21 | vm_blank | CostSelection | A | 0/32 | 0/32 | 0/32 | 32 / 2 / 18 / 0 | 0 | no_benefit 2, no_edit 30 | 9/32 = 0.2812 | — | 26 / 0 / 0 / 0 |
| 22 | vm_copy | CostSelection | A | 0/32 | 0/32 | 0/32 | 32 / 12 / 18 / 0 | 0 | no_benefit 11, no_edit 20, no_expression 1 | 18/32 = 0.5625 | — | 26 / 0 / 0 / 0 |
| 23 | graph_unprepared | CostSelection | B | 0/32 | 0/32 | 0/32 | 32 / 4 / 1 / 0 | 0 | no_edit 28, no_expression 4 | 14/33 = 0.4242 | — | 0 / 3 / 1 / 22 |
| 24 | graph_prepared | CostSelection | B | 21/32 | 21/32 | 21/32 | 32 / 21 / 21 / 0 | 21 | no_benefit 20, no_edit 11, no_expression 1 | 26/32 = 0.8125 | 42 readings, all 0 | 0 / 5 / 0 / 23 |
| 25 | vm_unprepared | CostSelection | B | 0/32 | 0/32 | 0/32 | 32 / 1 / 0 / 0 | 0 | no_edit 31, no_expression 1 | 9/32 = 0.2812 | — | 24 / 0 / 0 / 0 |
| 26 | vm_prepared | CostSelection | B | 20/32 | 20/32 | 20/32 | 32 / 9 / 21 / 0 | 21 | no_benefit 9, no_edit 23 | 13/32 = 0.4062 | 40 readings, all 0 | 29 / 0 / 0 / 0 |

Legacy-panel `pairs`: 171; 5,283 pair-lineages carry a first mutation,
parent or RNG divergence and 742 a nonzero `proposal_discovery_difference`
(paired arms diverge after genotype/site divergence by the `rng_control`
note; no added-draw intervention).

## Qualified-path payload readings (`QualifiedPath::payload_readings()`)

One row per step of each qualified construction path (seven of the nine
fixed forms; `graph_blank` and `vm_blank` carry a growth gap and are not
qualified), read against the path's start
on its task, then the two verbatim-copy negative controls (a `CopyNode` of
the founder's module, activated as entry, on each backend). Produced by
`cargo test -p v3-core recruitment_paths_print_payload_readings_table --
--ignored --nocapture` at the committed source (the test prints; it asserts
nothing). `ancestral_loss` = score(step) − score(birth payload on the same
route); a step whose payload equals birth reads 0 by identity. Every path's
useful last step diverges its payload with `ancestral_loss` ≥ 4 and
incumbents preserved; every earlier step reads 0; both verbatim copies read
`bypass_loss` 4 but `ancestral_loss` 0, so the ancestral test — not the
bypass test — is what separates copied computation from specialization.

| Form | Backend | Task | Step | Score | payload_changed | bypass_loss | ancestral_loss | task_live / score_gain / bypass_loss / ancestral_loss / incumbents_preserved | holds |
| --- | --- | --- | --- | ---: | --- | ---: | ---: | --- | --- |
| graph_copy | Graph | A | gate_edge_added | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_copy | Graph | A | activated | 8 | true | 4 | 4 | true / true / true / true / true | true |
| graph_split | Graph | A | gate_edge_added | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_split | Graph | A | activated | 8 | true | 4 | 4 | true / true / true / true / true | true |
| vm_copy | Vm | A | leading_halt_removed | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_copy | Vm | A | activated | 8 | true | 4 | 4 | true / true / true / true / true | true |
| graph_unprepared | Graph | B | ring_added | 4 | false | 0 | 0 | true / false / false / false / true | false |
| graph_unprepared | Graph | B | cue_edge_retargeted | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_unprepared | Graph | B | direction_node | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_unprepared | Graph | B | direction_doubled | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_unprepared | Graph | B | direction_read | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_unprepared | Graph | B | activated | 8 | true | 4 | 6 | true / true / true / true / true | true |
| vm_unprepared | Vm | B | ring_added | 4 | false | 0 | 0 | true / false / false / false / true | false |
| vm_unprepared | Vm | B | read_ref_idx_1 | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_unprepared | Vm | B | read_sub_idx_1 | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_unprepared | Vm | B | read_sub_idx_2 | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_unprepared | Vm | B | direction_doubled | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_unprepared | Vm | B | activated | 8 | true | 4 | 6 | true / true / true / true / true | true |
| graph_detour | Graph | A | cue_added | 4 | false | 0 | 0 | true / false / false / false / true | false |
| graph_detour | Graph | A | slot_emits_move | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_detour | Graph | A | direction_node | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_detour | Graph | A | direction_doubled | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_detour | Graph | A | direction_read | 4 | true | 0 | 0 | true / false / false / false / true | false |
| graph_detour | Graph | A | gate_edge_added | 8 | true | 4 | 4 | true / true / true / true / true | true |
| vm_detour | Vm | A | cue_added | 4 | false | 0 | 0 | true / false / false / false / true | false |
| vm_detour | Vm | A | read_cue | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_detour | Vm | A | double_to_east | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_detour | Vm | A | write_direction | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_detour | Vm | A | skip_when_zero | 4 | true | 0 | 0 | true / false / false / false / true | false |
| vm_detour | Vm | A | push_move | 8 | true | 4 | 4 | true / true / true / true / true | true |
| verbatim_copy (control) | Graph | A | verbatim | 8 | false | 4 | 0 | true / false / true / false / true | false |
| verbatim_copy (control) | Vm | A | verbatim | 8 | false | 4 | 0 | true / false / true / false / true | false |

## Predeclaration table against the measurements (verdict only)

| Indicator | Predeclared | Measured | Verdict |
| --- | --- | --- | --- |
| Six normalized counters, both profiles | differ from T17.F02 by the two default changes only; pinned identity hash unchanged | gate: all six `ok` vs both references, `severe` false; goal: all six `ok`, `severe` false, largest `plasticity_updates` −46.011157% (flag level, under the 50% severe line); identity-hash check is the reviewer's (`baseline_worlds` unchanged per the Verification item, not re-measured here) | recorded, no severe flag |
| Founder neighborhood, drift walk, evolved trajectories, diversity and cognition | same attribution; no floor | founder rows equal T17.F02 in every world and the gate; evolved rows, drift (2,000: 0.001 / 0.0005 / 0.001 vs 0.005, no gate), lineage entropy and clade counts moved (tables above) | recorded |
| Founder and evolved `mesh_execution` | existing keys unchanged; `route_destination_varies` never true where position false; founder readings equal | founder false/false in gate and all three worlds; raw goal tally 0 destination-true-with-position-false out of 39 blocks; existing keys present with the T17.F02 shapes | met |
| Legacy panel, unprepared arms | 0/32 proposal and retained discovery | 21 unprepared arms (7 forms × 3 selectors) all 0/32 / 0/32 | met |
| Legacy panel, prepared arms | discovery near F06's 17–18/32; every discovery `ancestral_loss` 0, `bypass_only` | 19–21/32 proposal discovery (arms 12, 13, 16, 17, 24, 26); 229 discovery readings all `ancestral_loss` 0; `specialized` 0, `bypass_only` 15/20/16/20/21/21 | recorded (discovery above the F06 range; no floor) |
| S0 unprepared arms (predeclared as 18 of 27; 21 = 7 forms × 3 selectors) | no floor; ladder counts and `eligible_site_fraction` are the result; exposure reported | 19 of 21 arms `specialized` 0; arm 9 (`vm_copy`/Selection) and arm 22 (`vm_copy`/CostSelection) each carry 1 `retained` lineage (at 16/64/256); arm 8 one `loss_not_selected`; `specialized_after_window` 1 in arms 8, 9 and 22; `eligible_site_fraction` 0.53–0.98; requested per birth 0.055–0.466 (tables above) | recorded |
| S0 prepared arms | expression as F06; specialization 0 unless a payload edit diverges | expression 64/64 in the Drift/Selection arms (12, 13, 16, 17), 43/64 and 45/64 under CostSelection (24, 26); `specialized` 0; `specialized_after_window` 1 in arms 13 and 16; discovery 42–50/64; `bypass_only` 33–50 | recorded |
| S0 Drift arms | genome size may grow; p50/p90 reported; pilot projection is the cap check | gen-512 p50 52.5–187, p90 128–478 (max 4,658 in arm 17); projection inside the launch margin; run complete, 1.342 GiB, 162.5 s | met |
| Negative controls | verbatim copies 0 `ancestral_loss` | both verbatim-copy controls read `ancestral_loss` 0 (`bypass_loss` 4), every qualified path's useful last step reads ≥ 4 with incumbents preserved (per-step table above); panel: 0 nonzero `ancestral_loss` among 229 discoveries | recorded |
| Wall per creature-tick, both profiles | no direction | gate −3.35% / −17.68% `ok`; goal −6.68% `ok` | recorded |
| Committed goal summary bytes | grows by the new keys; S0 summary under 4 MB, per-lineage rows | goal summary 7,632,294 (T17.F02 6,873,055, +759,239); S0 summary 2,405,314 with 1,728 lineage rows and no proposal rows | met |
| Caps | goal experiment < 120 s; goal profile < 15 min; S0 < 2 h and 2 GiB; pilot < 10 min | recruitment 11.5 s; goal 460.4 s CLI / 494 s outer; S0 162.5 s and 1.342 GiB; pilot 6.8 s CLI (140 s outer with the build) | met |

## Deferred (final review, 2026-09-20; spec-owner rulings)

Code findings the review ruled record-only for this feature; the spec's
Notes for AI Agents point here. Each is a cleanup for the feature that next
touches the site (F08–F10 extend the chain facts), not a behavior defect.

- Deferred: chain facts (retained discovery, horizons, ladder, classification)
  are restated at six sites — `records.rs` `Lineage`, `CompactLineage`,
  `ChainFacts`, `LineageFacts::assemble`, `v3-cli/src/recruitment.rs`
  `LineageRow::of`, `v3-cli/src/bench/artifacts.rs` projection; refactor to
  one owned chain-facts struct before F08–F10 extend them.
- Deferred: `EditSurface::of` (`recruitment.rs` ~136) matches a wildcard
  and falls back on `operator.domain()`; match the topology operators
  exhaustively so a new operator is a compile error, not a silent `Other`.
- Deferred: `class_key` (`records.rs` ~769) hand-rolls the snake_case that
  `LineageClass`'s serde derive already produces.
- Deferred: `event.outcome.starts_with("Applied")` is a string pattern on
  an outcome that is an enum upstream (`experiment.rs` ~605, `records.rs`
  ~968); carry the enum or a boolean instead.
- Deferred: the streamed S0 JSON is built by popping braces from serialized
  text (`v3-cli/src/recruitment.rs` ~384 head, ~402 footer); a
  `serde_json::Serializer` sequence or an explicit envelope struct removes
  the text surgery.
- Deferred: exit status 3 on an incomplete run is asserted by no
  spawned-binary test (the in-process test checks `incomplete` only).
- Deferred: the raw file may exceed `--byte-cap` by up to `threads − 1`
  lineage records (cap checked after each append; recorded in
  `docs/benchmark-artifacts.md`).
- Deferred: spec dates are UTC (the 2026-09-20 run dates fall on the
  2026-09-19 local evening).

## Mutation gate (roadmap-mutation-specialist, 2026-09-20)

| Run | Command | Result |
| --- | --- | --- |
| Attempt 1 (no gate evidence) | `MUTANTS_ITERATE=0 make rust-mutants` at 922445d1 | `cargo mutants` exit 4: the unmutated baseline failed on `recruitment::tests::default_paths_follow_the_feature_and_pilot_naming` (`v3-cli`), which resolved default output paths from the process working directory; cargo-mutants tests a plain copy of the tree that is not a Git checkout. Zero mutants tested. Test-only fix: the test now builds its own throwaway Git checkout (`git_checkout` fixture in `crates/v3-cli/src/recruitment.rs`). |
| Fresh run | `MUTANTS_ITERATE=0 make rust-mutants`, diff against 3806dc91 | `387 mutants tested in 55m: 74 missed, 251 caught, 62 unviable`; `run-mode.txt` = `fresh`; output `~/.local/share/petri-tools/mutants/t13-f07/mutants.out` (`missed.txt` 74 lines, `timeout.txt` empty); the measured diff includes the attempt-1 test fix. |
| Incremental pass 1 (not closure evidence) | `MUTANTS_OUT=~/.local/share/petri-tools/mutants/t13-f07-triage MUTANTS_ITERATE=1 make rust-mutants` (a copy of the fresh output so the recorded path stays untouched) | `74 mutants tested in 13m: 3 missed, 71 caught`: the two equivalents (rows 15, 30) and `records.rs:229` (row 65, precedence). |
| Incremental pass 2 (not closure evidence) | same, after the gate-only/sink-only fixtures | `3 mutants tested in 2m: 2 missed, 1 caught`: the two equivalents remain; `timeout.txt` empty. |
| Verification after triage | `cargo test -p v3-core recruitment` (92 + 3 passed, 1 ignored), `cargo test -p v3-cli` (all binaries), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `make roadmap-check` | all exit 0; `missed.txt` (74) and `timeout.txt` (empty) at the recorded path unchanged since the fresh run. |

Survivor triage. Production code, test selection and tool configuration are
unchanged; every kill is a new or strengthened test, so no second fresh run
is required. Tests live in `crates/v3-core/src/neighborhood/recruitment_paths/tests.rs`
(`rp::`), the inline `experiment::tests` module (`ex::`),
`crates/v3-core/src/neighborhood/recruitment/tests.rs` (`rec::`),
`crates/v3-cli/src/recruitment.rs` tests (`cli::`) and
`crates/v3-cli/tests/cli.rs` (`bin::`).

| # | Survivor (`missed.txt`) | Resolution | Test |
| --- | --- | --- | --- |
| 1 | `v3-cli/src/main.rs:413:5: replace run_recruitment with ()` | killed | `bin::recruitment_cli_passes_every_option_into_the_written_summary` (spawned binary; exit 3, both artifacts) |
| 2 | `main.rs:421:9: delete field pilot from Options in run_recruitment` | killed | same (`summary.pilot`) |
| 3 | `main.rs:422:9: delete field threads` | killed | same (`summary.threads == 1`) |
| 4 | `main.rs:423:9: delete field wall_cap` | killed | same (`--wall-cap-secs 0` stops with `wall_cap`, 0 lineages) |
| 5 | `main.rs:424:9: delete field byte_cap` | killed | same (`--byte-cap 1` stops with `byte_cap`, 1 lineage) |
| 6 | `main.rs:425:9: delete field replay_check` | killed | same (`replay_check` object present) |
| 7 | `main.rs:426:9: delete field raw` | killed | same (`--out` path written and recorded) |
| 8 | `main.rs:427:9: delete field summary` | killed | same (`--summary-out` path written) |
| 9 | `main.rs:428:9: delete field source_revision` | killed | same (`source_revision == git rev-parse HEAD` of the fixture checkout) |
| 10 | `v3-cli/src/recruitment.rs:32:37: replace * with +` (`DEFAULT_BYTE_CAP`) | killed | `cli::default_caps_are_two_hours_and_two_gibibytes` |
| 11 | `recruitment.rs:204:9: replace HashingWriter::flush with Ok(())` | killed | `cli::hashing_writer_forwards_flush_to_its_inner_writer` |
| 12 | `recruitment.rs:244:5: replace io_error with String::new()` | killed | `cli::an_unwritable_raw_path_is_named_in_the_error` |
| 13 | `recruitment.rs:244:5: replace io_error with "xyzzy".into()` | killed | same |
| 14 | `recruitment.rs:361:51: delete ! in open_stream` | killed | `cli::a_raw_path_in_a_missing_directory_is_created` |
| 15 | `v3-core/src/neighborhood/recruitment.rs:127:13: delete match arm TopologyAddNode … InputRefRawFieldMutation in EditSurface::of` | equivalent | Every operator in the deleted arm reports `MutationDomain::Topology` or `MutationDomain::InputRef` (`mutation/types/mod.rs` `domain`), which the wildcard arm maps to `EditSurface::Other` as well. |
| 16 | `recruitment.rs:195:55: replace + with - in Exposure::discarded` | killed | `rec::exposure_totals_sum_every_surface` |
| 17 | `experiment.rs:74:28: replace > with >= in uses` (`score_gain`) | killed | `rp::recruitment_paths_reduced_run_ladders_agree_with_their_records` (generation 0 reads no score gain against itself) |
| 18 | `experiment.rs:138:46: replace - with + in uses` (`ancestral_loss`) | killed | `rp::recruitment_paths_known_specializing_lineage_classifies_retained_at_the_primary_horizon` (`ancestral_loss == Some(4)`, `score_loss == 4`) |
| 19 | `experiment.rs:138:46: replace - with / in uses` | killed | same |
| 20 | `experiment.rs:402:13: replace && with \|\| in recruit_reading` | killed | `ex::recruit_reading_finds_the_discovered_module_among_same_depth_siblings` |
| 21 | `experiment.rs:455:13: replace && with \|\| in legacy_retention` (`is_present`) | killed | `rp::recruitment_paths_known_deleting_lineage_reads_deleted_retention` |
| 22 | `experiment.rs:454:13: replace && with \|\| in legacy_retention` (`created_depth`) | killed | same |
| 23 | `experiment.rs:453:21: replace == with != in legacy_retention` (`node`) | killed | same |
| 24 | `experiment.rs:510:19: replace == with != in specialized_horizons` | killed | `rp::recruitment_paths_known_specializing_lineage_is_censored_before_the_primary_horizon` |
| 25 | `experiment.rs:565:33: delete ! in lineage` (`eligibility = is_empty()` at generation 0) | killed | `rp::recruitment_paths_known_lineage_keeps_generation_zero_eligibility_once_its_cohort_goes_unreachable` |
| 26 | `experiment.rs:606:21: replace && with \|\| in lineage` (`local_edit`) | killed | `rp::recruitment_paths_reduced_run_ladders_agree_with_their_records` (a local edit names a cohort module) |
| 27 | `experiment.rs:610:39: replace \|= with &= in lineage` (`bypass_only`) | killed | same (`bypass_only` equals the retained zero-ancestral-loss reading) |
| 28 | `experiment.rs:614:53: replace == with != in lineage` (`ancestral_loss == Some(0)`) | killed | same |
| 29 | `experiment.rs:646:35: replace \|= with &= in lineage` (`eligibility` per generation) | killed | `rp::recruitment_paths_known_lineage_keeps_generation_zero_eligibility_once_its_cohort_goes_unreachable` |
| 30 | `experiment.rs:646:38: delete ! in lineage` (`eligibility \|= is_empty()`) | equivalent | Every fixed start has its authored module 2 reachable at generation 0 (`ex::every_start_reaches_but_does_not_express_its_cohort_module_at_generation_zero`), so `eligibility` is already true before the per-generation update, which can only raise it. |
| 31 | `experiment.rs:678:46: replace && with \|\| in reachable_cohort` | killed | `rp::recruitment_paths_reduced_run_ladders_agree_with_their_records` (founder-node edits do not count as local edits) |
| 32 | `experiment.rs:686:5: replace expressed with true` | killed | `ex::every_start_reaches_but_does_not_express_its_cohort_module_at_generation_zero`; `rp::…ladders_agree…` (`expression` equals a dispatched cohort module) |
| 33 | `experiment.rs:688:53: replace && with \|\| in expressed` | killed | same |
| 34 | `experiment.rs:688:81: replace >= with < in expressed` | killed | same |
| 35 | `experiment.rs:713:40: replace \|= with &= in note_proposal_discovery` | killed | `rp::…known_specializing_lineage_classifies_retained…` (`proposal_specialized`) |
| 36 | `experiment.rs:775:24: replace += with *= in transitions` (`local_edit`) | killed | `rp::recruitment_paths_transitions_count_every_ladder_flag_across_lineages` |
| 37 | `experiment.rs:776:24: replace += with *= in transitions` (`expression`) | killed | same |
| 38 | `experiment.rs:777:25: replace += with -= in transitions` (`specialized`) | killed | same |
| 39 | `experiment.rs:777:25: replace += with *= in transitions` | killed | same |
| 40 | `experiment.rs:778:43: replace += with -= in transitions` (`proposal_specialized_lineages`) | killed | same |
| 41 | `experiment.rs:778:43: replace += with *= in transitions` | killed | same |
| 42 | `experiment.rs:779:35: replace += with -= in transitions` (`specialized_proposals`) | killed | same |
| 43 | `experiment.rs:779:35: replace += with *= in transitions` | killed | same |
| 44 | `experiment.rs:780:25: replace += with *= in transitions` (`bypass_only`) | killed | same |
| 45 | `experiment.rs:782:57: replace += with -= in transitions` (`retained_at`) | killed | same |
| 46 | `experiment.rs:782:57: replace += with *= in transitions` | killed | same |
| 47 | `experiment.rs:783:35: replace == with != in transitions` (`Retained`) | killed | same |
| 48 | `experiment.rs:788:22: replace += with *= in transitions` (`applicable.0`) | killed | same (`eligible_site_fraction == estimate(3, 6)`) |
| 49 | `experiment.rs:933:9: replace ReplayCheck::merge with ()` | killed | `rp::recruitment_paths_replay_check_merge_sums_counts_and_keeps_the_first_mismatch` |
| 50 | `experiment.rs:933:24: replace += with -= in ReplayCheck::merge` | killed | same |
| 51 | `experiment.rs:933:24: replace += with *= in ReplayCheck::merge` | killed | same |
| 52 | `experiment.rs:934:22: replace += with -= in ReplayCheck::merge` | killed | same |
| 53 | `experiment.rs:934:22: replace += with *= in ReplayCheck::merge` | killed | same |
| 54 | `experiment.rs:983:9: replace Assay::starts with Vec::leak(Vec::new())` | killed | `rp::recruitment_paths_assay_exposes_the_nine_starts_in_order` |
| 55 | `qualification.rs:172:26: replace - with + in payload_reading` (`ancestral_loss`) | killed | `rp::recruitment_paths_qualified_payload_readings_match_the_recorded_table` |
| 56 | `qualification.rs:172:26: replace - with / in payload_reading` | killed | same |
| 57 | `qualification.rs:176:40: replace - with + in payload_reading` (`bypass_loss`) | killed | same |
| 58 | `qualification.rs:176:40: replace - with / in payload_reading` | killed | same |
| 59 | `qualification.rs:185:31: replace > with >= in payload_reading` (`score_gain`) | killed | same |
| 60 | `records.rs:82:22: replace > with >= in Sizes::fits` (`batches`) | killed | `rp::recruitment_paths_sizes_reject_every_zero_dimension_and_name_the_supply_rules` |
| 61 | `records.rs:83:30: replace > with >= in Sizes::fits` (`lineages`) | killed | same |
| 62 | `records.rs:84:31: replace > with >= in Sizes::fits` (`discovery`) | killed | same |
| 63 | `records.rs:118:9: replace Supply::rule with ""` | killed | same |
| 64 | `records.rs:118:9: replace Supply::rule with "xyzzy"` | killed | same |
| 65 | `records.rs:229:13: replace \|\| with && in DestinationKind::of` (`wired \|\| (gate && sinks)` by precedence) | killed | `rp::recruitment_paths_destination_kind_reads_a_wired_action_slot_alone_as_an_effect` (gate-only and sink-only fixtures; survived incremental pass 1 with the bank-only fixture, killed in pass 2) |
| 66 | `records.rs:228:13: replace \|\| with && in DestinationKind::of` | killed | same |
| 67 | `records.rs:261:29: replace += with *= in DestinationKindCounts::merge` | killed | `rp::recruitment_paths_destination_kind_counts_merge_asymmetric_pools` |
| 68 | `records.rs:375:9: replace TaskReading::preserves_correct_scenes with true` | killed | `rp::recruitment_paths_incumbents_are_preserved_only_when_no_correct_scene_is_lost` |
| 69 | `records.rs:384:17: delete ! in TaskReading::preserves_correct_scenes` | killed | same |
| 70 | `records.rs:564:13: replace && with \|\| in Specialization::holds` | killed | `rp::recruitment_paths_specialization_holds_only_with_every_component` |
| 71 | `records.rs:565:13: replace && with \|\| in Specialization::holds` | killed | same |
| 72 | `records.rs:770:5: replace class_key with "xyzzy".into()` | killed | `rp::recruitment_paths_class_keys_name_every_class` |
| 73 | `records.rs:1112:39: replace += with -= in LineageFacts::assemble` (`specialized_proposals`) | killed | `rp::…known_specializing_lineage_classifies_retained…` (`specialized_proposals == 128`) |
| 74 | `records.rs:1112:39: replace += with *= in LineageFacts::assemble` | killed | same |

The spawned-binary test in row 1 also covers the review's deferred
"exit status 3 asserted by no spawned-binary test" bullet above.
