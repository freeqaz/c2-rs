#!/usr/bin/env python3
"""fbm_note.py — record, in FUNCTION_BYTE_MATCH.md's own trap-6 block, that the
relocation gap has now been exercised by a shipped mechanism and what it cost.

Lane w-splice scratch. The doc already says the gap exists and that closing it
is "a `c2-obj` rung"; that rung is half-built now, and a lane that reads the
paragraph and not this addendum will assume the gap is still only theoretical.
"""

ANCHOR = """   > day it happened, which is the whole point of measuring a caveat instead of
   > writing one. Board **#884**. Closing it means comparing relocation records
   > against the census's callee names — a `c2-obj` rung.
"""

NOTE = """   >
   > **⚠ 2026-08-08 — THE GAP WAS EXERCISED, and it decided a rung.** Lane
   > `w-splice` shipped a mechanism that *replaces a caller's relocations with
   > its callee's* (`crates/c2-core/src/splice.rs`), so for the 723 functions it
   > moves, "the bytes are right" and "the function is right" stopped being the
   > same question. A per-symbol check against the reference obj's own
   > relocation records — `ObjImage::text_comdat_relocs`, the `c2-obj` reader
   > this paragraph asks for, board **#994** — found **150** wrong targets in
   > the first shipped version of that rule, **77** in the second and **1** in
   > the third. **FBM scored every one of them `exact`.** Each round changed the
   > rule (#989, #991) and the shipped tip is 723 of 723 verified with 0
   > disagreements, `fnbyte-exact-relocated` unmoved at 4,664.
   >
   > Two things follow for this document. **The trap is not theoretical any
   > more** — it has a measured instance count and a rung that would have
   > shipped green without it. And **the reader is built**: closing #884 for the
   > whole 4,664 is now a matter of running that comparison over every credited
   > function rather than only over the ones one mechanism moves, which is what
   > a concurrent lane is doing in `gap/fnbytes.rs`.
"""

p = "docs/FUNCTION_BYTE_MATCH.md"
s = open(p).read()
assert ANCHOR in s, "trap 6's closing lines moved"
assert "w-splice" not in s, "already appended"
open(p, "w").write(s.replace(ANCHOR, ANCHOR + NOTE, 1))
print("appended")
