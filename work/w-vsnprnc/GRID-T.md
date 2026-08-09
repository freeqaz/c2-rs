# GRID-T — the second function, isolated refusal by refusal

`vsprintf_s` is 12 bytes: `mr r7,r6 ; li r6,0 ; b _vsprintf_s_l`, unframed, its
own `.text` COMDAT, **no `.pdata` record and no `$M` label**, in a TU whose
other function is framed and has both. Graded with real `c2.dll` at the
workload's own flags and cwd.

| cell | shape | verdict |
|---|---|---|
| `c5` | C++ linkage, 5-arg IDENTITY passthrough tail call | **match** |
| `ecshort` | the same body, `extern "C"`, names ≤ 8 chars | vocab-gap |
| `eclong` | the same body, `extern "C"`, names > 8 chars | **match** |
| `cppshort` | C++ linkage, source names ≤ 8 chars (decorated names are long) | **match** |
| **`c5lit`** | **C++ linkage, one ascending move + one literal in the slot it vacates** — 12 B, **byte-identical to `vsprintf_s`** | **vocab-gap** |
| `c5litec` | the same, `extern "C"`, long names | vocab-gap |
| `p1` | a framed function THEN an unframed leaf — `vsnprnc`'s own order | **match** |
| **`p2`** | **the unframed leaf FIRST, then the framed one** — the LIVE label-charge order | **match** |
| **`p3`** | **two leaves then a framed one** — a second label-charge cell | **match** |

## Three things this settles

1. **`extern "C"` is NOT a refusal. The 8-byte COFF inline name field is.**
   `ecshort` refuses and `eclong` matches on the *same four bytes*; the only
   difference is whether the defined symbol's name fits inline. That is
   w-extdata's `INLINE_NAME_MAX` clause doing its job. **It does not touch
   `vsnprnc`** — `_vsprintf_s_l` (13) and `vsprintf_s` (11) are both over.
   Recorded because a probe family built on short `extern "C"` names reads as
   "the shape refuses" when the shape is fine, which is how a lane prices a free
   thing at one refusal.

2. **The second function's refusal is exactly one clause**, `call-arg-lit-permuted:mid`,
   and `c5lit` reproduces its twelve bytes in three lines.

3. **A framed function sharing a TU with unframed leaves is FREE — and the label
   charge was measured LIVE, not read off `docs/LABEL_COUNTER.md`.** `p2` and
   `p3` put the leaf(s) *ahead* of the framed function, so a wrong charge shifts
   the framed function's `$M` numbers and the cell fails. All three match. The
   PREREG's R5 — priced as a refusal, and the one that owned decline clause
   D-COFF — **costs zero, and `codegen/coff.rs` does not need to be touched.**
