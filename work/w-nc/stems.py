#!/usr/bin/env python3
"""w-nc — the replication controls: TU count, body count, demangled STEM count.

`w-jump` (#2000) established that a class whose bodies and TUs are the same
number is one TU wearing many names. `w-band` (#2246) established that the
`bodies == TUs` test is **structurally blind to templates**, because a mangled
name embeds its arguments — so `??1?$_STLP_alloc_proxy@H…` and
`??1?$_STLP_alloc_proxy@M…` are two bodies of one STEM. Every ranking table this
lane prints carries all three columns.

The STEM is the MSVC mangled name with its template arguments removed: strip the
leading `?`s, take the name up to the first `@@`, and inside that keep only the
qualifier chain's identifiers, dropping everything a `?$` template head
parameterises.

usage: stems.py SCAN.jsonl KEY [KEY...]
       stems.py SCAN.jsonl --all-keys N     top N keys with all three columns
"""
import json
import sys
from collections import defaultdict


def stem(name):
    """The template-argument-free identity of a mangled name."""
    n = name
    if not n.startswith("?"):
        return n  # a C name (`mmioClose`, `_vsprintf_s_l`) is its own stem
    # `??0Vector3@@QAA@MMM@Z` -> the qualifier chain is `Vector3`
    body = n.lstrip("?")
    parts = body.split("@")
    out = []
    for p in parts:
        if p == "":
            break
        if p.startswith("$"):
            # `?$basic_string` — a template head. Keep the head name only; every
            # argument after it is what #2246 says must not be counted as
            # separate work.
            out.append(p[1:].split("@")[0])
            break
        out.append(p)
    return "::".join(out) if out else body


def main():
    scan = sys.argv[1]
    rest = sys.argv[2:]
    rows = [json.loads(l) for l in open(scan)]
    rows = [r for r in rows if "src" in r]

    # key -> (TUs, bodies, stems)
    tus = defaultdict(set)
    bodies = defaultdict(set)
    stems = defaultdict(set)
    for r in rows:
        for k, v in r["emit"].items():
            if not k.startswith("fnbyte-parsefn|"):
                continue
            _, nm, why = k.split("|", 2)
            tus[why].add(r["src"])
            bodies[why].add((r["src"], nm))
            stems[why].add(stem(nm))

    if rest and rest[0] == "--all-keys":
        n = int(rest[1])
        keys = sorted(bodies, key=lambda k: -len(bodies[k]))[:n]
    else:
        keys = rest

    print(f"{'key':<52} {'bodies':>7} {'TUs':>6} {'STEMs':>7}  replication")
    for k in keys:
        b, t, s = len(bodies[k]), len(tus[k]), len(stems[k])
        flags = []
        if b == t:
            flags.append("bodies==TUs (#2000)")
        if s < b:
            flags.append(f"template collapse {b}->{s} (#2246)")
        print(f"{k:<52} {b:>7} {t:>6} {s:>7}  {'; '.join(flags) or '-'}")


if __name__ == "__main__":
    main()
