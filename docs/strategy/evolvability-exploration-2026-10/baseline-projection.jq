# Deterministic summary projection of the reference arm, frozen at cycle 0
# (2026-10-01) and used unchanged after: calibration with thresholds, seeds,
# resolved config digest, the reference arm's results, fidelity and its ladder
# verdicts. Timing, provenance revision, dirty state and other arms excluded.
{calibration: .calibration,
 seeds: .provenance.seeds,
 config_digest: .provenance.config_digest,
 reference: [.arms[] | select(.role == "reference")],
 fidelity: .fidelity,
 ladder_reference: ((.ladder.arms // []) | map(select(.role == "reference")))}
