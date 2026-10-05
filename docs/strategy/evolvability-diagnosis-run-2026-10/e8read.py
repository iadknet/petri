"""Diagnosis run row E8: read the Canyon production runs (`v3-cli run` samples
every 500 ticks) for every arm and seed and apply the row's readings over
ticks 15,000 to 20,000 (samples 30 to 40): mean population, mean genome size
in founder units (97), births per creature-tick (delta spawns over
population-ticks), extinction interval, plus the immediate-effect pair
(hold-mutoff against default-mutoff) and the decision rule (default past
10 x founder by 20,000 ticks moves the size defect ahead of the encoding
defect).

usage: python3 e8read.py <e8-dir> <out.json>"""
import hashlib
import json
import os
import sys

FOUNDER = 97
ARMS = ("default", "x4", "hold", "default-mutoff", "hold-mutoff")
SEEDS = (1022, 2022)
WINDOW = (15_000, 20_000)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read_run(path):
    rows = [json.loads(l) for l in open(path) if l.strip()]
    samples = [r for r in rows if r.get("event_type") == "tick_sample"]
    completed = any(r.get("event_type") == "run_completed" for r in rows)
    win = [s for s in samples if WINDOW[0] <= s["tick"] <= WINDOW[1]]
    alive = [s for s in win if s["population"] > 0]
    last_alive = max((s["tick"] for s in samples if s["population"] > 0), default=None)
    first_zero = min((s["tick"] for s in samples if s["population"] == 0), default=None)
    births = None
    if len(alive) >= 2:
        d_spawn = alive[-1]["reproduction_actions_spawned_total"] - alive[0]["reproduction_actions_spawned_total"]
        creature_ticks = sum(s["population"] for s in alive[:-1]) * 500
        births = d_spawn / creature_ticks if creature_ticks else None
    mean = lambda k: (sum(s[k] for s in alive) / len(alive)) if alive else None
    return {
        "completed": completed, "samples": len(samples), "last_tick": samples[-1]["tick"] if samples else None,
        "window_samples": len(win), "window_alive_samples": len(alive),
        "mean_population": mean("population"), "mean_energy": mean("mean_energy"),
        "mean_genome_size": mean("mean_genome_size"),
        "mean_genome_size_founder_units": (mean("mean_genome_size") / FOUNDER) if alive else None,
        "mean_generation": mean("mean_generation"),
        "births_per_creature_tick": births,
        "extinction_interval": None if first_zero is None else [last_alive, first_zero],
        "final": {k: samples[-1][k] for k in ("tick", "population", "mean_genome_size", "mean_generation",
                                                  "reproduction_actions_spawned_total", "mutation_events_applied_total")} if samples else None,
        "trajectory_size": [(s["tick"], round(s["mean_genome_size"], 1), s["population"]) for s in samples if s["tick"] % 2500 == 0],
        "sha256": sha(path),
    }


def main():
    d, out_path = sys.argv[1:3]
    runs = {}
    for arm in ARMS:
        for seed in SEEDS:
            p = os.path.join(d, f"{arm}-s{seed}", "samples.ndjson")
            if os.path.exists(p):
                r = read_run(p)
                applied = os.path.join(d, f"{arm}-s{seed}", "applied.json")
                if os.path.exists(applied):
                    a = json.load(open(applied))
                    r["applied"] = {"per_unit_rate": a["mutation"]["per_unit_rate"],
                                    "hold": a["energy"]["lifecycle"].get("reproduce_hold_per_founder_size")}
                runs[f"{arm}-s{seed}"] = r

    def val(arm, seed, k):
        r = runs.get(f"{arm}-s{seed}")
        return None if r is None else r.get(k)
    reading = {}
    for k in ("mean_genome_size_founder_units", "births_per_creature_tick", "mean_population"):
        reading[k] = {arm: {str(s): val(arm, s, k) for s in SEEDS} for arm in ARMS}
    default_units = [val("default", s, "mean_genome_size_founder_units") for s in SEEDS]
    size_defect_ahead = all(u is not None and u > 10 for u in default_units)
    immediate = {}
    for s in SEEDS:
        a, b = runs.get(f"hold-mutoff-s{s}"), runs.get(f"default-mutoff-s{s}")
        if a and b:
            immediate[str(s)] = {k: (a[k], b[k]) for k in ("mean_population", "births_per_creature_tick", "mean_energy")}
    out = {"row": "E8", "world": "Canyon country", "window_ticks": WINDOW, "founder_units": FOUNDER,
           "runs": runs, "reading": reading, "default_past_10x_founder_both_seeds": size_defect_ahead,
           "immediate_effect_hold_vs_default_mutoff": immediate,
           "complete": all(f"{a}-s{s}" in runs and runs[f"{a}-s{s}"]["completed"] for a in ARMS for s in SEEDS),
           "reader_sha256": sha(__file__)}
    json.dump(out, open(out_path, "w"), indent=1)
    for k, r in runs.items():
        print(f"{k:22s} done {r['completed']} pop {r['mean_population']} size/97 {r['mean_genome_size_founder_units']} "
              f"births/ct {r['births_per_creature_tick']} ext {r['extinction_interval']} applied {r.get('applied')}")
    print("default past 10x founder on both seeds:", size_defect_ahead)


if __name__ == "__main__":
    main()
