"""Diagnosis run row E5: read the twin-screen summaries of the AddProjection
variants (`random`, `shared`, `aligned`, `scrambled`) and the A/A check
against native, pooled over the 16 strata per assay. The primary contrast is
the arm's *fired* births (an AddProjection event applied) against their own
native twins (same parent, same seed; the operator is the only difference):
shares of confirmed-helpful and harmful children per fired birth on each
side; secondary, the fired births against native overall. Intervals are
Clopper-Pearson at the Bonferroni level of the row's six contrasts
(one-sided 0.025 / 6, i.e. two-sided 1 - 0.05 / 6 = 99.17 %), and the
contrast ratios take the lower bound of the numerator over the upper bound
of the denominator (and the reverse). A ratio whose lower bound exceeds 2 is
the declared "several times" effect; a null whose upper bound is at or above
2 is inconclusive.

usage: python3 e5read.py <screen-dir> <out.json>   (files <assay>-s<stratum>-<variant>.ndjson)"""
import hashlib
import json
import math
import os
import sys

VARIANTS = ("random", "shared", "aligned", "scrambled", "aa")
KEYS = ("drawn", "identical", "evaluated", "neutral", "harmful", "helpful_a", "confirmed", "sterility_attributable", "fired_births")
ALPHA_TWO_SIDED = 0.05 / 6


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def betacdf(x, a, b):
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    lbeta = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1 - x)
    front = math.exp(lbeta)

    def cf(x, a, b):
        MAXIT, EPS, FPMIN = 300, 3e-14, 1e-300
        qab, qap, qam = a + b, a + 1, a - 1
        c, d = 1.0, 1.0 - qab * x / qap
        d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
        h = d
        for m in range(1, MAXIT + 1):
            m2 = 2 * m
            aa = m * (b - m) * x / ((qam + m2) * (a + m2))
            d = 1.0 + aa * d
            d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
            c = 1.0 + aa / (c if abs(c) > FPMIN else FPMIN)
            h *= d * c
            aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
            d = 1.0 + aa * d
            d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
            c = 1.0 + aa / (c if abs(c) > FPMIN else FPMIN)
            de = d * c
            h *= de
            if abs(de - 1.0) < EPS:
                break
        return h
    if x < (a + 1) / (a + b + 2):
        return front * cf(x, a, b) / a
    return 1.0 - front * cf(1 - x, b, a) / b


def cp(k, n, alpha=ALPHA_TWO_SIDED):
    """Exact Clopper-Pearson at two-sided level 1 - alpha."""
    if n == 0:
        return (0.0, 1.0)

    def solve(target, a, b):
        lo, hi = 0.0, 1.0
        for _ in range(100):
            mid = (lo + hi) / 2
            if betacdf(mid, a, b) < target:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2
    lower = 0.0 if k == 0 else solve(alpha / 2, k, n - k + 1)
    upper = 1.0 if k == n else solve(1 - alpha / 2, k + 1, n - k)
    return (lower, upper)


def zero():
    return {k: 0 for k in KEYS}


def add(acc, tally):
    for k in KEYS:
        acc[k] += tally.get(k, 0)


def share(k, n):
    lo, hi = cp(k, n)
    return {"count": k, "of": n, "share": (k / n) if n else None, "cp_bonferroni": [lo, hi]}


def ratio(num, den):
    """Ratio of shares with bounds from the two intervals."""
    if not num["of"] or not den["of"] or den["share"] in (None, 0):
        return None
    lo = num["cp_bonferroni"][0] / den["cp_bonferroni"][1] if den["cp_bonferroni"][1] else None
    hi = (num["cp_bonferroni"][1] / den["cp_bonferroni"][0]) if den["cp_bonferroni"][0] else float("inf")
    return {"point": (num["share"] / den["share"]) if den["share"] else None, "lower": lo, "upper": hi,
            "declared_effect_supported": lo is not None and lo > 2.0,
            "doubling_excluded": hi is not None and hi < 2.0}


def main():
    d, out_path = sys.argv[1:3]
    pooled = {}
    files = {}
    for name in sorted(os.listdir(d)):
        if not name.endswith(".ndjson"):
            continue
        assay, rest = name.split("-s", 1)
        stratum, variant = rest[:-len(".ndjson")].split("-", 1)
        summary = None
        for line in open(os.path.join(d, name)):
            row = json.loads(line)
            if row.get("probe") == "r5-screen":
                summary = row
        if summary is None:
            continue
        key = (assay, variant)
        p = pooled.setdefault(key, {"native": zero(), "arm": zero(), "fired_arm": zero(), "fired_native": zero(),
                                    "twin_equal": 0, "strata": 0, "pairs": 0, "mutation_sha256": summary["arm_mutation_sha256"]})
        add(p["native"], summary["native"])
        add(p["arm"], summary["arm_side"])
        add(p["fired_arm"], summary["fired"]["arm_side"])
        add(p["fired_native"], summary["fired"]["native"])
        p["twin_equal"] += summary["twin"]["twin_equal"]
        p["strata"] += 1
        p["pairs"] += summary["pairs"]
        files[name] = sha(os.path.join(d, name))
    out = {"row": "E5", "assays": {}, "files_sha256": files, "reader_sha256": sha(__file__),
           "intervals": f"Clopper-Pearson two-sided {1 - ALPHA_TWO_SIDED:.4f} (Bonferroni over six contrasts)"}
    for assay in sorted({a for a, _ in pooled}):
        block = {"variants": {}, "contrasts": {}}
        for variant in VARIANTS:
            p = pooled.get((assay, variant))
            if not p:
                continue
            nat, arm, fired, twins = p["native"], p["arm"], p["fired_arm"], p["fired_native"]
            v = {
                "strata": p["strata"], "pairs": p["pairs"], "twin_equal": p["twin_equal"], "mutation_sha256": p["mutation_sha256"],
                "native_confirmed_per_birth": share(nat["confirmed"], nat["drawn"]),
                "native_harmful_per_birth": share(nat["harmful"], nat["drawn"]),
                "arm_confirmed_per_birth": share(arm["confirmed"], arm["drawn"]),
                "arm_harmful_per_birth": share(arm["harmful"], arm["drawn"]),
                "fired_births": share(fired["drawn"], arm["drawn"]),
                "fired_confirmed_per_fired_birth": share(fired["confirmed"], fired["drawn"]),
                "fired_harmful_per_fired_birth": share(fired["harmful"], fired["drawn"]),
                "fired_identical": fired["identical"], "fired_evaluated": fired["evaluated"],
                "twin_confirmed_per_fired_birth": share(twins["confirmed"], twins["drawn"]),
                "twin_harmful_per_fired_birth": share(twins["harmful"], twins["drawn"]),
            }
            v["fired_vs_twin_confirmed"] = ratio(v["fired_confirmed_per_fired_birth"], v["twin_confirmed_per_fired_birth"])
            v["fired_vs_twin_harmful"] = ratio(v["fired_harmful_per_fired_birth"], v["twin_harmful_per_fired_birth"])
            v["fired_confirmed_vs_native_per_birth"] = ratio(v["fired_confirmed_per_fired_birth"], v["native_confirmed_per_birth"])
            block["variants"][variant] = v
        vs = block["variants"]
        if "aligned" in vs:
            for other in ("scrambled", "shared"):
                if other in vs:
                    block["contrasts"][f"aligned_vs_{other}_fired_confirmed"] = ratio(
                        vs["aligned"]["fired_confirmed_per_fired_birth"], vs[other]["fired_confirmed_per_fired_birth"])
        out["assays"][assay] = block
    json.dump(out, open(out_path, "w"), indent=1)
    for assay, block in out["assays"].items():
        for variant, v in block["variants"].items():
            print(f"{assay} {variant:9s} strata {v['strata']} pairs {v['pairs']} twin_equal {v['twin_equal']} fired {v['fired_births']['count']} "
                  f"native conf {v['native_confirmed_per_birth']['share']:.4f} harm {v['native_harmful_per_birth']['share']:.4f} | "
                  f"fired conf {v['fired_confirmed_per_fired_birth']['share']} twin conf {v['twin_confirmed_per_fired_birth']['share']} "
                  f"fired harm {v['fired_harmful_per_fired_birth']['share']} twin harm {v['twin_harmful_per_fired_birth']['share']} "
                  f"vs twin {v['fired_vs_twin_confirmed'] and {k: (round(x, 3) if isinstance(x, float) else x) for k, x in v['fired_vs_twin_confirmed'].items()}}")
        print(assay, "contrasts", block["contrasts"])


if __name__ == "__main__":
    main()
