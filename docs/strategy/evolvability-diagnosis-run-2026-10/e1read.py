"""Diagnosis run row E1: read the two diagnostic one-edge opportunity
summaries (sets `one-edge` and `one-edge-control`) and apply the row's
decision rule (amendments A1, A2): within-set A/Z under T20.F01's verdict
rule; world branch only if (a), (b), (c) are each negative in Canyon and
Confluence with pooled A/Z <= 1.00 for (a) and (c); variation branch if (a)
or (c) is positive anywhere; otherwise mixed (E6 decides). Cross-set
comparisons are descriptive. Writes a compact summary.

usage: python3 e1read.py <one-edge-summary.json> <one-edge-control-summary.json> <out.json>"""
import hashlib
import json
import sys


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read_set(path):
    s = json.load(open(path))
    out = {"set": s["arm_set"]["set"], "control_sink": s["arm_set"]["control_sink"],
           "control_sink_seed": s["arm_set"]["control_sink_seed"], "incomplete": s["incomplete"],
           "stop_reason": s["stop_reason"], "wall_secs": s["wall_secs"], "threads": s["threads"],
           "horizon": s["horizon"], "raw_sha256": s["raw"]["sha256"], "summary_sha256": sha(path),
           "slots": {}, "worlds": {}}
    for slot in s["arm_set"]["slots"]:
        c = slot["competence"]
        out["slots"][slot["label"]] = {
            "reference": slot["reference"], "sub_idx": slot["sub_idx"], "sink": slot["sink"],
            "weight": slot["weight"], "ring_family": slot["ring_family"],
            "z_is_twin": c["battery_inert_differs"] == 0 and c["context_inert_differs"] == 0,
            "competence": c,
        }
    for w in s["worlds"]:
        name = w["case"]["name"]
        rows = {}
        for v in w["verdicts"]:
            rows[v["family"]] = {
                "verdict": v["verdict"], "pooled_a_over_z": v["pooled"], "informative": v["informative"],
                "above_one": v["above_one"], "below_line": v["below_line"],
                "exposed_share": v["exposed"] / v["sampled"] if v["sampled"] else None,
                "applied_share": v["applied"] / v["sampled"] if v["sampled"] else None,
                "sampled": v["sampled"], "ratios": v["ratios"],
            }
        # A/F and Z/F descriptive, pooled over replicates.
        births = {}
        for rep in w["replicates"]:
            for arm in rep["arms"]:
                births[arm["arm"]] = births.get(arm["arm"], 0) + arm["births"]
        f = births.get("F", 0)
        for label in rows:
            a, z = births.get(f"A_{label}", 0), births.get(f"Z_{label}", 0)
            rows[label]["a_over_f_pooled"] = a / f if f else None
            rows[label]["z_over_f_pooled"] = z / f if f else None
            rows[label]["births_a"], rows[label]["births_z"] = a, z
        out["worlds"][name] = {"replicates": len(w["replicates"]), "founder_fallback": w["founder_fallback"],
                               "births_f": f, "births_i": births.get("I", births.get("I (founder fallback)", 0)),
                               "slots": rows}
    return out


def decide(one_edge):
    demand = [w for w in one_edge["worlds"] if "Canyon" in w or "Confluence" in w]
    rows = {w: one_edge["worlds"][w]["slots"] for w in one_edge["worlds"]}
    void = [k for k, v in one_edge["slots"].items() if not v["z_is_twin"]]
    positive_anywhere = any(rows[w][k]["verdict"] == "positive" for w in rows for k in ("a", "c") if k not in void)

    def neg_and_flat(w, k):
        r = rows[w][k]
        pooled = None if r["pooled_a_over_z"] in (None, "inf") else float(r["pooled_a_over_z"])
        negative = r["verdict"] == "negative"
        flat = k == "b" or (pooled is not None and pooled <= 1.0)
        return negative and flat

    world = bool(demand) and all(neg_and_flat(w, k) for w in demand for k in ("a", "b", "c")) and not void
    branch = "variation" if positive_anywhere else ("world" if world else "mixed (E6 decides)")
    return {"demand_worlds": demand, "void_slots": void, "positive_anywhere": positive_anywhere,
            "world_rule_met": world, "branch": branch}


def main():
    one_edge, control, out_path = sys.argv[1:4]
    a = read_set(one_edge)
    b = read_set(control)
    out = {"row": "E1", "diagnostic": True, "sets": {"one-edge": a, "one-edge-control": b},
           "decision": decide(a),
           "cross_set_note": "descriptive only: the two sets are two competitions",
           "reader_sha256": sha(__file__)}
    json.dump(out, open(out_path, "w"), indent=1)
    for name, s in out["sets"].items():
        print(name, "control_sink", s["control_sink"], "incomplete", s["incomplete"], f"{s['wall_secs']:.0f}s")
        for k, v in s["slots"].items():
            print(f"  slot {k}: {v['reference']} sub {v['sub_idx']} -> {v['sink']} w {v['weight']} twin_ok {v['z_is_twin']}")
        for w, ww in s["worlds"].items():
            for k, r in ww["slots"].items():
                print(f"  {w:22s} {k}: {r['verdict']:13s} pooled A/Z {r['pooled_a_over_z']} inf {r['informative']} >1 {r['above_one']} <1.05 {r['below_line']} "
                      f"exposed {r['exposed_share']:.3f} applied {r['applied_share']:.3f} A/F {r['a_over_f_pooled']:.3f} Z/F {r['z_over_f_pooled']:.3f}")
    print("decision", out["decision"])


if __name__ == "__main__":
    main()
