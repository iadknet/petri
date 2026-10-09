"""Gate B0.2 (i): the census probe's inline samples (prototype binary, flag
off) against a `v3-cli run` samples stream of the launch-revision production
binary, strictly: the expected tick set on both sides, every compared field
present, equal values at every tick, both runs completed to the last tick,
and the census header's config_digest equal to the run's run_started digest.

usage: python3 b02check.py <probe samples.ndjson> <probe census.ndjson> <v3-cli run.ndjson> <last tick> <out.json>"""
import hashlib
import json
import sys

FIELDS = ("population", "mean_genome_size", "mean_generation", "reproduction_actions_spawned_total")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def rows(path):
    out = []
    for line in open(path):
        line = line.strip()
        if line.startswith("{"):
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                pass
    return out


def main():
    samples_path, census_path, run_path, last, out = sys.argv[1:6]
    last = int(last)
    expected = set(range(500, last + 1, 500))
    probe = rows(samples_path)
    census = rows(census_path)
    run = rows(run_path)
    checks = []

    def check(name, ok, detail=None):
        checks.append({"check": name, "pass": bool(ok), "detail": detail})

    p_samples = {r["tick"]: r for r in probe if r.get("kind") == "sample"}
    r_samples = {r["tick"]: r for r in run if r.get("event_type") == "tick_sample"}
    check("probe tick set equals expected", set(p_samples) == expected, {"probe": sorted(p_samples), "expected": sorted(expected)})
    check("run tick set equals expected", set(r_samples) == expected, {"run": sorted(r_samples)})
    missing = [(t, f) for t in expected for f in FIELDS if t in p_samples and f not in p_samples[t]]
    missing += [(t, f) for t in expected for f in FIELDS if t in r_samples and f not in r_samples[t]]
    check("every compared field present on both sides", not missing, {"missing": missing})
    mismatches = []
    for t in sorted(expected):
        if t in p_samples and t in r_samples:
            for f in FIELDS:
                if f in p_samples[t] and f in r_samples[t] and p_samples[t][f] != r_samples[t][f]:
                    mismatches.append({"tick": t, "field": f, "probe": p_samples[t][f], "run": r_samples[t][f]})
    check("every compared value equal at every tick", not mismatches, {"mismatches": mismatches})
    footer = next((r for r in probe if r.get("kind") == "footer"), None)
    check("probe completed to the last tick", footer is not None and footer.get("ticks") == last and not footer.get("extinct"), footer)
    completed = next((r for r in run if r.get("event_type") == "run_completed"), None)
    check("run completed", completed is not None, completed)
    p_header = next((r for r in census if r.get("kind") == "header"), None)
    r_started = next((r for r in run if r.get("event_type") == "run_started"), None)
    check("config digests equal", p_header is not None and r_started is not None and p_header.get("config_digest") == r_started.get("config_digest"),
          {"probe": (p_header or {}).get("config_digest"), "run": (r_started or {}).get("config_digest")})
    result = {"gate": "B0.2 (i) smoke", "probe_samples": samples_path, "run": run_path, "last_tick": last,
              "sha256": {"probe_samples": sha(samples_path), "probe_census": sha(census_path), "run": sha(run_path)},
              "checks": checks, "pass": all(c["pass"] for c in checks)}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("B0.2 smoke", "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
