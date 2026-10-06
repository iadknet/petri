"""Diagnosis run row E3: three readings kept apart.
(i) analytic energy cost of the declared reference (1 genome unit) and of
    the zero-weight edge (1 more unit) at the carry cost per unit per tick,
    over E2's lifetime proxy (mean living age at tick 20,000), as a fraction
    of the per-birth energy (transfer fraction x mean living energy);
(ii) the mutational loss hazard per birth of each state from E4
    (`lose_declaration` on parent (b), the authored edge's fates on (c));
(iii) run 5's twin screen on the three prepared parents (strata 1 to 3 of the
    E3 freeze; `native` arm), both assays: shares of drawn children that are
    identical, neutral, harmful, helpful on bank A, confirmed on bank B,
    sterility-attributable, with 95 % CP bounds over drawn and over
    evaluated (non-identical) children.

usage: python3 e3read.py <e2-canyon.json> <e2-confluence.json> <e4.json> <e3-dir> <transfer_fraction> <out.json>"""
import hashlib
import json
import math
import os
import sys

CARRY = 1e-4  # energy per genome unit per tick (production default)
STRATA = {"1": "founder+declared", "2": "founder+declared+zero-edge", "3": "founder+declared+wrong-sign-edge"}


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def cp95(k, n):
    """Exact Clopper-Pearson via the regularized incomplete beta (bisection)."""
    if n == 0:
        return (0.0, 1.0)

    def betacdf(x, a, b):
        # continued fraction (Numerical Recipes betai)
        if x <= 0:
            return 0.0
        if x >= 1:
            return 1.0
        lbeta = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * math.log(1 - x)
        front = math.exp(lbeta)

        def cf(x, a, b):
            MAXIT, EPS, FPMIN = 300, 3e-14, 1e-300
            qab, qap, qam = a + b, a + 1, a - 1
            c, d = 1.0, 1.0 - qab * x / qap
            d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
            h = d
            for m in range(1, MAXIT + 1):
                m2 = 2 * m
                aa = m * (b - m) * x / ((qam + m2) * (a + m2))
                d = 1.0 + aa * d
                d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
                c = 1.0 + aa / (c if abs(c) > FPMIN else FPMIN)
                h *= d * c
                aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
                d = 1.0 + aa * d
                d = 1.0 / (d if abs(d) > FPMIN else FPMIN)
                c = 1.0 + aa / (c if abs(c) > FPMIN else FPMIN)
                de = d * c
                h *= de
                if abs(de - 1.0) < EPS:
                    break
            return h
        if x < (a + 1) / (a + b + 2):
            return front * cf(x, a, b) / a
        return 1.0 - front * cf(1 - x, b, a) / b

    def solve(target_cdf, a, b):
        lo, hi = 0.0, 1.0
        for _ in range(100):
            mid = (lo + hi) / 2
            if betacdf(mid, a, b) < target_cdf:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2
    lower = 0.0 if k == 0 else solve(0.025, k, n - k + 1)
    upper = 1.0 if k == n else solve(0.975, k + 1, n - k)
    return (lower, upper)


def rate(k, n):
    lo, hi = cp95(k, n)
    return {"count": k, "of": n, "share": (k / n) if n else None, "cp95": [lo, hi]}


def main():
    e2c, e2f, e4p, e3dir, transfer, out_path = sys.argv[1:7]
    transfer = float(transfer)
    e4 = json.load(open(e4p))
    # (i) analytic
    analytic = {}
    for path in (e2c, e2f):
        e2 = json.load(open(path))
        cp = next((c for c in e2["checkpoints"] if c["tick"] == 20000), None)
        if cp is None:
            analytic[e2["world"]] = None
            continue
        age, energy = cp["sample"]["mean_age"], cp["sample"]["mean_energy"]
        per_birth = transfer * energy
        analytic[e2["world"]] = {
            "mean_living_age_20k": age, "mean_living_energy_20k": energy, "transfer_fraction": transfer,
            "per_birth_energy": per_birth,
            "declared_reference_units": 1, "zero_edge_units": 1,
            "declared_cost_over_lifetime": CARRY * 1 * age,
            "zero_edge_total_cost_over_lifetime": CARRY * 2 * age,
            "declared_cost_share_of_birth": (CARRY * age / per_birth) if per_birth else None,
            "zero_edge_cost_share_of_birth": (CARRY * 2 * age / per_birth) if per_birth else None,
            "energy_neutral_within_1e-3": (CARRY * 2 * age / per_birth) < 1e-3 if per_birth else None,
        }
    # (ii) hazards from E4
    hazards = {}
    for p in e4["parents"]:
        if p["parent"] in ("founder+declared", "founder+declared+zero-edge"):
            hazards[p["parent"]] = {
                "lose_declaration_per_birth": p["rates_all_births"]["lose_declaration"],
                "lose_declaration_by_class": p["lose_declaration_by_class"],
                "authored_edge": p.get("authored_edge"),
            }
    # (iii) screen
    screen = {}
    for a in ("food", "wall"):
        for s, name in STRATA.items():
            path = os.path.join(e3dir, "screen", f"{a}-s{s}.ndjson")
            if not os.path.exists(path):
                screen[f"{a}-s{s}"] = None
                continue
            summary = None
            for line in open(path):
                row = json.loads(line)
                if row.get("probe") == "r5-screen":
                    summary = row
            if summary is None:
                screen[f"{a}-s{s}"] = None
                continue
            nat = summary["native"]
            drawn, ev = nat["drawn"], nat["evaluated"]
            screen[f"{a}-s{s}"] = {
                "parent": name, "parent_sha256": summary["parent_sha256"], "source": summary["source"],
                "parent_bank_a": summary["parent_bank_a"], "parent_bank_b": summary["parent_bank_b"],
                "sterility_gain": summary["sterility_gain"], "pairs": summary["pairs"],
                "drawn": drawn, "identical": nat["identical"], "evaluated": ev,
                "over_drawn": {k: rate(nat[k], drawn) for k in ("neutral", "harmful", "helpful_a", "confirmed", "sterility_attributable")},
                "over_evaluated": {k: rate(nat[k], ev) for k in ("neutral", "harmful", "helpful_a", "confirmed", "sterility_attributable")},
                "by_operator": nat["by_operator"], "file_sha256": sha(path),
            }
    out = {"row": "E3", "analytic": analytic, "hazards": hazards, "screen": screen,
           "note": "the screen's shares are properties of each prepared parent's one-step neighborhood, not a selection coefficient of carrying the state",
           "reader_sha256": sha(__file__)}
    json.dump(out, open(out_path, "w"), indent=1)
    print(json.dumps(analytic, indent=1))
    for k, v in screen.items():
        if v:
            print(k, v["parent"], "drawn", v["drawn"], "identical", v["identical"], "evaluated", v["evaluated"],
                  {kk: round(vv["share"], 4) for kk, vv in v["over_evaluated"].items() if vv["share"] is not None})


if __name__ == "__main__":
    main()
