"""Gate V0.2 (ii) (vote-margin read run): the v2 census smoke. Passes when
the run completed, every reached checkpoint carries a `vote_deltas` block
whose parents each carry `exposure_contexts`, and every channel (original
and option C's) carries its v2 record: a class from the plan's table,
`consumers`, per-pass records when `delta_passes` > 0, parameter fields
when `scenes_actions_differ` > 0. Records the block size and wall time and
whether any executed target channel existed (if none, the structural
checks alone pass and the fixtures carry the classification).

usage: python3 smokecheck.py <dir> <name> <out.json>"""
import json
import sys

CLASSES = {"vote-causal", "param-causal", "action-unresolved", "unexposed", "vm-unresolved",
           "silent-local-decision", "silent-local-intermediate", "silent-mixed", "masked",
           "suppressing", "moving-residual"}


def channel_problems(where, c):
    v2 = c.get("v2")
    if not v2:
        return [f"{where}: no v2"]
    out = []
    if v2.get("class") not in CLASSES:
        out.append(f"{where}: class {v2.get('class')}")
    if "consumers" not in v2:
        out.append(f"{where}: no consumers")
    if c["stats"]["delta_passes"] > 0 and len(v2.get("passes", [])) != c["stats"]["delta_passes"]:
        out.append(f"{where}: {len(v2.get('passes', []))} pass records for {c['stats']['delta_passes']} delta passes")
    if c["stats"]["scenes_actions_differ"] > 0 and v2["params"]["action_changing_executions"] != c["stats"]["scenes_actions_differ"]:
        out.append(f"{where}: parameter executions disagree with scenes_actions_differ")
    return out


def main():
    d, name, out = sys.argv[1:4]
    rows = [json.loads(l) for l in open(f"{d}/census-{name}.ndjson") if l.startswith("{")]
    header = next(r for r in rows if r.get("kind") == "header")
    footer = next((r for r in rows if r.get("kind") == "footer"), None)
    checkpoints = [r for r in rows if r.get("kind") == "checkpoint"]
    problems, per = [], []
    if footer is None or footer.get("ticks") != header["horizon"] and not footer.get("extinct"):
        problems.append("run did not complete")
    if header.get("vote_delta_version") != "vote-delta-v2":
        problems.append("version")
    for cp in checkpoints:
        block = cp.get("vote_deltas")
        if block is None:
            problems.append(f"{cp['tick']}: no vote_deltas")
            continue
        channels = c_channels = 0
        classes = {}
        for i, p in enumerate(block):
            if "exposure_contexts" not in p:
                problems.append(f"{cp['tick']} parent {i}: no exposure_contexts")
                continue
            for c in p["channels"]:
                channels += 1
                problems += channel_problems(f"{cp['tick']} p{i} {c['family']}:{c['channel']}", c)
                classes[c["v2"]["class"]] = classes.get(c["v2"]["class"], 0) + 1
            for c in p["exposure_contexts"]["channels"]:
                c_channels += 1
                problems += channel_problems(f"{cp['tick']} p{i} C {c['family']}:{c['channel']}", c)
            if len(p["exposure_contexts"]["original_coverage"]) != len(p["channels"]):
                problems.append(f"{cp['tick']} p{i}: coverage rows != original candidates")
        per.append({"tick": cp["tick"], "parents": len(block), "channels": channels, "c_channels": c_channels,
                    "classes": classes, "vote_deltas_bytes": len(json.dumps(block, separators=(",", ":"))),
                    "vote_delta_secs": cp.get("vote_delta_secs"), "census_secs": cp.get("census_secs")})
    result = {"pass": not problems and bool(checkpoints), "problems": problems[:50], "checkpoints": per,
              "any_executed_target_channel": any(p["channels"] for p in per), "footer": footer}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    print(json.dumps({k: result[k] for k in ("pass", "any_executed_target_channel")}), per)
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
