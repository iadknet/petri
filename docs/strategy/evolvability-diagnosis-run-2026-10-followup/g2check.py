"""Gate G2: the N3 census's input_use blocks at 20,000 and 50,000 and its e6
block at 20,000 must hash-equal E2's Confluence blocks, each serialized once
with sorted keys and compact separators.

usage: python3 g2check.py <n3 census-*.ndjson> <e2 census-confluence.ndjson> <out.json>"""
import hashlib
import json
import sys


def blocks(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        if r.get("kind") == "checkpoint":
            out[r["tick"]] = r
    return out


def h(obj):
    return hashlib.sha256(json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def main():
    n3_path, e2_path, out = sys.argv[1:4]
    n3, e2 = blocks(n3_path), blocks(e2_path)
    checks = []
    for tick, key in ((20000, "input_use"), (50000, "input_use"), (20000, "e6")):
        a = n3.get(tick, {}).get(key)
        b = e2.get(tick, {}).get(key)
        checks.append({"tick": tick, "block": key, "n3_present": a is not None, "e2_present": b is not None,
                       "n3_sha256": h(a) if a is not None else None, "e2_sha256": h(b) if b is not None else None,
                       "equal": a is not None and b is not None and h(a) == h(b)})
    result = {"gate": "G2", "n3": n3_path, "e2": e2_path, "checks": checks,
              "pass": all(c["equal"] for c in checks), "n3_checkpoints": sorted(n3), "e2_checkpoints": sorted(e2)}
    open(out, "w").write(json.dumps(result, indent=1) + "\n")
    for c in checks:
        print("EQUAL" if c["equal"] else "DIFFERS", c["tick"], c["block"])
    print("G2", "pass" if result["pass"] else "FAIL")
    sys.exit(0 if result["pass"] else 1)


if __name__ == "__main__":
    main()
