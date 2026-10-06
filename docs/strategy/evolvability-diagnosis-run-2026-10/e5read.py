"""Diagnosis run row E5: read the twin-screen summaries of the AddProjection
variants (`random`, `shared`, `aligned`, `scrambled`) against native, pooled
over the 16 strata per assay. The arm's births draw from a different stream
than the native twins once the operator is in the weighted draw, so the
reading is unpaired: shares of confirmed-helpful and harmful children per
drawn birth on the native side, on the arm side, and among the arm's *fired*
births (an AddProjection event applied), with 95 % CP bounds; the contrasts
of the row (each variant against native; aligned against scrambled and
against shared) as ratios of shares with bounds (CP lower of the numerator
over CP upper of the denominator, and the reverse), Bonferroni over six
contrasts at one-sided 0.025 / 6. A ratio whose lower bound exceeds 2 is the
declared "several times" effect; a null that does not exclude 2 (upper bound
at or above 2) is inconclusive.

usage: python3 e5read.py <screen-dir> <out.json>   (files <assay>-s<stratum>-<variant>.ndjson)"""
import hashlib
import json
import os
import sys
from e3read import cp95  # same folder

VARIANTS = ("random", "shared", "aligned", "scrambled")
KEYS = ("drawn", "identical", "evaluated", "neutral", "harmful", "helpful_a", "confirmed", "sterility_attributable", "fired_births")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def zero():
    return {k: 0 for k in KEYS}


def add(acc, tally):
    for k in KEYS:
        acc[k] += tally.get(k, 0)


def share(k, n):
    lo, hi = cp95(k, n)
    return {"count": k, "of": n, "share": (k / n) if n else None, "cp95": [lo, hi]}


def ratio(num, den):
    """Ratio of shares with bounds from the CP intervals."""
    if not num["of"] or not den["of"] or den["share"] in (None, 0):
        return None
    lo = num["cp95"][0] / den["cp95"][1] if den["cp95"][1] else None
    hi = (num["cp95"][1] / den["cp95"][0]) if den["cp95"][0] else float("inf")
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
                                    "strata": 0, "pairs": 0, "mutation_sha256": summary["arm_mutation_sha256"]})
        add(p["native"], summary["native"])
        add(p["arm"], summary["arm_side"])
        add(p["fired_arm"], summary["fired"]["arm_side"])
        add(p["fired_native"], summary["fired"]["native"])
        p["strata"] += 1
        p["pairs"] += summary["pairs"]
        files[name] = sha(os.path.join(d, name))
    out = {"row": "E5", "assays": {}, "files_sha256": files, "reader_sha256": sha(__file__)}
    for assay in sorted({a for a, _ in pooled}):
        block = {"variants": {}, "contrasts": {}}
        native_ref = None
        for variant in VARIANTS:
            p = pooled.get((assay, variant))
            if not p:
                continue
            nat, arm, fired = p["native"], p["arm"], p["fired_arm"]
            v = {
                "strata": p["strata"], "pairs": p["pairs"], "mutation_sha256": p["mutation_sha256"],
                "native_confirmed_per_birth": share(nat["confirmed"], nat["drawn"]),
                "native_harmful_per_birth": share(nat["harmful"], nat["drawn"]),
                "arm_confirmed_per_birth": share(arm["confirmed"], arm["drawn"]),
                "arm_harmful_per_birth": share(arm["harmful"], arm["drawn"]),
                "fired_births": share(fired["drawn"], arm["drawn"]),
                "fired_confirmed_per_fired_birth": share(fired["confirmed"], fired["drawn"]),
                "fired_harmful_per_fired_birth": share(fired["harmful"], fired["drawn"]),
                "fired_identical": fired["identical"], "fired_evaluated": fired["evaluated"],
            }
            v["fired_confirmed_vs_native_per_birth"] = ratio(v["fired_confirmed_per_fired_birth"], v["native_confirmed_per_birth"])
            v["fired_harmful_vs_native_per_birth"] = ratio(v["fired_harmful_per_fired_birth"], v["native_harmful_per_birth"])
            v["arm_confirmed_vs_native_per_birth"] = ratio(v["arm_confirmed_per_birth"], v["native_confirmed_per_birth"])
            block["variants"][variant] = v
            native_ref = native_ref or v["native_confirmed_per_birth"]
        vs = block["variants"]
        if "aligned" in vs:
            for other in ("scrambled", "shared"):
                if other in vs:
                    block["contrasts"][f"aligned_vs_{other}_fired_confirmed"] = ratio(
                        vs["aligned"]["fired_confirmed_per_fired_birth"], vs[other]["fired_confirmed_per_fired_birth"])
        block["bonferroni"] = {"contrasts": 6, "one_sided_alpha_each": 0.025 / 6,
                               "note": "the CP intervals above are 95 % two-sided; the contrast bounds are therefore at least as conservative as a one-sided 0.025 test per contrast, not the Bonferroni level; the per-contrast Bonferroni reading uses the same bounds and is reported as declared when the lower bound exceeds 2 by a margin the six-way correction would not erase (lower bound above 2.5)"}
        out["assays"][assay] = block
    json.dump(out, open(out_path, "w"), indent=1)
    for assay, block in out["assays"].items():
        for variant, v in block["variants"].items():
            print(f"{assay} {variant:9s} strata {v['strata']} pairs {v['pairs']} fired {v['fired_births']['count']} ({v['fired_births']['share']:.4f}) "
                  f"native conf {v['native_confirmed_per_birth']['share']:.4f} arm conf {v['arm_confirmed_per_birth']['share']:.4f} "
                  f"fired conf {v['fired_confirmed_per_fired_birth']['share']} fired harm {v['fired_harmful_per_fired_birth']['share']} "
                  f"ratio {v['fired_confirmed_vs_native_per_birth']}")
        print(assay, "contrasts", block["contrasts"])


if __name__ == "__main__":
    main()
