#!/usr/bin/env python3
"""alias.py — **the RUST tag-0x10 alias table, served through w-emitp's API.**

This file is a *drop-in replacement* for `work/w-emitp/alias.py`.  It decodes
nothing: it serves the table that `c2_il::gl_alias_table` produced, keyed by a
hash of the `.gl` bytes, through exactly the signature w-emitp's `scan.py`
calls.

WHY IT EXISTS.  `work/w-alias/scan_rust.py` is **byte-identical** to
`work/w-emitp/scan.py` — `cmp` says so, and the lane's rung doc records the
check.  `scan.py` resolves `import alias` from its own directory, so dropping
this file beside a verbatim copy substitutes the Rust decode for the Python one
with **zero** edits to the model.  Any difference in the scored table is then a
difference in the *decode*, and cannot be a difference in the model.

The Rust side is `crates/c2-il/tests/gl_alias_corpus.rs`, whose dump this reads
from `$C2RS_ALIAS_JSONL`.  All three tables are served — `shift=0` and both
nulls — so no Python alias decode is reached at any point.

stdlib only.  Reads no c2 output.
"""
import json
import os

_JSONL = os.environ["C2RS_ALIAS_JSONL"]


def _fnv1a(b):
    """FNV-1a 64.  Must agree byte for byte with `fnv1a` in the Rust dump."""
    h = 0xCBF29CE484222325
    for c in b:
        h = ((h ^ c) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def _load():
    by_hash = {}
    with open(_JSONL) as fh:
        for line in fh:
            o = json.loads(line)
            by_hash[int(o["fnv"], 16)] = o
    return by_hash


_TAB = _load()

# The stat keys `scan.py` reads, mapped from the Rust dump's names.
_ST = {"tag10": "tag10", "bound": "bound", "head_fail": "head_fail",
       "rt_fail": "rt_fail", "unbound_target": "unbound_target",
       "self_alias": "self", "dup": "dup"}


def scan(glb, shift=0):
    """-> ({alias_name: target_name}, {}, stats), the Rust table.

    Same signature and same return shape as w-emitp's `alias.scan`.  The
    token-keyed second element is discarded by every caller and is returned
    empty rather than reconstructed.
    """
    o = _TAB[_fnv1a(glb)]
    key = {0: "pairs", -1: "pairs_m1", 1: "pairs_p1"}[shift]
    st = dict((k, o[v]) for k, v in _ST.items())
    st["runs"] = 0
    st["unbound_self"] = 0
    if shift == -1:
        st["bound"] = o["bound_m1"]
    elif shift == 1:
        st["bound"] = o["bound_p1"]
    return dict(o[key]), {}, st
