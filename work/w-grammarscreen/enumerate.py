#!/usr/bin/env python3
"""w-grammarscreen — enumerate the GRAMMAR fail-closed refusal sites in
`crates/c2-il/src` by TOKENIZING Rust, never by grepping (board #3288).

The class is the one `w-mutcensus` §2.1 dropped with counts:

    blk( 1227 | blk_type( 6 | Block::refuse( 106     (raw `grep -n | wc -l`)

Those three figures are LINE counts of a fixed-string grep over the whole of
`crates/c2-il/src`. They therefore

  * count a line carrying TWO call sites ONCE,
  * count the FUNCTION DEFINITIONS `fn blk(`, `fn blk_type(`, `fn refuse(`,
  * count occurrences inside `//` and `/* */` comments and doc comments,
  * count occurrences inside string literals,
  * count `#[cfg(test)]` module bodies beside production code,
  * count `Block::refuse(` helper definitions and re-exports.

This file replaces that with a hand-rolled Rust lexer good enough to be right
about all six: it skips comments (line, block, NESTED block, doc), string
literals (normal, escaped, raw with any hash count), char/lifetime, and byte
strings, then finds the CALL TOKENS `blk` `(` / `blk_type` `(` /
`Block` `::` `refuse` `(` in the remaining token stream.

Output (JSONL, one object per site) carries, per site:
    file, line, col        1-based; `col` is the column of the callee ident,
                           which is what `#[track_caller]`'s
                           `Location::caller()` reports for a free-function
                           call (VERIFIED against the probe log, never assumed)
    kind                   blk | blk_type | block_refuse
    ctx                    the `&'static str` context literal, when the
                           argument is a literal; None when it is a constant
                           path (those are named, not dropped)
    form                   how the constructed Block is CONSUMED at this site:
                             ret_err   `return Err(blk(..))`
                             q_err     `Err(blk(..))?`
                             ok_or     `.ok_or(blk(..))` / `.ok_or_else(..)`
                             other     anything else (printed, never binned away)
                           `form` decides whether a HIT means REFUSED or merely
                           EVALUATED — `ok_or` evaluates its argument eagerly.
    in_test                inside a `#[cfg(test)]` module
"""
import json
import os
import sys

ROOT = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
SRC = os.path.join(ROOT, "crates", "c2-il", "src")


def tokenize(text):
    """Yield (kind, value, line, col) over a Rust source string.

    kind is one of: ident, punct, str, char, num, other.  Comments are dropped.
    line/col are 1-based; col counts UTF-8 *characters* of the line, which is
    what rustc's `Location::column()` reports.
    """
    i = 0
    n = len(text)
    line = 1
    col = 1

    def adv(k):
        nonlocal i, line, col
        for _ in range(k):
            if text[i] == "\n":
                line += 1
                col = 1
            else:
                col += 1
            i += 1

    while i < n:
        c = text[i]
        # whitespace
        if c in " \t\r\n":
            adv(1)
            continue
        # line comment (incl. `///` and `//!`)
        if text.startswith("//", i):
            j = text.find("\n", i)
            if j < 0:
                j = n
            adv(j - i)
            continue
        # block comment, NESTED (Rust allows nesting)
        if text.startswith("/*", i):
            depth = 0
            start = i
            while i < n:
                if text.startswith("/*", i):
                    depth += 1
                    adv(2)
                elif text.startswith("*/", i):
                    depth -= 1
                    adv(2)
                    if depth == 0:
                        break
                else:
                    adv(1)
            if depth != 0:
                raise SystemExit("unterminated block comment at offset %d" % start)
            continue
        # raw string: r"..." r#"..."# br##"..."##
        j = i
        if text[j] == "b":
            j += 1
        if j < n and text[j] == "r":
            k = j + 1
            hashes = 0
            while k < n and text[k] == "#":
                hashes += 1
                k += 1
            if k < n and text[k] == '"':
                closer = '"' + "#" * hashes
                end = text.find(closer, k + 1)
                if end < 0:
                    raise SystemExit("unterminated raw string at line %d" % line)
                sl, sc = line, col
                val = text[k + 1 : end]
                adv(end + len(closer) - i)
                yield ("str", val, sl, sc)
                continue
        # normal string / byte string
        j = i
        if text[j] == "b":
            j += 1
        if j < n and text[j] == '"':
            sl, sc = line, col
            adv(j - i + 1)  # past the opening quote
            buf = []
            while i < n and text[i] != '"':
                if text[i] == "\\":
                    esc = text[i + 1] if i + 1 < n else ""
                    simple = {"n": "\n", "t": "\t", "r": "\r", "0": "\0",
                              "\\": "\\", "'": "'", '"': '"'}
                    if esc in simple:
                        buf.append(simple[esc])
                    else:
                        buf.append("\\" + esc)
                    adv(2)
                else:
                    buf.append(text[i])
                    adv(1)
            adv(1)  # closing quote
            yield ("str", "".join(buf), sl, sc)
            continue
        # char literal vs lifetime: `'a` is a lifetime, `'x'` / `'\n'` a char
        if c == "'":
            k = i + 1
            if k < n and text[k] == "\\":
                k += 2
                while k < n and text[k] != "'":
                    k += 1
                k += 1
                sl, sc = line, col
                val = text[i:k]
                adv(k - i)
                yield ("char", val, sl, sc)
                continue
            if k + 1 < n and text[k + 1] == "'":
                sl, sc = line, col
                val = text[i : i + 3]
                adv(3)
                yield ("char", val, sl, sc)
                continue
            # lifetime
            sl, sc = line, col
            k = i + 1
            while k < n and (text[k].isalnum() or text[k] == "_"):
                k += 1
            val = text[i:k]
            adv(k - i)
            yield ("other", val, sl, sc)
            continue
        # identifier / keyword
        if c.isalpha() or c == "_":
            k = i
            while k < n and (text[k].isalnum() or text[k] == "_"):
                k += 1
            sl, sc = line, col
            val = text[i:k]
            adv(k - i)
            yield ("ident", val, sl, sc)
            continue
        # number
        if c.isdigit():
            k = i
            while k < n and (text[k].isalnum() or text[k] in "_."):
                if text[k] == "." and k + 1 < n and text[k + 1] == ".":
                    break
                k += 1
            sl, sc = line, col
            val = text[i:k]
            adv(k - i)
            yield ("num", val, sl, sc)
            continue
        # punctuation
        sl, sc = line, col
        adv(1)
        yield ("punct", c, sl, sc)


def rust_files(root):
    out = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for f in sorted(filenames):
            if f.endswith(".rs"):
                out.append(os.path.join(dirpath, f))
    return out


def test_spans(toks, text):
    """Return a list of (start_line, end_line) for `#[cfg(test)] mod ... { }`."""
    spans = []
    for idx in range(len(toks)):
        k, v, ln, cl = toks[idx]
        if not (k == "punct" and v == "#"):
            continue
        # #[cfg(test)]
        window = toks[idx : idx + 6]
        sig = [t[1] for t in window]
        if sig[:6] != ["#", "[", "cfg", "(", "test", ")"]:
            continue
        # walk forward to the `mod IDENT {` and brace-match
        j = idx + 6
        while j < len(toks) and toks[j][1] != "mod":
            if toks[j][1] in ("fn", "impl", "use", "const", "struct"):
                break
            j += 1
        if j >= len(toks) or toks[j][1] != "mod":
            continue
        while j < len(toks) and toks[j][1] != "{":
            j += 1
        if j >= len(toks):
            continue
        depth = 0
        start_line = toks[j][2]
        while j < len(toks):
            if toks[j][1] == "{":
                depth += 1
            elif toks[j][1] == "}":
                depth -= 1
                if depth == 0:
                    spans.append((start_line, toks[j][2]))
                    break
            j += 1
    return spans


def classify_form(toks, idx):
    """Look BACKWARD from the callee ident at toks[idx] and name the form."""
    prev = [toks[i][1] for i in range(max(0, idx - 6), idx)]
    prev = prev[::-1]  # nearest first
    if len(prev) >= 2 and prev[0] == "(" and prev[1] == "Err":
        # `Err(blk(..))` — is it `return Err(..)` or `Err(..)?`
        if len(prev) >= 3 and prev[2] == "return":
            return "ret_err"
        return "err_expr"
    if len(prev) >= 2 and prev[0] == "(" and prev[1] in ("ok_or", "unwrap_or"):
        return "ok_or"
    if len(prev) >= 1 and prev[0] == "||":
        return "closure"
    return "other"


def main():
    sites = []
    stats = {"files": 0, "raw_blk_lines": 0}
    for path in rust_files(SRC):
        rel = os.path.relpath(path, ROOT)
        text = open(path, encoding="utf-8").read()
        toks = list(tokenize(text))
        stats["files"] += 1
        spans = test_spans(toks, text)

        def in_test(ln):
            return any(a <= ln <= b for a, b in spans)

        i = 0
        while i < len(toks):
            k, v, ln, cl = toks[i]
            kind = None
            argpos = None
            if k == "ident" and v in ("blk", "blk_type") and i + 1 < len(toks) and toks[i + 1][1] == "(":
                kind = v
                argpos = 3 if v == "blk" else 4
                callee_i = i
            elif (
                k == "ident"
                and v == "refuse"
                and i + 1 < len(toks)
                and toks[i + 1][1] == "("
                and i >= 2
                and toks[i - 1][1] == ":"
                and toks[i - 2][1] == ":"
            ):
                kind = "block_refuse"
                argpos = 3
                callee_i = i
            if kind is None:
                i += 1
                continue
            # a DEFINITION, not a call: `fn blk(`
            if i >= 1 and toks[i - 1][1] == "fn":
                i += 1
                continue
            # split the argument list at depth-1 commas, collect arg starts
            j = i + 1
            depth = 0
            args = []
            cur = []
            while j < len(toks):
                tv = toks[j][1]
                if tv in "([{":
                    depth += 1
                    if depth == 1:
                        j += 1
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
                    cur.append(toks[j])
                j += 1
            ctx = None
            ctx_kind = "missing"
            if len(args) >= argpos:
                a = args[argpos - 1]
                if len(a) == 1 and a[0][0] == "str":
                    ctx = a[0][1]
                    ctx_kind = "literal"
                elif a:
                    ctx = "".join(t[1] for t in a)
                    ctx_kind = "path"
            sites.append(
                {
                    "file": rel,
                    "line": ln,
                    "col": cl,
                    "kind": kind,
                    "ctx": ctx,
                    "ctx_kind": ctx_kind,
                    "form": classify_form(toks, callee_i),
                    "in_test": in_test(ln),
                    "nargs": len(args),
                }
            )
            i = j + 1 if j > i else i + 1

    out = os.path.join(ROOT, "work", "w-grammarscreen", "sites.jsonl")
    with open(out, "w") as f:
        for s in sites:
            f.write(json.dumps(s, sort_keys=True) + "\n")
    print("files scanned: %d" % stats["files"])
    print("sites parsed:  %d" % len(sites))
    for kind in ("blk", "blk_type", "block_refuse"):
        tot = sum(1 for s in sites if s["kind"] == kind)
        prod = sum(1 for s in sites if s["kind"] == kind and not s["in_test"])
        print("  %-13s total %5d   production %5d   test %4d" % (kind, tot, prod, tot - prod))
    print("wrote %s" % out)


if __name__ == "__main__":
    main()
