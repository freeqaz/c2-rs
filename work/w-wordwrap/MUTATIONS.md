# w-wordwrap — must-fail mutations, and the two cells that cannot be graded

Each row breaks one **conjunction** of `global_store_leaf`'s fence, rebuilds,
and re-grades this lane's nine fixture cells with the BYTE judge
(`work/w-wordwrap/mutate.sh` + `mut.py`; transcripts under `out/fxbyte-mut-*`).

**The grading is `fnbyte-*` and not the TU verdict, and that is forced rather
than chosen.** The class's object is a non-COMDAT `.bss` that
`coff::writer::emit_obj_multi` refuses by name, so no fixture of this class can
reach `match` and the verdict column says nothing about it. `fnbyte` is still
the oracle — real `c2`'s own obj, the bytes **and** all four relocation records
— so a mutation that admits a `_neg` cell surfaces as **`fnbyte-differs`**,
which is the same evidence a `mismatch` would be and is strictly per-function.

`mutate.sh` **refuses on a dirty `crates/`** (#2668) and **verifies its own
restore** afterwards (#2699: a lane's own restore trap discarded the fix it was
written to grade).

---

## The graded four

| # | clause deleted | cell | base | mutated |
|---|---|---|---:|---|
| **M1** | `gstore-value-is-not-the-formal` — the value token must be `params[0]` | `wwrap_gstore_gg_neg.cpp` | refused 1 | **differs 1** |
| **M2** | the `2C` no-conversion clause **and** the store-type restatement, **together** | `wwrap_gstore_conv_neg.cpp` | refused 1 | **differs 1** |
| **M3** | the one-formal arity fence **and** the value-token comparison, **together** | `wwrap_gstore_second_neg.cpp` | refused 1 | **differs 1** |
| **M4** | the absence of GRID T's `86 45 …` row — float admitted at width 4 | `wwrap_gstore_float_neg.cpp` | refused 1 | **differs 1** |

Every one of the four turns a refusal into a **wrong emit**, which is what a
must-fail mutation has to produce; a mutation that merely moves a body from one
refusal key to another has graded nothing (#1465's shape).

The accepted cells stay `exact 1` and `exact 3` under all four, so each mutation
is separating exactly the cell it names.

### M2 and M3 are MERGED on purpose, and M3 is the lesson learned here

`w-xtea2` **#2665**: a cell fenced by several clauses grades **none** of them,
because deleting one leaves the others refusing it, and the repair is *merging*
the clauses into one mutation rather than adding cells.

**M3's first run is that finding reproduced live rather than quoted.** Deleting
the arity fence alone came back **GREEN** — `second_neg` stayed `refused 1` —
because its value is `params[1]` and `val_tok != params[0]` refused the body
anyway. The two clauses are one conjunction over that cell. The grid above is
the second run, with both deleted.

M2 was merged before its first run, from the same rule: `conv_neg`'s stream
carries a `2C` **and** a store type that does not restate the value's, and
either clause alone refuses it.

---

## M5 — VOID, and the reason is named

**The mode gate cannot be graded by deleting it**, because a second,
bundle-level gate refuses `/Od` independently. With the parser clause deleted,
`c2rs census` at `/Od /GS- /c` still reads

```text
  [  0] GAP opt-mode-00800005   cflow-straight+expr-modeled …
```

— `census_functions`' own post-parse gate (b), which raises `OPT_MODE` before a
row can be `InClass`. And at `/Od` without `/Gy` there is no COMDAT `.text`
split at all, so `fnbyte-denominator` is **0** and the byte judge has nothing to
compare; at `/Od /Gy` the denominator exists and every body reads `refused`.

So the clause is **redundant** with a gate one layer up. It is kept anyway, and
that is board **#1638**'s rule rather than caution: the gate belongs in the
PARSER so the census reports the class's own refusal key rather than a generic
one. Recorded as VOID rather than counted, because a clause no mutation can
break is not a clause any mutation has graded.

---

## The three cells that no single deletion can grade, named rather than counted

`wwrap_gstore_lit_neg.cpp` (the value is a literal), `wwrap_gstore_two_neg.cpp`
(two statements) and `wwrap_gstore_sub_neg.cpp` (a subscripted destination) are
refused **structurally**: deleting the clause that names them desynchronizes the
token cursor rather than admitting the body, so the parse fails anyway and the
mutation comes back green for the wrong reason.

They are kept as cells because they are the compiled record of what c2 emits for
each neighbour — 16 B with `lis r10`, 20 B with both high halves hoisted, 16 B
with the REFLO moved onto an `addi` — which is the evidence the class's three
fixed words rest on. **They are not counted among the must-fail mutations**, and
#2698's rule is why: a cell that grades none of its clauses is documentation,
not a fence, and calling it one inflates the count.

---

## The control

Reverting the class's dispatcher arm alone (`if false && …`) returns
`wwrap_gstore.cpp` and `wwrap_gstore_widths.cpp` to `vocab-gap` with
`fnbyte-exact 0` — i.e. every accepted cell is accepted *by this production* and
by nothing else. Recorded as a different kind of evidence from the four
must-fails and not counted among them (`w-xtea3` §5.1's convention).
