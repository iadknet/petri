"""Row V1 (and D) reader (vote-margin read run): from a v2 census, per
checkpoint, the plan's class table per channel with counts (overall and
per family), the per-consumer exposure and local endpoints summarized, the
per-pass effective-kind record summarized per channel, the parameter fields
of every action-changing channel, the pooled per-pass median of |delta| /
within-kind margin with N3's labels computed on it (N3's reader's rule with
the pooled statistic in place of the median of per-channel medians), the
family readings, option C's block beside (never merged), the consistency
counts, the three predictions' outcomes and the routing table.

usage: python3 v1read.py <census ndjson> <out.json>"""
import json
import sys

ORDER = ["vote-causal", "param-causal", "action-unresolved", "unexposed", "vm-unresolved",
         "silent-local-decision", "silent-local-intermediate", "silent-mixed", "masked",
         "suppressing", "moving-residual"]
ROUTE = {
    "vote-causal": "the funnel's existing causal",
    "param-causal": "T11.F27 action-parameter surface (a second causality route)",
    "action-unresolved": "T22 instrument",
    "unexposed": "T22 instrument (panel exposure per consumer); world exposure to T20",
    "vm-unresolved": "T22 instrument (VM read-to-write attribution)",
    "silent-local-decision": "T22 first (weight, saturation, unconsumed parameter), then T19/T11",
    "silent-local-intermediate": "T22 first (cross-node path tracing), then T20",
    "silent-mixed": "reported with the split",
    "masked": "T19 (pass-end rule, incumbent) with per-pass shortfalls",
    "suppressing": "reported: inert suppression of a non-winner",
    "moving-residual": "reported, neutral",
}


def med(values):
    values = sorted(v for v in values if v is not None)
    return values[len(values) // 2] if values else None


def channel_row(parent, c):
    v2, st = c["v2"], c["stats"]
    exposed = [x for x in v2["consumers"] if sum(x["exposed"].values()) > 0]
    return {
        "parent": parent["parent_index"], "generation": parent["generation"], "family": c["family"],
        "channel": c["channel"], "class": v2["class"],
        "consumers": len(v2["consumers"]),
        "consumers_graph": sum(1 for x in v2["consumers"] if x["backend"] == "graph"),
        "consumers_vm": sum(1 for x in v2["consumers"] if x["backend"] == "vm"),
        "consumers_executed": v2["consumers_executed"], "consumers_exposed": v2["consumers_exposed"],
        "exposed_endpoint_classes": sorted({x["endpoint_class"] for x in exposed}),
        "all_endpoint_classes": sorted({x["endpoint_class"] for x in v2["consumers"]}),
        "exposed_executions": v2["exposed_executions"],
        "exposed_by_group": {g: sum(x["exposed"][g] for x in v2["consumers"]) for g in
                             ("battery", "recorded", "authored", "sequences", "exposure")},
        "executed_by_group": {g: sum(x["executed"][g] for x in v2["consumers"]) for g in
                              ("battery", "recorded", "authored", "sequences", "exposure")},
        "graph_abs_weight": c.get("consumers_graph_abs_weight"),
        "delta_passes": st["delta_passes"], "scenes_actions_differ": st["scenes_actions_differ"],
        "divergent_passes": st["divergent_passes"], "unmatched_passes": st["unmatched_passes"],
        "ratio_median": st["ratio_median"], "most_moved_sink": st["most_moved_sink"],
        "masking": {k: v2["masking"][k] for k in ("delta_passes", "commit_both_passes", "raising_passes",
                                                   "lowering_passes", "no_kind_moved_passes", "divergent_passes",
                                                   "guarded_passes_baseline", "shortfall_median", "shortfall_min",
                                                   "moved_kinds")},
        "params": {k: v2["params"][k] for k in ("action_changing_executions", "aligned_passes", "unmatched_passes",
                                                 "committed_differs_passes",
                                                 "executions_without_differing_aligned_commit", "fields_differ",
                                                 "fields_consumed", "fields_differ_on_changed_commits",
                                                 "field_passes")},
        "unexposed_with_effect": v2["unexposed_with_effect"],
        "selection_not_reconstructed": sum(1 for p in v2["passes"] for ok in p["selection_reconstructed"] if not ok),
    }


def labels(pooled, families, contributing):
    """N3's predeclared labels with the pooled per-pass statistic."""
    fam_ratio = med([f["stats"]["ratio_median"] for f in families if f["stats"]["margined_passes"] > 0])
    sub = med([f["l1_subadditivity_ratio"] for f in families])
    res = med([f["additive_residual_ratio"] for f in families])
    cancelling = sum(1 for f in families
                     if f["additive_residual_ratio"] is not None and f["additive_residual_ratio"] < 0.2
                     and f["channels_positive_on_most_moved"] > 0 and f["channels_negative_on_most_moved"] > 0
                     and f["most_moved_family_over_max_channel"] is not None
                     and f["most_moved_family_over_max_channel"] < 1.0)
    if pooled is None:
        label = "undefined (no margined delta pass)"
    elif pooled >= 0.5:
        label = "neither"
    elif cancelling > 0:
        label = f"cancellation among channels (in {cancelling} of {len(families)} families)"
    elif res is not None and res >= 0.2:
        label = "subadditive interaction"
    elif pooled < 0.1 and (fam_ratio is None or fam_ratio < 0.1) and sub is not None and 0.8 <= sub <= 1.2 \
            and res is not None and res < 0.2:
        label = "dilution"
    else:
        label = "mixed"
    return {"label": label, "channels_with_margined_passes": contributing, "family_ratio_median_of_medians": fam_ratio, "subadditivity_median": sub,
            "additive_residual_median": res, "families": len(families), "families_cancelling": cancelling,
            "families_defined_subadditivity": sum(1 for f in families if f["l1_subadditivity_ratio"] is not None)}


def checkpoint(cp):
    block = cp["vote_deltas"] or []
    rows, ratios, families, c_rows, coverage = [], [], [], [], []
    for p in block:
        for c in p["channels"]:
            rows.append(channel_row(p, c))
            ratios += [r["ratio"] for r in c["v2"]["passes"] if r["ratio"] is not None]
        families += p["families"]
        x = p["exposure_contexts"]
        for c in x["channels"]:
            c_rows.append(channel_row(p, c))
        for o in x["original_coverage"]:
            coverage.append({"parent": p["parent_index"], **{k: o[k] for k in (
                "family", "channel", "in_exposure_candidates", "exposed", "executed_zero", "coverage_unresolved")}})
    counts = {k: sum(1 for r in rows if r["class"] == k) for k in ORDER}
    by_family = {}
    for r in rows:
        by_family.setdefault(r["family"], {k: 0 for k in ORDER})[r["class"]] += 1
    pooled = med(ratios)
    silent = [r for r in rows if r["class"] in ORDER[3:8]]
    moving = [r for r in rows if r["class"] in ORDER[8:11]]
    acting = [r for r in rows if r["class"] in ORDER[0:3]]
    exposed_consumers = [x for p in block for c in p["channels"] for x in c["v2"]["consumers"]
                         if sum(x["exposed"].values()) > 0]
    return {
        "tick": cp["tick"], "living": cp["living"], "parents": len(block),
        "parents_with_executed_target_channels": sum(1 for p in block if p["executed_target_channels"] > 0),
        "channels": len(rows), "class_counts": counts, "class_counts_by_family": by_family,
        "routing": {k: {"channels": counts[k], "routed_to": ROUTE[k]} for k in ORDER},
        "pooled_ratio_median": pooled, "pooled_ratio_passes": len(ratios),
        "n3_labels_on_pooled": labels(pooled, families, sum(1 for p in block for c in p["channels"] if any(r["ratio"] is not None for r in c["v2"]["passes"]))),
        "consumers": {
            "total": sum(r["consumers"] for r in rows), "graph": sum(r["consumers_graph"] for r in rows),
            "vm": sum(r["consumers_vm"] for r in rows), "executed": sum(r["consumers_executed"] for r in rows),
            "exposed": len(exposed_consumers),
            "exposed_endpoint_classes": {k: sum(1 for x in exposed_consumers if x["endpoint_class"] == k)
                                         for k in ("decision", "intermediate", "none", "dynamic")},
        },
        "consistency": {"unexposed_with_effect": sum(1 for r in rows if r["unexposed_with_effect"]),
                        "selection_not_reconstructed_passes": sum(r["selection_not_reconstructed"] for r in rows)},
        "silent_channels": len(silent), "moving_channels": len(moving), "action_changing_channels": len(acting),
        "exposure_contexts": {
            "c_candidates": len(c_rows),
            "c_class_counts": {k: sum(1 for r in c_rows if r["class"] == k) for k in ORDER},
            "original_candidates_in_c_candidates": sum(1 for o in coverage if o["in_exposure_candidates"]),
            "original_consumers": {"exposed": sum(o["exposed"] for o in coverage),
                                   "executed_zero": sum(o["executed_zero"] for o in coverage),
                                   "coverage_unresolved": sum(o["coverage_unresolved"] for o in coverage)},
            "coverage": coverage, "channels": c_rows,
        },
        "rows": rows,
    }


def predictions(cps):
    out = {}
    last = next((c for c in cps if c["tick"] == 50000), None)
    if last is None:
        out["1"] = {"outcome": "not reached", "checkpoint": 50000}
    else:
        n = last["silent_channels"]
        k = last["class_counts"]["unexposed"] + last["class_counts"]["silent-local-intermediate"]
        out["1"] = {"checkpoint": 50000, "classes_4_8": n, "unexposed_or_intermediate": k,
                    "silent_local_decision": last["class_counts"]["silent-local-decision"],
                    "outcome": "vacuous" if n == 0 else ("met" if 2 * k >= n else "failed"),
                    "most_silent_local_decision": n > 0 and 2 * last["class_counts"]["silent-local-decision"] > n}
    out["2"] = []
    for c in cps:
        n, m = c["moving_channels"], c["class_counts"]["masked"]
        out["2"].append({"checkpoint": c["tick"], "classes_9_11": n, "masked": m,
                         "outcome": "vacuous" if n == 0 else ("met" if 2 * m < n else "failed")})
    out["3"] = []
    for c in cps:
        acting = [r for r in c["rows"] if r["class"] in ORDER[0:3]]
        bad = [r for r in acting if r["class"] != "param-causal" or len(r["params"]["fields_differ"]) != 1]
        out["3"].append({"checkpoint": c["tick"], "action_changing": len(acting),
                         "param_causal_one_field": len(acting) - len(bad),
                         "failing": [{k: r[k] for k in ("parent", "family", "channel", "class")} |
                                     {"fields_differ": r["params"]["fields_differ"]} for r in bad],
                         "outcome": "vacuous" if not acting else ("met" if not bad else "failed")})
    return out


def main():
    path, out = sys.argv[1:3]
    lines = [json.loads(l) for l in open(path) if l.startswith("{")]
    header = next(l for l in lines if l.get("kind") == "header")
    footer = next((l for l in lines if l.get("kind") == "footer"), None)
    extinct = next((l["tick"] for l in lines if l.get("kind") == "extinct"), None)
    cps = [checkpoint(l) for l in lines if l.get("kind") == "checkpoint"]
    reached = [c["tick"] for c in cps]
    complete = footer is not None and (footer["ticks"] == header["horizon"] or footer.get("extinct"))
    every_classed = all(r["class"] in ORDER for c in cps for r in c["rows"])
    result = {
        "world": header["world"], "seed": header["seed"], "horizon": header["horizon"],
        "checkpoints_scheduled": header["checkpoints"], "checkpoints_reached": reached,
        "footer": footer, "extinct": extinct, "vote_delta_version": header["vote_delta_version"],
        "row_status": "completed" if complete and set(reached) == set(header["checkpoints"]) else "incomplete",
        "every_executed_target_channel_classed": every_classed,
        "predictions": predictions(cps), "checkpoints": cps,
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in cps:
        print(c["tick"], "channels", c["channels"], {k: v for k, v in c["class_counts"].items() if v},
              "pooled", c["pooled_ratio_median"], c["n3_labels_on_pooled"]["label"])
    print("predictions", json.dumps({k: (v["outcome"] if isinstance(v, dict) else [x["outcome"] for x in v])
                                     for k, v in result["predictions"].items()}))


if __name__ == "__main__":
    main()
