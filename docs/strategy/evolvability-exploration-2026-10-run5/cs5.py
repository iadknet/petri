"""Run 5 comparison set (rule 5, descriptive only; never deciding).

One step (cs/): refinement, M4, M5 seed-paired with native, N = 4,096 per
stratum: twin cells b (arm-only counted) and c (native-only counted) per assay,
with the one-sided sign p for b > c and the two-sided p.

Two steps (twostep/): M1, M2, M1 + refinement against native: per stratum, 64
changed-neutral children per side, unit = child with at least one counted
grandchild. Per assay, the stratified exact test conditions on each stratum's
total positive units T_i: the arm's count is hypergeometric(128, 64, T_i) under
the null; the p-value for the arm total >= observed comes from the exact
convolution across strata (Mantel-Haenszel type)."""
import json
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(__file__))
import analyze5 as a  # noqa: E402

P = ".bench-artifacts/lab/exploration/run5"


def rows(dirpath, probe):
    out = []
    for name in sorted(os.listdir(dirpath)):
        if name.endswith(".ndjson"):
            with open(os.path.join(dirpath, name)) as f:
                for line in f:
                    r = json.loads(line)
                    if r.get("probe") == probe:
                        out.append(r)
    return out


def one_step():
    tally = defaultdict(lambda: {"b": 0, "c": 0, "pairs": 0, "strata": 0})
    for r in rows(f"{P}/cs", "r5-screen"):
        t = tally[(r["arm"], r["assay"])]
        t["b"] += r["twin"]["cells"]["b"]
        t["c"] += r["twin"]["cells"]["c"]
        t["pairs"] += r["range"][1] - r["range"][0]
        t["strata"] += 1
    out = []
    for (arm, assay), t in sorted(tally.items()):
        out.append({"arm": arm, "assay": assay, **t,
                    "p_one_sided": a.sign_p(t["b"], t["c"]),
                    "p_two_sided": a.sign_p_two_sided(t["b"], t["c"])})
    return out


def hyper_pmf(n_total, n_arm, t):
    """Arm count distribution given t positives among n_total units, n_arm of them the arm's."""
    return [math.comb(n_arm, k) * math.comb(n_total - n_arm, t - k) / math.comb(n_total, t)
            for k in range(0, min(t, n_arm) + 1)]


def convolve(dists):
    acc = [1.0]
    for d in dists:
        nxt = [0.0] * (len(acc) + len(d) - 1)
        for i, x in enumerate(acc):
            for j, y in enumerate(d):
                nxt[i + j] += x * y
        acc = nxt
    return acc


def two_step():
    by = {}
    status = defaultdict(int)
    for r in rows(f"{P}/twostep", "r5-two-step"):
        status[r["status"]] += 1
        by[(r["side"], r["assay"], str(r["stratum"]))] = r
    out = []
    for arm in ("m1", "m2", "m1refine"):
        for assay in ("food", "wall"):
            dists, obs, nat_total, used = [], 0, 0, 0
            complete = True
            for s in range(16):
                x, n = by.get((arm, assay, str(s))), by.get(("native", assay, str(s)))
                if not x or not n or x["status"] != "complete" or n["status"] != "complete":
                    complete = False
                    continue
                t = x["units_positive"] + n["units_positive"]
                dists.append(hyper_pmf(x["children_used"] + n["children_used"], x["children_used"], t))
                obs += x["units_positive"]
                nat_total += n["units_positive"]
                used += x["children_used"]
            dist = convolve(dists)
            p = sum(dist[obs:]) if obs < len(dist) else 0.0
            out.append({"arm": arm, "assay": assay, "complete": complete, "units_per_side": used,
                        "arm_positive": obs, "native_positive": nat_total, "p_one_sided": p})
    return out, dict(status)


if __name__ == "__main__":
    one = one_step()
    two, status = two_step()
    res = {"one_step": one, "two_step": two, "two_step_status": status}
    with open(f"{P}/cs5.json", "w") as f:
        json.dump(res, f, indent=1)
    for r in one:
        print(json.dumps(r))
    for r in two:
        print(json.dumps(r))
    print(json.dumps(status))
