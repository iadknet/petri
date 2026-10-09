"""Deletion-rate run gates S0-F and S0 (docs/strategy/deletion-rate-run-2026-10.md).

usage:
  python3 s0read.py s0f <dir> <out.json>
      <dir> holds e4-frozen.json, e4-proto.json (e4_transition_rates at None,
      10^6 births), s0-founder-frozen.json, s0-founder-none.json and
      s0-founder-keep003.json (s0_deletion_hazards, founder only).
      Exit 0 when every S0-F check passes, 1 otherwise.
  python3 s0read.py s0 <dir> <out.json>
      <dir> holds s0-none.json and s0-keep003.json (founder plus the cohort
      dump parents). Exit 0 on a go with the manipulation check's (a) and (b)
      passing, 3 on a no-go, 2 when (a) or (b) fails (the treatment arms do
      not run), 4 on a go with a lone (c) or (d) failure (recorded, the arms
      run)."""
import json
import math
import statistics
import sys

E4_LOSS = 0.018643
KEEP = 0.03
OTHER_MIN = 4000
OTHER_TOL = 0.05
DECREASING = ("InputRefPrune", "GraphRemoveGraphEdge", "GraphRemoveInternalGraphNode", "GraphDisableHebbian",
              "GraphDisableRewardModulation", "TopologyRemoveNode", "TopologyRemoveRouteTarget", "VmDeleteInstruction")
CLASSES = ("silent_input_entry", "graph_edge", "compute_node", "plasticity_config", "reward_config", "mesh_node",
           "route_target", "vm_instruction")


def betacf(a, b, x):
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c, d = 1.0, 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
    h = d
    for m in range(1, 400):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
        c = 1.0 + aa / c if abs(1.0 + aa / c) > 1e-300 else 1e-300
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        d = 1.0 / (d if abs(d) > 1e-300 else 1e-300)
        c = 1.0 + aa / c if abs(1.0 + aa / c) > 1e-300 else 1e-300
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-12:
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
    if n == 0:
        return (0.0, 1.0)
    lo = 0.0 if k == 0 else beta_inv(0.025, k, n - k + 1)
    hi = 1.0 if k >= n else beta_inv(0.975, k + 1, n - k)
    return (lo, hi)


def two_sided_p(k, n, p):
    """Exact two-sided binomial p-value by doubling the smaller tail."""
    if n == 0:
        return 1.0
    lower = 1.0 - betai(k + 1, n - k, p) if k < n else 1.0  # P(X <= k)
    upper = betai(k, n - k + 1, p) if k > 0 else 1.0  # P(X >= k)
    return min(1.0, 2 * min(lower, upper))


def load(path):
    return json.loads(open(path).read())


def projection(row):
    return {k: v for k, v in row.items() if k != "wall_secs"}


def by_name(doc):
    return {r["parent"]: r for r in doc["parents"]}


def manipulation(none, keep, checks, pooled_out):
    """The manipulation check at 0.03 pooled over the parents of two S0 runs."""
    n_rows, k_rows = by_name(none), by_name(keep)
    names = sorted(n_rows)
    req = [(p, n_rows[p]["requested_events"], k_rows.get(p, {}).get("requested_events")) for p in names]
    a_ok = len(n_rows) == len(k_rows) and all(x == y for _, x, y in req)
    checks.append({"check": "(a) requested events equal to None's for every parent", "pass": a_ok,
                   "detail": [r for r in req if r[1] != r[2]][:10]})
    births = sum(n_rows[p]["births"] for p in names)
    ops = sorted({op for r in list(n_rows.values()) + list(k_rows.values()) for op in r["applied_by_operator"]})

    def pooled(rows, op):
        return sum(r["applied_by_operator"].get(op, 0) for r in rows.values())

    def not_kept(rows, op):
        return sum(r["skipped_by_operator_reason"].get(f"{op}|DeletionNotKept", 0) for r in rows.values())

    b_ok, c_fail, d_fail = True, [], []
    for op in DECREASING:
        a0, a1, nk = pooled(n_rows, op), pooled(k_rows, op), not_kept(k_rows, op)
        entry = {"applied_none": a0, "applied_keep": a1, "not_kept": nk}
        if a1 + nk > 0:
            lo, hi = cp95(a1, a1 + nk)
            entry["kept_share"] = {"k": a1, "n": a1 + nk, "rate": a1 / (a1 + nk), "cp95": [lo, hi], "contains": lo <= KEEP <= hi,
                                   "p": two_sided_p(a1, a1 + nk, KEEP)}
            b_ok &= lo <= KEEP <= hi
        lo, hi = cp95(a1, births)
        target = KEEP * a0 / births
        entry["applied_rate_keep"] = {"k": a1, "births": births, "cp95": [lo, hi], "target": target, "contains": lo <= target <= hi,
                                      "p": two_sided_p(a1, births, target) if target > 0 else None}
        if not lo <= target <= hi:
            c_fail.append(op)
        pooled_out["decreasing"][op] = entry
    checks.append({"check": "(b) every Decreasing operator's kept share has a CP interval containing 0.03", "pass": b_ok,
                   "detail": {op: pooled_out["decreasing"][op].get("kept_share") for op in DECREASING}})
    checks.append({"check": "(c) every Decreasing operator's applied rate at 0.03 has a CP interval containing 0.03 x its None rate",
                   "pass": not c_fail, "detail": c_fail})
    for op in ops:
        if op in DECREASING:
            continue
        a0, a1 = pooled(n_rows, op), pooled(k_rows, op)
        rel = (a1 - a0) / a0 if a0 else None
        pooled_out["other"][op] = {"applied_none": a0, "applied_keep": a1, "relative": rel, "checked": a0 >= OTHER_MIN}
        if a0 >= OTHER_MIN and abs(rel) > OTHER_TOL:
            d_fail.append(op)
    checks.append({"check": f"(d) every other operator with >= {OTHER_MIN} applied at None within {OTHER_TOL:.0%} of None", "pass": not d_fail,
                   "detail": {op: pooled_out["other"][op] for op in d_fail}})
    return a_ok, b_ok, c_fail, d_fail


def s0f(d, out):
    checks = []
    fr, pr = load(f"{d}/e4-frozen.json"), load(f"{d}/e4-proto.json")
    fp, pp = by_name(fr), by_name(pr)
    same = sorted(fp) == sorted(pp) and all(projection(fp[k]) == projection(pp[k]) for k in fp)
    checks.append({"check": "(1) E4 at None: prototype probe equals the frozen probe in the deterministic projection on all three parents",
                   "pass": same, "detail": [k for k in fp if k in pp and projection(fp[k]) != projection(pp[k])]})
    checks.append({"check": "(1) E4 config blocks equal", "pass": fr["config"] == pr["config"], "detail": {"frozen": fr["config"], "proto": pr["config"]}})
    sf, sn = load(f"{d}/s0-founder-frozen.json"), load(f"{d}/s0-founder-none.json")
    sfr, snr = by_name(sf), by_name(sn)
    checks.append({"check": "(1) S0 founder row at None: prototype probe equals the frozen probe in the deterministic projection",
                   "pass": sorted(sfr) == sorted(snr) and all(projection(sfr[k]) == projection(snr[k]) for k in sfr)})
    checks.append({"check": "(1) S0 config blocks equal", "pass": sf["config"] == sn["config"], "detail": {"frozen": sf["config"], "proto": sn["config"]}})
    loss = pp["founder+declared"]["lost_declaration"]
    checks.append({"check": f"(2) founder+declared lost_declaration CP interval contains E4's {E4_LOSS}",
                   "pass": loss["cp95"][0] <= E4_LOSS <= loss["cp95"][1], "detail": loss})
    keep = load(f"{d}/s0-founder-keep003.json")
    pooled = {"decreasing": {}, "other": {}}
    manipulation(sn, keep, checks, pooled)
    result = {"gate": "S0-F", "checks": checks, "pooled": pooled, "pass": all(c["pass"] for c in checks)}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("S0-F", "pass" if result["pass"] else "FAIL")
    return 0 if result["pass"] else 1


def s0(d, out):
    none, keep = load(f"{d}/s0-none.json"), load(f"{d}/s0-keep003.json")
    rows = [r for r in none["parents"] if r["parent"] != "founder"]
    founder = next((r for r in none["parents"] if r["parent"] == "founder"), None)
    classes = {}
    for c in CLASSES:
        carriers = [r for r in rows if r["classes"][c]["sites"] >= 1]
        ls = [r["classes"][c]["l_400"] for r in carriers]
        hs = [r["classes"][c]["h"] for r in carriers]
        classes[c] = {
            "operator": rows[0]["classes"][c]["operator"] if rows else None,
            "carriers": len(carriers), "qualifies": len(carriers) >= 10,
            "median_l_400": statistics.median(ls) if ls else None,
            "median_h": statistics.median(hs) if hs else None,
            "l_400_quartiles": statistics.quantiles(ls, n=4) if len(ls) >= 2 else ls,
            "median_sites": statistics.median([r["classes"][c]["sites"] for r in carriers]) if carriers else None,
            "founder": None if founder is None else {k: founder["classes"][c][k] for k in ("sites", "applied", "h", "l_400")},
        }
    go_classes = [c for c, v in classes.items() if v["qualifies"] and v["median_l_400"] is not None and v["median_l_400"] >= 0.5]
    checks = []
    pooled = {"decreasing": {}, "other": {}}
    a_ok, b_ok, c_fail, d_fail = manipulation(none, keep, checks, pooled)

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
        "genome_size_median": med([r["genome_size"] for r in rows]),
        "requested_per_birth_median": med([r["requested_events_per_birth"] for r in rows]),
    }
    result = {
        "gate": "S0", "births_per_parent": none["config"]["births_per_parent"], "evolved_parents": len(rows),
        "dumps": none.get("dumps"), "classes": classes, "go_classes": go_classes, "go": bool(go_classes),
        "manipulation": {"checks": checks, "pooled": pooled, "a": a_ok, "b": b_ok, "c_failures": c_fail, "d_failures": d_fail},
        "descriptive": desc,
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c, v in classes.items():
        print(f"{c:20s} carriers {v['carriers']:2d} median L {v['median_l_400']} median h {v['median_h']} founder {v['founder']}")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("S0 evolved parents", len(rows), "go" if go_classes else "no-go", go_classes)
    if not (a_ok and b_ok):
        return 2
    if not go_classes:
        return 3
    return 4 if (c_fail or d_fail) else 0


if __name__ == "__main__":
    mode, d, out = sys.argv[1:4]
    sys.exit(s0f(d, out) if mode == "s0f" else s0(d, out))
