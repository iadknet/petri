# T03.F12 long Canyon run, goal seed 22

Command and timing: `run.sh`, `timing.txt` (1,604 s wall of a 14,400 s budget; exit status 0).
Source revision f380899e, release binary, `scripts/bench-wait`, alone.

- Raw file: `.bench-artifacts/t03-f12-genome-length-dependent-reproductive-time/canyon/canyon.ndjson` in the main checkout (ignored), 332,377 bytes, SHA-256 `b674b57905550bc1c3295229b5af622df44f31a6b51c24ba89a1cfb1f2a08c14`; its `canyon-config.json` is 4,661 bytes.
- 80 `tick_sample` events (every 500 ticks, 500 to 40,000), then `run_completed` with `ticks_executed` 40000, `final_population` 1122.
- Decisive check: population above zero at every sample; the minimum over the 80 samples is 1,110 (tick 2,500). Extinction never occurred.
- Founder `mean_genome_size` is 97.

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

Descriptive only (different substrate, no same-binary control): `mean_genome_size` at 20,000 is 6.91x the founder against E2's 16x; the largest value in the table is 15.37x at 35,000.
