"""Run 5 analysis (plan rules 5-7): merge r5_screen range files against a
frozen expected-coverage manifest, enforce the validity gate, and emit one
gated J verdict: `pass`, `supported negative` or `inconclusive`.

usage:
  python3 analyze5.py size  <pilot-dir> <manifest-out>      # rule 5 sizing (writes N into the manifest)
  python3 analyze5.py pcj   <pcj-pilot-dir> <manifest>      # rule 6 PC-J power (writes it into the manifest)
  python3 analyze5.py main  <manifest> <main-dir> <aa-dir> <rerun-dir> <aacheck-dir> [<pcj-dir>]
  python3 analyze5.py aacheck <probe-aa-dir> <main-dir> <aa-dir> <assay> <stratum> <end>

A directory holds r5_screen NDJSON range files (`*.ndjson`): summary rows have
probe == "r5-screen", child records probe == "r5-screen-child".

The manifest (JSON, frozen and committed before the main draw):
  {"assays": {"food": {"strata": ["0", ..., "15"], "N": int}, "wall": {...}},
   "fixtures": {"passed": true, "commits": [...]},
   "pcj": {"status": "constructed" | "not constructible", "N": 1000000,
           "power": float | null, "powered": bool}}"""
import json
import math
import os
import sys
from collections import defaultdict

ALPHA = 0.025
P_JUMP = 0.1
FIRE_TAIL = 0.0005  # each tail of the 99.9 % two-sided exact binomial range
PCJ_N = 1_000_000


# ---------- exact binomial helpers (log space; n may be in the millions) ----------

def _log_pmf(i, n, lp, lq, lg):
    return lg - math.lgamma(i + 1) - math.lgamma(n - i + 1) + i * lp + (n - i) * lq


def binom_cdf(k, n, p):
    """P(Binomial(n, p) <= k)."""
    if k < 0:
        return 0.0
    if k >= n:
        return 1.0
    if p <= 0.0:
        return 1.0
    if p >= 1.0:
        return 0.0
    lp, lq, lg = math.log(p), math.log1p(-p), math.lgamma(n + 1)
    mode = int((n + 1) * p)

    def tail(indices):
        # Terms fall monotonically away from the mode, so stop once they are
        # negligible against the first (largest) term.
        total, first = 0.0, None
        for i in indices:
            lt = _log_pmf(i, n, lp, lq, lg)
            if first is None:
                first = lt
            elif lt < first - 60:
                break
            total += math.exp(lt)
        return total

    if k < mode:
        return min(1.0, tail(range(k, -1, -1)))
    return max(0.0, 1.0 - tail(range(k + 1, n + 1)))


def cp_upper(b, n, alpha=ALPHA):
    """Exact one-sided Clopper-Pearson upper bound; 1 when b >= n - 1 (plan convention)."""
    if b >= n - 1:
        return 1.0
    lo, hi = 0.0, 1.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if binom_cdf(b, n, mid) > alpha:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def cp_lower(c, n, alpha=ALPHA):
    """Exact one-sided Clopper-Pearson lower bound; 0 when c <= 1 (plan convention)."""
    if c <= 1:
        return 0.0
    lo, hi = 0.0, 1.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if 1.0 - binom_cdf(c - 1, n, mid) < alpha:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def half_tail_ge(k, m):
    """P(Binomial(m, 1/2) >= k): exact integer sum for m <= 2,000, else the
    log-space binomial tail (the A/A's independent seeds give m in the tens of
    thousands, where the integer sum stalls)."""
    if k <= 0:
        return 1.0
    if k > m:
        return 0.0
    if m <= 2_000:
        return sum(math.comb(m, i) for i in range(k, m + 1)) / 2 ** m
    return max(0.0, 1.0 - binom_cdf(k - 1, m, 0.5))


def sign_p(b, c):
    """One-sided exact sign test p for b > c."""
    return half_tail_ge(b, b + c)


def sign_p_two_sided(b, c):
    m = b + c
    if m == 0:
        return 1.0
    return min(1.0, 2 * half_tail_ge(max(b, c), m))


def fire_rate_ok(fired, events, p=P_JUMP):
    """Exact 99.9 % two-sided binomial range around p."""
    if events == 0:
        return False
    lower = binom_cdf(fired, events, p)            # P(X <= fired)
    upper = 1.0 - binom_cdf(fired - 1, events, p)  # P(X >= fired)
    return lower >= FIRE_TAIL and upper >= FIRE_TAIL


def bounds(b, c, n):
    up, lo = cp_upper(b, n), cp_lower(c, n)
    theta_u = math.inf if lo == 0 else up / lo
    return {"cp_upper_b": up, "cp_lower_c": lo, "theta_u": theta_u, "delta_u": up - lo}


def poisson_pmf(k, lam):
    if lam <= 0:
        return 1.0 if k == 0 else 0.0
    return math.exp(k * math.log(lam) - lam - math.lgamma(k + 1))


def sign_power_poisson(lam_b, lam_c):
    """Exact power of the one-sided sign test at ALPHA with b, c independent Poisson."""
    kb = int(lam_b + 10 * math.sqrt(lam_b + 1) + 20)
    kc = int(lam_c + 10 * math.sqrt(lam_c + 1) + 20)
    pc = [poisson_pmf(c, lam_c) for c in range(kc + 1)]
    power = 0.0
    for b in range(1, kb + 1):
        wb = poisson_pmf(b, lam_b)
        if wb < 1e-15:
            continue
        for c, wc in enumerate(pc):
            if wc >= 1e-15 and sign_p(b, c) <= ALPHA:
                power += wb * wc
    return power


# ---------- loading and coverage ----------

def load(dirpath):
    summaries, children = [], []
    if not dirpath or not os.path.isdir(dirpath):
        return summaries, children
    for name in sorted(os.listdir(dirpath)):
        if not name.endswith(".ndjson"):
            continue
        with open(os.path.join(dirpath, name)) as f:
            for line in f:
                row = json.loads(line)
                if row.get("probe") == "r5-screen":
                    row["_file"] = name
                    summaries.append(row)
                elif row.get("probe") == "r5-screen-child":
                    row["_file"] = name
                    children.append(row)
    return summaries, children


def counted(children, arm, tag, side="native", summaries=None, problems=None):
    """{file: set of (assay, stratum, index)} of `side` records that are
    confirmed and not sterility-attributable. With `summaries`, every record
    (any side) of this arm/tag must sit in a file holding that arm/tag's
    summary, with the same assay and stratum and an index inside its range;
    anything else is a problem (orphan or out-of-range record)."""
    owner = {}
    if summaries is not None:
        for s in summaries:
            if s.get("arm") == arm and s.get("tag") == tag:
                owner[s["_file"]] = s
    out = defaultdict(set)
    for ch in children:
        if ch.get("arm") != arm or ch.get("tag") != tag:
            continue
        if summaries is not None:
            s = owner.get(ch["_file"])
            if s is None:
                problems.append(f"records: {ch['_file']} has records but no {arm}/{tag} summary")
                continue
            lo, hi = s["range"]
            if ch.get("assay") != s["assay"] or str(ch.get("stratum")) != str(s["stratum"]) \
                    or not (isinstance(ch.get("index"), int) and lo <= ch["index"] < hi):
                problems.append(f"records: {ch['_file']} record outside its summary's assay, stratum or range")
                continue
        if ch.get("side") == side and ch.get("label") == "confirmed" \
                and not (ch.get("sterility") or {}).get("attributable"):
            out[ch["_file"]].add((ch["assay"], str(ch["stratum"]), ch["index"]))
    return out


def counted_native(children, arm, tag):
    return counted(children, arm, tag)


def aa_from_records(main_s, main_c, aa_s, aa_c, expected):
    """The A/A check (rule 6) computed by pairing native children on the
    `run5-aa` seeds (aa dir: arm `native`, tag `run5-aa`, whose arm side is a
    copy) with the main draw's native twins (tag `run5`) by child index. This is
    the probe's `aa` statistic (b = run5-aa-only counted, c = run5-only
    counted) without re-evaluating the main draw's native side. Records must be
    complete: per range file, the counted native records equal the summary's
    native.counted."""
    aa_m, problems = coverage(aa_s, "native", "run5-aa", expected)
    main_sets = counted(main_c, "j", "run5", summaries=main_s, problems=problems)
    aa_sets = counted(aa_c, "native", "run5-aa", summaries=aa_s, problems=problems)
    for s in main_s:
        if s.get("arm") == "j" and s.get("tag") == "run5":
            if len(main_sets.get(s["_file"], ())) != (s.get("native") or {}).get("counted"):
                problems.append(f"records: {s['_file']} native records do not match native.counted")
    for s in aa_s:
        if s.get("arm") == "native" and s.get("tag") == "run5-aa":
            if len(aa_sets.get(s["_file"], ())) != (s.get("native") or {}).get("counted"):
                problems.append(f"records: {s['_file']} native records do not match native.counted")
    n_all = set().union(*main_sets.values()) if main_sets else set()
    a_all = set().union(*aa_sets.values()) if aa_sets else set()
    cells = {}
    for assay, strata in expected.items():
        for stratum, n in strata.items():
            nn = {i for (a_, s_, i) in n_all if a_ == assay and s_ == stratum and i < n}
            aa = {i for (a_, s_, i) in a_all if a_ == assay and s_ == stratum and i < n}
            cells[(assay, stratum)] = {"b": len(aa - nn), "c": len(nn - aa)}
    return aa_m, cells, problems


META = ("parent_sha256", "arm_mutation_sha256", "native_mutation_sha256")
CELLS = ("b", "c", "both", "neither")
JFIELDS = ("applied_weight_events", "fired_jumps", "fired_births", "twin_identity_violations")


def _nonneg_int(v):
    return isinstance(v, int) and not isinstance(v, bool) and v >= 0


def row_problems(r):
    """Schema check for one r5-screen summary row: provenance present, integer
    range, complete non-negative cells summing to the range length, and for J
    rows non-negative counters with zero identity violations."""
    where = f"{r.get('_file')}"
    out = []
    if r.get("assay") not in ("food", "wall"):
        out.append(f"{where}: bad assay")
    rng = r.get("range")
    if not (isinstance(rng, list) and len(rng) == 2 and all(_nonneg_int(x) for x in rng) and rng[0] < rng[1]):
        out.append(f"{where}: bad range")
        return out
    for field in META:
        if not (isinstance(r.get(field), str) and r[field]):
            out.append(f"{where}: missing {field}")
    banks = r.get("banks")
    if not (isinstance(banks, dict) and all(isinstance(banks.get(k), dict) and isinstance(banks[k].get("sha256"), str)
                                            and banks[k]["sha256"] and _nonneg_int(banks[k].get("scenes"))
                                            and banks[k]["scenes"] > 0 for k in ("a", "b"))):
        out.append(f"{where}: missing or malformed banks")
    cells = (r.get("twin") or {}).get("cells")
    if not (isinstance(cells, dict) and set(cells) == set(CELLS) and all(_nonneg_int(cells[k]) for k in CELLS)):
        out.append(f"{where}: twin cells incomplete or not non-negative integers")
    elif sum(cells[k] for k in CELLS) != rng[1] - rng[0]:
        out.append(f"{where}: twin cells sum {sum(cells.values())} is not the range length {rng[1] - rng[0]}")
    if r.get("arm") == "j":
        j = r.get("j")
        if not (isinstance(j, dict) and all(_nonneg_int(j.get(k)) for k in JFIELDS)):
            out.append(f"{where}: J counters missing or not non-negative integers")
        elif j["twin_identity_violations"] > 0:
            out.append(f"{where}: {j['twin_identity_violations']} twin-identity violations")
    return out


def coverage(summaries, arm, tag, expected):
    """Merge one arm/tag against `expected` = {assay: {stratum: N}}. Returns
    (groups, problems); a group holds cells and j totals over 0..N."""
    groups = defaultdict(list)
    problems = []
    for s in summaries:
        if s.get("arm") == arm and s.get("tag") == tag:
            problems.extend(row_problems(s))
            groups[(s.get("assay"), str(s.get("stratum")))].append(s)
    for key in groups:
        if key[0] not in expected or key[1] not in expected[key[0]]:
            problems.append(f"{arm}/{tag}: unexpected group {key}")
    merged = {}
    for assay, strata in expected.items():
        for stratum, n in strata.items():
            rows = sorted(groups.get((assay, stratum), []), key=lambda r: r["range"][0])
            if not rows:
                problems.append(f"{arm}/{tag}: missing {assay}/{stratum}")
                continue
            if any(row_problems(r) for r in rows):
                continue  # already reported; never aggregate a malformed row
            pos = 0
            for r in rows:
                start, end = r["range"]
                if start != pos:
                    problems.append(f"{arm}/{tag}: {assay}/{stratum} gap or overlap at {pos} (next {start})")
                pos = max(pos, end)
            if pos != n:
                problems.append(f"{arm}/{tag}: {assay}/{stratum} covers 0..{pos}, expected 0..{n}")
            for field in META:
                values = {json.dumps(r.get(field), sort_keys=True) for r in rows}
                if len(values) > 1:
                    problems.append(f"{arm}/{tag}: {assay}/{stratum} conflicting {field}")
            banks = {json.dumps(r.get("banks"), sort_keys=True) for r in rows}
            if len(banks) > 1:
                problems.append(f"{arm}/{tag}: {assay}/{stratum} conflicting banks")
            cells, jstats, violations = defaultdict(int), defaultdict(int), 0
            for r in rows:
                for k, v in r["twin"]["cells"].items():
                    cells[k] += v
                j = r.get("j") or {}
                for k in ("applied_weight_events", "fired_jumps", "fired_births"):
                    jstats[k] += j.get(k) or 0
                if arm == "j" and not isinstance(j.get("twin_identity_violations"), int):
                    problems.append(f"{arm}/{tag}: {assay}/{stratum} {r['_file']} lacks twin_identity_violations")
                violations += j.get("twin_identity_violations") or 0
            merged[(assay, stratum)] = {"n": pos, "cells": dict(cells), "j": dict(jstats),
                                        "violations": violations, "parent": rows[0].get("parent_sha256"),
                                        "banks": rows[0].get("banks"),
                                        "arm_mutation": rows[0].get("arm_mutation_sha256"),
                                        "native_mutation": rows[0].get("native_mutation_sha256"),
                                        "files": [r["_file"] for r in rows]}
    # Across strata: one bank pair per assay, one arm and one native mutation block.
    for assay in expected:
        banks = {json.dumps(g["banks"], sort_keys=True) for (a, _), g in merged.items() if a == assay}
        if len(banks) > 1:
            problems.append(f"{arm}/{tag}: {assay} strata use different banks")
    for field in ("arm_mutation", "native_mutation"):
        if len({g[field] for g in merged.values()}) > 1:
            problems.append(f"{arm}/{tag}: strata disagree on {field}")
    return merged, problems


def expected_strata(manifest):
    return {a: {s: v["N"] for s in v["strata"]} for a, v in manifest["assays"].items()}


STRATA = [str(i) for i in range(16)]


def manifest_problems(manifest):
    """The manifest must name exactly food and wall, strata 0..15, an integer N
    in [20,000, 1,000,000] (each parent's twin-identity check then covers at
    least 10,000 seeds), passed fixtures, and consistent PC-J evidence."""
    problems = []
    assays = manifest.get("assays") or {}
    if set(assays) != {"food", "wall"}:
        problems.append(f"manifest assays {sorted(assays)} are not food and wall")
    for assay, spec in assays.items():
        if list(spec.get("strata") or []) != STRATA:
            problems.append(f"manifest {assay} strata are not 0..15")
        n = spec.get("N")
        if not isinstance(n, int) or not 20_000 <= n <= 1_000_000:
            problems.append(f"manifest {assay} N {n!r} is not an integer in [20,000, 1,000,000]")
    if not (manifest.get("fixtures") or {}).get("passed") is True:
        problems.append("manifest fixtures not recorded as passed")
    pcj_m = manifest.get("pcj")
    if not isinstance(pcj_m, dict) or pcj_m.get("status") not in ("constructed", "not constructible"):
        problems.append("manifest PC-J status missing or invalid")
    elif pcj_m["status"] == "not constructible":
        if any(k in pcj_m for k in ("power", "powered", "N", "pilot_b0", "pilot_c0")):
            problems.append("manifest PC-J not constructible but carries power fields")
    else:
        power = pcj_m.get("power")
        finite = isinstance(power, float) and math.isfinite(power) and 0.0 <= power <= 1.0
        if not finite or not isinstance(pcj_m.get("powered"), bool):
            problems.append("manifest PC-J constructed without a finite power in [0, 1] and a boolean powered")
        elif pcj_m["powered"] != (power >= 0.8):
            problems.append("manifest PC-J powered flag disagrees with its power")
        if pcj_m.get("N") != PCJ_N:
            problems.append("manifest PC-J N is not 1,000,000")
        if not (_nonneg_int(pcj_m.get("pilot_b0")) and _nonneg_int(pcj_m.get("pilot_c0"))):
            problems.append("manifest PC-J pilot counts missing")
    return problems


# ---------- modes ----------

def size(pilot_dir, manifest_out):
    summaries, _ = load(pilot_dir)
    pilot_n = {}
    for s in summaries:
        if s["arm"] == "j" and s["tag"] == "run5-pilot":
            pilot_n.setdefault(s["assay"], set()).add(str(s["stratum"]))
    expected = {a: {s: 20_000 for s in sorted(st, key=int)} for a, st in pilot_n.items()}
    merged, problems = coverage(summaries, "j", "run5-pilot", expected)
    if problems or set(expected) != {"food", "wall"} or any(len(v) != 16 for v in expected.values()):
        print(json.dumps({"problems": problems, "strata": {a: len(v) for a, v in expected.items()}}))
        sys.exit(2)
    manifest = {"assays": {}, "pilot": {}}
    for assay in ("food", "wall"):
        b = sum(merged[(assay, s)]["cells"].get("b", 0) for s in expected[assay])
        c = sum(merged[(assay, s)]["cells"].get("c", 0) for s in expected[assay])
        m0 = b + c
        n_star = min(1_000_000, max(20_000, math.ceil(20_000 * 150 / max(m0, 1))))
        exp_m = m0 * n_star / 20_000
        mm = max(1, round(exp_m))
        crit = next(k for k in range(mm + 2) if half_tail_ge(k, mm) <= ALPHA)
        power = sum(math.comb(mm, i) * (2 / 3) ** i * (1 / 3) ** (mm - i) for i in range(crit, mm + 1))
        manifest["assays"][assay] = {"strata": sorted(expected[assay], key=int), "N": n_star}
        manifest["pilot"][assay] = {"b0": b, "c0": c, "m0": m0, "expected_m": exp_m, "power_theta2": power}
    with open(manifest_out, "w") as f:
        json.dump(manifest, f, indent=1)
    print(json.dumps(manifest))


def pcj(pcj_dir, manifest_path):
    with open(manifest_path) as f:
        manifest = json.load(f)
    summaries, _ = load(pcj_dir)
    merged, problems = coverage(summaries, "j", "run5-pcj-pilot", {"wall": {"pcj": 100_000}})
    if problems:
        print(json.dumps({"problems": problems}))
        sys.exit(2)
    g = merged[("wall", "pcj")]
    b0, c0 = g["cells"].get("b", 0), g["cells"].get("c", 0)
    scale = PCJ_N / g["n"]
    power = sign_power_poisson(b0 * scale, c0 * scale)
    manifest.setdefault("pcj", {}).update({"status": "constructed", "N": PCJ_N, "pilot_b0": b0,
                                           "pilot_c0": c0, "power": power, "powered": power >= 0.8})
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=1)
    print(json.dumps(manifest["pcj"]))


def main(manifest_path, main_dir, aa_dir, rerun_dir, aacheck_dir, pcj_dir=None):
    with open(manifest_path) as f:
        manifest = json.load(f)
    expected = expected_strata(manifest)
    summaries, children = load(main_dir)
    merged, problems = coverage(summaries, "j", "run5", expected)
    aa_s, aa_c = load(aa_dir)
    aa_m, aa_cells, aa_problems = aa_from_records(summaries, children, aa_s, aa_c, expected)
    m_problems = manifest_problems(manifest)
    validity = {"manifest_problems": m_problems, "main_problems": problems, "aa_problems": aa_problems}
    # Same parents and banks in the A/A run; its arm side is the native engine.
    for key, g in merged.items():
        if key in aa_m and (aa_m[key]["parent"] != g["parent"] or aa_m[key]["banks"] != g["banks"]):
            aa_problems.append(f"aa: {key} parent or banks differ from the main draw")
    natives = {g["native_mutation"] for g in merged.values()}
    for key, g in aa_m.items():
        if g["native_mutation"] not in natives or g["arm_mutation"] != g["native_mutation"]:
            aa_problems.append(f"aa: {key} mutation blocks are not the main draw's native block")
    validity["fixtures_passed"] = not any("fixtures" in p for p in m_problems)
    validity["twin_identity_violations"] = sum(g["violations"] for g in merged.values())
    fired = sum(g["j"].get("fired_jumps", 0) for g in merged.values())
    events = sum(g["j"].get("applied_weight_events", 0) for g in merged.values())
    validity["fire_rate"] = {"fired": fired, "events": events, "ok": fire_rate_ok(fired, events)}
    ab = sum(v["b"] for v in aa_cells.values())
    ac = sum(v["c"] for v in aa_cells.values())
    equivalence = aacheck_ok(aacheck_dir)
    validity["aa"] = {"b": ab, "c": ac, "p_two_sided": sign_p_two_sided(ab, ac),
                      "record_equivalence": equivalence,
                      "ok": not aa_problems and equivalence and sign_p_two_sided(ab, ac) >= 0.01}
    # Founder rerun: both assays' stratum 0 rerun over all of 0..N, file for file
    # byte-identical to the main draw's files for that stratum.
    r_summ, _ = load(rerun_dir)
    founders = {a_: {"0": expected[a_]["0"]} for a_ in ("food", "wall") if a_ in expected and "0" in expected[a_]}
    r_m, r_problems = coverage(r_summ, "j", "run5", founders)
    rerun_ok = not r_problems and len(founders) == 2
    for key, g in r_m.items():
        main_files = sorted(merged.get(key, {}).get("files", []))
        if sorted(g["files"]) != main_files:
            rerun_ok = False
            r_problems.append(f"rerun: {key} files differ from the main draw's ranges")
            continue
        for name in main_files:
            with open(os.path.join(rerun_dir, name), "rb") as f1, open(os.path.join(main_dir, name), "rb") as f2:
                if f1.read() != f2.read():
                    rerun_ok = False
                    r_problems.append(f"rerun: {name} differs")
    validity["founder_rerun"] = {"ok": rerun_ok, "problems": r_problems}
    valid = (not m_problems and not problems and validity["aa"]["ok"] and validity["fixtures_passed"]
             and validity["twin_identity_violations"] == 0 and validity["fire_rate"]["ok"] and rerun_ok)
    validity["valid"] = valid

    per = defaultdict(lambda: {"b": 0, "c": 0, "n": 0, "strata": {}})
    for (assay, stratum), g in merged.items():
        sb, sc = g["cells"].get("b", 0), g["cells"].get("c", 0)
        per[assay]["b"] += sb
        per[assay]["c"] += sc
        per[assay]["n"] += g["n"]
        per[assay]["strata"][stratum] = (sb, sc)
    b = sum(o["b"] for o in per.values())
    c = sum(o["c"] for o in per.values())
    p = sign_p(b, c)
    worst = None
    for assay, o in sorted(per.items()):
        for s, (sb, sc) in sorted(o["strata"].items(), key=lambda kv: int(kv[0])):
            if worst is None or sb - sc > worst["excess"]:
                worst = {"assay": assay, "stratum": s, "excess": sb - sc, "b": sb, "c": sc}
    p_drop = sign_p(b - worst["b"], c - worst["c"]) if worst else 1.0
    rules = {"p": p, "p_ok": p <= ALPHA,
             "ratio": (b / c) if c else (None if b == 0 else "inf"),
             "ratio_ok": (c == 0 and b > 0) or (c > 0 and b / c >= 1.5),
             "no_reversal": all(o["b"] + o["c"] < 10 or o["b"] >= o["c"] for o in per.values()),
             "drop_largest": {**(worst or {}), "p": p_drop, "ok": p_drop <= 0.05}}
    passed = valid and rules["p_ok"] and rules["ratio_ok"] and rules["no_reversal"] and rules["drop_largest"]["ok"]

    neg = {}
    for assay in ("food", "wall"):
        o = per.get(assay, {"b": 0, "c": 0, "n": 0})
        bd = bounds(o["b"], o["c"], o["n"]) if o["n"] else {"theta_u": math.inf, "delta_u": math.inf}
        neg[assay] = {"b": o["b"], "c": o["c"], "n": o["n"], **bd,
                      "ok": o["n"] > 0 and (bd["theta_u"] < 2 or bd["delta_u"] < 2.5e-4)}

    pcj_m = manifest.get("pcj") or {}
    pcj_gate = {"status": pcj_m.get("status"), "powered": pcj_m.get("powered") is True}
    if pcj_m.get("status") == "constructed" and pcj_m.get("powered") is True:
        ps, _ = load(pcj_dir)
        pm, pp = coverage(ps, "j", "run5", {"wall": {"pcj": PCJ_N}})
        g = pm.get(("wall", "pcj"))
        gb = g["cells"].get("b", 0) if g else 0
        gc = g["cells"].get("c", 0) if g else 0
        if g is not None and g["violations"]:
            pp.append(f"pcj: {g['violations']} twin-identity violations")
        pcj_gate.update({"problems": pp, "b": gb, "c": gc, "p": sign_p(gb, gc),
                         "holds": not pp and g is not None and sign_p(gb, gc) <= ALPHA})
        pcj_gate["gate_ok"] = pcj_gate["holds"]
    else:
        pcj_gate["gate_ok"] = True  # not constructible or underpowered: reported, not gating

    if not valid:
        verdict = "inconclusive"
        reason = "void screen"
    elif passed:
        verdict, reason = "pass", "rule 7 pass"
    elif all(neg[a]["ok"] for a in ("food", "wall")) and pcj_gate["gate_ok"]:
        verdict, reason = "supported negative", "rule 7 negative"
    else:
        verdict, reason = "inconclusive", "neither pass nor supported negative"

    tail = defaultdict(float)
    counted = [ch for ch in children if ch.get("arm") == "j" and ch.get("tag") == "run5"
               and ch.get("label") == "confirmed" and not (ch.get("sterility") or {}).get("attributable")]
    for ch in sorted(counted, key=lambda r: (r["assay"], int(r["stratum"]) if str(r["stratum"]).isdigit() else -1,
                                             r["index"], r["side"])):
        tail[(ch["assay"], ch["side"])] += ch["delta_b"]
    out = {"verdict": verdict, "reason": reason, "validity": validity,
           "per_assay": {a: {"b": o["b"], "c": o["c"], "n": o["n"]} for a, o in per.items()},
           "pooled": {"b": b, "c": c}, "pass_rules": rules, "negative": neg, "pcj": pcj_gate,
           "upper_tail_sum_delta_b": {f"{a}/{s}": v for (a, s), v in sorted(tail.items())}}
    print(json.dumps(out, indent=1, default=str))
    return out


def aacheck(probe_aa_dir, main_dir, aa_dir, assay, stratum, end):
    """Equivalence check before the record-based A/A is used: the probe's own
    `aa` cells (arm aa, tag run5, range 0..end) must equal the record-based
    cells restricted to child indices below `end`."""
    end = int(end)
    problems = []
    ps, pc = load(probe_aa_dir)
    rows = [s for s in ps if s.get("arm") == "aa" and s.get("tag") == "run5"
            and s.get("assay") == assay and str(s.get("stratum")) == stratum]
    if len(rows) != 1 or rows[0]["range"] != [0, end]:
        problems.append("probe aa sample missing or not 0..end")
        probe, p_native, p_arm = {"b": None, "c": None}, set(), set()
    else:
        probe = rows[0]["twin"]["cells"]
        sel = lambda sets: {k[2] for ks in sets.values() for k in ks if k[0] == assay and k[1] == stratum}
        p_native = sel(counted(pc, "aa", "run5", "native", ps, problems))
        p_arm = sel(counted(pc, "aa", "run5", "arm", ps, problems))
    main_s, main_c = load(main_dir)
    aa_s, aa_c = load(aa_dir)
    n_set = {k[2] for ks in counted(main_c, "j", "run5", "native", main_s, problems).values() for k in ks
             if k[0] == assay and k[1] == stratum and k[2] < end}
    a_set = {k[2] for ks in counted(aa_c, "native", "run5-aa", "native", aa_s, problems).values() for k in ks
             if k[0] == assay and k[1] == stratum and k[2] < end}
    rec = {"b": len(a_set - n_set), "c": len(n_set - a_set)}
    # Record-level identity, not only counts: the probe's run5 side must be the
    # main draw's native records and its run5-aa side the A/A run's, and the
    # sample must not be vacuous.
    match = (not problems and p_native == n_set and p_arm == a_set and len(n_set) > 0 and len(a_set) > 0
             and rec["b"] == probe["b"] and rec["c"] == probe["c"])
    out = {"assay": assay, "stratum": stratum, "end": end, "probe": {"b": probe["b"], "c": probe["c"]},
           "records": rec, "run5_records": len(n_set), "run5_aa_records": len(a_set),
           "sets_equal": p_native == n_set and p_arm == a_set, "problems": problems, "match": match}
    with open(os.path.join(probe_aa_dir, f"aacheck-{assay}-{stratum}.json"), "w") as f:
        json.dump(out, f)
    print(json.dumps(out))
    if not match:
        sys.exit(3)
    return out


def aacheck_ok(probe_aa_dir):
    """Both predeclared equivalence samples passed (rule 6, ledger row V-A)."""
    # Wall 0 replaces the predeclared wall 7, whose sample was vacuous (ledger row V-A2).
    for assay, stratum in (("food", "0"), ("wall", "0")):
        path = os.path.join(probe_aa_dir or "", f"aacheck-{assay}-{stratum}.json")
        if not os.path.exists(path):
            return False
        with open(path) as f:
            if json.load(f).get("match") is not True:
                return False
    return True


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "aacheck":
        aacheck(*sys.argv[2:8])
    if mode == "size":
        size(sys.argv[2], sys.argv[3])
    elif mode == "pcj":
        pcj(sys.argv[2], sys.argv[3])
    elif mode == "main":
        main(*sys.argv[2:8])
