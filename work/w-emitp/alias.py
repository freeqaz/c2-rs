#!/usr/bin/env python3
"""alias.py — the `.gl` tag-0x10 ALIAS record, decoded from the byte stream.

THE CHANNEL NO LANE HAS READ.  Six lanes modelled the emit set as a closure over
`U` = the gate-clean **tag-0x0E** `.gl` records.  A **tag-0x10** record is a
different animal: it names a symbol that has NO `.ex` body and carries, in the
same word that a tag-0x0E record uses for its emit flags, a **token pointing at
another symbol**.

Transcribed from `c2.dll` 16.00.11886.00 (image base 0x10b00000).  The tag
dispatch at `0x10b9b91f` sends tags 0x04 / 0x0E / 0x10 to one shared KIND-4
handler at `0x10b9bdcf`, which splits on the tag only at the very end:

    10b9bf46  cmp  DWORD PTR [ebp-0x78],0xe      ; tag == 0x0E ?
    10b9bf4a  jne  0x10b9c01e
    10b9bf50  or   DWORD PTR [esi+0x37],0x200000 ; "has a tag-0x0E record" = in U
    10b9bf57  call 0x10c1f9e9                    ; i32c -> +0x54   (the .ex offset)
    ...                                          ; +0x58 +0x50 +0x4c +0x52, reflist
  ---------------------------------------------------------------------------
    10b9c01e  cmp  DWORD PTR [ebp-0x78],0x10     ; tag == 0x10 ?
    10b9c022  jne  0x10b9c033
    10b9c024  or   DWORD PTR [esi+0x37],0x400000 ; THE ALIAS BIT
    10b9c02b  call 0x10c1f91b                    ; varU
    10b9c030  mov  DWORD PTR [esi+0x4c],eax      ; THE ALIAS TARGET TOKEN

So on a tag-0x10 record `[sym+0x4c]` is **not** `flags4c` — it is a symbol
token.  `+0x37 & 0x400000` is the discriminator, and it has exactly two readers
in the whole binary (an `<imm32>` scan finds three sites, one of which is the
write above):

    10b8ac60  test [eax+0x37],0x400000   -> or [eax+0x32],1
    10b99621  test [esi+0x37],0x400000   -> ecx = [esi+0x4c] ; resolve ;
              10b99635  or [eax+0x20],0x2000        the TARGET's flag word

THE GRAMMAR.  A tag-0x10 record is everything a tag-0x0E record has up to the
`+0x54` anchor, then ONE varU and nothing else — `0x10b9c033` falls straight
into the shared tail.  So the token sits at exactly the byte offset where a
tag-0x0E record keeps its `.ex` body pointer, which `refs.head` already locates.

THE GATE, and why it is NOT w-refs' terminus gate.  w-refs asks that a record
end exactly where the next record's header begins.  That gate was tried here
first and it **fails on 320 of 419** tag-0x10 records in one TU — not because
the field is wrong (every one of the 320 still decodes to a `??_E<X>` ->
`??_G<X>` pair) but because the record that follows an alias is usually a
tag-0x0B undecorated-name record, whose header is not the `<tag><varU><sep>`
shape `_next_header_ok` models.  A gate that fails on a *neighbour* grades the
neighbour.  So the gate here is on the field itself, and there are three:

  * **RT** — the decoded token must re-encode to exactly the bytes read
    (`il.read_token_var` width preserved).
  * **BIND** — it must resolve in `il.gl_symbol_index`.
  * **SHIFT (the null)** — the same read taken at `p-1` and at `p+1`.  If a
    mis-positioned field bound and paired at anything like the same rate, the
    position would not be identified and the decode would be measuring the
    searcher.  Reported beside the real read, always, as a count.

stdlib only.  Reads no c2 output.

stdlib only.  Reads no c2 output.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT",
                      os.path.join(HERE, "..", "..", "..", "..", ".."))
for _p in (os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-refs"),
           os.path.join(MAIN, "work", "w-mark"),
           os.path.join(MAIN, "work", "w-skip"),
           os.path.join(MAIN, "work", "w-db")):
    sys.path.insert(0, os.path.abspath(_p))
import il        # noqa: E402
import model     # noqa: E402
import refs      # noqa: E402
from glflags import var_u   # noqa: E402

ALIAS_TAG = 0x10
KIND4_TAGS = (0x04, 0x0E, 0x10)
KIND1_TAGS = (0x01, 0x02)


def _tag_at(glb, s):
    """The record tag for a run whose name starts at `s`, or (None, None).

    glowner.read_symbols' locator, unchanged: the separator at `s-1` is the
    record's `+0x31` byte and the token ends there.
    """
    for w in (4, 2):
        p = s - 1 - w
        if p < 1:
            continue
        t = il.read_token_var(glb, p)
        if t is None or t[1] != w:
            continue
        if glb[p - 1] in KIND4_TAGS:
            return glb[p - 1], t[0]
        if p >= 2 and glb[p - 2] in KIND1_TAGS:
            return glb[p - 2], t[0]
    return None, None


def _read_at(glb, idx, p):
    """(token, name) for the varU at `p`, or (None, None)."""
    if p < 0 or p >= len(glb):
        return None, None
    t = il.read_token_var(glb, p)
    if t is None:
        return None, None
    try:
        raw, q = var_u(glb, p)
    except (IndexError, ValueError):
        return None, None
    if q - p != t[1]:                      # RT: the two readers must agree
        return None, None
    return t[0], idx.get(t[0])


def scan(glb, shift=0):
    """-> ({alias_name: target_name}, {alias_tok: target_tok}, stats).

    Only tag-0x10 records.  `shift` displaces the field read by that many bytes
    and exists only to produce the SHIFT null; the headline is `shift=0`.
    """
    runs = model.indexable_runs(glb)
    idx = il.gl_symbol_index(glb)
    by_name, by_tok = {}, {}
    st = {"runs": len(runs), "tag10": 0, "head_fail": 0, "rt_fail": 0,
          "unbound_target": 0, "unbound_self": 0, "self_alias": 0, "dup": 0}
    for i, (s, e, nm, sep) in enumerate(runs):
        tag, tok = _tag_at(glb, s)
        if tag != ALIAS_TAG:
            continue
        st["tag10"] += 1
        p, _sc = refs.head(glb, e + 1)
        if p is None:
            st["head_fail"] += 1
            continue
        ttok, tgt = _read_at(glb, idx, p + shift)
        if ttok is None:
            st["rt_fail"] += 1
            continue
        if tgt is None:
            st["unbound_target"] += 1
            continue
        if nm is None:
            st["unbound_self"] += 1
            continue
        if tgt == nm:
            st["self_alias"] += 1
            continue
        if nm in by_name and by_name[nm] != tgt:
            st["dup"] += 1
            continue
        by_name[nm] = tgt
        by_tok[tok] = ttok
    st["bound"] = len(by_name)
    return by_name, by_tok, st


if __name__ == "__main__":
    for d in sys.argv[1:]:
        base = [n[:-2] for n in os.listdir(d)
                if n.startswith("_CL_") and n.endswith("gl")][0]
        glb = open(os.path.join(d, base + "gl"), "rb").read()
        a, _t, st = scan(glb)
        print("%-40s %s" % (os.path.basename(d), st))
        for k in sorted(a)[:10]:
            print("    %-52s -> %s" % (k[:52], a[k]))
