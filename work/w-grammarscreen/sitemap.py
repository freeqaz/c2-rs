#!/usr/bin/env python3
"""w-grammarscreen — translate PROBE addresses into FROZEN site keys.

`rederive.py`'s out-of-frame report on `P1` printed **11 hits the frozen
enumeration could not place**, and reading them found two defects — both in the
address translation, neither in the population:

  1. **The probe patch MOVES LINES.** Installing `#[track_caller]` and the probe
     call adds 6 lines to `crates/c2-il/src/func/body/mod.rs`, so every site
     below an insertion reports a line number 2, 4 or 6 higher than the frozen
     enumeration's. Ten of the eleven are this.
  2. **`Location::column()` reports the PATH START of a qualified call.** For
     `crate::func::body::blk_type(..)` that is the column of `crate`, not of
     `blk_type`. `annotate.py` handled this for `Block::refuse` only; the
     eleventh hit, `mcall_cmp.rs:215:33`, is a qualified `blk_type`.

Both are fixed HERE, in the translation, and **neither changes the frozen site
set**: this file asserts, per file, that the patched tree carries exactly the
same number of sites in exactly the same `ctx` order as `sites.jsonl`, and
refuses to emit a map otherwise. The k-th site of a file maps to the k-th, which
is sound precisely because the probe patch adds no site of this class (it adds
`crate::grammarprobe::hit(..)` calls, which are not `blk` / `blk_type` /
`::refuse`).

    sitemap.py            write sitemap.json for the CURRENT (patched) tree
    sitemap.py --selftest assert the scan reproduces the frozen sites.jsonl
                          exactly on a tree with no probe applied
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate import tokenize, rust_files, test_spans  # noqa: E402

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
SRC = os.path.join(ROOT, "crates", "c2-il", "src")
LANE = os.path.join(ROOT, "work", "w-grammarscreen")


def scan(text):
    """[(line, col_ident, col_path, kind, ctx)] in source order."""
    toks = list(tokenize(text))
    out = []
    i = 0
    while i < len(toks):
        k, v, ln, cl = toks[i]
        kind = argpos = None
        if k == "ident" and v in ("blk", "blk_type") and i + 1 < len(toks) and toks[i + 1][1] == "(":
            kind, argpos = v, (3 if v == "blk" else 4)
        elif (k == "ident" and v == "refuse" and i + 1 < len(toks) and toks[i + 1][1] == "("
              and i >= 2 and toks[i - 1][1] == ":" and toks[i - 2][1] == ":"):
            kind, argpos = "block_refuse", 3
        if kind is None or (i >= 1 and toks[i - 1][1] == "fn"):
            i += 1
            continue
        # walk BACK over `ident :: ident :: …` to the start of the qualified path
        j = i
        while j >= 2 and toks[j - 1][1] == ":" and toks[j - 2][1] == ":" and \
                j >= 3 and toks[j - 3][0] == "ident":
            j -= 3
        col_path = toks[j][3]
        # argument split, for the ctx
        j2 = i + 1
        depth = 0
        args, cur = [], []
        while j2 < len(toks):
            tv = toks[j2][1]
            if tv in "([{":
                depth += 1
                if depth == 1:
                    j2 += 1
                    continue
            elif tv in ")]}":
                depth -= 1
                if depth == 0:
                    args.append(cur)
                    break
            if depth == 1 and tv == ",":
                args.append(cur)
                cur = []
            else:
                cur.append(toks[j2])
            j2 += 1
        ctx = None
        if len(args) >= argpos:
            a = args[argpos - 1]
            if len(a) == 1 and a[0][0] == "str":
                ctx = a[0][1]
            elif a:
                ctx = "".join(t[1] for t in a)
        out.append((ln, cl, col_path, kind, ctx))
        i = j2 + 1 if j2 > i else i + 1
    return out


def frozen_by_file():
    rows = [json.loads(l) for l in open(os.path.join(LANE, "sites.jsonl"))]
    by = {}
    for r in rows:
        by.setdefault(r["file"], []).append(r)
    for v in by.values():
        v.sort(key=lambda r: (r["line"], r["col"]))
    return by


def main():
    selftest = "--selftest" in sys.argv
    frozen = frozen_by_file()
    mapping = {}
    files = 0
    for path in rust_files(SRC):
        rel = os.path.relpath(path, ROOT)
        cur = scan(open(path, encoding="utf-8").read())
        fr = frozen.get(rel, [])
        if not cur and not fr:
            continue
        files += 1
        if len(cur) != len(fr):
            sys.exit("REFUSED: %s has %d sites in the tree and %d frozen — the "
                     "populations differ, so the k-th-to-k-th map is unsound"
                     % (rel, len(cur), len(fr)))
        for (ln, cl, cp, kind, ctx), f in zip(cur, fr):
            if kind != f["kind"] or ctx != f["ctx"]:
                sys.exit("REFUSED: %s site order diverged at line %d: tree "
                         "(%s, %r) vs frozen (%s, %r)" % (rel, ln, kind, ctx, f["kind"], f["ctx"]))
            if selftest and (ln != f["line"] or cl != f["col"]):
                sys.exit("SELFTEST FAILED: %s line/col %d:%d vs frozen %d:%d"
                         % (rel, ln, cl, f["line"], f["col"]))
            tgt = "%s:%d:%d" % (f["file"], f["line"], f["col"])
            mapping["%s:%d:%d" % (rel, ln, cl)] = tgt
            mapping["%s:%d:%d" % (rel, ln, cp)] = tgt
    if selftest:
        print("SELFTEST OK: the scan reproduces sites.jsonl exactly over %d files" % files)
        return
    out = os.path.join(LANE, "sitemap.json")
    json.dump(mapping, open(out, "w"), indent=0, sort_keys=True)
    print("wrote %s — %d probe addresses over %d files" % (out, len(mapping), files))


if __name__ == "__main__":
    main()
