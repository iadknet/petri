"""Diagnosis follow-up rows N1 and N2: read every census-probe run under
<n12-dir> (arms default, hold2, slow and their mutation-off pairs, seeds 1022
and 2022), apply E8's window readings over ticks 15,000 to 20,000 (mean
population, mean genome size in founder units, births per creature-tick,
extinction interval), extract the typed target-family funnel rows of the
selected cohort at every checkpoint, and apply the rows' decision rules.

usage: python3 n12read.py <n12-dir> <out.json>"""
import hashlib
import json
import os
import sys

FOUNDER = 97
ARMS = ("default", "hold2", "slow", "default-mutoff", "hold2-mutoff", "slow-mutoff")
SEEDS = (1022, 2022)
WINDOW = (15_000, 20_000)
TARGET_ROWS = ("AreaFoodSummary:0", "AreaFoodSummary:1", "NeighborBarrierRing")
FAMILY_FIELDS = ["family_declared", "family_connected", "family_executed", "family_causal",
                 "family_causal_original", "out_of_width_consumers"]


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def num(x):
    return None if x == "-" else int(x)


def rows(path):
    return [json.loads(l) for l in open(path) if l.strip()]


def family_rows(cohort):
    out = {}
    for line in cohort["family_rows"]:
        parts = line.split(" ")
        if parts[0] in TARGET_ROWS:
            out[parts[0]] = dict(zip(FAMILY_FIELDS, map(num, parts[1:])))
    for name in TARGET_ROWS:
        out.setdefault(name, {k: 0 for k in FAMILY_FIELDS})
    for e in out.values():
        e["declared_unconnected"] = (e["family_declared"] or 0) - (e["family_connected"] or 0)
    return out


def read_run(d, name):
    samples_path = os.path.join(d, name, f"samples-{name}.ndjson")
    census_path = os.path.join(d, name, f"census-{name}.ndjson")
    if not os.path.exists(samples_path):
        return None
    samples = [r for r in rows(samples_path) if r.get("kind") == "sample"]
    footer = next((r for r in rows(samples_path) if r.get("kind") == "footer"), None)
    header = next((r for r in rows(samples_path) if r.get("kind") == "header"), None)
    extinct = next((r for r in rows(samples_path) if r.get("kind") == "extinct"), None)
    win = [s for s in samples if WINDOW[0] <= s["tick"] <= WINDOW[1]]
    alive = [s for s in win if s["population"] > 0]
    last_alive = max((s["tick"] for s in samples if s["population"] > 0), default=None)
    births = None
    if len(alive) >= 2:
        d_spawn = alive[-1]["reproduction_actions_spawned_total"] - alive[0]["reproduction_actions_spawned_total"]
        creature_ticks = sum(s["population"] for s in alive[:-1]) * 500
        births = d_spawn / creature_ticks if creature_ticks else None
    mean = lambda k: (sum(s[k] for s in alive) / len(alive)) if alive else None
    checkpoints = {}
    if os.path.exists(census_path):
        for r in rows(census_path):
            if r.get("kind") == "checkpoint":
                selected = next((c for c in r["input_use"]["cohorts"] if c["cohort"] == "selected"), None)
                checkpoints[str(r["tick"])] = {
                    "living": r["living"],
                    "selected_parents": r["selected_parents"],
                    "parents_evaluated": selected.get("parents_evaluated") if selected else None,
                    "target_rows": family_rows(selected) if selected else None,
                    "e6_gradient_informative": (r.get("e6") or {}).get("shares", {}).get("gradient_informative"),
                }
    return {
        "completed": footer is not None,
        "extinct_tick": extinct["tick"] if extinct else None,
        "prototypes": header.get("prototypes") if header else None,
        "samples": len(samples),
        "last_tick": samples[-1]["tick"] if samples else None,
        "window_alive_samples": len(alive),
        "mean_population": mean("population"),
        "mean_energy": mean("mean_energy"),
        "mean_genome_size": mean("mean_genome_size"),
        "mean_genome_size_founder_units": (mean("mean_genome_size") / FOUNDER) if alive else None,
        "mean_generation": mean("mean_generation"),
        "births_per_creature_tick": births,
        "extinction_interval": None if extinct is None else [last_alive, extinct["tick"]],
        "final": {k: samples[-1][k] for k in ("tick", "population", "mean_genome_size", "mean_generation",
                                                  "reproduction_actions_spawned_total")} if samples else None,
        "trajectory": [(s["tick"], round(s["mean_genome_size"], 1), s["population"]) for s in samples if s["tick"] % 2500 == 0],
        "checkpoints": checkpoints,
        "footer": footer,
        "sha256_samples": sha(samples_path),
        "sha256_census": sha(census_path) if os.path.exists(census_path) else None,
    }


def main():
    d, out_path = sys.argv[1:3]
    runs = {}
    for arm in ARMS:
        for seed in SEEDS:
            r = read_run(d, f"{arm}-s{seed}")
            if r is not None:
                runs[f"{arm}-s{seed}"] = r

    def val(arm, seed, k):
        r = runs.get(f"{arm}-s{seed}")
        return None if r is None else r.get(k)

    def reaches(arm, seed):
        r = runs.get(f"{arm}-s{seed}")
        return bool(r and r["completed"] and r["extinct_tick"] is None and r["last_tick"] == 20_000)

    def declared(arm, seed, row):
        r = runs.get(f"{arm}-s{seed}")
        cp = (r or {}).get("checkpoints", {}).get("20000")
        if not cp or not cp.get("target_rows"):
            return None
        return cp["target_rows"][row]["family_declared"]

    n1 = {"per_seed": {}}
    for s in SEEDS:
        ds, hs = val("default", s, "mean_genome_size_founder_units"), val("hold2", s, "mean_genome_size_founder_units")
        dp, hp = val("default", s, "mean_population"), val("hold2", s, "mean_population")
        n1["per_seed"][str(s)] = {
            "hold2_reaches_20000": reaches("hold2", s),
            "size_below_default": None if None in (ds, hs) else hs < ds,
            "population_ratio": None if not dp or hp is None else hp / dp,
            "population_at_least_half": None if not dp or hp is None else hp >= 0.5 * dp,
            "births_default": val("default", s, "births_per_creature_tick"),
            "births_hold2": val("hold2", s, "births_per_creature_tick"),
            "pair": {"default_mutoff_extinction": val("default-mutoff", s, "extinction_interval"),
                     "hold2_mutoff_extinction": val("hold2-mutoff", s, "extinction_interval")},
        }
    n1["validated"] = all(v["hold2_reaches_20000"] and v["size_below_default"] and v["population_at_least_half"]
                          for v in n1["per_seed"].values()) if len(n1["per_seed"]) == 2 else None

    n2 = {"per_seed": {}, "typed_rows": {}}
    for s in SEEDS:
        dp, sp = val("default", s, "mean_population"), val("slow", s, "mean_population")
        n2["per_seed"][str(s)] = {
            "slow_reaches_20000": reaches("slow", s),
            "population_ratio": None if not dp or sp is None else sp / dp,
            "population_at_least_half": None if not dp or sp is None else sp >= 0.5 * dp,
            "size_default": val("default", s, "mean_genome_size_founder_units"),
            "size_slow": val("slow", s, "mean_genome_size_founder_units"),
            "pair": {"default_mutoff_extinction": val("default-mutoff", s, "extinction_interval"),
                     "slow_mutoff_extinction": val("slow-mutoff", s, "extinction_interval")},
        }
    for row in TARGET_ROWS:
        n2["typed_rows"][row] = {
            str(s): {"default": declared("default", s, row), "slow": declared("slow", s, row),
                     "slow_above_default": None if None in (declared("default", s, row), declared("slow", s, row))
                     else declared("slow", s, row) > declared("default", s, row)}
            for s in SEEDS
        }
    any_row_above_on_both = any(all((v[str(s)]["slow_above_default"] is True) for s in SEEDS) for v in n2["typed_rows"].values())
    n2["any_typed_row_above_default_on_both_seeds"] = any_row_above_on_both
    n2["validated_given_g3"] = (all(v["slow_reaches_20000"] and v["population_at_least_half"] for v in n2["per_seed"].values())
                                and any_row_above_on_both) if len(n2["per_seed"]) == 2 else None

    result = {"rows": ["N1", "N2"], "world": "Canyon country", "window_ticks": list(WINDOW), "founder_units": FOUNDER,
              "runs": runs, "n1": n1, "n2": n2}
    open(out_path, "w").write(json.dumps(result, indent=1) + "\n")
    for name, r in runs.items():
        print(name, "completed" if r["completed"] else "incomplete", "extinct", r["extinct_tick"],
              "pop", None if r["mean_population"] is None else round(r["mean_population"]),
              "size x", None if r["mean_genome_size_founder_units"] is None else round(r["mean_genome_size_founder_units"], 2),
              "births", None if r["births_per_creature_tick"] is None else round(r["births_per_creature_tick"], 5))
    print("N1", n1["validated"], "N2 (given G3)", n2["validated_given_g3"])


if __name__ == "__main__":
    main()
