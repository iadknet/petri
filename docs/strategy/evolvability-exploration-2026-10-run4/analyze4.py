"""Run 4 rule 4: noise-calibrated paired outcomes of every arm X against the A/A
control N, both read against the reference R, on the 48 fresh pairs per assay.

usage: python3 analyze4.py [--pooled]   (from the worktree root)

Per pair (seed batch s, replicate i):
- reach: X-only when X reached, R did not and N did not; N-only the converse.
- rung: on the eligible set E (R did not reach, R reads `fail` at its stall
  rung, N reads `pass` or `fail` there); N departs on `pass`; X departs on
  `pass` (strict, for A) or on anything but `fail` (lenient, for B).
- masking (B only): rows of `mask-<arm>-<combo>.ndjson` (run 3's r3_masking
  probe); a masked reach makes the pair X-only for reach, a masked rung on a
  pair in E makes it X-only for the rung; a missing probe output for a pair
  whose elites attempt reproduction is unresolved.

Outputs one JSON line per arm and assay (and the A/A control's own line), and
writes analysis4.json beside this file. `--pooled` adds run 3's batches 3 and
4 as exploratory (arms from run 3's evidence)."""
import json
import math
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from cpmath import beta_ppf_lower, beta_ppf_upper  # noqa: E402

HERE = ".bench-artifacts/lab/exploration/run4"
RUN3 = "/Users/istefanek/projects/petri/.bench-artifacts/lab/exploration/run3"
ARMS = ["m1", "m2", "refine", "m1refine", "m4", "m5"]
ALPHA = 0.025


def cp_upper(b, n):
    return 1.0 if b >= n - 1 else beta_ppf_upper(b, n, ALPHA)


def cp_lower(c, n):
    return 0.0 if c <= 1 else beta_ppf_lower(c, n, ALPHA)


def load(path):
    with open(path) as f:
        return json.load(f)


def arm(summary, name):
    return next(a for a in summary["arms"] if a["name"] == name)


def ladder(summary, name):
    return next(a for a in summary["ladder"]["arms"] if a["name"] == name)


def status_at(rep, rung):
    for s in rep["statuses"]:
        if s["rung"] == rung:
            return s["status"]
    return None


def seeds(assay):
    with open(f"{HERE}/seeds-{assay}.txt") as f:
        return [int(line) for line in f if line.strip()]


def mask_rows(path):
    if not os.path.exists(path):
        return None
    rows = {}
    with open(path) as f:
        for line in f:
            row = json.loads(line)
            if row.get("probe") == "r3-masking":
                rows[row["replicate"]] = row
            if row.get("probe") == "r3-masking-verdict":
                rows["verdict"] = row
    # A probe run without a passing scene-identity check is unresolved.
    return rows if rows.get("verdict", {}).get("scene_identity") is True else None


REPLICATES = 8
HOLDOUTS = {101, 102, 111, 112, 121, 122}


def replicate_ids(arm_summary):
    return [rep["replicate"] for rep in arm_summary["replicates"]]


def pairs(x, assay, s, x_dir, n_dir):
    """Per-replicate pair readings for arm x (or None for N alone).

    Reach is tri-state (True, False, None for an incomplete replicate); an
    unknown reading never supplies X-only or N-only evidence. R's stall rung
    exists only when R did not reach (run 3 closing review 1, disposition 1:
    a reached reference replicate has no stall rung and the rung contrast
    never uses that pair), hence `r_reached is False` in E."""
    sx = load(f"{x_dir}/summary.json") if x else None
    sn = load(f"{n_dir}/summary.json")
    r_arm, n_arm = arm(sn, "native"), arm(sn, "native-aa")
    r_lad, n_lad = ladder(sn, "native"), ladder(sn, "native-aa")
    for a_ in (r_arm, n_arm, r_lad, n_lad):
        assert replicate_ids(a_) == list(range(REPLICATES)), f"{n_dir}: replicate ids"
    if sx is not None:
        assert arm(sx, "native")["replicates"] == r_arm["replicates"], f"{x_dir}: R results differ"
        assert ladder(sx, "native")["replicates"] == r_lad["replicates"], f"{x_dir}: R ladder differs"
        x_arm, x_lad = arm(sx, x), ladder(sx, x)
        assert replicate_ids(x_arm) == list(range(REPLICATES)), f"{x_dir}: replicate ids"
        assert replicate_ids(x_lad) == list(range(REPLICATES)), f"{x_dir}: ladder replicate ids"
    fraction = (sn.get("calibration") or {}).get("selected", {}).get("food_fraction")
    out = []
    for i, r in enumerate(r_arm["replicates"]):
        rung = r_lad["replicates"][i]["first_not_pass"]
        r_status = status_at(r_lad["replicates"][i], rung) if rung else None
        n_status = status_at(n_lad["replicates"][i], rung) if rung else None
        n_reached = n_arm["replicates"][i]["reached"]
        eligible = r["reached"] is False and rung is not None and r_status == "fail" \
            and n_status in ("pass", "fail")
        row = {"seed": s, "replicate": i, "fraction": fraction, "r_reached": r["reached"],
               "n_reached": n_reached, "r_rung": rung, "r_status": r_status,
               "n_status": n_status, "eligible": eligible,
               "n_reach_effect": n_reached is True and r["reached"] is False,
               "n_departs": eligible and n_status == "pass",
               "r_incomplete": r["incomplete"], "n_incomplete": n_arm["replicates"][i]["incomplete"]}
        if sx is not None:
            xr = x_arm["replicates"][i]
            x_status = status_at(x_lad["replicates"][i], rung) if rung else None
            known = r["reached"] is False
            row.update({"x_reached": xr["reached"], "x_status": x_status,
                        "x_reach_effect": xr["reached"] is True and r["reached"] is False,
                        "x_only_reach": known and xr["reached"] is True and n_reached is False,
                        "n_only_reach": known and n_reached is True and xr["reached"] is False,
                        "x_departs_strict": eligible and x_status == "pass",
                        "x_departs_lenient": eligible and x_status != "fail",
                        "x_incomplete": xr["incomplete"],
                        "x_first_not_pass": x_lad["replicates"][i]["first_not_pass"],
                        "final_best_gap": None if xr["final_best"] is None or r["final_best"] is None
                        else xr["final_best"] - r["final_best"]})
        out.append(row)
    return out


def counts(rows, xk, nk, pool=None):
    pool = rows if pool is None else pool
    b = sum(1 for p in pool if p[xk] and not p[nk])
    c = sum(1 for p in pool if p[nk] and not p[xk])
    n = len(pool)
    res = {"n": n, "b": b, "c": c}
    if n:
        res.update({"excess": (b - c) / n, "U": cp_upper(b, n) - cp_lower(c, n)})
    return res


def counts_bc(pool, bk, ck):
    """b and c read directly from per-pair X-only / N-only flags."""
    b = sum(1 for p in pool if p[bk])
    c = sum(1 for p in pool if p[ck])
    n = len(pool)
    res = {"n": n, "b": b, "c": c}
    if n:
        res.update({"excess": (b - c) / n, "U": cp_upper(b, n) - cp_lower(c, n)})
    return res


def baseline_log_ok(arms_and_combos):
    """Every invocation must have logged a passing rule 5 check, and none a failure."""
    with open(f"{HERE}/run4.log") as f:
        log = f.read().splitlines()
    if any("MISMATCH" in line or " NO " in line for line in log):
        return False, ["a MISMATCH or NO line is in run4.log"]
    missing = []
    for a_, combo in arms_and_combos:
        ok = (f"{a_} {combo} baseline ok", f"{a_} {combo} built-in rows and projection without digest ok")
        if not any(line in ok for line in log):
            missing.append(f"{a_} {combo}")
    return not missing, missing


def formal_seeds(assay):
    s = seeds(assay)
    ok = len(s) == 6 and len(set(s)) == 6 and all(x >= 5 and x not in HOLDOUTS for x in s)
    return s, ok


def analyse(assay, x, fresh=True, pooled=False):
    rows = []
    batches, formal = formal_seeds(assay) if fresh else ([], False)
    if not formal and not pooled:
        return {"assay": assay, "arm": x, "missing": "formal seed batches are not six distinct fresh seeds"}
    if pooled:
        batches = [3, 4] + batches
    for s in batches:
        combo = f"{assay}-s{s}"
        n_dir = f"{HERE}/aa-{combo}"
        if x is None:
            x_dir = None
        elif s in (3, 4):
            x_dir = f"{RUN3}/{x}-{combo}"
        else:
            x_dir = f"{HERE}/{x}-{combo}"
        for d in [n_dir] + ([x_dir] if x_dir else []):
            if not os.path.exists(f"{d}/summary.json"):
                return {"assay": assay, "arm": x, "missing": d}
        rows += pairs(x, assay, s, x_dir, n_dir)
    if not pooled and len(rows) != REPLICATES * 6:
        return {"assay": assay, "arm": x, "missing": f"{len(rows)} pairs, not 48"}
    e = [p for p in rows if p["eligible"]]
    n_dep = sum(1 for p in e if p["n_departs"])
    scope = len(e) >= 16 and n_dep <= 0.25 * len(e)
    res = {"assay": assay, "arm": x or "native-aa", "pairs": len(rows),
           "incomplete": sum(1 for p in rows if p["r_incomplete"] or p["n_incomplete"]
                             or p.get("x_incomplete")),
           "R_reached": sum(1 for p in rows if p["r_reached"]),
           "N_reach_effect": sum(1 for p in rows if p["n_reach_effect"]),
           "E": len(e), "N_departs_in_E": n_dep, "rung_in_scope": scope}
    if x is None:
        return res, rows
    res["raw"] = {"arm_only_reach_vs_R": sum(1 for p in rows if p["x_reach_effect"]),
                  "R_only_reach": sum(1 for p in rows if p["r_reached"] and not p["x_reached"]),
                  "departures_strict_in_E": sum(1 for p in e if p["x_departs_strict"]),
                  "departures_lenient_in_E": sum(1 for p in e if p["x_departs_lenient"]),
                  "final_best_gap_mean": mean([p["final_best_gap"] for p in rows])}
    res["reach"] = counts_bc(rows, "x_only_reach", "n_only_reach")
    res["rung_strict"] = counts(e, "x_departs_strict", "n_departs")
    res["rung_lenient"] = counts(e, "x_departs_lenient", "n_departs")
    # Rule 3's descriptive breakdowns: per seed batch and per food density.
    per = {}
    for key, sel in [(f"seed {s}", lambda p, s=s: p["seed"] == s) for s in batches] + \
            [(f"fraction {f}", lambda p, f=f: p["fraction"] == f)
             for f in sorted({p["fraction"] for p in rows if p["fraction"] is not None})]:
        sub = [p for p in rows if sel(p)]
        sub_e = [p for p in sub if p["eligible"]]
        per[key] = {"pairs": len(sub), "reach": counts_bc(sub, "x_only_reach", "n_only_reach"),
                    "rung_strict": counts(sub_e, "x_departs_strict", "n_departs")}
    res["breakdown"] = per
    # Masking (B only).
    unresolved = []
    masked_reach = [dict(p) for p in rows]
    masked_rung = [dict(p) for p in e]
    by_key = {(p["seed"], p["replicate"]): p for p in masked_rung}
    for s in batches:
        combo = f"{assay}-s{s}"
        m = mask_rows(f"{HERE}/mask-{x}-{combo}.ndjson")
        if m is None:
            unresolved.append(combo)
            continue
        for p in masked_reach:
            if p["seed"] == s and p["replicate"] in m and m[p["replicate"]]["masked_reach"]:
                p["x_only_reach"], p["n_only_reach"] = True, False
        for (ks, ki), p in by_key.items():
            if ks == s and ki in m and m[ki]["masked_rung"]:
                p["x_departs_lenient"], p["n_departs"] = True, False
    res["masking_unresolved"] = unresolved
    res["reach_masked"] = counts_bc(masked_reach, "x_only_reach", "n_only_reach")
    res["rung_lenient_masked"] = counts(masked_rung, "x_departs_lenient", "n_departs")
    a_reach = res["reach"]["n"] >= 16 and \
        res["reach"]["b"] - res["reach"]["c"] >= math.ceil(res["reach"]["n"] / 4)
    a_rung = scope and res["rung_strict"]["n"] > 0 and \
        res["rung_strict"]["b"] - res["rung_strict"]["c"] >= math.ceil(res["rung_strict"]["n"] / 4)
    res["A_reach"], res["A_rung"] = a_reach, a_rung
    res["B_reach"] = res["reach"]["n"] >= 16 and res["reach_masked"]["U"] < 0.25
    res["B_rung"] = (not scope) or res["rung_lenient_masked"]["U"] < 0.25
    return res, rows


def mean(values):
    v = [x for x in values if x is not None]
    return sum(v) / len(v) if v else None


def main():
    pooled = "--pooled" in sys.argv
    out = {"control": {}, "arms": {}, "outcome": None}
    for assay in ("food", "wall"):
        res, rows = analyse(assay, None, pooled=pooled)
        out["control"][assay] = res
        print(json.dumps(res))
    for x in ARMS:
        out["arms"][x] = {}
        for assay in ("food", "wall"):
            res = analyse(assay, x, pooled=pooled)
            res = res[0] if isinstance(res, tuple) else res
            out["arms"][x][assay] = res
            print(json.dumps(res))
    complete = all("missing" not in out["arms"][x][a] for x in ARMS for a in ("food", "wall"))
    if not pooled:
        combos = [(x, f"{a}-s{s}") for x in ARMS for a in ("food", "wall") for s in seeds(a)]
        combos += [("aa", f"{a}-s{s}") for a in ("food", "wall") for s in [3, 4] + seeds(a)]
        log_ok, log_missing = baseline_log_ok(combos)
        out["baseline_log"] = {"ok": log_ok, "missing": log_missing}
        complete = complete and log_ok
    else:
        out["note"] = "pooled is exploratory; masks for seeds 3 and 4 are not computed (masking unavailable there)"
    if complete and not pooled:
        cand = [x for x in ARMS
                if (out["arms"][x]["food"]["A_reach"] and out["arms"][x]["wall"]["A_reach"])
                or (out["arms"][x]["food"]["A_rung"] and out["arms"][x]["wall"]["A_rung"])]
        # An incomplete (byte-capped) replicate has a censored reach, so it
        # blocks B (conservative; decided before the analysis point).
        b_ok = all(not out["arms"][x][a]["masking_unresolved"] and out["arms"][x][a]["B_reach"]
                   and out["arms"][x][a]["B_rung"] and out["arms"][x][a]["incomplete"] == 0
                   for x in ARMS for a in ("food", "wall"))
        out["outcome"] = {"A_candidates": cand, "B_supported": b_ok and not cand}
        print(json.dumps(out["outcome"]))
    name = "analysis4-pooled.json" if pooled else "analysis4.json"
    with open(f"{HERE}/{name}", "w") as f:
        json.dump(out, f, indent=1)


if __name__ == "__main__":
    main()
