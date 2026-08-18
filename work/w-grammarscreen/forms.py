#!/usr/bin/env python3
"""w-grammarscreen — classify each site as DIVERGING or EAGER.

The screen records that a site's constructor was **evaluated**. For a site in
diverging position — `return Err(blk(..))`, `return Err(Some(Block::refuse(..)))`,
`Err(blk(..))?` — evaluation IS refusal. For `.ok_or(blk(..))` it is not:
`ok_or` evaluates its argument on every pass, refusing or not.

So the reach figure has two readings and this file separates them:

    EVALUATED  the site's expression ran            (all forms)
    REFUSED    the site's expression ran AND the
               parse returned through it            (diverging forms only)

`quiet` is sound for every form — evaluated ⊇ refused — so only the reached
bucket needs the split.

The rule: walk back from the callee token to the nearest statement boundary
(`;`  `{`  `}`  `=>`  `,` at depth 0 relative to the site). The site is
DIVERGING if the first token of that statement is `return`, or if the enclosing
expression is closed by `?`.
"""
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate import tokenize  # noqa: E402

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
LANE = os.path.join(ROOT, "work", "w-grammarscreen")


def classify_src(lines, line, col, toks, i):
    """Source-text rule, deliberately simple and checked by eye against a
    printed sample rather than trusted: take the text of the statement up to the
    site (the site's own line prefix, plus up to four preceding lines when the
    statement is wrapped), cut it at the last statement boundary, and ask
    whether what remains starts with `return`."""
    # EXACT, token-level, and it comes FIRST: `.ok_or(X)` / `.unwrap_or(X)`
    # evaluate `X` unconditionally, so a hit there is EVALUATED and not
    # REFUSED — even when the whole expression is followed by `?`, which is why
    # this test must precede the `?` test and not follow it.
    if i >= 2 and toks[i - 1][1] == "(" and toks[i - 2][1] in ("ok_or", "unwrap_or"):
        return "eager-ok_or"
    buf = []
    for k in range(max(0, line - 5), line - 1):
        buf.append(lines[k])
    buf.append(lines[line - 1][: col - 1])
    # Strip line comments before cutting: a `//` comment containing `;` or `=>`
    # otherwise moves the cut and turns a `return Err(..)` into "unclassified".
    # This was found by printing the bucket, not by reasoning about it.
    def decomment(x):
        return x.split("//")[0]
    txt = " ".join(decomment(x).strip() for x in buf)
    for sep in ("{", "}", ";", "=>"):
        if sep in txt:
            txt = txt[txt.rindex(sep) + len(sep):]
    txt = txt.strip()
    if txt.startswith("return"):
        return "diverging-return"
    # forward: `?` immediately after the call's closing parens
    depth = 0
    j = i + 1
    while j < len(toks):
        t = toks[j][1]
        if t in "([{":
            depth += 1
        elif t in ")]}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    k = j + 1
    while k < len(toks) and toks[k][1] in ")]}":
        k += 1
    if k < len(toks) and toks[k][1] == "?":
        return "diverging-question"
    if "ok_or(" in txt or "ok_or_else(" in txt or "unwrap_or(" in txt:
        return "eager-ok_or"
    return "eager-other"


def main():
    sites = [json.loads(l) for l in open(os.path.join(LANE, "sites_annotated.jsonl"))]
    cache = {}
    for s in sites:
        f = s["file"]
        if f not in cache:
            # Read from git, NOT from the worktree: the probe patch is applied
            # while this runs and it shifts line numbers in `func/body/mod.rs`.
            # The frozen enumeration addresses the CLEAN tree, so the clean tree
            # is what this must tokenize.
            blob = subprocess.run(["git", "-C", ROOT, "show", "HEAD:" + f],
                                  capture_output=True, text=True, check=True).stdout
            cache[f] = (list(tokenize(blob)), blob.splitlines())
        toks, lines = cache[f]
        idx = None
        for i, (k, v, ln, cl) in enumerate(toks):
            if ln == s["line"] and cl == s["col"]:
                idx = i
                break
        s["div"] = (classify_src(lines, s["line"], s["col"], toks, idx)
                    if idx is not None else "UNRESOLVED")
    out = os.path.join(LANE, "sites_forms.jsonl")
    with open(out, "w") as fh:
        for s in sites:
            fh.write(json.dumps(s, sort_keys=True) + "\n")
    import collections
    print(collections.Counter(s["div"] for s in sites))
    print("wrote %s" % out)


if __name__ == "__main__":
    main()
