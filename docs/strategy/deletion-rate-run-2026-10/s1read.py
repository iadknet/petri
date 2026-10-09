"""Deletion-rate row S1 (docs/strategy/deletion-rate-run-2026-10.md): read every
census-probe run under <s1-dir>/<arm>-s<seed>/ (arms default, del-1.0, del-0.3,
del-0.03; seeds 1022, 2022, 3022, 4022), take the output-wiring reader's
window readings (ticks 15,000 to 20,000), typed target-family rows, P (the sum
over the three typed rows of family_executed + family_causal) and vote-delta
tallies at both checkpoints, and apply the plan's draft validation rule as
written, with the predeclared reading details.

usage: python3 s1read.py <s1-dir> <s0-summary.json or -> <out.json>"""
import hashlib
import json
import os
import sys

FOUNDER = 97
ARMS = ("default", "del-1.0", "del-0.3", "del-0.03")
SEEDS = (1022, 2022, 3022, 4022)
WINDOW = (15_000, 20_000)
CHECKPOINTS = ("10000", "20000")
TARGET_ROWS = ("AreaFoodSummary:0", "AreaFoodSummary:1", "NeighborBarrierRing")
FAMILY_FIELDS = ["family_declared", "family_connected", "family_executed", "family_causal",
                 "family_causal_original", "out_of_width_consumers"]
MIN_EFFECT = 5


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def num(x):
    return None if x == "-" else int(x)


def rows(path):
    return [json.loads(l) for l in open(path) if l.strip()]


def norm_family(name):
    return name.replace(" ", "")


def family_rows(cohort):
    out = {}
    for line in cohort["family_rows"]:
        parts = line.split(" ")
        if parts[0] in TARGET_ROWS:
            out[parts[0]] = dict(zip(FAMILY_FIELDS, map(num, parts[1:])))
    for name in TARGET_ROWS:
        out.setdefault(name, {k: 0 for k in FAMILY_FIELDS})
    return out


def vote_reading(block):
    parents = block or []
    channels = [(p["parent_index"], c) for p in parents for c in p["channels"]]
    s = lambda c, k: c["stats"].get(k)
    target = [(pi, c) for pi, c in channels if norm_family(c["family"]).split(":")[0] in ("AreaFoodSummary", "NeighborBarrierRing")]
    action = [(pi, c) for pi, c in target if s(c, "scenes_actions_differ") > 0]
    by_family = {}
    for pi, c in action:
        by_family.setdefault(norm_family(c["family"]), set()).add(pi)
    return {
        "parents": len(parents),
        "executed_target_channels": len(target),
        "channels_with_action_queue_change": len(action),
        "channels_with_any_vote_delta": sum(1 for pi, c in target if s(c, "delta_passes") > 0),
        "distinct_action_change_parents": len({pi for pi, c in action}),
        "action_change_parents_by_family": {f: sorted(v) for f, v in by_family.items()},
    }


def read_run(d, name):
    samples_path = os.path.join(d, name, f"samples-{name}.ndjson")
    census_path = os.path.join(d, name, f"census-{name}.ndjson")
    if not os.path.exists(samples_path):
        return None
    all_rows = rows(samples_path)
    samples = [r for r in all_rows if r.get("kind") == "sample"]
    footer = next((r for r in all_rows if r.get("kind") == "footer"), None)
    header = next((r for r in all_rows if r.get("kind") == "header"), None)
    extinct = next((r for r in all_rows if r.get("kind") == "extinct"), None)
    win = [s for s in samples if WINDOW[0] <= s["tick"] <= WINDOW[1]]
    alive = [s for s in win if s["population"] > 0]
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
                typed = family_rows(selected) if selected else None
                vote = vote_reading(r.get("vote_deltas"))
                evaluated = selected.get("parents_evaluated") if selected else None
                p = sum((v["family_executed"] or 0) + (v["family_causal"] or 0) for v in typed.values()) if typed else None
                agree = None
                if typed:
                    agree = {f: {"funnel_family_causal": typed[f]["family_causal"] or 0,
                                 "vote_delta_parents": len(vote["action_change_parents_by_family"].get(f, []))} for f in TARGET_ROWS}
                checkpoints[str(r["tick"])] = {
                    "living": r["living"],
                    "parents_evaluated": evaluated,
                    "target_rows": typed,
                    "P": p if evaluated == 20 else None,
                    "P_raw": p,
                    "vote_deltas": vote,
                    "agreement": agree,
                    "agreement_holds": None if agree is None else all(v["funnel_family_causal"] == v["vote_delta_parents"] for v in agree.values()),
                    "births": (r.get("births") or {}).get("pooled"),
                    "e6_gradient_informative": (r.get("e6") or {}).get("shares", {}).get("gradient_informative"),
                    "memory_either": (r.get("memory_sensitivity") or {}).get("different_from_either_count"),
                    "census_secs": r.get("census_secs"),
                }
    reaches = bool(footer and footer.get("ticks") == 20_000 and footer.get("extinct") is False and extinct is None)
    killed = footer is None
    mg, mgen = mean("mean_genome_size"), mean("mean_generation")
    return {
        "completed": footer is not None, "killed_or_incomplete": killed, "reaches_20000": reaches,
        "extinct_tick": extinct["tick"] if extinct else None,
        "prototypes": header.get("prototypes") if header else None,
        "config_digest": header.get("config_digest") if header else None,
        "samples": len(samples), "last_tick": samples[-1]["tick"] if samples else None,
        "mean_population": mean("population"),
        "mean_genome_size_founder_units": (mg / FOUNDER) if mg is not None else None,
        "mean_generation": mgen,
        "size_per_generation": (mg / mgen) if mg is not None and mgen else None,
        "births_per_creature_tick": births,
        "min_population_sample": min(((s["population"], s["tick"]) for s in samples), default=None),
        "final": {k: samples[-1][k] for k in ("tick", "population", "mean_genome_size", "mean_generation")} if samples else None,
        "trajectory": [(s["tick"], round(s["mean_genome_size"], 1), s["population"]) for s in samples if s["tick"] % 2500 == 0],
        "checkpoints": checkpoints,
        "footer": footer,
        "sha256_samples": sha(samples_path),
        "sha256_census": sha(census_path) if os.path.exists(census_path) else None,
    }


def main():
    d, s0_path, out_path = sys.argv[1:4]
    s0_go = None
    if s0_path != "-":
        s0_go = bool(json.load(open(s0_path)).get("go"))
    runs = {}
    for arm in ARMS:
        for seed in SEEDS:
            r = read_run(d, f"{arm}-s{seed}")
            if r is not None:
                runs[f"{arm}-s{seed}"] = r

    def run(arm, seed):
        return runs.get(f"{arm}-s{seed}")

    def P(arm, seed, t):
        r = run(arm, seed)
        if r is None:
            return None
        return (r["checkpoints"].get(t) or {}).get("P")

    per_seed = {}
    for s in SEEDS:
        cps = {}
        supports = True
        for t in CHECKPOINTS:
            pd, pc, pt = P("default", s, t), P("del-1.0", s, t), P("del-0.03", s, t)
            if None in (pd, pc, pt):
                cps[t] = {"P_default": pd, "P_del_1_0": pc, "P_del_0_03": pt, "P_del_0_3": P("del-0.3", s, t), "defined": False}
                supports = False
                continue
            n, e = abs(pc - pd), pt - pd
            ok = e >= MIN_EFFECT and e > n
            cps[t] = {"P_default": pd, "P_del_1_0": pc, "P_del_0_3": P("del-0.3", s, t), "P_del_0_03": pt, "N": n, "E": e,
                      "defined": True, "E_ge_5_and_gt_N": ok}
            supports &= ok
        dflt, low = run("default", s), run("del-0.03", s)
        dp = dflt["mean_population"] if dflt and dflt["reaches_20000"] else None
        lp = low["mean_population"] if low else None
        persists = bool(low and low["reaches_20000"])
        ratio = (lp / dp) if (dp and lp is not None) else None
        p20 = {a: P(a, s, "20000") for a in ("default", "del-0.3", "del-0.03")}
        dose = None if None in p20.values() else (p20["del-0.03"] >= p20["del-0.3"] >= p20["default"])
        vote20 = ((low or {}).get("checkpoints", {}).get("20000") or {}).get("vote_deltas") or {}
        vote_clause = (vote20.get("channels_with_action_queue_change") or 0) >= 1
        per_seed[str(s)] = {
            "checkpoints": cps, "supports": supports,
            "del_0_03_persists": persists, "default_reaches_20000": bool(dflt and dflt["reaches_20000"]),
            "population_ratio": ratio, "population_at_least_half": None if ratio is None else ratio >= 0.5,
            "dose_order_20000": dose, "P_20000": p20,
            "vote_delta_action_channels_del_0_03_20000": vote20.get("channels_with_action_queue_change"),
            "vote_clause": vote_clause,
            "min_population_del_0_03": (low or {}).get("min_population_sample"),
        }
    supporting = [s for s, v in per_seed.items() if v["supports"]]
    complete = all(f"{a}-s{s}" in runs for a in ARMS for s in SEEDS)
    clauses = {
        "s0_go": s0_go,
        "persists_all_four_with_half_population": all(v["del_0_03_persists"] and v["population_at_least_half"] for v in per_seed.values()),
        "supporting_seeds": len(supporting), "at_least_3_of_4_support": len(supporting) >= 3,
        "dose_order_3_of_4": sum(1 for v in per_seed.values() if v["dose_order_20000"]) >= 3,
        "vote_clause_on_every_supporting_seed": all(per_seed[s]["vote_clause"] for s in supporting),
    }
    validated = bool(complete and s0_go and clauses["persists_all_four_with_half_population"] and clauses["at_least_3_of_4_support"]
                     and clauses["dose_order_3_of_4"] and clauses["vote_clause_on_every_supporting_seed"])
    result = {"row": "S1", "world": "Canyon country", "window_ticks": list(WINDOW), "founder_units": FOUNDER, "min_effect": MIN_EFFECT,
              "complete": complete, "runs": runs, "per_seed": per_seed, "clauses": clauses, "validated": validated,
              "note": "lead-selection screen on 20 parents per cohort; the plan's draft rule as written with the predeclared reading details"}
    open(out_path, "w").write(json.dumps(result, indent=1) + "\n")
    for name, r in runs.items():
        cp = {t: (c["P"], c["vote_deltas"]["channels_with_action_queue_change"]) for t, c in r["checkpoints"].items()}
        print(name, "reaches" if r["reaches_20000"] else "NOT", "pop", None if r["mean_population"] is None else round(r["mean_population"]),
              "size x", None if r["mean_genome_size_founder_units"] is None else round(r["mean_genome_size_founder_units"], 2),
              "min", r["min_population_sample"], "P/action", cp)
    for s, v in per_seed.items():
        print(s, "supports" if v["supports"] else "no", {t: (c.get("N"), c.get("E")) for t, c in v["checkpoints"].items()},
              "ratio", v["population_ratio"], "dose", v["dose_order_20000"], "vote", v["vote_clause"])
    print("S1 complete:", complete, "validated:", validated, clauses)


if __name__ == "__main__":
    main()
