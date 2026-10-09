"""Gate B0: read the E4 probe's off, on and on-10^7 outputs on the
prototype binary and apply the predeclared rules (born-connected note,
Gates, after amendment A1):
 off: per parent lost_declaration (b, c), declared_vote_node_target,
      attempted_events_per_birth and InputRefAdd applied per birth within
      5 % of the follow-up's G3 off output; connect_useful_signed counts
      reported with CP intervals; the deterministic projection (every field
      but wall_secs and the config metadata) compared with G3's raw off
      output, genome_size / executed_resolved / frozen_age first.
 on:  (1) parent (a): single_add connect_any_given_declared exactly 1, every
      single_add destination a vote sink, every sub-value in 0..6; all-birth
      connect_any CP contains the declared point.
      (2) the 10^7 run: single_add connect_useful_signed_given_declared CP
      contains 2/189; all-birth connect_useful_signed CP against 2/189 x
      declared (compatibility, reported); the 10^6 interval reported.
      (3) attempted_events_per_birth within 5 % of off on every parent.
      (4) InputRefAdd applied per birth within 5 % of off on every parent.
      (5) lost_declaration (b, c) within 5 % of off, routes beside.

usage: python3 b0read.py <g3 e4-off.json (reference)> <e4-off.json> <e4-on.json> <e4-on-10m.json> <out.json>"""
import hashlib
import json
import sys

USEFUL = 2.0 / 189.0


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def parents(path):
    d = json.load(open(path))
    return d, {p["parent"]: p for p in d["parents"]}


def within(a, b, rel):
    return b != 0 and abs(a - b) / abs(b) <= rel


def add_rate(p):
    return p["applied_by_operator"].get("InputRefAdd", 0) / p["births"]


def projection(p):
    """Every field but wall_secs and the probe-version blocks (the G3
    reference carries the N2 `decay` block, this probe `single_add_births`)."""
    q = json.loads(json.dumps(p))
    for k in ("wall_secs", "decay", "single_add_births"):
        q.pop(k, None)
    return q


def main():
    ref_path, off_path, on_path, on10_path, out = sys.argv[1:6]
    ref_doc, ref = parents(ref_path)
    off_doc, off = parents(off_path)
    on_doc, on = parents(on_path)
    on10_doc, on10 = parents(on10_path)
    checks = []

    def check(name, ok, detail=None, gating=True):
        checks.append({"check": name, "pass": bool(ok), "gating": gating, "detail": detail})

    names = ["founder", "founder+declared", "founder+declared+zero-edge"]
    # Off path. The identity readings are recorded, not gating (the note:
    # "expected ... and recorded if found"); the 5 % rules are the gate.
    for n in names:
        r, o = ref[n], off[n]
        check(f"off {n} genome_size/executed_resolved/frozen_age equal G3 off (recorded)",
              (r["genome_size"], r["executed_resolved"], r["frozen_age"]) == (o["genome_size"], o["executed_resolved"], o["frozen_age"]),
              {"ref": [r["genome_size"], r["executed_resolved"], r["frozen_age"]], "off": [o["genome_size"], o["executed_resolved"], o["frozen_age"]]},
              gating=False)
        check(f"off {n} deterministic projection equals G3 off (recorded)", projection(r) == projection(o),
              {"differing_keys": [k for k in projection(r) if projection(r)[k] != projection(o).get(k)],
               "keys_only_in_ref": sorted(set(projection(r)) - set(projection(o))),
               "keys_only_in_off": sorted(set(projection(o)) - set(projection(r)))},
              gating=False)
        if r["lost_declaration"] is not None:
            check(f"off {n} lost_declaration within 5 %", within(o["lost_declaration"]["rate"], r["lost_declaration"]["rate"], 0.05),
                  {"off": o["lost_declaration"]["rate"], "ref": r["lost_declaration"]["rate"]})
        check(f"off {n} declared_vote_node_target within 5 %", within(o["declared_vote_node_target"]["rate"], r["declared_vote_node_target"]["rate"], 0.05),
              {"off": o["declared_vote_node_target"]["rate"], "ref": r["declared_vote_node_target"]["rate"]})
        check(f"off {n} attempted_events_per_birth within 5 %", within(o["attempted_events_per_birth"], r["attempted_events_per_birth"], 0.05),
              {"off": o["attempted_events_per_birth"], "ref": r["attempted_events_per_birth"]})
        check(f"off {n} InputRefAdd applied per birth within 5 %", within(add_rate(o), add_rate(r), 0.05), {"off": add_rate(o), "ref": add_rate(r)})
    useful_off = {n: {"off": off[n]["connect_useful_signed"], "ref": ref[n]["connect_useful_signed"]} for n in names}
    # On path.
    a = on["founder"]
    sa = a["single_add_births"]
    check("on (1) founder single-Add connect_any_given_declared exactly 1",
          sa["declared_vote_node_target"]["count"] > 0 and sa["connect_any_given_declared"]["count"] == sa["declared_vote_node_target"]["count"],
          {"declared": sa["declared_vote_node_target"], "connect_any": sa["connect_any_given_declared"]})
    locs = list(sa["connect_by_location"].keys())
    check("on (1) founder single-Add destinations all ActionVote sinks", locs and all(k.startswith("sink:ActionVote(") for k in locs), {"locations": sa["connect_by_location"]})
    subs = [int(k) for k in sa["connect_by_sub"].keys()]
    check("on (1) founder single-Add sub-values in 0..6", subs and all(0 <= s <= 6 for s in subs), {"subs": sa["connect_by_sub"]})
    ca, da = a["connect_any"], a["declared_vote_node_target"]
    check("on (1) founder all-birth connect_any CP contains declared point", ca["cp95"][0] <= da["rate"] <= ca["cp95"][1],
          {"connect_any": ca, "declared": da})
    a10 = on10["founder"]
    sa10 = a10["single_add_births"]
    u10 = sa10["connect_useful_signed_given_declared"]
    check("on (2) 10^7 founder single-Add useful-signed share CP contains 2/189", u10["cp95"][0] <= USEFUL <= u10["cp95"][1],
          {"share": u10, "predicted": USEFUL})
    all10 = a10["connect_useful_signed"]
    pred10 = USEFUL * a10["declared_vote_node_target"]["rate"]
    check("on (2) 10^7 founder all-birth useful-signed CP contains 2/189 x declared (compatibility)", all10["cp95"][0] <= pred10 <= all10["cp95"][1],
          {"all_birth": all10, "predicted": pred10, "declared": a10["declared_vote_node_target"]})
    useful_on_1m = {"all_birth": a["connect_useful_signed"], "predicted": USEFUL * da["rate"], "single_add_share": sa["connect_useful_signed_given_declared"]}
    for n in names:
        check(f"on (3) {n} attempted_events_per_birth within 5 % of off", within(on[n]["attempted_events_per_birth"], off[n]["attempted_events_per_birth"], 0.05),
              {"on": on[n]["attempted_events_per_birth"], "off": off[n]["attempted_events_per_birth"]})
        check(f"on (4) {n} InputRefAdd applied per birth within 5 % of off", within(add_rate(on[n]), add_rate(off[n]), 0.05),
              {"on": add_rate(on[n]), "off": add_rate(off[n])})
    for n in names[1:]:
        check(f"on (5) {n} integrated lost_declaration within 5 % of off", within(on[n]["lost_declaration"]["rate"], off[n]["lost_declaration"]["rate"], 0.05),
              {"on": on[n]["lost_declaration"], "off": off[n]["lost_declaration"], "routes_on": on[n]["lost_declaration_by_ops"], "routes_off": off[n]["lost_declaration_by_ops"]})
    result = {
        "gate": "B0",
        "reference": {"path": ref_path, "sha256": sha(ref_path), "config": ref_doc["config"]},
        "off": {"path": off_path, "sha256": sha(off_path), "config": off_doc["config"]},
        "on": {"path": on_path, "sha256": sha(on_path), "config": on_doc["config"]},
        "on_10m": {"path": on10_path, "sha256": sha(on10_path), "config": on10_doc["config"]},
        "checks": checks,
        "pass": all(c["pass"] for c in checks if c["gating"]),
        "recorded_identity_checks_pass": all(c["pass"] for c in checks if not c["gating"]),
        "useful_signed_off_vs_reference": useful_off,
        "useful_signed_on_1m_founder": useful_on_1m,
        "on_founder_single_add": sa,
        "on_10m_founder_single_add": sa10,
        "on_10m_founder_all_birth": {k: a10[k] for k in ("declared_vote_node_target", "connect_any", "connect_cardinal_move", "connect_useful_component_move",
                                                           "connect_useful_signed", "connect_useful_wrong_sign", "attempted_events_per_birth", "applied_events_per_birth", "births", "wall_secs")},
        "on_pooled_destinations_descriptive": {n: on[n]["occurrences"]["connect_by_location"] for n in names},
        "wall_secs": {"off": {n: off[n]["wall_secs"] for n in names}, "on": {n: on[n]["wall_secs"] for n in names}, "on_10m": {n: on10[n]["wall_secs"] for n in names}},
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", "" if c["gating"] else "(recorded)", c["check"])
    print("B0", "pass" if result["pass"] else "FAIL", "| identity recorded:", "equal" if result["recorded_identity_checks_pass"] else "differs")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
