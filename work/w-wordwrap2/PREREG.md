# w-wordwrap2 — PREREG

**Frozen before the first `crates/` change.** `git status` over `crates/` is
clean in the commit that carries this file.

Commission: convert `src/system/rndobj/wordwrap.cpp`, TU match 25 → 26.
Board rows **#3030–#3069**.

---

## 0. WORKLOAD STAMP (#2392 — dc3 is not pinned), taken BEFORE the first measurement

```text
c2-rs        6f3ba44c505306d1fe25184bbbd1a0fd8c002529  (merge base with master)
             worktree .claude/worktrees/agent-a6419a796edcaffd3
             branch   worktree-agent-a6419a796edcaffd3
base binary  work/w-wordwrap2/c2rs-base   md5 383363f1fa928fbed4e38283821584f5
             built at the merge base and KEPT; every "base" column is its run
dc3-decomp   b5a9e00a0f6bde9389fc26db881ef4d6a1cf97de   2026-08-10T07:26:55Z
             878 TUs.  dc3 has NOT moved under this lane so far; the generated
             block in docs/STATUS.md already reads workload `b5a9e00a0`.
workload     work/dc3-workload/files.txt  md5 09189d4a41713c77e14dca9af5050b58
             work/dc3-workload/flags.txt  md5 ef3b32e8ac8d3ab89a8be0a0a60e40c8
             the COMMITTED pair, NOT a regenerated one (#2700).
flags        /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc + 8 /I roots
cl.exe/c2.dll/c1xx.dll   compilers/X360/16.00.11886.00
wibo         ../../wibo/build/release/wibo   (wibo 1.2.0-c2rs.1)
capture cache  <main checkout>/work/capture-cache, shared by every worktree
```

**Base scan, this lane's own run of the base binary over all 878 TUs**
(`work/w-wordwrap2/base_scan.log`):

```text
match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 846 · capture-fail 7
factor-a 28 · factor-b 338 · factor-c 169 · factor-d 24 · factor-e 3
b-and-c 151 · a-and-b-and-c 27 · FRONTIER 2 · frontier-if-a 124
fnbyte-exact 35811 · fnbyte-differs 1898 · fnbyte-refused 114622
frontier-bytefrac-top  src/system/rndobj/wordwrap.cpp  12/816 = 1.5%
```

---

## 1. `CEILING.md` §11.4, RUN FIRST AND OFF THIS LANE'S OWN CAPTURE

### Item 1 — ask the BYTE judge, not the census

`work/w-wordwrap2/base_tu.log`, this lane's own single-TU scan:

```text
fnbyte-exact 1 · fnbyte-denominator 3 · fnbyte-differs 0 · fnbyte-refused 2
```

`?WordWrap_SetOption` is exact (w-wordwrap's `global_store_leaf`, already paid).
The other two are `fnbyte-refused`, both behind the READER — `frontier-codegen`
reads *2 behind the reader (66%), 0 measurable, 1 already byte-exact*, so the
lower bound on this TU's codegen distance is 0 and its true distance is
unbounded by any instrument (that column's own caveat).

### Item 2 — is this T1 (all bodies exact, blocker not codegen)?

**No.** 1 of 3, not 3 of 3. Checked, not assumed. So NC-1's six obligations and
NC-2's four are not what is missing and the remaining distance really is
codegen — plus the writer term item 3 forecasts.

### Item 3 — the SYMBOL TABLE as a FORECAST (the item that matters most here)

`work/w-wordwrap2/ref/wordwrap.dump`, 9 sections / 31 symbols / 2,490 B, this
lane's own reference obj at the workload's own flags:

```text
   1 .drectve   2 .debug$S   3 .XBLD$W   4 .bss(588)   5 .XBLD$W
   6 .text(12)  7 .text(164) 8 .text(640) 9 .pdata(8)

  [ 8] .bss                                     sec=4 val=0x0   STATIC aux(len=588 sel=0)
  [10] ?g_uOption@@3IA                          sec=4 val=0x248 EXTERNAL
  [11] ?g_LineBreakTable@@3PAULineBreakEntry@@A sec=4 val=0x0   EXTERNAL
  [24] $M2667  sec=8 val=0x280 LABEL
  [25] $M2666  sec=8 val=0xc   LABEL
  [28] $T2668  sec=9 val=0x0   STATIC
  [29] __restgprlr_29  [30] __savegprlr_29      sec=0 EXTERNAL
```

**The forecast, stated as what the WRITER will owe and not as a checklist:**

1. a **non-COMDAT `.bss`** in the shell slot **between the two `.XBLD$W`
   watermarks** — `emit_comdat_obj` has no insertion point there and returns
   `None` on `IlDataDef::uninitialized` **by name** (#2722, #2727);
2. **two objects in one section**, at `?g_LineBreakTable +0x0` and
   `?g_uOption +0x248` — the REVERSE of declaration order;
3. their **symbols** in the opposite permutation to their storage
   (`?g_uOption` first), inside the `.bss` group and BEFORE the second
   watermark, so every later symbol index shifts by 4;
4. the section characteristics `0xc0400080` — **ALIGN_8BYTES on a section whose
   widest member needs 4**;
5. the objects are shared by **three** functions, one of them **framed**, where
   `emit_comdat_obj`'s `data_defs` model is one COMDAT `.data` owned by one
   unframed function;
6. `__savegprlr_29` / `__restgprlr_29` (paid — `wb-frame`), the `$M/$M/$T`
   triple on the third function only (label channel), and `_fltused` absent.

### Item 4 — is the refusal LIST MEMBERSHIP?

No. Neither remaining key ends in a hex type tag: `expr-brfalse` on
`?IsEastAsianChar` and `expr-cmp-eq` on `?WordWrap_CanBreakLineAt`. Both are
construct keys, not positive-list questions.

### Item 5 — do not trust the key's LAYER, and grep the WORKLOAD for the construct

Both keys are fall-throughs, re-confirmed on this lane's own census run:
`?IsEastAsianChar`'s window stops at `>38<` fifteen bytes into a 502-byte body,
and `?WordWrap_CanBreakLineAt`'s at `>1f<` seventeen bytes into a 2,661-byte
one. Neither names more than the first statement.

**The workload grep, run with a control (#3007's rule).** Two instruments:

* *whole-body range-chain returns* — a function whose entire body is
  `return <chain of (x>=A && x<=B) joined by ||>;` — reports **0**, and it
  **misses its own known positive**: `IsEastAsianChar` has a `g_uOption & 4`
  guard in front of the return, so it is not a single-`return` body. **That
  zero is UNMEASURED and is published as such.**
* the instrument that *does* see the control — any body under 1,400 chars with
  ≥ 2 `>=… && …<=` range tests — reports **17 bodies in 13 TUs**, control
  present. Of the 17 only **three** are call-free range-chain leaves:
  `IsEastAsianChar`, `UTF8.cpp`'s `WToUpper` and `WToLower`. The two UTF8 ones
  are a **different construct** — they produce a *value* through an `else if`
  ladder with `(us & 1)` parity tests, not a bool from `||`-joined ranges.

**So the `?IsEastAsianChar` transcription's population is 1** and it is worth
`+1 fnbyte-exact` and 0 conversions. That is the opposite of `w-wordwrap`'s
`global_store_leaf` result and it is measured the same way.

### Item 6 — check factor A before pricing reader or emitter work

A holds: `wordwrap.cpp` is one of `A∧B∧C`'s 27 and one of the FRONTIER's 2.

### Item 7 — check the board

`#2727` (the `.bss` rung, OPEN, two graded cells left), `#2722`, `#2720`,
`#2625`, `#807` (a `NeedsClass` verdict MASKS an `Unclassified` shortfall on
**this exact TU** — teaching the port `cflow-if-n` alone leaves it
`Unclassified`), `#2387`/`#1416` (the key layer), `#1981` (`counted_accum_loop`
excludes a memory reference BY NAME, and all three binary searches carry an
`lhzx`), `#184` (`MAX_OBJECTS_PER_SECTION`), `#1179` (the third `.bss` slot).

### Item 8 — quote the GATE's number, and it is `gate_cause`

`work/w-wordwrap2/base_tu.jsonl`:

```text
gate_cause   "body-out-of-class"
gate_causes  ["body-out-of-class", "unclaimed-gl-symbol"]
gl_body_starts [3, 3]      selective_bind [3, 3, 2, 0]
emit-bound 3 == emit-gate-segments 3        fn_names 5 vs fn_total 3
```

**No `gl-stop-*`, no `bind-*`: the gate BINDS.** `selective_bind`'s third field
is 2 — the two `.bss` data names — which is the same two `fn_names 5` over-counts
by, and `unclaimed-gl-symbol` is those two names and nothing else.

**Item 8's seventh question — WHICH LANE WOULD HAVE HAD TO PAY EACH TERM** — is
run in §2 and it is where this lane's finding is.

### Item 9 — read the port's FENCES, and ask PREDICATE or GROUND SET

T1 does not fire, so item 9 is run anyway because M12 depends on it.

* `comdat::fenced_inlined_callee` — `?IsEastAsianChar` is **164** emitted bytes
  against `INLINE_DECLINE_BYTES` = 128, so the fence PERMITS the two `bl` sites
  in `?WordWrap_CanBreakLineAt`. Not in the way. (w-wordwrap measured this; it
  is re-read at this tip, not inherited.)
* `gl::plain_external_defined_names` — the GROUND SET half that refused
  `mmio.cpp`. All five `.gl` names here are `?`-mangled and the walk yields all
  of them, so the exemption is live and the fence is not the blocker.
* `elide`'s mechanism E and `splice`'s S7 need an accepted body and this TU has
  one; neither applies to `?WordWrap_SetOption`.

**Verdict: neither a fence PREDICATE nor a fence GROUND SET refuses this TU.**
`gate_cause` is `body-out-of-class` and it means what it says.

---

## 2. THE TWO ROUTE CHECKS (deliverable 1)

### Route 1 — the whole-TU emitter (factor E). **DECLINED, with a reason.**

`w-main2` took this route because `IlBundle::functions()` refuses `src/Main.cpp`
at two *reader* clauses that the whole-TU path does not have to satisfy, and
because three obj-level facts (`Value = 8`, two code regions in one COMDAT, the
EH record set) have no representation in `Selected`/`emit_comdat_obj` at all.
`w-mmio3` checked the same route and declined it because ten of eleven bodies
already shipped per-function and none of `w-main2`'s three triggers was present.

**This TU has neither shape.** Its three triggers, each checked:

| `w-main2` trigger | `wordwrap.cpp` |
|---|---|
| the refusal is a READER clause the whole-TU path skips | **No** — `gate_cause` is `body-out-of-class`; the reader is *at* the bodies |
| an obj-level fact `Selected` cannot express (`Value != 0`, 2 code regions) | **No** — three ordinary COMDATs, every function `Value = 0`, one region each |
| an EH record set / a `.rdata` the writer lacks | **No** — `eh-none` on all 3, `maxState 0`, sections are `.drectve .debug$S .XBLD$W .bss .XBLD$W .text ×3 .pdata` |

And the decisive one: **a whole-TU emitter still has to emit 816 bytes of
PowerPC.** `Main.cpp` is 124 bytes and `TomCryptLicense.cpp`/`ZlibLicense.cpp`
are 24 each; every registered whole-TU recognizer to date emits a body whose
*shape* is fixed by the recognizer, and `?WordWrap_CanBreakLineAt`'s 160 words
are not fixed by anything. The route removes the reader, not the codegen, and
this TU's price is codegen.

**Answer: route 1 buys 0 of this TU's 21 mechanisms.** The only thing it could
have bought is the `.bss` placement — and that is route 2, which is cheaper to
pay directly.

### Route 2 — the `.bss` placement (#2727). **PRICED, and it converts ZERO by itself.**

`A∧B∧C` = 27, `match` = 25, `FRONTIER` = 2 = {`wordwrap.cpp`, `keygen_xbox.cpp`}.
`work/w-wordwrap2/ref/keygen_xbox.obj` (31 sections / 103 symbols) **has no
`.bss` at all**. So the only TU in the whole reachable set that needs this term
is `wordwrap.cpp`, whose other mechanisms are unpaid.

> **Paying the `.bss` term converts 0 TUs today, and that is a hard bound rather
> than an estimate**: `codegen-gap` is 0 over all 878, so every non-matching TU
> is refused at the reader before the writer is consulted, and the two TUs the
> writer could ever be the last blocker for are the frontier's two.

**But the term is much cheaper than #2727 priced it, and that is this lane's
finding.** GRID B (`work/w-wordwrap2/probe/p*.cpp`, 9 cells, real `c2.dll` under
wibo at the workload's own flags, `work/w-wordwrap2/probe/grid_b.txt`) shows
**every layout rule the shell `.bss` needs is ALREADY DERIVED AND SHIPPED**, in
`coff::data::emit_data_obj` — the *functionless* writer, paid by `w-sect`,
`w-bss` and `w-order3`, three lanes that named neither `wordwrap.cpp` nor this
term:

| rule | where it already lives | GRID B cell |
|---|---|---|
| **S1′** slot B — an eager EXTERNAL non-COMDAT `.bss` sits between the watermarks | `data.rs` Rule S1′ | p1 p2 p3 p4 p6 p7 p8 p9 |
| **A1** the storage walk is `.gl` record order, forwards | `data.rs` Rule A1 | p7 (`g3@0 g1@4 g2@8`), p8, p9 |
| **A3′** a plain bump, `placement_align` = `max(t, 1/4/8 by size)`, no free list | `container::placement_align` | p8 (`big@0`, `g1@0x80`), p9 |
| **B1** section nibble = max over members of `placement_align` | `data::section_nibble` | p1/p2/p7 nibble 3 · p8/p9 nibble **4** |
| **Y1-extern** symbols in **reverse `.gl`** order | `data.rs` Rule Y1 | p2 (`g2 g1`), p7 (`g2 g1 g3`), p8, p9 |
| slot **C** — an INTERNAL-linkage object first reached from a function body goes *after* its first referrer's `.text` | #1179, unpaid, must REFUSE | **p5** (`.text(S1) · .bss · .text(R)`) |

**`p9.obj` reproduces `wordwrap.obj`'s `.bss` exactly** — 588 B, characteristics
`0xc0400080`, `?g_first@+0x248` and `?g_arr@+0x0`, symbols in that order — from
two leaf functions and two declarations. So the term is a **COMPOSITION**, not a
derivation: teaching `emit_comdat_obj` to place a section the sibling writer
already knows how to lay out.

That is §11.4 item 8's seventh question (#3001) answered in the same direction
as `w-mmio3`'s: **two of the priced mechanisms were already in the tree, paid by
a lane pricing something else.**

---

## 3. THE PRICE, RE-DERIVED AT THIS BASE — 21 → 20, and 3 are PAID

`w-wordwrap` published 21 (3 + 6 + 12). Re-derived here against `crates/` at
`6f3ba44c` and against this lane's own obj and capture.

| # | mechanism | body | state at this base |
|---|---|---|---|
| M1 | the `.bss` store leaf `lis/st?/blr` production | SetOption | **PAID** — `codegen::global_store_leaf` (w-wordwrap) |
| M2 | `IlDataDef::uninitialized` + `Bindings::resolve_bss_def` | SetOption | **PAID** — w-wordwrap |
| M3 | the four STORE spellings of a low half in `data_defs_of` | SetOption | **PAID** — w-wordwrap |
| M4 | a `.bss` LOAD as a low half (`lis`/`lwz` REFHI/REFLO on a locally-defined object) | IsEastAsian | UNPAID (the `addi`/`lwz` forms exist for `static_scan_loop`; the *defined-`.bss`* binding on a load does not) |
| M5 | `rlwinm. rD,rS,0,31−k,31−k` for `x & (1<<k)`, and `bt 2` on its CR0 | IsEastAsian | UNPAID |
| M6 | the range test `cmplwi cr6,LO ; bt 24 ; cmplwi cr6,HI ; bf 25` | IsEastAsian | UNPAID |
| M7 | `||`-joined range chains with a SHARED false arm, at 3 and 4 ranges | IsEastAsian | UNPAID |
| M8 | the bool materialisation `li 1 / b .+8 / li 0 / clrlwi ,24` | IsEastAsian | UNPAID |
| M9 | `clrlwi rD,r3,16` re-materialised per chain (not CSE'd across the guard) | IsEastAsian | UNPAID |
| M10 | the 112-byte frame with `__savegprlr_29`/`__restgprlr_29` | CanBreak | **PAID** — `wb-frame` / `codegen::frame` at `saved_gprs 3` |
| M11 | the label channel: `$M2666@+0x0c`, `$M2667@+0x280`, `$T2668`, on the THIRD function of a TU whose first two are leaves | CanBreak | UNPAID (lead arithmetic; §7.6/§7.6a form) |
| M12 | two `bl` to a same-TU callee that the inline fence PERMITS (164 > 128) | CanBreak | reader-side, UNPAID |
| M13 | `mr 5,3` in the prologue to free r3 for the two calls | CanBreak | UNPAID |
| M14 | inlined binary search #1 — `sub · srawi ,1 · addze · add · slwi ,2 · lhzx` | CanBreak | UNPAID |
| M15 | inlined binary search #2 (same idiom, different result field) | CanBreak | UNPAID |
| M16 | inlined binary search #3 (`cantBreakAfter`, the `+3` byte) | CanBreak | UNPAID |
| M17 | a data-dependent `do{}while(lo<=hi)` back edge fed by TWO update blocks — **not** `w-bdnz`'s counted class, and #1981 excludes a memory reference from it BY NAME | CanBreak | UNPAID |
| M18 | **a basic block placed AFTER the epilogue** — `.text+0x274` sits below the `b __restgprlr_29` at `+0x270` | CanBreak | UNPAID |
| M19 | the register plan over ~40 blocks: r31 = `ch`, r30 = the table base, r29 = the option word | CanBreak | UNPAID |
| M20 | ~40-block branch-target arithmetic, forward and backward, with three shared tails | CanBreak | UNPAID |
| M21 | the `.bss` SHELL PLACEMENT — the writer term (route 2) | whole obj | UNPAID; **its five layout rules are PAID**, only the composition is not |
| M22 | `data_defs_of` refuses **two `lis rT,sym@ha` in one body** BY NAME, and `?WordWrap_CanBreakLineAt` has exactly two | CanBreak | UNPAID (NEW — no published price names it) |
| M23 | `emit_comdat_obj` refuses `data_defs` on a **framed** function BY NAME, and `?WordWrap_CanBreakLineAt` is framed and carries two | whole obj | UNPAID (NEW — no published price names it) |

**23 named, 3 PAID, 20 live.** The count went *up* by two and *down* by three:
`w-wordwrap`'s M1–M3 shipped in its own lane, and two refusals it never named
(M22, M23) are live and are exactly the ones the `.bss` composition alone does
not clear.

**160 words is still 5.5× the largest body ever transcribed** and M14–M20 is a
lane of its own at least twice over. `?IsEastAsianChar`'s M4–M9 is a second
lane, worth `+1 fnbyte-exact` (item 5's population check) and 0 conversions.

**So `wordwrap.cpp` DECLINES at this lane, at N = 20 live mechanisms.**

---

## 4. WHAT THIS LANE SHIPS — `bss_shell`

The one term whose whole price is a composition of already-paid rules: **the
TU-level non-COMDAT `.bss` in shell slot B on a FUNCTION-BEARING TU**.

Class, every clause a refusal:

1. **1 or 2** uninitialized objects TU-wide (`MAX_OBJECTS_PER_SECTION`, #184);
2. every one **eager EXTERNAL** — an internal-linkage one takes slot C (p5) and
   is refused;
3. not COMDAT, not thread-local, size > 0 (`resolve_bss_def`'s gates);
4. every function unframed and non-float (the incumbent `data_defs` clause,
   unchanged — M23 stays live and is named as such);
5. objects walked in `.gl` record order, bumped by `placement_align`; symbols in
   reverse `.gl` order; section nibble the max.

---

## 5. PREDICTIONS

Rows downstream of a conversion the lane may never reach are marked **cond.**
Rows whose falsifier is another row's negation are **not banked** (`w-main2`).

| # | prediction | p |
|---|---|---:|
| **P1** | `wordwrap.cpp` CONVERTS, 25 → 26 | **0.02** |
| **P1a** | *cond. ¬P1* — the decline is published with N ≥ 15 named and each marked paid/unpaid | 0.90 |
| **P2** | **`fnbyte-exact` delta is EXACTLY 0** — the commission expects a positive delta and this lane's route cannot produce one, because a writer term is invisible to a per-function byte judge (§16.1's finding, applied forward instead of discovered) | **0.85** |
| **P3** | TU match 25 → 25, and the 878-TU verdict SET moves for **nothing** | 0.90 |
| **P4** | `mismatch` 0 on every gate row, both corpora unsampled | 0.90 |
| **P5** | route 1 (whole-TU) is declined with a named reason, and the reason is that the route removes the reader and not the codegen | 0.85 |
| **P6** | route 2 is priced and its conversion yield is **0**, bounded by `keygen_xbox.cpp` having no `.bss` | 0.90 |
| **P7** | ≥ 3 of the `.bss` term's layout rules turn out ALREADY PAID by `emit_data_obj` | 0.80 |
| **P8** | `bss_shell` SHIPS and `fixtures/cpp/wwrap_gstore.cpp` goes `codegen-gap` → **match** | **0.60** |
| **P9** | *cond. P8* — `wwrap_gstore_widths.cpp` does **NOT** reach match, because its three objects exceed `MAX_OBJECTS_PER_SECTION` | 0.85 |
| **P10** | *cond. P8* — a two-object cell (`?g_arr`/`?g_first`, wordwrap's own permutation) is byte-exact on the first graded run | 0.55 |
| **P11** | *cond. P8* — no `codegen/` file is touched; the change is `coff/` + one `c2-il` reader accessor | 0.75 |
| **P12** | the label channel is not touched — no `label_lead`/`label_slots`/`plan_labels` edit | 0.85 |
| **P13** | `hatch-red` still REFUSES, pre-existing, reproduced at THIS lane's base | 0.85 |
| **P14** | the gate's first run is REFUSED for a dirty `crates/` or the sweep `.pyc` (#2979) | 0.55 |
| **P15** | *cond. P8* — the mutation grid grades ≥ 3 cells, and ≥ 1 cell grades nothing and is NAMED rather than counted | 0.70 |

**Unlosable rows, flagged with their falsifiers** (`w-main2`'s rule):

* **P4** is nearly unlosable — its falsifier is a live wrong emit, which has
  fired five times in the project's history and never on a writer-only lane
  whose new path is unreachable from the workload. **Banked at low weight.**
* **P3**'s falsifier is a workload TU converting, which **P1's 0.02 already
  prices**; and P6's falsifier is the same event. P3 and P6 are therefore **not
  three independent rows** — P1, P3 and P6 share one falsifier.
* **P2**'s falsifier is *"a body shipped"*, which is P8's negation only if P8's
  route were per-function; it is not (P8 is a writer term). P2 and P8 are
  independent.

**Effective independent count: 10 of 15.** The dependent cluster is
{P1, P1a, P3, P6} (one event) and {P4} (banked at low weight); the ten that
stand alone are P2, P5, P7, P8, P9, P10, P11, P12, P13, P14, P15 less one for
P9/P10 sharing `MAX_OBJECTS_PER_SECTION` — **10**.

**The row that will decide the outcome is P8**, and its true antecedent is *the
`.bss` section can be inserted into the shell without disturbing the symbol
indices every other section's relocations resolve against* — not *the layout
rules are known*, which §2 already establishes. Written down here so that if
P8 misses, the antecedent is on the record rather than reconstructed.
