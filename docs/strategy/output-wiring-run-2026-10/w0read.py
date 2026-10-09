"""Gate W0: read the E4 probe's outputs (frozen probe; prototype probe at
votes; prototype probe at non_clear at 10^6 and 10^7 births) and apply the
predeclared rules (output-wiring note, Gates):
 votes: the prototype probe's votes output equals the frozen probe's in the
        deterministic projection (every field but wall_secs and the config
        metadata) on all three parents, genome_size / executed_resolved /
        frozen_age compared first (gating: default is T20.F16's engine).
 non_clear: (1) parent (a): single_add connect_any_given_declared exactly 1,
        every single_add destination a sink: key that is not a ClearSlot and
        not a compute: key, every sub-value in 0..6; all-birth connect_any CP
        contains the declared point. (2) the 10^7 run: all 78 non-clear sink
        keys hit; the ActionVote share among single_add destinations has a
        95 % CP interval containing 27/78; per-kind shares and the
        useful-signed share against 2/189 x 27/78 reported. (3) attempted
        events per birth equal to the votes path's on every parent. (4)
        InputRefAdd applied per birth within 5 % of the votes path's. (5)
        lost_declaration (b, c) within 5 % of the votes path's, routes beside.

usage: python3 w0read.py <frozen e4.json> <votes e4.json> <wide e4.json> <wide-10m e4.json> <out.json>"""
import hashlib
import json
import math
import sys

VOTE_SHARE = 27.0 / 78.0
USEFUL_WIDE = (2.0 / 189.0) * VOTE_SHARE
KIND_SHARES = {"CustomOutput": 24 / 78, "RouterGate": 8 / 78, "WriteSlot": 16 / 78, "ActionVote": 27 / 78, "ActionParam": 3 / 78}


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
    q = json.loads(json.dumps(p))
    for k in ("wall_secs",):
        q.pop(k, None)
    return q


def binom_cdf(k, n, p):
    if p <= 0:
        return 1.0
    if p >= 1:
        return 1.0 if k >= n else 0.0
    total = 0.0
    lp, lq = math.log(p), math.log(1 - p)
    for i in range(0, k + 1):
        total += math.exp(math.lgamma(n + 1) - math.lgamma(i + 1) - math.lgamma(n - i + 1) + i * lp + (n - i) * lq)
    return min(total, 1.0)


def cp95(k, n):
    """Clopper-Pearson 95 % interval by bisection."""
    if n == 0:
        return [0.0, 1.0]
    lo, hi = 0.0, 1.0
    if k > 0:
        a, b = 0.0, k / n
        for _ in range(60):
            m = (a + b) / 2
            if 1 - binom_cdf(k - 1, n, m) < 0.025:
                a = m
            else:
                b = m
        lo = a
    if k < n:
        a, b = k / n, 1.0
        for _ in range(60):
            m = (a + b) / 2
            if binom_cdf(k, n, m) < 0.025:
                b = m
            else:
                a = m
        hi = b
    return [lo, hi]


def kind_of(key):
    """`sink:WriteSlot(3)` -> WriteSlot; `sink:ActionVote(Move(2))` -> ActionVote."""
    if not key.startswith("sink:"):
        return None
    return key[5:].split("(")[0]


def main():
    frozen_path, votes_path, wide_path, wide10_path, out = sys.argv[1:6]
    frozen_doc, frozen = parents(frozen_path)
    votes_doc, votes = parents(votes_path)
    wide_doc, wide = parents(wide_path)
    wide10_doc, wide10 = parents(wide10_path)
    checks = []

    def check(name, ok, detail=None, gating=True):
        checks.append({"check": name, "pass": bool(ok), "gating": gating, "detail": detail})

    names = ["founder", "founder+declared", "founder+declared+zero-edge"]
    for n in names:
        f, v = frozen[n], votes[n]
        check(f"votes {n} genome_size/executed_resolved/frozen_age equal frozen",
              (f["genome_size"], f["executed_resolved"], f["frozen_age"]) == (v["genome_size"], v["executed_resolved"], v["frozen_age"]),
              {"frozen": [f["genome_size"], f["executed_resolved"], f["frozen_age"]], "votes": [v["genome_size"], v["executed_resolved"], v["frozen_age"]]})
        check(f"votes {n} deterministic projection equals frozen", projection(f) == projection(v),
              {"differing_keys": [k for k in projection(f) if projection(f)[k] != projection(v).get(k)],
               "keys_only_in_frozen": sorted(set(projection(f)) - set(projection(v))),
               "keys_only_in_votes": sorted(set(projection(v)) - set(projection(f)))})
    check("votes config metadata equals frozen config metadata", frozen_doc["config"] == votes_doc["config"],
          {"frozen": frozen_doc["config"], "votes": votes_doc["config"]})
    a = wide["founder"]
    sa = a["single_add_births"]
    check("non_clear (1) founder single-Add connect_any_given_declared exactly 1",
          sa["declared_vote_node_target"]["count"] > 0 and sa["connect_any_given_declared"]["count"] == sa["declared_vote_node_target"]["count"],
          {"declared": sa["declared_vote_node_target"], "connect_any": sa["connect_any_given_declared"]})
    locs = list(sa["connect_by_location"].keys())
    check("non_clear (1) founder single-Add destinations all non-clear sinks",
          locs and all(k.startswith("sink:") and not k.startswith("sink:ClearSlot(") for k in locs), {"locations": sa["connect_by_location"]})
    subs = [int(k) for k in sa["connect_by_sub"].keys()]
    check("non_clear (1) founder single-Add sub-values in 0..6", subs and all(0 <= s <= 6 for s in subs), {"subs": sa["connect_by_sub"]})
    ca, da = a["connect_any"], a["declared_vote_node_target"]
    check("non_clear (1) founder all-birth connect_any CP contains declared point", ca["cp95"][0] <= da["rate"] <= ca["cp95"][1],
          {"connect_any": ca, "declared": da})
    a10 = wide10["founder"]
    sa10 = a10["single_add_births"]
    locs10 = sa10["connect_by_location"]
    total10 = sum(locs10.values())
    by_kind = {}
    for k, c in locs10.items():
        by_kind[kind_of(k)] = by_kind.get(kind_of(k), 0) + c
    distinct = len([k for k in locs10 if k.startswith("sink:")])
    check("non_clear (2) 10^7 founder single-Add destinations hit all 78 non-clear sinks",
          distinct == 78 and all(not k.startswith("sink:ClearSlot(") and k.startswith("sink:") for k in locs10),
          {"distinct_sinks": distinct, "total": total10, "clear_or_compute_keys": [k for k in locs10 if k.startswith("sink:ClearSlot(") or not k.startswith("sink:")]})
    votes10 = by_kind.get("ActionVote", 0)
    vote_cp = cp95(votes10, total10)
    check("non_clear (2) 10^7 founder single-Add ActionVote share CP contains 27/78", vote_cp[0] <= VOTE_SHARE <= vote_cp[1],
          {"votes": votes10, "total": total10, "share": votes10 / total10 if total10 else None, "cp95": vote_cp, "predicted": VOTE_SHARE})
    kind_shares = {k: {"count": by_kind.get(k, 0), "share": by_kind.get(k, 0) / total10 if total10 else None,
                       "cp95": cp95(by_kind.get(k, 0), total10), "predicted": KIND_SHARES[k],
                       "cp_contains": cp95(by_kind.get(k, 0), total10)[0] <= KIND_SHARES[k] <= cp95(by_kind.get(k, 0), total10)[1]} for k in KIND_SHARES}
    u10 = sa10["connect_useful_signed_given_declared"]
    useful = {"share": u10, "predicted": USEFUL_WIDE, "cp_contains": u10["cp95"][0] <= USEFUL_WIDE <= u10["cp95"][1]}
    for n in names:
        check(f"non_clear (3) {n} attempted_events_per_birth equal to votes", wide[n]["attempted_events_per_birth"] == votes[n]["attempted_events_per_birth"],
              {"non_clear": wide[n]["attempted_events_per_birth"], "votes": votes[n]["attempted_events_per_birth"]})
        check(f"non_clear (4) {n} InputRefAdd applied per birth within 5 % of votes", within(add_rate(wide[n]), add_rate(votes[n]), 0.05),
              {"non_clear": add_rate(wide[n]), "votes": add_rate(votes[n])})
    for n in names[1:]:
        check(f"non_clear (5) {n} integrated lost_declaration within 5 % of votes", within(wide[n]["lost_declaration"]["rate"], votes[n]["lost_declaration"]["rate"], 0.05),
              {"non_clear": wide[n]["lost_declaration"], "votes": votes[n]["lost_declaration"], "routes_non_clear": wide[n]["lost_declaration_by_ops"], "routes_votes": votes[n]["lost_declaration_by_ops"]})
    result = {
        "gate": "W0",
        "frozen": {"path": frozen_path, "sha256": sha(frozen_path), "config": frozen_doc["config"]},
        "votes": {"path": votes_path, "sha256": sha(votes_path), "config": votes_doc["config"]},
        "non_clear": {"path": wide_path, "sha256": sha(wide_path), "config": wide_doc["config"]},
        "non_clear_10m": {"path": wide10_path, "sha256": sha(wide10_path), "config": wide10_doc["config"]},
        "checks": checks,
        "pass": all(c["pass"] for c in checks if c["gating"]),
        "non_clear_10m_kind_shares_descriptive": kind_shares,
        "non_clear_10m_useful_signed_descriptive": useful,
        "non_clear_founder_single_add": sa,
        "non_clear_10m_founder_single_add": sa10,
        "non_clear_10m_founder_all_birth": {k: a10[k] for k in ("declared_vote_node_target", "connect_any", "connect_cardinal_move", "connect_useful_component_move",
                                                                 "connect_useful_signed", "connect_useful_wrong_sign", "attempted_events_per_birth", "applied_events_per_birth", "births", "wall_secs")},
        "votes_founder_all_birth": {k: votes["founder"][k] for k in ("declared_vote_node_target", "connect_any", "connect_useful_signed", "attempted_events_per_birth", "applied_events_per_birth", "births")},
        "pooled_destinations_descriptive": {n: wide[n]["occurrences"]["connect_by_location"] for n in names},
        "wall_secs": {"frozen": {n: frozen[n]["wall_secs"] for n in names}, "votes": {n: votes[n]["wall_secs"] for n in names},
                      "non_clear": {n: wide[n]["wall_secs"] for n in names}, "non_clear_10m": {n: wide10[n]["wall_secs"] for n in names}},
    }
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("PASS" if c["pass"] else "FAIL", c["check"])
    print("W0", "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
