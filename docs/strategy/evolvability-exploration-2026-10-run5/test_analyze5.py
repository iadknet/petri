"""Synthetic end-to-end checks of analyze5.main's gate (run with python3)."""
import contextlib
import io
import json
import os
import shutil
import tempfile

import analyze5 as a

STRATA = [str(i) for i in range(16)]


def summary(assay, stratum, arm, tag, start, end, b, c, fired=0, events=0, violations=0, native_idx=()):
    row = {"probe": "r5-screen", "assay": assay, "stratum": stratum, "arm": arm, "tag": tag,
           "range": [start, end], "parent_sha256": f"p-{assay}-{stratum}",
           "arm_mutation_sha256": "nm" if arm in ("aa", "native") else "am", "native_mutation_sha256": "nm",
           "banks": {"a": {"scenes": 32, "sha256": "A"}, "b": {"scenes": 32, "sha256": "B"}},
           "twin": {"cells": {"b": b, "c": c, "both": 0, "neither": end - start - b - c}},
           "native": {"counted": len(native_idx)},
           "j": {"applied_weight_events": events, "fired_jumps": fired, "fired_births": fired,
                 "twin_identity_violations": violations}}
    row["_records"] = [{"probe": "r5-screen-child", "arm": arm, "tag": tag, "assay": assay, "stratum": stratum,
                        "index": i, "side": "native", "label": "confirmed", "delta_b": 1.0,
                        "sterility": {"attributable": False}} for i in native_idx]
    return row


def aacheck(root, ok=True):
    """The equivalence evidence directory: both predeclared samples, passing unless `ok` is False."""
    d = os.path.join(root, "aacheck")
    if not os.path.isdir(d):
        os.makedirs(d)
        for assay, stratum in (("food", "0"), ("wall", "0")):
            with open(os.path.join(d, f"aacheck-{assay}-{stratum}.json"), "w") as f:
                json.dump({"match": ok}, f)
    return d


def write(dirpath, rows):
    os.makedirs(dirpath, exist_ok=True)
    for r in rows:
        name = f"{r['arm']}-{r['assay']}-{r['stratum']}-{r['range'][0]}.ndjson"
        records = r.pop("_records", [])
        with open(os.path.join(dirpath, name), "w") as f:
            f.write(json.dumps(r) + "\n")
            for rec in records:
                f.write(json.dumps(rec) + "\n")


def scenario(root, n, per_stratum_bc, strata=STRATA, aa=True, rerun=True, fixtures=True, fire=(1000, 10000)):
    main_rows, aa_rows = [], []
    for assay in ("food", "wall"):
        for s in strata:
            b, c = per_stratum_bc
            main_rows.append(summary(assay, s, "j", "run5", 0, n, b, c, fire[0], fire[1]))
            aa_rows.append(summary(assay, s, "native", "run5-aa", 0, n, 0, 0))
    write(os.path.join(root, "main"), main_rows)
    if aa:
        write(os.path.join(root, "aa"), aa_rows)
    if rerun:
        os.makedirs(os.path.join(root, "rerun"), exist_ok=True)
        for name in os.listdir(os.path.join(root, "main")):
            if "-0-" in name:
                shutil.copy(os.path.join(root, "main", name), os.path.join(root, "rerun", name))
    manifest = {"assays": {a_: {"strata": STRATA, "N": n} for a_ in ("food", "wall")},
                "fixtures": {"passed": fixtures}, "pcj": {"status": "not constructible"}}
    with open(os.path.join(root, "manifest.json"), "w") as f:
        json.dump(manifest, f)
    with contextlib.redirect_stdout(io.StringIO()):
        return a.main(os.path.join(root, "manifest.json"), os.path.join(root, "main"),
                      os.path.join(root, "aa"), os.path.join(root, "rerun"), aacheck(root))


def tamper(root, fn, manifest_fn=None):
    """Re-run main after `fn(root)` edits files and `manifest_fn(m)` edits the manifest."""
    if fn:
        fn(root)
    path = os.path.join(root, "manifest.json")
    if manifest_fn:
        with open(path) as f:
            m = json.load(f)
        manifest_fn(m)
        with open(path, "w") as f:
            json.dump(m, f)
    with contextlib.redirect_stdout(io.StringIO()):
        return a.main(path, os.path.join(root, "main"), os.path.join(root, "aa"), os.path.join(root, "rerun"), aacheck(root))


def edit_rows(dirpath, match, change):
    for name in os.listdir(dirpath):
        p = os.path.join(dirpath, name)
        with open(p) as f:
            row = json.loads(f.readline())
        if match(name, row):
            change(row)
            with open(p, "w") as f:
                f.write(json.dumps(row) + "\n")


def check(name, cond):
    print(("ok  " if cond else "FAIL") + " " + name)
    return cond


def run():
    ok = True
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "good-pass"), 1_000_000, (4, 0))
        ok &= check("complete strong excess passes", out["verdict"] == "pass")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "two-strata"), 1_000_000, (4, 0), strata=["0", "1"])
        ok &= check("missing strata is void", out["verdict"] == "inconclusive" and not out["validity"]["valid"])
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "no-aa"), 1_000_000, (4, 0), aa=False)
        ok &= check("missing A/A is void", out["verdict"] == "inconclusive")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "no-rerun"), 1_000_000, (4, 0), rerun=False)
        ok &= check("missing rerun is void", out["verdict"] == "inconclusive")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "no-fixtures"), 1_000_000, (4, 0), fixtures=False)
        ok &= check("failed fixtures is void", out["verdict"] == "inconclusive")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "bad-fire"), 1_000_000, (4, 0), fire=(1, 100))
        ok &= check("bad fire rate is void", out["verdict"] == "inconclusive")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "null"), 1_000_000, (5, 5))
        ok &= check("balanced 80/80 per assay is a supported negative", out["verdict"] == "supported negative")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "empty"), 1_000_000, (0, 0))
        ok &= check("no discordance at the cap is a supported negative via delta", out["verdict"] == "supported negative")
    with tempfile.TemporaryDirectory() as t:
        out = scenario(os.path.join(t, "small"), 20_000, (1, 1))
        ok &= check("16/16 per assay over 320,000 births excludes delta >= 2.5e-4",
                    out["verdict"] == "supported negative" and out["negative"]["food"]["delta_u"] < 2.5e-4)
    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "reversal")
        rows, aa_rows = [], []
        for s in STRATA:
            rows.append(summary("food", s, "j", "run5", 0, 20_000, 20, 2, 1000, 10000))
            rows.append(summary("wall", s, "j", "run5", 0, 20_000, 0, 5, 1000, 10000))
            for assay in ("food", "wall"):
                aa_rows.append(summary(assay, s, "native", "run5-aa", 0, 20_000, 0, 0))
        write(os.path.join(root, "main"), rows)
        write(os.path.join(root, "aa"), aa_rows)
        os.makedirs(os.path.join(root, "rerun"))
        for name in os.listdir(os.path.join(root, "main")):
            if "-0-" in name:
                shutil.copy(os.path.join(root, "main", name), os.path.join(root, "rerun", name))
        with open(os.path.join(root, "manifest.json"), "w") as f:
            json.dump({"assays": {x: {"strata": STRATA, "N": 20_000} for x in ("food", "wall")},
                       "fixtures": {"passed": True}, "pcj": {"status": "not constructible"}}, f)
        with contextlib.redirect_stdout(io.StringIO()):
            out = a.main(os.path.join(root, "manifest.json"), os.path.join(root, "main"),
                         os.path.join(root, "aa"), os.path.join(root, "rerun"), aacheck(root))
        ok &= check("food excess with a wall reversal is inconclusive",
                    out["verdict"] == "inconclusive" and out["validity"]["valid"])
    # Record-based A/A: pairing by index, and record completeness.
    def records_case(aa_counted_extra):
        with tempfile.TemporaryDirectory() as t:
            root = os.path.join(t, "records")
            n = 1_000_000
            main_rows, aa_rows = [], []
            for assay in ("food", "wall"):
                for s in STRATA:
                    mi = (1, 2, 3) if (assay, s) == ("food", "0") else ()
                    ai = (2, 3, 4, 5) if (assay, s) == ("food", "0") else ()
                    main_rows.append(summary(assay, s, "j", "run5", 0, n, 5, 5, 1000, 10000, native_idx=mi))
                    row = summary(assay, s, "native", "run5-aa", 0, n, 0, 0, native_idx=ai)
                    if (assay, s) == ("food", "0"):
                        row["native"]["counted"] += aa_counted_extra
                    aa_rows.append(row)
            write(os.path.join(root, "main"), main_rows)
            write(os.path.join(root, "aa"), aa_rows)
            os.makedirs(os.path.join(root, "rerun"))
            for name in os.listdir(os.path.join(root, "main")):
                if "-0-" in name:
                    shutil.copy(os.path.join(root, "main", name), os.path.join(root, "rerun", name))
            with open(os.path.join(root, "manifest.json"), "w") as f:
                json.dump({"assays": {x: {"strata": STRATA, "N": n} for x in ("food", "wall")},
                           "fixtures": {"passed": True}, "pcj": {"status": "not constructible"}}, f)
            with contextlib.redirect_stdout(io.StringIO()):
                return a.main(os.path.join(root, "manifest.json"), os.path.join(root, "main"),
                              os.path.join(root, "aa"), os.path.join(root, "rerun"), aacheck(root))

    out = records_case(0)
    ok &= check("record-based A/A pairs by index (b 2, c 1)",
                out["validity"]["aa"]["b"] == 2 and out["validity"]["aa"]["c"] == 1 and out["validity"]["valid"])
    out = records_case(1)
    ok &= check("incomplete A/A records are void", not out["validity"]["valid"] and out["verdict"] == "inconclusive")

    def extra_file(root, name, rows):
        with open(os.path.join(root, name), "w") as f:
            for r in rows:
                f.write(json.dumps(r) + "\n")

    def rec(i, arm="native", tag="run5-aa", attributable=False, stratum="0"):
        return {"probe": "r5-screen-child", "arm": arm, "tag": tag, "assay": "food", "stratum": stratum,
                "index": i, "side": "native", "label": "confirmed", "delta_b": 1.0,
                "sterility": {"attributable": attributable}}

    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "c")
        scenario(root, 1_000_000, (5, 5))
        out = tamper(root, lambda r: extra_file(os.path.join(r, "aa"), "orphan.ndjson", [rec(7)]))
        ok &= check("an orphan A/A record file is void", not out["validity"]["valid"])
    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "c")
        scenario(root, 1_000_000, (5, 5))

        def out_of_range(r):
            p = os.path.join(r, "aa", "native-food-0-0.ndjson")
            with open(p, "a") as f:
                f.write(json.dumps(rec(2_000_000)) + "\n")
        out = tamper(root, out_of_range)
        ok &= check("an out-of-range A/A record is void", not out["validity"]["valid"])
    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "c")
        scenario(root, 1_000_000, (5, 5))

        def attributable(r):
            p = os.path.join(r, "aa", "native-food-0-0.ndjson")
            with open(p, "a") as f:
                f.write(json.dumps(rec(9, attributable=True)) + "\n")
        out = tamper(root, attributable)
        ok &= check("a sterility-attributable A/A record is not counted",
                    out["validity"]["valid"] and out["validity"]["aa"]["b"] == 0)
    with tempfile.TemporaryDirectory() as t:
        root = os.path.join(t, "c")
        os.makedirs(os.path.join(root, "aacheck"))
        for assay, stratum in (("food", "0"), ("wall", "0")):
            with open(os.path.join(root, "aacheck", f"aacheck-{assay}-{stratum}.json"), "w") as f:
                json.dump({"match": assay == "wall"}, f)
        scenario(root, 1_000_000, (5, 5))
        out = tamper(root, None)
        ok &= check("a failed equivalence sample is void", not out["validity"]["valid"])

    # The aacheck function itself, on a synthetic probe sample.
    def aacheck_case(probe_native, probe_arm, cells):
        with tempfile.TemporaryDirectory() as t:
            root = os.path.join(t, "q")
            main_row = summary("food", "0", "j", "run5", 0, 50, 0, 0, native_idx=(1, 2, 3))
            aa_row = summary("food", "0", "native", "run5-aa", 0, 50, 0, 0, native_idx=(2, 3, 4))
            write(os.path.join(root, "main"), [main_row])
            write(os.path.join(root, "aa"), [aa_row])
            probe = summary("food", "0", "aa", "run5", 0, 20, cells[0], cells[1], native_idx=probe_native)
            arm_recs = [dict(rec(i, arm="aa", tag="run5"), side="arm") for i in probe_arm]
            probe["_records"] += arm_recs
            write(os.path.join(root, "probe"), [probe])
            try:
                with contextlib.redirect_stdout(io.StringIO()):
                    a.aacheck(os.path.join(root, "probe"), os.path.join(root, "main"),
                              os.path.join(root, "aa"), "food", "0", "20")
                return True
            except SystemExit:
                return False

    ok &= check("aacheck accepts identical record sets", aacheck_case((1, 2, 3), (2, 3, 4), (1, 1)))
    ok &= check("aacheck rejects different record sets", not aacheck_case((1, 2), (2, 3, 4), (1, 0)))
    # Codex round 3 reproductions: each must be void.
    void_cases = {
        "partial founder rerun": (lambda r: [
            os.remove(os.path.join(r, "rerun", n)) for n in os.listdir(os.path.join(r, "rerun")) if "wall" in n], None),
        "manifest with two strata": (None, lambda m: m["assays"]["food"].update({"strata": ["0", "1"]})),
        "manifest N too small": (None, lambda m: [v.update({"N": 100}) for v in m["assays"].values()]),
        "missing twin violations field": (lambda r: edit_rows(
            os.path.join(r, "main"), lambda n, row: row["stratum"] == "3", lambda row: row["j"].pop("twin_identity_violations")), None),
        "banks differ across strata": (lambda r: edit_rows(
            os.path.join(r, "main"), lambda n, row: row["stratum"] == "5",
            lambda row: row["banks"]["a"].update({"sha256": "OTHER"})), None),
        "A/A uses a different native block": (lambda r: edit_rows(
            os.path.join(r, "aa"), lambda n, row: True,
            lambda row: row.update({"native_mutation_sha256": "x", "arm_mutation_sha256": "x"})), None),
        "PC-J evidence missing": (None, lambda m: m.pop("pcj")),
        "PC-J powered flag contradicts power": (None, lambda m: m.update(
            {"pcj": {"status": "constructed", "N": 1_000_000, "power": 0.99, "powered": False,
                     "pilot_b0": 3, "pilot_c0": 0}})),
        "PC-J not constructible with power fields": (None, lambda m: m.update(
            {"pcj": {"status": "not constructible", "power": 0.99, "powered": True}})),
        "PC-J NaN power": (None, lambda m: m.update(
            {"pcj": {"status": "constructed", "N": 1_000_000, "power": float("nan"), "powered": False,
                     "pilot_b0": 3, "pilot_c0": 0}})),
        "PC-J negative power": (None, lambda m: m.update(
            {"pcj": {"status": "constructed", "N": 1_000_000, "power": -1.0, "powered": False,
                     "pilot_b0": 3, "pilot_c0": 0}})),
        "provenance removed everywhere": (lambda r: [edit_rows(
            os.path.join(r, d), lambda n, row: True,
            lambda row: [row.pop(k, None) for k in ("parent_sha256", "arm_mutation_sha256",
                                                    "native_mutation_sha256", "banks")])
            for d in ("main", "aa", "rerun")], None),
        "empty twin cells": (lambda r: [edit_rows(
            os.path.join(r, d), lambda n, row: True, lambda row: row["twin"].update({"cells": {}}))
            for d in ("main", "rerun")], None),
        "identity violations that cancel": (lambda r: [edit_rows(
            os.path.join(r, "main"), lambda n, row, s=s: row["stratum"] == s,
            lambda row, v=v: row["j"].update({"twin_identity_violations": v})) for s, v in (("2", 1), ("3", -1))], None),
    }
    for label, (fn, mfn) in void_cases.items():
        with tempfile.TemporaryDirectory() as t:
            root = os.path.join(t, "case")
            scenario(root, 1_000_000, (5, 5))
            out = tamper(root, fn, mfn)
            ok &= check(f"{label} is void", out["verdict"] == "inconclusive" and not out["validity"]["valid"])
    print("all ok" if ok else "SOME FAILED")


if __name__ == "__main__":
    run()
