#!/usr/bin/env python3
"""w-readpx — the frontier's reader column, re-derived at this tip.

Reads the scratch `READPX` lines off a scan's stderr (see `scan.sh` and the
diff quoted in the rung) and prints, for the 9 frontier TUs:

  * the reader column by first-blocker key, with a column-sum assertion;
  * the per-function listing, name by name;
  * the set difference against `docs/whitebox/WB_READER_FINDINGS.md` §1's
    48-row table (the morning's base `c34c388c`).

No absolute path lives in this file; the scan stem is `$1`.
"""
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ERR = os.path.join(HERE, (sys.argv[1] if len(sys.argv) > 1 else "base") + ".err")

# The 9 frontier TUs, read off the scan's own FRONTIER block (never hard-coded
# from a prior document).
OUT = os.path.join(HERE, (sys.argv[1] if len(sys.argv) > 1 else "base") + ".out")


def frontier_tus(path):
    tus, on = [], False
    for line in open(path, encoding="utf-8", errors="replace"):
        if line.startswith("  FRONTIER — "):
            on = True
            continue
        if on:
            if not line.startswith("       ") and not line.startswith("      "):
                break
            parts = [p.strip() for p in line.split("|")]
            if len(parts) != 3:
                break
            tus.append(parts[2])
    return tus


def rows(path):
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) != 9:
            continue
        yield {
            "tu": f[1], "name": f[2], "fnb": f[3], "key": f[4],
            "cflow": f[5], "off": f[6], "cls": f[7], "bytes": int(f[8]),
        }


def main():
    fr = set(frontier_tus(OUT))
    print("FRONTIER TUs: %d" % len(fr))
    all_rows = [r for r in rows(ERR)]
    print("READPX rows (whole workload emitted denominator): %d" % len(all_rows))
    fr_rows = [r for r in all_rows if r["tu"] in fr]
    print("frontier emitted denominator: %d" % len(fr_rows))
    buckets = collections.Counter(r["fnb"] for r in fr_rows)
    for k in sorted(buckets):
        print("  %-24s %d" % (k, buckets[k]))
    refused = [r for r in fr_rows if r["fnb"] == "fnbyte-refused"]
    print("\nreader column (frontier, fnbyte-refused): %d" % len(refused))

    by_key = collections.Counter(r["key"] for r in refused)
    by_key_tus = collections.defaultdict(set)
    for r in refused:
        by_key_tus[r["key"]].add(r["tu"])
    print("\n| key | n | TUs | example function |")
    print("|---|---:|---:|---|")
    tot = 0
    for k, n in sorted(by_key.items(), key=lambda kv: (-kv[1], kv[0])):
        ex = sorted(r["name"] for r in refused if r["key"] == k)[0]
        print("| `%s` | %d | %d | `%s` |" % (k, n, len(by_key_tus[k]), ex))
        tot += n
    print("| **total** | **%d** | %d | |" % (tot, len(set(r["tu"] for r in refused))))
    assert tot == len(refused), "COLUMN SUM BROKEN: %d != %d" % (tot, len(refused))
    print("\nCOLUMN-SUM ASSERT: %d keys sum to %d == %d refused  OK"
          % (len(by_key), tot, len(refused)))
    print("distinct keys: %d ; distinct names: %d ; distinct TUs: %d"
          % (len(by_key), len(set(r["name"] for r in refused)),
             len(set(r["tu"] for r in refused))))

    print("\n--- the full listing ---")
    print("| TU | function | key | cflow | cflow_off | bytes |")
    print("|---|---|---|---|---|---:|")
    for r in sorted(refused, key=lambda r: (r["tu"], r["name"])):
        print("| `%s` | `%s` | `%s` | `%s` | `%s` | %d |"
              % (r["tu"].split("/")[-1], r["name"], r["key"], r["cflow"],
                 r["off"] or "-", r["bytes"]))

    # per-TU
    print("\n--- per frontier TU ---")
    for tu in sorted(fr):
        rs = [r for r in fr_rows if r["tu"] == tu]
        rf = [r for r in rs if r["fnb"] == "fnbyte-refused"]
        ex = [r for r in rs if r["fnb"] == "fnbyte-exact"]
        keys = collections.Counter(r["key"] for r in rf)
        print("%-52s den %2d exact %2d reader %2d | %s"
              % (tu, len(rs), len(ex), len(rf),
                 " ".join("%s*%d" % (k, v) for k, v in sorted(keys.items()))))


main()
