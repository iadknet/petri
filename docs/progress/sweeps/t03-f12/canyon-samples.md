# T03.F12 long Canyon run, goal seed 22

Command and timing: `run.sh`, `timing.txt`. Shipped form B (`f380899e`): 1,604 s wall of a 14,400 s budget, exit status
0. The executed retreat, form A (`0436dbf6`, reverted by user decision 2026-10-08): 1,669 s, exit status 0. The raw files
sit in the main checkout's ignored `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/`: form B
directly in `canyon/`, form A in `canyon/form-a/`.

Founder `mean_genome_size` is 97 in both runs.

## Form B (pre-birth gestation, shipped), source revision f380899e

Release binary, `scripts/bench-wait`, alone, telemetry on (the CLI default, `petri-telemetry` stack up).

Raw file (ignored): `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/canyon.ndjson` in the
main checkout, 332,377 bytes, SHA-256 `b674b57905550bc1c3295229b5af622df44f31a6b51c24ba89a1cfb1f2a08c14`; its
`canyon-config.json` is 4,661 bytes.
80 `tick_sample` events, `final_population` 1122; minimum over the 80 samples 1,110 (tick 2,500); extinction never
occurred.

| tick | population | mean_genome_size | x founder | mean_generation | mean_mesh_nodes |
| --- | --- | --- | --- | --- | --- |
| 2500 | 1110 | 159.53 | 1.64 | 56.53 | 5.56 |
| 5000 | 2225 | 179.97 | 1.86 | 109.44 | 7.06 |
| 7500 | 3170 | 221.35 | 2.28 | 169.71 | 12.31 |
| 10000 | 2081 | 286.82 | 2.96 | 227.99 | 15.74 |
| 12500 | 2861 | 333.82 | 3.44 | 294.47 | 18.98 |
| 15000 | 2139 | 361.80 | 3.73 | 351.23 | 21.85 |
| 17500 | 1491 | 480.79 | 4.96 | 418.19 | 30.35 |
| 20000 | 1405 | 670.57 | 6.91 | 488.73 | 42.60 |
| 22500 | 1753 | 785.99 | 8.10 | 551.57 | 49.88 |
| 25000 | 1528 | 911.17 | 9.39 | 594.89 | 55.57 |
| 27500 | 1753 | 973.96 | 10.04 | 646.71 | 62.49 |
| 30000 | 1637 | 1212.49 | 12.50 | 689.30 | 77.50 |
| 32500 | 1505 | 1257.97 | 12.97 | 737.97 | 86.02 |
| 35000 | 1422 | 1490.79 | 15.37 | 779.45 | 98.19 |
| 37000 (first sample past 36,652) | 1286 | 1321.03 | 13.62 | 809.29 | 93.09 |
| 40000 | 1122 | 1288.87 | 13.29 | 858.43 | 96.10 |

Descriptive only (different substrate, no same-binary control): `mean_genome_size` at 20,000 is 6.91x the founder against
E2's 16x; the largest table value is 15.37x at 35,000.

## Form A (post-birth reproductive hold; executed retreat, reverted by user decision 2026-10-08), source revision 0436dbf6

Release binary, `scripts/bench-wait`, alone, telemetry on (the CLI default, `petri-telemetry` stack up).

- Raw file: `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/form-a/canyon.ndjson` in the main
  checkout (ignored), 330,750 bytes, SHA-256 `c6bc909d49bedddde5065adf6ae4a5cd2d871ac8919f9e74e82ac2574da68b6a`; its
  `canyon-config.json` is 4,669 bytes.
- 80 `tick_sample` events (every 500 ticks, 500 to 40,000), then `run_completed` with `ticks_executed` 40000,
  `final_population` 2726.
- Decisive check: population above zero at every sample; the minimum over the 80 samples is 689 (tick 3,500). Extinction
  never occurred.

| tick | population | mean_genome_size | x founder | mean_generation | mean_mesh_nodes |
| --- | --- | --- | --- | --- | --- |
| 2500 | 1302 | 119.56 | 1.23 | 65.19 | 4.36 |
| 5000 | 1047 | 146.19 | 1.51 | 129.60 | 7.65 |
| 7500 | 1622 | 177.56 | 1.83 | 154.95 | 9.54 |
| 10000 | 2146 | 233.63 | 2.41 | 202.75 | 13.85 |
| 12500 | 3312 | 334.38 | 3.45 | 270.70 | 19.75 |
| 15000 | 3048 | 394.61 | 4.07 | 333.69 | 23.52 |
| 17500 | 2115 | 415.43 | 4.28 | 379.74 | 26.28 |
| 20000 | 2729 | 422.80 | 4.36 | 416.08 | 27.16 |
| 22500 | 2069 | 465.19 | 4.80 | 484.76 | 30.47 |
| 25000 | 2931 | 487.56 | 5.03 | 500.69 | 34.32 |
| 27500 | 2218 | 576.24 | 5.94 | 564.57 | 40.38 |
| 30000 | 1611 | 716.65 | 7.39 | 606.49 | 49.71 |
| 32500 | 2250 | 847.41 | 8.74 | 642.16 | 62.13 |
| 35000 | 2574 | 1025.11 | 10.57 | 709.33 | 71.22 |
| 37000 (first sample past 36,652) | 2692 | 880.49 | 9.08 | 754.20 | 61.57 |
| 40000 | 2726 | 941.70 | 9.71 | 857.20 | 65.75 |

Descriptive only (different substrate, no same-binary control): `mean_genome_size` at 20,000 is 4.36x the founder against
E2's 16x; the largest sample value is 11.02x at 35,500.
