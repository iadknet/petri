"""Gate G1: compare a census-probe run's inline samples with an E8 `v3-cli run`
samples file at every common tick on population, mean_genome_size,
mean_generation and reproduction_actions_spawned_total.

usage: python3 g1check.py <probe samples-*.ndjson> <e8 samples.ndjson> [out.json]"""
import json
import sys

FIELDS = ("population", "mean_genome_size", "mean_generation", "reproduction_actions_spawned_total")


def load(path, kind, key):
    rows = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        if r.get(key) == kind:
            rows[r["tick"]] = r
    return rows


def main():
    probe = load(sys.argv[1], "sample", "kind")
    e8 = load(sys.argv[2], "tick_sample", "event_type")
    common = sorted(set(probe) & set(e8))
    mismatches = []
    for tick in common:
        for f in FIELDS:
            a, b = probe[tick].get(f), e8[tick].get(f)
            if a != b:
                mismatches.append({"tick": tick, "field": f, "probe": a, "e8": b})
    result = {
        "gate": "G1",
        "probe": sys.argv[1],
        "e8": sys.argv[2],
        "common_ticks": len(common),
        "first_tick": common[0] if common else None,
        "last_tick": common[-1] if common else None,
        "probe_only_ticks": sorted(set(probe) - set(e8)),
        "e8_only_ticks": sorted(set(e8) - set(probe)),
        "mismatches": mismatches,
        "equal": not mismatches and bool(common),
    }
    text = json.dumps(result, indent=1)
    if len(sys.argv) > 3:
        open(sys.argv[3], "w").write(text + "\n")
    print(json.dumps({k: result[k] for k in ("common_ticks", "first_tick", "last_tick", "equal")}),
          "mismatches:", len(mismatches))
    sys.exit(0 if result["equal"] else 1)


if __name__ == "__main__":
    main()
