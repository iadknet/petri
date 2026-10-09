"""Gate S0.2 (deletion-rate run; the output-wiring run's W0.2 (ii) checker
with the field renamed to deletion_keep_probability and, for gate S0, the
cohort dump files compared by file hash at every reached checkpoint when the
header says the dump was on): two census-probe runs compared
for identity on every sample row (the compared fields, each required
present), the extinction tick, the footer (required present, ticks and
extinct flag), and every scheduled checkpoint block (required present,
serialized once with sorted keys and compact separators, hashed), timing
fields excluded; the header compared on every key except the allowed
differences, which must differ exactly as the mode says. Expected sample
ticks and checkpoints are derived from the header's horizon, sample_every
and checkpoints and the run's own completion or extinction record, so a
truncated run cannot pass by matching omissions. The flag comparison is
vacuous only when both runs are verified extinct before the first
scheduled checkpoint.

usage: python3 identity.py <mode: equal|flag-differs> <ref dir> <ref name> <cand dir> <cand name> <out.json>"""
import hashlib
import json
import sys

SAMPLE_FIELDS = ("population", "mean_genome_size", "mean_generation", "mean_age", "mean_energy", "births_total",
                 "reproduction_actions_spawned_total", "mutation_events_applied_total")
BLOCKS = ("living", "selected_parents", "selected_generations", "age_deciles", "energy_deciles", "sample",
          "input_use", "e6", "vote_deltas", "births", "sink_use", "memory_loop", "memory_sensitivity",
          "temporal_memory_sensitivity")
TIMING = ("census_secs", "sim_secs_so_far", "vote_delta_secs", "births_secs", "sink_use_secs", "memory_secs")
# `world` is the run's label (PETRI_E2_NAME), which names the arm; `recipe`
# the recipe path. Under flag-differs the destination leaf of `prototypes`
# and the resulting `config_digest` may differ too; nothing else.
ALLOWED_HEADER_EQUAL = ("recipe", "world")
ALLOWED_HEADER_DIFFERS = ("config_digest", "prototypes", "recipe", "world")
FIELD = "deletion_keep_probability"


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


def expected(run):
    """Expected sample ticks and checkpoints from the header and the run's
    terminal tick (completion or extinction), or None when undeterminable."""
    h, f = run["header"], run["footer"]
    if not h or not f or "horizon" not in h or "sample_every" not in h or "checkpoints" not in h:
        return None
    terminal = run["extinct"] if run["extinct"] is not None else f.get("ticks")
    if terminal is None:
        return None
    every = h["sample_every"]
    samples = [t for t in range(every, h["horizon"] + 1, every) if t <= terminal and not (run["extinct"] == t)]
    # The census writes an extinction record instead of a sample at the
    # extinction tick; a sample at that tick is impossible.
    checkpoints = [t for t in h["checkpoints"] if t <= terminal and run["extinct"] != t]
    return {"samples": samples, "checkpoints": checkpoints, "terminal": terminal}


def main():
    mode, rd, rn, cd, cn, out = sys.argv[1:7]
    ref, cand = load(rd, rn), load(cd, cn)
    checks = []

    def check(name, ok, detail=None):
        checks.append({"check": name, "pass": bool(ok), "detail": detail})

    check("header present on both runs", bool(ref["header"]) and bool(cand["header"]))
    # Amendment A1 (review (a) finding 7): the policy metadata is required
    # present, so matching omissions cannot compare equal, and the cohort
    # dump must be on in both runs.
    for label, run in (("ref", ref), ("cand", cand)):
        h = run["header"] or {}
        check(f"{label} header carries config_digest and prototypes.{FIELD}",
              bool(h.get("config_digest")) and isinstance(h.get("prototypes"), dict) and FIELD in h["prototypes"],
              {"config_digest": h.get("config_digest"), "prototypes": h.get("prototypes")})
        check(f"{label} header has the cohort dump on", h.get("cohort_dump") is True, h.get("cohort_dump"))
        for t, row in run["checkpoints"].items():
            mc = (row.get("births") or {}).get("mutation_config")
            if not (isinstance(mc, dict) and FIELD in mc and "per_unit_rate" in mc):
                check(f"{label} births mutation_config populated at {t}", False, mc)
    check("footer present on both runs", bool(ref["footer"]) and bool(cand["footer"]),
          {"ref": ref["footer"], "cand": cand["footer"]})
    er, ec = expected(ref), expected(cand)
    check("expected tick sets determinable on both runs", er is not None and ec is not None, {"ref": er, "cand": ec})
    if er and ec:
        check("ref samples are exactly its expected ticks", sorted(ref["samples"]) == er["samples"],
              {"have": sorted(ref["samples"]), "expected": er["samples"]})
        check("cand samples are exactly its expected ticks", sorted(cand["samples"]) == ec["samples"],
              {"have": sorted(cand["samples"]), "expected": ec["samples"]})
        check("ref checkpoints are exactly its expected ticks", sorted(ref["checkpoints"]) == er["checkpoints"],
              {"have": sorted(ref["checkpoints"]), "expected": er["checkpoints"]})
        check("cand checkpoints are exactly its expected ticks", sorted(cand["checkpoints"]) == ec["checkpoints"],
              {"have": sorted(cand["checkpoints"]), "expected": ec["checkpoints"]})
    for label, run in (("ref", ref), ("cand", cand)):
        h, f = run["header"] or {}, run["footer"] or {}
        if run["extinct"] is None:
            check(f"{label} non-extinct footer reaches the header horizon", f.get("ticks") == h.get("horizon") and f.get("extinct") is False,
                  {"footer_ticks": f.get("ticks"), "horizon": h.get("horizon"), "extinct_flag": f.get("extinct")})
        else:
            check(f"{label} extinction record, footer ticks and extinct flag agree", f.get("ticks") == run["extinct"] and f.get("extinct") is True,
                  {"footer_ticks": f.get("ticks"), "extinct_record": run["extinct"], "extinct_flag": f.get("extinct")})
    check("sample tick sets equal", set(ref["samples"]) == set(cand["samples"]),
          {"ref": sorted(ref["samples"]), "cand": sorted(cand["samples"])})
    mism, missing = [], []
    for t in sorted(set(ref["samples"]) & set(cand["samples"])):
        for f in SAMPLE_FIELDS:
            if f not in ref["samples"][t] or f not in cand["samples"][t]:
                missing.append({"tick": t, "field": f})
            elif ref["samples"][t][f] != cand["samples"][t][f]:
                mism.append({"tick": t, "field": f, "ref": ref["samples"][t][f], "cand": cand["samples"][t][f]})
    check("every sample field present on both sides at every tick", not missing, {"missing": missing[:20], "count": len(missing)})
    check("every sample field equal at every tick", not mism, {"mismatches": mism[:20], "count": len(mism)})
    check("extinction tick equal", ref["extinct"] == cand["extinct"], {"ref": ref["extinct"], "cand": cand["extinct"]})
    rf, cf = ref["footer"] or {}, cand["footer"] or {}
    check("footer ticks and extinct present and equal",
          "ticks" in rf and "extinct" in rf and (rf.get("ticks"), rf.get("extinct")) == (cf.get("ticks"), cf.get("extinct")),
          {"ref": {k: rf.get(k) for k in ("ticks", "extinct")}, "cand": {k: cf.get(k) for k in ("ticks", "extinct")}})
    check("checkpoint tick sets equal", set(ref["checkpoints"]) == set(cand["checkpoints"]),
          {"ref": sorted(ref["checkpoints"]), "cand": sorted(cand["checkpoints"])})
    block_hashes = {}
    bad, absent, flags = [], [], []
    for t in sorted(set(ref["checkpoints"]) & set(cand["checkpoints"])):
        for b in BLOCKS:
            vr, vc = ref["checkpoints"][t].get(b), cand["checkpoints"][t].get(b)
            if vr is None or vc is None:
                absent.append(f"{t}:{b}")
            if b == "births":
                # The births block embeds the arm's mutation_config (the
                # field itself); it is compared apart, the counts hashed.
                vr = {k: v for k, v in (vr or {}).items() if k != "mutation_config"}
                vc = {k: v for k, v in (vc or {}).items() if k != "mutation_config"}
                mr = (ref["checkpoints"][t].get("births") or {}).get("mutation_config") or {}
                mc = (cand["checkpoints"][t].get("births") or {}).get("mutation_config") or {}
                flags.append({"tick": t, "ref": mr.get(FIELD), "cand": mc.get(FIELD),
                              "rest_equal": {k: v for k, v in mr.items() if k != FIELD} == {k: v for k, v in mc.items() if k != FIELD}})
            hr, hc = canon(vr), canon(vc)
            block_hashes[f"{t}:{b}"] = {"ref": hr, "cand": hc}
            if hr != hc:
                bad.append(f"{t}:{b}")
        extra = set(ref["checkpoints"][t]) ^ set(cand["checkpoints"][t])
        if extra - set(TIMING):
            bad.append(f"{t}:keys:{sorted(extra)}")
    check("every scheduled block present (not null) on both sides at every reached checkpoint", not absent, {"absent": absent})
    check("every timing-free checkpoint block hash-equal (births counts without mutation_config)", not bad, {"differing": bad})
    dumps = {}
    if True:
        for t in sorted(set(ref["checkpoints"]) | set(cand["checkpoints"])):
            pr, pc = f"{rd}/cohort-{rn}-t{t}.json", f"{cd}/cohort-{cn}-t{t}.json"
            try:
                dumps[str(t)] = {"ref": sha(pr), "cand": sha(pc)}
            except OSError as e:
                dumps[str(t)] = {"error": str(e)}
        check("cohort dump present on both sides and file-hash-equal at every reached checkpoint",
              bool(dumps) and all("error" not in v and v["ref"] == v["cand"] for v in dumps.values()), dumps)
    first = min((ref["header"] or {}).get("checkpoints") or [0]) if ref["header"] else 0
    if not flags:
        both_extinct_early = (ref["extinct"] is not None and cand["extinct"] is not None
                              and ref["extinct"] < first and cand["extinct"] < first and not ref["checkpoints"] and not cand["checkpoints"])
        check(f"births mutation_config comparison vacuous only because both runs are verified extinct before the first checkpoint {first}",
              both_extinct_early, {"ref_extinct": ref["extinct"], "cand_extinct": cand["extinct"], "first_checkpoint": first})
    elif mode == "equal":
        check("births block mutation_config entirely equal at every checkpoint", all(f["ref"] == f["cand"] and f["rest_equal"] for f in flags), flags)
    else:
        check(f"births block mutation_config differs only in {FIELD} at every checkpoint",
              all(f["ref"] != f["cand"] and f["rest_equal"] for f in flags), flags)
    rh, ch = ref["header"] or {}, cand["header"] or {}
    diff = sorted(k for k in set(rh) | set(ch) if rh.get(k) != ch.get(k))
    allowed = ALLOWED_HEADER_EQUAL if mode == "equal" else ALLOWED_HEADER_DIFFERS
    disallowed = [k for k in diff if k not in allowed]
    check("header equal outside the allowed keys (every other nested value compared)", not disallowed, {"differing": diff, "disallowed": disallowed})
    if mode == "equal":
        check("config_digest and prototypes equal (same binary policy, same recipe)",
              rh.get("config_digest") == ch.get("config_digest") and rh.get("prototypes") == ch.get("prototypes"),
              {"ref": {"config_digest": rh.get("config_digest"), "prototypes": rh.get("prototypes")},
               "cand": {"config_digest": ch.get("config_digest"), "prototypes": ch.get("prototypes")}})
    else:
        check("config_digest differs (the field serializes at non_clear)", rh.get("config_digest") != ch.get("config_digest"),
              {"ref": rh.get("config_digest"), "cand": ch.get("config_digest")})
        check(f"prototypes.{FIELD} differs",
              (rh.get("prototypes") or {}).get(FIELD) != (ch.get("prototypes") or {}).get(FIELD),
              {"ref": rh.get("prototypes"), "cand": ch.get("prototypes")})
        rp = {k: v for k, v in (rh.get("prototypes") or {}).items() if k != FIELD}
        cp = {k: v for k, v in (ch.get("prototypes") or {}).items() if k != FIELD}
        check("prototypes otherwise equal (the destination leaf alone may differ)", rp == cp, {"ref": rp, "cand": cp})
    result = {"mode": mode, "ref": {"dir": rd, "name": rn, "sha256": ref["sha"]}, "cand": {"dir": cd, "name": cn, "sha256": cand["sha"]},
              "expected": {"ref": er, "cand": ec}, "checks": checks, "pass": all(c["pass"] for c in checks), "block_hashes": block_hashes, "dump_hashes": dumps}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("identity", rn, "vs", cn, "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
