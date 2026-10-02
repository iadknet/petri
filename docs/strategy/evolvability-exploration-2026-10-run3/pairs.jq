# Paired contrast of arm $arm against the reference in one summary (run 3,
# corrected after closing review 2). Per pair (replicate index): reach of
# each, the reference's first non-pass rung and status, the arm's status at
# that rung, and the final-best gap.
# - A rung departure (run 3 plan rule 8) needs the reference to read `fail`
#   with adequacy at its stall rung and the arm to read `pass` with adequacy
#   there; reach and rung movement are separate effects, so an arm that
#   reached but reads `fail` or `inconclusive` at that rung is not a departure.
# - A reference replicate that reached has no stall rung and is not used.
# - A used pair is unusable when the reference's read there is not `fail` or
#   the arm's read there is neither `pass` nor `fail`.
. as $s
| ([$s.arms[] | select(.role == "reference")][0].replicates) as $ref
| ([$s.arms[] | select(.name == $arm)][0].replicates) as $arm_r
| ([$s.ladder.arms[] | select(.role == "reference")][0].replicates) as $ref_l
| ([$s.ladder.arms[] | select(.name == $arm)][0].replicates) as $arm_l
| [range(0; $ref | length) as $i
   | ($ref_l[$i].first_not_pass) as $rung
   | ([$ref_l[$i].statuses[] | select(.rung == $rung)][0].status) as $ref_status
   | ([$arm_l[$i].statuses[] | select(.rung == $rung)][0].status) as $arm_status
   | (($ref[$i].reached | not) and $rung != null) as $used
   | ($used and ($ref_status != "fail" or ($arm_status != "pass" and $arm_status != "fail"))) as $unusable
   | {replicate: $i,
      ref_reached: $ref[$i].reached, arm_reached: $arm_r[$i].reached,
      ref_rung: $rung, ref_status: $ref_status, arm_status_at_ref_rung: $arm_status,
      arm_first_not_pass: $arm_l[$i].first_not_pass,
      final_best_gap: ($arm_r[$i].final_best - $ref[$i].final_best),
      arm_only_reach: ($arm_r[$i].reached and ($ref[$i].reached | not)),
      ref_only_reach: ($ref[$i].reached and ($arm_r[$i].reached | not)),
      rung_used: $used,
      unusable: $unusable,
      departure: ($used and ($unusable | not) and $ref_status == "fail" and $arm_status == "pass")}]
