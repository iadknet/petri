# Compact keep-list for a lab summary cited by the results note: provenance,
# calibration point and threshold, per-arm reach and final bests, ladder
# verdicts and per-replicate rung statuses.
{source: {revision: .provenance.git_revision, dirty: .provenance.dirty,
          config_digest: .provenance.config_digest, overlays: .provenance.overlays,
          arena_sha256: .provenance.arena.sha256, seeds: .provenance.seeds, sizes: .provenance.sizes},
 calibration: {verdict: .calibration.verdict, selected: .calibration.selected,
               reach_threshold: .calibration.reach_threshold},
 arms: [.arms[] | {name, role, policy, reached_fraction, wilson_95,
        final_best: [.replicates[].final_best], generation_to_threshold: [.replicates[].generation_to_threshold]}],
 ladder: [(.ladder.arms // [])[] | {name, verdict,
          statuses: [.replicates[] | [.statuses[] | "\(.rung):\(.status) \(.successes)/\(.trials)"]]}],
 exit_code, incomplete, wall_seconds: .timing.wall_seconds}
