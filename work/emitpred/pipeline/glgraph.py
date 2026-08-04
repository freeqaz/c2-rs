#!/usr/bin/env python3
"""glgraph.py — the ODR-use reference graph, read straight out of `.gl`.

Each `.gl` symbol record is

    <kind> <token> <sep> <name> <fixed header ...> ( <token> <refcount> )*

i.e. c1xx writes, per symbol, the **complete list of symbols that symbol
references**, each with a use count. That list is the reference relation
PHASE7_PLAN §2's fixpoint runs over — for *data* symbols too, so a static table
of function pointers (`Easing.h`'s `gEaseFuncs[]`) links to every function whose
address it takes, and a vftable links to its slots.

Witness (`src/system/os/HolmesUtl.cpp`, dc3 @ fbf097a5): the record for
`?HolmesXboxPath@@YA?AVString@@PBD0@Z` carries exactly

    ?c_str@FixedString@@QBAPBDXZ  ??1String@@UAA@XZ  ??0String@@QAA@XZ
    ??0String@@QAA@PBD@Z  ??0String@@QAA@ABV0@@Z  ??4String@@QAAAAV0@PBD@Z
    ?FileQualifiedFilename@@YAXAAVString@@PBD@Z  DmMapDevkitDrive
    ??$MakeString@PBDVString@@@@YAPBDPBDABQBDABVString@@@Z

which is that function's callee set.

The payload is scanned for *any* operand token the `.gl` symbol index resolves,
stepping past a token when one is found. That is an over-approximation of the
reference list (a header field can alias a token value); it is deliberately on
the safe side, because §2's fixpoint must not lose an edge.
"""
import il
import model


def records(glb):
    """[(name, payload_start, payload_end)] in `.gl` record order."""
    runs = model.indexable_runs(glb)
    out = []
    for i, (s, e, nm, _sep) in enumerate(runs):
        nxt = runs[i + 1][0] - 1 if i + 1 < len(runs) else len(glb)
        out.append((nm, e, max(e, nxt)))
    return out


def ref_graph(glb):
    """{name: {referenced names}} over every `.gl` symbol — functions and data."""
    idx = il.gl_symbol_index(glb)
    out = {}
    for nm, a, b in records(glb):
        acc = out.setdefault(nm, set())
        p = a
        while p < b - 1:
            t = il.read_token_var(glb, p)
            if t is not None and t[0] in idx:
                acc.add(idx[t[0]])
                p += t[1]
            else:
                p += 1
        acc.discard(nm)
    return out
