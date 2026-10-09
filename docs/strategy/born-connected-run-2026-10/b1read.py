"""Born-connected row B1: read every census-probe run under <b1-dir> (arms
default, born, default-mutoff, born-mutoff; seeds 1022 and 2022), apply E8's
window readings over ticks 15,000 to 20,000 (mean population, mean genome
size in founder units, births per creature-tick, extinction interval), the
typed target-family funnel rows at every checkpoint, the vote-delta tallies
with the agreement check (per typed family after deduplicating channels by
parent, and per channel row), the births block (dead/silent/changed per all
and per mutated births), and the row's predeclared validation rule with its
resolutions (i) to (v).

usage: python3 b1read.py <b1-dir> <out.json>"""
import hashlib
import json
import os
import sys

FOUNDER = 97
ARMS = ("default", "born", "default-mutoff", "born-mutoff")
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
    for e in out.values():
        e["declared_unconnected"] = (e["family_declared"] or 0) - (e["family_connected"] or 0)
    return out


def channel_rows(cohort):
    """The funnel's per-channel rows of the target families: (family, channel) -> causal parents."""
    out = {}
    for line in cohort["rows"]:
        parts = line.split(" ")
        fam = parts[0]
        if fam in TARGET_ROWS and len(parts) > 5:
            ch = parts[1].rstrip("*")
            try:
                out[(fam, int(ch))] = {"declared": num(parts[2]), "connected": num(parts[3]), "executed": num(parts[4]), "causal": num(parts[5])}
            except ValueError:
                pass
    return out


def vote_reading(block):
    parents = block or []
    channels = [(p["parent_index"], c) for p in parents for c in p["channels"]]
    s = lambda c, k: c["stats"].get(k)
    target = [(pi, c) for pi, c in channels if norm_family(c["family"]).split(":")[0] in ("AreaFoodSummary", "NeighborBarrierRing")]
    action = [(pi, c) for pi, c in target if s(c, "scenes_actions_differ") > 0]
    delta = [(pi, c) for pi, c in target if s(c, "delta_passes") > 0]
    by_family_action = {}
    by_channel_action = {}
    for pi, c in action:
        fam = norm_family(c["family"])
        by_family_action.setdefault(fam, set()).add(pi)
        by_channel_action.setdefault((fam, c["channel"]), set()).add(pi)
    return {
        "parents": len(parents),
        "parents_with_executed_target_channels": sum(1 for p in parents if p["executed_target_channels"] > 0),
        "executed_target_channels": len(target),
        "channels_with_action_queue_change": len(action),
        "channels_with_any_vote_delta": len(delta),
        "channels_vote_delta_without_action_change": sum(1 for pi, c in target if s(c, "delta_passes") > 0 and s(c, "scenes_actions_differ") == 0),
        "channels_action_change_without_vote_delta": sum(1 for pi, c in target if s(c, "delta_passes") == 0 and s(c, "scenes_actions_differ") > 0),
        "distinct_parents_with_action_change_by_family": {f: sorted(v) for f, v in by_family_action.items()},
        "distinct_parents_with_action_change_by_channel": {f"{f}#{ch}": sorted(v) for (f, ch), v in by_channel_action.items()},
        "action_change_channels": [{"parent": pi, "family": c["family"], "channel": c["channel"], "scenes_actions_differ": s(c, "scenes_actions_differ"),
                                    "delta_passes": s(c, "delta_passes"), "consumers_graph": c["consumers_graph"],
                                    "consumers_graph_abs_weight": round(c["consumers_graph_abs_weight"], 4), "consumers_vm": c["consumers_vm"]} for pi, c in action],
        # Distinct readings among the action-changing channels, grouped by
        # (family, channel, consumer counts, summed |weight|, delta passes):
        # phenotype-summary groups; shared ancestry is a hypothesis, not a bound
        # (review (b) findings 2 and 13).
        "distinct_action_change_readings": sorted({(norm_family(c["family"]), c["channel"], c["consumers_graph"], c["consumers_vm"],
                                                   round(c["consumers_graph_abs_weight"], 4), s(c, "delta_passes")) for pi, c in action}),
        "distinct_action_change_parents": len({pi for pi, c in action}),
        "vote_delta_channels": [{"parent": pi, "family": c["family"], "channel": c["channel"], "delta_passes": s(c, "delta_passes"),
                                 "max_abs_delta": s(c, "max_abs_delta"), "most_moved_kind": s(c, "most_moved_kind")} for pi, c in delta],
    }


def agreement(typed, chans, vote):
    out = {"by_family": {}, "by_channel": {}, "pass": True}
    for fam, row in typed.items():
        funnel = row["family_causal"] or 0
        v = len(vote["distinct_parents_with_action_change_by_family"].get(fam, []))
        out["by_family"][fam] = {"funnel_family_causal": funnel, "vote_delta_parents": v, "agree": funnel == v}
        out["pass"] &= funnel == v
    for (fam, ch), row in chans.items():
        funnel = row["causal"] or 0
        v = len(vote["distinct_parents_with_action_change_by_channel"].get(f"{fam}#{ch}", []))
        if funnel or v:
            out["by_channel"][f"{fam}#{ch}"] = {"funnel_causal": funnel, "vote_delta_parents": v, "agree": funnel == v}
            out["pass"] &= funnel == v
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
                typed = family_rows(selected) if selected else None
                chans = channel_rows(selected) if selected else {}
                vote = vote_reading(r.get("vote_deltas"))
                births_block = r.get("births") or {}
                checkpoints[str(r["tick"])] = {
                    "living": r["living"],
                    "selected_parents": r["selected_parents"],
                    "parents_evaluated": selected.get("parents_evaluated") if selected else None,
                    "target_rows": typed,
                    "executed_plus_causal": sum((v["family_executed"] or 0) + (v["family_causal"] or 0) for v in typed.values()) if typed else None,
                    "vote_deltas": vote,
                    "agreement": agreement(typed, chans, vote) if typed else None,
                    "births": {k: births_block.get(k) for k in ("parents", "births_per_parent", "mutation_config", "pooled")},
                    "e6_gradient_informative": (r.get("e6") or {}).get("shares", {}).get("gradient_informative"),
                    "secs": {k: r.get(k) for k in ("census_secs", "vote_delta_secs", "births_secs")},
                }
    return {
        "completed": footer is not None,
        "extinct_tick": extinct["tick"] if extinct else None,
        "prototypes": header.get("prototypes") if header else None,
        "config_digest": header.get("config_digest") if header else None,
        "samples": len(samples),
        "last_tick": samples[-1]["tick"] if samples else None,
        "window_alive_samples": len(alive),
        "mean_population": mean("population"),
        "mean_energy": mean("mean_energy"),
        "mean_genome_size": mean("mean_genome_size"),
        "mean_genome_size_founder_units": (mean("mean_genome_size") / FOUNDER) if alive else None,
        "mean_generation": mean("mean_generation"),
        "size_per_generation": (mean("mean_genome_size") / mean("mean_generation")) if alive and mean("mean_generation") else None,
        "births_per_creature_tick": births,
        "extinction_interval": None if extinct is None else [last_alive, extinct["tick"]],
        "final": {k: samples[-1][k] for k in ("tick", "population", "mean_genome_size", "mean_generation",
                                                  "reproduction_actions_spawned_total")} if samples else None,
        "min_population_sample": min(((s["population"], s["tick"]) for s in samples), default=None),
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

    def run(arm, seed):
        return runs.get(f"{arm}-s{seed}")

    def val(arm, seed, k):
        r = run(arm, seed)
        return None if r is None else r.get(k)

    def reaches(arm, seed):
        r = run(arm, seed)
        return bool(r and r["completed"] and r["extinct_tick"] is None and r["last_tick"] == 20_000)

    def cp(arm, seed, tick="20000"):
        r = run(arm, seed)
        return (r or {}).get("checkpoints", {}).get(tick)

    rule = {"per_seed": {}}
    for s in SEEDS:
        dflt, born = run("default", s), run("born", s)
        dp, bp = val("default", s, "mean_population"), val("born", s, "mean_population")
        d20, b20 = cp("default", s), cp("born", s)
        default_defined = bool(dflt and dflt["completed"] and dflt["extinct_tick"] is None and dp)
        causal_where_default_none = None
        exc = None
        vote_cond = None
        if d20 and b20 and d20["target_rows"] and b20["target_rows"]:
            exc = b20["executed_plus_causal"] > d20["executed_plus_causal"]
            causal_where_default_none = any((b20["target_rows"][row]["family_causal"] or 0) >= 1 and (d20["target_rows"][row]["family_causal"] or 0) == 0
                                            for row in TARGET_ROWS)
            vote_cond = b20["vote_deltas"]["channels_with_action_queue_change"] >= 1
        per = {
            "born_reaches_20000": reaches("born", s),
            "default_reaches_20000": reaches("default", s),
            "default_defined_in_window": default_defined,
            "population_ratio": None if not dp or bp is None else bp / dp,
            "population_at_least_half": None if not dp or bp is None else bp >= 0.5 * dp,
            "executed_plus_causal_default": d20["executed_plus_causal"] if d20 else None,
            "executed_plus_causal_born": b20["executed_plus_causal"] if b20 else None,
            "executed_plus_causal_exceeds": exc,
            "causal_incidence_where_default_none": causal_where_default_none,
            "vote_delta_action_change_channel": vote_cond,
            "parents_evaluated_20": (d20 or {}).get("parents_evaluated") == 20 and (b20 or {}).get("parents_evaluated") == 20,
            "agreement_default": (d20 or {}).get("agreement", {}).get("pass") if d20 else None,
            "agreement_born": (b20 or {}).get("agreement", {}).get("pass") if b20 else None,
            "size_default": val("default", s, "mean_genome_size_founder_units"),
            "size_born": val("born", s, "mean_genome_size_founder_units"),
            "births_default": val("default", s, "births_per_creature_tick"),
            "births_born": val("born", s, "births_per_creature_tick"),
            "pair": {"default_mutoff_extinction": val("default-mutoff", s, "extinction_interval"),
                     "born_mutoff_extinction": val("born-mutoff", s, "extinction_interval")},
        }
        per["agreement_holds"] = bool(per["agreement_default"] and per["agreement_born"])
        # The agreement check is blocking (resolution (ii)): no validated
        # verdict without it (review (b) finding 14).
        per["seed_validates"] = bool(per["born_reaches_20000"] and default_defined and per["population_at_least_half"] and exc
                                     and causal_where_default_none and vote_cond and per["parents_evaluated_20"] and per["agreement_holds"])
        rule["per_seed"][str(s)] = per
    both = len(rule["per_seed"]) == 2
    rule["validated"] = (all(v["seed_validates"] for v in rule["per_seed"].values()) if both else None)
    rule["agreement_holds_on_both"] = (all(v["agreement_holds"] for v in rule["per_seed"].values()) if both else None)
    rule["note"] = ("lead-selection screen on 20 parents per cohort; the rule needs every clause on both seeds; a default extinct "
                    "or empty in the window leaves the seed undefined and the row not validated by the rule (resolution iii)")
    result = {"row": "B1", "world": "Canyon country", "window_ticks": list(WINDOW), "founder_units": FOUNDER, "runs": runs, "rule": rule}
    open(out_path, "w").write(json.dumps(result, indent=1) + "\n")
    for name, r in runs.items():
        print(name, "completed" if r["completed"] else "incomplete", "extinct", r["extinct_tick"],
              "pop", None if r["mean_population"] is None else round(r["mean_population"]),
              "size x", None if r["mean_genome_size_founder_units"] is None else round(r["mean_genome_size_founder_units"], 2),
              "births", None if r["births_per_creature_tick"] is None else round(r["births_per_creature_tick"], 5))
        for t, c in r["checkpoints"].items():
            tr = c["target_rows"] or {}
            print("  ", t, {k: [v["family_declared"], v["family_connected"], v["family_executed"], v["family_causal"]] for k, v in tr.items()},
                  "vote", {k: c["vote_deltas"][k] for k in ("executed_target_channels", "channels_with_action_queue_change", "channels_with_any_vote_delta")},
                  "agree", (c["agreement"] or {}).get("pass"),
                  "births", {k: (c["births"]["pooled"] or {}).get(k) for k in ("dead_per_all_births", "dead_per_mutated_births", "changed_per_mutated_births", "silent_per_mutated_births")} if c["births"].get("pooled") else None)
    print("B1 validated:", rule["validated"], "agreement on both:", rule["agreement_holds_on_both"])
    if rule["agreement_holds_on_both"] is False:
        print("AGREEMENT FAILURE: the funnel's causal counts and the vote-delta action changes disagree; no verdict is read")
        sys.exit(1)


if __name__ == "__main__":
    main()
