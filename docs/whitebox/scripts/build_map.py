#!/usr/bin/env python3
"""build_map.py — generate docs/whitebox/c2_functions.tsv from the flat Ghidra export.

PROVENANCE: output is DISASSEMBLY-DERIVED. See docs/whitebox/C2_MAP.md and
docs/whitebox/DISCLOSURE.md. Nothing produced here may be pasted into crates/
without a disclosure entry.

Usage:
  python3 build_map.py <export-dir> <labels-dir> <out.tsv>

<export-dir>  the flat export produced by ExportFlat.java (functions.tsv,
              calls.tsv, strings.tsv, xrefs.tsv)
<labels-dir>  directory of *.tsv label files contributed by analysis children,
              rows: addr <TAB> cluster <TAB> confidence <TAB> evidence
              (lines starting with '#' ignored). These OVERRIDE mechanical
              clusters.
<out.tsv>     destination

Mechanical clusters are derived only from facts that require no judgement
(thunk-ness, which imported DLL a function calls, which literals it references).
Everything else is emitted with cluster/confidence 'unknown' by design: an
unlabelled row is a correct row, a guessed row is not.
"""
import sys, os, collections

def main():
    exp, labdir, outp = sys.argv[1], sys.argv[2], sys.argv[3]

    funcs = []
    with open(os.path.join(exp, "functions.tsv")) as f:
        hdr = f.readline()
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) < 9:
                continue
            funcs.append(p)

    # --- which imported DLL does each function call directly? ---
    # calls.tsv rows whose callee address is EXTERNAL:* are import calls; the
    # owning DLL comes from symbols.tsv's namespace column.
    dll_of = {}
    with open(os.path.join(exp, "symbols.tsv")) as f:
        f.readline()
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) >= 5 and p[0].startswith("EXTERNAL:"):
                ns = p[4].split("::")[0].upper()
                dll_of[p[1]] = ns

    imports_by_func = collections.defaultdict(set)
    callees_by_func = collections.defaultdict(set)
    with open(os.path.join(exp, "calls.tsv")) as f:
        f.readline()
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) < 4:
                continue
            caller, callee_a, callee_n = p[0], p[2], p[3]
            callees_by_func[caller].add(callee_a)
            if callee_a.startswith("EXTERNAL:"):
                d = dll_of.get(callee_n)
                if d:
                    imports_by_func[caller].add((d, callee_n))

    # --- literals referenced by each function (navigational evidence) ---
    strs_by_func = collections.defaultdict(list)
    with open(os.path.join(exp, "strings.tsv")) as f:
        f.readline()
        for line in f:
            p = line.rstrip("\n").split("\t")
            if len(p) < 6:
                continue
            val = p[5]
            for fa in [x for x in p[4].split(",") if x]:
                strs_by_func[fa].append(val)

    # --- child-contributed labels override ---
    labels = {}
    if os.path.isdir(labdir):
        for fn in sorted(os.listdir(labdir)):
            if not fn.endswith(".tsv"):
                continue
            with open(os.path.join(labdir, fn)) as f:
                for line in f:
                    line = line.rstrip("\n")
                    if not line or line.startswith("#") or line.lower().startswith("addr\t"):
                        continue
                    p = line.split("\t")
                    if len(p) < 3:
                        continue
                    a = p[0].strip().lower().replace("0x", "")
                    ev = p[3] if len(p) > 3 else ""
                    labels[a] = (p[1].strip(), p[2].strip(),
                                 (ev + " [src:" + fn.rsplit('.', 1)[0] + "]").strip())

    DLLCLUS = {
        "MSDISXXX.DLL": "msdis-client",
        "MSOBJXX.DLL": "msobj-client",
        "MSPDBXX.DLL": "pdb-client",
        "PGODB100.DLL": "pgo-client",
    }

    # ---- pass 1: mechanical cluster per function ----
    mech = {}
    for p in funcs:
        addr, name, thunk = p[0], p[2], p[7]
        if thunk == "thunk":
            mech[addr] = ("thunk", "high", "ghidra-resolved thunk: " + name)
            continue
        imps = imports_by_func.get(addr, set())
        dlls = {d for d, _ in imps}
        hits = sorted({DLLCLUS[d] for d in dlls if d in DLLCLUS})
        clus, conf = ("unknown", "unknown")
        if len(hits) == 1:
            clus, conf = hits[0], "high"
        elif len(hits) > 1:
            clus, conf = "+".join(hits), "high"
        evbits = []
        if imps:
            named = sorted(n for d, n in imps if d in DLLCLUS)
            if named:
                evbits.append("calls " + ",".join(named[:4]))
        ss = strs_by_func.get(addr, [])
        if ss:
            seen, uniq = set(), []
            for s in ss:
                if s not in seen:
                    seen.add(s)
                    uniq.append(s)
            evbits.append("str:" + " | ".join(x[:40] for x in uniq[:3]))
        mech[addr] = (clus, conf, "; ".join(evbits)[:220] if evbits else "-")

    # ---- pass 2: exclusive-reachability propagation ----
    # If every direct caller of F sits in one non-unknown cluster C, F is only
    # used by C. This is a mechanical fact about the call graph, not a guess
    # about what F does, so it is published at 'medium' and prefixed 'only-from:'.
    callers = collections.defaultdict(set)
    for caller, cs in callees_by_func.items():
        for c in cs:
            if not c.startswith("EXTERNAL:"):
                callers[c].add(caller)
    base = lambda c: c[len("only-from:"):] if c.startswith("only-from:") else c
    for _ in range(6):
        changed = 0
        for p in funcs:
            a = p[0]
            if mech[a][0] != "unknown":
                continue
            cs = callers.get(a, set())
            if not cs or len(cs) > 64:
                continue
            kinds = {base(mech[c][0]) for c in cs if c in mech}
            if len(kinds) == 1:
                k = kinds.pop()
                if k not in ("unknown", "thunk"):
                    mech[a] = ("only-from:" + k, "medium",
                               "all %d callers in cluster %s" % (len(cs), k))
                    changed += 1
        if not changed:
            break

    n_lab = collections.Counter()
    out = open(outp, "w")
    out.write("# DISASSEMBLY-DERIVED — generated by docs/whitebox/scripts/build_map.py.\n")
    out.write("# Do not hand-edit; do not paste any value into crates/ without a\n")
    out.write("# docs/whitebox/DISCLOSURE.md entry. 'unknown' is a valid, respectable value.\n")
    out.write("addr\tsize\tsymbol_or_empty\tcluster\tconfidence\tncallers\tncallees\tevidence\n")
    for p in funcs:
        addr, size, name = p[0], p[1], p[2]
        key = addr.lower()
        sym = "" if name.startswith("FUN_") else name
        if key in labels:
            clus, conf, ev = labels[key]
        else:
            clus, conf, ev = mech[addr]
        n_lab[clus] += 1
        out.write("\t".join([addr, size, sym, clus, conf, p[4], p[5], ev]) + "\n")
    out.close()

    tot = len(funcs)
    sys.stderr.write("functions: %d\n" % tot)
    for k, v in n_lab.most_common():
        sys.stderr.write("  %-24s %5d  (%.1f%%)\n" % (k, v, 100.0 * v / tot))
    sys.stderr.write("child-labelled rows: %d\n" % sum(1 for f in funcs if f[0].lower() in labels))


if __name__ == "__main__":
    main()
