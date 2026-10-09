"""Deletion-rate run, descriptive drift check (not a gate): the frozen census's
`default` samples against the output-wiring run's `default` samples, tick for
tick on population, mean genome size, mean generation and cumulative spawns.
Equal means that run's numbers (and the born run's it reproduced) describe
today's production after `38ba9906`.

usage: python3 driftcheck.py <wiring samples.ndjson> <frozen samples.ndjson> <out.json>"""
import json
import sys

FIELDS = ("population", "mean_genome_size", "mean_generation", "reproduction_actions_spawned_total")


def samples(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("kind") == "sample":
            out[r["tick"]] = r
    return out


a, b, out = sys.argv[1:4]
wa, fb = samples(a), samples(b)
diff = [{"tick": t, "field": f, "wiring": wa[t][f], "frozen": fb[t][f]}
        for t in sorted(set(wa) & set(fb)) for f in FIELDS if wa[t][f] != fb[t][f]]
result = {"wiring": a, "frozen": b, "ticks_compared": len(set(wa) & set(fb)), "tick_sets_equal": set(wa) == set(fb),
          "differences": diff[:50], "difference_count": len(diff), "equal": set(wa) == set(fb) and not diff}
open(out, "w").write(json.dumps(result, indent=1) + "\n")
print("drift", "equal" if result["equal"] else f"DIFFERENT ({len(diff)} field differences)", result["ticks_compared"], "ticks")
