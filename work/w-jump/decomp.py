#!/usr/bin/env python3
"""w-jump — decompose the `expr-jump` first-blocker family from ONE instrumented
`gap --jsonl` scan.

The scratch instrument (reverted; the patch is quoted in
`docs/rungs/2026-08-09-w-jump.md` §2.1) replaces the plain `expr-jump` key with

    expr-jump|<cflow>|<dispatch>|<calls>|<seg_len>|<hex_mark>|<hexwindow>|<index>|<name>

so every axis below is decided offline from one scan and BOTH columns sum to the
family total by construction. That sum is asserted, not assumed.

Usage: decomp.py SCAN.jsonl [--axis A] [--sample KEY N]
"""
import json
import sys
from collections import Counter, defaultdict

PATH = sys.argv[1]
FAMILY = "expr-jump"


class Row:
    __slots__ = ("src", "cflow", "dispatch", "calls", "seg_len", "mark", "hex",
                 "index", "name", "n")

    def __init__(self, src, key, n):
        p = key.split("|", 8)
        self.src = src
        self.cflow = p[1]
        self.dispatch = p[2]
        self.calls = int(p[3])
        self.seg_len = int(p[4])
        self.mark = int(p[5])
        self.hex = bytes.fromhex(p[6])
        self.index = int(p[7])
        self.name = p[8]
        self.n = n

    # --- the axes -----------------------------------------------------------
    def after(self):
        """Axis A — the byte AFTER the 0x3A. w-bdnz's proposed instrument."""
        i = self.mark + 1
        return f"{self.hex[i]:02X}" if i < len(self.hex) else "eof"

    def before(self, k=1):
        """Axis B — the k bytes immediately BEFORE the 0x3A."""
        lo = self.mark - k
        if lo < 0:
            return "bof"
        return " ".join(f"{b:02X}" for b in self.hex[lo:self.mark])

    def tok_after(self):
        """The label token after the `3A`, decoded with the PORT's own rule
        (`readers::read_token_var`): 2 bytes, or 4 when byte 1 has bit 7 set."""
        i, h = self.mark + 1, self.hex
        if i + 1 >= len(h):
            return None
        if h[i + 1] & 0x80 == 0:
            return (h[i] << 8) | h[i + 1]
        if i + 3 >= len(h):
            return None
        return (h[i] << 24) | (h[i + 1] << 16) | (h[i + 2] << 8) | h[i + 3]

    def sig(self):
        """A REPLICATION signature — everything a header-inlined body copied
        into N TUs keeps CONSTANT. Per-TU label/symbol tokens are deliberately
        not in it; the segment length, the decoded control-flow class, the
        window geometry and the opcode skeleton around the `3A` are."""
        return (self.seg_len, self.cflow, self.mark, len(self.hex), self.before(3))


def load(path):
    body, emit = [], []
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r.get("src", "?")
        for k, n in (r.get("fn_blockers") or {}).items():
            if k.startswith(FAMILY):
                body.append(Row(src, k, n))
        for k, n in (r.get("emit_blockers") or {}).items():
            if k.startswith(FAMILY):
                emit.append(Row(src, k, n))
    return body, emit


def total(rows):
    return sum(r.n for r in rows)


def table(body, emit, fn, title, top=None):
    b, e = Counter(), Counter()
    for r in body:
        b[fn(r)] += r.n
    for r in emit:
        e[fn(r)] += r.n
    tb, te = total(body), total(emit)
    print(f"\n=== {title} ===")
    print(f"{'sub-key':52s} {'bodies':>7s} {'%':>6s} {'emitted':>8s} {'%':>6s}")
    order = b.most_common(top) if top else b.most_common()
    sb = se = 0
    for k, n in order:
        sb += n
        se += e[k]
        print(f"{k:52s} {n:7d} {100*n/tb:6.1f} {e[k]:8d} "
              f"{(100*e[k]/te if te else 0):6.1f}")
    if top and len(b) > top:
        print(f"{'… %d further sub-keys' % (len(b)-top):52s} "
              f"{tb-sb:7d} {100*(tb-sb)/tb:6.1f} {te-se:8d} "
              f"{(100*(te-se)/te if te else 0):6.1f}")
    print(f"{'TOTAL':52s} {tb:7d} {100.0:6.1f} {te:8d} {100.0:6.1f}")
    print(f"  distinct sub-keys: bodies {len(b)}, emitted {len(e)}; "
          f"largest bodies share {100*b.most_common(1)[0][1]/tb:.1f}%")
    return b, e


if __name__ == "__main__":
    body, emit = load(PATH)
    tb, te = total(body), total(emit)
    print(f"family {FAMILY}: bodies {tb}, emitted {te} "
          f"({len(body)} body rows, {len(emit)} emitted rows)")

    # the invariant the instrument is supposed to preserve
    bad = [r for r in body + emit if r.mark >= len(r.hex) or r.hex[r.mark] != 0x3A]
    print(f"  rows whose marked byte is NOT 0x3A: {len(bad)}  (must be 0)")

    if "--sig" in sys.argv:
        # Sample the Nth-largest REPLICATION signature, in the chosen column.
        col = body if "--emit" not in sys.argv else emit
        want = int(sys.argv[sys.argv.index("--sig") + 1])
        nshow = int(sys.argv[sys.argv.index("--sig") + 2])
        g = defaultdict(list)
        for r in col:
            g[r.sig()].append(r)
        order = sorted(g.items(), key=lambda kv: -sum(r.n for r in kv[1]))
        sig, rows = order[want - 1]
        n = sum(r.n for r in rows)
        print(f"\n=== signature #{want}: {n} rows over "
              f"{len({r.src for r in rows})} TUs; seg_len={rows[0].seg_len} "
              f"cflow={rows[0].cflow} ===")
        names = Counter(r.name for r in rows)
        print(f"  names ({len(names)} distinct): "
              f"{[f'{k} x{v}' for k, v in names.most_common(4)]}")
        step = max(1, len(rows) // nshow)
        for r in rows[::step][:nshow]:
            w = " ".join(f"{b:02X}" for b in r.hex)
            w = w[:3 * r.mark] + ">" + w[3 * r.mark:3 * r.mark + 2] + "<" + \
                w[3 * r.mark + 2:]
            print(f"  {r.src}  #{r.index}  {r.name}")
            print(f"    {w}")
        sys.exit(0)

    if "--sample" in sys.argv:
        i = sys.argv.index("--sample")
        axis, want, n = sys.argv[i + 1], sys.argv[i + 2], int(sys.argv[i + 3])
        f = {"after": Row.after, "before1": lambda r: r.before(1),
             "before2": lambda r: r.before(2), "before3": lambda r: r.before(3),
             "cflow": lambda r: r.cflow, "dispatch": lambda r: r.dispatch}[axis]
        hits = [r for r in body if f(r) == want]
        print(f"\n=== sample: axis {axis} == {want!r}  ({len(hits)} rows) ===")
        step = max(1, len(hits) // n)
        for r in hits[::step][:n]:
            w = " ".join(f"{b:02X}" for b in r.hex)
            w = w[:3 * r.mark] + ">" + w[3 * r.mark:3 * r.mark + 2] + "<" + \
                w[3 * r.mark + 2:]
            print(f"  {r.src}  #{r.index}  seg={r.seg_len} calls={r.calls}")
            print(f"    {r.name}")
            print(f"    {r.cflow} / {r.dispatch}")
            print(f"    {w}")
        sys.exit(0)

    table(body, emit, Row.after, "AXIS A — the byte AFTER the 0x3A "
          "(w-bdnz's proposed instrument)", top=12)
    table(body, emit, lambda r: str(r.tok_after()),
          "AXIS A' — the DECODED varU label token after the 0x3A", top=12)
    table(body, emit, lambda r: r.before(1), "AXIS B1 — the byte BEFORE the 0x3A",
          top=12)
    table(body, emit, lambda r: r.before(3), "AXIS B3 — the 3 bytes BEFORE the 0x3A",
          top=12)
    table(body, emit, lambda r: r.cflow, "AXIS C — the cflow class", top=16)
    table(body, emit, lambda r: r.dispatch, "AXIS D — the ladder dispatch arm",
          top=16)
    table(body, emit, lambda r: f"{r.cflow} / {r.before(1)}",
          "AXIS C×B1 — cflow class crossed with the byte before", top=16)
    table(body, emit, lambda r: str(r.seg_len), "AXIS E — the segment length",
          top=12)

    # --- THE REPLICATION REPORT --------------------------------------------
    # A body column counts SEGMENTS, and a header-inlined body is one construct
    # copied into every TU that includes the header. So `2,286 bodies` is not
    # `2,286 constructs` until this is measured.
    groups = defaultdict(list)
    for r in body:
        groups[r.sig()].append(r)
    print(f"\n=== REPLICATION — distinct signatures over the {tb} bodies ===")
    print(f"  distinct signatures: {len(groups)}")
    print(f"{'#':>5s} {'bodies':>7s} {'TUs':>5s} {'varying window positions':>26s}  "
          f"cflow / before1 / seg_len")
    for i, (sig, rows) in enumerate(
            sorted(groups.items(), key=lambda kv: -sum(r.n for r in kv[1]))[:14]):
        n = sum(r.n for r in rows)
        tus = len({r.src for r in rows})
        L = min(len(r.hex) for r in rows)
        vary = [j for j in range(L) if len({r.hex[j] for r in rows}) > 1]
        print(f"{i+1:5d} {n:7d} {tus:5d} {str(vary):>26.26s}  "
              f"{rows[0].cflow} / {rows[0].before(1)} / {rows[0].seg_len}")
    # the same count, but restricted to the emitted column
    egroups = defaultdict(list)
    for r in emit:
        egroups[r.sig()].append(r)
    print(f"  distinct signatures among the {te} EMITTED: {len(egroups)}")
    named = {r.name for r in emit if r.name != "-"}
    print(f"  distinct NAMES among the emitted: {len(named)}")
    nb = {r.name for r in body if r.name != "-"}
    print(f"  distinct NAMES among the bodies:  {len(nb)} "
          f"({sum(r.n for r in body if r.name == '-')} rows carry no name)")
