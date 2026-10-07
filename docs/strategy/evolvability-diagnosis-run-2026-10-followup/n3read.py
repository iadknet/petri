"""Diagnosis follow-up row N3: read the Confluence census (checkpoints 20,000
and 50,000) with its vote-delta blocks, aggregate the per-channel and
per-family readings of the selected cohort, check agreement with the funnel's
causal counts, and apply the row's predeclared classification (dilution,
cancellation among channels, subadditive interaction, neither, mixed).

usage: python3 n3read.py <census-confluence.ndjson> <samples-confluence.ndjson> <out.json>"""
import hashlib
import json
import statistics
import sys

TARGETS = ("AreaFoodSummary", "NeighborBarrierRing")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def rows(path):
    return [json.loads(l) for l in open(path) if l.strip()]


def med(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def family_causal(input_use):
    selected = next((c for c in input_use["cohorts"] if c["cohort"] == "selected"), None)
    out = {}
    if not selected:
        return out
    for line in selected["family_rows"]:
        parts = line.split(" ")
        if parts[0].split(":")[0] in TARGETS:
            out[parts[0]] = {"declared": parts[1], "connected": parts[2], "executed": parts[3], "causal": parts[4]}
    return out, selected.get("parents_evaluated")


def checkpoint_reading(r):
    deltas = r.get("vote_deltas") or []
    channels = [c for p in deltas for c in p["channels"]]
    families = [f for p in deltas for f in p["families"]]
    s = lambda c, k: c["stats"].get(k)
    delta_channels = [c for c in channels if s(c, "delta_passes") > 0]
    margined = [c for c in delta_channels if s(c, "margined_passes") > 0]
    reading = {
        "parents": len(deltas),
        "parents_with_executed_target_channels": sum(1 for p in deltas if p["executed_target_channels"] > 0),
        "executed_target_channels": len(channels),
        "channels_with_any_delta": len(delta_channels),
        "channels_with_margined_passes": len(margined),
        "per_channel": {
            "ratio_median_of_medians": med([s(c, "ratio_median") for c in margined]),
            "ratio_max": max((s(c, "ratio_max") or 0.0 for c in margined), default=None),
            "max_abs_delta_median": med([s(c, "max_abs_delta") for c in delta_channels]),
            "max_abs_delta_max": max((s(c, "max_abs_delta") for c in delta_channels), default=None),
            "within_kind_margin_median_of_medians": med([s(c, "within_kind_margin_median") for c in margined]),
            "decision_margin_median_of_medians": med([s(c, "decision_margin_median") for c in delta_channels]),
            "delta_passes_total": sum(s(c, "delta_passes") for c in channels),
            "margined_passes_total": sum(s(c, "margined_passes") for c in channels),
            "zero_margin_passes_total": sum(s(c, "zero_margin_passes") for c in channels),
            "singleton_kind_passes_total": sum(s(c, "singleton_kind_passes") for c in channels),
            "kindless_passes_total": sum(s(c, "kindless_passes") for c in channels),
            "delta_reaches_margin_passes_total": sum(s(c, "delta_reaches_margin_passes") for c in channels),
            "divergent_passes_total": sum(s(c, "divergent_passes") for c in channels),
            "unmatched_passes_total": sum(s(c, "unmatched_passes") for c in channels),
            "guarded_passes_total": sum(s(c, "guarded_passes") for c in channels),
            "no_commit_passes_total": sum(s(c, "no_commit_passes") for c in channels),
            "committed_differs_passes_total": sum(s(c, "committed_differs_passes") for c in channels),
            "channels_with_action_queue_change": sum(1 for c in channels if s(c, "scenes_actions_differ") > 0),
            # Unanticipated class: the committed action changes while no vote
            # moves, which the mesh allows only through the action-parameter
            # surface (VM-written parameters decoded at the commit).
            "channels_action_change_without_vote_delta": sum(
                1 for c in channels if s(c, "scenes_actions_differ") > 0 and s(c, "delta_passes") == 0),
            "channels_vote_delta_without_action_change": sum(
                1 for c in channels if s(c, "scenes_actions_differ") == 0 and s(c, "delta_passes") > 0),
            "channels_graph_consumers_only": sum(1 for c in channels if c["consumers_graph"] > 0 and c["consumers_vm"] == 0),
            "channels_vm_consumers_only": sum(1 for c in channels if c["consumers_vm"] > 0 and c["consumers_graph"] == 0),
            "channels_both_consumer_kinds": sum(1 for c in channels if c["consumers_vm"] > 0 and c["consumers_graph"] > 0),
            "channels_no_delta_no_change": sum(1 for c in channels if s(c, "scenes_actions_differ") == 0 and s(c, "delta_passes") == 0),
            "consumers_graph_median": med([c["consumers_graph"] for c in channels]),
            "consumers_graph_abs_weight_median": med([c["consumers_graph_abs_weight"] for c in channels]),
            "most_moved_kind_counts": {},
        },
        "per_family": {
            "families": len(families),
            "families_with_any_delta": sum(1 for f in families if f["stats"]["delta_passes"] > 0),
            "ratio_median_of_medians": med([f["stats"]["ratio_median"] for f in families if f["stats"]["margined_passes"] > 0]),
            "subadditivity_ratio_median": med([f["l1_subadditivity_ratio"] for f in families]),
            "additive_residual_ratio_median": med([f["additive_residual_ratio"] for f in families]),
            "families_with_opposing_signs_on_most_moved": sum(
                1 for f in families if f["channels_positive_on_most_moved"] > 0 and f["channels_negative_on_most_moved"] > 0),
            "families_cancelling": sum(
                1 for f in families
                if (f["additive_residual_ratio"] is not None and f["additive_residual_ratio"] < 0.2)
                and f["channels_positive_on_most_moved"] > 0 and f["channels_negative_on_most_moved"] > 0
                and (f["most_moved_family_over_max_channel"] is not None and f["most_moved_family_over_max_channel"] < 1.0)),
            "families_with_action_queue_change": sum(1 for f in families if f["stats"]["scenes_actions_differ"] > 0),
            "divergent_passes_total": sum(f["stats"]["divergent_passes"] for f in families),
        },
        "channels": channels,
        "families": families,
    }
    kinds = {}
    for c in delta_channels:
        k = s(c, "most_moved_kind") or "none"
        kinds[k] = kinds.get(k, 0) + 1
    reading["per_channel"]["most_moved_kind_counts"] = kinds
    # Predeclared classification.
    pc, pf = reading["per_channel"], reading["per_family"]
    rc, rf = pc["ratio_median_of_medians"], pf["ratio_median_of_medians"]
    sub, res = pf["subadditivity_ratio_median"], pf["additive_residual_ratio_median"]
    param_only = pc["channels_action_change_without_vote_delta"]
    if not delta_channels and param_only > 0:
        verdict = ("no executed target channel moved a vote; %d of %d changed committed actions with zero vote delta "
                   "(action-parameter surface, outside the predeclared classes)" % (param_only, len(channels)))
    elif not delta_channels:
        verdict = "no executed target channel moved a vote (inconclusive)"
    elif rc is not None and rc >= 0.5:
        verdict = "neither"
    elif pf["families_cancelling"] > 0:
        verdict = "cancellation among channels (in %d of %d families)" % (pf["families_cancelling"], len(families))
    elif res is not None and res >= 0.2:
        verdict = "subadditive interaction"
    elif (rc is not None and rc < 0.1) and (rf is None or rf < 0.1) and (sub is not None and 0.8 <= sub <= 1.2) and (res is not None and res < 0.2):
        verdict = "dilution"
    else:
        verdict = "mixed"
    reading["classification"] = verdict
    return reading


def main():
    census_path, samples_path, out = sys.argv[1:4]
    census = rows(census_path)
    samples = rows(samples_path)
    header = next(r for r in census if r.get("kind") == "header")
    footer = next((r for r in census if r.get("kind") == "footer"), None)
    extinct = next((r for r in census if r.get("kind") == "extinct"), None)
    checkpoints = {}
    for r in census:
        if r.get("kind") != "checkpoint":
            continue
        fc = family_causal(r["input_use"])
        reading = checkpoint_reading(r)
        reading["funnel_target_rows"], reading["parents_evaluated"] = (fc if isinstance(fc, dict) else fc[0]), (None if isinstance(fc, dict) else fc[1])
        causal_rows = sum(int(v["causal"]) for v in reading["funnel_target_rows"].values() if v["causal"] != "-")
        reading["agreement"] = {
            "funnel_causal_incidences": causal_rows,
            "channels_with_action_queue_change": reading["per_channel"]["channels_with_action_queue_change"],
        }
        reading["vote_delta_secs"] = r.get("vote_delta_secs")
        reading["census_secs"] = r.get("census_secs")
        reading["living"] = r.get("living")
        checkpoints[str(r["tick"])] = reading
    trajectory = [{k: s.get(k) for k in ("tick", "population", "mean_genome_size", "mean_generation", "mean_age", "mean_energy")}
                  for s in samples if s.get("kind") == "sample" and s["tick"] % 2500 == 0]
    result = {
        "row": "N3", "world": header["world"], "seed": header["seed"], "threads": header["threads"],
        "horizon": header["horizon"], "checkpoints_requested": header["checkpoints"], "vote_delta_version": header.get("vote_delta_version"),
        "complete": footer is not None and not (footer or {}).get("extinct"), "extinct": extinct["tick"] if extinct else None,
        "footer": footer, "trajectory": trajectory,
        "checkpoints": {t: {k: v for k, v in c.items() if k not in ("channels", "families")} for t, c in checkpoints.items()},
        "provenance": {"census_sha256": sha(census_path), "samples_sha256": sha(samples_path), "reader": "n3read.py"},
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    detail = out.replace(".json", "-channels.json")
    open(detail, "w").write(json.dumps({t: {"channels": c["channels"], "families": c["families"]} for t, c in checkpoints.items()}) + "\n")
    for t, c in result["checkpoints"].items():
        print(t, "channels", c["executed_target_channels"], "ratio med", c["per_channel"]["ratio_median_of_medians"],
              "family sub", c["per_family"]["subadditivity_ratio_median"], "residual", c["per_family"]["additive_residual_ratio_median"],
              "->", c["classification"])


if __name__ == "__main__":
    main()
