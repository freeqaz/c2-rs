#!/usr/bin/env python3
"""diffcheck.py — did this lane change any DECIDING code?

    diffcheck.py <base-rev>

`w-self2b` could check "0 added lines outside doc comments and `mod tests`",
because it added no field. **This lane adds fields and populates them**, so that
check would fail and would be the wrong one anyway. The claim here is narrower
and stronger:

  1. **`alloc::allocate`'s body is BYTE-IDENTICAL to the base revision's.** It
     is the one function that decides a register, and every one of the ten dead
     keys would have been a change to it.
  2. **`alloc::all_in`'s body is byte-identical too** — the guard the store
     emitters actually call before emitting.
  3. **`eat_offset_adds`'s observable contract is unchanged**: same name, same
     signature, and its two existing tests are unedited. A peer lane reads it.
  4. Every added line outside those functions is a **doc comment, a struct
     field, a `#[cfg(test)]` test, or the carrier's own constructors** — listed,
     not asserted, so a reader can see what is left.

Exits non-zero if 1, 2 or 3 fails. `git` is the only input; no toolchain.
"""

import re
import subprocess
import sys

FILES = {
    "alloc": "crates/c2-core/src/codegen/alloc.rs",
    "store": "crates/c2-core/src/codegen/leaf/store.rs",
    "desig": "crates/c2-il/src/func/body/shapes/designator.rs",
}


def at(rev, path):
    return subprocess.run(["git", "show", "%s:%s" % (rev, path)],
                          capture_output=True, text=True, check=True).stdout


def body(src, sig):
    """The text of one function, from its signature line to its closing brace
    at the same indentation. Brace-counted, not regex-matched."""
    i = src.find(sig)
    if i < 0:
        return None
    depth, j, started = 0, i, False
    while j < len(src):
        if src[j] == "{":
            depth += 1
            started = True
        elif src[j] == "}":
            depth -= 1
            if started and depth == 0:
                return src[i:j + 1]
        j += 1
    return None


def main(base):
    ok = True

    for name, sig in (("allocate", "pub fn allocate(producers: &[Producer]"),
                      ("all_in", "pub fn all_in(producers: &[Producer]")):
        a = body(at(base, FILES["alloc"]), sig)
        b = body(at("HEAD", FILES["alloc"]), sig)
        if a is None or b is None:
            print("  %-28s NOT FOUND at one end — inconclusive" % name)
            ok = False
            continue
        same = a == b
        print("  %-28s %s  (%d bytes)"
              % (name, "BYTE-IDENTICAL" if same else "**CHANGED**", len(b)))
        if not same:
            ok = False

    # 3. the reader's contract
    a = at(base, FILES["desig"])
    b = at("HEAD", FILES["desig"])
    sig = "pub(crate) fn eat_offset_adds(seg: &[u8], p: &mut usize)" \
          " -> Option<(i32, Option<(u8, u8)>)>"
    print("  %-28s %s" % ("eat_offset_adds signature",
                          "UNCHANGED" if (sig in a and sig in b) else "**MOVED**"))
    if not (sig in a and sig in b):
        ok = False
    for t in ("fn the_offset_add_run_is_one_walk_with_two_readings",
              "fn an_empty_offset_add_run_reports_no_retype"):
        ta, tb = body(a, t), body(b, t)
        same = ta is not None and ta == tb
        print("  %-28s %s" % (t.replace("fn ", "")[:28],
                              "UNEDITED" if same else "**EDITED**"))
        if not same:
            ok = False

    # 4. what is left, listed rather than asserted
    print("\n  ADDED LINES IN `crates/`, BY KIND (listed, not asserted)")
    d = subprocess.run(["git", "diff", "%s..HEAD" % base, "--unified=0", "--",
                        "crates/"], capture_output=True, text=True).stdout
    kinds = {}
    for line in d.split("\n"):
        if not line.startswith("+") or line.startswith("+++"):
            continue
        s = line[1:].strip()
        if s.startswith("///") or s.startswith("//!") or s.startswith("//"):
            k = "doc / comment"
        elif not s:
            k = "blank"
        else:
            k = "code"
        kinds[k] = kinds.get(k, 0) + 1
    for k in sorted(kinds):
        print("    %-16s %5d" % (k, kinds[k]))

    print("\n  %s" % ("PASS — no deciding code changed" if ok
                      else "FAIL — a deciding function moved"))
    return 0 if ok else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1]))
