#!/usr/bin/env python3
"""w-seclayout — the join the commission asks for, BY NAME.

For one READ TU: the records a counterfactual walk (26-stop removed) would hand
`Bindings::per_record` / `::selective`, against the function symbols c2's real
obj actually carries, and the aux `Selection` byte of each one's COMDAT.

Prints three sets:
  * EMITTED and NAMED   — a record exists and c2 emitted the symbol
  * NAMED but NOT EMITTED — the port would emit a function c2 discarded
                            (#232's direction: a refusal becoming a wrong emit)
  * EMITTED but NOT NAMED — the port would omit a function c2 emitted

and the `26`-introduction / linkage / flags bytes against the Selection byte,
which is the question "is this a routing problem".

  emitjoin.py <name>       (reads work/w-seclayout/{cap,obj}/<name>)
"""
import glob
import sys

sys.path.insert(0, "work/w-seclayout")
from seclayout import read_obj, IMAGE_SCN_LNK_COMDAT, SEL  # noqa: E402


def dstem(n):
    """#2243's template collapse: `??$NAME@<args>` -> `??$NAME`, and any
    mangled name -> its leading `?…@` identifier + class chain, so a count of
    distinct names does not fail open on template instantiations."""
    if n.startswith("??$"):
        return n.split("@", 1)[0]
    if n.startswith("?"):
        parts = n.split("@@", 1)
        head = parts[0]
        # collapse a template argument list inside the class chain too
        return head.split("@?$", 1)[0] if "@?$" in head else head
    return n


def main():
    name = sys.argv[1]
    tsv = f"work/w-seclayout/cap/{name}/walk.tsv"
    rows = []
    for line in open(tsv).read().splitlines()[1:]:
        pos, start, verdict, lk, fl, i26, nm = line.split("\t")
        rows.append((int(start), verdict, lk, fl, i26 == "1", nm))

    secs = read_obj(glob.glob(f"work/w-seclayout/obj/{name}.obj")[0])
    emitted = {}
    for s in secs:
        if s["name"] != ".text":
            continue
        for sym, _v in s["syms"]:
            emitted[sym] = (s["n"], s["sel"], bool(s["chars"] & IMAGE_SCN_LNK_COMDAT))

    named = {r[5] for r in rows}
    print(f"== {name}:  {len(rows)} counterfactual records, "
          f"{len(emitted)} function symbols in {sum(1 for s in secs if s['name'] == '.text')} `.text` sections")

    both = sorted(named & set(emitted))
    only_named = sorted(named - set(emitted))
    only_emitted = sorted(set(emitted) - named)
    print(f"   EMITTED and NAMED     : {len(both)}")
    print(f"   NAMED but NOT EMITTED : {len(only_named)}   "
          f"<- the port would emit these; c2 did not  (#232's direction)")
    for n in only_named:
        r = next(x for x in rows if x[5] == n)
        print(f"        26={int(r[4])} lk={r[2]} fl={r[3]}  {n}")
    print(f"   EMITTED but NOT NAMED : {len(only_emitted)}   "
          f"<- the port would omit a body c2 emitted")
    for n in only_emitted:
        print(f"        {n}")

    # Does the `26` introduction predict the aux Selection byte?
    print("   --- does `26`-introduction predict the COMDAT Selection byte?")
    tab = {}
    for start, verdict, lk, fl, i26, nm in rows:
        if nm not in emitted:
            continue
        sel = emitted[nm][1]
        tab[(i26, sel)] = tab.get((i26, sel), 0) + 1
    for (i26, sel), n in sorted(tab.items()):
        print(f"        26-introduced={int(i26)}  Selection={SEL.get(sel, sel)}({sel})  x{n}")
    print("   --- does the `.gl` FLAGS byte (name_nul+5) predict it?")
    tab = {}
    for start, verdict, lk, fl, i26, nm in rows:
        if nm not in emitted:
            continue
        tab[(lk, fl, emitted[nm][1])] = tab.get((lk, fl, emitted[nm][1]), 0) + 1
    for (lk, fl, sel), n in sorted(tab.items(), key=lambda kv: str(kv[0])):
        print(f"        linkage={lk} flags={fl} -> Selection={SEL.get(sel, sel)}({sel})  x{n}")

    # #2243's stem collapse over the `26`-introduced names.
    i26names = [r[5] for r in rows if r[4]]
    stems = {dstem(n) for n in i26names}
    print(f"   --- #2243 stem column: {len(i26names)} `26`-introduced names -> "
          f"{len(set(i26names))} distinct names -> {len(stems)} distinct stems")


main()
