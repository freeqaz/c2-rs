#!/usr/bin/env python3
"""build_funcs.py — generate docs/whitebox/ref/FUNCS.tsv, the WHOLE-IMAGE index.

`ADDR.tsv` is bounded by PROSE: `build_ref.py` writes a row only for an address
that is already cited in `docs/` or already carries a hand label
(`addrs = set(cites) | set(labels)`).  That is the right shape for "what is
already known about this address" and the wrong shape for "I am holding an
address nobody has ever written about".  At base `e82c9ede6` that second case is
**4,287 of the image's 4,919 functions** — 87.1 % — and for every one of them the
reference answers with silence.

This script writes the complement: **one row per function in the image**, with
the four things that are derivable by mechanism and that a lane actually needs
before it decides whether to read the body —

  * **where it sits**: the ICE-derived translation unit, and whether that
    attribution is a fact (`in-anchor`) or a hypothesis (`gap`);
  * **whether anyone has been here**: paged / labelled / cited / none, plus the
    label text when there is one;
  * **who talks to it**: caller and callee counts, and `hop`, the call-graph
    distance to the nearest function the record already covers;
  * **what it touches**: the string literals and the imported CRT/Win32
    functions it references — the two hooks that most often identify a function
    without reading a single instruction.

## Provenance, and it is not uniform across the columns

* `tu`, `label`, `page`, `hop`, and every address: **TIER 2, white-box.**
* the 53 **file names**: **TIER 1** — c2's C1001 path prints `compiler file
  '%s'`, so `strings c2.dll` is sufficient (`DISCLOSURE.md`).
* the `strings` column: **TIER 1 text at a TIER 2 address.**  The literal is
  plain `strings` output; *which function references it* is white-box.
* the `imports` column: **TIER 1** — the import directory is public PE metadata.

## What `conf = mech` means, and why it is a separate value

A row this script fills in by joining tables carries confidence **`mech`**.  It
is weaker than `[R]`: `[R]` asserts *"the instructions were read correctly"*,
`mech` asserts only *"these tables join here"*.  `ref/README.md` §2 prices why
the distinction is not bureaucracy — a claim read correctly out of a small clean
function was still **wrong about c2** — and a mechanical join is a rung below
that.  Rows that inherit a hand label keep the hand label's confidence.

NAVIGATION ONLY.  Nothing here may enter `crates/` without a `DISCLOSURE.md`
row naming the address.

Tooling, like `build_ref.py` and `plot_perf.py` — outside the workspace's
std-only Rust constraint.

Usage:
    python3 docs/whitebox/scripts/build_funcs.py [repo-root] [export-dir]

`export-dir` defaults to `$C2RS_GHIDRA_EXPORT`, then `~/ghidra-projects/export/c2`.
The export is machine-local and is never committed.
"""

import os
import sys
import glob
import importlib.util
from collections import defaultdict, deque

HERE = os.path.dirname(os.path.abspath(__file__))


def _load_build_ref():
    """Import build_ref.py as a module.

    Deliberate: TU_PAGE, RANGE_PAGE, RANGE_TU and GHIDRA_MISSED must have ONE
    definition.  Two copies of a subsystem map that drift is how `C2_MAP.md` §3E
    came to name the wrong containing function for the emit walk.
    """
    spec = importlib.util.spec_from_file_location(
        "build_ref", os.path.join(HERE, "build_ref.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


BR = _load_build_ref()

MAX_STR = 3          # string literals shown per row
MAX_IMP = 3          # imports shown per row
STR_CLIP = 46        # per-literal clip
HOP_MAX = 6          # BFS ceiling; beyond this the answer is "far", not a number


def load_strings(export):
    """function addr -> [literal, ...], from strings.tsv's xref_funcs column."""
    out = defaultdict(list)
    path = os.path.join(export, "strings.tsv")
    if not os.path.exists(path):
        return out
    for r in BR.read_tsv(path, skip_comments=False):
        if r[0] == "addr" or len(r) < 6:
            continue
        text = r[5]
        for fa in r[4].split(","):
            fa = fa.strip()
            if not fa or fa == "-":
                continue
            try:
                out[int(fa, 16)].append(text)
            except ValueError:
                continue
    return out


def load_imports(export):
    """caller addr -> {import name}.  calls.tsv names EXTERNAL callees."""
    out = defaultdict(set)
    path = os.path.join(export, "calls.tsv")
    for r in BR.read_tsv(path, skip_comments=False):
        if r[0] == "caller_addr" or len(r) < 4:
            continue
        if not r[2].startswith("EXTERNAL"):
            continue
        try:
            a = int(r[0], 16)
        except ValueError:
            continue
        out[a].add(r[3])
    return out


def clip(s, n):
    s = s.replace("\t", " ").replace("\n", "\\n").replace("\r", "")
    return s if len(s) <= n else s[:n - 1] + "\u2026"


def bfs_hops(seeds, callers, callees, limit):
    """Undirected call-graph distance from the covered set.

    Undirected on purpose: a lane holding an unknown function wants "two calls
    away from the register allocator" whichever direction the edge points.
    """
    dist = {a: 0 for a in seeds}
    q = deque(seeds)
    while q:
        a = q.popleft()
        d = dist[a]
        if d >= limit:
            continue
        for b in list(callees.get(a, ())) + list(callers.get(a, ())):
            if b not in dist:
                dist[b] = d + 1
                q.append(b)
    return dist


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else os.path.abspath(
        os.path.join(HERE, "..", "..", ".."))
    export = (sys.argv[2] if len(sys.argv) > 2
              else os.environ.get("C2RS_GHIDRA_EXPORT")
              or os.path.expanduser("~/ghidra-projects/export/c2"))

    need = ("functions.tsv", "calls.tsv", "strings.tsv")
    missing = [f for f in need if not os.path.exists(os.path.join(export, f))]
    if missing:
        sys.stderr.write(
            "build_funcs.py: flat export incomplete at %s (missing %s).\n"
            "Set C2RS_GHIDRA_EXPORT or regenerate per C2_MAP_METHOD.md §3-4.\n"
            % (export, ", ".join(missing)))
        return 2

    fns = BR.load_functions(export)
    ghidra_n = len(fns)
    thunks = set()
    for r in BR.read_tsv(os.path.join(export, "functions.tsv"),
                         skip_comments=False):
        if r[0] == "addr" or len(r) < 8:
            continue
        if r[7] == "thunk":
            try:
                thunks.add(int(r[0], 16))
            except ValueError:
                pass

    missed = {}
    for entry, end, name, why in BR.GHIDRA_MISSED:
        if entry not in fns:
            fns[entry] = (end - entry, name)
            missed[entry] = why

    callers, callees = BR.load_calls(export)
    tus = BR.load_tus(root)
    labels = BR.load_labels(root)
    c2fn = BR.load_c2_functions(root)
    cites = BR.scan_citations(root)
    owned = BR.scan_pages(root)
    strs = load_strings(export)
    imps = load_imports(export)

    sorted_fns = sorted((a, s, n) for a, (s, n) in fns.items())

    # An ADDRESS is covered; a FUNCTION is covered when any address inside it is.
    # Rolling that up is the whole point: `ADDR.tsv` answers about addresses and
    # a lane arrives holding a function.
    f_paged, f_labelled, f_cited = {}, {}, set()
    for a in set(cites) | set(labels) | set(owned):
        c = BR.containing(sorted_fns, a)
        key = a if a in fns else (c[0] if c else None)
        if key is None:
            continue
        if a in owned:
            f_paged.setdefault(key, owned[a])
        if a in labels:
            f_labelled.setdefault(key, labels[a])
        if a in cites:
            f_cited.add(key)

    covered = set(f_paged) | set(f_labelled)
    hops = bfs_hops(covered, callers, callees, HOP_MAX)

    out_path = os.path.join(root, "docs/whitebox/ref/FUNCS.tsv")
    cols = ["addr", "size", "kind", "tu", "tu_conf", "subsys", "page", "cover",
            "conf", "label", "ncallers", "ncallees", "hop", "nstr", "strings",
            "nimp", "imports"]

    stat = defaultdict(int)
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write("# DISASSEMBLY-DERIVED (tier 2: addresses, tu, label, hop;\n"
                 "# tier 1: the 53 file names, the string TEXT, the import names).\n"
                 "# GENERATED by docs/whitebox/scripts/build_funcs.py -- do not hand-edit.\n"
                 "# ONE ROW PER FUNCTION IN THE IMAGE -- the complement of ADDR.tsv,\n"
                 "# which has a row only for an address already cited or labelled.\n"
                 "# Every addr is an absolute VA in c2.dll sha256 %s.\n"
                 "# NAVIGATION ONLY.  Nothing here may enter crates/ without a\n"
                 "# docs/whitebox/DISCLOSURE.md row naming the address.\n"
                 "# cover:   paged > labelled > cited > none\n"
                 "# conf:    a hand label's own confidence, else `mech` -- WEAKER than [R].\n"
                 "#          [R] says the instructions were read correctly; `mech` says only\n"
                 "#          that these tables join here.\n"
                 "# tu_conf: in-anchor (a fact) | gap (a hypothesis, C2_MAP.md 3.1)\n"
                 "#          no-ice-site (a TU the partition cannot see) | n/a\n"
                 "# hop:     undirected call-graph distance to the nearest paged/labelled\n"
                 "#          function; 0 = itself, >%d prints as `%d+`, `-` = unreachable.\n"
                 % (BR.C2_SHA256, HOP_MAX, HOP_MAX))
        fh.write("\t".join(cols) + "\n")

        for a, size, gname in sorted_fns:
            tu_name, tu_conf = BR.tu_for(tus, a)
            sub, page = BR.page_for(a, tu_name, tu_conf, owned)
            if a in f_paged:
                page = f_paged[a]
                sub = BR.PAGE_SUBSYS.get(page, sub)

            lab = f_labelled.get(a)
            if lab:
                cover = "paged" if a in f_paged else "labelled"
                conf = lab[1] or "unknown"
                text = (lab[0] + ": " + lab[2]) if lab[2] else lab[0]
            elif a in f_paged:
                cover, conf, text = "paged", "mech", ""
            elif a in f_cited:
                cover, conf, text = "cited", "mech", ""
            else:
                cover, conf, text = "none", "mech", ""
            if not text:
                meta = c2fn.get(a)
                if meta and len(meta) >= 3 and meta[2] not in ("unknown", ""):
                    text = meta[2]
                    if len(meta) > 3 and meta[3] not in ("", "unknown"):
                        conf = meta[3]
            stat[cover] += 1

            kind = ("ghidra-missed" if a in missed
                    else "thunk" if a in thunks else "ghidra")

            h = hops.get(a)
            hs = "-" if h is None else (str(h) if h < HOP_MAX else "%d+" % HOP_MAX)

            sl = strs.get(a, [])
            il = sorted(imps.get(a, ()))
            if sl:
                stat["has_str"] += 1
            if il:
                stat["has_imp"] += 1
            if sl or il:
                stat["has_strong_hook"] += 1
            if tu_conf == "in-anchor":
                stat["tu_fact"] += 1
            if hs != "-":
                stat["hop_reachable"] += 1

            fh.write("\t".join([
                "%08x" % a, str(size), kind, tu_name, tu_conf, sub, page,
                cover, conf, clip(text, 200) or "-",
                str(len(callers.get(a, ()))), str(len(callees.get(a, ()))), hs,
                str(len(sl)),
                " | ".join(clip(s, STR_CLIP) for s in sl[:MAX_STR]) or "-",
                str(len(il)), ",".join(il[:MAX_IMP]) or "-",
            ]) + "\n")

    total = len(sorted_fns)
    sys.stderr.write(
        "build_funcs.py: %d rows -> %s\n"
        "  ghidra functions %d  + ghidra-missed %d  = %d\n"
        "  cover: paged %d  labelled %d  cited %d  none %d\n"
        "  tu attribution in-anchor (a fact): %d = %.1f%%\n"
        "  strong hook (a string or an import): %d = %.1f%%"
        "   [string %d, import %d]\n"
        "  reachable within %d call hops of a paged/labelled function: %d = %.1f%%\n"
        % (total, os.path.relpath(out_path, root),
           ghidra_n, len(missed), total,
           stat["paged"], stat["labelled"], stat["cited"], stat["none"],
           stat["tu_fact"], 100.0 * stat["tu_fact"] / total,
           stat["has_strong_hook"], 100.0 * stat["has_strong_hook"] / total,
           stat["has_str"], stat["has_imp"],
           HOP_MAX, stat["hop_reachable"], 100.0 * stat["hop_reachable"] / total))
    return 0


if __name__ == "__main__":
    sys.exit(main())
