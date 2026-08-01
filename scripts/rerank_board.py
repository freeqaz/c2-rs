#!/usr/bin/env python3
"""Build the EMITTED board from a `c2rs gap` row dump, and diff two of them.

Tooling, deliberately outside the std-only Rust workspace (same standing as
`scripts/plot_perf.py`). It reads only artifacts the harness already produces:

  * the row dump  — `C2RS_ROW_DUMP='*' C2RS_ROW_DUMP_EMITTED=1 C2RS_ROW_DUMP_OUT=…`
    one TSV line per census row that binds to a symbol c2 actually emitted;
  * the scan JSONL — per-TU `fn_blockers` (body column) and `emit_blockers`.

**Why a script and not a report column.** `docs/ROADMAP.md` §8.6's standing rule
is that a histogram cannot answer a question about a JOINT, and `clean` is a
joint: `cflow-straight*` AND `eh-none` AND `calls<2`. The report prints each
axis as its own `BTreeMap`, so `clean` is unanswerable from it and answerable
from one pass over the dump. That is the same argument `gap.rs::row_dump` makes
for existing at all.

Usage:
    rerank_board.py board  <dump.tsv> <scan.jsonl>
    rerank_board.py diff   <base.tsv> <base.jsonl> <tip.tsv> <tip.jsonl>
    rerank_board.py bodies <base.jsonl> <tip.jsonl>
"""

import collections
import json
import sys

# The in-class shape labels. A row carrying one of these is ACCEPTED, not
# blocked, and belongs in the denominator's numerator rather than on the board.
# Kept as a prefix/exact set rather than a guess: any key that is not a known
# blocking key shape would otherwise silently inflate the board.
#
# **The in-class label set is DERIVED, never transcribed.** A hand-written list
# of `FnVerdict::InClass("…")` labels was wrong on first use — it missed
# `call-sequence*` and `float-leaf` and silently moved 1,785 accepted rows onto
# the board. The authority is the scan's own `emit_blockers` key set: a key that
# the scan never files as a blocker is, by construction, an accepted shape.
#
# The derivation is CHECKED and not trusted: `board()` asserts
#     in-class rows == the scan's `emit-in-class`
#     blocked  rows == sum(emit_blockers)
#     in-class + blocked == the scan's `emit-bound`
# so a drift between the dump and the report is a hard failure rather than a
# quietly inflated board.


def load_rows(path):
    """(key, is_emitted, frame, cflow, eh) per dump line."""
    rows = []
    with open(path) as f:
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) < 9:
                continue
            rows.append((p[2], p[3] == "EMITTED", p[5], p[6], p[7]))
    return rows


def clean(frame, cflow, eh):
    """§8.7's `clean`: a hard ceiling on what widening a key alone is worth."""
    return cflow.startswith("cflow-straight") and eh == "eh-none" and frame != "calls-2plus"


def scan_totals(jsonl):
    """The scan's own emit map and blocked-key set — the board's authority."""
    emit = collections.Counter()
    agg = collections.Counter()
    for line in open(jsonl):
        r = json.loads(line)
        if r.get("record"):
            continue
        for k, n in (r.get("emit_blockers") or {}).items():
            emit[k] += n
        for k, n in (r.get("emit") or {}).items():
            agg[k] += n
    return emit, agg


def board(path, jsonl):
    emit_blockers, agg = scan_totals(jsonl)
    blocked_keys = set(emit_blockers)
    rows = load_rows(path)
    emitted = [r for r in rows if r[1]]
    per = collections.Counter()
    per_clean = collections.Counter()
    inclass = 0
    for key, _e, frame, cflow, eh in emitted:
        if key not in blocked_keys:
            inclass += 1
            continue
        per[key] += 1
        if clean(frame, cflow, eh):
            per_clean[key] += 1
    # The three checks. Each could fail; none has a tolerance.
    checks = [
        ("in-class == emit-in-class", inclass, agg.get("emit-in-class", -1)),
        ("blocked == sum(emit_blockers)", sum(per.values()), sum(emit_blockers.values())),
        ("bound == emit-bound", len(emitted), agg.get("emit-bound", -1)),
    ]
    for name, got, want in checks:
        if got != want:
            raise SystemExit(f"BOARD CHECK FAILED [{name}]: dump={got} report={want} ({path})")
    return {
        "rows": len(rows),
        "emitted": len(emitted),
        "in_class": inclass,
        "blocked": sum(per.values()),
        "clean_total": sum(per_clean.values()),
        "per": per,
        "per_clean": per_clean,
    }


def fmt(n):
    return f"{n:,}"


def cmd_board(path, jsonl, top=25):
    b = board(path, jsonl)
    print(f"bound emitted rows {fmt(b['emitted'])}  in-class {fmt(b['in_class'])}  "
          f"blocked {fmt(b['blocked'])}  clean {fmt(b['clean_total'])} "
          f"({100 * b['clean_total'] / max(b['blocked'], 1):.2f} %)")
    print(f"{'rank':>4}  {'emitted':>8}  {'clean':>7}  key")
    for i, (k, n) in enumerate(b["per"].most_common(top), 1):
        print(f"{i:>4}  {n:>8}  {b['per_clean'][k]:>7}  {k}")


def cmd_diff(base_p, base_j, tip_p, tip_j, top=25):
    a, z = board(base_p, base_j), board(tip_p, tip_j)
    print("## totals")
    for name, d in (("base", a), ("tip", z)):
        print(f"  {name}: bound-emitted {fmt(d['emitted'])}  in-class {fmt(d['in_class'])}  "
              f"blocked {fmt(d['blocked'])}  clean {fmt(d['clean_total'])}")
    ra = {k: i for i, (k, _) in enumerate(a["per"].most_common(), 1)}
    rz = {k: i for i, (k, _) in enumerate(z["per"].most_common(), 1)}

    print(f"\n## tip top {top}")
    print(f"{'rank':>4} {'(was)':>6}  {'emitted':>8} {'(was)':>8}  {'clean':>7}  key")
    for i, (k, n) in enumerate(z["per"].most_common(top), 1):
        was_r = ra.get(k)
        was_n = a["per"].get(k, 0)
        tag = "NEW" if was_r is None else str(was_r)
        print(f"{i:>4} {tag:>6}  {n:>8} {was_n:>8}  {z['per_clean'][k]:>7}  {k}")

    print(f"\n## rows that DIED (base top {top}, gone or shrunk >90 % at tip)")
    for k, n in a["per"].most_common(top):
        m = z["per"].get(k, 0)
        if m == 0 or m < 0.1 * n:
            print(f"  {n:>8} -> {m:<8}  (rank {ra[k]} -> {rz.get(k, '-')})  {k}")

    print(f"\n## rows that APPEARED (tip top {top}, absent or <10 % at base)")
    for k, n in z["per"].most_common(top):
        m = a["per"].get(k, 0)
        if m == 0 or m < 0.1 * n:
            print(f"  {m:>8} -> {n:<8}  (rank {ra.get(k, '-')} -> {rz[k]})  {k}")

    # §8.7 ranks the emitted board a second way — by `clean` ceiling, "a hard
    # ceiling on what widening a key alone is worth". The two orders disagree,
    # and the disagreement is the point: a row can be large and entirely
    # phase-gated, or small and entirely takeable.
    print(f"\n## tip top {top} BY CLEAN CEILING")
    rca = {k: i for i, (k, _) in enumerate(a["per_clean"].most_common(), 1)}
    print(f"{'rank':>4} {'(was)':>6}  {'clean':>7} {'(was)':>7}  {'emitted':>8}  key")
    for i, (k, n) in enumerate(z["per_clean"].most_common(top), 1):
        was = rca.get(k)
        print(f"{i:>4} {str(was) if was else 'NEW':>6}  {n:>7} {a['per_clean'].get(k, 0):>7}  "
              f"{z['per'][k]:>8}  {k}")

    print("\n## biggest movers by |delta emitted| (all keys)")
    keys = set(a["per"]) | set(z["per"])
    deltas = sorted(keys, key=lambda k: abs(z["per"].get(k, 0) - a["per"].get(k, 0)), reverse=True)
    for k in deltas[:top]:
        pa, pz = a["per"].get(k, 0), z["per"].get(k, 0)
        if pa == pz:
            continue
        print(f"  {pa:>8} -> {pz:<8} ({pz - pa:+7})  {k}")


def load_bodies(path):
    per = collections.Counter()
    emit = collections.Counter()
    tot = inc = etot = einc = 0
    with open(path) as f:
        for line in f:
            r = json.loads(line)
            if r.get("record"):
                continue
            tot += r.get("fn_total", 0)
            inc += r.get("fn_in_class", 0)
            for k, n in (r.get("fn_blockers") or {}).items():
                per[k] += n
            for k, n in (r.get("emit_blockers") or {}).items():
                emit[k] += n
            e = r.get("emit") or {}
            etot += e.get("emit-emitted", 0)
            einc += e.get("emit-in-class", 0)
    return per, emit, (tot, inc, etot, einc)


def cmd_bodies(base_p, tip_p, top=30):
    pa, ea, ta = load_bodies(base_p)
    pz, ez, tz = load_bodies(tip_p)
    print(f"base: bodies {fmt(ta[1])}/{fmt(ta[0])}  emitted {fmt(ta[3])}/{fmt(ta[2])}")
    print(f"tip : bodies {fmt(tz[1])}/{fmt(tz[0])}  emitted {fmt(tz[3])}/{fmt(tz[2])}")
    print(f"NUMERATOR DELTA: bodies {tz[1] - ta[1]:+}  emitted {tz[3] - ta[3]:+}")

    def summarize(label, a, z):
        print(f"\n## {label}: keys by |delta|")
        keys = set(a) | set(z)
        moved = sorted(keys, key=lambda k: abs(z.get(k, 0) - a.get(k, 0)), reverse=True)
        shown = 0
        for k in moved:
            d = z.get(k, 0) - a.get(k, 0)
            if d == 0:
                continue
            print(f"  {a.get(k, 0):>9} -> {z.get(k, 0):<9} ({d:+9})  {k}")
            shown += 1
            if shown >= top:
                break
        print(f"  ... net over ALL keys: {sum(z.get(k, 0) for k in keys) - sum(a.get(k, 0) for k in keys):+}")

    summarize("BODY column", pa, pz)
    summarize("EMITTED column", ea, ez)


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    cmd = sys.argv[1]
    if cmd == "board":
        cmd_board(sys.argv[2], sys.argv[3])
    elif cmd == "diff":
        cmd_diff(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5])
    elif cmd == "bodies":
        cmd_bodies(sys.argv[2], sys.argv[3])
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
