"""Diagnosis run row E10: descriptive reading of run 6's two completed main
batches (food-s12 at 1,000 generations, wall-s12 at 500), per the diagnosis
plan's ledger. Per arm: reached count, generation of reach, final best,
validation mean (last-generation rows), final elite genome size. ALT against N
(native-aa) by replicate: b, c and run 4 rule 4's bound U from the frozen
stat6.py; R against N discordance; the ALT draw's parity from F1. Descriptive:
8 pairs decide nothing.

usage: python3 e10read.py <run6-dir> <alt-draws.ndjson> <out.json>"""
import hashlib
import json
import os
import re
import sys

sys.path.insert(0, sys.argv[1])
import stat6 as s  # noqa: E402

ARMS = ("native", "native-aa", "alt", "founder-only", "mutation-off", "shuffled-score", "comparator", "random-walk")
CHECKPOINTS = (100, 250, 500, 1000)
GEN_RE = re.compile(r'"arm":"([^"]+)","role":"[^"]+","policy":"[^"]+","replicate":(\d+),"generation":(\d+)')
VAL_RE = re.compile(r'"validation_mean":(null|[-0-9.eE+]+)')
BEST_RE = re.compile(r'"best":([-0-9.eE+]+)')


def median(xs):
    """Ordinary median: the mean of the two middle values when the count is even
    (review (d), finding 3: the first version took the upper middle value)."""
    if not xs:
        return None
    s = sorted(xs)
    n = len(s)
    return s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def last_rows(path):
    """(arm, replicate) -> (generation, validation_mean, best) of the last row."""
    out = {}
    with open(path) as f:
        for line in f:
            m = GEN_RE.search(line[:400])
            if not m:
                continue
            key, gen = (m.group(1), int(m.group(2))), int(m.group(3))
            if key in out and out[key][0] >= gen:
                continue
            v = VAL_RE.search(line)
            b = BEST_RE.search(line[:600])
            val = None if v is None or v.group(1) == "null" else float(v.group(1))
            out[key] = (gen, val, float(b.group(1)) if b else None)
    return out


def read_batch(d, assay, seed, horizon):
    summ = json.load(open(f"{d}/summary.json"))
    rows = last_rows(f"{d}/rows.ndjson")
    arms = {a["name"]: a for a in summ["arms"]}
    per_arm = {}
    for name in ARMS:
        a = arms[name]
        reps = []
        for r in a["replicates"]:
            last = rows.get((name, r["replicate"]), (None, None, None))
            reps.append({
                "replicate": r["replicate"], "reached": r["reached"],
                "generation_to_threshold": r.get("generation_to_threshold"),
                "generations_run": r.get("generations_run"), "incomplete": r["incomplete"],
                "final_best": r.get("final_best"), "last_row_generation": last[0],
                "validation_mean": last[1],
                "final_genome_size": ((r.get("readings") or {}).get("last") or {}).get("shape", {}).get("genome_size"),
            })
        sizes = [x["final_genome_size"] for x in reps if x["final_genome_size"] is not None]
        vals = [x["validation_mean"] for x in reps if x["validation_mean"] is not None]
        curve = {}
        for g in CHECKPOINTS:
            if g > horizon:
                continue
            curve[str(g)] = sum(1 for x in reps if x["reached"] and x["generation_to_threshold"] is not None
                                and x["generation_to_threshold"] < g)
        per_arm[name] = {
            "role": a["role"], "policy": a["policy"], "reached": sum(1 for x in reps if x["reached"]),
            "of": len(reps), "incomplete": sum(1 for x in reps if x["incomplete"]),
            "reached_fraction": a.get("reached_fraction"), "wilson_95": a.get("wilson_95"),
            "generations_of_reach": [x["generation_to_threshold"] for x in reps if x["reached"]],
            "final_best": [x["final_best"] for x in reps],
            "final_best_mean": sum(x["final_best"] for x in reps) / len(reps),
            "validation_mean": vals, "validation_mean_mean": sum(vals) / len(vals) if vals else None,
            "final_genome_size": sizes,
            "final_genome_size_median": median(sizes),
            "cumulative_reached_before_generation": curve, "replicates": reps,
        }

    def contrast(x, n_):
        b = c = 0
        for rx, rn in zip(per_arm[x]["replicates"], per_arm[n_]["replicates"]):
            if rx["reached"] is True and rn["reached"] is False:
                b += 1
            elif rn["reached"] is True and rx["reached"] is False:
                c += 1
        n = len(per_arm[x]["replicates"])
        return {"n": n, "b": b, "c": c, "excess": (b - c) / n,
                "U": s.cp_upper(b, n) - s.cp_lower(c, n),
                "under_0_25": s.cp_upper(b, n) - s.cp_lower(c, n) < 0.25}

    gen_ok = summ["provenance"]["sizes"]["generations"] == horizon
    genomes = {g["name"]: g["sha256"] for g in summ["provenance"]["genomes"]}
    return {
        "batch": f"{assay}-s{seed}", "assay": summ["assay"], "horizon": horizon, "horizon_matches_frozen": gen_ok,
        "exit_code": summ["exit_code"], "incomplete": summ["incomplete"],
        "calibration_verdict": summ["calibration"]["verdict"],
        "sizes": summ["provenance"]["sizes"], "genome_sha256": genomes,
        "wall_seconds": summ.get("timing", {}).get("wall_seconds"),
        "alt_vs_n": contrast("alt", "native-aa"), "r_vs_n": contrast("native", "native-aa"),
        "arms": per_arm,
        "provenance": {"summary_sha256": sha(f"{d}/summary.json"), "rows_sha256": sha(f"{d}/rows.ndjson"),
                       "rows_bytes": os.path.getsize(f"{d}/rows.ndjson")},
    }


def main():
    run6, draws_path, out_path = sys.argv[1:4]
    freeze = json.load(open(f"{run6}/freeze.json"))
    draws = {(d["assay"], d["seed"]): d for d in map(json.loads, open(draws_path))}
    batches = {}
    for assay in ("food", "wall"):
        b = read_batch(f"{run6}/main/{assay}-s12", assay, 12, freeze["horizon"][assay])
        d = draws[(assay, 12)]
        b["alt_draw"] = {"genome_sha256": d["genome_sha256"], "frozen_match": d["genome_sha256"] == freeze["alt_sha256"][f"{assay}-s12"]
                         and b["genome_sha256"].get("arm:alt") == d["genome_sha256"],
                         "genome_size_after": d["genome_size_after"], "parity": d["parity"]}
        batches[assay] = b
    both = all(batches[a]["alt_vs_n"]["b"] - batches[a]["alt_vs_n"]["c"] >= 2 for a in batches)
    out = {
        "row": "E10", "kind": "descriptive", "source": "run 6 main batches food-s12 and wall-s12 (the only completed ones)",
        "decision_rule": "a clear ALT excess on both assays (b - c >= 2 of 8 on each) reorders E5 behind a dense-afferent variant",
        "alt_excess_on_both": both,
        "batches": batches,
        "reader_sha256": sha(os.path.abspath(__file__)), "stat6_sha256": sha(f"{run6}/stat6.py"),
    }
    json.dump(out, open(out_path, "w"), indent=1)
    for a, b in batches.items():
        print(a, "alt_vs_n", b["alt_vs_n"], "r_vs_n", b["r_vs_n"])
        for name in ARMS:
            p = b["arms"][name]
            print(f"  {name:15s} reached {p['reached']}/{p['of']} gens {p['generations_of_reach']} best_mean {p['final_best_mean']:.3f} "
                  f"val_mean {p['validation_mean_mean']} size_median {p['final_genome_size_median']} sizes {p['final_genome_size']}")
    print("alt_excess_on_both", both)


if __name__ == "__main__":
    main()
