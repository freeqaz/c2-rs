#!/usr/bin/env python3
"""target370.py — pull board #980's cluster out of a `--fnbyte-diff-jsonl` file.

The cluster is defined by the JUDGE's bytes, not by a name: c2's whole body for
the symbol is the single word `4e800020` and it carries no relocation. Prints
one `<tu>\t<sym>` row per member, and a summary by template family.

    target370.py <fndiff.jsonl> [--list]
"""
import json
import sys
import collections


def rows(path):
    for line in open(path):
        line = line.strip()
        if line:
            yield json.loads(line)


def main(argv):
    path = argv[1]
    want_list = "--list" in argv
    hits = [r for r in rows(path) if r["ref_hex"] == ["4e800020"]]
    print(f"{len(hits)} differs whose whole reference body is one blr", file=sys.stderr)
    tus = {r["tu"] for r in hits}
    print(f"{len(tus)} TUs", file=sys.stderr)
    fam = collections.Counter(r["sym"].split("@")[0] for r in hits)
    for k, v in fam.most_common(10):
        print(f"  {v:5d}  {k}", file=sys.stderr)
    ports = collections.Counter(tuple(r["port_hex"]) for r in hits)
    for k, v in ports.most_common(10):
        print(f"  port {v:5d}  {' '.join(k)}", file=sys.stderr)
    if want_list:
        for r in sorted(hits, key=lambda r: (r["tu"], r["sym"])):
            print(f"{r['tu']}\t{r['sym']}")


if __name__ == "__main__":
    main(sys.argv)
