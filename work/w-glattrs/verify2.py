#!/usr/bin/env python3
"""verify2.py — EVERY published count of this lane, DERIVED A SECOND WAY.

Board **#3288**, in its strengthened form: *a published count that was never
diffed against a second, differently-built one is unverified in EITHER
direction* — one member of that family was an OVER-count wrong at its own base.
A lane's own inherited figure is exactly the kind that gets carried forward.

So this file **does not import `glrec`**. It is a second implementation of the
`.gl` record walk, written from the record spelling rather than from the first
walker, and it re-derives:

  * total records and SIZE forms, under both framings
  * whole-file verdicts under both readers, and the rescued count
  * the escaped-SIZE call-edge count (the 309 this lane INHERITED from
    `w-sizebracket` and must not carry)
  * the total call-edge count
  * the ATTR vocabulary and the rival-width scores
  * the high-byte witness count

and prints a DIFF against the figures the rung publishes. A mismatch is a
failure of this lane, not of the checker.

Deliberate differences from `glrec.py`, so that a shared bug cannot survive:

  * the record walk scans with `re`-free explicit indexing over a `memoryview`
    and locates names by a forward pass with a dict of run-ends, where `glrec`
    uses a reverse `rposition`/monotone cursor;
  * the framing predicates are written out inline per byte rather than shared;
  * SIZE/SRCPOS are decoded by a single parameterised varint reader taking the
    payload width, where `glrec` inlines each field.
"""

import collections
import os
import sys

# ---------------------------------------------------------------- PUBLISHED
# The figures the rung and the board rows state. Edited only when a number
# genuinely changes; the point is that this list is typed from the WRITE-UP and
# compared against a fresh computation.
PUBLISHED = {
    "records_incumbent": 28838,
    "records_relaxed": 1461374,
    "forms_incumbent_direct": 28739,
    "forms_incumbent_escape": 99,
    "forms_incumbent_high": 0,
    "forms_relaxed_direct": 1403908,
    "forms_relaxed_escape": 57466,
    "forms_relaxed_high": 0,
    "files": 870,
    "incumbent_ok": 801,
    "incumbent_refused": 69,
    "incumbent_refused_size_escape": 60,
    "incumbent_refused_attr_conflict": 9,
    "new_ok": 857,
    "new_refused": 13,
    "rescued": 56,
    "relaxed_incumbent_ok": 32,
    "relaxed_refused_size_escape": 838,
    "relaxed_new_ok": 805,
    "relaxed_rescued": 773,
    "edges_total": 7667,
    "edges_escaped_relaxed": 309,
    "edges_escaped_incumbent": 2,
    "vocab_size": 10,
    "rival_w1": 3,
    "rival_w2": 0,
    "rival_w3": 99,
    "rival_w5": 1,
}

MAX_NAME_TO_OFFSET = 64


def runs_of(gl):
    """Name runs, as a list of (end, name) — a FORWARD pass, and the result is
    indexed by end position rather than searched backwards."""
    out = []
    n = len(gl)
    i = 0
    while i < n:
        c = gl[i]
        if c != 0 and c != 0x26:
            i += 1
            continue
        j = i + 1
        k = j
        while k < n:
            d = gl[k]
            if d == 0 or d == 0x26:
                break
            k += 1
        if k >= n or k == j:
            i += 1
            continue
        ok = True
        for x in range(j, k):
            if not (0x21 <= gl[x] <= 0x7E):
                ok = False
                break
        if ok:
            f = gl[j]
            if f == 0x3F or (0x41 <= f <= 0x5A) or (0x61 <= f <= 0x7A) or f == 0x5F:
                out.append((k, gl[j:k].decode("ascii")))
        i = k
    return out


def varint(gl, q, payload):
    """One byte, or `0x80` then `payload` little-endian bytes. Returns
    (value, next_q, form)."""
    b = gl[q]
    if b == 0x80:
        if q + 1 + payload > len(gl):
            return None, None, "trunc"
        v = 0
        for k in range(payload):
            v |= gl[q + 1 + k] << (8 * k)
        return v, q + 1 + payload, "escape"
    if b < 0x80:
        return b, q + 1, "direct"
    return b - 0x100, q + 1, "high"


def walk2(gl, relaxed, esc_width=2):
    """Yield (verdict, dict). A second implementation of the same walk."""
    rs = runs_of(gl)
    ends = [e for e, _ in rs]
    n = len(gl)
    p = 0
    ri = -1
    while p + 5 <= n:
        # framing, written out per byte
        ok = p >= 7 and gl[p] == 0x80 and gl[p - 7] == 0x80
        if ok:
            ok = gl[p - 4] == 0 and gl[p - 3] == 0 and gl[p - 2] == 0 and gl[p - 1] == 0
        if ok and not relaxed:
            ok = gl[p - 5] == 0x10
        if ok and relaxed:
            ok = p + 4 < n and (
                gl[p + 1] | (gl[p + 2] << 8) | (gl[p + 3] << 16) | (gl[p + 4] << 24)
            ) < 0x0100_0000
        if not ok:
            nx = gl.find(b"\x80", p + 1)
            p = n if nx < 0 else nx
            continue
        while ri + 1 < len(ends) and ends[ri + 1] <= p:
            ri += 1
        if ri < 0 or p - ends[ri] > MAX_NAME_TO_OFFSET:
            yield ("noname", {})
            return
        q = p + 5
        if q >= n:
            yield ("trunc", {})
            return
        _, q2, f1 = varint(gl, q, 4)
        if f1 == "high":
            yield ("srcpos", {})
            return
        if f1 == "trunc" or q2 is None:
            yield ("trunc", {})
            return
        q = q2
        if q >= n:
            yield ("trunc", {})
            return
        size, q3, f2 = varint(gl, q, esc_width)
        if f2 == "trunc" or q3 is None:
            yield ("trunc", {})
            return
        yield ("ok", {
            "name": rs[ri][1], "size": size, "form": f2,
            "attr": gl[q3] if q3 < n else None, "p": p,
        })
        p += 5


def verdict2(gl, relaxed):
    inc = new = None
    recs = []
    for v, r in walk2(gl, relaxed):
        if v != "ok":
            inc = inc or v
            new = new or v
            break
        recs.append(r)
        if r["form"] == "escape" and inc is None:
            inc = "size-escape"
        if r["form"] == "high":
            inc = inc or "size-high"
            new = new or "size-high"
        if r["attr"] is None:
            inc = inc or "no-attr"
            new = new or "no-attr"
    seen = {}
    for r in recs:
        if r["name"] in seen and seen[r["name"]] != r["attr"]:
            inc = inc or "attr-conflict"
            new = new or "attr-conflict"
            break
        seen[r["name"]] = r["attr"]
    return inc, new, recs


def safe(src):
    return "".join(c if c.isalnum() else "_" for c in src) + ".gl"


def main(argv):
    gldir, errlog = argv[1], argv[2]
    got = {}
    blobs = {}
    for fn in sorted(os.listdir(gldir)):
        if fn.endswith(".gl"):
            blobs[fn] = open(os.path.join(gldir, fn), "rb").read()
    got["files"] = len(blobs)

    for tag, relaxed in (("incumbent", False), ("relaxed", True)):
        forms = collections.Counter()
        incc = collections.Counter()
        newc = collections.Counter()
        rescued = 0
        for fn, gl in blobs.items():
            inc, new, recs = verdict2(gl, relaxed)
            for r in recs:
                forms[r["form"]] += 1
            incc[inc or "OK"] += 1
            newc[new or "OK"] += 1
            if inc is not None and new is None:
                rescued += 1
        pre = "" if tag == "incumbent" else "relaxed_"
        got[f"records_{tag}"] = sum(forms.values())
        for f in ("direct", "escape", "high"):
            got[f"forms_{tag}_{f}"] = forms[f]
        if tag == "incumbent":
            got["incumbent_ok"] = incc["OK"]
            got["incumbent_refused"] = sum(v for k, v in incc.items() if k != "OK")
            got["incumbent_refused_size_escape"] = incc["size-escape"]
            got["incumbent_refused_attr_conflict"] = incc["attr-conflict"]
            got["new_ok"] = newc["OK"]
            got["new_refused"] = sum(v for k, v in newc.items() if k != "OK")
            got["rescued"] = rescued
        else:
            got["relaxed_incumbent_ok"] = incc["OK"]
            got["relaxed_refused_size_escape"] = incc["size-escape"]
            got["relaxed_new_ok"] = newc["OK"]
            got["relaxed_rescued"] = rescued

    # ---- the call edges, and the 309 this lane INHERITED
    edges = []
    for line in open(errlog, errors="replace"):
        if line.startswith("GC-EDGE\t"):
            _, tu, caller, callee, arm = line.rstrip("\n").split("\t")
            edges.append((safe(tu), callee, arm))
    got["edges_total"] = len(edges)
    for tag, relaxed in (("incumbent", False), ("relaxed", True)):
        dec = {}
        for fn, gl in blobs.items():
            m = {}
            for v, r in walk2(gl, relaxed):
                if v == "ok":
                    m.setdefault(r["name"], r)
            dec[fn] = m
        n = 0
        for tu, callee, _ in edges:
            r = dec.get(tu, {}).get(callee)
            if r and r["form"] == "escape":
                n += 1
        got[f"edges_escaped_{tag}"] = n

    # ---- vocabulary and rival widths, on the 99 incumbent-framing escaped records
    vocab = set()
    for gl in blobs.values():
        for v, r in walk2(gl, False):
            if v == "ok" and r["form"] == "direct" and r["attr"] is not None:
                vocab.add(r["attr"])
    got["vocab_size"] = len(vocab)
    for w in (1, 2, 3, 5):
        good = 0
        for gl in blobs.values():
            for v, r in walk2(gl, False, esc_width=w - 1):
                if v == "ok" and r["form"] == "escape" and r["attr"] in vocab:
                    good += 1
        got[f"rival_w{w}"] = good

    bad = 0
    print(f"{'figure':>34} {'published':>11} {'re-derived':>11}  verdict")
    for k in PUBLISHED:
        p, g = PUBLISHED[k], got.get(k, "MISSING")
        ok = p == g
        bad += not ok
        print(f"{k:>34} {p:>11} {str(g):>11}  {'ok' if ok else '*** DIFFERS ***'}")
    print(f"\n{len(PUBLISHED) - bad} of {len(PUBLISHED)} agree; {bad} differ")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
