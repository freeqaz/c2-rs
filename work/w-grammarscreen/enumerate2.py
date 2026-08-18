#!/usr/bin/env python3
"""w-grammarscreen — re-derive, BY PARSING, the OTHER classes `w-mutcensus`
§2.1 dropped with counts. These are OUT of this lane's probe frame (§1.2 of the
prereg); they are re-counted because #3288's rule is that any enumeration
quoted as a denominator owes a second, differently-built count, and all three
of these figures are quoted as denominators in `w-mutcensus`' drop table.

`w-mutcensus` at `3835469c` recorded:
    IlBundle::dyninit_tu  `return None` clauses  12  (1 mutated, 11 dropped)
    IlBundle::data_tu     `return None` clauses  14  (1 mutated, 13 dropped)
    shape-file OptWordMode comparison sites      18  (all dropped)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate import tokenize, rust_files, test_spans  # noqa: E402

ROOT = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
SRC = os.path.join(ROOT, "crates", "c2-il", "src")


def fn_body(toks, name):
    """Return the token slice of `fn <name>(...) ... { ... }`, brace-matched."""
    for i in range(len(toks) - 1):
        if toks[i][1] == "fn" and toks[i + 1][1] == name:
            j = i
            depth = 0
            while j < len(toks):
                if toks[j][1] == "{":
                    if depth == 0:
                        start = j
                    depth += 1
                elif toks[j][1] == "}":
                    depth -= 1
                    if depth == 0:
                        return toks[start : j + 1]
                j += 1
    return None


def count_return_none(body):
    out = []
    for i in range(len(body) - 1):
        if body[i][1] == "return" and body[i + 1][1] == "None":
            out.append((body[i][2], body[i][3]))
    return out


def main():
    path = os.path.join(SRC, "func", "bundle.rs")
    toks = list(tokenize(open(path, encoding="utf-8").read()))
    print("== `return None` clauses, brace-matched function bodies in func/bundle.rs ==")
    total = 0
    for fn in ("dyninit_tu", "data_tu", "provide_data_tu"):
        b = fn_body(toks, fn)
        if b is None:
            print("  %-16s NOT FOUND at this head" % fn)
            continue
        hits = count_return_none(b)
        total += len(hits)
        print("  %-16s body lines %d..%d   `return None` clauses: %d"
              % (fn, b[0][2], b[-1][2], len(hits)))
        print("       lines: %s" % ", ".join(str(l) for l, _ in hits))
    print("  TOTAL over the three: %d" % total)

    print()
    print("== OptWordMode comparison sites in the shape files (non-test) ==")
    shapes = os.path.join(SRC, "func", "body", "shapes")
    grand = 0
    for f in rust_files(shapes):
        text = open(f, encoding="utf-8").read()
        tk = list(tokenize(text))
        spans = test_spans(tk, text)
        n = 0
        lines = []
        for i, (k, v, ln, cl) in enumerate(tk):
            if v != "OptWordMode":
                continue
            if any(a <= ln <= b for a, b in spans):
                continue
            # a COMPARISON site: the identifier participates in `==` / `!=`
            win = [t[1] for t in tk[max(0, i - 6):i + 8]]
            if "==" in win or "!=" in win or "=" in win:
                n += 1
                lines.append(ln)
        if n:
            grand += n
            print("  %-40s %2d   lines %s" % (os.path.relpath(f, shapes), n, lines))
    print("  TOTAL: %d" % grand)


if __name__ == "__main__":
    main()
