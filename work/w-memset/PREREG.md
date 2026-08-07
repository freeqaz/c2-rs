# w-memset — PREREG

    Lane:    w-memset, worktree off master `217d4a85`
    Branch:  wt-w-memset
    Written: before one line of reader code and before the first `cl.exe` of any
             CELL. Orientation probes (§0) ran first and are declared, because
             they are what refuted the brief's premise and they consumed
             already-committed lane work rather than grading anything.

---

## 0. THE BRIEF'S PREMISE IS STALE, and this is declared before any prediction

The brief commissions *"read the `expr-intrinsic-memset` production, so the 369
mechanism-E bodies behind it become reachable"*, citing `w-seq` §5 (master
`039b718`).

**That production has already been read, and the reader is on master.** Lane
`w-inl0` (rung `docs/rungs/2026-08-08-w-inl0.md`, merged at `2d0b79cd`) ships
`crates/c2-il/src/func/body/shapes/no_effect.rs` — a decode-only recognizer for
exactly this body — plus `FnCensus::no_effect_callee` and
`c2_core::elide::Reduction`. It converted **138** of board #980's 370 and
published the residue.

The brief's instruction *"grep -ril 'memset\|intrinsic' docs/ and read the OLDEST
hit … check whether this production has already been measured — if it has,
consume that work rather than redoing it"* is therefore answered: **it has**, by
`docs/IL_INTRINSIC_CALL.md` (2026-07-30, the id table and the fail-closed
decode) and by `docs/rungs/2026-08-08-w-inl0.md` (the production, read). This
lane consumes both and re-derives none of it.

### 0.1 What is actually left, measured on master `217d4a85`

One 878-TU scan at the tip of master (`work/w-memset/base_scan.txt`) reads:

```
fnbyte-blr-stop-expr-intrinsic-memset        231
fnbyte-blr-stop-callee-unbound                 1
fnbyte-blr-stop2-return-scope-close-cflow-label  228
fnbyte-blr-stop2-module-end-0x4D                 2
fnbyte-blr-stop2-callee-unbound                  1
```

and `work/w-inl0/target370.py` re-derived on this scan's
`--fnbyte-diff-jsonl` says **232** differs whose whole reference body is one
`4e800020`, **all** `??$_Destroy_Range@…`, **all** with the port emitting the
same two words `38a00000 4bfffffc`.

So the number this rung can be about is **232**, not 369, and the production that
blocks it is **not** `expr-intrinsic-memset`.

### 0.2 The chain, read from the workload and from a hand cell, both

`src/lazer/meta_ham/CharacterProvider.cpp`, through `c2rs census --fn` (added by
this lane) — five levels, three of them refused:

| # | function | census key | what it is |
|---|---|---|---|
| 1 | `??$_Destroy_Range@PAVSymbol@@…` | **in class**, `multiarg-tail-call` | THE DIFFER. port `li r5,0 ; b`, c2 `blr` |
| 2 | `??$__destroy_range@PAVSymbol@@V1@@…` | `expr-intrinsic-memset` | the dead temporary — **already read** (`w-inl0`) |
| 3 | `??$__destroy_range_aux@PAVSymbol@@…` (`__false_type`) | `return-scope-close-cflow-label`, `cflow-loop` | **A LOOP. NOT READ.** |
| 4 | `??$_Destroy@VSymbol@@…` | `expr-intrinsic-memset` | a second dead temporary — already read |
| 5 | `??$__destroy_aux@VSymbol@@…` (`__false_type`) | `expr-lit-type-8207` | **a pseudo-destructor. NOT READ, and it is a SEED.** |

`work/w-inl0/cells/m06.cpp` reproduces levels 1–3 and its `?aux` opens with the
**same production, byte for byte modulo token width and line markers**:

```
cell      4c 4f 11 53 53 3a fa09 29 fb09 26 f509 33 86 41 12 04 0f 86 43 8120 4b 29 fa09 b9 …
workload  4c 4f 11 53 4f 01 <L> 53 3a b5ba0500 29 b6ba0500 26 e6ad0500 33 86 41 12 04 0f 86 43 c6fe02 4b 29 b5ba0500 b9 …
```

`w-fix` §5.1 (board **#953**) records that a hand cell and the workload can reach
one source idiom through *different* productions and that neither substitutes for
the other. **Here they do not diverge**, and that is registered as a claim to be
checked (P2), not assumed.

### 0.3 THE STOP, and why this lane does not step over it

Level 5's body is

```
4c 4f 11 53 <line> 33 86 41 74 00 · 33 82 07 03 00 · 44 · 4b <line> <return plumbing>
```

— an `int` literal, a **`void` literal**, a bind and a discard. There is **no
call in it at all**. For it to reduce to nothing it must **SEED** the fixpoint.

`c2_core::elide::Reduction` has exactly two variants and its own doc pins the
asymmetry: *"a refused body contributes a **link and never a seed**"*.
`NoEffectCall(&str)` cannot express level 5, so **with `elide.rs` untouched, no
chain through a pseudo-destructor leaf can close.**

The lane brief says: *"Do not change `crates/c2-core/src/elide.rs`'s rule — you
are feeding it, not editing it; if you believe E's rule must change, STOP and
report."* **E's rule must change, and this lane therefore stops at it and reports
it with a price** (§4 P4). Nothing in `crates/c2-core/` is edited.

---

## 1. What this lane will build

**Decode-only, `crates/c2-il/` only.** `parse_segment` byte-for-byte unchanged,
census verdict still `FnVerdict::Blocked`, census key unchanged,
`IlBundle::functions()` untouched — the same four-way containment `w-inl0` §3
established and board **#971** condition 4 requires.

> **THE DESTROY-LOOP READER (S3).** A body whose whole content is
>
> ```
>   53                      the loop's scope
>   3A  <Lcond>             goto COND
>   29  <Lincr>       INCR:
>       <induction step>    one lvalue, one literal, one op, discarded
>   29  <Lcond>       COND:
>       <load a> <load b> 20        a comparison of two of this function's formals
>       38 <Lexit>                  branch-false out
>       <ONE call statement>        the SAME closed vocabulary `no_effect_call`
>                                   already walks, plus its dead temporaries
>       3A <Lincr>                  goto INCR
>   29 <Lexit>        EXIT:
>   54 <n>  <return plumbing to the segment end>
> ```
>
> emits nothing **provided** the call's callee reduces to nothing. It returns
> that callee's token — a **LINK**, exactly like S1, and never a seed.

Soundness is the same four things `w-inl0` §2.3 states, and they are re-stated
because a loop earns one more:

1. **The walk is TOTAL over the segment** — it ends on `eat_return_plumbing`'s
   fail-closed terminal, so nothing else in the body can read anything.
2. **The answer is CONDITIONAL on the callee**, resolved by `elide.rs`'s own
   fixpoint, never by this reader.
3. **The induction step is PURE** — one lvalue of this function's own formals,
   one literal, one arithmetic op, discarded. Anything else refuses.
4. **Every label is matched.** `Lcond`, `Lincr` and `Lexit` are read and required
   to be the three distinct labels the shape mints, so a body with an extra
   branch target is refused rather than assumed to be this loop.

---

## 2. Cells

`work/w-memset/cells/`, `sha256` manifest committed **before the first
`cl.exe`**, graded by **real c2 under wibo**, at the workload's own flags **and**
with `/Ob0` appended (an absent REL24 at `/O1` alone cannot tell E from I —
`w-empty` §2). Every cell carries an **anchor** whose callee the TU does not
define; a cell whose anchor loses its relocation is **refused, not scored**.

The axes are **structural**, not values (`w-inl0`'s `L1` and #644 both say a
value that is constant on one corpus is not a gate):

| axis | varied over |
|---|---|
| the loop's **body** | one call · two calls · a call plus a store · an empty body |
| the loop's **callee** | empty · non-empty · external · itself a dead-temporary |
| the **induction** | `++f` · `f += k` · a second induction variable · none |
| the **condition** | `f != l` · `f < l` · a condition over a non-formal |
| the **leaf** | the pseudo-destructor · a real destructor call · a real `memset` |
| the **element** | scalar (the 138 that already closed) · class (the 232 that did not) |

---

## 3. THE DECLINE FLOOR, registered against the incumbent

The incumbent is master `217d4a85`, measured by this lane's own baseline scan,
not quoted from a rung:

```
fnbyte-exact 35986 · reloc-differs 861 · whole-TU 2 · differs 2334 · partial 0
fnbyte-refused 130579 · unbound 9217 · denominator 178977 · exact-bytes 36847
fnbyte-elided 1654 / -elided-exact 1654 · mismatch 0 · TU match 10
factors A 28 · B 338 · C 169 · D 10 · E 2 · FRONTIER 17
```

**The `crates/c2-il/` change is REVERTED, not argued about, if any of:**

1. `fnbyte-exact` < **35,986**, or **any** function enters `differs` (checked per
   `(TU, emit_name)` from the scan's own witness keys, never by subtracting
   totals);
2. `mismatch` > 0 at any gate row, or `gate.sh` is not 18/18 PASS;
3. `fnbyte-reloc-differs` > **861**;
4. `fnbyte-refused` ≠ **130,579**, or `vocab-gap` ≠ 861 — either would mean
   `parse_segment` or `functions()` moved, which this lane forbids itself;
5. `fnbyte-elided` ≠ `fnbyte-elided-exact` — a body the closure produced that
   real c2 disagrees with;
6. any peer lane's `gap-metric` key family stops printing (`peerkeys.py`, both
   ends).

---

## 4. Predictions

| # | prediction |
|---|---|
| **P1** | **The claim I most expect to lose.** With `elide.rs` untouched, the loop reader alone converts **0** of the 232 — because every one of them bottoms out at a pseudo-destructor leaf that must SEED. Registered as a point prediction of **0**, and it loses if any of the 232 converts. |
| **P2** | The hand cell and the workload reach the loop through the **same** production: the reader written against the cell fires on `??$__destroy_range_aux@PAVSymbol@@…` in `src/lazer/meta_ham/CharacterProvider.cpp` without a second grammar. (`w-fix` #953 says this need not hold.) |
| **P3** | Real c2 emits **one `4e800020` and zero relocations** for the loop function `?aux` itself, at the workload's flags **and** at `/Ob0` — so it is E, not I, and the chain's shape is c2's own DCE (`w-inl0` §5 measured this for the wrapper; this registers it for the LOOP body). |
| **P4** | The residue after this lane is priced to **one missing capability, not one missing production**: an E **seed** from a refused body. Point prediction — the number of `differs` that would close if `Reduction` gained a seeding variant is **232**, and the number of `fnbyte-noeffect-*` rows whose chain stops at the pseudo-destructor is **2,572** (`fnbyte-noeffect-stop-expr-lit-type-8207`). |
| **P5** | The loop reader fires on **> 0** workload rows and the number it fires on is **≥ 228** (the `blr-stop2` count), because that key already proves at least 228 loops sit at the bottom of a read chain. |
| **P6** | **CONTROL** — `git diff master..HEAD -- crates/c2-core/` is **empty**; `parse_segment`/`parse_segment_detail` are byte-for-byte unchanged; `fnbyte-refused` 130,579 and `vocab-gap` 861 at both ends; `mismatch` 0; the FBM partition identity holds (`fnbyte-partition-broken` 0). |
| **P7** | **TWO INSTRUMENTS.** A crate-free Python re-implementation of the reader over the captured `.ex` and the shipped Rust reader agree **count-by-count** on one named TU, and any discrepancy is explained rather than closed. |
| **P8** | **MUTATIONS.** Three registered must-fail edits — drop the label matching, drop the purity of the induction step, drop the totality terminal — each goes **RED** with a distinct named message, and each is verified to have changed the file it names before its run is read (board #951). |

### 4.1 The honest expectation, stated up front

The brief asks *"what you predict the 369 do once parsed (E, and how many — the
honest expectation is that some will not be E and you should say so up front)"*.

**The honest expectation is that ZERO of the 232 convert in this lane**, that all
232 *are* mechanism E in c2 (P3), and that what separates them from the port is
**not** a decode refusal alone. That is a registered decline, and §3's floor
exists so it cannot be talked into a conversion.

---

## 5. What is NOT in scope, said before it is tempting

* **`expr-op-0x27`** (573 blocked differs) — the brief names it as a documented
  dead end (boards #622, #662, #970) and this lane does not go there.
* **Widening `IlBundle::functions()`** to *accept* the dead-temporary body
  (`w-inl0` §9.5, worth ~369 more `exact`) — declined for #971 condition 4's
  reason, and because it needs a `Selected` arm in `crates/c2-core/`, which
  lane **w-target** owns this wave.
* **Any change to `crates/c2-core/src/elide.rs`.** §0.3.
