#!/usr/bin/env python3
"""w-grammarscreen — ANNOTATE the frozen site set with its enclosing function.

This adds a DERIVED FIELD to sites the frozen `sites.jsonl` already contains.
It does not add, remove or move a site — `sites.jsonl` is written once, by
`enumerate.py`, before the first probe, and is never rewritten. The annotated
copy is a separate file so that the two can be diffed.

Why the enclosing function matters, and it is the sharpest thing this lane can
say about its UNKNOWN bucket: a quiet site splits in two.

  * **quiet, and NO sibling site in the same function ever fired** — the corpus
    plausibly never ENTERS this function at all. A statement about dispatch.
  * **quiet, and a sibling site in the same function DID fire** — the corpus
    demonstrably reaches this function and never takes this branch. A statement
    about the branch.

Neither is a proof of deadness. Both are strictly more informative than
"quiet", and the second is the population where a witness is plausibly cheap:
control already arrives in the function.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate import tokenize  # noqa: E402

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LANE = os.path.join(ROOT, "work", "w-grammarscreen")


def fn_spans(toks):
    """[(name, first_line, last_line)] for every `fn NAME ... { }` in a file."""
    out = []
    i = 0
    while i < len(toks) - 1:
        if toks[i][1] == "fn" and toks[i + 1][0] == "ident":
            name = toks[i + 1][1]
            j = i + 2
            depth = 0
            start = None
            while j < len(toks):
                t = toks[j][1]
                if t == ";" and depth == 0:
                    break  # a trait method declaration, no body
                if t == "{":
                    if depth == 0:
                        start = j
                    depth += 1
                elif t == "}":
                    depth -= 1
                    if depth == 0:
                        out.append((name, toks[start][2], toks[j][2]))
                        i = j
                        break
                j += 1
        i += 1
    return out


def main():
    sites = [json.loads(l) for l in open(os.path.join(LANE, "sites.jsonl"))]
    cache = {}
    tokcache = {}
    for s in sites:
        f = s["file"]
        if f not in cache:
            cache[f] = fn_spans(list(tokenize(open(os.path.join(ROOT, f), encoding="utf-8").read())))
        best = None
        for name, a, b in cache[f]:
            if a <= s["line"] <= b:
                # innermost wins (closures / nested fns)
                if best is None or (b - a) < (best[2] - best[1]):
                    best = (name, a, b)
        s["fn"] = best[0] if best else None
        s["fn_span"] = [best[1], best[2]] if best else None
        # `Location::column()` is the column of the CALL EXPRESSION's span, and
        # for a qualified call `Block::refuse(..)` that span starts at `Block`,
        # not at `refuse`. Record the path-start column as an alternate key so a
        # probe hit matches on either. Which one rustc actually reports is
        # VERIFIED from the probe log (`rederive.py` prints every out-of-frame
        # hit in full) and never assumed.
        s["col_alt"] = s["col"]
        if s["kind"] == "block_refuse":
            toks = tokcache.setdefault(
                f, list(tokenize(open(os.path.join(ROOT, f), encoding="utf-8").read()))
            )
            for i, (k, v, ln, cl) in enumerate(toks):
                if ln == s["line"] and cl == s["col"] and v == "refuse":
                    if i >= 3 and toks[i - 1][1] == ":" and toks[i - 2][1] == ":":
                        s["col_alt"] = toks[i - 3][3]
                    break
    out = os.path.join(LANE, "sites_annotated.jsonl")
    with open(out, "w") as fh:
        for s in sites:
            fh.write(json.dumps(s, sort_keys=True) + "\n")
    n = sum(1 for s in sites if s["fn"] is None)
    print("annotated %d sites; %d with no enclosing fn" % (len(sites), n))
    fns = {}
    for s in sites:
        fns.setdefault((s["file"], s["fn"]), 0)
        fns[(s["file"], s["fn"])] += 1
    print("distinct enclosing functions: %d" % len(fns))
    print("wrote %s" % out)


if __name__ == "__main__":
    main()
