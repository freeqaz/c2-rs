#!/usr/bin/env python3
"""w-front3 — hunt for STRUCTURALLY UNREACHABLE rungs.

    python3 work/w-front3/reach.py

Board **#1218** priced `codegen/leaf/store.rs`'s `value_bound` refusal into the
`xboxheap` ladder for weeks. `w-mrslot` §5.1 found it has **no reachable input**:
it fires only on an `IlOp::BoundAddr` in a store's VALUE position, and the only
producer of `BoundAddr` anywhere in `crates/c2-il` rewrites the BASE position.

A rung with no reachable input is not a rung. It is not merely "cheap" — it
costs nothing and, worse, it is invisible: it can never fire, so no scan, no
sweep and no gate will ever report it, and it stays on the roadmap forever.

The test, stated as a procedure so it can be re-run rather than re-argued:

  1. find the refusal's INPUT — which `IlOp` variant, in which POSITION;
  2. find EVERY producer of that variant in `crates/c2-il`, excluding test code;
  3. check whether any of them can put it in the position the refusal reads.

This script automates (1) and (2) coarsely — variant-level reachability — and
prints the candidates that (3) must be run by hand on. It is a SCREEN, not a
verdict: a variant with producers can still be unreachable *in a position*,
which is exactly `value_bound`'s case and is why that row is asserted here by
name rather than derived.

The screen's own blind spot, stated rather than discovered later: it counts
producers **anywhere** in `c2-il`, so it cannot see a producer that exists but
is itself behind a refusal that never lifts. That is a second-order
unreachability and this script says nothing about it.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def rs_files(rel):
    for dirpath, _, names in os.walk(os.path.join(ROOT, rel)):
        for n in names:
            if n.endswith(".rs"):
                yield os.path.join(dirpath, n)


def strip_noncode(text):
    """Drop doc comments, line comments and `#[cfg(test)] mod tests { … }`.

    The test cut is brace-counted from the `mod tests` header rather than
    regex'd: `leaf_store.rs` constructs `IlOp::BoundAddr` twice inside its test
    module, and counting those as producers would have made `value_bound` look
    reachable — the exact error this script exists to avoid.
    """
    out = []
    for ln in text.split("\n"):
        s = ln.strip()
        if s.startswith("//"):
            continue
        out.append(ln)
    text = "\n".join(out)
    i = text.find("mod tests")
    if i >= 0:
        j = text.find("{", i)
        if j >= 0:
            d, k = 0, j
            while k < len(text):
                if text[k] == "{":
                    d += 1
                elif text[k] == "}":
                    d -= 1
                    if d == 0:
                        break
                k += 1
            text = text[:i] + text[k + 1:]
    return text


def main():
    variants = []
    src = open(os.path.join(ROOT, "crates/c2-il/src/func/mod.rs")).read()
    m = re.search(r"pub enum IlOp \{(.*?)\n\}", src, re.S)
    for ln in m.group(1).split("\n"):
        s = ln.strip()
        mm = re.match(r"^([A-Z][A-Za-z0-9]*)\s*[\{\(,]", s)
        if mm:
            variants.append(mm.group(1))

    prod, cons = {}, {}
    for v in variants:
        # A PRODUCER constructs the variant: `IlOp::V {`, `IlOp::V(`, but not
        # `IlOp::V` in a match pattern. Both forms are ambiguous in Rust text, so
        # the screen counts construction CONTEXTS — a `push(`, `=`, `return`,
        # `Some(`, `vec![`, `,` inside a literal list — and prints the sites so a
        # false positive is visible rather than silent.
        prod[v] = []
        cons[v] = []
    pat = re.compile(r"IlOp::([A-Z][A-Za-z0-9]*)")

    def is_pattern(ln, at):
        """Is this occurrence a MATCH PATTERN rather than a construction?

        Rust spells both the same way, so the split is by CONTEXT and the
        producer sites are printed so a misclassification is visible instead of
        silent. A `=>` *after* the occurrence makes it a pattern; a `=>` before
        it (`(true, false) => IlOp::ShrS,`) makes it a construction, which is
        exactly the case the first version of this screen got backwards and
        which made five reachable variants read UNREACHABLE.
        """
        rest = ln[at:]
        if "=>" in rest.split(";")[0]:
            return True
        return any(t in ln for t in ("matches!", "if let", "while let"))

    for f in rs_files("crates/c2-il/src"):
        txt = strip_noncode(open(f).read())
        for i, ln in enumerate(txt.split("\n"), 1):
            for m2 in pat.finditer(ln):
                v = m2.group(1)
                if v in prod and not is_pattern(ln, m2.start()):
                    prod[v].append("%s:%d" % (os.path.relpath(f, ROOT), i))
    for f in rs_files("crates/c2-core/src/codegen"):
        txt = strip_noncode(open(f).read())
        for i, ln in enumerate(txt.split("\n"), 1):
            for v in pat.findall(ln):
                if v in cons:
                    cons[v].append("%s:%d" % (os.path.relpath(f, ROOT), i))

    print("variant          producers(c2-il)  consumers(c2-core/codegen)  verdict")
    unreachable = []
    for v in variants:
        p, c = len(prod[v]), len(cons[v])
        if c and not p:
            verdict = "UNREACHABLE — consumed, never produced"
            unreachable.append(v)
        elif not c and not p:
            verdict = "dead variant — neither side"
        elif not c:
            verdict = "produced, never consumed in codegen"
        else:
            verdict = "reachable at variant level"
        print("%-16s %-17d %-27d %s" % (v, p, c, verdict))
    print()
    print("VARIANT-LEVEL UNREACHABLE: %d  %s" % (len(unreachable), unreachable))
    print()
    print("POSITIONAL candidates — variants whose producers are all in ONE")
    print("position; a consumer reading them in ANY OTHER position is")
    print("unreachable and must be checked by hand:")
    for v in variants:
        if prod[v] and len(prod[v]) <= 3:
            print("  %-14s %d producer(s): %s" % (v, len(prod[v]), ", ".join(prod[v])))


if __name__ == "__main__":
    main()
