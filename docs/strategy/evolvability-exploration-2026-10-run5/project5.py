"""Run 5 compact projection (versioned; the note's numbers regenerate from it):
per-assay and per-stratum J twin cells, fired-birth class totals, A/A cells per
stratum, pilot cells per stratum, PC-J pilot, PC-F, comparison-set counts with
their changed-child denominators, and raw provenance (bytes per directory)."""
import json
import os
from collections import defaultdict

P = ".bench-artifacts/lab/exploration/run5"
VERSION = 1


def summaries(sub, probe="r5-screen"):
    out = []
    d = f"{P}/{sub}"
    for name in sorted(os.listdir(d)):
        if name.endswith(".ndjson"):
            with open(os.path.join(d, name)) as f:
                for line in f:
                    r = json.loads(line)
                    if r.get("probe") == probe:
                        out.append(r)
    return out


def dir_bytes(sub):
    d = f"{P}/{sub}"
    return sum(os.path.getsize(os.path.join(d, n)) for n in os.listdir(d))


def side_counts(s):
    keys = ("drawn", "identical", "evaluated", "neutral", "harmful", "helpful_a", "confirmed",
            "sterility_attributable", "counted", "inert")
    return {k: s.get(k, 0) for k in keys}


def add(acc, d):
    for k, v in d.items():
        acc[k] = acc.get(k, 0) + v


def j_main():
    per_stratum = defaultdict(lambda: {"b": 0, "c": 0, "both": 0, "neither": 0, "pairs": 0,
                                       "weight_events": 0, "fired_jumps": 0, "fired_births": 0})
    fired = defaultdict(lambda: {"pairs": 0, "j": {}, "native": {}})
    for r in summaries("main"):
        key = (r["assay"], str(r["stratum"]))
        ps = per_stratum[key]
        for k in ("b", "c", "both", "neither"):
            ps[k] += r["twin"]["cells"][k]
        ps["pairs"] += r["range"][1] - r["range"][0]
        for k in ("applied_weight_events", "fired_jumps", "fired_births"):
            ps["weight_events" if k == "applied_weight_events" else k] += r["j"][k]
        f = fired[r["assay"]]
        f["pairs"] += r["fired"]["pairs"]
        add(f["j"], side_counts(r["fired"]["arm_side"]))
        add(f["native"], side_counts(r["fired"]["native"]))
    strata = [{"assay": a, "stratum": s, **v} for (a, s), v in sorted(per_stratum.items(), key=lambda kv: (kv[0][0], int(kv[0][1])))]
    return strata, dict(fired)


def cells_by_stratum(sub, arm, tag):
    out = defaultdict(lambda: {"b": 0, "c": 0, "pairs": 0, "native_counted": 0})
    for r in summaries(sub):
        if r["arm"] == arm and r["tag"] == tag:
            o = out[(r["assay"], str(r["stratum"]))]
            o["b"] += r["twin"]["cells"]["b"]
            o["c"] += r["twin"]["cells"]["c"]
            o["pairs"] += r["range"][1] - r["range"][0]
            o["native_counted"] += r["native"]["counted"]
    return [{"assay": a, "stratum": s, **v} for (a, s), v in sorted(out.items())]


def comparison_set():
    out = defaultdict(lambda: {"b": 0, "c": 0, "pairs": 0, "native_side": {}, "arm_side": {}})
    for r in summaries("cs"):
        o = out[(r["arm"], r["assay"])]
        o["b"] += r["twin"]["cells"]["b"]
        o["c"] += r["twin"]["cells"]["c"]
        o["pairs"] += r["range"][1] - r["range"][0]
        add(o["native_side"], side_counts(r["native"]))
        add(o["arm_side"], side_counts(r["arm_side"]))
    two = defaultdict(lambda: {"children_used": 0, "units_positive": 0, "complete": 0})
    for r in summaries("twostep", "r5-two-step"):
        t = two[(r["side"], r["assay"])]
        t["children_used"] += r["children_used"]
        t["units_positive"] += r["units_positive"]
        t["complete"] += int(r["status"] == "complete")
    return ([{"arm": a, "assay": s, **v} for (a, s), v in sorted(out.items())],
            [{"side": a, "assay": s, **v} for (a, s), v in sorted(two.items())])


def aa_record_strata():
    """The record-based A/A cells per stratum (ledger row V-A), from analyze5."""
    import analyze5 as a
    with open(f"{P}/manifest.json") as f:
        expected = a.expected_strata(json.load(f))
    main_s, main_c = a.load(f"{P}/main")
    aa_s, aa_c = a.load(f"{P}/aa")
    _, cells, problems = a.aa_from_records(main_s, main_c, aa_s, aa_c, expected)
    assert not problems, problems
    return [{"assay": k[0], "stratum": k[1], **v}
            for k, v in sorted(cells.items(), key=lambda kv: (kv[0][0], int(kv[0][1])))]


def pcf_max_delta():
    best = {}
    with open(f"{P}/pcf/native-food-pcf-0.ndjson") as f:
        for line in f:
            r = json.loads(line)
            if r.get("probe") == "r5-screen-child" and r.get("side") == "native" and r.get("label") == "confirmed":
                for k in ("delta_a", "delta_b"):
                    best[k] = max(best.get(k, float("-inf")), r[k])
    return best


def tree_bytes():
    total = 0
    for root, _, files in os.walk(P):
        total += sum(os.path.getsize(os.path.join(root, n)) for n in files)
    return total


def main():
    import sys
    sys.path.insert(0, P)
    strata, fired = j_main()
    one, two = comparison_set()
    pcf = summaries("pcf")[0]
    proj = {
        "projection": "run5-compact", "version": VERSION,
        "j_strata": strata, "j_fired_births": fired,
        "aa_strata_record_based": aa_record_strata(),
        "pilot_strata": cells_by_stratum("pilot", "j", "run5-pilot"),
        "pcj_pilot": cells_by_stratum("pcj-pilot", "j", "run5-pcj-pilot"),
        "pcf": {"pairs": pcf["pairs"], "native": side_counts(pcf["native"]),
                "max_confirmed": pcf_max_delta(),
                "parent_bank_a": pcf["parent_bank_a"], "parent_bank_b": pcf["parent_bank_b"]},
        "comparison_one_step": one, "comparison_two_step": two,
        "screen_output_bytes": {sub: dir_bytes(sub) for sub in ("pilot", "pcj-pilot", "main", "rerun", "aa",
                                                                "aacheck", "pcf", "cs", "twostep")},
    }
    proj["screen_output_bytes"]["subtotal"] = sum(proj["screen_output_bytes"].values())
    proj["evidence_tree_bytes"] = tree_bytes()
    with open(f"{P}/run5-projection.json", "w") as f:
        json.dump(proj, f, separators=(",", ":"))
    print(json.dumps({"j_strata": len(strata), "aa_strata": len(proj["aa_strata_record_based"]),
                      "aa_b": sum(x["b"] for x in proj["aa_strata_record_based"]),
                      "aa_c": sum(x["c"] for x in proj["aa_strata_record_based"]),
                      "pcf_max": proj["pcf"]["max_confirmed"],
                      "screen_output_subtotal": proj["screen_output_bytes"]["subtotal"],
                      "evidence_tree_bytes": proj["evidence_tree_bytes"]}))


if __name__ == "__main__":
    main()
