#!/usr/bin/env python3
"""refs.py — the `.gl` PER-SYMBOL REFERENCE LIST, decoded from the byte stream.

This is the relation c2's own emit fixpoint runs over.  Both halves are
transcribed instruction by instruction from `c2.dll` 16.00.11886.00 (image base
0x10b00000); nothing here is fitted against any c2 output.

THE READER — `0x10b9bf99` .. `0x10b9c007`, the tail of the tag-0x0E arm of the
shared `.gl` record handler at `0x10b9bdcf`:

    10b9bf99  f7 46 4c 00 10 00 00   test DWORD [esi+0x4c],0x1000  <- the gate
    10b9bfa0  74 67                  je   no-list
    10b9bfa4  call 10b27db2                                 list head for esi
    10b9bfa9  cmp  ds:0x10c6d070,0
    10b9bfb5  call i32c        (if set)                     COUNT, wide form
    10b9bfbc  call i16c ; movzx eax,ax   (if clear)         COUNT, narrow form
    10b9bfc7  test eax,eax ; je no-list
  loop:
    10b9bfce  call varU        -> [ebp-0x7c]                THE TOKEN
    10b9bfd6  call i16c ; movzx ebx,ax                      THE USE COUNT
    10b9bfde  test bx,bx ; je  skip-the-alloc     <<< refcount 0 IS NOT AN EDGE
    10b9bfe9  call 10c2022a (alloc 0x1b/0xc)
    10b9bff1  mov [node+0x4],token ; mov [node+0x8],count
    10b9bffb  node->next = head[0x14] ; head[0x14] = node
    10b9c003  cmp count,0 ; jne loop

Two consequences worth stating, because they are semantics and not layout:

  * the list is gated on **`flags4c & 0x1000`** — the same word whose `0x20` bit
    w-roots showed is the emit SEED (`_2026-08-04-w-roots-findings.md` §3);
  * an entry whose **use count is zero is parsed and then dropped on the floor**
    (`test bx,bx / je`), so it is not an edge.

THE WALKER — `0x10b276e4`, which is what makes this list the emit relation:

    Mark(sym, edx):
      if (sym[0x4c] & 0x20) return;              already marked -> stop
      if (ds:0x10c462c4 && edx == 0) return;
      sym[0x4c] |= 0x20;                         MARK: the seed bit itself
      ecx = sym[0x80]; if (!ecx) return;         the reference list
      for (node = ecx[0xc]; node; node = node[0]):
          tgt = node[4][4]
          if (tgt[0x37] & 0x400) continue;       storage class 0xa -> SKIPPED
          if (tgt[0x4c] & 0x20) continue;        already marked
          Mark(tgt, edi)

so c2's emit set is the least fixpoint of `flags4c |= 0x20` over exactly this
list, and `p2/main.c`'s walk loop at `10b7f16b` then compiles every symbol whose
`0x20` survived.  `+0x37 & 0x400` is set at `10b9be44` for storage-class nibble
`0xa` (`and ecx,0xfffffe9f / or ecx,0x480`), where the nibble is bits 0..3 of the
GetByte at `10b9be0e` (`movsx al / shl 5 / xor +0x37 / and 0x1e0 / xor`).

THE KNOWN-ANSWER GATE.  A decoded list must end **exactly** where the next
record's header begins: `<tag> <varU token> <GetByte 0x00|0x26> <name>`.  A wrong
count width, a wrong pair layout or a missed escape lands somewhere else, so the
gate is not satisfiable by accident — it is checked per record and reported as a
count, never as a status.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "emitpred", "pipeline"))
sys.path.insert(0, os.path.join(HERE, "..", "w-roots"))
import il      # noqa: E402
import model   # noqa: E402
from glflags import var_u, i16c, i32c, get_byte   # noqa: E402
from chain import skipvar, blob, i64c             # noqa: E402

LIST_BIT = 0x1000
SEED_BIT = 0x20
DONE_BIT = 0x02
EXTERN_CLASS = 0xA          # +0x37 & 0x400 <- storage-class nibble 0xa
NAME_SEPARATORS = (0x00, 0x26)


def head(b, p):
    """Name-NUL+1 -> (position of the +0x54 anchor field, storage-class byte).

    w-roots' `record.head`, unchanged, plus the storage-class byte the walker's
    `0x400` skip is derived from.  Returns (None, None) on desync.
    """
    try:
        sc, p = get_byte(b, p)                    # 10b9be0e storage class
        _, p = i32c(b, p)                         # +0x40
        _, p = var_u(b, p)                        # +0x20 flags
        _, p = var_u(b, p)                        # +0x0c owner (unconditional)
        optw, p = i32c(b, p)
        if optw & 1:
            _, p = i32c(b, p)
            n, p = i32c(b, p)
            for _ in range(max(0, n)):
                p = skipvar(b, p)
        if optw & 2:
            _, p = var_u(b, p)
        _, p = i32c(b, p)                         # +0x2c type index
        _, p = i32c(b, p)                         # debug
        m, p = i32c(b, p)
        for _ in range(max(0, m)):
            _, p = i32c(b, p)
            p = skipvar(b, p)
            k, p = i32c(b, p)
            for _ in range(max(0, k)):
                _, p = i32c(b, p)
                c, p = get_byte(b, p)
                p = blob(b, p) if c else i64c(b, p)
        return p, sc
    except (IndexError, ValueError):
        return None, None


def tail(b, o):
    """From the anchor at `o`: (flags4c, flag_pos, flag_width, pos_after_+0x52)."""
    _, p = i32c(b, o)          # +0x54
    _, p = i32c(b, p)          # +0x58
    _, p = i16c(b, p)          # +0x50
    fp = p
    raw, p = var_u(b, p)       # +0x4c
    fw = p - fp
    _, p = i16c(b, p)          # +0x52
    return (raw & ~0x4), fp, fw, p


def reflist(b, p, wide_count):
    """10b9bf99..10b9c007.  From just past the `+0x52` i16c.

    Returns (pairs, end, cpos) with pairs = [(token, refcount, tokpos)] in
    stream order, `end` the first byte past the list, `cpos` the position of the
    count field (or None when there is none).  Raises on a truncated stream.
    """
    n, _ = (i32c(b, p) if wide_count else i16c(b, p))
    cpos = p
    _, p = (i32c(b, p) if wide_count else i16c(b, p))
    if not wide_count:
        n &= 0xFFFF                                 # movzx eax,ax
    out = []
    for _ in range(max(0, n)):
        tp = p
        t = il.read_token_var(b, p)
        if t is None:
            raise IndexError("token")
        _, p = var_u(b, p)                          # same width, same bytes
        cnt, p = i16c(b, p)
        out.append((t[0], cnt & 0xFFFF, tp))
    return out, p, cpos


def _next_header_ok(b, q, s_next):
    """The terminus gate: `q` must be the next record's tag byte, i.e.
    <tag><varU token><0x00|0x26><name at s_next>."""
    if s_next is None or q >= len(b) - 3:
        return False
    t = il.read_token_var(b, q + 1)
    if t is None:
        return False
    sep = q + 1 + t[1]
    return sep + 1 == s_next and b[sep] in NAME_SEPARATORS


def scan(glb, exb, wide_count=False, apply_extern_skip=True):
    """{name: rec} for every gate-clean tag-0x0E record, plus stats.

    `rec` carries `flags`, `ex` (the `.ex` body start), `seed`, `sclass`,
    `refs` (the decoded token list) and `term` (terminus gate verdict).
    Duplicate `.ex` offsets are dropped fail-closed, as in w-roots.
    """
    import struct
    starts = set(il.split_ex(exb))
    runs = model.indexable_runs(glb)
    n = len(glb)
    hits = []
    st = {"runs": len(runs), "recs": 0, "dup_ex": 0, "list_bit": 0,
          "term_ok": 0, "term_bad": 0, "term_ok_nolist": 0, "term_bad_nolist": 0,
          "pairs": 0, "pairs_zero": 0, "wide_discriminating": 0,
          "extern_class": 0, "ex_segments": len(starts)}
    for i, (s, e, nm, sep) in enumerate(runs):
        s_next = runs[i + 1][0] if i + 1 < len(runs) else None
        p, sc = head(glb, e + 1)
        if p is None or p + 5 > n or glb[p] != 0x80:
            continue
        v = struct.unpack_from("<I", glb, p + 1)[0]
        if v not in starts:
            continue
        flags, fp, fw, q = tail(glb, p)
        st["recs"] += 1
        pairs = []
        cpos = None
        if flags & LIST_BIT:
            st["list_bit"] += 1
            try:
                pairs, q, cpos = reflist(glb, q, wide_count)
            except (IndexError, ValueError):
                q = None
        ok = _next_header_ok(glb, q, s_next) if q is not None else False
        if flags & LIST_BIT:
            st["term_ok" if ok else "term_bad"] += 1
            if cpos is not None and glb[cpos] == 0x80:
                st["wide_discriminating"] += 1
        else:
            st["term_ok_nolist" if ok else "term_bad_nolist"] += 1
        st["pairs"] += len(pairs)
        st["pairs_zero"] += sum(1 for _, c, _ in pairs if c == 0)
        if (sc & 0xF) == EXTERN_CLASS:
            st["extern_class"] += 1
        hits.append((nm, flags, fp, fw, v, sc, pairs, ok))
    seen = {}
    for h in hits:
        seen.setdefault(h[4], []).append(h)
    out = {}
    for v, lst in seen.items():
        if len(lst) != 1:
            st["dup_ex"] += 1
            continue
        nm, flags, fp, fw, v, sc, pairs, ok = lst[0]
        out[nm] = {"flags": flags, "fpos": fp, "fwidth": fw, "ex": v,
                   "sclass": sc, "refs": pairs, "term": ok,
                   "skip": apply_extern_skip and (sc & 0xF) == EXTERN_CLASS,
                   "seed": bool(flags & SEED_BIT) and not (flags & DONE_BIT)}
    st["bound"] = len(out)
    return out, st


def edges(glb, recs, U, drop_zero=True):
    """{owner: {referenced names}} over the decoded lists, restricted to `U`."""
    idx = il.gl_symbol_index(glb)
    get = idx.get
    out = {}
    for nm, r in recs.items():
        if not r["refs"]:
            continue
        acc = out.setdefault(nm, set())
        for tok, cnt, _ in r["refs"]:
            if drop_zero and cnt == 0:
                continue
            f = get(tok)
            if f is not None and f != nm and f in U:
                acc.add(f)
    return out
