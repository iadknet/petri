"""Synthetic checks of analyze4 on fake summaries (no real run 4 data is read)."""
import contextlib
import io
import json
import os
import tempfile

import analyze4 as a

SEEDS = [5, 6, 7, 8, 9, 10]


def rep(i, reached, incomplete=False):
    return {"replicate": i, "reached": reached, "incomplete": incomplete, "final_best": 1.0}


def lad(i, first, status):
    return {"replicate": i, "first_not_pass": first, "statuses": [{"rung": "benefit", "status": status}]}


def summary(arms):
    return {"calibration": {"selected": {"food_fraction": 0.04}},
            "arms": [{"name": n, "replicates": [rep(i, r[i]) for i in range(8)]} for n, r, _ in arms],
            "ladder": {"arms": [{"name": n, "replicates": [lad(i, "benefit", s[i]) for i in range(8)]}
                                for n, _, s in arms]}}


def write(path, obj):
    os.makedirs(path, exist_ok=True)
    with open(os.path.join(path, "summary.json"), "w") as f:
        json.dump(obj, f)


def build(here, x_reached, x_status, log_extra=""):
    r = ([False] * 8, ["fail"] * 8)
    n = ([False] * 8, ["fail"] * 8)
    lines = []
    for assay in ("food", "wall"):
        with open(os.path.join(here, f"seeds-{assay}.txt"), "w") as f:
            f.write("\n".join(map(str, SEEDS)) + "\n")
        for s in [3, 4] + SEEDS:
            write(os.path.join(here, f"aa-{assay}-s{s}"),
                  summary([("native", *r), ("native-aa", *n)]))
            lines.append(f"aa {assay}-s{s} baseline ok")
        for x in a.ARMS:
            for s in SEEDS:
                write(os.path.join(here, f"{x}-{assay}-s{s}"),
                      summary([("native", *r), (x, x_reached, x_status)]))
                lines.append(f"{x} {assay}-s{s} baseline ok")
                with open(os.path.join(here, f"mask-{x}-{assay}-s{s}.ndjson"), "w") as f:
                    f.write(json.dumps({"probe": "r3-masking-verdict", "scene_identity": True}) + "\n")
    with open(os.path.join(here, "run4.log"), "w") as f:
        f.write("\n".join(lines) + "\n" + log_extra)


def run_case(x_reached, x_status, log_extra=""):
    with tempfile.TemporaryDirectory() as here:
        build(here, x_reached, x_status, log_extra)
        a.HERE = here
        with contextlib.redirect_stdout(io.StringIO()):
            a.main()
        with open(os.path.join(here, "analysis4.json")) as f:
            return json.load(f)


def check(name, cond):
    print(("ok  " if cond else "FAIL") + " " + name)
    return cond


ok = True
out = run_case([True] * 8, ["pass"] * 8)
ok &= check("every arm reaching everywhere is an A candidate on reach",
            out["outcome"] and set(out["outcome"]["A_candidates"]) == set(a.ARMS))
out = run_case([False] * 8, ["fail"] * 8)
ok &= check("identical arms give B (no candidates)", out["outcome"] == {"A_candidates": [], "B_supported": True})
out = run_case([None] * 8, ["fail"] * 8)
ok &= check("unknown arm reach supplies no A evidence",
            out["outcome"] is not None and out["outcome"]["A_candidates"] == [])
out = run_case([False] * 8, ["fail"] * 8, log_extra="m4 food-s5 baseline MISMATCH\n")
ok &= check("a logged mismatch blocks any formal outcome", out["outcome"] is None)
b = out["arms"]["m1"]["food"]["breakdown"]
ok &= check("breakdown per seed and density present", "seed 5" in b and "fraction 0.04" in b)


def shuffled_n_ladder():
    with tempfile.TemporaryDirectory() as here:
        build(here, [False] * 8, ["fail"] * 8)
        p = os.path.join(here, "aa-food-s5", "summary.json")
        with open(p) as f:
            s = json.load(f)
        lad_n = next(x for x in s["ladder"]["arms"] if x["name"] == "native-aa")
        lad_n["replicates"].reverse()
        with open(p, "w") as f:
            json.dump(s, f)
        a.HERE = here
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                a.main()
            return False
        except AssertionError:
            return True


ok &= check("a reordered N ladder is refused", shuffled_n_ladder())
print("all ok" if ok else "SOME FAILED")
