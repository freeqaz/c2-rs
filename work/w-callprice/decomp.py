#!/usr/bin/env python3
"""w-callprice — decompose `expr-call-in-expr-*` BY THE EMITTED COLUMN, with the
TU-replication discount, from ONE instrumented `gap --jsonl` scan.

The scratch instrument (reverted; the patch is quoted in
`docs/rungs/2026-08-09-w-callprice.md` §2) replaces the family's plain key with

    <key>|<cflow>|<dispatch>|<prod>|<calls>|<seg_len>|<hex_mark>|<hex>|<index>|<name>

so every axis below is decided offline from one scan, and BOTH columns sum to
the family total by construction. **That sum is asserted, not assumed.**

Usage:
  decomp.py SCAN.jsonl [--base BASE.jsonl] [--top N]
  decomp.py SCAN.jsonl --sample KEY N [--emit]
  decomp.py SCAN.jsonl --names KEY N [--emit]
"""
import json
import sys
from collections import Counter, defaultdict

FAMILY = "expr-call-in-expr"


class Row:
    __slots__ = ("src", "key", "cflow", "dispatch", "prod", "calls", "seg_len",
                 "mark", "hex", "index", "name", "n")

    def __init__(self, src, raw, n):
        p = raw.split("|", 9)
        self.src = src
        self.key = p[0]
        self.cflow = p[1]
        self.dispatch = p[2]
        self.prod = p[3]
        self.calls = int(p[4])
        self.seg_len = int(p[5])
        self.mark = int(p[6])
        self.hex = bytes.fromhex(p[7])
        self.index = int(p[8])
        self.name = p[9]
        self.n = n

    def sig(self):
        """A REPLICATION signature — everything a header-inlined body copied
        into N TUs keeps CONSTANT. Per-TU symbol/label tokens are deliberately
        not in it; the key, the segment length, the decoded axes and the window
        geometry are."""
        return (self.key, self.seg_len, self.cflow, self.dispatch, self.prod,
                self.calls, self.mark, len(self.hex))

    def window(self):
        w = " ".join(f"{b:02X}" for b in self.hex)
        i = 3 * self.mark
        return w[:i] + ">" + w[i:i + 2] + "<" + w[i + 2:]


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


def base_totals(path):
    fb = fe = 0
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        for k, n in (r.get("fn_blockers") or {}).items():
            if k.startswith(FAMILY):
                fb += n
        for k, n in (r.get("emit_blockers") or {}).items():
            if k.startswith(FAMILY):
                fe += n
    return fb, fe


def total(rows):
    return sum(r.n for r in rows)


def axis(body, emit, fn, title, top=None):
    b, e = Counter(), Counter()
    for r in body:
        b[fn(r)] += r.n
    for r in emit:
        e[fn(r)] += r.n
    tb, te = total(body), total(emit)
    print(f"\n=== {title} ===")
    print(f"{'sub-key':56s} {'emitted':>8s} {'%':>6s} {'bodies':>9s} {'%':>6s} {'em/1k':>6s}")
    order = e.most_common(top) if top else e.most_common()
    sb = se = 0
    for k, n in order:
        se += n
        sb += b[k]
        print(f"{k:56s} {n:8d} {100*n/te:6.2f} {b[k]:9d} {100*b[k]/tb:6.2f} "
              f"{(1000*n/b[k] if b[k] else 0):6.1f}")
    if top and len(e) > top:
        print(f"{'… %d further' % (len(e)-top):56s} {te-se:8d} {100*(te-se)/te:6.2f} "
              f"{tb-sb:9d} {100*(tb-sb)/tb:6.2f}")
    print(f"{'TOTAL':56s} {te:8d} {100.0:6.2f} {tb:9d} {100.0:6.2f}")
    return b, e


def replication(rows, keyfn, title, top=20):
    """THE DELIVERABLE'S DISCOUNT. For each group: the raw count, the distinct
    TUs it spans, the distinct mangled NAMES in it, and the multiplicity of its
    largest name. `count == TUs == 1 name` is one construct replicated, not N."""
    g = defaultdict(list)
    for r in rows:
        g[keyfn(r)].append(r)
    tot = total(rows)
    print(f"\n=== {title} ===")
    print(f"{'#':>3s} {'count':>7s} {'%':>6s} {'TUs':>5s} {'names':>6s} "
          f"{'sigs':>5s} {'topname x':>9s} {'DISCOUNTED':>10s}  group")
    order = sorted(g.items(), key=lambda kv: -total(kv[1]))
    disc_tot = 0
    for i, (k, rows_) in enumerate(order):
        n = total(rows_)
        names = Counter(r.name for r in rows_ if r.name != "-")
        nn = len(names)
        disc = nn if nn else n
        disc_tot += disc
        if i < top:
            tus = len({r.src for r in rows_})
            sigs = len({r.sig() for r in rows_})
            topn = names.most_common(1)[0][1] if names else 0
            print(f"{i+1:3d} {n:7d} {100*n/tot:6.2f} {tus:5d} {nn:6d} {sigs:5d} "
                  f"{topn:9d} {disc:10d}  {k[:70]}")
    print(f"    total {tot}, DISCOUNTED total (distinct names per group) {disc_tot}")
    return g


if __name__ == "__main__":
    path = sys.argv[1]
    body, emit = load(path)
    tb, te = total(body), total(emit)
    print(f"family {FAMILY}: bodies {tb}, emitted {te} "
          f"({len(body)} body rows, {len(emit)} emitted rows)")

    if "--base" in sys.argv:
        fb, fe = base_totals(sys.argv[sys.argv.index("--base") + 1])
        print(f"BASE scan (un-instrumented): bodies {fb}, emitted {fe}")
        assert (fb, fe) == (tb, te), (
            f"INSTRUMENT CHANGED THE FAMILY TOTAL: base {(fb, fe)} vs "
            f"instrumented {(tb, te)} — every table below is void (PREREG P2/§4)")
        print("  ASSERTED: the compound key preserves both columns exactly.")

    if "--sample" in sys.argv:
        i = sys.argv.index("--sample")
        want, nshow = sys.argv[i + 1], int(sys.argv[i + 2])
        col = emit if "--emit" in sys.argv else body
        hits = [r for r in col if r.key == want]
        print(f"\n=== sample {want} ({'emitted' if '--emit' in sys.argv else 'bodies'}): "
              f"{total(hits)} over {len({r.src for r in hits})} TUs, "
              f"{len({r.name for r in hits if r.name != '-'})} names ===")
        hits.sort(key=lambda r: r.seg_len)
        step = max(1, len(hits) // nshow)
        for r in hits[::step][:nshow]:
            print(f"  {r.src}  #{r.index}  seg={r.seg_len} calls={r.calls}")
            print(f"    {r.name}")
            print(f"    {r.cflow} / {r.dispatch} / {r.prod}")
            print(f"    {r.window()}")
        sys.exit(0)

    if "--names" in sys.argv:
        i = sys.argv.index("--names")
        want, nshow = sys.argv[i + 1], int(sys.argv[i + 2])
        col = emit if "--emit" in sys.argv else body
        hits = [r for r in col if r.key == want]
        names = Counter(r.name for r in hits for _ in range(r.n))
        print(f"\n=== names for {want}: {total(hits)} rows, {len(names)} distinct ===")
        for nm, c in names.most_common(nshow):
            tus = len({r.src for r in hits if r.name == nm})
            print(f"  {c:6d} in {tus:4d} TUs  {nm}")
        sys.exit(0)

    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 20

    axis(body, emit, lambda r: r.prod,
         "AXIS PROD — the production tag, crossed with the EMITTED column "
         "for the first time", top=18)
    axis(body, emit, lambda r: r.dispatch,
         "AXIS DISPATCH — which arm of the body ladder claimed the body", top=12)
    axis(body, emit, lambda r: r.cflow, "AXIS CFLOW", top=12)
    axis(body, emit, lambda r: r.key.split("-then-")[0],
         "AXIS FORM — the receiver form alone (the key's head)", top=18)
    axis(body, emit, lambda r: ("-then-" + r.key.split("-then-", 1)[1])
         if "-then-" in r.key else "(whole / no second blocker)",
         "AXIS BLOCKER2 — the SECOND blocker alone (the key's tail)", top=18)
    axis(body, emit, lambda r: str(r.calls), "AXIS CALLS — call tokens in the body",
         top=10)

    replication(emit, lambda r: r.key,
                "REPLICATION — the EMITTED column by key, discounted", top=top)
    replication(body, lambda r: r.key,
                "REPLICATION — the BODY column by key, discounted", top=12)

    names = Counter()
    for r in emit:
        if r.name != "-":
            names[r.name] += r.n
    print(f"\n=== the emitted column's own name histogram ===")
    print(f"  distinct names among the {te} emitted: {len(names)} "
          f"({sum(r.n for r in emit if r.name == '-')} rows carry no name)")
    for nm, c in names.most_common(12):
        print(f"  {c:6d} {100*c/te:5.2f}%  {nm}")
