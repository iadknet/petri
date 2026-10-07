"""Gate G3: read the E4 probe's raw off and on outputs on the N2 binary and
apply the predeclared equivalence rules (follow-up note, Gates, amendment
A2):
 off: founder+declared and founder+declared+zero-edge lost_declaration CP
      intervals contain E4's point estimates (0.0186426, 0.0079606) and the
      points are within 5 %;
 on:  decay.firing CP interval contains 0.005; the connected parent's
      lost_declaration CP interval contains the off point and is within
      10 %; the founder's attempted_events_per_birth within 5 % of off and
      its applied events per birth excluding InputRefPrune within 5 % of
      off; InputRefPrune applied 0 on every parent.

usage: python3 g3read.py <e4-off.json> <e4-on.json> <out.json>"""
import hashlib
import json
import sys

E4 = {"founder+declared": 0.0186426, "founder+declared+zero-edge": 0.0079606}
RATE = 0.005


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def parents(path):
    d = json.load(open(path))
    return d, {p["parent"]: p for p in d["parents"]}


def within(a, b, rel):
    return b != 0 and abs(a - b) / abs(b) <= rel


def applied_without_prune(p):
    ops = p["applied_by_operator"]
    total = sum(ops.values()) - ops.get("InputRefPrune", 0)
    return total / p["births"]


def main():
    off_path, on_path, out = sys.argv[1:4]
    off_doc, off = parents(off_path)
    on_doc, on = parents(on_path)
    checks = []

    def check(name, ok, detail):
        checks.append({"check": name, "pass": bool(ok), **detail})

    for name, e4 in E4.items():
        r = off[name]["lost_declaration"]
        lo, hi = r["cp95"]
        check(f"off {name} lost_declaration CP contains E4", lo <= e4 <= hi, {"rate": r["rate"], "cp95": r["cp95"], "e4": e4})
        check(f"off {name} lost_declaration within 5 % of E4", within(r["rate"], e4, 0.05), {"rate": r["rate"], "e4": e4})
    for name in E4:
        d = on[name]["decay"]
        check(f"on {name} decay eligible > 0", d["eligible"] > 0, {"eligible": d["eligible"], "applied": d["applied"]})
    fire = on["founder+declared"]["decay"]["firing"]
    check("on founder+declared decay firing CP contains 0.005",
          fire is not None and fire["cp95"][0] <= RATE <= fire["cp95"][1], {"firing": fire})
    c_on = on["founder+declared+zero-edge"]["lost_declaration"]
    c_off = off["founder+declared+zero-edge"]["lost_declaration"]
    check("on connected lost_declaration CP contains off point", c_on["cp95"][0] <= c_off["rate"] <= c_on["cp95"][1],
          {"on": c_on, "off_rate": c_off["rate"]})
    check("on connected lost_declaration within 10 % of off", within(c_on["rate"], c_off["rate"], 0.10),
          {"on_rate": c_on["rate"], "off_rate": c_off["rate"]})
    a_on, a_off = on["founder"]["attempted_events_per_birth"], off["founder"]["attempted_events_per_birth"]
    check("on founder attempted_events_per_birth within 5 % of off", within(a_on, a_off, 0.05), {"on": a_on, "off": a_off})
    w_on, w_off = applied_without_prune(on["founder"]), applied_without_prune(off["founder"])
    check("on founder applied events excluding InputRefPrune within 5 % of off", within(w_on, w_off, 0.05),
          {"on": w_on, "off": w_off, "raw_applied_on": on["founder"]["applied_events_per_birth"],
           "raw_applied_off": off["founder"]["applied_events_per_birth"]})
    prune_on = sum(p["applied_by_operator"].get("InputRefPrune", 0) for p in on.values())
    check("on InputRefPrune applied 0 (ordinary loop)", prune_on == 0, {"prune_applied": prune_on})
    result = {
        "gate": "G3",
        "off": {"path": off_path, "sha256": sha(off_path), "config": off_doc["config"]},
        "on": {"path": on_path, "sha256": sha(on_path), "config": on_doc["config"]},
        "checks": checks,
        "pass": all(c["pass"] for c in checks),
        "on_integrated_loss_founder_declared": on["founder+declared"]["lost_declaration"],
        "useful_connect_founder_declared": {
            "off": off["founder+declared"]["connect_useful_signed"],
            "on": on["founder+declared"]["connect_useful_signed"],
            "e4_rate_10e7": 5.5e-6,
        },
        "lose_to_useful_connect_ratio": {
            arm: {
                "point": p["lost_declaration"]["rate"] / p["connect_useful_signed"]["rate"] if p["connect_useful_signed"]["rate"] else None,
                "bounds": [p["lost_declaration"]["cp95"][0] / p["connect_useful_signed"]["cp95"][1] if p["connect_useful_signed"]["cp95"][1] else None,
                           p["lost_declaration"]["cp95"][1] / p["connect_useful_signed"]["cp95"][0] if p["connect_useful_signed"]["cp95"][0] else None],
                "on_e4_denominator": p["lost_declaration"]["rate"] / 5.5e-6,
            }
            for arm, p in (("off", off["founder+declared"]), ("on", on["founder+declared"]))
        },
        "decay_route_note": "decay.by_operator is the whole-child decay-route mix over every silent entry visited (the watched declaration, the founder's own ring entry and entries created within the birth); the summaries do not attribute routes to the watched entry",
        "on_integrated_loss_founder_declared_by_ops": on["founder+declared"]["lost_declaration_by_ops"],
        "on_decay_founder_declared": on["founder+declared"]["decay"],
        "on_decay_founder": on["founder"]["decay"],
        "off_lost_by_ops_founder_declared": off["founder+declared"]["lost_declaration_by_ops"],
        "off_applied_by_operator_founder": off["founder"]["applied_by_operator"],
        "on_applied_by_operator_founder": on["founder"]["applied_by_operator"],
        "on_skip_note": "SilentEntryDecaysOnly skips are not in the probe output; they are the gap between attempted and applied",
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("G3", "pass" if result["pass"] else "FAIL")


if __name__ == "__main__":
    main()
