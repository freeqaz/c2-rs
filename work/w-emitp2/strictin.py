#!/usr/bin/env python3
"""strictin.py — the `.in` stream read to **w-tag02's measured grammar**, and a
second reading restricted to what **the shipping reader** can see.

Three readers, one stream, so the lane can say which of them the emit-predicate
channel has actually been running on.

**STRICT** — `work/w-tag02/GRAMMAR.md` and
`crates/c2-il/src/func/ininit.rs::read_offset`:

    02 <varU token> <offset: byte 00..7F, else 80 + LE32> <n == 04>

**INSTREAM** — `work/w-mark/instream.py`, which every emit-predicate lane's
`02`-node channel has run on since w-mark:

    02 <varU token> <i32c addend> <i32c width>

`i32c` (`0x10c1f9e9`) is a *signed byte, or `0x80` escape then LE32*.  The two
consume **identical bytes** on every offset in `00..7F` and on every `80`
escape.  They can differ on exactly two things and this file counts both rather
than arguing about them: an offset short form in **`0x81..0xFF`** (instream
reads a negative one-byte offset; the strict grammar calls it a desync, because
every measured negative offset ESCAPES — `-4` is `80 fc ff ff ff`), and a
trailing `<n>` other than `04`.

**CRATE** — the acceptance `crates/c2-il/src/func/ininit.rs` applies.  The
shipping reader is an **anchor scan**, not a sequential record parser: it starts
a record only at `00 01` or `00 02`, and `read_elements` returns `Err` —
discarding the **whole** record's symbol addresses — the moment it meets an
element tag that is not `01`/`02`, an element type outside `01`/`02`/`05`, or a
width outside `{1, 2, 4}`.  So a record whose FIRST element is a tag-`03` blob or
a tag-`08` zero fill is never anchored at all (it is in neither `records` nor
the residue), and a record that meets one MIDWAY contributes nothing.  This
reader reproduces that acceptance on top of the sequential framing, so the two
line up record for record.  **It is a transcription and is graded as one** —
`two_readers.py` checks its counts against the crate's own cursor.

Everything else — the record framing, the tag-01 scalar, the tag-03 blob and the
tag-08 zero fill — is `instream`'s, by value, because changing two things at once
measures neither.

**Nothing here reads any c2 output.**  The grammar is transcribed from
`work/w-tag02/GRAMMAR.md` and `ininit.rs`, both of which predate this lane.

stdlib only.
"""
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT",
                      os.path.abspath(os.path.join(HERE, "..", "..")))
for _p in (os.path.join(MAIN, "work", "emitpred", "pipeline"),
           os.path.join(MAIN, "work", "w-roots"),
           os.path.join(MAIN, "work", "w-mark")):
    sys.path.insert(0, os.path.abspath(_p))
import il                      # noqa: E402
import instream                # noqa: E402
from glflags import i16c, i32c  # noqa: E402
from chain import i64c         # noqa: E402

REC_TAGS = instream.REC_TAGS
SYM = 0x02
ADDRESS_WIDTH = 0x04
CRATE_TYPES = (0x01, 0x02)
CRATE_WIDTHS = (1, 2, 4)


class Desync(Exception):
    """A byte the measured grammar does not admit.  Never a silent skip."""

    def __init__(self, why):
        Exception.__init__(self, why)
        self.why = why


def read_offset(b, p):
    """`ininit.rs::read_offset`, transcribed.  Short form is `00..7F` ONLY."""
    b0 = b[p]
    if b0 < 0x80:
        return b0, p + 1
    if b0 != 0x80:
        raise Desync("offset-high-bit")
    return struct.unpack_from("<i", b, p + 1)[0], p + 5


def node(b, p, elems, st):
    """One element, appended to `elems` as a describing tuple."""
    k = b[p]
    p += 1
    st["elem"] += 1
    if k == 0x01:
        t, p = i32c(b, p)
        w, p = i32c(b, p)
        st["e01"] += 1
        if t == 5:
            if w not in (4, 8) or p + w > len(b):
                raise Desync("fp-width")
            elems.append((0x01, t, w))
            return p + w
        if w == 2:
            _, p = i16c(b, p)
        elif w in (1, 4):
            _, p = i32c(b, p)
        elif w == 8:
            _, p = i64c(b, p)
        else:
            raise Desync("scalar-width")
        elems.append((0x01, t, w))
        return p
    if k == SYM:
        st["e02"] += 1
        tok, p = instream.var_u_be(b, p)
        if 0x80 < b[p] <= 0xFF:
            st["off_hi"] += 1          # THE ONLY DISAGREEMENT WITH instream
        off, p = read_offset(b, p)
        if off > 0x7F or off < 0:
            st["off_esc"] += 1
        if b[p] != ADDRESS_WIDTH:
            st["n_not_04"] += 1
            raise Desync("n!=04")
        p += 1
        elems.append((SYM, tok, 0))
        return p
    if k == 0x03:
        st["e03"] += 1
        n, p = i16c(b, p)
        if n < 0 or p + n > len(b):
            raise Desync("blob-len")
        elems.append((0x03, 0, n))
        return p + n
    if k == 0x08:
        st["e08"] += 1
        _, p = i32c(b, p)
        elems.append((0x08, 0, 0))
        return p
    raise Desync("element-tag-%02x" % k)


def parse_ex(data):
    """-> (clean, [(tag, flagbyte, owner, [element...])], stats)."""
    recs = []
    st = {"rec": 0, "elem": 0, "e01": 0, "e02": 0, "e03": 0, "e08": 0,
          "off_hi": 0, "off_esc": 0, "n_not_04": 0, "why": ""}
    p = 0
    n = len(data)
    clean = False
    try:
        while p < n:
            if p == n - 1 and data[p] == 0x07:
                clean = True
                break
            tag = data[p]
            if tag not in REC_TAGS:
                st["why"] = "record-tag-%02x" % tag
                break
            q = p + 1
            fl = None
            if tag == 0x07:
                fl = data[q]
                q += 1
            owner, q = instream.var_u_be(data, q)
            _, q = i32c(data, q)
            elems = []
            while q < n and data[q] not in REC_TAGS:
                q = node(data, q, elems, st)
            recs.append((tag, fl, owner, elems))
            st["rec"] += 1
            p = q
        else:
            clean = True
    except Desync as ex:
        st["why"] = ex.why
    except (IndexError, ValueError, struct.error) as ex:
        st["why"] = "raw:%s" % type(ex).__name__
    return clean, recs, st


def _crate_verdict(elems):
    """(accepted, first_tag, why) under `ininit.rs`'s acceptance."""
    if not elems:
        return False, None, "empty"
    first = elems[0][0]
    if first not in (0x01, SYM):
        return False, first, "UNANCHORED"
    for k, a, w in elems:
        if k == SYM:
            continue
        if k != 0x01:
            return False, first, "unknown-type"
        if a == 0x05:
            return False, first, "floating-point"
        if a not in CRATE_TYPES:
            return False, first, "unknown-type"
        if w not in CRATE_WIDTHS:
            return False, first, "unknown-width"
    return True, first, None


def parse_records(data):
    """`marks.parse_records`'s signature under the STRICT grammar, so it drops
    in as the node source: -> (clean, [(tag, flagbyte, owner, [tok...])])."""
    clean, recs, st = parse_ex(data)
    parse_records.last = st
    return (clean, [(t, f, o, [e[1] for e in el if e[0] == SYM])
                    for t, f, o, el in recs])


parse_records.last = None


def parse_records_crate(data):
    """`marks.parse_records`'s signature restricted to what the SHIPPING reader
    can see.  A refused record contributes NO symbol addresses, exactly as
    `read_elements`' `Err` arm discards the whole `refs` vector."""
    clean, recs, st = parse_ex(data)
    st = dict(st)
    st.update({"c_rec": 0, "c_unanchored": 0, "c_refused": 0, "c_accepted": 0,
               "c_e02": 0, "c_e02_unanchored": 0, "c_e02_refused": 0,
               "c_rec_with_sym": 0, "c_elem": 0, "c_failclosed": 0,
               "c_e02_failclosed": 0})
    why = {}
    out = []
    for t, f, o, el in recs:
        ok, first, w = _crate_verdict(el)
        nsym = sum(1 for e in el if e[0] == SYM)
        if ok:
            st["c_rec"] += 1
            st["c_accepted"] += 1
            st["c_elem"] += len(el)
            st["c_e02"] += nsym
            if nsym:
                st["c_rec_with_sym"] += 1
            out.append((t, f, o, [e[1] for e in el if e[0] == SYM]))
            continue
        if w == "UNANCHORED":
            st["c_unanchored"] += 1
            st["c_e02_unanchored"] += nsym
        elif first == SYM:
            # THE FAIL-CLOSED ARM, `ininit.rs`: a `00 02` candidate that does
            # not frame all the way to its `07` is "not a record" — it is in
            # NEITHER `records` NOR the residue, and the scan resumes one byte
            # on. Counted here separately so the silence is visible.
            st["c_failclosed"] = st.get("c_failclosed", 0) + 1
            st["c_e02_failclosed"] = st.get("c_e02_failclosed", 0) + nsym
        else:
            st["c_rec"] += 1
            st["c_refused"] += 1
            st["c_e02_refused"] += nsym
            why[w] = why.get(w, 0) + 1
        out.append((t, f, o, []))
    st["c_why"] = why
    parse_records_crate.last = st
    return (clean, out)


parse_records_crate.last = None


def counters(data):
    """(clean, strict-records, strict-stats, crate-records, crate-stats)."""
    cl, rs = parse_records(data)
    sts = dict(parse_records.last)
    _cl2, rc = parse_records_crate(data)
    stc = dict(parse_records_crate.last)
    return cl, rs, sts, rc, stc


if __name__ == "__main__":
    for d in sys.argv[1:]:
        base = None
        for nm in os.listdir(d):
            if nm.startswith("_CL_") and nm.endswith("in"):
                base = nm
        b = open(os.path.join(d, base), "rb").read()
        cl, rs, sts, rc, stc = counters(b)
        ci, ri = instream.parse(b)
        print("%-34s strict clean=%s recs=%d tag02=%d off_hi=%d off_esc=%d "
              "n_not_04=%d why=%s | crate recs=%d tag02=%d unanchored=%d "
              "refused=%d | instream clean=%s recs=%d"
              % (os.path.basename(d), cl, len(rs), sts["e02"], sts["off_hi"],
                 sts["off_esc"], sts["n_not_04"], sts["why"] or "-",
                 stc["c_rec"], stc["c_e02"], stc["c_unanchored"],
                 stc["c_refused"], ci, len(ri)))
