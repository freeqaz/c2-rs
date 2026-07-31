#!/usr/bin/env python3
"""Ground truth for the relational operator family `1F`-`24`.

Two modes, matching the two pieces of evidence `BARE_BINARY_OPS` demands
(`crates/c2-il/src/func/body/mcall.rs`), and the write-up in
`docs/rungs/2026-07-31-relational-bare.md`.

  --il DIR      **Is the token bare?** Read a `c2rs census ... --keep-il`
                bundle and print, for every relational opcode found, the byte
                that FOLLOWS it. A bare operator is followed straight away by
                its consumer; one that carries a TYPE is followed by `86 xx xx`.
                The compound-assign family is printed beside it as the control,
                because the whole reason this file exists is that `19` was read
                and `1F`-`24` was inferred from it across a family boundary.

  --base/--grant/--parser JSONL
                **Does the grant redistribute 1:1?** Diff two `c2rs gap
                --jsonl` scans and print the column that would falsify the
                bareness reading: SUM OF ALL KEY DELTAS. A byte that is not one
                byte wide desyncs the completeness matcher and scatters its row
                across the hex tail, so the sum stops being 0 and the total
                blocked count moves. Printed on every run, never remembered.

std-lib only (the workspace rule applies to tooling by convention here too).
"""

import argparse
import collections
import json
import os
import re
import sys

# Capture-verified (`expr_opcode_name`, and re-read by `--il` below). Signedness
# is NOT in the opcode: signed and unsigned probes emit the same byte and differ
# only in the operand TYPE (`86 41 74` int vs `86 42 75` unsigned).
RELATIONAL = {0x1F: "==", 0x20: "!=", 0x21: "<=", 0x22: "<", 0x23: ">=", 0x24: ">"}

# The control family. `19` is a compound assign, and reading it was what produced
# the wrong inference about its numeric neighbours. `0F`/`16` are its members that
# a probe can force into a single TU (`+=` and `>>=`).
COMPOUND_ASSIGN = {0x0F: "+=", 0x16: ">>="}

# Already-granted members, for a same-run comparison rather than a quoted one.
GRANTED_BARE = {0x09: "<<", 0x0A: ">>", 0x0B: "&", 0x0C: "|", 0x0D: "^"}

TYPE_LEAD = 0x86  # an inline TYPE starts `86 xx xx`
LIT = bytes((0x33,))  # `33 <TYPE> <varint>` — the literal that feeds an operator


# The probe, kept HERE rather than in `fixtures/cpp/` on purpose: it grades
# nothing and admits nothing, so it must not enter `c2rs bench`. `--emit-probe`
# writes it to scratch so the capture is re-runnable without a tracked artefact.
#
# Every relation appears in VALUE position and in BRANCH position, signed and
# unsigned, and the compound-assign CONTROL shares the TU -- reading `19`'s
# family in a different capture from the relations is the exact mistake this
# probe exists to stop.
PROBE_CPP = """\
struct S { int flags; int Flags() const; };
int gk(int);

int v_lt(unsigned x) { return x <  3; }
int v_le(unsigned x) { return x <= 3; }
int v_gt(unsigned x) { return x >  3; }
int v_ge(unsigned x) { return x >= 3; }
int v_eq(unsigned x) { return x == 3; }
int v_ne(unsigned x) { return x != 3; }

int s_lt(int x) { return x <  3; }
int s_le(int x) { return x <= 3; }
int s_gt(int x) { return x >  3; }
int s_ge(int x) { return x >= 3; }
int s_eq(int x) { return x == 3; }
int s_ne(int x) { return x != 3; }

int b_lt(unsigned x) { if (x <  3) { return gk(1); } return 0; }
int b_ge(int x)      { if (x >= 3) { return gk(2); } return 0; }

/* the CONTROL: 19's family, in the same TU */
int c_addassign(int x) { x += 3; return x; }
int c_shrassign(unsigned x) { x >>= 3; return int(x); }

/* the already-granted bare set, as the third control */
int g_and(unsigned x) { return int(x & 3); }
int g_or(unsigned x)  { return int(x | 3); }
int g_xor(unsigned x) { return int(x ^ 3); }
int g_shr(unsigned x) { return int(x >> 3); }
int g_shl(unsigned x) { return int(x << 3); }
"""


def hexs(bs):
    return " ".join(f"{b:02x}" for b in bs)


def read_bundle(d):
    """Yield (name, bytes) for every `.ex` in a --keep-il directory."""
    out = []
    for fn in sorted(os.listdir(d)):
        if fn.endswith(".ex"):
            with open(os.path.join(d, fn), "rb") as f:
                out.append((fn, f.read()))
    return out


def scan_widths(data, table):
    """Every site where `33 <TYPE:3> <lit:1> <op>` appears: report what follows.

    Anchoring on the literal is what makes the reading a reading rather than a
    byte hunt -- an operator byte value can occur inside a token or a varint, and
    only the operand-then-operator context pins it as an opcode.
    """
    hits = collections.defaultdict(list)
    for i in range(len(data) - 9):
        if data[i] != 0x33 or data[i + 1] != TYPE_LEAD:
            continue
        op_at = i + 5  # 33 | 86 xx xx | <1-byte lit> | op
        op = data[op_at]
        if op not in table:
            continue
        nxt = data[op_at + 1]
        hits[op].append((nxt, hexs(data[i : op_at + 4])))
    return hits


def cmd_il(d):
    bundles = read_bundle(d)
    if not bundles:
        sys.exit(f"no .ex in {d} (run `c2rs census <cpp> --keep-il {d}`)")
    print(f"# operator width, read from {len(bundles)} captured .ex bundle(s) in {d}")
    print("#   a TYPE-carrying operator is followed by 86 xx xx;")
    print("#   a bare operator is followed by its consumer (2c convert, 38 branch, 41 ret, ...)")
    verdicts = {}
    for title, table in (
        ("RELATIONAL 1F-24  (the family under test)", RELATIONAL),
        ("COMPOUND-ASSIGN   (the CONTROL -- 19's family; must show 86)", COMPOUND_ASSIGN),
        ("ALREADY GRANTED   (BARE_BINARY_OPS; must show no 86)", GRANTED_BARE),
    ):
        print(f"\n## {title}")
        seen = False
        for name, data in bundles:
            for op, sites in sorted(scan_widths(data, table).items()):
                seen = True
                follows = collections.Counter(n for n, _ in sites)
                bare = all(n != TYPE_LEAD for n in follows)
                verdicts[op] = bare and verdicts.get(op, True)
                tag = "BARE" if bare else "CARRIES A TYPE"
                foll = ", ".join(f"{n:02x}x{c}" for n, c in follows.most_common())
                print(f"  {op:02x} {table[op]:>3s}  {tag:<14s} n={len(sites):<3d} follows: {foll}")
                print(f"          e.g.  {sites[0][1]}")
        if not seen:
            print("  (no site in this bundle)")
    rel = {op: verdicts.get(op) for op in RELATIONAL}
    print()
    if all(v is True for v in rel.values()):
        print("VERDICT: all six relational opcodes are BARE -- one byte, no TYPE.")
    elif any(v is False for v in rel.values()):
        print(f"VERDICT: NOT uniformly bare -- {rel}. The exclusion stands; fix the doc.")
    else:
        missing = [f"{o:02x}" for o, v in rel.items() if v is None]
        print(f"VERDICT: INCOMPLETE -- no site for {missing}. Widen the probe, do not infer.")


def hist(path):
    h = collections.Counter()
    n_tu = 0
    for line in open(path):
        d = json.loads(line)
        if "record" in d:
            continue
        n_tu += 1
        for k, v in (d.get("fn_blockers") or {}).items():
            h[k] += v
    return h, n_tu


WHOLE = re.compile(r"-whole\d*$")


def cmd_diff(base, other, label, top):
    a, ntu_a = hist(base)
    b, ntu_b = hist(other)
    delta = sorted(
        ((k, b[k] - a[k]) for k in set(a) | set(b) if b[k] != a[k]), key=lambda x: -x[1]
    )
    total = sum(d for _, d in delta)
    print(f"\n===== {label}:  {os.path.basename(base)} -> {os.path.basename(other)} =====")
    print(f"  TUs                         {ntu_a} -> {ntu_b}")
    print(f"  total blocked functions     {sum(a.values())} -> {sum(b.values())}")
    print(f"  distinct blocker keys       {len(a)} -> {len(b)}")
    wa = sum(v for k, v in a.items() if WHOLE.search(k))
    wb = sum(v for k, v in b.items() if WHOLE.search(k))
    print(f"  whole-body completeness     {wa} -> {wb}   (delta {wb - wa:+d})")
    print()
    print(f"  >>> SUM OF ALL KEY DELTAS = {total}   <<<  THE FALSIFIER: must be exactly 0.")
    print("      A byte that is not one byte wide desyncs the matcher and scatters")
    print("      its row across the hex tail; the sum then moves off 0.")
    if total != 0:
        print("      *** NON-ZERO -- the bareness reading is REFUTED by this run. ***")
    print()
    print(f"  top {top} gains / losses:")
    for k, d in delta[:top]:
        print(f"    {d:+8d}  {k}")
    if len(delta) > 2 * top:
        print(f"    ... {len(delta) - 2 * top} keys between ...")
    for k, d in delta[-top:]:
        print(f"    {d:+8d}  {k}")
    # the cmp family's own outcome split, which is the number a rung gets ranked by
    out = collections.Counter()
    for k, v in b.items():
        m = re.search(r"-(cmp-\w\w)-and-", k)
        if m:
            out["whole" if WHOLE.search(k) else "more"] += v
    if out:
        tot = out["whole"] + out["more"]
        print()
        print(f"  cmp-family completeness after the grant: {out['whole']} whole / {tot} "
              f"({100.0 * out['whole'] / tot:.1f} %)")
        solo = sum(v for k, v in b.items() if "cmp-" in k and re.search(r"-whole$", k))
        print(f"  ...of which complete on the COMPARE ALONE (-whole, one admission): {solo}")
        print("     (0 here means compare+bool-result is ONE rung, not two)")


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--emit-probe", metavar="PATH",
                    help="write the relational probe .cpp (feed it to `c2rs census --keep-il`)")
    ap.add_argument("--il", help="a `c2rs census --keep-il` directory: read operator widths")
    ap.add_argument("--base", help="baseline `c2rs gap --jsonl` scan")
    ap.add_argument("--grant", help="scan with BARE_BINARY_OPS extended by 1F-24")
    ap.add_argument("--parser", help="scan with 1F-24 also admitted in parse_expr")
    ap.add_argument("--top", type=int, default=15)
    a = ap.parse_args()
    if a.emit_probe:
        os.makedirs(os.path.dirname(os.path.abspath(a.emit_probe)), exist_ok=True)
        with open(a.emit_probe, "w") as f:
            f.write(PROBE_CPP)
        print(f"wrote {a.emit_probe}")
        print(f"next:  c2rs census {a.emit_probe} --keep-il <dir>")
        print(f"then:  {sys.argv[0]} --il <dir>")
    if not (a.il or a.base):
        if a.emit_probe:
            return
        ap.error("need --emit-probe, --il and/or --base")
    if a.il:
        cmd_il(a.il)
    if a.base and a.grant:
        cmd_diff(a.base, a.grant, "mcall completeness grant (1F-24 bare)", a.top)
    if a.base and a.parser:
        cmd_diff(a.base, a.parser, "parse_expr grant (where the free-standing row goes)", a.top)


if __name__ == "__main__":
    main()
