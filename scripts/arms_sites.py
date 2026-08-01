#!/usr/bin/env python3
"""Decompose the receiver-designator `prod` sites from a `c2rs gap` row dump.

Tooling, deliberately outside the std-only Rust workspace (same standing as
`scripts/rerank_board.py`, which this borrows `clean` from). It answers the one
question board #142 asks and no histogram can: of the emitted functions blocked
at a receiver designator, **which construct actually stands there**, crossed
with `clean`, with completeness, and with the census key the row was filed
under.

`clean` is a JOINT (`cflow-straight*` AND `eh-none` AND `calls<2`) and the gap
report prints each axis as its own map, so it is unanswerable from the report
and answerable from one pass over the dump — `gap.rs::row_dump`'s own argument
for existing.

Usage:
    arms_sites.py sites  <dump.tsv>
    arms_sites.py arms   <dump.tsv>            # needs the refined `prod` axis
    arms_sites.py keys   <dump.tsv> <prod-prefix>
    arms_sites.py row    <dump.tsv> <key> [n]  # print n witness rows with hex
    arms_sites.py agree  <dump.tsv>            # census key vs receiver construct
"""

import collections
import sys

# The three receiver-designator sites, as `prod` values. After this lane's
# refinement each is a PREFIX of a family (`…/<construct>`), so every lookup
# below is a prefix test and the pre-refinement dumps still read correctly.
SITES = (
    "tail-recv-not-a-plain-b9-load",
    "chain-recv-not-a-plain-b9-load",
    "cmp-second-recv-not-a-plain-b9-load",
)


def site_of(prod):
    for s in SITES:
        if prod == s or prod.startswith(s + "/"):
            return s
    return None


def clean(frame, cflow, eh):
    return cflow.startswith("cflow-straight") and eh == "eh-none" and frame != "calls-2plus"


def rows(path, emitted_only=True):
    with open(path) as f:
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) < 11:
                continue
            r = dict(
                src=p[0], idx=p[1], key=p[2], emitted=(p[3] == "EMITTED"), name=p[4],
                frame=p[5], cflow=p[6], eh=p[7], disp=p[8], prod=p[9], comp=p[10],
                hex_mark=int(p[11]) if len(p) > 11 and p[11].isdigit() else 0,
                hex=p[12] if len(p) > 12 else "",
            )
            if emitted_only and not r["emitted"]:
                continue
            yield r


def is_complete(comp):
    """The `Complete` field's whole-body readings (§9.14.4's closed vocabulary)."""
    return comp.startswith("complete-whole")


def cmd_sites(path):
    tot = collections.Counter()
    cln = collections.Counter()
    cc = collections.Counter()
    comps = collections.Counter()
    n = 0
    for r in rows(path):
        n += 1
        s = site_of(r["prod"])
        if s is None:
            continue
        tot[s] += 1
        if clean(r["frame"], r["cflow"], r["eh"]):
            cln[s] += 1
            comps[r["comp"]] += 1
            if is_complete(r["comp"]):
                cc[s] += 1
    print(f"emitted rows in the dump: {n}")
    print(f"{'site':<40}{'emitted':>9}{'clean':>8}{'clean&complete':>16}")
    for s in SITES:
        print(f"  {s:<38}{tot[s]:>9}{cln[s]:>8}{cc[s]:>16}")
    print(f"  {'TOTAL':<38}{sum(tot.values()):>9}{sum(cln.values()):>8}{sum(cc.values()):>16}")
    print("\ncompleteness of the clean stock:")
    for k, v in comps.most_common():
        print(f"  {v:>7}  {k}")


def cmd_arms(path):
    """The decomposition #142 asks for: emitted / clean / clean&complete per arm."""
    tot = collections.Counter()
    cln = collections.Counter()
    cc = collections.Counter()
    names = collections.defaultdict(set)
    for r in rows(path):
        s = site_of(r["prod"])
        if s is None:
            continue
        arm = r["prod"].split("/", 1)[1] if "/" in r["prod"] else "(unrefined)"
        tot[arm] += 1
        if clean(r["frame"], r["cflow"], r["eh"]):
            cln[arm] += 1
            names[arm].add(r["name"])
            if is_complete(r["comp"]):
                cc[arm] += 1
    te, tc, tk = sum(tot.values()), sum(cln.values()), sum(cc.values())
    print(f"{'receiver construct':<34}{'emitted':>9}{'clean':>8}{'cln&cmp':>9}"
          f"{'%clean':>8}{'names':>8}")
    for arm, e in tot.most_common():
        print(f"  {arm:<32}{e:>9}{cln[arm]:>8}{cc[arm]:>9}"
              f"{100.0 * cln[arm] / max(tc, 1):>7.1f}%{len(names[arm]):>8}")
    print(f"  {'TOTAL':<32}{te:>9}{tc:>8}{tk:>9}")


def cmd_keys(path, prefix):
    keys = collections.Counter()
    ckeys = collections.Counter()
    for r in rows(path):
        if not (r["prod"] == prefix or r["prod"].startswith(prefix)):
            continue
        keys[r["key"]] += 1
        if clean(r["frame"], r["cflow"], r["eh"]):
            ckeys[r["key"]] += 1
    print(f"{'census key':<62}{'emitted':>9}{'clean':>8}")
    for k, v in keys.most_common(25):
        print(f"  {k:<60}{v:>9}{ckeys[k]:>8}")
    print(f"  {'TOTAL':<60}{sum(keys.values()):>9}{sum(ckeys.values()):>8}")


def cmd_row(path, key, n=6):
    seen = 0
    for r in rows(path):
        if r["key"] != key and site_of(r["prod"]) != key and r["prod"] != key:
            continue
        print(f"{r['src']}#{r['idx']} {r['name']}")
        print(f"    key={r['key']} prod={r['prod']} comp={r['comp']} "
              f"{r['frame']} {r['cflow']} {r['eh']} {r['disp']}")
        print(f"    mark={r['hex_mark']} hex={r['hex']}")
        seen += 1
        if seen >= n:
            return


# ---- the C-item control: does the census key name the receiver construct? ----
#
# The key is minted by whichever reader stopped LAST; the receiver construct is
# where the member-call production stopped FIRST. §9.13 found those are not the
# same reader. This crosses them, per row, so the claim is a measurement.
#
# The mapping is deliberately GENEROUS to the "keys are trustworthy" hypothesis:
# any key whose text contains the construct's own noun counts as agreement. A
# control has to be able to come out both ways, and stacking the definition
# against the hypothesis you expect to win is how absence gets read as success.
NOUN = {
    "intrinsic-this-adjust": ("this-adjust", "intrinsic"),
    "intrinsic-base-member": ("base-member", "intrinsic"),
    "intrinsic-other": ("intrinsic",),
    "off-add": ("op-0x27", "op-0x28", "off-add"),
    "deref-load": ("op-0x30", "deref-load"),
    "plain-call": ("op-0xbd", "plain-call", "call"),
    "call-in-expr": ("op-0x26", "call-in-expr", "call"),
    "virtual": ("op-0x67", "op-0x9a", "virtual"),
    "convert": ("op-0x2c", "convert"),
    "temp-bind": ("op-0x9b", "temp-bind"),
    "ternary": ("op-0x43", "ternary"),
    "class-descriptor": ("op-0x66", "class-descriptor"),
}


def nouns_for(arm):
    """The words a census key could use for this receiver construct."""
    base = arm.split("-op-0x")[0]
    for k, v in NOUN.items():
        if base.endswith(k):
            return v
    if "-op-0x" in arm:
        return ("op-0x" + arm.split("-op-0x")[1],)
    return ()


def cmd_agree(path):
    agree = 0
    disagree = 0
    undecidable = 0
    dis = collections.Counter()
    agr = collections.Counter()
    for r in rows(path):
        if site_of(r["prod"]) is None or "/" not in r["prod"]:
            continue
        if not clean(r["frame"], r["cflow"], r["eh"]):
            continue
        if is_complete(r["comp"]):
            continue
        arm = r["prod"].split("/", 1)[1]
        ns = nouns_for(arm)
        if not ns:
            undecidable += 1
            continue
        key = r["key"].lower()
        if any(n in key for n in ns):
            agree += 1
            agr[(arm, r["key"])] += 1
        else:
            disagree += 1
            dis[(arm, r["key"])] += 1
    tot = agree + disagree + undecidable
    print(f"clean-not-complete rows at the three sites: {tot}")
    print(f"  key NAMES the receiver construct : {agree:>7} ({100.0*agree/max(tot,1):.1f}%)")
    print(f"  key names something ELSE         : {disagree:>7} ({100.0*disagree/max(tot,1):.1f}%)")
    print(f"  undecidable (no noun for the arm): {undecidable:>7}")
    print("\n  top DISAGREEING (construct, key) pairs — the second-reader stops:")
    for (a, k), v in dis.most_common(12):
        print(f"    {v:>7}  {a:<28} key={k}")
    print("\n  top AGREEING pairs:")
    for (a, k), v in agr.most_common(8):
        print(f"    {v:>7}  {a:<28} key={k}")


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    cmd = sys.argv[1]
    if cmd == "sites":
        cmd_sites(sys.argv[2])
    elif cmd == "arms":
        cmd_arms(sys.argv[2])
    elif cmd == "keys":
        cmd_keys(sys.argv[2], sys.argv[3])
    elif cmd == "row":
        cmd_row(sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 6)
    elif cmd == "agree":
        cmd_agree(sys.argv[2])
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
