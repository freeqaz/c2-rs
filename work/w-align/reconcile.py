#!/usr/bin/env python3
"""reconcile.py — the two-instrument check for lane w-align. Read-only.

    reconcile.py <probe.txt> <glread.txt ...>

Instrument A: the PRODUCTION cursor (`crates/c2-il/tests/in_init_probe.rs`'s
`gl-data` line — `data_object_at` itself, via `IlBundle::gl_data_report`).

Instrument B: `work/w-align/glread.py` — a crate-free Python re-implementation
of `data_object_at`'s frame from the documented grammar, which also reads the
ORACLE alignment out of c2's own obj.

Neither may be the other's witness, so the comparison lives here, outside both.
Every named object record is checked three ways and a disagreement is PRINTED
with both numbers rather than resolved:

    A.accepted == B.accepted        the two readers agree on accept/refuse
    A.align    == B.tag-align       the two readers agree on the alignment
    A.align    == c2.align          the reading agrees with REAL c2

The third comparison is the only one that is evidence; the first two are what
makes it trustworthy. A record c2 defines in no section (`c2:-`) is skipped in
the third and reported in a separate count.
"""
import re
import sys

ALIGN_OF_TAG = {0x82: 1, 0x84: 2, 0x86: 4, 0x88: 8}


def load_probe(path):
    """cell -> {name: align} from the production cursor."""
    out = {}
    for line in open(path):
        if "\tgl-data records=" not in line:
            continue
        cell, rest = line.split("\t", 1)
        body = rest[rest.index("[") + 1:rest.rindex("]")]
        d = {}
        for tok in body.split():
            parts = tok.split(":")
            name = parts[0]
            d[name] = int(parts[2].split("=")[1])
        out[cell] = d
    return out


def load_glread(paths):
    """cell -> {name: (tag, mark, refused, c2align)} from the crate-free parser."""
    out, cell, cur = {}, None, None
    for path in paths:
        for line in open(path):
            if line.startswith("== "):
                cell = line[3:].split()[0]
                out.setdefault(cell, {})
                cur = None
            elif line.startswith("   ") and not line.startswith("      "):
                cur = line.strip()
            elif "gl: tag=" in line and cur:
                kv = dict(re.findall(r"(\w+)=(\S+)", line))
                out[cell][cur] = {
                    "tag": int(kv["tag"], 16),
                    "mark": kv["mark"],
                    "refused": kv["refused"],
                    "size": int(kv["size"]),
                    "c2align": None,
                }
            elif "c2: section=" in line and cur:
                kv = dict(re.findall(r"(\w+)=(\S+)", line))
                out[cell][cur]["c2align"] = int(kv["align"])
                cur = None
            elif "c2: (symbol" in line and cur:
                cur = None
    return out


probe = load_probe(sys.argv[1])
glr = load_glread(sys.argv[2:])

agree_accept = disagree_accept = 0
agree_align = disagree_align = 0
oracle_ok = oracle_bad = oracle_none = 0
rows = []
for cell in sorted(set(probe) | set(glr)):
    a = probe.get(cell, {})
    b = glr.get(cell, {})
    for name in sorted(set(a) | set(b)):
        if name.startswith(("__C1", "__C2", "??_C")):
            continue
        if name.endswith("$initializer$") or name == "sL$initializer$":
            continue                      # the port's own slot, not a typed object
        rec = b.get(name)
        a_acc = name in a
        b_acc = rec is not None and rec["refused"] == "None"
        if a_acc == b_acc:
            agree_accept += 1
        else:
            disagree_accept += 1
            rows.append(f"ACCEPT-DISAGREE {cell}/{name}: cursor={a_acc} python={b_acc}")
        if not a_acc:
            continue
        b_align = ALIGN_OF_TAG.get(rec["tag"] & ~0x40) if rec else None
        if a[name] == b_align:
            agree_align += 1
        else:
            disagree_align += 1
            rows.append(f"ALIGN-DISAGREE {cell}/{name}: cursor={a[name]} python={b_align}")
        c2 = rec["c2align"] if rec else None
        if c2 is None:
            oracle_none += 1
        elif c2 == a[name]:
            oracle_ok += 1
        else:
            oracle_bad += 1
            rows.append(f"ORACLE-DISAGREE {cell}/{name}: read={a[name]} c2={c2}")

for r in rows:
    print(r)
print(f"accept:  agree={agree_accept} disagree={disagree_accept}")
print(f"align:   agree={agree_align} disagree={disagree_align}")
print(f"oracle:  confirmed={oracle_ok} contradicted={oracle_bad} no-section={oracle_none}")
