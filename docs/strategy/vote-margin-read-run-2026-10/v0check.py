"""Gate V0 (vote-margin read run): the v1 census and the v2 census on the
same world, seed and checkpoints, compared for identity. Blocking: every
sample row (the compared fields, each required present), the extinction
record, the footer (ticks and extinct flag), the checkpoint tick sets
derived from each header and its own terminal record, and at every reached
checkpoint the `input_use` and `e6` blocks and the v1 whitelist projection
of `vote_deltas`, each serialized once with sorted keys and compact
separators and hashed. The header must be equal on every nested value
except `vote_delta_version` and keys only v2 carries (and the recipe and
label). The other checkpoint blocks are hashed and reported, not blocking.
Also reports, per checkpoint, the v1 block's executed target channels (the
extension rule) and v2's block size.

usage: python3 v0check.py <v1 dir> <v1 name> <v2 dir> <v2 name> <out.json>"""
import hashlib
import json
import sys

SAMPLE_FIELDS = ("population", "mean_genome_size", "mean_generation", "mean_age", "mean_energy", "births_total",
                 "reproduction_actions_spawned_total", "mutation_events_applied_total")
BLOCKING = ("input_use", "e6")
REPORTED = ("living", "selected_parents", "selected_generations", "age_deciles", "energy_deciles", "sample",
            "births", "sink_use", "memory_loop", "memory_sensitivity", "temporal_memory_sensitivity")
PARENT_KEYS = ("parent_index", "generation", "genome_size", "nodes", "reachable_nodes", "executed_target_channels",
               "channels", "families")
CHANNEL_KEYS = ("family", "channel", "consumers_graph", "consumers_graph_abs_weight", "consumers_vm", "stats")
FAMILY_KEYS = ("family", "channels", "stats", "l1_subadditivity_ratio", "additive_residual_ratio",
               "channels_positive_on_most_moved", "channels_negative_on_most_moved",
               "most_moved_family_over_max_channel")
HEADER_MAY_DIFFER = ("vote_delta_version", "recipe", "world")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def rows(path):
    return [json.loads(l) for l in open(path) if l.strip().startswith("{")]


def canon(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def project(block):
    """The v1 schema's parent, channel, family and DeltaStats fields only."""
    if block is None:
        return None
    out = []
    for p in block:
        q = {k: p.get(k) for k in PARENT_KEYS if k not in ("channels", "families")}
        q["channels"] = [{k: c.get(k) for k in CHANNEL_KEYS} for c in p.get("channels", [])]
        q["families"] = [{k: f.get(k) for k in FAMILY_KEYS} for f in p.get("families", [])]
        out.append(q)
    return out


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


def expected(run):
    h, f = run["header"], run["footer"]
    if not h or not f:
        return None
    terminal = run["extinct"] if run["extinct"] is not None else f.get("ticks")
    if terminal is None:
        return None
    every = h["sample_every"]
    samples = [t for t in range(every, h["horizon"] + 1, every) if t <= terminal and run["extinct"] != t]
    checkpoints = [t for t in h["checkpoints"] if t <= terminal and run["extinct"] != t]
    return {"samples": samples, "checkpoints": checkpoints, "terminal": terminal}


def main():
    rd, rn, cd, cn, out = sys.argv[1:6]
    ref, cand = load(rd, rn), load(cd, cn)
    checks = []

    def check(name, ok, detail=None):
        checks.append({"check": name, "pass": bool(ok), "detail": detail})

    check("header and footer present on both runs",
          all((ref["header"], ref["footer"], cand["header"], cand["footer"])))
    er, ec = expected(ref), expected(cand)
    check("expected tick sets determinable on both runs", er is not None and ec is not None, {"v1": er, "v2": ec})
    if er and ec:
        for label, run, e in (("v1", ref, er), ("v2", cand, ec)):
            check(f"{label} samples are exactly its expected ticks", sorted(run["samples"]) == e["samples"])
            check(f"{label} checkpoints are exactly its expected ticks", sorted(run["checkpoints"]) == e["checkpoints"],
                  {"have": sorted(run["checkpoints"]), "expected": e["checkpoints"]})
    for label, run in (("v1", ref), ("v2", cand)):
        h, f = run["header"] or {}, run["footer"] or {}
        if run["extinct"] is None:
            check(f"{label} non-extinct footer reaches the header horizon",
                  f.get("ticks") == h.get("horizon") and f.get("extinct") is False)
        else:
            check(f"{label} extinction record and footer agree", f.get("ticks") == run["extinct"] and f.get("extinct") is True)
    mism, missing = [], []
    for t in sorted(set(ref["samples"]) | set(cand["samples"])):
        for f in SAMPLE_FIELDS:
            a, b = ref["samples"].get(t, {}), cand["samples"].get(t, {})
            if f not in a or f not in b:
                missing.append({"tick": t, "field": f})
            elif a[f] != b[f]:
                mism.append({"tick": t, "field": f, "v1": a[f], "v2": b[f]})
    check("every sample field present on both sides at every tick", not missing, {"missing": missing[:20]})
    check("every sample field equal at every tick", not mism, {"mismatches": mism[:20], "count": len(mism)})
    check("extinction tick equal", ref["extinct"] == cand["extinct"])
    rf, cf = ref["footer"] or {}, cand["footer"] or {}
    check("footer ticks and extinct equal", (rf.get("ticks"), rf.get("extinct")) == (cf.get("ticks"), cf.get("extinct")))
    check("checkpoint tick sets equal", set(ref["checkpoints"]) == set(cand["checkpoints"]),
          {"v1": sorted(ref["checkpoints"]), "v2": sorted(cand["checkpoints"])})
    hashes, bad, absent, reported, per_tick = {}, [], [], {}, {}
    for t in sorted(set(ref["checkpoints"]) & set(cand["checkpoints"])):
        a, b = ref["checkpoints"][t], cand["checkpoints"][t]
        for blk in BLOCKING:
            if a.get(blk) is None or b.get(blk) is None:
                absent.append(f"{t}:{blk}")
            hashes[f"{t}:{blk}"] = {"v1": canon(a.get(blk)), "v2": canon(b.get(blk))}
            if canon(a.get(blk)) != canon(b.get(blk)):
                bad.append(f"{t}:{blk}")
        va, vb = a.get("vote_deltas"), b.get("vote_deltas")
        if va is None or vb is None:
            absent.append(f"{t}:vote_deltas")
        hv1, hv2 = canon(project(va)), canon(project(vb))
        hashes[f"{t}:vote_deltas_projection"] = {"v1": hv1, "v2": hv2, "v1_raw_equals_projection": canon(va) == hv1}
        if hv1 != hv2:
            bad.append(f"{t}:vote_deltas_projection")
        for blk in REPORTED:
            reported[f"{t}:{blk}"] = canon(a.get(blk)) == canon(b.get(blk))
        per_tick[t] = {
            "v1_parents_with_executed_target_channels": sum(1 for p in (va or []) if p.get("executed_target_channels", 0) > 0),
            "v1_executed_target_channels": sum(p.get("executed_target_channels", 0) for p in (va or [])),
            "v2_vote_deltas_bytes": len(json.dumps(vb, separators=(",", ":"))),
            "v1_vote_deltas_bytes": len(json.dumps(va, separators=(",", ":"))),
            "vote_delta_secs": {"v1": a.get("vote_delta_secs"), "v2": b.get("vote_delta_secs")},
            "census_secs": {"v1": a.get("census_secs"), "v2": b.get("census_secs")},
        }
    check("blocking blocks present on both sides at every reached checkpoint", not absent, {"absent": absent})
    check("input_use, e6 and the vote_deltas whitelist projection hash-equal at every checkpoint", not bad, {"differing": bad})
    rh, ch = ref["header"] or {}, cand["header"] or {}
    diff = sorted(k for k in set(rh) | set(ch) if rh.get(k) != ch.get(k))
    v2_only = sorted(k for k in set(ch) - set(rh))
    disallowed = [k for k in diff if k not in HEADER_MAY_DIFFER and k not in v2_only]
    check("header equal outside vote_delta_version, v2-only keys, recipe and label", not disallowed,
          {"differing": diff, "v2_only": v2_only, "disallowed": disallowed,
           "vote_delta_version": {"v1": rh.get("vote_delta_version"), "v2": ch.get("vote_delta_version")}})
    sim = {"v1": rf.get("sim_secs"), "v2": cf.get("sim_secs")}
    result = {"v1": {"dir": rd, "name": rn, "sha256": ref["sha"]}, "v2": {"dir": cd, "name": cn, "sha256": cand["sha"]},
              "expected": {"v1": er, "v2": ec}, "checks": checks, "pass": all(c["pass"] for c in checks),
              "block_hashes": hashes, "reported_blocks_equal": reported, "per_checkpoint": per_tick,
              "footer_sim_secs": sim, "timing_fields_excluded": ["census_secs", "sim_secs_so_far", "vote_delta_secs",
                                                                 "births_secs", "sink_use_secs", "memory_secs",
                                                                 "footer sim_secs, census_secs, elapsed_secs"]}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("V0", rn, "vs", cn, "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
