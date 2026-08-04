#!/usr/bin/env python3
"""marks.py — a faithful replay of `0x10b98e26`, WITH the owner skips.

w-mark's `I` is the UNFILTERED reading: every function named by any `02` node in
the TU.  This module replays the walk instruction for instruction instead, so
the roots are the set c2 would actually Mark.

    driver          0x10b98e26   (one caller chain, before the compile loop)
    WalkInit        0x10b98b00
    RecurseSym (S5) 0x10b98c0f
    Mark            0x10b276e4   (the closure; applied by the caller)

The gates, by address:

    G0   0x10b98e4a  [rec+0x20] & 1                    NOT MODELLED (prereg 9.3)
    S1   0x10b98e9f  ([owner+0x20] & 0x60) == 0x20     skip the record ENTIRELY
    W1   0x10b98b09  [owner+0x30] != 1                 no node walk
    W2   0x10b98b14  !([owner+0x20] & 0x480)           no node walk
    A    0x10b98ba8  [[owner+0xc]+0x4d] == 0x1d        Mark ONE target, ABORT
    S3m  0x10b98be8  kind4 && +0x37&0x200000 && !+0x37&0x400   -> MARK
    S2   0x10b98ecd  [[owner+0xc]+0x4d] == 0x1d        skip the S5 pass
    S3   0x10b98ed9  kind1 && [owner+0x20] & 0x4000    skip the S5 pass
    S5m  0x10b98c7f  kind4 && +0x37&0x200000 && !+0x37&0x400   -> MARK

MAPPING ONTO THE CORPUS, stated because it is where a decode becomes a proxy:

  * `[t+0x30]==4 && [t+0x37]&0x200000` is exactly "has a tag-0x0E `.gl` record",
    since `0x200000` is set at `0x10b9bf50` in the tag-0x0E arm only.  So it is
    `name in U`, w-refs' set, unchanged.
  * `[t+0x37]&0x400` is storage-class nibble 0xa (`0x10b9be44`), w-refs' `skip`.
  * `[t+0x4c]&2` is w-roots' DONE bit, read from the same flag word as the seed.
  * `[t+0x32]&4` is the walk's own DFS re-entrancy marker: emulated as a stack.

TWO PLACES WHERE THIS DECODER KNOWS LESS THAN c2, AND BOTH ARE RESOLVED
INCLUSIVELY so that a blind spot can never be mistaken for a filter:

  1. An `in` record whose owner token does not bind to a decoded `.gl` record.
     c2 resolves it; this decoder may not.  **LOOSE (the headline): the record's
     nodes are contributed unfiltered**, exactly as w-mark did.  STRICT (a
     sensitivity, reported beside it): the record is dropped.
  2. `0x10b9893b` hands `WalkInit` ONE record per owner token; a symbol with
     several records would have the others walked by the driver anyway.  This
     replay uses the **union** of an owner's records' nodes, in stream order.

stdlib only.  Reads no c2 output.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
sys.path.insert(0, os.path.join(HERE, "..", "w-refs"))
sys.path.insert(0, os.path.join(HERE, "..", "w-mark"))
import il          # noqa: E402
import instream    # noqa: E402
import glowner     # noqa: E402
from glflags import i32c   # noqa: E402

REC_TAGS = instream.REC_TAGS
S1_MASK, S1_VALUE = 0x60, 0x20
WALK_BITS = 0x480
S3_BIT = 0x4000
TYPE_1D = 0x1D
DONE_BIT = 0x02


def parse_records(data):
    """instream.parse, but keeping the owner token per record IN STREAM ORDER
    and the tag-0x07 flags byte.  -> (clean, [(tag, flagbyte, owner, [tok...])])"""
    recs = []
    p = 0
    n = len(data)
    try:
        while p < n:
            if p == n - 1 and data[p] == 0x07:
                return (True, recs)
            tag = data[p]
            if tag not in REC_TAGS:
                return (False, recs)
            q = p + 1
            fl = None
            if tag == 0x07:
                fl = data[q]
                q += 1
            owner, q = instream.var_u_be(data, q)
            _, q = i32c(data, q)
            refs = []
            while q < n and data[q] not in REC_TAGS:
                q = instream.node(data, q, refs)
            recs.append((tag, fl, owner, refs))
            p = q
    except (IndexError, ValueError):
        return (False, recs)
    return (True, recs)


class Replay(object):
    def __init__(self, glb, inrecs, syms, idx, U, urecs, loose=True):
        self.glb = glb
        self.syms = syms          # token -> .gl owner record
        self.idx = idx            # token -> name
        self.U = U                # names with a tag-0x0E record
        self.urecs = urecs        # name -> refs.scan record
        self.loose = loose
        self.marks = set()
        self.nodes = {}           # owner token -> [node tokens], stream order
        self.records = []
        self._tk = {}
        for tag, fl, owner, toks in inrecs:
            self.records.append(owner)
            self.nodes.setdefault(owner, []).extend(toks)
        self.stat = {"rec": len(self.records), "owner_unbound": 0,
                     "s1": 0, "s2": 0, "s3": 0, "w1": 0, "w2": 0,
                     "walk_enabled": 0, "abort_1d": 0, "loose_fallback": 0,
                     "type_known": 0, "type_unknown": 0, "flagbyte_nonzero": 0}
        for tag, fl, owner, toks in inrecs:
            if fl:
                self.stat["flagbyte_nonzero"] += 1

    # ---- helpers -----------------------------------------------------
    def type_kind(self, otok):
        """[[owner+0xc]+0x4d], or None when the owner has no +0x0c token
        (`+0x20 & 0x200` clear -> the module default, which this decoder does
        not have) or the type record is not uniquely locatable."""
        if otok in self._tk:
            return self._tk[otok]
        r = self.syms.get(otok)
        v = None
        if r is not None and r["typetok"] is not None:
            v = glowner.type_kind(self.glb, r["typetok"])
        self._tk[otok] = v
        if v is None:
            self.stat["type_unknown"] += 1
        else:
            self.stat["type_known"] += 1
        return v

    def is_fn(self, tok):
        nm = self.idx.get(tok)
        return nm if (nm is not None and nm in self.U) else None

    # ---- 0x10b98b00 --------------------------------------------------
    def walk_init(self, otok, stack, tk_owner=None):
        r = self.syms.get(otok)
        if r is None:
            return 1
        if r["kind"] != 1:                       # W1, 0x10b98b09
            self.stat["w1"] += 1
            return 1
        if not (r["f20"] & WALK_BITS):           # W2, 0x10b98b14
            self.stat["w2"] += 1
            return 1
        tk = self.type_kind(otok)
        for ntok in self.nodes.get(otok, ()):
            nm = self.is_fn(ntok)
            if nm is not None:
                rec = self.urecs[nm]
                if rec["skip"]:                  # +0x37 & 0x400 -> Redirect
                    continue
                if tk == TYPE_1D and not (rec["flags"] & DONE_BIT):
                    self.marks.add(nm)           # 0x10b98c08
                    self.stat["abort_1d"] += 1
                    return 0
            if ntok in stack:                    # [t+0x32] & 4
                continue
            stack.add(ntok)
            rr = self.walk_init(ntok, stack)
            stack.discard(ntok)
            if rr == 0:
                return 0
            if nm is not None:
                self.marks.add(nm)               # 0x10b98be8
        return 1

    # ---- 0x10b98c0f --------------------------------------------------
    def recurse_sym(self, tok, stack):
        nm = self.is_fn(tok)
        if nm is not None:
            if not self.urecs[nm]["skip"]:
                self.marks.add(nm)               # 0x10b98c7f
            return
        r = self.syms.get(tok)
        if r is None or r["kind"] != 1:
            return
        if not (r["f20"] & WALK_BITS):           # 0x10b98c89
            return
        for ntok in self.nodes.get(tok, ()):
            if ntok in stack:
                continue
            stack.add(ntok)
            self.recurse_sym(ntok, stack)
            stack.discard(ntok)

    # ---- 0x10b98e26 --------------------------------------------------
    def run(self):
        seen_owner = set()
        for otok in self.records:
            r = self.syms.get(otok)
            if r is None:
                self.stat["owner_unbound"] += 1
                if self.loose:
                    self.stat["loose_fallback"] += 1
                    for ntok in self.nodes.get(otok, ()):
                        nm = self.is_fn(ntok)
                        if nm is not None and not self.urecs[nm]["skip"]:
                            self.marks.add(nm)
                continue
            if (r["f20"] & S1_MASK) == S1_VALUE:          # SKIP 1
                self.stat["s1"] += 1
                continue
            if otok not in seen_owner:
                seen_owner.add(otok)
                if r["kind"] == 1 and (r["f20"] & WALK_BITS):
                    self.stat["walk_enabled"] += 1
            rr = self.walk_init(otok, set([otok]))
            if rr == 0:
                continue
            if self.type_kind(otok) == TYPE_1D:           # SKIP 2
                self.stat["s2"] += 1
                continue
            if r["kind"] == 1 and (r["f20"] & S3_BIT):    # SKIP 3
                self.stat["s3"] += 1
                continue
            self.recurse_sym(otok, set([otok]))
        return self.marks, self.stat


def replay(glb, inb, urecs, U, loose=True):
    """-> (marks, stat, clean, syms_stat)."""
    clean, inrecs = parse_records(inb)
    syms, sst = glowner.read_symbols(glb)
    idx = il.gl_symbol_index(glb)
    rp = Replay(glb, inrecs, syms, idx, U, urecs, loose=loose)
    marks, stat = rp.run()
    stat["syms_bound"] = sst["bound"]
    stat["syms_k1"] = sst["k1"]
    stat["syms_k4"] = sst["k4"]
    stat["syms_hdrfail"] = sst["hdr_fail"]
    return marks, stat, clean, sst
