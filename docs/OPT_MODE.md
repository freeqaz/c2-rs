# The per-function optimization word, and which mode the port actually targets

`.ex` carries a **per-function optimization-settings word** immediately after each
`4F 1F` function-start marker, and the port has never read it. Everything below is
from live captures.

The consequence was the headline when this was written: **the port's byte-exactness
was a claim about `/Ox`, and the entire real workload is `/O1`.** Those two modes
emit different code for the same source, including for the core MVP class.

**Both are now supported** (`2a19090`), and the mode is read from the word rather
than assumed — 31 fixtures byte-exact at `/Ox`, 24 at `/O1`, 0 mismatches in either
(§4.1). What follows is the characterization that got there, kept because the
remaining `/O1` gaps are all described in terms of it.

## 1. The encoding

```
4F 1F 80 <LE32 word>
```

Seven bytes, at the start of every function segment. `split_functions` anchors on
the `4F 1F` and the port then skips straight past the word.

## 2. Observed values

One source (`int f(int a) { return a + 1; }`), varying only the compile flags:

| flags | word |
|---|---|
| `/Ox` | `00a00005` |
| `/O2` | `00a00005` |
| `/O1` | `00200005` |
| `/Od` | `00800005` |
| `/Ot` alone | `00800005` |
| `/Ob0` alone | `00800005` |
| `/Oy-` alone | `00800005` |

And varying only the source, at `/Ox`:

| source | word |
|---|---|
| baseline | `00a00005` |
| `#pragma optimize("", off)` | `00800004` |
| `#pragma optimize("", on)` | `00a00005` |
| `#pragma optimize("s", on)` | `00200005` |
| `#pragma optimize("t", on)` | `00a00005` |

Nothing else moved it: `static`, `__declspec(noinline)`, `__forceinline`,
`__declspec(dllexport)`, `extern "C"`, `void` return, `float` return, parameter
count, and a tail call all leave it at `00a00005`.

### 2.1 A reading of the bits — hypothesis, not established

Two bits move, and the flag semantics line up if `0x00200000` is *optimizations
enabled* and `0x00800000` is *favor speed*:

* `/Ox` and `/O2` set both — optimize, for speed.
* `/O1` sets only `0x00200000` — optimize, for size. `#pragma optimize("s", on)`
  under `/Ox` produces the identical word, which is the cross-check: two very
  different ways of saying "optimize for size" agree on the encoding.
* `/Od` sets only `0x00800000` — not optimizing; the speed/size preference is
  still at its default. `/Ot`, `/Ob0` and `/Oy-` *alone* land here too, correctly:
  none of them implies an `/O` level.

The low nibble is `5` everywhere except `#pragma optimize("", off)`, which gives
`4`. Not explained. Treat the whole word as opaque and compare it whole.

## 3. `/Ox` and `/O1` emit different code

Reference objs for the same fixture, differing only in the `/O` flag:

| fixture | `/Ox` `.text` | `/O1` `.text` | |
|---|---|---|---|
| `w5_chain` | 68 B | 68 B | differs |
| `w5_tree2` | 64 B | 64 B | identical |
| `w5_tree3` | 112 B | 112 B | differs |
| `il_accum4` | 144 B | 136 B | differs |
| `il_reassoc` | 224 B | 176 B | differs |
| `w6_rel_k` | 464 B | 428 B | differs |
| `w13b_fconst` | 16 B | 16 B | identical |
| `il_call_perm` | 108 B | 96 B | differs |
| `il_deep_chain` | 76 B | 72 B | differs |

Three mechanisms are visible so far — and a **fourth** that is not a mechanism of
the same kind, added 2026-08-04 and stated first because it bounds everything
below it.

### 3.0 REFUTED — the modes differ in BLOCK STRUCTURE, not only in a register field

**Board #258 (measured by `w-cross`), #272 (implemented by `w-conv`). Recorded
here by `w-book4`; `crates/` was corrected first and this document was not, which
is the defect this section closes.**

Every claim in §3.1–§3.3 and §4.2 is measured on **straight-line integer chains
and depth-2 trees** — bodies with **one block**. This document then stated the
conclusion without the qualifier, and the un-qualified form is false:

```
void e(int a){ if(a) v0(); else v1(); v2(); }

  /O1         52 B, and it contains an intra-section `48000008`
  /Ox, /O2    68 B, and it contains NO `b` at all
```

`/Ox` and `/O2` **tail-duplicate** the join block and its epilogue into every
arm — the join's `bl` and all four epilogue words appear **twice**; `/O1`
**shares** them behind a branch. That is a different **opcode set** and a
different **block count**, not a different register field.

Three consequences, in decreasing order of how likely they are to be
re-discovered the hard way:

1. **Anyone quoting §3.1's rule outside a straight-line chain is quoting a
   refuted claim.** The rule holds where it was measured. It was never measured
   on a body with more than one block.
2. **The threshold is a c2 COST MODEL and is still open.** It is bracketed by one
   cell either side — a **one**-call join duplicates, a **two**-call join does
   not — which is the same class as `CFG_SHAPE.md` §3.5's declined fold bands
   (board #187). Do not fit it from the two bracketing cells.
3. **The guarded early return is the case where there is no threshold to fit**,
   and it is therefore the only one implemented: the duplicated block is the
   epilogue, whose length is a constant of the frame class, and `/Ox` copies it
   in **every measured cell**. `crate::codegen::calls::call_seq_text` is the one
   place that reads `OptMode` for anything other than a register field. The
   `void` early return is the control — it is **byte-identical at `/O1` and
   `/Ox`**, so the split is a property of the duplicated epilogue and not of the
   branch (board #273).

**This is also what blocks #191's other half.** The `else` arm's `b` is out of
reach for three independent reasons and only one of them is the branch: the tail
duplication here, `CFG_SHAPE.md` §3.6's dead epilogue, and board #261 — c2
propagates the branch condition's **value range** into the arm (`if(a!=0) a1(b);
else a1(a);` emits `li r3,0` for `a` on the else path), so an `else` arm cannot
be lowered from the IL's operands alone.

### 3.1 `/O1` allocates by liveness; `/Ox` descends regardless

This is the big one, because it is the rule the port spent the most effort on.
Enumerated over all 108 three- and four-operator integer chains (every operator
combination from `+ - *` over five parameters):

* **47 of 108 differ** between the modes;
* the **word counts are identical in every case** — so instruction *selection*
  agrees for this whole class, and only register assignment and padding differ.

For a left-linear chain, where only one intermediate is live at a time:

```
a * b * c * d
/Ox   mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6      descending r11, r10
/O1   mullw r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6      r11 reused

a - b - c - d
/Ox   subf r11,r4,r3 ; subf r10,r5,r11 ; subf r3,r6,r10
/O1   subf r11,r4,r3 ; subf r11,r5,r11 ; subf r3,r6,r11
```

But `/O1` is not simply "always r11". Where two subexpressions are live at once
it uses a second register, exactly as it must:

```
a * b + c * d
/O1   mullw r10,r3,r4 ; mullw r11,r5,r6 ; add r3,r10,r11
a + b * c + d * e
/O1   mullw r10,r4,r5 ; mullw r11,r6,r7 ; add r10,r10,r11 ; add r3,r11,r3
```

18 of the 108 keep a non-r11 intermediate at `/O1`, and they are the tree shapes.
So the rule is **ordinary liveness-based allocation: reuse r11 when the previous
intermediate is dead, take a second register when it is not.** `/Ox` descends
even when the previous intermediate *is* dead.

Which means `il_accum4.cpp`'s rule — "c2 decides accumulator-versus-descending
once for the whole chain; a chain with no addition gives each intermediate its own
descending register" — is a **`/Ox` rule**, and a peculiar one: at `/Ox` whether a
linear chain descends depends on the operators in it, which liveness cannot
explain. Establishing it cost 270 mis-emits found by a generated sweep. None of
that work transfers to `/O1`, where the same chains follow the simpler rule.

(A naive "rewrite every r8/r9/r10 to r11" transform of the `/Ox` output
reproduces `/O1` for 477 of 513 words and fails on exactly the 36 belonging to
those 18 tree shapes — which is how the over-strong "unconditionally r11" reading
of the first two captures was caught. Two linear chains cannot distinguish
"always r11" from "r11 when dead".)

#### The rule, stated exactly

Enumerating all 27 depth-2 trees `(a op b) op (c op d)` over `+ - *` and
disassembling both modes side by side settles it. In **all 27**, instruction
selection, operand order and operand *choice* are identical; the only thing that
ever differs is which register an intermediate is written to. And where two values
are genuinely live at once, the two modes agree exactly — same registers, same
assignment:

```
(a * b) + (c * d)   both   mullw r10,r3,r4 ; mullw r11,r5,r6 ; add   r3,r10,r11
(a + b) * (c + d)   both   add   r11,r3,r4 ; add   r10,r5,r6 ; mullw r3,r11,r10
```

The difference appears only where the previous intermediate is **dead**:

```
(a * b) * (c * d)   /Ox    mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
                    /O1    mullw r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6
(a * b) - (c + d)   /Ox    mullw r11,r3,r4 ; subf  r10,r5,r11 ; subf  r3,r6,r10
                    /O1    mullw r11,r3,r4 ; subf  r11,r5,r11 ; subf  r3,r6,r11
```

So:

> **Inside a straight-line chain, `/O1` is `/Ox` with exactly one change: an
> intermediate whose predecessor is already dead is written to r11 rather than to
> a fresh descending register. Simultaneously-live values are allocated
> identically in both modes.**

That is a local change to the allocator — the dead-intermediate case of the
descending path becomes "reuse r11" — and nothing else *in a straight-line chain*
moves. It also means the `/Ox` operand-order and canonicalization decisions,
which is where the reverse-engineering effort actually went, are shared by both
targets.

> **⚠ THE QUALIFIER IS LOAD-BEARING, and this blockquote did not carry it until
> 2026-08-04.** It read *"`/O1` is `/Ox` with exactly one change"* flat, and as a
> general claim about the two modes that is **REFUTED**. See §3.0 below, board
> **#258**. `OptMode`'s own doc comment carried the same un-qualified sentence
> and was corrected in place by lane `w-conv` (board **#272**, `65ee0c5`); this
> document is the other half of that correction and was made by lane `w-book4`.
> **The two documents were out of step with each other for a day and with the
> bytes for longer** — the shape #194's audit exists for.

### 3.2 Strength reduction is `/Ox`-only

```
a * 9   /Ox   rlwinm r11,r3,3,… ; add r3,r3,r11     (a<<3) + a
        /O1   mulli  r3,r3,9                        one instruction
```

Favor-size keeps the multiply. This is the same class of behaviour as the `a + a`
→ `slwi` folding already documented as c2's (not c1xx's) — but it is conditional
on the mode, which the existing notes do not say.

### 3.3 The section-layout difference is `/Gy`, not the mode

An earlier version of this section claimed "`/Ox` pads between functions, `/O1`
does not", from the `00000000` fillers visible in `il_reassoc`'s `/Ox` `.text`.
The mechanism is different and worth stating correctly, because it is a *second*
axis that the mode comparison was silently mixing in:

**`/O1` and `/O2` imply `/Gy`; `/Ox` does not.** So `il_reassoc` at `/Ox` is one
packed 224-byte `.text` whose functions are 8-byte aligned, and at `/O1` it is
**16 separate COMDAT `.text` sections** with no padding to do. The 224 → 176 drop
is the fillers disappearing because the functions no longer share a section — not
a codegen decision at all. Every `.text`-size comparison in the §3 table is
contaminated by this, which is why the per-function tables in §4.2 and the rule in
§3.1 were derived by extracting each function's bytes via its symbol rather than
by concatenating sections.

Controlling for it changes the picture in the port's favour. Comparing `/O2`
against `/O1` — same layout, differing only in the mode:

* `/Ox` and `/O2` are **byte-identical** on every function of eight fixtures, once
  the tail branch's REL24 displacement (which is section layout) is masked. Two
  modes sharing the optimization word emit the same code, so keying the codegen on
  the word is sound.
* `/O2` vs `/O1` differs **only in register fields**, never an opcode, and only in
  the chain class — trees, calls, reassociation and float are identical.
  **Over this corpus, every body of which has ONE BLOCK.** §3.0 is the
  counterexample and it is a different opcode set *and* a different block count;
  read this bullet as a statement about the eight fixtures, not about the modes.

The practical consequence: `/Gy` is an *argv* fact the IL does not record, so the
port has to be told (`PortC2::with_function_level_linking`), while the mode is an
*`.ex`* fact it can read. Two different channels for two different axes, and
conflating them is what hid three COMDAT mis-emits until `scripts/mode_lane.sh`
compiled the fixtures with `/Gy` for the first time (§4.1).

## 4. What this means for the roadmap

The census and the gap scan run on the real workload, whose flags are
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I…` — **`/O1`**. Every fixture is
captured with the default `/Ox /GS- /c`. So the two halves of the project have
been measuring different targets:

* the **numerator** (fixtures, sweeps, the byte-exactness claim) is `/Ox`;
* the **denominator** (110,277 in-class functions of 2,462,571; the blocker
  histogram; the 878-TU scan) is `/O1`.

An in-class function counted by the census is not a function the port can emit —
it is a function the port can *decode*. If it were emitted today it would be
emitted with `/Ox` register allocation against an `/O1` reference.

This has been invisible because no non-trivial real TU ever reached codegen. Of
the six TUs the last scan reported as `match`, **five have `fn_total = 0`** — they
are empty modules, the four-section shell with no `.text`, where the mode cannot
matter. The sixth (`src/system/utl/Spew.cpp`, two functions) is a real match, and
its two bodies are shapes where the modes agree, like `w5_tree2` above. So the
`match` column has never yet exercised mode-dependent codegen.

`codegen-gap` being 0 while everything else is `vocab-gap` is the same fact from
the other side: the decode gate refuses first, so the codegen has not yet been
asked a question it would answer wrongly.

### 4.0 The hazard, reproduced

Not an inference. One TU, entirely inside the port's accepted class, run through
the real harness twice with only the `/O` flag changed:

```sh
echo 'int chain4(int a,int b,int c,int d){return a*b*c*d;}' > modeproof.cpp
echo '/nologo /c /Ox' > oxflags.txt
echo '/nologo /c /O1' > o1flags.txt
c2rs gap --list list.txt --flags-file oxflags.txt --jobs 1   #  match      1  100.0%
c2rs gap --list list.txt --flags-file o1flags.txt --jobs 1   #  mismatch   1  100.0%
```

Both report `FUNCTION CENSUS (P2b): 1/1 functions in class (100.00%)`. The census
cannot see the difference; the byte compare can. `/Ox` is byte-exact and `/O1` is
a wrong-bytes emit, from the same source through the same port.

So the "one decode widening away" risk is not hypothetical — it is the current
state for any real `/O1` TU that comes fully in class. Exactly one does today
(`Spew.cpp`), and it matches because its two bodies happen to agree across modes.
The next one will not necessarily be so lucky, and nothing in the pipeline would
flag it as a mode problem rather than a codegen bug.

### 4.1 Order of work

1. **Gate on the word — LANDED** (`187a897`). `IlBundle::opt_words` exposes the
   word per segment and `PortC2::build` refuses anything but `00a00005`, naming
   the mode it found. Fails closed, so it covers the mode variations not
   enumerated here as well as the ones that are.

   Enforced in `c2-core` rather than in the IL parser, which is the one place this
   project deliberately departs from "gates live in the parser". The word is a
   *codegen-target* property: gating it in `functions()` would report a
   perfectly-decoded `/O1` TU as `vocab-gap`, blaming the IL model for something
   it read correctly, and gating it in the census would replace every real
   function's actual blocking feature with this one and destroy the histogram
   that ranks the roadmap. As a `codegen-gap` with a named reason it is both
   honest and useful — and it is the project's **first non-zero `codegen-gap`**,
   because until now the decode gate always refused first.

   Cost, stated plainly: the workload goes from **6 matching TUs to 5**.
   `src/system/utl/Spew.cpp` matched at `/O1` because its two bodies happen to
   agree across modes, and the port cannot know that. Trading a match that was
   correct by luck for the closure of a whole wrong-bytes class is what
   `GAPS.md` §6 already commits to. The other five are empty modules and are
   unaffected.
2. **Re-target to `/O1` — LANDED** (`2a19090`). `codegen::OptMode` carries the
   mode, read from the word; the chain allocator reuses r11 for a dead predecessor
   under `/O1`. Byte-exact on **24 of 88 fixtures** at `/O1`, 0 mismatches.

   Depth-2 trees needed no change — all 27 agree register-for-register across the
   modes. The comparison spines are reallocated at `/O1` and refuse, since that
   rule is not enumerated (below).

   `scripts/mode_lane.sh <mode>` grades every fixture at ONE chosen mode via
   `c2rs gap` (which takes `--flags-file`, where `c2rs diff` does not). It is a
   lane, not *the* lane — **run `scripts/gate.sh`**, which runs every lane in
   `scripts/lanes.txt` (12 of them since 2026-07-31, `/EHsc` crossed over all
   six code-shape configurations) and reports one result per lane. The table
   below is the four modes somebody typed, and none of them compiles `/EH` on a
   workload whose every TU is built `/EHsc`; see `docs/GAPS.md` §7. State when
   this was written:

   | mode | match | mismatch | codegen-gap |
   |---|---|---|---|
   | `/Ox` | 31 | 0 | 0 |
   | `/O2` | 26 | 0 | 5 |
   | `/Ox /Gy` | 26 | 0 | 5 |
   | `/O1` | 24 | 0 | 7 |
   | `/Od` | 1 | 0 | 30 |

   Running that lane immediately found **three pre-existing wrong-bytes emits**,
   none of them about the mode: the COMDAT emitter duplicated callee symbols and
   batched relocations away from their own sections (both bugs already fixed once,
   in the *packed* emitter), and the framed-call shortcut bypassed the `/Gy` branch
   so its refusal was unreachable. They reproduce at `/Ox /Gy`. The reason nothing
   had caught them: `/O1` implies `/Gy` and every fixture compiled `/Ox`, which
   does not — so the COMDAT emitter had never been run on a fixture that calls,
   floats or frames. A second lane over the same corpus was worth three bugs.

3. **Still open at `/O1`.** The comparison spines (`w6_rel_k`, 14 of 19 leaves
   reallocated), float under `/Gy` (`_fltused` position in the per-function COMDAT
   symbol order), a pooled `.rdata` constant under `/Gy`, the framed call under
   `/Gy`, and the one scheduling difference in `w5_tree_neg`'s `n_spill`. All
   refuse; none mis-emits.
4. **Re-run the sweeps under `/O1`.**
 `scripts/expr_sweep.sh` compiles with the
   default `/Ox`, so its 2589 green cases say nothing about `/O1`. The sweep
   needs a mode parameter, and the `/O1` lane should be the one that gates.

### 4.2 Measured scope of the re-target

Better news than §4 sounds. Every function in eleven fixtures spanning the whole
accepted class was compiled both ways and each difference classified as
*allocation-only* (same opcode sequence, different register fields) or
*selection* (different opcodes):

| fixture | identical | allocation-only | selection |
|---|---|---|---|
| `w5_chain` | 1 | 3 | 0 |
| `w5_tree3` | 0 | 4 | 0 |
| `il_accum4` | 7 | 2 | 0 |
| `il_reassoc` | 16 | 0 | 0 |
| `w6_rel_k` | 5 | 14 | 0 |
| `il_call_perm` | 1 | 6 | 0 |
| `il_deep_chain` | 0 | 3 | 0 |
| `w13b_fconst` | 1 | 0 | 0 |
| `w13b_ffold` | 5 | 0 | 0 |
| `w6_k_boundary` | 0 | 2 | 0 |
| `w5_tree_neg` | 4 | 6 | 1 |

So of roughly ninety functions, **one** differs by more than register assignment.

> **Read that with its population, 2026-08-04.** The eleven fixtures span the
> port's accepted class **as it stood**, and that class was straight-line: not one
> of these ninety functions has a second block. The count is *"one of ninety
> single-block functions"*, and §3.0 shows a two-block body differing in opcode
> set and block count. **`1/90` is not a base rate for "how often the modes
> differ" — it is a base rate inside the one shape where they mostly do not.**

Three consequences worth stating plainly:

* **The reassociation and canonicalization work is mode-independent.** All 16 of
  `il_reassoc` are byte-identical across modes, as are both float fixtures. The
  expensive semantic work — commutative canonicalization by register, additive
  reassociation, FP constant folding, the relational spines' *shape* — carries
  over untouched. What does not carry over is the allocator.
* **The one selection difference is scheduling, not selection.** `w5_tree_neg`'s
  `n_spill` has the same 18 instructions in both modes, reordered: `/Ox` computes
  `7d642a14` before `7d433214`, `/O1` the reverse. That is the framed/spill shape,
  so it needs its own characterization, but it is one shape and not a class.
* **`a * 9` is not in any fixture.** The strength-reduction difference (§3.2) does
  not appear in this table because nothing in the corpus multiplies by a constant
  — which is itself worth noting, since `mulli`-versus-shift is exactly the kind of
  size/speed tradeoff `/O1` should be full of. The corpus does not sample it.

## 5. How this was found

Not by a fixture, and not by the census. `#pragma optimize("", off)` was one entry
in a batch of probes aimed at the *obj shell* — sections and symbols, the axis the
fixture corpus had almost no coverage of. It came back as a mismatch at obj offset
8, which looked like the `.drectve` class (`il_drectve_pragma.cpp`); it was not.
Diffing that TU's IL against the same source without the pragma isolated four
changed bytes, and two of them were the word documented above.

The generalizable part: the port skips bytes it does not model, and a skipped byte
is indistinguishable from a byte that is always the same. Every fixed-width field
the port steps over is a candidate for this — the same shape as the source-line
marker that turned out to carry a varint payload, and the aggregate TYPE that
`read_type` still mis-reads (`IL_TYPE_TAGS.md` §1).

## 6. The `0x4` bit: `fp_contract`, and `00200001` — RESOLVED 2026-07-30

§2.1 left the low nibble unexplained ("the low nibble is `5` everywhere except
`#pragma optimize("", off)`, which gives `4`. Not explained."). The census then
found a real-workload bucket the port refuses on this ground alone:
**`opt-mode-00200001`, 202 functions and 136 of the calls-0 population**, all in
`HamRibbon.cpp` and `Ribbon.cpp`.

### 6.1 The encoding is a varint, not a fixed `80 <LE32>`

§1 records the form as `4F 1F 80 <LE32 word>`. That is the *long* branch of the
IL varint (`CODEGEN_PPC_MVP.md` W3): a word below `0x80` is a single byte.

```
  #pragma optimize("", off) at /O1 -> 4f 1f 04 4f 20 80 fe 00 …
                                            ^^ the whole word, = 0x00000004
```

Anything scanning for the literal `4F 1F 80` misses these functions entirely.

### 6.2 Each bit, measured one at a time

`int f(int a){return a+1;}`, varying exactly one thing:

| variation | word |
|---|---|
| `/O1` | `00200005` |
| `/Ox` | `00a00005` |
| `/Od` | `00800005` |
| `/O1 /Og-` | `00000005` |
| `/O1` + `#pragma optimize("g",off)` | `00000005` |
| `/O1` + `#pragma optimize("",off)` | `00000004` |
| `/O1` + `#pragma optimize("s",off)` | `00a00005` |
| `/O1` + **`#pragma fp_contract(off)`** | **`00200001`** |
| `/Ox` + `#pragma fp_contract(off)` | `00a00001` |
| `/O1 /Ob0`, `/O1 /Oy-`, `/O1 /EHsc` | `00200005` (unmoved) |
| `/O1` + `#pragma inline_depth(0)`, `auto_inline(off)`, `function(memcpy)` | `00200005` (unmoved) |
| `/O1` + `optimize("t"/"y"/"a"/"w"/"p", off)` | `00200005` (unmoved) |

So:

* `0x00200000` = **global optimizations on** (`/Og`; cleared by `/Og-` and by
  `optimize("g",off)`) — which sharpens §2.1's "optimizations enabled".
* `0x00800000` = favour speed (unchanged reading).
* `0x00000004` = **floating-point contraction enabled** (`#pragma fp_contract`).
* `0x00000001` = still unexplained; only `optimize("",off)` clears it.

The pragma is **per function**, not per TU — a mid-file
`#pragma fp_contract(off)` gives `00200005 00200001` for the two functions in
source order, which is the refutation of "it is a TU-level flag that the word
happens to carry".

### 6.3 `00200001` is `/O1` and existing `/O1` codegen already matches it

The pragma sits at `src/system/hamobj/HamRibbon.cpp:139` and
`src/system/rndobj/Ribbon.cpp:271` in the dc3 tree — two lines that account for
the whole 202-function bucket.

Its only effect on emitted bytes is that a `*` feeding a `+`/`-` stops fusing:

```
float f(float a,float b,float c){ return a*b+c; }
  contract on   ec2118ba 4e800020                    fmadds f1,f1,f2,f3 ; blr
  contract off  ec0100b2 ec20182a 4e800020           fmuls  f0,f1,f2 ; fadds f1,f0,f3 ; blr
```

Compiled against the **whole fixture corpus** at `/O1`, with and without the
pragma prepended, comparing the concatenated `.text` raw bytes of every COMDAT:

```
identical .text: 129   differing: 1
differing fixture: w13_fneg
```

`w13_fneg` is the fixture whose entire purpose is FMA contraction
(`CODEGEN_W13_FLOAT.md` N1) — its `n_fma`, `n_fms`, `n_fnms`, `n_fma2`,
`n_dfma2`, `n_fma_tree`, `n_rank` and `n_spill` all change; every non-contracting
function in it (`n_k_add`, `n_self_add`, `n_div_k`, `n_i2f`, `n_f2i`,
`n_plus_zero`, …) is byte-identical.

> **`00200001` should be accepted as `/O1`.** Every class the port emits today
> is byte-identical under it, because `codegen.rs`'s contraction guard already
> refuses "an FP expression mixing `*` with `+`/`-`" as out of class — the exact
> and only set of bodies the bit changes. Accepting the word cannot turn a
> refusal into a wrong byte for any currently-emitted class; it can only turn a
> refusal into a match.
>
> The one thing an implementation must **not** do is treat `0x4` as ignorable
> when the FP-contraction rung is eventually built: with the bit clear the
> correct lowering for `a*b+c` is `fmuls`+`fadds`, and a contracting emitter
> would produce a valid, wrong, `fuzzy`-invisible `fmadds`.

### 6.4 TAKEN, 2026-07-30 — and the `/Ox` half measured too

`opt_word_mode` accepts `00200001` as `/O1` (`OPT_WORD_O1_NO_FP_CONTRACT`) and
`00a00001` as `/Ox` (`OPT_WORD_OX_NO_FP_CONTRACT`). The second was **not**
inherited from §6.3's argument — it got its own run of the same corpus-scale
experiment, at `/Ox`: **145 byte-identical `.text`, 1 differing**, and the one is
`w13_fneg` again, the FMA fixture, which is refused. Same shape, own measurement,
because "the bit does the same thing at the other mode" is exactly the kind of
claim this document exists to stop being assumed.

It is worth 0 functions on the workload (which compiles `/O1`) and it is why the
fixture **grades in every lane**: `c2rs bench` and `c2rs diff` use the `/Ox`
profile, and a positive fixture that reports `NotImplemented` in the default lane
is the decoration `GAPS.md` §6 records `w13_fabi.cpp` as having been for months.

**Census 482,542 → 482,748, +206**, mismatch 0, disagreement 0, and the two
`opt-mode` keys disappear entirely (**570 → 568 keys**) — the whole bucket, since
this gate is applied last and only to otherwise-in-class functions, so everything
under it was already complete. 188 `calls-0` and 18 `calls-1`. Estimate **+206,
exact**: the counterfactual here is the census key itself.

`fixtures/cpp/w29_fp_contract.cpp` is **16/16 in class, `Port=Match`**, and it
carries the pragma over the integer, pointer, store, compare and floating-point
classes. It deliberately does **not** carry a body that would contract — that
case is the one thing the bit changes, it lives in `w13_fneg.cpp`, and it is
refused.

### 6.5 The word is a varint, and the reader was not (roadmap #52)

§6.1 recorded the encoding correctly and `opt_word_at` was never updated: it
required `seg[2] == 0x80` and read four little-endian bytes, so a short-form word
returned `None`.

**Audited, and the damage is naming rather than bytes.** `opt_word_mode(None)`
refuses, so the reader was fail-closed the whole time — but the census key it
produced was `opt-mode-00000000`, which asserts the word is *zero* when it is in
fact unread, and a wrong name is the one thing this instrument cannot survive.
Fixed to read both branches, with `81..FF` refused rather than sign-extended the
way an operand-stream varint would be (an optimization word is a bit field, not a
number). Worth **0 functions** on the 878-TU workload: no otherwise-in-class
function there takes the short branch.

One test moved with it, and the way it moved is the finding in miniature:
`opt_words_reports_an_unreadable_prefix_rather_than_guessing` used `4F 1F 11 …`
as its "unreadable" case, on the reading that anything but `80` was unreadable.
`11` is the perfectly readable short-form word 17. The test now uses `F1`, and
asserts the short form is *read* rather than merely tolerated.
