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

**`--splice`'s SPLICE set is WRONG and `--authors` supersedes it** (lane
`w-s7`, 2026-08-28; `WB_S7_FINDINGS.md` §3).  Read whole from the image, the
primitive band is `0x10bd3815`..`0x10bd3901` and holds **EIGHT** functions, not
four.  `--splice` names 3 of the 8 (`0x10bd3852` unlink, `0x10bd3892` move-after,
`0x10bd38b0` move-before) plus the scheduler relink; `dump_tuple_splice.py`
names a DIFFERENT 5; **the union of the two published sets is 8 of the 11
authors and 3 are named by neither.**  Both omit `0x10bd3824` INSERT BEFORE and
`0x10bd3815` INSERT AFTER from the *reach* test -- the two most-used tuple
primitives in the image (214 and 138 direct calls, 1,213 / 379 address-takes).
And the premise the docstring flags as unverified is FALSE:
`0x10bd5516` unlinks `tuple+0`/`tuple+0x10` INLINE and never calls
`0x10bd3852`, with 299 direct callers.  `--splice` is kept, unchanged, so
`WB_F0PRICE_FINDINGS.md` §4.2 stays reproducible; **do not quote its bracket.**

Usage:
    python3 docs/whitebox/scripts/f0_pipeline.py --verify
    python3 docs/whitebox/scripts/f0_pipeline.py --stages
    python3 docs/whitebox/scripts/f0_pipeline.py --band
    python3 docs/whitebox/scripts/f0_pipeline.py --splice     (SUPERSEDED)
    python3 docs/whitebox/scripts/f0_pipeline.py --authors
    python3 docs/whitebox/scripts/f0_pipeline.py --s7
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

# ---------------------------------------------------------------- lane w-s7 --
# The ORDER-AUTHOR set, read whole from the pinned image at 0x10bd3815 ..
# 0x10bd3901 plus the two inline splicers.  Sizes are the disassembled extents;
# every body was read.  See WB_S7_FINDINGS.md sections 3.1 and 3.2.
AUTHORS = [
    ("10bd3815", 15, "INSERT AFTER  (at, new)",         "band"),
    ("10bd3824", 17, "INSERT BEFORE (at, new)",         "band"),
    ("10bd3835", 29, "SPLICE CHAIN AFTER (at, chain)",  "band"),
    ("10bd3852", 31, "UNLINK (t)",                      "band"),
    ("10bd3871", 33, "UNLINK RANGE (a, b)",             "band"),
    ("10bd3892", 30, "MOVE AFTER  = unlink + ins after", "band"),
    ("10bd38b0", 32, "MOVE BEFORE = unlink + ins before", "band"),
    ("10bd38d0", 50, "MOVE RANGE (a, b, c)",            "band"),
    ("10be626c", 278, "scheduler bulk relink",          "sched"),
    ("10bd5516", 67, "INLINE unlink + free -- NEVER calls 0x10bd3852",
     "inline"),
    ("10bd5577", 131, "INLINE insert-before -- NEVER calls 0x10bd3824",
     "inline"),
]
AUTHOR_SET = {a for a, _, _, _ in AUTHORS}

# S7's ten depth-1 passes, in the order 0x10b7e032 calls them, with the gate
# each one runs under and whether that gate is SATISFIED on this project's
# measured configuration (/O1 /EHsc /GR, no POGO; w-restim's 2,946 functions,
# on all of which sym+0x20 bit 12 was clear).  `reached` is written out per row
# and NOT derived from the gate string -- the first draft matched on the
# substring "0x1000" and marked 0x10c12099 dead, whose gate needs it CLEAR.
S7_GATES = [
    ("10c21b03", False, "sym+0x20 & 0x1000 SET", "SEH driver; calls 0x10b35c78 itself"),
    ("10be46f0", False, "sym+0x20 & 0x1000 SET", "ehexcept walk over the 0x2e_ pseudo-ops"),
    ("10b3c6e5", False, "0x1000 SET and DAT_10c3de20 == 0", "merger, mode 0"),
    ("10b35c78", False, "sym+0x20 & 0x1000 SET", "THE SPLICER -- unlink + insert at 0x1b"),
    ("10b9d6be", False, "DAT_10c3de20 == 2 (POGO)", "dynamic instruction count"),
    ("10b36169", True, "unconditional", "0x2e8 -> late jump-table expansion"),
    ("10c12099", True, "/Og and 0x1000 CLEAR", "inserts 0x284 via the 0x10bd3824 POINTER"),
    ("10b821c3", True, "DAT_10c2e308 (/Og and not /GL-off)", "records the emitted range"),
    ("10c275a7", True, "unconditional", "mints a 0x2eb via the 0x10bd3815 POINTER"),
    ("10b3421b", True, "unconditional", "the emit driver -> 0x10b338f5, the emit walk"),
]


def repo_root():
    return os.path.dirname(os.path.dirname(os.path.dirname(
        os.path.dirname(os.path.abspath(__file__)))))


def load_xrefs():
    """from_func -> set of DATA-referenced targets.  Address-takes only.

    A CALL edge is in calls.tsv; a function POINTER handed to a shared builder
    is not, and for three of the eight splice primitives the pointer form is the
    DOMINANT one.  Any partition built on calls.tsv alone under-sees insertion.
    """
    exp = os.environ.get("C2RS_EXPORT",
                         os.path.expanduser("~/ghidra-projects/export/c2"))
    p = os.path.join(exp, "xrefs.tsv")
    if not os.path.exists(p):
        print("SKIP: %s absent" % p)
        sys.exit(2)
    data = {}
    with open(p) as fh:
        for r in csv.reader(fh, delimiter="\t"):
            if not r or r[0] == "from" or len(r) < 4:
                continue
            if r[2] == "DATA":
                data.setdefault(r[3], set()).add(r[1])
    return data


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


def _union(meta, calls):
    u = set()
    for name, ent, fixed in STAGES:
        u |= passes_of(meta, calls, ent, fixed)
    return u


def cmd_authors():
    """The order-author set, and the 34 re-partitioned against it.

    Prints the DIFF against --splice's four-address set so the correction is
    legible rather than a second, silently-disagreeing enumeration (#3505's
    family).  --splice is left untouched and still reproduces w-f0price.
    """
    meta, calls = load()
    data = load_xrefs()

    print("THE ORDER-AUTHOR SET -- read whole from the pinned image (w-s7)\n")
    for a, size, what, kind in AUTHORS:
        ncall = sum(1 for k, v in calls.items() if a in v)
        nptr = sum(1 for k, v in data.items() if a in v)
        mark = "  <-- in --splice's set" if a in SPLICE else ""
        print("  0x%s  %3d B  %-8s %-36s callers %4d  addr-takers %4d%s"
              % (a, size, kind, what, ncall, nptr, mark))
    r8 = {"10bd3815", "10bd3824", "10bd3835", "10bd3852", "10bd38d0"}
    print("\n  --splice names %d of the %d authors."
          % (len(SPLICE & AUTHOR_SET), len(AUTHORS)))
    print("  dump_tuple_splice.py (w-read-r8) names a DIFFERENT %d." % len(r8))
    print("  union of the two published sets: %d of %d.  Named by NEITHER: %s"
          % (len((SPLICE | r8) & AUTHOR_SET), len(AUTHORS),
             ", ".join("0x" + a for a in sorted(AUTHOR_SET - SPLICE - r8))))

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

    union = _union(meta, calls)
    old_a, old_b, old_c = set(), set(), set()
    for a in union:
        if calls.get(a, set()) & SPLICE:
            old_a.add(a)
        elif reach(a) & SPLICE:
            old_b.add(a)
        else:
            old_c.add(a)

    new_a, new_b, new_c = [], [], []
    for a in sorted(union, key=lambda x: -sz(meta, x)):
        direct = calls.get(a, set()) & AUTHOR_SET
        ptr = data.get(a, set()) & AUTHOR_SET
        if direct or ptr:
            new_a.append((a, sorted(direct), sorted(ptr)))
        elif reach(a) & AUTHOR_SET:
            new_b.append(a)
        else:
            new_c.append(a)

    print("\nA. AUTHORS an order directly (call or function pointer): %d of %d, %d B"
          % (len(new_a), len(union), sum(sz(meta, x[0]) for x in new_a)))
    for a, direct, ptr in new_a:
        tag = ""
        if ptr:
            tag = "PTR " + ",".join("0x" + p for p in ptr)
        if a in old_c:
            tag += "   *** --splice said C: CANNOT REORDER ***"
        elif a in old_b:
            tag += "   (--splice said B)"
        print(row(meta, a, tag))
    print("\nB. reaches an author transitively: %d of %d, %d B"
          % (len(new_b), len(union), sum(sz(meta, x) for x in new_b)))
    for a in new_b:
        print(row(meta, a, "(--splice said C)" if a in old_c else ""))
    print("\nC. reaches no author: %d of %d, %d B"
          % (len(new_c), len(union), sum(sz(meta, x) for x in new_c)))
    for a in new_c:
        print(row(meta, a))
    print("\n  --splice   A=%d  B=%d  C=%d      bracket %d..%d of %d"
          % (len(old_a), len(old_b), len(old_c),
             len(old_a), len(old_a) + len(old_b), len(union)))
    print("  --authors  A=%d  B=%d  C=%d      bracket %d..%d of %d"
          % (len(new_a), len(new_b), len(new_c),
             len(new_a), len(new_a) + len(new_b), len(union)))
    moved = [a for a, _, _ in new_a if a in old_c] + [a for a in new_b if a in old_c]
    print("\n  rows --splice called 'cannot reorder' that CAN: %d -> %s"
          % (len(moved), sorted(moved)))

    # Sensitivity: which correction moves the count?  Reported so the result is
    # not one undifferentiated jump.  #3505's family is about instruments that
    # measure themselves; the antidote is showing each term's contribution.
    print("\n  SENSITIVITY -- A-group size under each author set:")
    for label, aset, use_ptr in (
            ("--splice's 4 (calls only)", SPLICE, False),
            ("the 8-primitive band, calls only",
             AUTHOR_SET - {"10bd5516", "10bd5577", "10be626c"}, False),
            ("+ the scheduler relink", AUTHOR_SET - {"10bd5516", "10bd5577"}, False),
            ("+ function-pointer edges", AUTHOR_SET - {"10bd5516", "10bd5577"}, True),
            ("+ the two INLINE splicers", AUTHOR_SET, True)):
        n = 0
        for a in union:
            if calls.get(a, set()) & aset:
                n += 1
            elif use_ptr and (data.get(a, set()) & aset):
                n += 1
        print("      A = %2d of %d   %s" % (n, len(union), label))
    print("\n  Group C is STILL not a proof: 0x10bd5516 shows the inline form")
    print("  exists, and this lane enumerated inline splicers by READING, not")
    print("  by a pattern scan.  The population of inline splicers is OPEN.")
    return 0


def cmd_s7():
    """Stage S7 read whole: what runs, under what gate, and what is dead here."""
    meta, calls = load()
    data = load_xrefs()
    print("S7 = FUN_10b7e032 @ 0x10b7e032, 225 B, called once per function from")
    print("0x10b7e701 in the orchestrator 0x10b7e6af (DISCLOSURE W-STAGETAP-3).\n")
    print("The gate that splits it: `mov eax,[esi]; test [eax+0x20],0x1000` at")
    print("0x10b7e03a -- bit 12 of the SYMBOL flag word reached through func+0.")
    print("c2 NEVER sets that bit: the image holds no or/and writing 0x1000 to")
    print("any +0x20.  It arrives from the IL.  Measured CLEAR on 2,946 of")
    print("2,946 functions / 384 fixtures (w-restim), because sched0's site")
    print("0x10b7e00c sits inside the block 0x10b7dfea skips when it is SET.\n")
    live = dead = 0
    for a, reached, gate, what in S7_GATES:
        if reached:
            live += sz(meta, a)
        else:
            dead += sz(meta, a)
        print("  0x%s %5d B  %-9s %-34s %s"
              % (a, sz(meta, a), "live" if reached else "UNREACHED",
                 gate, what))
    tot = live + dead
    print("\n  live on this project's measured configuration: %d B of %d (%.0f%%)"
          % (live, tot, 100.0 * live / tot))
    print("  unreached: %d B of %d (%.0f%%) -- and BOTH of S7's tuple splicers"
          % (dead, tot, 100.0 * dead / tot))
    print("  (0x10c21b03, 0x10b35c78) are in the unreached half.\n")
    print("  0x10b35c78's ONLY two callers are 0x10b7e032 and 0x10c21b03, and")
    print("  0x10c21b03's only caller is 0x10b7e032 -- both inside the gate.")
    for a in ("10b35c78", "10c21b03"):
        cs = sorted(k for k, v in calls.items() if a in v)
        print("      callers of 0x%s: %s" % (a, ", ".join("0x" + c for c in cs)))
    print("\n  What CAN still change the emitted sequence in the live half:")
    for a, reached, gate, what in S7_GATES:
        if not reached:
            continue
        ptr = sorted(data.get(a, set()) & AUTHOR_SET)
        dcl = sorted(calls.get(a, set()) & AUTHOR_SET)
        if ptr or dcl:
            print("      0x%s  pointer=%s  call=%s"
                  % (a, [("0x" + p) for p in ptr] or "-",
                     [("0x" + c) for c in dcl] or "-"))
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
            "--band": cmd_band, "--splice": cmd_splice, "--merger": cmd_merger,
            "--authors": cmd_authors, "--s7": cmd_s7}
    if len(args) != 1 or args[0] not in cmds:
        print(__doc__)
        return 2
    return cmds[args[0]]()


if __name__ == "__main__":
    sys.exit(main())
