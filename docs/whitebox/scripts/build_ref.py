#!/usr/bin/env python3
"""build_ref.py — generate docs/whitebox/ref/ADDR.tsv, the address index.

The whitebox record has five sources of truth about a `c2.dll` address and
nothing that joins them:

  1. `docs/whitebox/labels/*.tsv`      hand-earned labels  (cluster, confidence, note)
  2. `docs/whitebox/c2_functions.tsv`  the mechanical per-function table
  3. `docs/whitebox/c2_tus.tsv`        the ICE-derived translation-unit partition
  4. `$C2RS_GHIDRA_EXPORT/*.tsv`       the flat Ghidra export (functions, data, calls)
  5. `docs/**/*.md`                    ~1,000 addresses cited in prose

This script performs that join and writes one row per address that is either
cited in `docs/` or carries a hand label.  It is *navigation only*: it copies no
value out of the binary into `crates/` and needs no `DISCLOSURE.md` row.

Tooling, like `build_map.py` and `plot_perf.py` — outside the workspace's
std-only Rust constraint.

Usage:
    python3 docs/whitebox/scripts/build_ref.py [repo-root] [export-dir]

`export-dir` defaults to `$C2RS_GHIDRA_EXPORT`, then to
`~/ghidra-projects/export/c2`.  The export is machine-local and is never
committed; no absolute path is baked into this file.
"""

import os
import re
import sys
import glob
from collections import defaultdict

# The exact image every address below is a VA in.  Stated so a reader who has a
# different c2.dll knows immediately that none of this applies.
C2_SHA256 = "c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258"

IMG_LO = 0x10B00000
IMG_HI = 0x10C7D000
TEXT_CODE_LO = 0x10B266D0          # below this, .text is pooled read-only data
DATA_LO = 0x10C2E000

# An address citation, with or without the `0x` prefix.  The naive form
# `\b10[bc][0-9a-f]{5}\b` is WRONG in THREE ways and this lane's own PREREG
# denominator was built with it (ref/README.md 6.1):
#   * `\b` does not fire between `x` and `1`, so it MISSES every `0x10b9b8e9`;
#   * a plain unanchored grep MATCHES substrings inside the sha256 and inside
#     byte dumps;
#   * the record cites addresses as `FUN_10b8303c` / `DAT_10c2e234` /
#     `LAB_10c1bfe2` as often as bare, and `_` is a word character, so a
#     `\b`-anchored OR a `[^0-9A-Za-z_]`-anchored pattern misses all of those.
ADDR_RE = re.compile(
    r"(?:(?:FUN|DAT|LAB|SUB|UNK|PTR_DAT)_(10[bc][0-9a-f]{5})"
    r"|(?<![0-9A-Za-z_])(?:0[xX])?(10[bc][0-9a-f]{5}))(?![0-9A-Za-z_])")
# bare form, for the label tables whose first column is exactly the address
BARE_ADDR_RE = re.compile(r"10[bc][0-9a-f]{5}\Z")

# ---------------------------------------------------------------------------
# subsystem / page assignment
#
# Keyed on the ICE-derived TU name where one exists.  The scheduler band has no
# TU of its own -- it sits in an anchor gap between except.c and emit.cpp,
# because a translation unit with no C1001 site is invisible to c2_tus.tsv by
# construction (WB_DAGORDER_FINDINGS.md 1).  So it needs an explicit range.
# ---------------------------------------------------------------------------
TU_PAGE = {
    "coff.c":      ("coff",     "P_COFF.md"),
    "coffemit.c":  ("coff",     "P_COFF.md"),
    "p2symtab.c":  ("section",  "P_SECTION.md"),
    "emit.cpp":    ("section",  "P_SECTION.md"),
    "color.c":     ("regalloc", "P_REGALLOC.md"),
    "globregs.c":  ("regalloc", "P_REGALLOC.md"),
    "regasg.c":    ("regalloc", "P_REGALLOC.md"),
    "dag.c":       ("dag",      "P_DAG.md"),
    "inline.c":    ("inline",   "P_INLINE.md"),
    "ptinl.c":     ("inline",   "P_INLINE.md"),
    "ehexcept.c":  ("eh",       "P_EH.md"),
    "except.c":    ("eh",       "P_EH.md"),
    "ssa_seh.c":   ("eh",       "P_EH.md"),
}

# (lo, hi, subsys, page) -- overrides the TU mapping.  Half-open.
RANGE_PAGE = [
    (0x10BE5CCE, 0x10BE663F, "dag", "P_DAG.md"),   # the scheduler band, no TU
]

# TU-name overrides.  c2_tus.tsv is built from C1001 sites, so a translation
# unit with NO ICE site is invisible to it and its code is silently absorbed
# into the preceding file's gap (C2_MAP.md 7.1).  That is not hypothetical: the
# instruction scheduler is such a TU, and "there is no sched.c" -- a true
# statement about the INSTRUMENT -- stood as board #1823 for months as a claim
# about the IMAGE (WB_DAGORDER_FINDINGS.md 1).  Naming the known ones stops the
# index from repeating it.
RANGE_TU = [
    (0x10BE5CCE, 0x10BE663F, "(unnamed TU: no ICE site)"),
]

# Subsystem name for each page, for the `subsys` column.
PAGE_SUBSYS = {
    "P_COFF.md": "coff", "P_SECTION.md": "section", "P_REGALLOC.md": "regalloc",
    "P_DAG.md": "dag", "P_INLINE.md": "inline", "P_EH.md": "eh",
}

# Functions Ghidra's auto-analysis did NOT create, verified by hand in
# objdump_intel.asm.  Without these the addresses inside them resolve to
# nothing, which is how `C2_MAP.md` 3E came to say the emit walk loop is
# "inside FUN_10b7f1ff" -- it is not; it is inside a tail-jump target that
# Ghidra never made a function.  See ref/README.md 6.2.
# (entry, end_exclusive, name, why)
GHIDRA_MISSED = [
    (0x10B7F022, 0x10B7F1FF, "SUB_10b7f022",
     "the emit walk; tail-jump target of FUN_10b7f1ff (jmp at 0x10b7f362), "
     "no Ghidra function"),
]


def read_tsv(path, skip_comments=True):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if not line:
                continue
            if skip_comments and line.startswith("#"):
                continue
            rows.append(line.split("\t"))
    return rows


def load_functions(export):
    """entry addr -> (size, name).  Also returns a sorted list for containment."""
    fns = {}
    rows = read_tsv(os.path.join(export, "functions.tsv"))
    for r in rows:
        if r[0] == "addr":
            continue
        try:
            a = int(r[0], 16)
            sz = int(r[1])
        except ValueError:
            continue
        fns[a] = (sz, r[2])
    return fns


def load_data(export):
    """data addr -> (size, type)."""
    out = {}
    for r in read_tsv(os.path.join(export, "data.tsv")):
        if r[0] == "addr":
            continue
        try:
            a = int(r[0], 16)
            sz = int(r[1])
        except ValueError:
            continue
        out[a] = (sz, r[2] if len(r) > 2 else "")
    return out


def load_calls(export):
    callers = defaultdict(set)
    callees = defaultdict(set)
    for r in read_tsv(os.path.join(export, "calls.tsv")):
        if r[0] == "caller_addr":
            continue
        try:
            a = int(r[0], 16)
        except ValueError:
            continue
        try:
            b = int(r[2], 16)
        except ValueError:
            continue
        callees[a].add(b)
        callers[b].add(a)
    return callers, callees


def load_tus(root):
    """[(start, end, file)] sorted by start."""
    tus = []
    for r in read_tsv(os.path.join(root, "docs/whitebox/c2_tus.tsv")):
        if r[0] == "file" or len(r) < 3:
            continue
        try:
            tus.append((int(r[1], 16), int(r[2], 16), r[0]))
        except ValueError:
            continue
    tus.sort()
    return tus


def load_labels(root):
    """addr -> (cluster, confidence, note, source-file)."""
    out = {}
    for path in sorted(glob.glob(os.path.join(root, "docs/whitebox/labels/*.tsv"))):
        base = os.path.basename(path)
        for r in read_tsv(path):
            if len(r) < 2:
                continue
            if not BARE_ADDR_RE.fullmatch(r[0].strip()):
                continue
            a = int(r[0], 16)
            cluster = r[1] if len(r) > 1 else ""
            conf = r[2] if len(r) > 2 else ""
            note = r[3] if len(r) > 3 else ""
            # first label wins; later files append their cluster if it differs
            if a in out:
                prev = out[a]
                if cluster and cluster not in prev[0].split("|"):
                    out[a] = (prev[0] + "|" + cluster, prev[1], prev[2],
                              prev[3] + "," + base)
            else:
                out[a] = (cluster, conf, note, base)
    return out


def load_c2_functions(root):
    """addr -> (name, cluster, confidence) from the mechanical table."""
    out = {}
    path = os.path.join(root, "docs/whitebox/c2_functions.tsv")
    if not os.path.exists(path):
        return out
    for r in read_tsv(path):
        if r[0] == "addr" or len(r) < 4:
            continue
        try:
            a = int(r[0], 16)
        except ValueError:
            continue
        out[a] = tuple(r[1:6])
    return out


def scan_citations(root):
    """addr -> {relpath: count}.  docs/whitebox/ref/ is excluded so the index
    does not cite itself."""
    cites = defaultdict(lambda: defaultdict(int))
    docs = os.path.join(root, "docs")
    for dirpath, dirnames, filenames in os.walk(docs):
        dirnames[:] = [d for d in dirnames if d not in ("ref", "grids", "__pycache__")]
        for fn in filenames:
            if not fn.endswith(".md"):
                continue
            path = os.path.join(dirpath, fn)
            rel = os.path.relpath(path, root)
            try:
                text = open(path, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            for m in ADDR_RE.finditer(text):
                a = int(m.group(1) or m.group(2), 16)
                if IMG_LO <= a < IMG_HI:
                    cites[a][rel] += 1
    return cites


def containing(sorted_fns, addr):
    """(entry, size, name) of the function containing addr, or None."""
    lo, hi = 0, len(sorted_fns) - 1
    best = None
    while lo <= hi:
        mid = (lo + hi) // 2
        if sorted_fns[mid][0] <= addr:
            best = sorted_fns[mid]
            lo = mid + 1
        else:
            hi = mid - 1
    if best and best[0] <= addr < best[0] + best[1]:
        return best
    return None


def tu_for(tus, addr):
    """(file, 'in-anchor') if inside an anchor range, else (prev-file, 'gap')."""
    for lo, hi, name in RANGE_TU:
        if lo <= addr < hi:
            return name, "no-ice-site"
    if not (TEXT_CODE_LO <= addr < DATA_LO):
        # data, or the pooled read-only block below the first function: the
        # TU partition is about CODE and says nothing here
        return "n/a", "n/a"
    prev = None
    for start, end, name in tus:
        if start <= addr <= end:
            return name, "in-anchor"
        if addr > end:
            prev = name
        else:
            break
    if prev:
        return prev, "gap"
    return "-", "gap"


ROW_ADDR_RE = re.compile(
    r"^\|\s*`?(?:0[xX])?(10[bc][0-9a-f]{5})`?\s*\|")


def scan_pages(root):
    """addr -> page.  A page OWNS an address when the address is the FIRST cell
    of one of that page's table rows -- i.e. it has an entry, not a mention.
    Authoring a page is therefore what puts an address on it; the index never
    guesses."""
    owned = {}
    for path in sorted(glob.glob(os.path.join(root, "docs/whitebox/ref/P_*.md"))):
        page = os.path.basename(path)
        for line in open(path, encoding="utf-8", errors="replace"):
            m = ROW_ADDR_RE.match(line)
            if m:
                owned.setdefault(int(m.group(1), 16), page)
    return owned


def page_for(addr, tu_name, tu_conf, owned):
    page = owned.get(addr)
    if page:
        return PAGE_SUBSYS.get(page, "-"), page
    for lo, hi, sub, pg in RANGE_PAGE:
        if lo <= addr < hi:
            return sub, pg
    if tu_name in TU_PAGE:
        sub, pg = TU_PAGE[tu_name]
        # a gap attribution is a hypothesis (C2_MAP.md 3.1); it still points at
        # the right page, and tu_conf says how far to trust it
        return sub, pg
    return "-", "-"


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", "..", ".."))
    export = (sys.argv[2] if len(sys.argv) > 2
              else os.environ.get("C2RS_GHIDRA_EXPORT")
              or os.path.expanduser("~/ghidra-projects/export/c2"))

    missing = [f for f in ("functions.tsv", "data.tsv", "calls.tsv")
               if not os.path.exists(os.path.join(export, f))]
    if missing:
        sys.stderr.write(
            "build_ref.py: flat export incomplete at %s (missing %s).\n"
            "Set C2RS_GHIDRA_EXPORT or regenerate per C2_MAP_METHOD.md 3-4.\n"
            % (export, ", ".join(missing)))
        return 2

    fns = load_functions(export)
    dat = load_data(export)
    callers, callees = load_calls(export)
    tus = load_tus(root)
    labels = load_labels(root)
    c2fn = load_c2_functions(root)
    cites = scan_citations(root)
    owned = scan_pages(root)

    for entry, end, name, _why in GHIDRA_MISSED:
        fns.setdefault(entry, (end - entry, name))

    sorted_fns = sorted((a, s, n) for a, (s, n) in fns.items())

    addrs = set(cites) | set(labels)
    out_path = os.path.join(root, "docs/whitebox/ref/ADDR.tsv")
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    n_func_entry = n_in_func = n_data = n_unmapped = 0
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write("# DISASSEMBLY-DERIVED (tier 2: addresses).  GENERATED by\n"
                 "# docs/whitebox/scripts/build_ref.py -- do not hand-edit.\n"
                 "# Every addr is an absolute VA in c2.dll sha256 %s.\n"
                 "# NAVIGATION ONLY.  Nothing here may enter crates/ without a\n"
                 "# docs/whitebox/DISCLOSURE.md row naming the address.\n"
                 "# kind: func-entry | in-func | data | unmapped\n"
                 "# tu_conf: in-anchor (a fact) | gap (a hypothesis, C2_MAP.md 3.1)\n"
                 "#          no-ice-site (a TU the partition cannot see) | n/a (not code)\n"
                 % C2_SHA256)
        fh.write("\t".join(["addr", "kind", "func", "func_size", "ncallers",
                            "ncallees", "tu", "tu_conf", "subsys", "page",
                            "conf", "label", "n_cites", "cites"]) + "\n")
        for a in sorted(addrs):
            lab = labels.get(a)
            c = containing(sorted_fns, a)
            if a in fns:
                kind = "func-entry"
                n_func_entry += 1
                func, fsize = "%08x" % a, str(fns[a][0])
            elif c:
                kind = "in-func"
                n_in_func += 1
                func, fsize = "%08x" % c[0], str(c[1])
            elif a in dat or a >= DATA_LO or a < TEXT_CODE_LO:
                kind = "data"
                n_data += 1
                func = "%08x" % a if a in dat else "-"
                fsize = str(dat[a][0]) if a in dat else "-"
            else:
                kind = "unmapped"
                n_unmapped += 1
                func, fsize = "-", "-"

            fkey = int(func, 16) if func != "-" else None
            ncr = str(len(callers.get(fkey, ()))) if fkey is not None else "-"
            nce = str(len(callees.get(fkey, ()))) if fkey is not None else "-"

            tu_name, tu_conf = tu_for(tus, fkey if fkey is not None else a)
            sub, page = page_for(a, tu_name, tu_conf, owned)

            if lab:
                conf = lab[1] or "unknown"
                text = (lab[0] + ": " + lab[2]) if lab[2] else lab[0]
            else:
                meta = c2fn.get(fkey) if fkey is not None else None
                if meta and len(meta) >= 4 and meta[2] not in ("unknown", ""):
                    conf = meta[3] if len(meta) > 3 else "unknown"
                    text = meta[2]
                else:
                    conf = "unknown"
                    text = ""
            text = text.replace("\t", " ").strip()
            if len(text) > 240:
                text = text[:237] + "..."

            cc = cites.get(a, {})
            n_cites = sum(cc.values())
            cite_s = ",".join("%s:%d" % (k.replace("docs/", ""), v)
                              for k, v in sorted(cc.items(),
                                                 key=lambda kv: (-kv[1], kv[0]))[:6])
            fh.write("\t".join(["%08x" % a, kind, func, fsize, ncr, nce,
                                tu_name, tu_conf, sub, page, conf,
                                text or "-", str(n_cites), cite_s or "-"]) + "\n")

    total = len(addrs)
    resolved = n_func_entry + n_in_func
    sys.stderr.write(
        "build_ref.py: %d rows -> %s\n"
        "  cited in docs/: %d   hand-labelled: %d\n"
        "  func-entry %d  in-func %d  data %d  unmapped %d\n"
        "  resolved to a containing function: %d/%d = %.1f%%\n"
        % (total, os.path.relpath(out_path, root), len(cites), len(labels),
           n_func_entry, n_in_func, n_data, n_unmapped,
           resolved, total, 100.0 * resolved / max(total, 1)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
