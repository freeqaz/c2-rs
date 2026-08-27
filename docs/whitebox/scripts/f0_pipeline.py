#!/usr/bin/env python3
"""Enumerate c2's POST-ALLOCATOR pipeline — lane `w-f0price`, item F0's price.

Whitebox tooling (outside the std-only `crates/` workspace, per CLAUDE.md).

`WB_ITEMF_FINDINGS.md` §6.1 prices item **F0** at **8 lanes** and the price is
an ENUMERATION of eight named sub-items, one lane each.  Sub-item 7 is *"the
lowering band `0x10b7dd2c`/`0x10b7ddff`/`0x10b7de4a` — three passes, unread by
any lane"*, and `rungs/2026-08-15-itemfprice.md` §10 item 4 says of exactly
those three addresses:

    "Reading their sizes is a grep away and would tighten F0's 8.
     Not taken: it is F0's work, not a pricing lane's."

**That check returns 425 bytes and would tighten F0 the WRONG WAY.**  The three
entries hold no transformation logic at all — they are pass DRIVERS, and the
naive size read measures the driver rather than the work.  This script exists
so nobody repeats the measurement in the form that misleads: it walks the
pipeline `FUN_10b7e6af` drives and reports, at depth 1, what each stage
actually invokes.

`--splice` additionally partitions those passes by whether they can move a
tuple at all, which is the quantity F0 is denominated in.  Order is authored
through the tuple list's splice primitives:

    0x10bd3852   unlink            (tuple+0 = next, tuple+0x10 = prev)
    0x10bd38b0   unlink + insert BEFORE anchor
    0x10bd3892   unlink + insert AFTER  anchor
    0x10be626c   the scheduler's bulk relink into scheduled order

**The partition is a BRACKET, not a verdict**: a direct caller of one of those
CAN move tuples; a function reaching one only transitively MAY; and group C's
"reaches none" is sound only under the premise that no pass rewires `tuple+0` /
`tuple+0x10` inline.  This script does not verify that premise and the findings
say so.  Two documents name the same primitive set independently
(`WB_DAGCLIENTS_FINDINGS.md` §2 "the splices"; `ref/P_BLOCKORDER.md` §1's emit
walk follows `tuple+0` and does nothing else), which is corroboration and not
proof.

Inputs are the committed `ref/FUNCS.tsv` (sizes, TU, coverage) and the Ghidra
flat export's `calls.tsv`.  Neither is the image; `--verify` re-checks the
pinned image's sha256 so an address quoted from here is quotable.

Usage:
    python3 docs/whitebox/scripts/f0_pipeline.py --verify
    python3 docs/whitebox/scripts/f0_pipeline.py --stages
    python3 docs/whitebox/scripts/f0_pipeline.py --band
    python3 docs/whitebox/scripts/f0_pipeline.py --splice
    python3 docs/whitebox/scripts/f0_pipeline.py --merger

Env:
    C2RS_EXPORT   Ghidra flat export dir (default ~/ghidra-projects/export/c2)
    C2RS_IMAGE    the pinned c2.dll   (default compilers/X360/16.00.11886.00/c2.dll)

Exits 2 with `SKIP:` when an input is absent — never fails a caller.
"""
import csv
import hashlib
import os
import sys

SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

# `FUN_10b7e6af` @ 0x10b7e6af, read whole from the flat export.  The stage
# drivers in the order it calls them; S1 is inside 0x10b7dc51, which ENDS with
# the register allocator 0x10b31c9a, so S1..S7 is everything downstream of the
# allocation.  S8 is gated on DAT_10c6f1c8 (the POGO instrument).
STAGES = [
    ("S1  sched pass 3 (tail of 0x10b7dc51)", None, ["10be6382"]),
    ("S2  0x10b7dd2c", "10b7dd2c", None),
    ("S3  0x10b7ddff", "10b7ddff", None),
    ("S4  0x10b7de4a", "10b7de4a", None),
    ("S5  0x10b7ded5", "10b7ded5", None),
    ("S6  0x10b7df57  the FINAL (mode 0) schedule", "10b7df57", None),
    ("S7  0x10b7e032  the emit tail", "10b7e032", None),
    ("S8  0x10b9c836  (gated DAT_10c6f1c8)", "10b9c836", None),
]
BAND = ["10b7dd2c", "10b7ddff", "10b7de4a"]      # F0 sub-item 7
ABORT = "10bec297"                                # the abort poll, not a pass
SPLICE = {"10bd3852", "10bd38b0", "10bd3892", "10be626c"}
MERGER_WORKER = "10b3c2cc"                        # 0x10b3c6e5's per-tuple walker
NAMED_BY_F0 = {"10b3b167": "K1 (sub-item 5)",
               "10b3b41b": "K2 (sub-item 5)",
               "10b3baa8": "M4 (sub-item 6)"}


def repo_root():
    return os.path.dirname(os.path.dirname(os.path.dirname(
        os.path.dirname(os.path.abspath(__file__)))))


def load():
    root = repo_root()
    funcs = os.path.join(root, "docs/whitebox/ref/FUNCS.tsv")
    exp = os.environ.get("C2RS_EXPORT",
                         os.path.expanduser("~/ghidra-projects/export/c2"))
    callsf = os.path.join(exp, "calls.tsv")
    for p in (funcs, callsf):
        if not os.path.exists(p):
            print("SKIP: %s absent" % p)
            sys.exit(2)
    meta = {}
    with open(funcs) as fh:
        for r in csv.reader(fh, delimiter="\t"):
            if not r or r[0].startswith("#") or r[0] == "addr":
                continue
            meta[r[0]] = {"size": int(r[1]), "tu": r[3], "cover": r[7]}
    calls = {}
    with open(callsf) as fh:
        for r in csv.reader(fh, delimiter="\t"):
            if not r or r[0] == "caller_addr":
                continue
            calls.setdefault(r[0], set()).add(r[2])
    return meta, calls


def sz(meta, a):
    return meta.get(a, {}).get("size", 0)


def row(meta, a, tag=""):
    m = meta.get(a, {})
    return "    %s  %6d B  %-26s cover=%-9s %s" % (
        a, m.get("size", 0), m.get("tu", "?"), m.get("cover", "?"), tag)


def passes_of(meta, calls, ent, fixed):
    return set(fixed) if fixed else (calls.get(ent, set()) - {ABORT})


def cmd_verify():
    img = os.environ.get("C2RS_IMAGE", os.path.join(
        repo_root(), "compilers/X360/16.00.11886.00/c2.dll"))
    if not os.path.exists(img):
        print("SKIP: pinned image absent (%s)" % img)
        sys.exit(2)
    with open(img, "rb") as fh:
        got = hashlib.sha256(fh.read()).hexdigest()
    print("image  %s" % img)
    print("sha256 %s" % got)
    print("expect %s" % SHA256)
    print("MATCH" if got == SHA256 else "*** MISMATCH — addresses NOT quotable ***")
    return 0 if got == SHA256 else 1


def cmd_stages():
    meta, calls = load()
    union = set()
    print("The post-allocator pipeline, from FUN_10b7e6af @ 0x10b7e6af.")
    print("Depth 1; the abort poll 0x10bec297 is excluded (143 sites, not a pass).\n")
    for name, ent, fixed in STAGES:
        ps = passes_of(meta, calls, ent, fixed)
        union |= ps
        print("--- %s   driver %d B  ->  %d passes, %d B"
              % (name, sz(meta, ent) if ent else 0, len(ps),
                 sum(sz(meta, a) for a in ps)))
        for a in sorted(ps, key=lambda x: -sz(meta, x)):
            print(row(meta, a))
    tus = sorted({meta.get(a, {}).get("tu", "?") for a in union})
    nnone = sum(1 for a in union if meta.get(a, {}).get("cover") == "none")
    print("\nUNION: %d distinct passes, %d B, %d TUs" %
          (len(union), sum(sz(meta, a) for a in union), len(tus)))
    print("cover=none (no document in this repo mentions them): %d of %d"
          % (nnone, len(union)))
    named = union & ({"10be6382"} | set(NAMED_BY_F0))
    print("named by any of F0's eight sub-items: %d of %d  -> %s"
          % (len(named), len(union), sorted(named)))
    return 0


def cmd_band():
    meta, calls = load()
    print("F0 sub-item 7 — 'the lowering band ... three passes, unread by any lane'\n")
    tot = 0
    for a in BAND:
        ps = calls.get(a, set()) - {ABORT}
        tot += sz(meta, a)
        print("  entry %s  %d B  ->  %d passes" % (a, sz(meta, a), len(ps)))
    d1 = set()
    for a in BAND:
        d1 |= calls.get(a, set()) - {ABORT}
    print("\n  the three ENTRIES total %d B  <-- what 'reading their sizes' returns" % tot)
    print("  they drive %d depth-1 passes totalling %d B, across %d TUs"
          % (len(d1), sum(sz(meta, a) for a in d1),
             len({meta.get(x, {}).get("tu", "?") for x in d1})))
    print()
    for a in sorted(d1, key=lambda x: -sz(meta, x)):
        print(row(meta, a))
    return 0


def cmd_splice():
    meta, calls = load()

    def reach(f, limit=12):
        seen, fr = {f}, {f}
        for _ in range(limit):
            nxt = set()
            for x in fr:
                nxt |= calls.get(x, set())
            nxt -= seen
            if not nxt:
                break
            seen |= nxt
            fr = nxt
        return seen

    union = set()
    for name, ent, fixed in STAGES:
        union |= passes_of(meta, calls, ent, fixed)
    a_, b_, c_ = [], [], []
    for a in sorted(union, key=lambda x: -sz(meta, x)):
        if calls.get(a, set()) & SPLICE:
            a_.append(a)
        elif reach(a) & SPLICE:
            b_.append(a)
        else:
            c_.append(a)
    for title, lst in (("A. DIRECT caller of a splice primitive — CAN move tuples", a_),
                       ("B. reaches a splice transitively — MAY move tuples", b_),
                       ("C. reaches no splice primitive — cannot reorder*", c_)):
        print("\n%s: %d of %d, %d B" %
              (title, len(lst), len(union), sum(sz(meta, x) for x in lst)))
        for a in lst:
            print(row(meta, a))
    print("\n  * group C is sound only under the UNVERIFIED premise that no pass")
    print("    rewires tuple+0 / tuple+0x10 inline.  See the module docstring.")
    print("\n  the order-changing set is bracketed at %d..%d of %d passes."
          % (len(a_), len(a_) + len(b_), len(union)))
    return 0


def cmd_merger():
    meta, calls = load()
    cs = calls.get(MERGER_WORKER, set())
    print("0x10b3c6e5's per-tuple walker 0x%s (%d B) dispatches to %d callees, %d B."
          % (MERGER_WORKER, sz(meta, MERGER_WORKER), len(cs),
             sum(sz(meta, a) for a in cs)))
    print("F0 sub-items 5+6 name %d of them.\n"
          % sum(1 for a in cs if a in NAMED_BY_F0))
    for a in sorted(cs, key=lambda x: -sz(meta, x)):
        print(row(meta, a, NAMED_BY_F0.get(a, "")))
    print("\ncallers of the driver 0x10b3c6e5 (mode is its 2nd argument):")
    for a in sorted(k for k, v in calls.items() if "10b3c6e5" in v):
        print(row(meta, a))
    return 0


def main():
    args = sys.argv[1:]
    cmds = {"--verify": cmd_verify, "--stages": cmd_stages,
            "--band": cmd_band, "--splice": cmd_splice, "--merger": cmd_merger}
    if len(args) != 1 or args[0] not in cmds:
        print(__doc__)
        return 2
    return cmds[args[0]]()


if __name__ == "__main__":
    sys.exit(main())
