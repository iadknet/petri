"""Diagnosis run row E4: compact reading of the transition_rates probe output.
Per parent: the per-birth rates the row names with their 95 % CP intervals
(all births and single-event births), the loss of the prepared declaration
split by operator class (Prune / Swap / RawFieldMutation of the reference's
type index / other), connections split by landing (vote sinks versus compute
nodes), the authored edge's fate with "declaration changed" separated from a
true retarget (the probe's retarget class also catches a reference whose
index-7 entry was swapped or raw-mutated, because the edge then no longer
resolves to the declared reference), and the prune-to-useful-connect ratio
bounds of the row's decision rule.

usage: python3 e4read.py <e4-transition-rates.json> <out.json>"""
import hashlib
import json
import sys

ESTIMATE = {"declare": 3.2e-4, "lose_unconnected": 8.6e-3, "useful_connect": 4.8e-7}


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def by_class(ops_map):
    """Split an operator-combination map into classes by the InputRef
    operators present (a combination may hold several)."""
    out = {"prune_only": 0, "swap_only": 0, "raw_field_only": 0, "prune_any": 0, "swap_any": 0,
           "raw_field_any": 0, "add_only_or_none": 0, "total": 0}
    for key, n in ops_map.items():
        ops = key.split("|")[-1].split("+")
        out["total"] += n
        p, s, r = "InputRefPrune" in ops, "InputRefSwap" in ops, "InputRefRawFieldMutation" in ops
        out["prune_any"] += n if p else 0
        out["swap_any"] += n if s else 0
        out["raw_field_any"] += n if r else 0
        if p and not s and not r:
            out["prune_only"] += n
        elif s and not p and not r:
            out["swap_only"] += n
        elif r and not p and not s:
            out["raw_field_only"] += n
        elif not (p or s or r):
            out["add_only_or_none"] += n
    return out


def fates(lost_by_ops):
    """The authored edge's fate by stratum, with the probe's `retargeted`
    class split into true retargets (GraphRetargetGraphEdge applied) and
    declaration changes (InputRefSwap / InputRefRawFieldMutation applied)."""
    out = {}
    for key, n in lost_by_ops.items():
        fate, stratum, ops = key.split("|")
        ops = ops.split("+")
        if fate == "retargeted":
            if "GraphRetargetGraphEdge" in ops and not any(o in ops for o in ("InputRefSwap", "InputRefRawFieldMutation")):
                fate = "retargeted_true"
            elif any(o in ops for o in ("InputRefSwap", "InputRefRawFieldMutation")):
                fate = "declaration_changed"
            else:
                fate = "retargeted_other"
        d = out.setdefault(stratum, {})
        d[fate] = d.get(fate, 0) + n
    return out


def factor(measured, estimate):
    return None if not measured or not estimate else measured / estimate


def main():
    src, out_path = sys.argv[1:3]
    probe = json.load(open(src))
    rows = []
    for p in probe["parents"]:
        n = p["births"]
        occ = p["occurrences"]
        loc = occ["connect_by_location"]
        sinks = sum(v for k, v in loc.items() if k.startswith("sink:"))
        computes = sum(v for k, v in loc.items() if k.startswith("compute:"))
        move_sinks = sum(v for k, v in loc.items() if "ActionVote(Move(" in k)
        single = p["single_event_births"]
        row = {
            "parent": p["parent"], "genome_size": p["genome_size"], "nodes": p["nodes"],
            "executed_resolved": p["executed_resolved"], "births": n, "wall_secs": p["wall_secs"],
            "identical_share": p["identical"]["rate"], "zero_event_share": p["zero_event_births"]["rate"],
            "multi_event_share": p["multi_event_births"]["rate"], "applied_events_per_birth": p["applied_events_per_birth"],
            "single_event_share": single["share_of_births"],
            "rates_all_births": {
                "declare_target_on_vote_node": p["declared_vote_node_target"],
                "lose_declaration": p["lost_declaration"],
                "connect_any": p["connect_any"],
                "connect_cardinal_move": p["connect_cardinal_move"],
                "connect_useful_component_move": p["connect_useful_component_move"],
                "connect_useful_signed": p["connect_useful_signed"],
                "connect_useful_wrong_sign": p["connect_useful_wrong_sign"],
            },
            "rates_single_event_births": {
                "births": single["births"],
                "declare_target_on_vote_node": single["declared_vote_node_target"],
                "lose_declaration": single["lost_declaration"],
                "connect_any": single["connect_any"],
                "connect_cardinal_move": single["connect_cardinal_move"],
                "connect_useful_signed": single["connect_useful_signed"],
            },
            "connect_occurrences": {"total": sinks + computes, "sinks": sinks, "compute_nodes": computes,
                                    "move_sinks": move_sinks, "by_sub": occ["connect_by_sub"],
                                    "zero_weight_edges": occ["connect_zero_weight_edges"]},
            "lose_declaration_by_class": by_class(p["lost_declaration_by_ops"]) if p["lost_declaration"] else None,
            "authored_edge": None,
            "factor_vs_section_2_1": {
                "declare": factor(p["declared_vote_node_target"]["rate"], ESTIMATE["declare"]),
                "lose_unconnected": factor(p["lost_declaration"]["rate"], ESTIMATE["lose_unconnected"]) if p["lost_declaration"] else None,
                "useful_connect": factor(p["connect_useful_signed"]["rate"], ESTIMATE["useful_connect"]),
            },
        }
        if p["authored_reweighted"] is not None:
            row["authored_edge"] = {
                "all_births": {"reweighted": p["authored_reweighted"], "retargeted_probe_class": p["authored_retargeted"],
                               "removed": p["authored_removed"], "node_deleted": p["authored_node_deleted"]},
                "single_event": {k: single[f"authored_{k}"] for k in ("reweighted", "retargeted", "removed", "node_deleted")},
                "fates_by_stratum_and_operator_class": fates(p["authored_lost_by_ops"]),
            }
        if p["lost_declaration"]:
            lo_l, hi_l = single["lost_declaration"]["cp95"]
            lo_u, hi_u = single["connect_useful_signed"]["cp95"]
            row["ratio_lose_to_useful_connect"] = {
                "point_single": (single["lost_declaration"]["rate"] / single["connect_useful_signed"]["rate"]) if single["connect_useful_signed"]["rate"] else None,
                "point_all": (p["lost_declaration"]["rate"] / p["connect_useful_signed"]["rate"]) if p["connect_useful_signed"]["rate"] else None,
                "lower_bound": lo_l / hi_u if hi_u else None,
                "upper_bound": hi_l / lo_u if lo_u else None,
                "verdict": ("supported" if hi_u and lo_l / hi_u > 1e3 else
                            "refuted" if lo_u and hi_l / lo_u < 10 else "inconclusive"),
            }
        rows.append(row)
    out = {"row": "E4", "config": probe["config"], "elites": probe["elites"], "parents": rows,
           "section_2_1_estimates": ESTIMATE, "probe_sha256": sha(src), "reader_sha256": sha(__file__)}
    json.dump(out, open(out_path, "w"), indent=1)
    for r in rows:
        a = r["rates_all_births"]
        print(f"{r['parent']:28s} n={r['births']} ev/birth {r['applied_events_per_birth']:.3f} single {r['single_event_share']:.3f} multi {r['multi_event_share']:.3f}")
        print(f"   declare {a['declare_target_on_vote_node']['rate']:.2e}  lose {a['lose_declaration'] and a['lose_declaration']['rate']}  "
              f"connect_any {a['connect_any']['rate']:.2e} move {a['connect_cardinal_move']['rate']:.2e} useful {a['connect_useful_signed']['rate']:.2e} wrong {a['connect_useful_wrong_sign']['rate']:.2e}")
        if r["lose_declaration_by_class"]:
            print("   lose by class", r["lose_declaration_by_class"])
        print("   occurrences", r["connect_occurrences"]["sinks"], "sinks", r["connect_occurrences"]["compute_nodes"], "compute", r["connect_occurrences"]["move_sinks"], "move sinks")
        if r.get("ratio_lose_to_useful_connect"):
            print("   ratio", r["ratio_lose_to_useful_connect"])
        if r["authored_edge"]:
            print("   authored", r["authored_edge"]["fates_by_stratum_and_operator_class"])


if __name__ == "__main__":
    main()
