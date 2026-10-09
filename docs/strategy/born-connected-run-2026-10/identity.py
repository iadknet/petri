"""Gates B0.2 (ii) and B0.3: two census-probe runs compared for identity on
every sample row (the compared fields), the extinction tick, the footer's
ticks and extinct flag, and every checkpoint block (serialized once with
sorted keys and compact separators, hashed), timing fields excluded; the
header compared on every key except the allowed differences, which must
differ when the mode says so.

usage: python3 identity.py <mode: equal|flag-differs> <ref dir> <ref name> <cand dir> <cand name> <out.json>"""
import hashlib
import json
import sys

SAMPLE_FIELDS = ("population", "mean_genome_size", "mean_generation", "mean_age", "mean_energy", "births_total",
                 "reproduction_actions_spawned_total", "mutation_events_applied_total")
BLOCKS = ("living", "selected_parents", "selected_generations", "age_deciles", "energy_deciles", "sample",
          "input_use", "e6", "vote_deltas", "births")
TIMING = ("census_secs", "sim_secs_so_far", "vote_delta_secs", "births_secs")
# `world` is the run's label (PETRI_E2_NAME), which names the arm (amendment A3).
ALLOWED_HEADER = ("config_digest", "prototypes", "recipe", "world")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def rows(path):
    return [json.loads(l) for l in open(path) if l.strip().startswith("{")]


def canon(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def load(d, name):
    samples = rows(f"{d}/samples-{name}.ndjson")
    census = rows(f"{d}/census-{name}.ndjson")
    return {
        "samples": {r["tick"]: r for r in samples if r.get("kind") == "sample"},
        "extinct": next((r["tick"] for r in samples if r.get("kind") == "extinct"), None),
        "footer": next((r for r in samples if r.get("kind") == "footer"), None),
        "header": next((r for r in census if r.get("kind") == "header"), None),
        "checkpoints": {r["tick"]: r for r in census if r.get("kind") == "checkpoint"},
        "sha": {"samples": sha(f"{d}/samples-{name}.ndjson"), "census": sha(f"{d}/census-{name}.ndjson")},
    }


def main():
    mode, rd, rn, cd, cn, out = sys.argv[1:7]
    ref, cand = load(rd, rn), load(cd, cn)
    checks = []

    def check(name, ok, detail=None):
        checks.append({"check": name, "pass": bool(ok), "detail": detail})

    check("sample tick sets equal", set(ref["samples"]) == set(cand["samples"]),
          {"ref": sorted(ref["samples"]), "cand": sorted(cand["samples"])})
    mism = []
    for t in sorted(set(ref["samples"]) & set(cand["samples"])):
        for f in SAMPLE_FIELDS:
            if ref["samples"][t].get(f) != cand["samples"][t].get(f):
                mism.append({"tick": t, "field": f, "ref": ref["samples"][t].get(f), "cand": cand["samples"][t].get(f)})
    check("every sample field equal at every tick", not mism, {"mismatches": mism[:20], "count": len(mism)})
    check("extinction tick equal", ref["extinct"] == cand["extinct"], {"ref": ref["extinct"], "cand": cand["extinct"]})
    rf, cf = ref["footer"] or {}, cand["footer"] or {}
    check("footer ticks and extinct equal", (rf.get("ticks"), rf.get("extinct")) == (cf.get("ticks"), cf.get("extinct")),
          {"ref": {k: rf.get(k) for k in ("ticks", "extinct")}, "cand": {k: cf.get(k) for k in ("ticks", "extinct")}})
    check("checkpoint tick sets equal", set(ref["checkpoints"]) == set(cand["checkpoints"]),
          {"ref": sorted(ref["checkpoints"]), "cand": sorted(cand["checkpoints"])})
    block_hashes = {}
    bad = []
    flags = []
    for t in sorted(set(ref["checkpoints"]) & set(cand["checkpoints"])):
        for b in BLOCKS:
            vr, vc = ref["checkpoints"][t].get(b), cand["checkpoints"][t].get(b)
            if b == "births":
                # The births block embeds the arm's mutation_config (the flag
                # itself); it is compared apart (amendment A2), the counts hashed.
                vr = {k: v for k, v in (vr or {}).items() if k != "mutation_config"}
                vc = {k: v for k, v in (vc or {}).items() if k != "mutation_config"}
                fr = ((ref["checkpoints"][t].get("births") or {}).get("mutation_config") or {}).get("born_connected")
                fc = ((cand["checkpoints"][t].get("births") or {}).get("mutation_config") or {}).get("born_connected")
                flags.append({"tick": t, "ref": fr, "cand": fc})
            hr, hc = canon(vr), canon(vc)
            block_hashes[f"{t}:{b}"] = {"ref": hr, "cand": hc}
            if hr != hc:
                bad.append(f"{t}:{b}")
        extra = set(ref["checkpoints"][t]) ^ set(cand["checkpoints"][t])
        if extra - set(TIMING):
            bad.append(f"{t}:keys:{sorted(extra)}")
    check("every timing-free checkpoint block hash-equal (births counts without mutation_config)", not bad, {"differing": bad})
    # A run extinct before its first checkpoint has no births block; the
    # flag comparison is then vacuous and says so (amendment A3).
    if not flags:
        check("births block mutation_config.born_connected: no checkpoint reached on either run (vacuous)",
              not ref["checkpoints"] and not cand["checkpoints"], {"ref_extinct": ref["extinct"], "cand_extinct": cand["extinct"]})
    elif mode == "equal":
        check("births block mutation_config.born_connected equal at every checkpoint", all(f["ref"] == f["cand"] for f in flags), flags)
    else:
        check("births block mutation_config.born_connected differs (false vs true) at every checkpoint",
              all(f["ref"] is False and f["cand"] is True for f in flags), flags)
    rh, ch = ref["header"] or {}, cand["header"] or {}
    diff = sorted(k for k in set(rh) | set(ch) if rh.get(k) != ch.get(k))
    disallowed = [k for k in diff if k not in ALLOWED_HEADER]
    check("header equal outside the allowed keys", not disallowed, {"differing": diff, "disallowed": disallowed})
    if mode == "equal":
        check("header allowed keys equal too (same binary policy, same recipe)", not [k for k in diff if k in ("config_digest", "prototypes")],
              {"differing": diff})
    else:
        check("config_digest differs (the flag serializes when true)", rh.get("config_digest") != ch.get("config_digest"),
              {"ref": rh.get("config_digest"), "cand": ch.get("config_digest")})
        check("prototypes.born_connected differs", (rh.get("prototypes") or {}).get("born_connected") != (ch.get("prototypes") or {}).get("born_connected"),
              {"ref": rh.get("prototypes"), "cand": ch.get("prototypes")})
    result = {"mode": mode, "ref": {"dir": rd, "name": rn, "sha256": ref["sha"]}, "cand": {"dir": cd, "name": cn, "sha256": cand["sha"]},
              "checks": checks, "pass": all(c["pass"] for c in checks), "block_hashes": block_hashes}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("identity", rn, "vs", cn, "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
