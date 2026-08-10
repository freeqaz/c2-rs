#!/usr/bin/env python3
"""Grade the candidate rule
      Selection = SELECT_ANY(2) if (.gl flags & 0x20) else NODUPLICATES(1)
over a set of TUs: every framed record the GATE's own walk would BIND, joined
by name to the `.text` COMDAT that actually carries it in c2's obj.

  selrule.py <name> [<name> ...]

Reports agreements, disagreements, and — separately, because it is the control
that matters — whether any BYTE-EXACTLY-MATCHING TU carries a `flags & 0x20`
record, which would refute the rule and would break a match if it shipped.
"""
import glob
import sys

sys.path.insert(0, "work/w-seclayout")
from seclayout import read_obj, IMAGE_SCN_LNK_COMDAT  # noqa: E402

NODUP, ANY = 1, 2
FLAGS_COMDAT = 0x20


def grade(name):
    tsv = f"work/w-seclayout/cap/{name}/walk.tsv"
    objp = glob.glob(f"work/w-seclayout/obj/{name}.obj")
    if not objp:
        return None
    secs = read_obj(objp[0])
    emitted = {}
    for s in secs:
        if s["name"] != ".text":
            continue
        for sym, _v in s["syms"]:
            emitted[sym] = (s["sel"], bool(s["chars"] & IMAGE_SCN_LNK_COMDAT))
    agree = dis = unread = 0
    bad = []
    for line in open(tsv).read().splitlines()[1:]:
        _pos, _st, verdict, _lk, fl, _i26, nm = line.split("\t")
        if nm not in emitted:
            continue
        sel, is_comdat = emitted[nm]
        if not is_comdat:
            continue
        if fl == "":
            unread += 1
            continue
        want = ANY if int(fl) & FLAGS_COMDAT else NODUP
        if want == sel:
            agree += 1
        else:
            dis += 1
            bad.append((nm, fl, sel, want))
    return agree, dis, unread, bad


def main():
    tot_a = tot_d = tot_u = 0
    for name in sys.argv[1:]:
        r = grade(name)
        if r is None:
            print(f"  {name:<14} (no obj)")
            continue
        a, d, u, bad = r
        tot_a += a
        tot_d += d
        tot_u += u
        flag = "" if d == 0 else "   <-- REFUTES"
        print(f"  {name:<14} agree {a:>4}  disagree {d:>3}  unreadable {u:>3}{flag}")
        for nm, fl, sel, want in bad[:8]:
            print(f"        flags={fl} obj-sel={sel} rule-says={want}  {nm}")
    print(f"  TOTAL          agree {tot_a}  disagree {tot_d}  unreadable {tot_u}")


main()
