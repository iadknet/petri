"""Deletion-rate run gates S0-F and S0 (docs/strategy/deletion-rate-run-2026-10.md),
as amended by A1 (Codex review (a) findings 2 to 6 and 11).

Statistical checks are 95 % intervals. A containment miss is a mechanism defect
only when its exact two-sided binomial p-value is below 0.001; a miss with
p >= 0.001 is a recorded statistical miss and does not block (A1, finding 5).
Exact checks (identity, requested equality, parent sets) block on any failure.

usage:
  python3 s0read.py s0f <dir> <out.json>
      <dir> holds e4-frozen.json, e4-proto.json (e4_transition_rates at None,
      10^6 births), s0-founder-frozen.json, s0-founder-none.json and
      s0-founder-keep003.json (s0_deletion_hazards, founder only).
      Exit 0: every blocking check passes and nothing is left to record;
      4: blocking checks pass but a statistical miss or a tolerance failure
      needs a recorded disposition; 1: a blocking check fails.
  python3 s0read.py s0 <dir> <out.json>
      <dir> holds s0-none.json and s0-keep003.json (the founder plus the 40
      cohort parents). Exit 0: a go with nothing to record; 4: a go with a
      statistical miss, a (c) or (d) failure to record (the runner stops);
      3: a no-go; 2: a blocking failure (parent sets, (a), or a (b) defect)."""
import json
import math
import statistics
import sys

E4_LOSS = 0.018643
KEEP = 0.03
DEFECT_P = 0.001
OTHER_MIN = 4000
OTHER_TOL = 0.05
LINE = 400
DECREASING = ("InputRefPrune", "GraphRemoveGraphEdge", "GraphRemoveInternalGraphNode", "GraphDisableHebbian",
              "GraphDisableRewardModulation", "TopologyRemoveNode", "TopologyRemoveRouteTarget", "VmDeleteInstruction")
CLASSES = ("silent_input_entry", "graph_edge", "compute_node", "plasticity_config", "reward_config", "mesh_node",
           "route_target", "vm_instruction")
E4_PARENTS = ["founder", "founder+declared", "founder+declared+zero-edge"]
COHORTS = {"default-s1022": 1022, "default-s2022": 2022}


def betacf(a, b, x):
    tiny = 1e-300
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) > tiny else tiny)
    h = d
    for m in range(1, 400):
        m2 = 2 * m
        for aa in (m * (b - m) * x / ((qam + m2) * (a + m2)), -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))):
            d = 1.0 + aa * d
            d = 1.0 / (d if abs(d) > tiny else tiny)
            c = 1.0 + aa / c
            c = c if abs(c) > tiny else tiny
            h *= d * c
        if abs(d * c - 1.0) < 1e-12:
            break
    return h


def betai(a, b, x):
    """Regularized incomplete beta I_x(a, b)."""
    if x <= 0.0:
        return 0.0
    if x >= 1.0:
        return 1.0
    lbt = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1.0 - x)
    if x < (a + 1.0) / (a + b + 2.0):
        return math.exp(lbt) * betacf(a, b, x) / a
    return 1.0 - math.exp(lbt) * betacf(b, a, 1.0 - x) / b


def beta_inv(p, a, b):
    lo, hi = 0.0, 1.0
    for _ in range(200):
        mid = (lo + hi) / 2
        if betai(a, b, mid) < p:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def cp95(k, n):
    """Exact Clopper-Pearson 95 % interval of k successes in n trials."""
    if n == 0:
        return (0.0, 1.0)
    lo = 0.0 if k == 0 else beta_inv(0.025, k, n - k + 1)
    hi = 1.0 if k >= n else beta_inv(0.975, k + 1, n - k)
    return (lo, hi)


def two_sided_p(k, n, p):
    """Exact two-sided binomial p-value by doubling the smaller tail."""
    if n == 0 or p <= 0.0 or p >= 1.0:
        return 1.0
    lower = 1.0 - betai(k + 1, n - k, p) if k < n else 1.0  # P(X <= k)
    upper = betai(k, n - k + 1, p) if k > 0 else 1.0  # P(X >= k)
    return min(1.0, 2 * min(lower, upper))


def containment(name, k, n, p, checks):
    """A 95 % CP containment check with the A1 defect threshold."""
    lo, hi = cp95(k, n)
    pv = two_sided_p(k, n, p)
    contains = lo <= p <= hi
    state = "pass" if contains else ("defect" if pv < DEFECT_P else "statistical_miss")
    checks.append({"check": name, "kind": "statistical", "state": state, "pass": state != "defect",
                   "detail": {"k": k, "n": n, "rate": k / n if n else None, "cp95": [lo, hi], "target": p, "p": pv}})
    return state


def exact(name, ok, checks, detail=None):
    checks.append({"check": name, "kind": "exact", "state": "pass" if ok else "fail", "pass": bool(ok), "detail": detail})
    return bool(ok)


def load(path):
    return json.loads(open(path).read())


def projection(row):
    return {k: v for k, v in row.items() if k != "wall_secs"}


def by_name(doc, checks, label):
    names = [r["parent"] for r in doc["parents"]]
    exact(f"{label}: parent names unique", len(names) == len(set(names)), checks, [n for n in names if names.count(n) > 1][:5])
    return {r["parent"]: r for r in doc["parents"]}


def pooled(rows, op):
    return sum(r["applied_by_operator"].get(op, 0) for r in rows.values())


def not_kept(rows, op):
    return sum(r["skipped_by_operator_reason"].get(f"{op}|DeletionNotKept", 0) for r in rows.values())


def manipulation(n_rows, k_rows, checks, out, with_c):
    """The manipulation check at 0.03 over the parents of two runs."""
    names = sorted(n_rows)
    exact("(a) the same parents at both settings", sorted(k_rows) == names, checks)
    req = [(p, n_rows[p]["requested_events"], k_rows.get(p, {}).get("requested_events")) for p in names]
    exact("(a) requested events equal to None's for every parent", all(x == y for _, x, y in req), checks,
          [r for r in req if r[1] != r[2]][:10])
    births = sum(n_rows[p]["births"] for p in names)
    exact("births per parent equal at both settings", all(n_rows[p]["births"] == k_rows.get(p, {}).get("births") for p in names), checks)
    out["decreasing"], out["other"] = {}, {}
    for op in DECREASING:
        a0, a1, nk = pooled(n_rows, op), pooled(k_rows, op), not_kept(k_rows, op)
        entry = {"applied_none": a0, "applied_keep": a1, "not_kept": nk, "ratio": a1 / a0 if a0 else None}
        if a1 + nk > 0:
            entry["kept_share_state"] = containment(f"(b) {op} kept share contains {KEEP}", a1, a1 + nk, KEEP, checks)
        if with_c:
            # (c), the plan's form, A1 finding 4: events, not births; the
            # interval treats the 0.03 event count as Poisson (n = births is
            # a large-n approximation), so the check is descriptive.
            lo, hi = cp95(a1, births)
            target = KEEP * a0 / births
            entry["c_plan_form"] = {"k": a1, "births": births, "interval": [lo, hi], "target": target, "contains": lo <= target <= hi,
                                    "p_approx": two_sided_p(a1, births, target) if target > 0 else None}
        out["decreasing"][op] = entry
    d_fail = []
    for op in sorted({op for r in list(n_rows.values()) + list(k_rows.values()) for op in r["applied_by_operator"]}):
        if op in DECREASING:
            continue
        a0, a1 = pooled(n_rows, op), pooled(k_rows, op)
        rel = (a1 - a0) / a0 if a0 else None
        out["other"][op] = {"applied_none": a0, "applied_keep": a1, "relative": rel, "checked": a0 >= OTHER_MIN}
        if a0 >= OTHER_MIN and abs(rel) > OTHER_TOL:
            d_fail.append(op)
    checks.append({"check": f"(d) every other operator with >= {OTHER_MIN} applied at None within {OTHER_TOL:.0%}", "kind": "tolerance",
                   "state": "pass" if not d_fail else "tolerance_failure", "pass": True,
                   "detail": {op: out["other"][op] for op in d_fail}})
    return d_fail


def verdict(checks):
    blocking = [c for c in checks if not c["pass"]]
    to_record = [c for c in checks if c["pass"] and c["state"] not in ("pass",)]
    return blocking, to_record


def s0f(d, out):
    checks = []
    fr, pr = load(f"{d}/e4-frozen.json"), load(f"{d}/e4-proto.json")
    fp, pp = by_name(fr, checks, "E4 frozen"), by_name(pr, checks, "E4 proto")
    exact("(1) E4 parent sets are the three named parents on both probes", sorted(fp) == sorted(E4_PARENTS) == sorted(pp), checks)
    exact("(1) E4 at None: prototype equals frozen in the deterministic projection on every parent",
          all(k in pp and projection(fp[k]) == projection(pp[k]) for k in fp), checks,
          [k for k in fp if k in pp and projection(fp[k]) != projection(pp[k])])
    exact("(1) E4 config blocks equal", fr["config"] == pr["config"], checks, {"frozen": fr["config"], "proto": pr["config"]})
    sf, sn, sk = load(f"{d}/s0-founder-frozen.json"), load(f"{d}/s0-founder-none.json"), load(f"{d}/s0-founder-keep003.json")
    sfr, snr, skr = by_name(sf, checks, "S0 frozen"), by_name(sn, checks, "S0 None"), by_name(sk, checks, "S0 0.03")
    exact("(1) S0 parent set is the founder alone at every setting", sorted(sfr) == sorted(snr) == sorted(skr) == ["founder"], checks)
    exact("(1) S0 founder at None: prototype equals frozen in the deterministic projection",
          all(k in snr and projection(sfr[k]) == projection(snr[k]) for k in sfr), checks)
    exact("(1) S0 config blocks equal at None", sf["config"] == sn["config"], checks, {"frozen": sf["config"], "proto": sn["config"]})
    exact("S0 0.03 config differs from None's only in the keep field",
          {k: v for k, v in sk["config"].items() if k != "deletion_keep_probability"} == {k: v for k, v in sn["config"].items() if k != "deletion_keep_probability"}
          and sk["config"]["deletion_keep_probability"] == KEEP and sn["config"]["deletion_keep_probability"] is None, checks)
    loss = pp["founder+declared"]["lost_declaration"]
    containment(f"(2) founder+declared lost_declaration contains E4's {E4_LOSS}", loss["count"], loss["of"], E4_LOSS, checks)
    pooled_out = {}
    d_fail = manipulation(snr, skr, checks, pooled_out, with_c=False)
    blocking, to_record = verdict(checks)
    result = {"gate": "S0-F", "checks": checks, "pooled": pooled_out, "blocking": [c["check"] for c in blocking],
              "to_record": [c["check"] for c in to_record], "pass": not blocking}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print(c["state"].upper(), c["check"])
    print("S0-F", "pass" if not blocking else "FAIL", "to record:", len(to_record))
    return 1 if blocking else (4 if to_record else 0)


def s0(d, out):
    checks = []
    none, keep = load(f"{d}/s0-none.json"), load(f"{d}/s0-keep003.json")
    n_rows, k_rows = by_name(none, checks, "S0 None"), by_name(keep, checks, "S0 0.03")
    # A1 finding 6: the exact parent set, from the two named cohort dumps.
    dumps = none.get("dumps") or []
    exact("two cohort dumps, default-s1022 and default-s2022 at tick 20,000, at both settings",
          sorted((x["world"], x["seed"], x["tick"]) for x in dumps) == sorted((w, s, 20000) for w, s in COHORTS.items())
          and dumps == keep.get("dumps"), checks, dumps)
    evolved = sorted(p for p in n_rows if p != "founder")
    per_cohort = {w: sum(1 for p in evolved if p.startswith(f"{w}:t20000:")) for w in COHORTS}
    exact("the founder plus 40 evolved parents, 20 from each cohort", "founder" in n_rows and len(evolved) == 40
          and all(v == 20 for v in per_cohort.values()), checks, per_cohort)
    births = {r["births"] for r in list(n_rows.values()) + list(k_rows.values())}
    exact("one birth count for every parent at both settings", len(births) == 1, checks, sorted(births))
    exact("configs differ only in the keep field (None against 0.03)",
          {k: v for k, v in keep["config"].items() if k != "deletion_keep_probability"} == {k: v for k, v in none["config"].items() if k != "deletion_keep_probability"}
          and keep["config"]["deletion_keep_probability"] == KEEP and none["config"]["deletion_keep_probability"] is None, checks)
    rows = [n_rows[p] for p in evolved]
    founder = n_rows.get("founder")
    classes = {}
    for c in CLASSES:
        carriers = [r for r in rows if r["classes"][c]["sites"] >= 1]
        ls = [r["classes"][c]["l_400"] for r in carriers]
        per_parent = []
        for r in carriers:
            cl = r["classes"][c]
            # A1 finding 11: an interval on h per parent (applied events over
            # site-births, Poisson-like at this n), transformed to L.
            lo, hi = cp95(cl["applied"], cl["sites"] * r["births"])
            per_parent.append({"parent": r["parent"], "sites": cl["sites"], "applied": cl["applied"], "h": cl["h"], "h_interval": [lo, hi],
                               "l_400": cl["l_400"], "l_400_interval": [1 - (1 - lo) ** LINE, 1 - (1 - hi) ** LINE]})
        classes[c] = {
            "operator": rows[0]["classes"][c]["operator"] if rows else None,
            "carriers": len(carriers), "qualifies": len(carriers) >= 10,
            "median_l_400": statistics.median(ls) if ls else None,
            "median_l_400_of_lower_bounds": statistics.median([p["l_400_interval"][0] for p in per_parent]) if per_parent else None,
            "median_l_400_of_upper_bounds": statistics.median([p["l_400_interval"][1] for p in per_parent]) if per_parent else None,
            "median_h": statistics.median([p["h"] for p in per_parent]) if per_parent else None,
            "l_400_quartiles": statistics.quantiles(ls, n=4) if len(ls) >= 2 else ls,
            "median_sites": statistics.median([p["sites"] for p in per_parent]) if per_parent else None,
            "founder": None if founder is None else {k: founder["classes"][c][k] for k in ("sites", "applied", "h", "l_400")},
            "per_parent": per_parent,
        }
    go_classes = [c for c, v in classes.items() if v["qualifies"] and v["median_l_400"] is not None and v["median_l_400"] >= 0.5]
    pooled_out = {}
    d_fail = manipulation({p: n_rows[p] for p in n_rows}, {p: k_rows[p] for p in k_rows}, checks, pooled_out, with_c=True)

    def med(xs):
        xs = [x for x in xs if x is not None]
        return statistics.median(xs) if xs else None

    desc = {
        "r_edges_median": med([r["r_edges"]["r"] for r in rows]),
        "r_entries_median": med([r["r_entries"]["r"] for r in rows]),
        "remove_node_by_reachability_pooled": {k: sum(r["remove_node_applied_by_reachability"].get(k, 0) for r in rows)
                                               for k in sorted({k for r in rows for k in r["remove_node_applied_by_reachability"]})},
        "mesh_sites_reachable_median": med([r["sites"].get("mesh_node_reachable") for r in rows]),
        "mesh_sites_unreachable_median": med([r["sites"].get("mesh_node_unreachable") for r in rows]),
        "parents_with_unreachable_non_entry_nodes": sum(1 for r in rows if r["sites"].get("mesh_node_unreachable", 0) > 0),
        "genome_size_median": med([r["genome_size"] for r in rows]),
        "requested_per_birth_median": med([r["requested_events_per_birth"] for r in rows]),
    }
    blocking, to_record = verdict(checks)
    c_misses = [op for op, e in pooled_out["decreasing"].items() if e.get("c_plan_form") and not e["c_plan_form"]["contains"]]
    result = {
        "gate": "S0", "births_per_parent": none["config"]["births_per_parent"], "evolved_parents": len(rows),
        "dumps": dumps, "classes": classes, "go_classes": go_classes, "go": bool(go_classes),
        "manipulation": {"checks": checks, "pooled": pooled_out, "c_descriptive_misses": c_misses, "d_failures": d_fail},
        "blocking": [c["check"] for c in blocking], "to_record": [c["check"] for c in to_record] + [f"(c) {op}" for op in c_misses],
        "descriptive": desc,
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c, v in classes.items():
        print(f"{c:20s} carriers {v['carriers']:2d} median L {v['median_l_400']} [{v['median_l_400_of_lower_bounds']}, "
              f"{v['median_l_400_of_upper_bounds']}] median h {v['median_h']} founder {v['founder']}")
    for c in checks:
        print(c["state"].upper(), c["check"])
    print("S0 evolved parents", len(rows), "go" if go_classes else "no-go", go_classes, "to record:", result["to_record"])
    if blocking:
        return 2
    if not go_classes:
        return 3
    return 4 if result["to_record"] else 0


if __name__ == "__main__":
    mode, d, out = sys.argv[1:4]
    sys.exit(s0f(d, out) if mode == "s0f" else s0(d, out))
