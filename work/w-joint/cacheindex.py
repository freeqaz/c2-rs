#!/usr/bin/env python3
"""cacheindex.py — map workload TU -> `work/capture-cache` entry, from meta.txt.

The harness's capture cache stores, per key, the WHOLE `_CL_*` quintet
(`gl ex in db sy`) **and `out.obj`**.  Every lane so far re-ran `cl` to get IL
(w-emit, w-mark) or truth (w-emit), while 871 of the 878 workload TUs were
already sitting in the cache with their obj next to their IL.  This lane reads
the cache instead of re-running the toolchain, and pays for that shortcut with
an explicit agreement control against w-emit's independently captured truth
(`truth_data.py` KA-AGREE): if the two capture paths disagreed on even one TU
the control goes red and this file is unusable.

**Cache hygiene, which this repo has been burned by twice.**  The cache root is
~100k entries and 67 GB.  This script does exactly one `os.scandir` at depth 1
and then opens **only** `<entry>/meta.txt` by explicit path.  It never globs
`*/`, never recurses, and never stats the payload files.  Do not "improve" it
into a glob.

    usage: cacheindex.py <cache-root> <tulist.txt> <out.tsv> [dc3-rev]

`out.tsv` is `<src>\t<entry>` for every TU with a complete entry; every other
outcome is printed with its count AND its name, so a missing TU can never read
as a success.

**Two filters, and both are load-bearing.**  One source has up to 30 entries,
because the cache key carries the *worktree's* identity as well as the
toolchain's, so every lane that ever scanned the workload minted its own copy.
Those copies are **not** interchangeable:

* `key.bin`'s `tree <rev>+clean` line is the **dc3 workload rev**, and the
  cache holds entries from at least 18 different ones (`fbf097a5`, `9ad5c4c8`,
  `940d07dc`, …).  An entry from a different rev is a different corpus.  This
  script therefore requires `tree <REV>+clean` **exactly**, `REV` defaulting to
  the current `C2RS_DC3` HEAD, and `+DIRTY` is never accepted.
* the argv signature must be the workload's, so a fixture or a `gate.sh` lane
  entry can never be picked up for a workload TU.

The remaining duplicates are byte-identical apart from the `-Fo` path the obj
embeds in `S_OBJNAME`, which is why the entry is chosen by sorted order and why
`truth_data.py` extracts by *name* and never by file offset.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

STREAMS = ("gl", "ex", "in", "db", "sy")

# the workload's argv, minus every path-bearing argument
WORKLOAD_SIG = ("-typedil", "-W", "1", "-Gs4096", "-G604", "-QVMX128", "-QDD2",
                "-MT", "-Bd", "-Og", "-Ob2", "-Gy", "-EHs")


def meta_args(entry):
    p = os.path.join(entry, "meta.txt")
    try:
        with open(p, "r", errors="replace") as fh:
            lines = fh.read().splitlines()
    except OSError:
        return None
    return [l[4:] for l in lines if l.startswith("arg ")]


def sig_of(args):
    """argv with every path-bearing argument and its flag removed."""
    out = []
    skip = False
    for a in args:
        if skip:
            skip = False
            continue
        if a in ("-f", "-il"):
            skip = True
            continue
        if (a.startswith("/") or a.lower().startswith("z:")
                or a.startswith("-Fo") or a.startswith("-Fd")):
            continue
        out.append(a)
    return tuple(out)


def tree_of(entry):
    try:
        kb = open(os.path.join(entry, "key.bin"), "rb").read()
    except OSError:
        return None
    for ln in kb.decode("latin1").split("\n"):
        if ln.startswith("tree "):
            return ln[5:]
    return None


def meta_source(entry):
    """The `-f` argument of the capture that made this entry, host-normalized."""
    p = os.path.join(entry, "meta.txt")
    try:
        with open(p, "r", errors="replace") as fh:
            lines = fh.read().splitlines()
    except OSError:
        return None
    for i, ln in enumerate(lines):
        if ln == "arg -f" and i + 1 < len(lines) and lines[i + 1].startswith("arg "):
            s = lines[i + 1][4:].strip()
            if s[:2].lower() == "z:":
                s = s[2:]
            return s.replace("\\", "/")
    return None


def base_of(entry):
    """`_CL_xxxxxxxx` prefix shared by the five sub-streams, or None."""
    try:
        names = os.listdir(entry)
    except OSError:
        return None
    for n in names:
        if n.startswith("_CL_") and n.endswith("gl") and len(n) == 4 + 8 + 2:
            return n[:-2]
    return None


def build(cache, rev):
    """{src: [entry, ...]} over depth-1 entries at the wanted dc3 rev, clean,
    with the workload argv signature.  Rejections are counted by reason."""
    out = {}
    n = 0
    rej = {"no-meta": 0, "other-sig": 0, "other-tree": 0, "dirty-tree": 0}
    want_tree = rev + "+clean"
    with os.scandir(cache) as it:
        for de in it:
            if not de.is_dir() or de.name.startswith("."):
                continue
            n += 1
            args = meta_args(de.path)
            if args is None:
                rej["no-meta"] += 1
                continue
            if sig_of(args) != WORKLOAD_SIG:
                rej["other-sig"] += 1
                continue
            t = tree_of(de.path)
            if t != want_tree:
                rej["dirty-tree" if t and t.endswith("DIRTY") else
                    "other-tree"] += 1
                continue
            src = meta_source(de.path)
            if src is None:
                rej["no-meta"] += 1
                continue
            out.setdefault(src, []).append(de.path)
    return out, n, rej


def main():
    cache, tulist, outp = sys.argv[1], sys.argv[2], sys.argv[3]
    if len(sys.argv) > 4:
        rev = sys.argv[4]
    else:
        dc3 = os.environ.get(
            "C2RS_DC3",
            os.path.join(os.path.dirname(HERE), "..", "..", "dc3-decomp"))
        rev = subprocess.check_output(["git", "-C", dc3, "rev-parse", "HEAD"]
                                      ).decode().strip()
    srcs = [l.strip() for l in open(tulist) if l.strip()]

    idx, n_entries, rej = build(cache, rev)
    print("cache entries scanned: %d ; dc3 rev required: %s+clean" % (n_entries, rev))
    print("rejected: %s" % rej)
    print("distinct workload sources at that rev: %d" % len(idx))

    rows = []
    miss, ambig, incomplete = [], [], []
    for s in srcs:
        cands = idx.get(s, [])
        good = []
        for e in cands:
            b = base_of(e)
            if b is None or not os.path.exists(os.path.join(e, "out.obj")):
                continue
            if not all(os.path.exists(os.path.join(e, b + k)) for k in STREAMS):
                continue
            good.append(e)
        if not good:
            (incomplete if cands else miss).append(s)
            continue
        if len(good) > 1:
            ambig.append(s)
        rows.append((s, sorted(good)[0], len(good)))

    with open(outp, "w") as fh:
        for s, e, k in rows:
            fh.write("%s\t%s\t%d\n" % (s, e, k))

    print("TUs requested %d ; INDEXED %d ; no cache entry %d ; "
          "entry without obj+quintet %d ; MULTIPLE entries %d"
          % (len(srcs), len(rows), len(miss), len(incomplete), len(ambig)))
    for tag, lst in (("NO-ENTRY", miss), ("INCOMPLETE", incomplete),
                     ("AMBIGUOUS", ambig)):
        for s in lst[:40]:
            print("  %s %s" % (tag, s))
        if len(lst) > 40:
            print("  %s ... and %d more" % (tag, len(lst) - 40))


if __name__ == "__main__":
    main()
