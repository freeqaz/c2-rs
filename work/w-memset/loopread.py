#!/usr/bin/env python3
"""loopread.py — a crate-free RE-IMPLEMENTATION of `no_effect_loop`, over a
captured `.ex`.

Lane w-memset's **second instrument** (`PREREG.md` P7). It shares no code with
`crates/c2-il` and derives even the formals list independently: the Rust reader
gets them from `parse_formals` over the segment's declaration region, this one
reads the `2D <tok>` run that follows the `46` in the segment header. Two
derivations of one fact is the point; if they disagreed on which tokens are
formals, the counts below would not reconcile.

    loopread.py <file.ex> [--list] [--why]

Prints how many segments match the loop grammar and, with `--why`, a histogram
of the first clause that refused each near-miss (every segment that gets as far
as the `3A <label> 29 <label>` head).
"""
import collections
import sys

FN_START = bytes([0x4F, 0x1F])
LO_MARKER = bytes([0x4C, 0x4F, 0x11])
# `33 <int> 173 40 <int>` — the dead-temporary materialization head.
MEMSET_TEMP = bytes.fromhex("33864174" "80ad000000" "40864174")
INT_TYPE = bytes.fromhex("864174")
CMP_OPS = (0x20, 0x22)


def segments(ex):
    out, i = [], ex.find(FN_START)
    while i >= 0:
        j = ex.find(FN_START, i + 2)
        out.append((len(out), i, ex[i : j if j >= 0 else len(ex)]))
        i = j
    return out


class Cur:
    """A cursor that raises `Refuse(why)` instead of returning None, so the
    first clause to refuse is nameable."""

    class Refuse(Exception):
        pass

    def __init__(self, b, p):
        self.b, self.p = b, p

    def byte(self, v, why):
        if self.p >= len(self.b) or self.b[self.p] != v:
            raise Cur.Refuse(why)
        self.p += 1

    def peek(self):
        return self.b[self.p] if self.p < len(self.b) else None

    def tok(self, why):
        b, p = self.b, self.p
        if p + 1 >= len(b):
            raise Cur.Refuse(why)
        if b[p + 1] & 0x80 == 0:
            self.p += 2
            return (b[p] << 8) | b[p + 1]
        if p + 3 >= len(b):
            raise Cur.Refuse(why)
        self.p += 4
        return (b[p] << 24) | (b[p + 1] << 16) | (b[p + 2] << 8) | b[p + 3]

    def varint(self, why):
        b, p = self.b, self.p
        if p >= len(b):
            raise Cur.Refuse(why)
        if b[p] == 0x80:
            self.p += 5
        else:
            self.p += 1
        return True

    def typ(self, why):
        """`<tag> <kind> <LEB>` — walked only to find its end, which is all the
        Rust reader does with it here too."""
        b, p = self.b, self.p
        if p + 1 >= len(b):
            raise Cur.Refuse(why)
        n = 2
        while p + n < len(b) and b[p + n - 1] & 0x80:
            n += 1
        # A type is at least tag+kind+1; the trailing LEB continues while the
        # high bit is set on the PREVIOUS byte, which is the shape `read_type`
        # produces for `86 41 74`, `86 43 F4 08` and `82 16 86 20` alike.
        while p + n < len(b) and b[p + n - 1] & 0x80:
            n += 1
        self.p += n
        return n

    def marker(self):
        """`4F 01 <line>` and `4F 11`/`4F 02 …` statement markers."""
        while self.p + 1 < len(self.b) and self.b[self.p] == 0x4F and self.b[self.p + 1] == 0x01:
            self.p += 2
            self.p += 5 if self.peek() == 0x80 else 1


def formals(seg):
    """The `2D <tok>` run after the header's `46` — the segment's formals."""
    i = seg.find(bytes([0x53, 0x53, 0x26]))
    if i < 0:
        return None
    c = Cur(seg, i + 3)
    c.tok("own-token")
    if c.peek() != 0x46:
        return None
    c.p += 1
    out = []
    while c.peek() == 0x2D:
        c.p += 1
        out.append(c.tok("formal"))
    return out


def read_type_len(b, p):
    """`read_type`'s width, independently: `86 41 74` is 3, `86 43 F4 08` is 4,
    `82 16 86 20` is 4. The rule is a tag, a kind, then a LEB whose bytes
    continue while the high bit is set."""
    if p + 2 >= len(b):
        raise Cur.Refuse("type")
    n = 2
    while p + n < len(b) and b[p + n] & 0x80:
        n += 1
    return n + 1


def walk_loop(seg):
    """`no_effect_loop`'s grammar. Returns the callee token, or raises Refuse."""
    lo = seg.find(LO_MARKER)
    if lo < 0:
        raise Cur.Refuse("no-body")
    fs = formals(seg)
    if fs is None:
        raise Cur.Refuse("formals")
    c = Cur(seg, lo + 3)
    c.byte(0x53, "body-scope")
    c.marker()
    c.byte(0x53, "loop-scope")
    c.marker()

    def typ(why):
        n = read_type_len(c.b, c.p)
        c.p += n
        return n

    def label(op, why):
        c.byte(op, why)
        return c.tok(why + "-tok")

    l_cond = label(0x3A, "head-goto")
    l_incr = label(0x29, "incr-label")
    c.marker()
    # the induction step
    c.byte(0x26, "induction-lvalue")
    t = c.tok("induction-tok")
    if t not in fs:
        raise Cur.Refuse("induction-not-formal")
    c.byte(0x33, "stride-lit")
    typ("stride-type")
    c.varint("stride-value")
    c.byte(0x0F, "induction-op")
    typ("induction-op-type")
    c.byte(0x4B, "induction-discard")
    c.marker()
    if label(0x29, "cond-label") != l_cond:
        raise Cur.Refuse("cond-label-mismatch")
    for k in (1, 2):
        c.byte(0xB9, f"test-load{k}")
        t = c.tok(f"test-tok{k}")
        if t not in fs:
            raise Cur.Refuse(f"test-not-formal{k}")
        typ(f"test-type{k}")
    if c.peek() not in CMP_OPS:
        raise Cur.Refuse("cmp-op")
    c.p += 1
    l_exit = label(0x38, "exit-branch")
    c.marker()
    # ---- one discarded call statement --------------------------------------
    c.byte(0x26, "callee-push")
    callee = c.tok("callee-tok")
    c.byte(0xBD, "call-token")
    typ("call-ret-type")
    c.varint("call-flags")  # the 1-byte convention field
    c.p -= 1
    c.p += 1
    c.varint("call-fn-type")
    while True:
        c.marker()
        v = c.peek()
        if v == 0x4C:
            c.p += 1
            break
        if v == 0x33:
            if seg[c.p : c.p + len(MEMSET_TEMP)] == MEMSET_TEMP:
                walk_dead_temp(c, typ)
            else:
                c.byte(0x33, "lit")
                typ("lit-type")
                c.varint("lit-value")
                c.byte(0x55, "lit-push")
                typ("lit-push-type")
        elif v == 0xB9:
            c.byte(0xB9, "arg-load")
            t = c.tok("arg-tok")
            if t not in fs:
                raise Cur.Refuse("arg-not-formal")
            typ("arg-type")
            c.byte(0x55, "arg-push")
            typ("arg-push-type")
        else:
            raise Cur.Refuse(f"arg-0x{v:02X}" if v is not None else "arg-eof")
    c.byte(0x4B, "call-discard")
    c.marker()
    if label(0x3A, "continue") != l_incr:
        raise Cur.Refuse("continue-label-mismatch")
    c.marker()
    if label(0x29, "exit-label") != l_exit:
        raise Cur.Refuse("exit-label-mismatch")
    if len({l_cond, l_incr, l_exit}) != 3:
        raise Cur.Refuse("labels-not-distinct")
    c.marker()
    c.byte(0x54, "loop-scope-close")
    c.p += 1
    # the return plumbing, and the fail-closed terminal
    c.marker()
    c.byte(0x3A, "return-goto")
    c.tok("return-goto-tok")
    c.marker()
    c.byte(0x54, "return-scope-close")
    c.p += 1
    c.marker()
    c.byte(0x29, "return-label")
    c.tok("return-label-tok")
    for v in (0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00):
        c.byte(v, "fn-tail")
    rest = seg[c.p :]
    if rest and rest != bytes([0x4D]) and not set(rest) <= {0x00, 0x4D, 0x4F, 0x02, 0x20, 0x01}:
        raise Cur.Refuse("trailing")
    return callee


def walk_dead_temp(c, typ):
    c.p += len(MEMSET_TEMP)
    for _ in range(3):
        c.byte(0x33, "temp-lit")
        typ("temp-lit-type")
        c.varint("temp-lit-value")
        c.byte(0x55, "temp-lit-push")
        typ("temp-lit-push-type")
    for k in (1, 2):
        c.byte(0x9B, f"temp-bind{k}")
        typ(f"temp-bind-type{k}")
        c.tok(f"temp-tok{k}")
        c.byte(0x2C, f"temp-convert{k}")
        typ(f"temp-convert-type{k}")
        c.varint(f"temp-convert-value{k}")
        if k == 1:
            c.byte(0x55, "temp-dest-push")
            typ("temp-dest-push-type")
            c.byte(0x4C, "temp-apply")
        else:
            c.byte(0x44, "temp-bind-op")
            c.byte(0x55, "temp-arg-push")
            typ("temp-arg-push-type")


def main(argv):
    ex = open(argv[1], "rb").read()
    want_list = "--list" in argv
    why = "--why" in argv
    hits, misses = [], collections.Counter()
    for o, off, seg in segments(ex):
        try:
            callee = walk_loop(seg)
        except Cur.Refuse as e:
            misses[str(e)] += 1
            continue
        except (IndexError, ValueError) as e:  # a malformed walk is a refusal
            misses[f"exception:{type(e).__name__}"] += 1
            continue
        hits.append((o, callee))
    print(f"{len(hits)} segments match the destroy-loop grammar")
    if want_list:
        for o, callee in hits:
            print(f"  #{o}\tcallee 0x{callee:x}")
    if why:
        print("---- first clause that refused, by count (near misses first) ----")
        for k, v in misses.most_common(25):
            print(f"  {v:7d}  {k}")


if __name__ == "__main__":
    main(sys.argv)
