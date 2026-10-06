"""Diagnosis run rows E2 and E6: read a world's census (`census-<world>.ndjson`)
and samples, extract the target families' funnel rows per checkpoint
(selected and founder cohorts, with `parents_evaluated` as the denominator),
the E6 reading, the sample trajectory and the E3 proxies; apply the E2 row's
reading rule (premise qualified: share_causal strictly increasing over the
checkpoints and the target family's retained share above 0.9 at 50,000;
variation support: family_connected rises and share_causal does not).

usage: python3 e2read.py <census.ndjson> <samples.ndjson> <out.json>"""
import hashlib
import json
import sys

TARGETS = ("AreaFoodSummary", "NeighborBarrierRing")
FAMILY_FIELDS = ["family_declared", "family_connected", "family_executed", "family_causal",
                 "family_causal_original", "out_of_width_consumers"]
ROW_FIELDS = ["declared", "connected", "executed", "causal", "causal_original",
              "executed_outside_live", "causal_outside_live", "retention_pairs", "retained_causal_pairs"]


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def num(x):
    return None if x == "-" else int(x)


def family_rows(cohort):
    out = {}
    for line in cohort["family_rows"]:
        parts = line.split(" ")
        fam = parts[0]
        base = fam.split(":")[0]
        if base in TARGETS:
            out[fam] = dict(zip(FAMILY_FIELDS, map(num, parts[1:])))
    return out


def channel_retention(cohort):
    """Retention pairs and retained causal pairs pooled over the target
    families' channels."""
    out = {}
    for line in cohort["rows"]:
        parts = line.split(" ")
        fam = parts[0]
        base = fam.split(":")[0]
        if base not in TARGETS:
            continue
        counts = dict(zip(ROW_FIELDS, map(num, parts[2:])))
        e = out.setdefault(fam, {"retention_pairs": 0, "retained_causal_pairs": 0, "channels": 0, "channels_causal": 0})
        e["channels"] += 1
        e["channels_causal"] += 1 if (counts["causal"] or 0) > 0 else 0
        e["retention_pairs"] += counts["retention_pairs"] or 0
        e["retained_causal_pairs"] += counts["retained_causal_pairs"] or 0
    for e in out.values():
        e["retained_share"] = e["retained_causal_pairs"] / e["retention_pairs"] if e["retention_pairs"] else None
    return out


def main():
    census_path, samples_path, out_path = sys.argv[1:4]
    rows = [json.loads(l) for l in open(census_path)]
    samples = [json.loads(l) for l in open(samples_path)]
    header = next(r for r in rows if r["kind"] == "header")
    footer = next((r for r in rows if r["kind"] == "footer"), None)
    extinct = next((r for r in rows if r["kind"] == "extinct"), None)
    checkpoints = []
    e6 = None
    for r in rows:
        if r["kind"] != "checkpoint":
            continue
        cohorts = {}
        for c in r["input_use"]["cohorts"]:
            # The bench's `Indicator` serializes untagged: a defined cohort is
            # the block itself, an undefined one its reason string.
            if isinstance(c, dict) and "Defined" in c:
                c = c["Defined"]
            if isinstance(c, dict) and "cohort" in c:
                d = c
                cohorts[d["cohort"]] = {
                    "parents_requested": d["parents_requested"], "parents_evaluated": d["parents_evaluated"],
                    "consistency_violations": d["consistency_violations"],
                    "retention_parents": d["retention_parents"],
                    "retained_share_all_families": d["retained_share"],
                    "families": family_rows(d), "retention": channel_retention(d),
                }
            else:
                cohorts[str(c.get("Undefined") if isinstance(c, dict) else c)] = None
        cp = {"tick": r["tick"], "living": r["living"], "census_secs": r["census_secs"], "sim_secs_so_far": r["sim_secs_so_far"],
              "selected_parents": r["selected_parents"], "selected_generations": r["selected_generations"],
              "age_deciles": r["age_deciles"], "energy_deciles": r["energy_deciles"], "sample": r["sample"], "cohorts": cohorts}
        if r.get("e6"):
            e6 = r["e6"]
            cp["e6"] = e6
        checkpoints.append(cp)
    # Reading rule on the selected cohort.
    def share(cp, fam_prefix, field):
        """Typed-row incidences of the family summed over its food types,
        over parents_evaluated. A typed family row counts the parents with the
        stage on that typed family, so the sum over types counts
        parent-type incidences, not distinct parents (a parent declaring both
        types counts twice); the per-type rows are reported beside it. A
        family absent from the rows was declared on no reachable node of any
        parent: every stage is 0 of n."""
        sel = cp["cohorts"].get("selected")
        if not sel:
            return None
        n = sel["parents_evaluated"]
        tot = sum((v[field] or 0) for k, v in sel["families"].items() if k.startswith(fam_prefix))
        return tot / n if n else None

    def typed(cp, fam_prefix, field):
        sel = cp["cohorts"].get("selected")
        if not sel:
            return None
        return {k: (v[field] or 0) for k, v in sel["families"].items() if k.startswith(fam_prefix)}
    reading = {}
    for fam in TARGETS:
        causal = [share(cp, fam, "family_causal") for cp in checkpoints]
        connected = [share(cp, fam, "family_connected") for cp in checkpoints]
        declared = [share(cp, fam, "family_declared") for cp in checkpoints]
        last = checkpoints[-1]["cohorts"].get("selected") if checkpoints else None
        ret = None
        if last:
            pairs = sum(v["retention_pairs"] for k, v in last["retention"].items() if k.startswith(fam))
            kept = sum(v["retained_causal_pairs"] for k, v in last["retention"].items() if k.startswith(fam))
            ret = kept / pairs if pairs else None
        adequate = all(cp["cohorts"].get("selected", {}) and cp["cohorts"]["selected"]["parents_evaluated"] == 20 for cp in checkpoints)
        ticks = [cp["tick"] for cp in checkpoints]
        # The row's rules are evaluable only with the three declared
        # checkpoints (10,000, 20,000, 50,000) and adequacy at each; with
        # fewer the fields below are None and only the descriptive directions
        # are reported.
        evaluable = ticks == [10000, 20000, 50000] and adequate
        inc = lambda xs: all(x is not None for x in xs) and len(xs) >= 2 and all(b > a for a, b in zip(xs, xs[1:]))
        reading[fam] = {"share_declared_incidences": declared, "share_connected_incidences": connected,
                        "share_causal_incidences": causal,
                        "typed_rows": {str(cp["tick"]): {f: typed(cp, fam, f) for f in ("family_declared", "family_connected", "family_executed", "family_causal")} for cp in checkpoints},
                        "retained_share_at_last": ret, "adequate_20_parents": adequate, "checkpoints": ticks,
                        "rules_evaluable": evaluable,
                        "descriptive_direction": ("causal rising" if inc(causal) else "connected rising, causal not" if inc(connected) else "flat or mixed"),
                        "premise_qualified": (inc(causal) and (ret is not None and ret > 0.9)) if evaluable else None,
                        "variation_support": (inc(connected) and not inc(causal)) if evaluable else None}
    traj = [s for s in samples if s["kind"] == "sample"]
    out = {"row": "E2", "world": header["world"], "seed": header["seed"], "threads": header["threads"],
           "horizon": header["horizon"], "checkpoints_reached": [cp["tick"] for cp in checkpoints],
           "complete": footer is not None and not extinct and footer["ticks"] >= header["horizon"],
           "extinct": extinct, "footer": footer,
           "trajectory": [{k: s[k] for k in ("tick", "population", "mean_genome_size", "mean_generation", "mean_age", "mean_energy", "births_total")} for s in traj],
           "checkpoints": checkpoints, "reading": reading, "e6": e6,
           "provenance": {"census_sha256": sha(census_path), "samples_sha256": sha(samples_path), "reader_sha256": sha(__file__)}}
    json.dump(out, open(out_path, "w"), indent=1)
    print(header["world"], "complete", out["complete"], "checkpoints", out["checkpoints_reached"])
    for cp in checkpoints:
        print(f"  tick {cp['tick']}: living {cp['living']} parents {cp['selected_parents']} census {cp['census_secs']:.0f}s "
              f"size {cp['sample']['mean_genome_size']:.1f} gen {cp['sample']['mean_generation']:.1f} age {cp['sample']['mean_age']:.1f}")
        for fam in TARGETS:
            sel = cp["cohorts"].get("selected") or {}
            fr = {k: v for k, v in sel.get("families", {}).items() if k.startswith(fam)}
            print(f"     {fam}: {fr}")
    for fam, r in reading.items():
        print(fam, {k: r[k] for k in ("share_causal_incidences", "share_connected_incidences", "retained_share_at_last", "rules_evaluable", "descriptive_direction", "premise_qualified", "variation_support")})
    if e6:
        print("E6", e6["shares"], e6["step_comparison"], e6.get("lookahead3_comparison"))


if __name__ == "__main__":
    main()
