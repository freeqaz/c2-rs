# ROADMAP — from the MVP port to a fully implemented stack

Status: living document (written 2026-07). Describes where the port stands,
what is missing, and the ordered plan to close it. The invariants at the
bottom are **load-bearing** — every phase must preserve them.

## 1. Where we are

The foundation is proven and fast; the port itself is deliberately narrow.

> **Mode caveat — read before quoting any number below (2026-07-30).** Every
> fixture, every sweep case and therefore every byte-exactness claim in this
> document is captured at the default **`/Ox`**. The 878-TU dc3 workload — the
> source of every *coverage* number — compiles **`/O1`**. The two modes emit
> different code, and `.ex` says so in a per-function word after each `4F 1F`
> that the port does not read (`docs/OPT_MODE.md`). So the numerator is `/Ox` and
> the denominator is `/O1`, and they are not two views of one thing.
>
> The measured scope of the difference is narrower than that sounds: over ~90
> functions spanning the whole accepted class, every difference is
> allocation-only bar one scheduling case, and the reassociation and float work
> is byte-identical across modes. But `il_accum4.cpp`'s whole-chain accumulator
> rule is `/Ox`-only, and re-targeting `/O1` became its own phase, not
> a footnote. Five of the six TUs the last scan called `match` have
> `fn_total = 0` — empty modules — so the `match` column has never yet exercised
> mode-dependent codegen.
>
> **Resolution (measured 2026-07-30, HEAD `2724ca5`).** The caveat above is now
> historical: the port *reads* the per-function optimization word (187a897 —
> anything but the two known words refuses), `/O1` is a supported target
> (2a19090, abe0512 — the `/O1` allocator rule and comparison spines, plus the
> mis-emits the re-target exposed and fixed), and `scripts/mode_lane.sh` grades
> the whole fixture corpus per mode. Measured on the 90-fixture corpus
> (`bash scripts/mode_lane.sh <mode>`): **`/Ox` 32 match, `/O1` 28, `/O2` 28,
> `/Ox /Gy` 28 — 0 mismatch in every lane**. The numerator and denominator now
> speak the same modes; the residual gap between 32 and 28 is honest
> `codegen-gap` (shapes verified at `/Ox` only, refused elsewhere).

> **As-of marker.** The fixture ratio (21/41) and every census figure in §1 and
> §G5 were measured at commit **`cebfb88`** (W13b). Roughly forty further rungs
> landed in concurrent sessions on 2026-07-29/30 (the statement layer, chain
> canonicalization, multi-arg tail calls, indirect-load leaves, `/O1` support,
> the expression-layer decode, …) and are **not** reflected in those numbers.
>
> **Re-measured 2026-07-30 at HEAD `2724ca5` (independent review, §6b):**
> fixture gate **32 match / 0 mismatch / 59 refuse** over 91 fixtures
> (`c2rs diff` per file, default `/Ox`); mode lanes `/Ox` 32, `/O1` 28, `/O2`
> 28, `/Ox /Gy` 28 of 90, 0 mismatch each (`scripts/mode_lane.sh`); real
> workload **match 6 / mismatch 0 / codegen-gap 0 / vocab-gap 865 /
> capture-fail 7** of 878 TUs, **function census 109,501 / 2,462,571 (4.45%)**
> (`c2rs gap`, 41.4 s at `--jobs 16`); generated sweep **2,589 cases, 0
> mismatch** (`scripts/expr_sweep.sh`); `cargo test --workspace` green.
> Numbers in the body below that disagree with these are older snapshots —
> the progression tables in §G5 carry the deltas.

**Proven (differential, against real `c2.dll`/`c1xx.dll` 16.00.11886.00 under wibo):**

- **P0.1** — standalone-c2 IL-bundle replay is byte-exact on all fixtures
  (`c2rs replay`) **and on all 871 capturable TUs of the real dc3-decomp
  workload** (`c2rs gap --replay-every 1`, 2026-07-20 baseline, **re-proven at
  full strength on 2026-07-29** against the post-token-fix code: 871/871,
  0 diverged — after two real-workload fixes: a missing `lstrcpynA` shim in
  wibo that killed c2 on large TUs, and the `1033/clui.dll` resources symlink
  beside `c2host`, without which any TU that triggers a diagnostic dies with
  C1510). The reference side of the differential is real, on real code.
- **P-F0.1** — standalone-c1 (front-end) replay is byte-exact on all 25
  fixtures (`c2rs replay-c1`). The same porting path is open for `c1xx.dll`.
- **The port (`c2-core::PortC2`) is byte-exact on its accepted class**
  (`c2rs diff`, **21/41 fixtures Match, 0 mismatch** at cebfb88): straight-line
  integer add/sub/mul chains with immediate folding and wide constants (now
  including 3+-op `*`/`-` chains, see the mis-emit note below), **depth-2
  expression trees** (W5 trees), multi-function TUs of those, bare void tail
  calls, integer tail calls `return g(<arg>)`
  (passthrough / `+0` fold / arg-setup), the framed non-leaf
  `return g(a) + k` (6-section obj with `.pdata`), **the empty TU** (R1),
  **empty function bodies** (R2), **the COMDAT `.text`-per-function shape under
  `/Gy`** (R3), **comparison→boolean leaves** (W6: `return a <rel> k`,
  branchless), **float/double leaves over parameters** (W13a) and **one pooled
  floating-point constant per body** (W13b). Everything
  else returns `NotImplemented` — fail closed, never a guess.
- **R1 — the first nonzero match bucket.** `coff::emit_empty_obj` emits the
  720-byte four-section obj for a TU that defines no functions, recognized
  *positively* (`is_empty_module`: `.ex` carries neither a `LO` body marker
  nor a `4F 1F` function start). Gap scan: **match 0 → 5 of 878 TUs**. Two of
  the seven zero-function TUs are deliberately refused — they carry a stray
  `4F 1F` after the module end — a conservative miss, taken in preference to
  relaxing a fail-closed test.
- **R2 — empty function bodies.** `w10_empty_fn.cpp` is exact; the function
  analogue of R1, with no expression to select. The largest single census jump
  so far, from the smallest class: **7,954 → 78,028 functions in class
  (0.32% → 3.17%)**, because trivial accessors and destructors are everywhere.
  The `IL_CALL_GRAMMAR.md` §4.2 trailing-expression variant still rejects.
- **R3 — COMDAT `.text` per function under `/Gy`.** Forced by R2, which turned
  a latent flag dependency into a live mismatch: `/O1` and `/O2` imply `/Gy`,
  the bundle does not record it, and the same IL therefore legitimately yields
  two different objs. `coff::emit_comdat_obj` plus
  `PortC2::with_function_level_linking` carry it. `system/utl/Spew.cpp` became
  the **first function-bearing real TU to match**, taking the gap-scan bucket to
  **6/878** with mismatch back at 0. Full account: the `/Gy` box in §G5.
- **W6 — comparison → boolean, byte-exact.** `il_bool_materialization.cpp` is
  `Port=Match`, 6/6 functions in class. c2 lowers these **branchlessly** (no
  `cmpw`/`cmplw` at all) via carry-bit and bit-extraction idioms; the `k == 0`
  folds are mandatory and dispatched first. Full byte evidence, CONST/DERIVED
  split and fail-closed negatives: **`docs/CODEGEN_W6_COMPARE.md`**. `<`, `<=`
  and `>=` against a **non-zero** literal stay out of class — the spine's
  instruction order for a literal lhs is unresolved and guessing it would be a
  silent wrong-bytes emit.
- **W13a — float/double leaves over parameters, byte-exact.** `mvp_fmul3.cpp`
  is `Port=Match`. Spec and byte evidence: **`docs/CODEGEN_W13_FLOAT.md`**.
  The FP register model **shares nothing with the integer one**, and every
  difference is a place a grafted integer path would emit wrong bytes rather
  than run out of range: the pool is `[f0, f13, …, f1]` with `f0` allocatable
  and *first* and the result register `f1` *last* (`select_text`'s "refuse
  below `r9`" guard has no FP analogue); an FP `+` chain does **not** collapse
  into a single accumulator the way the integer one does; `fsubs fD,fA,fB` is
  `fA − fB`, the **opposite** of `encode_subf`'s load-bearing reversal; and
  `fmuls` takes the multiplier in the **C** field. Single precision is primary
  opcode 59 and double 63 with identical XO and register fields, so one encoder
  covers both. Gated hard against everything that mis-emits rather than
  overflows: FP **literals** (W13b — an FP constant costs an `.rdata` COMDAT
  plus a REFHI/REFLO relocation pair plus a GPR), `2C` converts, float/double
  mixing, any `*` under a `+`/`-` (c2's contraction to `fmadds`/`fmsubs` is
  **mandatory**), and repeated leaves (`a+a` is rewritten to `a*2.0f`, which is
  a constant again). Obj shell effect for this class: exactly one extra symbol,
  the undefined external `_fltused` — the *general* trigger rule for that symbol
  is still open (`CODEGEN_W13_FLOAT.md` §7).
- **W13b — one pooled floating-point constant per body, byte-exact**
  (cebfb88). `w13b_fconst.cpp` and `w13b_fdedup.cpp` are `Port=Match`. A float
  has no immediate form on PPC, so c2 gives each distinct value its own `.rdata`
  COMDAT (4 B/`0x40301040` for float, 8 B/`0x40401040` for double, `Selection=2`,
  big-endian contents) and loads it through `addis`+`lfs`/`lfd` with a
  REFHI+PAIR / REFLO+PAIR relocation quad; the pool is keyed on
  **(bit pattern, width)** TU-wide, so a `float` 1.0 and a `double` 1.0 are two
  sections and two `__real@…` symbols. Three things the captures corrected, all
  of them cases where the wrong rule matched the entire prior corpus:
  **(1)** a section's relocations sit after **that section's own** raw data, not
  after every section's — invisible while `.text` was the last section, and an
  emitter-wide fact rather than an FP one; **(2)** a constant claims its FP
  register *before* any interior temporary does, so the allocator cannot walk the
  emitted instruction list in order (witness `ke`: `fmuls f13,f1,f2` with the
  constant in `f0`); **(3)** the IL literal's trailer is one little-endian `u16`
  width, not a size byte plus an unexplained `00`.
  **Gated at one constant per body, and the reason is the interesting part:
  c2 — not c1xx — is the floating-point constant evaluator.** The IL still
  carries every literal the source wrote, so the backend folds
  (`a+0.0f`, `a*1.0f`, `a-0.0f` → bare `blr`, nothing pooled — but `a*0.0f` is
  **not** folded, so the gate is per `(operator, value)` pair, not per value),
  strength-reduces (`a/2.0f` → `fmuls` by `__real@3f000000`; `a/3.0f/7.0f` → one
  `fmuls` by 1/21, which is inexact and therefore a real numeric transform) and
  reassociates (`a*2.0f*b*3.0f` → `(a*b)*6.0f`). With two *surviving* constants
  the schedule also changes — every `addis` hoists into a prologue group and each
  `lfs` is placed at first use, so the REFLO site stops being `hi_off + 4` — and
  that is characterized by exactly two captures (`p1`, `p5`), which is not enough
  to implement from. The gates live in the **IL parser**
  (`c2-il::func::try_parse_float_leaf`), not in codegen, so the census and the
  emission gate cannot disagree about what is in class. Full byte evidence:
  **`docs/CODEGEN_W13_FLOAT.md` §5**.

- **W5 chains and depth-2 trees** — `*`/`-` chains past two operations,
  allocating temporaries down the `r11 → r10 → r9 …` cursor and **refusing below
  `r9`**; the rule and its eleven negative neighbours are in
  **`docs/CODEGEN_W5_SCRATCH.md`**. This is the change that fixed the mis-emit
  below. **Depth-2 trees have since landed too** (9b7df37): `w5_tree2.cpp` is
  `Port=Match` on all four shapes — left child into one scratch, right into
  another, root into `r3`. One characterized-but-unexplained wrinkle gates the
  depth: when the root is `+` the two children's registers are **swapped**
  relative to every other root operator, reproducibly and order-independently
  (`(a*b)+(c*d)` and `(c*d)+(a*b)` are byte-identical), so the `+` root is
  accepted at exactly this depth and refused above it. Depth-3 trees
  (`w5_tree3.cpp`) and all eleven functions of `w5_tree_neg.cpp` still fail
  closed.
- **Harness + tooling**: oracle self-test, corpus generator, obj→IL retrieval
  baseline, IL-space search prototype, edit gate; `perf`/`perf-scale`
  (~200–290× per obj, ~897k obj/s at 32 threads vs ~3.1k for real c2).
- **P2 / P2b measurement** — `c2rs gap` (real-workload TU scan) plus the
  function-level census (`c2rs census <cpp>`, and the scan-wide coverage +
  blocking-feature histogram in the `gap` report; the census also prints a
  bracketed hexdump of the bytes at each blocking site, which is what turned
  guessed opcode names into measured ones). The port's real coverage is now a
  *measured* number: **79,719 / 2,462,571 functions in class (3.24%)** over the
  871 capturable dc3 TUs, with the blocking-feature histogram that ranks the
  remaining work (§G5).
- **IL codec (K1/K2a)** — round-trip-gated typed-islands-over-opaque-spans
  model of `.ex`/`.gl`; float-leaf (`Box::Volume` shape) token vocabulary is
  typed; K3a length-consistent edits verified *as edits* against real c2.

### The first real mis-emit the differential caught (2026-07-29, 40749e7)

It was a **correctness bug, not a refusal**: `w5_chain.cpp` reported
`Port=Mismatch` — the port emitted an obj that *differs* from c2's. Root
cause: the port used one scratch (`r11`) for every intermediate, but c2 only
does that for **additive** chains —

```
a+b+c+d  ->  add   r11,r3,r4 ; add   r11,r11,r5 ; add   r3,r11,r6
a*b*c*d  ->  mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
a-b-c-d  ->  subf  r11,r4,r3 ; subf  r10,r5,r11 ; subf  r3,r6,r10
```

— an additive chain collapses into a running accumulator, while a `*`/`-`
chain gives every intermediate its own register descending from `r11`.

**The two rules coincide at exactly one intermediate**, which is why every
fixture up to `a-b-c` matched: the MVP corpus contained no 3-op `*`/`-` chain
and therefore *could not distinguish the two rules at all*. Byte-exactness on
it was consistent with both, and so was never evidence for the one the port
had implemented.

Fixed by keeping plan operands symbolic until emission, so `Base::Prev`
resolves to the previous entry's actual destination; allocation refuses below
`r9`, since the deepest characterized chain is `a*b*c*d*e` and outside that
class c2's allocator is demonstrably richer — it recycles dead registers and
it schedules, so numbering order is not emission order
(`docs/CODEGEN_W5_SCRATCH.md` §2, §6).

The lesson is a coverage lesson, and it is the one the negative-fixture
discipline exists to teach: **a corpus earns its keep by separating candidate
rules, not by being green.** A characterization fixture written to probe the
*neighbouring* class is what produced the discriminator — the differential
working exactly as designed, one rung earlier than intended.

**Staged but not yet ported** (fixtures/probes already in-repo, pointing at
the next classes):

- `w5_tree3.cpp` / `w5_tree_neg.cpp` — multi-scratch expression **trees** past
  depth 2. `w5_tree2.cpp` has since been ported (§1); depth 3 and the eleven
  negative neighbours still fail closed. The register-allocation rule, the
  evaluation order and those neighbours are fully characterized in
  `docs/CODEGEN_W5_SCRATCH.md`.
- `select_max`, `shift_mask` in `add3.cpp` — ternary select, shifts, `&`.
- `w13_fabi.cpp` / `w13_fops.cpp` / `w13_fscratch.cpp` / `w13_fneg.cpp` — the
  W13 characterization set: the FP calling convention, the four binary ops, the
  temporary cursor with its liveness skip and wrap, and the negatives (spills,
  int↔FP round trips, fused `fmadds`). All replay `ByteExact`; all still
  `Port=NotImplemented` **as whole TUs**, which is what pins the W13a/W13b
  boundary. Inside `w13_fneg.cpp`, N3's `n_k_add`/`n_k_dadd` are no longer
  negatives — W13b made both shapes byte-exact — while `n_k_ret` (a constant
  return with no operand) and `n_k_two` (two surviving constants) still refuse.
- `w13b_fpool.cpp` / `w13b_ffold.cpp` — the W13b negatives: bodies whose IL
  carries 2+ FP literals, and the identity folds. Both must keep refusing, and
  `w13b_ffold::q5` (`a * 0.0f`, which c2 does **not** fold) must keep *emitting*
  in the reference, since it is the only thing separating a per-`(operator,
  value)` gate from the wrong per-value one.
- `il_convert_scalar.cpp` / `il_intrinsic_call.cpp` — the P2d cast /
  intrinsic-call characterization set (19 scalar conversions; 12 `0x40` sites).
  Both replay `ByteExact` and both must keep refusing: the same `2C` token is
  simultaneously nothing, `extsb`, `extsh`, `clrlwi` and a 3-instruction
  `fctiwz` sequence depending on the **source** type, which the operand stack
  does not yet carry (`docs/IL_CAST_CONVERT.md` §4.2, §5).
- `il_call_return.cpp` — the call frontier: multi-arg calls, multiple calls,
  virtual calls, conditionals, locals, early returns.
- Out-of-class neighbor fixtures (`mvp_call_submod`, `…_twice`,
  `…_then_stmt`, …) — the fail-closed boundary is pinned by negative tests.

## 2. What "fully implemented" means here

The end state is **source→obj fully in-process, byte-exact, at port speed**:

```
PortC2(PortC1(foo.cpp)) == cl.exe /Ox /GS- /c foo.cpp     (timestamp zeroed)
```

for a stated, measured corpus — with no wibo, no process spawn, no `_CL_*`
files on disk. Concretely that means:

1. **Backend coverage**: the port accepts the function classes that actually
   occur in the target corpus (see G1), measured by the function census
   (P2b — landed; the metric is *functions in class*, §G5) —
   not "all of C++", which is unbounded (exceptions, RTTI, SEH, inline asm,
   template bloat). The coverage metric is the contract: a green run claims
   nothing outside it.
2. **Front-end coverage**: `c1-core` emits the 5-file bundle for the same
   source classes (Track D), so the composition needs no Microsoft binary in
   the loop at all.
3. **Throughput preserved**: the perf headline survives every widening step —
   each new class lands with a `perf` check, not just a byte check.

## 3. Gap analysis

### G1 — Backend class coverage (the main body of work)

The port is a **positive shape-matcher**: `c2-il::func::parse_segment`
accepts exactly the shapes it knows, codegen lowers them. The missing
classes, in the *original* intended working order — the census and the CALL
grammar have since landed and re-rank this list; see the note after the table
and §G5:

| # | Class | New mechanisms required | Staged? |
|---|-------|-------------------------|---------|
| W5 | **Multi-scratch expressions** (tree depth > 2, e.g. `(a+b)*(c+d)`) | COLOR scratch order beyond r11; operand-stack depth limit lifted | **chains DONE** (`r11→r10→r9…`, refused below r9) and **depth-2 trees DONE** (`w5_tree2.cpp` Match; the `+`-root register swap is characterized, not explained, so depth 3 refuses) — `CODEGEN_W5_SCRATCH.md` |
| W6 | **Integer comparisons → bool** (`x!=0`, `x>7`, signed/unsigned) | branchless carry/bit-extraction spines (**no** `cmpwi`/`cmplwi`), `subfe`/`addze`/`cntlzw`/`rlwinm`, mandatory `k == 0` folds | **compare-against-literal leaves DONE**, 6/6 in `il_bool_materialization.cpp`; `<`/`<=`/`>=` vs a non-zero literal still refused — `CODEGEN_W6_COMPARE.md` |
| W7 | **Shifts + bitwise** (`<< >> & \| ^ ~`, mul-by-const strength reduction) | `slw`/`srw`/`sraw`/`rlwinm`/`andi.` (the dot!); non-commutative hazard list grows | `shift_mask` |
| W8 | **Control flow** (if/else, ternary select, loops) | multi-block `.ex` (labels, branch tokens), `bc`/`b`, block layout order, compare+branch fusion | `select_max`, `il_call_return` conditionals |
| W9 | **Division / modulo** | `divw`/`divwu`, remainder via `mullw`+`subf`; const divisor → multiply-high | — |
| W10 | **General frames + locals** (spills, local temps) | frame-size model beyond the fixed 96B, `lwz`/`stw` to frame, `.pdata` generalization; **must solve the `.pdata` label-counter shift (W-UNW-1)** | `cached_return` |
| W11 | **Calls generalized** (multi-arg, stack args, multiple calls/externals, multi-fn TUs with calls, calls in expressions) | arg registers r3–r10 + stack spill, call sequencing, `.pdata` per function (W-UNW-1 again), extern pairing beyond the single-external shortcut | `il_call_return.cpp` |
| W12 | **Memory / struct access** (pointers, member loads, stores, arrays) | `lwz`/`lhz`/`lbz`/`lha` + sign/zero extension, store forms, member-offset addressing — the `Box::Volume` float leaf lives here | float-leaf codec already typed |
| W13 | **Float codegen** (`fmul3` and friends) | pool `[f0, f13, …, f1]` (**not** the integer shape), `fmuls`/`fadds`/`fsubs`/`fdivs`, mandatory `fmadds` contraction, `frsp`; float *constants* need an `.rdata` COMDAT + `addis`/`lfs` + a REFHI/REFLO pair (splits into 13a param-only leaves, 13b constants) | **13a DONE** (`mvp_fmul3.cpp` Match) and **13b DONE at one constant per body** (`w13b_fconst`/`w13b_fdedup` Match; `w13b_fpool`/`w13b_ffold` pin the boundary) — `CODEGEN_W13_FLOAT.md`. Two-or-more constants stays open: c2 is the constant evaluator and also reschedules |
| W14 | **Data sections / globals** (`.data`/`.rdata`, string literals, statics, arrays) | new COFF sections, `ADDR32` relocs, symbol storage classes | — |

Long tail, census-driven only: switch/jump tables, 64-bit ints (`addc`/`adde`
carry chains), unsigned variants everywhere, virtual/indirect calls (needs
W12 + `mtctr`/`bctrl`), `__declspec`s. Do not schedule these ahead of measured
demand. **Intrinsics have left the long tail**: the `0x40` production is the
largest schedulable thing on the board (§G5), so it is scheduled work now, not
tail work.

**Re-rank from the P2b census, now that the CALL grammar and the `0x40`
production are both characterized (INFERENCE from the measured histogram of
§G5 — the histogram is a measurement, the attribution of buckets to rungs is
not).** The largest buckets are no longer anonymous: `docs/IL_CALL_GRAMMAR.md`
decodes the CALL token, the three coexisting variable-width encodings and the
body/statement grammar, and `docs/IL_CAST_CONVERT.md` decodes the *second*
call token. Grouping the current top eight by what the bytes actually are:

- **Genuinely out-of-class calls — these must keep failing closed, they are
  not a "fix"** (`call-token-0xB9`, **15.3%**, the single largest bucket).
  This is *not* a missing opcode: `BD` is a postfix operator applied to
  whatever the operand stream pushed, and here the callee is an **expression**
  — `b9 <tok> <TYPE>` (indirect call) or `26 <method> <obj-expr> 99 …`
  (member call) — not the direct `26 <tok>`. An indirect call has **no
  relocatable callee name anywhere**; a member call needs a `this` argument
  and possibly vtable dispatch. Per `IL_CALL_GRAMMAR.md` §6.2 both must be
  rejected *before* emission, permanently, until W11/W12 give the port real
  argument passing and member addressing. Widening the parser to accept them
  without codegen would convert a refusal into a mis-emit.
- **The intrinsic-call family — the largest schedulable thing on the board,
  ~13%.** `expr-intrinsic-call` (**7.0%**) and `call-token-0x33` (**6.1%**)
  are the *same* production, the second differing only in that the result is
  assigned. `0x40` is a second CALL token, not the cast the census used to call
  it (see the correction below and §G2). Scheduling it requires: decoding
  `40 <TYPE>` plus its argument loop and the `66 02 <tok> <tok>` class-pair
  descriptor merely to stay aligned; then, for acceptance, an **allow-list of
  intrinsic ids pinned by controlled fixture** whose *argument literals* are
  also constrained — because c2's expansion depends on the literal values, not
  just the id (`IL_CAST_CONVERT.md` §1.4: one offset byte apart is the
  difference between zero instructions and a null-guarded four-instruction
  sequence). Decoding is cheap and unlocks the census; accepting is not.
- **Remaining call-shaped** blocks: `expr-call-in-expr` (**5.0%**) and
  `call-token-0x26` (**3.4%**) = **8.4%** — real **W11** demand (calls nested
  in expressions, `26 dest 26 callee BD …` assign-a-call-result statements).
- **Type-driven blocks**, i.e. the port only lowers `int` and the FP leaf of
  W13a/W13b: `expr-load-type-864540` (**float**, 3.4%) and
  `expr-load-type-888541` (**double**, 3.2%) = **6.6%**, with
  `expr-load-type-864383` (**void\***, 2.0%) just behind them. W13a took a bite
  out of the float row and W13b took **one further function** out of it
  (81,478 → 81,477) — the measurement is in §G5 and it is the clearest evidence
  on the board that these buckets are not made of the shapes the fixtures
  sample. What remains is honest **W12** demand plus the FP shapes W13b
  deliberately refuses (2+ constants, contraction, converts) — knowing how to
  *skip* a `double` is still not knowing how to lower it, and now: knowing how to
  lower a `double` *leaf* is not knowing how to lower the corpus' doubles either.
- **Statement/body-shaped** blocks: `body-0x53` (**2.9%**) is a body whose
  first statement is an `if`/compound, i.e. **W8** control flow. `body-0x3A`
  (previously 4.4%, the cheapest large bucket) is **gone** — that was empty
  function bodies, landed as R2.
- **W6 (comparisons) and W7 (shifts) still do not appear in the top eight**,
  and W6's leaf class has now landed anyway.

So the measured demand still says the W5→W6→W7→W8 order is not
demand-driven, but the shape of the demand has changed again: a large part of
the top is **out-of-class by construction** (member/indirect calls) rather than
schedulable work, and the schedulable head is now the **intrinsic-call family
(~13%)**, then generalized calls (8.4%), then float/double types (6.6%), then
control flow.

> **Standing caution — a census bucket may be a parser defect, and a census
> *name* may be a guess.** Two distinct failure modes, both of which have now
> fired more than once.
>
> **(a) The bucket is the instrument, not the corpus — twice.** The
> variable-token-width fix (§G2) deleted the `call-token-0x01…0x05` and
> `expr-load-type-0N00A6` families, which were pure misalignment. The
> CALL-token decode deleted the entire **`call-anchor-*` family** — previously
> ~12.4% of blocked functions (`call-anchor-0x00` 235,886, `-0x08` 43,269,
> `-0x20` 24,600 → **0**) — which was measuring a hardcoded 6-byte "anchor"
> that was never an anchor (§G2). Both times a large, plausible, stable bucket
> turned out to be the instrument.
>
> **(b) The name was guessed and was wrong — three times.**
> 1. **The relational opcodes.** Inferred from numeric order; three of six
>    labels were wrong and `==` had no name at all, until one probe per
>    relation was compiled (§G5, `CODEGEN_W6_COMPARE.md` §1.1).
> 2. **`call-anchor-*`.** Named for a structure that did not exist — a parser
>    defect wearing a plausible name, which is why it appears in both lists.
> 3. **`expr-cast`.** `0x40` was named `cast` from a single witness on the
>    conjecture that it is `40 <target-type>`. It is not a cast at all: it is a
>    second CALL token, the intrinsic call, and the real cast opcode is
>    `2C <TYPE> <varint>` (§G2, `docs/IL_CAST_CONVERT.md`). The wrong name
>    survived into two published histograms and a scheduled work item.
>
> **The rule, plainly: name a bucket only from a capture that pins it;
> otherwise leave it hex.** A hex bucket is a result. A guessed name is a lie
> that survives into the roadmap — it gets grouped, ranked and scheduled as if
> it were evidence. And before scheduling work against any bucket, dump the
> bytes at the recorded offset (`c2rs census`, which now prints them) and
> confirm the parse arrived there **aligned**.

### G2 — IL decode coverage

`func.rs` is a positive parser for a handful of shapes; the general `.ex`
grammar (comparisons `24`, shifts `09`, bitwise `0B`, ternary `43 42`, branch
tokens, labels, …) is still opaque, as are the `.ex` header/index region, the
FnHeader interior, most of `.gl`, and **all of `.sy`/`.in`/`.db`** (coverage
map: `IL_BUNDLE_MVP.md`). Each W-step decodes exactly the grammar it needs
(codec-first, round-trip gated); `.sy` (symbol/type table) becomes
load-bearing around W12–W14 when sizes and types stop being inferable from
`.ex` alone.

**IL tokens are variable width — 2 *or* 4 bytes, per token, not per file**
(measured 2026-07-29, commit 40f767d). `detect_token_width` returns one width
for a whole file, which is simply wrong on real TUs: in a single capture of
`system/world/Dir.cpp` the `4F 02` module marker appears both as
`4f 02 e3 09` (2-byte token) and as `4f 02 a4 96 03 00` (4-byte). The
discriminator is **bit 7 of the token's second byte**: clear → the token is
those 2 bytes, set → two more bytes follow. Applying that rule at every `B9`
LOAD site lands on a valid 3-byte operand type at **21,443 sites**.
`func::read_token_var` now implements this and the function parser no longer
consults `detect_token_width`.

Consequence worth remembering: a 2-byte read of a 4-byte token leaves the
parse standing on the token's own tail bytes, which then look like unknown
opcodes. That **fabricated census buckets** — the `call-token-0x01…0x05` and
`expr-load-type-0N00A6` families were misalignment, not vocabulary, and both
vanished once the width rule landed (in-class functions went 4,154 → 7,114 on
the identical instrument). Any future "new opcode" discovered by a parser
that reads fixed-width tokens is suspect until re-checked.

**Operand types use a different rule from tokens, and it is LEB128.**
`TYPE := <tag> <kind> <LEB128 id>` — 3, 4, **or 5** bytes. Settled by a
controlled fixture (`docs/IL_CALL_GRAMMAR.md`): a TU forced to 6000 used types
produces a 5-byte type pinned by the `55 … 4C 4B` call-end framing
(`86 43 9b b9 02`). On Dir.cpp's real call sites the distribution is 4157
3-byte, 3123 4-byte, **1358 5-byte** — so a "3 or 4" rule mis-parses one call
in six.

The same work independently **confirmed `read_token_var` is correct** for
operand tokens: a 32000-symbol fixture forces genuine 4-byte tokens, and
`v31999` loads as `b9 e2 86 01 00`, decoding to exactly `0x09E3 + 31999` with
the fixed `41` / `54 02 29` markers landing where the 4-byte read predicts.

**The CALL token is now decoded, not anchor-matched (landed 2026-07-29,
commit 2870fc1).**

```
CALL := BD <TYPE ret> <flags:1> <varint fn-type-id>          8–13 bytes
```

Nothing in it is fixed but the `BD`; a decoder finds the end field by field,
and no anchor is needed or possible. The old model matched a hardcoded 6-byte
`CALL_CALLEE_ANCHOR = 00 80 01 10 00 00`, which was never an anchor: it is
`flags = 0` followed by `varint(0x1001)`, and `0x1001` is merely the first
function type a *single-function fixture TU* happens to create — true of every
MVP fixture and of essentially nothing else. `read_type` implements the third
encoding in the format (`<tag> <kind> <LEB128 id>`, 3/4/5 bytes), widths pinned
in the tests by the fixed markers that bracket a type (`41` result-type, `55`
arg push, `4C 4B` call end), where a wrong width visibly swallows the marker.
The calling convention is restricted to `0x00` (cdecl) — `0x04` fastcall and
`0x40` varargs refuse rather than mis-emit — and the fn-type id is decoded only
to find the token's end and then **discarded**: it is not the callee (three
different callees sharing one signature produce byte-identical CALL tokens).
Effect on coverage: **7,114 → 7,954 functions in class**, and the
re-attribution exposed what the mis-parse had been hiding — float and double
operand types are now visible at 3.4% and 3.1%.

Full byte evidence for all three encodings, callee resolution via `.gl`, the
statement-list grammar and the ranked residual unknowns:
**`docs/IL_CALL_GRAMMAR.md`**. Its §6.3 validation is the strongest check
available that the model is right: a throwaway whole-body parser built from it
lands **exactly** on the fixed 7-byte function tail for 2,729 of Dir.cpp's
5,239 bodies (52.1%), and the residue is dominated by *unidentified opcodes*,
not width errors — a wrong token width, type width or CALL layout could not let
half a 1.5 MB real TU land on a fixed 7-byte pattern.

**There is a SECOND call token, `0x40`, and it is what the census used to call
`expr-cast` (characterized 2026-07-29, commit 9c7ba7d;
`docs/IL_CAST_CONVERT.md`).** The `cast` name was a guess from one witness and
it is refuted. `0x40` occupies exactly the slot `BD` occupies, and is the
*intrinsic* call:

```
INTRINSIC-CALL := 33 <int-TYPE> <selector>   the selector is a bare int literal
                  40 <TYPE result>            the call token — no flags, no fn-type id
                  ( <expr> 55 <TYPE> )*       arguments
                  4C                          apply
```

The decisive measurement: across three real TUs (`Dir.cpp`, `App.cpp`,
`Game.cpp`) `0x40` is preceded by a bare `int` constant at **6,838 of 6,839**
aligned sites, and the single exception is a parse-misalignment artifact. A
cast opcode would overwhelmingly follow LOADs and sub-expressions; this follows
a constant essentially 100% of the time. That constant is the intrinsic
selector — pinned by controlled fixture at 15 `abs`, 17 `fabs`, 159/160
`_rotl`/`_rotr`, 164 `strcpy`, 165 `strcmp`, 167 `strlen`, 170 `memcmp`,
172 `memcpy`, 173 `memset`, 1973 `sqrt`, plus a dominant **2113–2119**
class-layout / base-offset-adjustment family.

Three knock-on corrections:

- `call-token-0x33` (6.1%) is the **same production with an assigned result**,
  so the intrinsic-call family's real footprint is **~13%**, not 7% — the
  largest schedulable bucket in the histogram (§G1, §G5).
- `0x66` — which `IL_CALL_GRAMMAR.md` §7 ranked as its **#1 unidentified
  blocker** (1,148 Dir.cpp bodies) — is **not a call**. It is the class-layout
  family's class-pair descriptor, `66 02 <tok classA> <tok classB>`. Read that
  doc's ranked-unknowns table with this correction applied.
- **The real cast opcode is `2C <TYPE> <varint>`** (census bucket
  `expr-convert`), and it is the load-bearing hazard of the whole area: the
  *same* `2c 86 41 74 00` token is simultaneously nothing, an `extsb`, an
  `extsh`, a `clrlwi` and a 3-instruction `fctiwz` sequence, discriminated
  entirely by the **source** type — which the operand stack does not carry
  (`IL_CAST_CONVERT.md` §2.2, §4.2). A typed operand stack is the prerequisite,
  and it is the same prerequisite the float/double/`void*` buckets need.

<details><summary>Superseded: the "+1 continuation" reading (kept as a
worked example of a statistical result that was directionally right and
mechanically wrong)</summary>

The measurement below was taken before the LEB128 rule was known. It correctly
detected that types are *not* `+2`, but its "+1" conclusion is just LEB128
truncated at two payload bytes — it could not see the 5-byte case because the
test only offered lengths 3, 4 and 5 against a *next-byte-plausibility* oracle
that a 5-byte type also satisfies by accident. Measured
the same way over the same `.ex`: restricting to LOAD sites with an
unambiguous narrow token (so the type's start offset is known) and to types
whose second byte has bit 7 set, then asking which candidate type length
leaves a plausible next-production byte — 1,940 sites admit **only** a 4-byte
type, 1,606 are ambiguous between 3 and 4, and a 5-byte type (tag + a
`+2`-style wide token) is essentially never right (~151 across all
combinations). So a type reads as `<tag> <2-byte token> [+1 byte when bit 7 of
the token's second byte is set]` — a **+1** continuation, against the
operand token's **+2**. Clean witnesses (only length 4 works):

```text
b9 1d 12 | 86 43 83 08 | 55 86 43 83 08 4c 2c
b9 99 12 | 86 43 a0 08 | b9 98 12 86 43 a0 08
```

This was flagged provisional pending a controlled fixture, and the fixture
duly refuted it. Two lessons worth keeping: a statistic over a large real
corpus can be *precise and still wrong* when the candidate set omits the true
answer, and the fail-closed design meant nothing shipped had to be revised —
`read_token_var` is used only in operand-token positions and types were still
matched as the fixed 3-byte `INT_TYPE`, so the port was never at risk either
way.

</details>

**Outstanding (1)**: `crates/c2-il/src/codec.rs` still assumes a fixed 2-byte
token (`tok16`) and therefore carries the same latent defect on real TUs. It
is round-trip gated, so it fails *closed* (an opaque span) rather than
mis-decoding — no correctness exposure today — but it caps typed coverage on
real bundles and must adopt the variable-width read before the codec is
pointed at the real workload.

**Outstanding (2) — `read_varint`'s short form is a SIGNED byte** (measured by
controlled fixture, `IL_CAST_CONVERT.md` §3.2). The current model is
`b < 0x80 → value = b` (unsigned), `b == 0x80 → 4-byte LE`. Three corrections:

- the short form is a **signed** 8-bit value — `return -5;` is
  `33 86 41 74 fb`, not an escape;
- `-128` is **forced** into the escape form, because `0x80` is the escape
  marker and cannot also be a payload;
- the escape carries **8** payload bytes for tag-`0x88` types (`long long`,
  `unsigned long long`), not 4.

Today `read_varint` rejects `0x81..0xFF` outright. That is fail-closed and
therefore safe — no mis-decode is possible — but it means **every small
negative literal silently blocks its function**, i.e. a self-inflicted share of
the census that is a decode fix, not a codegen class. Fixing it also requires
passing the operand type in, since the escape width depends on the type's tag.

### G3 — Front-end port (`c1xx.dll` → `c1-core`)

Scoped in the FE plan (memory: `frontend-port-roadmap`); replay proof landed.

- **P-F0.2** — characterization probes: which argv tokens affect bundle bytes
  (expect only `-f`), line-number → `.db` deltas, comment/whitespace
  sensitivity → `docs/FE_BUNDLE_MVP.md`.
- **P-F1** — `c1-core` crate, `Frontend` trait, std-only lexer/recognizer for
  the backend MVP class. `.in` is a class constant, `.ex` via the codec's
  encode side, `.sy`/`.gl`/`.db` from observed record patterns + a trivial
  name mangler + the lowercased `-f` path. Grade 1: `PortC1 == captured` per
  file; Grade 2: `PortC2(PortC1(src)) == pipeline obj`.
- **P-F2** — widen in lockstep with the backend's accepted class; `perf-fe`.

Key risk: `.db` line-record semantics (smallest, least understood); any
preprocessor use leaves the class — the recognizer must fail closed.

### G4 — Architecture: shape-matcher → general lowering

`c2-core::passes` is an empty module tree; `paint/` holds Ghidra+LLM
first-draft Rust of the COLOR register allocator (scaffolding, not truth,
gitignored). The port does **not** need to reimplement c2's 35-pass
optimizer — the doctrine is I/O-behavioral, so only the *observable effects*
on emitted bytes matter, per accepted class. The architectural question is
when the accreting shape-matcher gets restructured into a real
IL→lower→emit pipeline. Answer: not yet — keep widening the matcher until
the CFG step (W8) and frames (W10) force a block/instruction IR, then
restructure `codegen` around it, keeping every differential gate green.
COLOR knowledge lands when W5/W10 demand real register-order modeling — the
W5 chain fix already needed the first piece of it (the descending
`r11→r10→r9…` cursor, `CODEGEN_W5_SCRATCH.md` §2), and the tree case will
need the liveness-gated, wrapping version of the same cursor.

### G5 — Coverage measurement (instrument LANDED; P2b census LANDED; baseline below)

The measuring tool is **`c2rs gap`** (real-workload gap scan): run the whole
pipeline — capture with the project's real flags → port → byte compare — over
real dc3 TUs, bucketing each into `capture-fail` / `vocab-gap` /
`codegen-gap` (keyed by the port's own `NotImplemented` reason) / `mismatch`
/ `match`, with JSONL records for longitudinal diffing and a
`--replay-every N` soundness lane extending P0.1 to real workloads.
`scripts/gen_dc3_workload.sh` generates the inputs from a dc3-decomp
checkout.

**P2b — function-level census: LANDED** (commits 63b1ad1, ec401a5, 56d5800).
The TU-level scan is all-or-nothing, so it could only ever report 871 ×
`vocab-gap` and could not rank W5–W14. There is now a per-function view:
`c2rs census <cpp> [--flags-file F --cwd D] [--keep-il DIR]` for a single TU,
and the `c2rs gap` report additionally prints scan-wide function coverage
plus the blocking-feature histogram. Each function segment is run through the
*same* positive parser and keeps its **first** blocking `(production, byte,
offset)` — so the histogram measures the unknown vocabulary honestly (hex
buckets, never guessed names) instead of asserting it.

The census also keeps a **window of bytes either side of each blocking site**
and prints one representative hexdump per feature, bracketing the offending
byte:

```
 1 x expr-cmp-gt
     ... b9 ed 09 86 41 74 b9 ee 09 86 41 74 >24< b9 ed 09 ...
```

That is the single highest-leverage line in the instrument: it is what turned
guessed opcode names into measured ones (the relational correction below, and
the `expr-cast` → intrinsic-call correction of §G2), and what exposed the true
CALL token shape and the meaning of `body-0x3A` / `body-0x53` on a real TU.
Per-function lines are suppressed above 64 functions, where only the histogram
is readable.

**Baseline (2026-07-29, re-run at cebfb88; 878 dc3 TUs, real `/O1 /Oi /EHsc`
flags, 37.3 s at `--jobs 16`). Every TU bucket is unchanged from the
end-of-day scan — W5 depth-2 trees and W13b moved no TU:**

| Class | TUs | % |
|---|---|---|
| **match** | **6** | **0.7** |
| mismatch | 0 | 0.0 |
| codegen-gap | 0 | 0.0 |
| **vocab-gap** | **865** | **98.5** |
| capture-fail | 7 | 0.8 |

**The match bucket is nonzero for the first time**, from two rungs. Five of the
six are empty TUs (R1, §1) — five of the seven zero-function TUs in the
workload, the other two refused on purpose (a stray `4F 1F` after the module
end defeats the positive `is_empty_module` test, and relaxing that test is not
worth 2 TUs). The sixth is `system/utl/Spew.cpp`, the **first function-bearing
real TU to match**, unlocked by R2 + R3 (empty function bodies, then the `/Gy`
COMDAT shape they exposed — box below).

**Function census: 79,719 / 2,462,571 functions in class (3.24%)** (re-measured
at cebfb88, 878 TUs, 37.3 s at `--jobs 16`). Progression on the identical
instrument:

| | in class | % |
|---|---:|---:|
| start of day | 4,154 | 0.17 |
| + variable token width (§G2, 40f767d) | 7,114 | 0.29 |
| + CALL-token decode (§G2, 2870fc1) | 7,954 | 0.32 |
| + empty function bodies (`w10_empty_fn.cpp`, a44c8f3) | 78,028 | 3.17 |
| + W13a float/double leaves (9c7ba7d) | 79,041 | 3.21 |
| + signed varint short form (66f408d) | 79,718 | 3.24 |
| + W5 depth-2 trees (9b7df37) **and** W13b one-constant bodies (cebfb88) | **79,719** | **3.24** |
| + the 2026-07-29/30 overnight ladder: statement layer + chain canonicalization + multi-arg tail calls + expression/intrinsic decode (≈ cebfb88 → 6edfef6) | 87,423 | 3.55 |
| + `/O1` support, `/O1` compare spines, indirect-load leaves, intrinsic-2117 decode (**HEAD `2724ca5`, re-measured 2026-07-30**) | **109,501** | **4.45** |
| + `.sy` locals, the line-70 `this` mis-emit fix, and the lexical-scope layer (**HEAD `b775afe`, 2026-07-30**) | **110,366** | **4.48** |
| + T3 narrow-integer getter leaves — `lbz`/`lhz`/`ld` (`a6304fa`) | 140,476 | 5.70 |
| + T2 aggregate-TYPE size decode (`58099c9`) — a correctness fix, measured yield **0** | 140,476 | 5.70 |
| − the argument-register precondition (`1158356`) — a **correctness fix that costs coverage** | **122,487** | **4.97** |
| − the variadic-function refusal (`8142c17`) — another correctness fix, cost **0** | 122,487 | 4.97 |
| + T1 width-4 pointer TYPEs through the leaf shapes (`8da703e`) | **210,603** | **8.55** |
| + `.sy` widths — six wrong encodings, so it had never bound on a real TU (`320f618`) | **211,012** | **8.57** |
| + `.sy` keyed to `.ex` by the exit label (`ca1469b`) | **228,298** | **9.27** |
| − variadic + dllexport refusals (`8142c17`, `62b9dfc`) — correctness, cost 0 | 228,298 | 9.27 |
| + the generated empty destructor's base delegation (`2faed1d`) | **246,162** | **10.00** |
| + the `66` class-pair descriptor's refs read as LEB128, not fixed pairs (`2e41e3f`) | **271,557** | **11.03** |
| + the member-sub-object destructor, any `addi`-range offset (`76cc2ec`) | **280,020** | **11.37** |
| + the generated destructor's **member** sub-object, at offset 0 and at any `addi`-range offset (`a62633c`) | **280,020** | **11.37** |
| + D1, the generated empty destructor's base delegation (`1caf463`) | **246,162** | **10.00** |

That last row goes the wrong way on purpose, and it is the most instructive row
in the table. A formal's list index had been standing in for its
argument-register number; a by-value aggregate wider than 8 bytes takes more than
one GPR and `int gb(Big v, H* h) { return h->mi; }` emitted `lwz r3,0(r4)` where
c2 emits `lwz r3,0(r6)`. Enforcing the precondition turns 17,989 admissions into
refusals. Mismatch outranks coverage, so the number is allowed to fall — but the
*reason* it falls this far is not the aggregates:

| refusal reason | functions |
|---|---:|
| `param-width-undetermined` — `.sy` did not bind, so widths are unknown | **567,549** |
| `param-multi-reg` — a genuinely multi-register parameter | **1** |

One. The entire bill is `.sy` failing to bind on real translation units, which is
now the #1 census blocker at 2.3× the next (`expr-call-in-expr`, 248,195) and is
the top of the worklist. It also retro-explains why the `.sy` int-locals rung
measured ~0 workload yield when it landed: `.sy` appears never to have bound on a
single real TU, so that rung has been fixture-only from the start.

Adversarial review found the mechanism — `read_record` read a size as a `u16`
where the stream carries a **varint** — and fixing it, plus five more wrong widths,
made `.sy` parse real translation units for the first time (it had managed 3 of
200). `param-multi-reg` went 1 -> **23**, the first honest count of that key: the
old 1 was a mis-read size exceeding 8, and a 16-byte class with a copy constructor
is really a 4-byte *pointer* passed by hidden reference, in **one** register.

**The widths were necessary and are not sufficient: +409 against a 567,549
bucket.** `param-width-undetermined` fell only to 554,056, because the constraint
moved to the block-to-segment **count** check (9,629 blocks against 9,602
segments). Relaxing that check is the trap, and it was measured rather than
reasoned about: "take the first `n_segments`" yields census +2,981 with **0
mismatch** while binding one function's data to another for **343,315 of 554,056**
functions, since the surplus blocks are interspersed rather than a tail. `GAPS.md`
§6 carries the general form — a green differential cannot grade a
*correspondence*, only a decode. The next rung needs a **key** (the block header
token) rather than a position.

**The key exists, and it closed the bucket: 211,012 -> 228,298 (+17,286), 8.57% ->
9.27%, mismatch 0, 0 functions lost, 0 TUs changing class.**
`param-width-undetermined` fell 554,056 -> **6,974**, and `expr-call-in-expr`
(304,111) is the head of the census for the first time. A `.sy` block's header
token is its `.ex` segment's **exit label** — the token named by both the `3A` and
the terminal `29` of the return plumbing — and that was established rather than
inferred from the old coincidence: over 871 TUs, 2,434,636 of 2,434,639 segments
yield an exit label, every one of those tokens names **exactly one** block, and the
bindings are strictly increasing in every file (0 violations). The correspondence
invariant the oracle cannot supply — every `.ex` `2D` formal token of a segment must
be declared by the block bound to it — holds for 99.95% of the candidate pairs
(38% under the positional relaxation above), and the 1,118 that fail it are refused,
so 100% of the bindings made are ones it confirmed. `param-multi-reg` went 23 ->
**1,851**: that construct is *reachable* now, and refusing it is the fail-closed
boundary rather than a regression. Checked, because an 80× jump in a key is also
what a decode bug looks like: the offending declared sizes are 16 B (1,810), 20 B
(24), 12 B (6), 24 B (6), 80 B (6) and 36 B (1) — no zero, nothing absurd, so every
one is a real by-value aggregate wider than one GPR and not a misread width.

Two findings worth carrying forward. First, the "surplus" blocks were never surplus:
the `.sy` block count equals the `.ex` **function-tail** count in all 856 files that
parse, so `bundle::split_function_bodies` is what misses ~1,972 bodies (0.08%) and
the census denominator 2,462,571 is that much short of the real function count —
a fixable under-count, and the next thing to check if a rung's numerator looks off.
Second, this rung is unfixturable by construction: c1 emits a body for every
function it compiles, so the block/segment mismatch only appears where the splitter
misses one, and a TU with a missed body cannot emit an obj at all (`gl_defined_names`
refuses it). The binding is graded by unit tests over transcribed `.ex`/`.sy` pairs
and by the workload scan; no fixture can grade it byte-exact, and writing one that
merely reaches the code would have graded nothing.

The first two steps were *decode* fixes, not new codegen — the expected shape
of progress while the wall is `vocab-gap`. The third is a 10× jump from one
very small class: empty bodies were ~4.4% of blocked functions by count, and
accepting them also unblocks the many TUs that are *mostly* trivial accessors
and destructors. The fourth is the smallest of the four and the most
codegen-heavy, which is the normal ratio once the decode wall stops being the
binding constraint.

**The last row is the honest one, and it is worth more than the four above it as
a lesson: two rungs of real codegen bought exactly ONE function.** W5 depth-2
trees moved the census by 0 (recorded as such when they landed) and W13b by 1 —
the `expr-load-type-864540` (float) bucket went 81,478 → 81,477. Both were
fixture-driven rungs, and this is what that looks like when it is measured
instead of assumed. It does not make them worthless: W13b is a prerequisite for
the float/double buckets *as a whole* (6.6% between them), it forced the
emitter-wide relocation-layout fix, and it is the first class in the port that
needs a second COFF section, relocations against a data symbol, and a TU-wide
pool. But the coverage claim is 1 function, and the strategy note in §4 —
demand-driven widening — is the rule these two rungs did not follow.

> ### The obj shape depends on argv the bundle does not record (`/Gy`)
>
> Landing empty function bodies immediately produced a **mismatch** on
> `system/utl/Spew.cpp`: the port emitted a 5-section obj with one packed
> `.text` against a reference with **six** sections and a separate 4-byte
> `.text` per function, characteristics `0x60401020` (COMDAT) rather than
> `0x60400020`.
>
> Root cause: **`/O1` and `/O2` imply `/Gy`** (function-level linking). The dc3
> workload compiles with `/O1`; every fixture here uses `/Ox`, which does not.
> So the same IL bundle legitimately produces two different objs depending on a
> compiler flag that the bundle never records — and **matching every fixture
> never licensed emitting for a real workload TU**. This is a different kind of
> gap from the W-ladder: not a missing *class*, but a missing *input*.
>
> `PortC2::with_function_level_linking` now carries it, `c2rs gap` and
> `c2rs prefilter` derive it from the project's real flags, and
> `coff::emit_comdat_obj` emits the COMDAT shape (one `.text` per function,
> aux `Selection = 1` NODUPLICATES, each function at `Value` 0 in its own
> section, no inter-function padding, 11 fixed symbols + 3 per function).
> `Spew.cpp` is now the **first function-bearing real TU to match** — the other
> five matches are empty TUs with no `.text` at all — taking the bucket to
> **6/878**.
>
> Standing implication: any future claim of the form "the port handles class X"
> must say *under which flags*. `c2rs perf`'s numbers, the fixture gate, and
> the workload scan are not all speaking about the same emitter configuration.

**Replay soundness re-proven at full strength on 2026-07-29 with the
post-fix code: 871/871 capturable TUs byte-exact, 0 diverged**
(`--replay-every 1`, 43.4 s at `--jobs 16`). The oracle holds on the entire
capturable real workload, so every IL-derived measurement above rests on a
reference side that was itself checked, not assumed.

Fixture-level status at cebfb88: **21/41 `Port=Match`, 0 mismatch**;
`cargo test --workspace --release` green (202 tests, toolchain present).

> **The denominator is 2,462,571 functions — not ~903k.** An earlier figure of
> 902,730 came from counting `.gl` mangled names; that is **not** the function
> count. `mangled_names` accepts only `?…@@…` forms, and `.gl` also lists
> externals. Measured on `system/world/Dir.cpp` (1.5 MB `.ex`): 2,153 `.gl`
> mangled names, 5,340 `4F 1F` fn-start markers, **5,239 `LO` body markers
> (`4C 4F 11`)**, 5,243 function tails (`4F 12 47 54 01 54 00`). The last two
> agree to 0.08%; the two-byte `4F 1F` scan is ~2% inflated by payload
> collisions. The census therefore anchors on the `LO` body marker
> (`func::split_function_bodies`). Do not re-derive a function count from
> `.gl`.

Blocking-feature histogram, top 8 (cebfb88 scan; percentages are of the
**2,382,852** *blocked* functions), with what each bucket is now known to be:

| Functions | % | Feature | What the bytes are |
|---:|---:|---|---|
| 363,684 | 15.3 | `call-token-0xB9` | **member / indirect calls** — the callee is an *expression*, not `26 <tok>` |
| 167,205 | 7.0 | `expr-intrinsic-call` | the `0x40` token — a **second call token**, not a cast (§G2) |
| 144,276 | 6.1 | `call-token-0x33` | the **same** intrinsic-call production, result assigned |
| 119,800 | 5.0 | `expr-call-in-expr` | a call nested inside an expression |
| 81,477 | 3.4 | `expr-load-type-864540` | **float** — one fewer than the previous scan; that one function is all W13b bought |
| 80,284 | 3.4 | `call-token-0x26` | `26 dest 26 callee BD …` (assign a call result) |
| 75,081 | 3.2 | `expr-load-type-888541` | **double** (3.1 % on the previous scan — a rounding-boundary move, not a corpus one) |
| 70,078 | 2.9 | `body-0x53` | first statement is an `if`/compound |

`expr-load-type-864383` (**void\***) sits just below this cut at **47,640
(2.0%)** — now quotable, because it comes from this scan rather than the
superseded one — followed by `expr-load-type-864275` (37,060 / 1.6%) and
`call-end-0x26` (36,640 / 1.5%). Behind the top eight is a long tail of
**1,050 more distinct features** (down from 1,217 at the mid-day measurement, as
retirements accumulated).

> **That table is superseded (re-measured 2026-07-30, HEAD `2724ca5`,
> `c2rs gap … --jobs 16`).** The expression-layer decode and the census
> de-conflation (6edfef6, a8851f7) re-attributed the head — `call-token-0xB9`
> and the intrinsic pair no longer appear under those names. The measured
> top eight, percentages of blocked functions:
>
> | Functions | % | Feature |
> |---:|---:|---|
> | 275,829 | 11.7 | `expr-call-in-expr` |
> | 170,401 | 7.2 | `body-0x53` (leading `if`/compound — statements/control flow) |
> | 149,168 | 6.3 | `expr-intrinsic-base-member-addr` (2117; decoded, mostly non-leaf bodies) |
> | 137,511 | 5.8 | `expr-intrinsic-this-adjust` (2113) |
> | 89,983 | 3.8 | `expr-load-type-864540` (float) |
> | 79,542 | 3.4 | `expr-load-type-888541` (double) |
> | 51,775 | 2.2 | `expr-load-type-864383` (void\*) |
> | 37,671 | 1.6 | `body-0x29` |
>
> … then `expr-intrinsic-memset` 1.5%, `expr-bit-and` 1.4%, `expr-convert`
> 1.0%, and **886 more distinct features**. Note the bucket-vs-win caution of
> `GAPS.md` §6 measured on this very head: decoding 2117 moved **32**
> functions of its 149,200, because the decode lands in the indirect-load
> *leaf* recognizer and most of those bodies are not leaves.

> **Superseded again (2026-07-30, HEAD `ca1469b`).** Census **228,298 /
> 2,462,571 = 9.27 %**; mismatch 0, codegen-gap 0, port-error 0, match 6.
> Percentages below are of the **2,234,273** blocked functions:
>
> | Functions | % | Feature |
> |---:|---:|---|
> | 304,104 | 13.6 | `expr-call-in-expr` |
> | 141,800 | 6.3 | `expr-intrinsic-this-adjust` (2113) |
> | 138,707 | 6.2 | `expr-intrinsic-base-member-addr` (2117) |
> | 92,724 | 4.2 | `expr-load-type-864540` (float) |
> | 79,158 | 3.5 | `expr-load-type-888541` (double) |
>
> `param-width-undetermined` was the head at **554,056 (24.6 %)** for exactly one
> measurement and is now **6,974**, which is the shape a *reader gap* has when it
> closes: the whole bucket moves at once. Note what it moved *to* rather than
> into class — 547 k functions cleared their first blocker and 17,286 became
> emittable, so `expr-call-in-expr` grew by 55 k, `this-adjust` by 24 k, and the
> float/double load buckets by 26 k and 22 k. First-blocker attribution behaving
> exactly as `GAPS.md` §6 describes; the census gain and the bucket drop measure
> different things and neither predicts the other.
>
> `body-0x53` has left this table by being ported (the lexical-scope layer), and
> `param-multi-reg` entered it at 1,851 — a real construct, its widths sampled
> (16 B ×1,810, 20 B ×24, 12 B ×6, 24 B ×6, 80 B ×6, 36 B ×1) rather than
> assumed, after the previous count of 1 turned out to be a misread field.

> **Superseded again (2026-07-30, HEAD `1caf463`) — D1, the generated empty
> destructor.** Census **246,162 / 2,462,571 = 10.00 %** (`scan-nonleaf.jsonl`,
> 878 TUs, same denominator, mismatch 0, codegen-gap 0, port-error 0, match 6).
> **+17,864 functions across 828 TUs, 0 lost, 0 TUs changing class**, and
> `expr-call-in-expr` fell **304,104 → 286,240**, which is *exactly* −17,864.
>
> That exactness is the finding. **No other blocker bucket moved by a single
> function** — none grew, and only this one shrank. So unlike every previous rung
> (`.sy`: 547 k functions cleared their first blocker and moved to their next;
> 2117: a 149,200 bucket that yielded 32) every function this rung cleared came
> **all the way into class**. That is what a whole-body-complete shape looks like
> in the histogram, and it is the reason to prefer picking one.
>
> Still 13.6 % → 12.9 % of blocked: the bucket is the #1 entry either way, and
> `docs/IL_CALL_IN_EXPR.md` §2 says why — 94 % of it is member calls with
> loaded, named, chained or computed receivers, all of which need real frames.

Two rows have **left** this table since the mid-day scan, and neither left by
being fixed in the corpus sense:

- **`body-0x3A` (107,253 / 4.4%) is gone** because it was **ported** — it was
  the empty function body, and R2 accepted it. This is the one legitimate way a
  bucket disappears.
- **The whole `call-anchor-*` family is gone** — 12.4% of blocked functions
  (235,886 + 43,269 + 24,600) — because it was measuring the port's own
  hardcoded anchor, not a real gap (§G2, and the standing caution in §G1).

Do not diff this histogram against an earlier one without accounting for both,
and note that every percentage moved simply because the blocked denominator
shrank from ~2.45 M to 2,383,530 and then to 2,382,852.

How to read the bucket names (`func::Block::feature`) — `<production>-0xNN`
means the parse was inside that grammar production and could not consume byte
`NN`:

- `call-token-*` — the byte where the `BD` CALL token was expected after the
  callee expression.
- `body-*` — the byte that opened the function body, where only a call ref
  (`26`), LOAD (`B9`) or literal (`33`) is modeled.
- `expr-*` — inside the operand stream; `expr-*-type-NNNNNN` reports the
  operand's whole inline type, because the type triple *is* the feature (int
  vs unsigned vs float vs pointer). Named buckets (`expr-intrinsic-call`,
  `expr-convert`, `expr-call-in-expr`, the relationals) now carry only
  **capture-verified** names — see the corrections below, and the standing
  caution in §G1 for why that qualifier is load-bearing.

**The census's relational opcode names were guessed, and were wrong.**
Compiling one probe per relation against the live toolchain (commit 45421f6,
`docs/CODEGEN_W6_COMPARE.md` §1.1) measured `0x1F` `==`, `0x20` `!=`,
`0x21` `<=`, `0x22` `<`, `0x23` `>=`, `0x24` `>`. The table had said `0x20`
`==`, `0x21` `!=`, `0x23` `<=`, `0x25` `>=`, and had **no** name for `==` at
all — so three buckets were mislabelled and every `==` landed in an unnamed
one. Diagnostic only (acceptance never consults the name) but the ranked
blocker lists in `docs/GAPS.md` are keyed on these strings.

**And `expr-cast` was guessed, and was wrong — the third such name.** `0x40` is
not `40 <target-type>` and is not a cast; it is the intrinsic-call token, the
real cast is `2C <TYPE> <varint>`, and the bucket is now `expr-intrinsic-call`
with `expr-convert` for the genuine article (§G2, `docs/IL_CAST_CONVERT.md`).
The wrong name did not just mislabel a row: it grouped 6.8% under "casts",
ranked that group as the cheapest large characterization job, and put it on
the next-actions list — while the *same* production's other 5.9%, sitting two
rows below under `call-token-0x33`, was scored as unrelated W11 demand. A hex
bucket is a result; a guessed name is a lie that survives into the roadmap.

Notes:

- **Replay soundness holds at full strength**: 871/871 byte-exact, 0 diverged
  on both the 2026-07-20 and the 2026-07-29 `--replay-every 1` passes. The
  oracle holds on the capturable real workload.
- The 7 `capture-fail`s are all `synth_xbox/soundtouch` files the real 360
  build excludes (x86-only `#error` guards) or builds with per-target flags —
  a workload-manifest refinement, not a port gap.
- **The TU wall is still `vocab-gap` for almost every TU that contains code**:
  those 865 TUs die at `c2_il` function decode before codegen is consulted,
  because `functions()` is all-or-nothing per TU. That is unchanged and
  expected — a TU with 700 functions of which 699 are in class is still one
  `vocab-gap`. Five of the 6 matches are TUs with *no* functions to decode; the
  sixth (`Spew.cpp`) is the first where every function in the TU happened to be
  in class at once. The census is what moves per widening step; the TU buckets
  move much later.

Remaining for G5: keep the JSONL baselines diffable scan-over-scan (coverage
must be monotone), and keep characterizing the head of the histogram. The two
largest named productions are now both characterized rather than unknown —
`call-token-0xB9` is member/indirect calls (`IL_CALL_GRAMMAR.md` §3.2/§6.2,
out of class by construction, not schedulable) and the `0x40`/`call-token-0x33`
pair is the intrinsic call (`IL_CAST_CONVERT.md`, **~13% and schedulable**, the
largest such bucket on the board). What remains genuinely unidentified at the
head is smaller and mostly type-driven; the cheapest coverage left in the
instrument itself is the `read_varint` signed-byte fix (§G2, Outstanding 2),
which is a decode defect blocking every small negative literal rather than a
corpus feature at all.

### G6 — Harness experiments maturity

`retrieve`/`search`/lifter-eval are prototypes riding the harness. They are
not the port and need no maturation for the stack goal; leave them, but keep
them out of the port's critical path.

## 4. Strategy

1. **Demand-driven widening.** P2 (census) before W5+. Every W-step is chosen
   by measured frequency, not intuition, and verified to move the number.
2. **The per-class recipe** (the loop W1–W4 already followed — every class
   lands the same way):
   1. Stage minimal `.cpp` fixtures: positive shapes *and* negative neighbors.
   2. `c2rs capture` + reference-compile; byte-classify the obj
      (CONST/DERIVED) in the relevant `docs/` doc.
   3. Extend the codec / positive parser to accept exactly the new shape —
      fail closed on the neighbors.
   4. Extend codegen + coff (new sections/relocs as needed).
   5. Gate: `c2rs diff` byte-exact on all positives, `NotImplemented` on all
      negatives, `cargo test --workspace` green, `c2rs perf` re-confirmed.
   6. Re-run the census: coverage up, no regressions, histogram re-ranked.
   7. Doc the class (extend `CODEGEN_PPC_MVP.md` / `IL_BUNDLE_MVP.md` or a
      new doc), update `fixtures/README.md`.
3. **Two-speed decode.** The round-trip codec (typed islands over opaque
   spans) generalizes ahead of the parser, so each W-step inherits tokenized
   grammar instead of raw bytes. Never weaken the round-trip gate to land a
   class.
4. **FE in parallel once P-F1 lands.** The backend ladder and the FE track
   share exactly one interface — the bundle — so they can widen in lockstep
   without blocking each other.
5. **Composition is the milestone that matters.** A `c2rs compose` subcommand
   (source→obj in-process, byte-exact, timed) is the demo that the stack is
   real; everything before it is enabling work.

## 5. The phases, in order

```
P2    gap-scan baseline with `c2rs gap` (dc3 workload) (G5)   [DONE 2026-07-20]
P2b   function-level census + blocking histogram      (G5; the driver) [DONE 2026-07-29]
P2c   CALL / body grammar characterization → IL_CALL_GRAMMAR.md [DONE 2026-07-29]
P2d   cast / intrinsic-call characterization → IL_CAST_CONVERT.md [DONE 2026-07-29]
R1    empty-TU obj emission — first nonzero match bucket   [DONE 2026-07-29: 5/878]
R2    empty function bodies                          [DONE 2026-07-29: census 0.32% → 3.17%]
R3    COMDAT .text per function under /Gy            [DONE 2026-07-29: match 5 → 6/878]
W5    multi-scratch expressions                     [CHAINS + DEPTH-2 TREES DONE 2026-07-29; depth 3 open]
W6    integer comparisons → bool                    [LEAVES DONE 2026-07-29; <,<=,>= vs k≠0 open]
W7    shifts + bitwise + strength reduction         (staged)
W8    control flow (if/else, ternary, loops)        (first CFG; IR restructure)
W9    division / modulo
W10   general frames + locals (+ .pdata labels W-UNW-1)
W11   calls generalized (+ W-UNW-1 for multi-fn TUs)
W12   memory / struct access
W13   float codegen                                  [13a DONE 2026-07-29; 13b DONE at ONE constant/body; 2+ constants open]
W14   data sections / globals
P-F0.2 FE characterization probes → FE_BUNDLE_MVP.md   (parallel track)
P-F1  c1-core MVP + Grade-1/2 gates                    (parallel track)
P-F2  FE widening in lockstep + perf-fe                (parallel track)
P3    c2rs compose — source→obj in-process, byte-exact + perf-fe scale
```

**The W-numbering above is the pre-census estimate and is not the running
order.** The current histogram (§G5), read against the now-characterized CALL,
body and intrinsic-call grammar, ranks the schedulable work as: the
**intrinsic-call family** (`expr-intrinsic-call` 7.0% + `call-token-0x33` 6.1%
= **~13%**, one production — the largest schedulable bucket on the board), then
**generalized calls** (`expr-call-in-expr` + `call-token-0x26`, 8.4% — W11),
then **non-int operand types** (float 3.4% + double 3.1% = 6.6%, plus `void*`
behind them — W13b/W12), with control flow (`body-0x53`, 2.9% — W8) behind
those. The single largest bucket of all, `call-token-0xB9` at **15.3%**, is
**not schedulable work at all**: it is member and indirect calls, which have no
relocatable callee name (indirect) or need a `this`/vtable model (member), and
which must keep failing closed until W11/W12 exist (§G1,
`IL_CALL_GRAMMAR.md` §6.2). The attribution of buckets to rungs remains
*inference*; the histogram itself is measurement.

What the intrinsic-call rung actually costs, so its rank is not mistaken for
cheapness: **decoding** it (`40 <TYPE>`, the `(<expr> 55 <TYPE>)* 4C` argument
loop and the `66 02 <tok> <tok>` class-pair descriptor) is small, and buys the
census immediately — bodies where an intrinsic call is not the *blocking*
feature start reaching their real blocker. **Accepting** it is a different
proposition: the id space is a c1xx-internal table that cannot be enumerated
from the IL, and c2's expansion depends on the argument *literal values* as
well as the id, so the only sound policy is an allow-list of ids pinned by
controlled fixture with their arguments constrained too
(`IL_CAST_CONVERT.md` §1.4, §4.1). Decoding is not accepting.

W8 remains the pivot whenever it is scheduled: it forces the block/instruction
IR, the `.ex` branch grammar, and compare/branch fusion. Everything after it
is easier to schedule but harder to keep honest — more of the obj becomes
derived, so the CONST/DERIVED classification discipline matters more per step,
not less.

## 6. Next actions (this week-scale)

1. ~~**P2**: land `c2rs gap`, run the dc3 baseline, commit the numbers.~~
   **DONE 2026-07-20** — see G5. Headline: 0 mismatch, 871/871 replay
   soundness, 99.2% vocab-gap. The scan cannot rank W5–W14 because decode
   fails TU-wholesale first.
2. ~~**P2b — function-level census**: classify each function inside each real
   bundle by its first blocking feature, so the histogram counts functions,
   not TUs.~~ **DONE 2026-07-29** (63b1ad1, ec401a5, 56d5800) — see G5.
   Denominator anchored on the `LO` body marker (**not** `.gl` names); the
   scan-wide coverage + histogram print from `c2rs gap`, and each feature
   carries a bracketed hexdump of its blocking site. Bundled finding: IL
   tokens are variable width (40f767d, §G2), which retired two fabricated
   bucket families and moved the numerator 4,154 → 7,114.
3. ~~**Characterize `call-token-0xB9` / the `26` prefix.**~~ **DONE
   2026-07-29** — `docs/IL_CALL_GRAMMAR.md` (characterization, 8131ba2) and
   `c2-il`: the CALL token is now decoded rather than anchor-matched
   (2870fc1, §G2). Results: `26 <tok>` is a **symbol/lvalue push**, not a
   call prefix; the callee is an arbitrary *expression*, so `call-token-0xB9`
   is member/indirect calls; the whole `call-anchor-*` family was the port's
   own bug and is gone; census 7,114 → **7,954** (0.32%).
4. ~~**R1 — empty-TU obj emission.**~~ **DONE 2026-07-29** (fa3410a):
   **match 0 → 5 of 878**, the first nonzero match bucket, which trips the
   downstream tripwire recorded in `GAPS.md` §5.
5. ~~**W5 chains (the mis-emit fix) + W6 compare leaves.**~~ **DONE
   2026-07-29** (40749e7) — see §1, `CODEGEN_W5_SCRATCH.md` §6 and
   `CODEGEN_W6_COMPARE.md`. The mis-emit is the first correctness bug the
   differential has caught; it outranked all widening work the moment it
   appeared, which is the rule §7 states.
6. ~~**Empty function bodies** (`body-0x3A`, 4.4% of blocked functions).~~
   **DONE 2026-07-29** as R2 (a44c8f3) — census **7,954 → 78,028
   (0.32% → 3.17%)**, the largest single jump so far. It also forced R3
   (987fc8b, the `/Gy` COMDAT shape) by turning a latent flag dependency into
   a live mismatch, which took the match bucket to 6/878. The
   `IL_CALL_GRAMMAR.md` §4.2 trailing-expression variant stays rejected.
7. ~~**Characterize `expr-cast`.**~~ **DONE 2026-07-29** —
   `docs/IL_CAST_CONVERT.md` (9c7ba7d). It **refuted the name**: `0x40` is a
   second CALL token, the intrinsic call; the real cast is `2C <TYPE> <varint>`;
   `call-token-0x33` is the same production with an assigned result; and `0x66`
   is the class-layout family's descriptor, not the call-family opcode
   `IL_CALL_GRAMMAR.md` §7 ranked #1 unknown (§G2).
8. ~~**W13a — float/double leaves.**~~ **DONE 2026-07-29** (9c7ba7d) — see §1
   and `docs/CODEGEN_W13_FLOAT.md`; `mvp_fmul3.cpp` is `Port=Match`, census
   78,028 → **79,041**, then 79,718 with the signed-varint fix.
9. ~~**W5 trees**~~ **DEPTH 2 DONE 2026-07-29** (9b7df37) — `w5_tree2.cpp` is
   `Port=Match` on all four shapes; depth 3 (`w5_tree3.cpp`) and all eleven
   `w5_tree_neg.cpp` functions still refuse. The gate on the depth is the
   unexplained `+`-root register swap (§1). Census movement: **0**.
10. ~~**W13b — float constants.**~~ **DONE 2026-07-29 at one constant per body**
    (cebfb88) — `w13b_fconst.cpp` and `w13b_fdedup.cpp` are `Port=Match`;
    `w13b_fpool.cpp` and `w13b_ffold.cpp` must keep refusing. See §1 and
    `docs/CODEGEN_W13_FLOAT.md` §5. Census movement: **+1 function** (79,718 →
    79,719). Two side-effects worth more than the coverage: it forced the
    emitter-wide relocation-layout fix (a section's relocations follow *that
    section's* raw data), and it established that **c2, not c1xx, evaluates
    floating-point constants** — so the two-or-more-constant case needs c2's
    constant evaluator *and* its scheduler, and stays closed.
11. ~~**Decode the `0x40` intrinsic-call production.**~~ **DONE 2026-07-30** —
   `docs/IL_INTRINSIC_CALL.md`, five tracked negatives
   (`fixtures/cpp/il_intrinsic_{nullary,bits,layout,fold,byval}.cpp`). The
   selector is decoded at both census sites, so `expr-intrinsic-call` resolves
   with **zero residue** (213,411/213,411) and 95.4 % of `call-token-0x33`
   turns out to be the same production behind a `26 <sym>` push. **The
   production's real footprint is 16.1 % of blocked functions — 381,488 — not
   9 %**, and **86 % of that is one family**: the class-layout adjustments
   2113…2119 at 329,205 functions (13.9 %), of which 2117 (6.3 %) and 2113
   (5.8 %) are each individually larger than every remaining operand-type
   bucket. Census movement: **zero, to the function** (87,423/2,462,571 before
   and after) — the decode replaces one `Err` with a better-labelled `Err` by
   construction. 18 of the workload's 20 selectors are now named by controlled
   fixture, including three `IL_CAST_CONVERT.md` left UNKNOWN (815 `_abs64`,
   1948 `__mftb`, 2119 `dynamic_cast`) and one it flagged unproven (337
   `throw`); three of its structural claims are corrected (2113-vs-2114 is the
   *null guard*, 2115's offset is not pre-negated, `0x66`'s `02` is an
   *arity*). **Acceptance stays closed** for three separately captured reasons
   (`IL_INTRINSIC_CALL.md` §5): the emission turns on literal argument values
   rather than the id, the result register is chosen by the consumer, and the
   constant-folding rule differs between the integer and floating halves.
11a. **The widening order changed as a result.** 2117 and 2113 are small
   lowerings (a folded `lwz` displacement; an unguarded `addi`) blocked only by
   the operand-stack *type tracking* that `IL_CAST_CONVERT.md` §5 already names
   as the prerequisite for `0x2C`, floats and pointers. The same three
   prerequisites now unlock ~14 % of blocked functions through this family, so
   they rank above the remaining per-type buckets rather than beside them.
12. **Fix `read_varint`'s signed short form** (§G2, Outstanding 2) — a decode
    defect, not a class: every small negative literal currently blocks its
    function. Needs the operand type threaded in for the 4-vs-8-byte escape.
13. **W5 depth-3 trees** — per `CODEGEN_W5_SCRATCH.md` §7 (the cursor rule, the
    level-order emission rule, and the six-row shape gate G1–G6);
    `w5_tree_neg.cpp`'s eleven functions must stay `NotImplemented`, and the
    `+`-root register swap needs explaining rather than re-observing before the
    depth limit moves.
14. **Port `codec.rs` to the variable-width token read** (`tok16` → the
    `read_token_var` rule), round-trip gate unchanged, so typed coverage can
    be measured on real bundles rather than fixtures (§G2).
15. **P-F0.2**: argv/line-number/whitespace probes → `docs/FE_BUNDLE_MVP.md`.

**Ordering note, now that two consecutive fixture-driven rungs have each bought
≈0 census:** items 11–15 above are *not* in priority order — item 11 (the
intrinsic-call decode, ~13% of blocked functions) is, and the two rungs just
landed are the argument for it. §4's first strategy point is demand-driven
widening; W5 trees and W13b were both taken on staged-fixture evidence, and the
census says what that is worth on this corpus. Continuing down the fixture pile
is a choice to keep paying that ratio.

**Ordering re-check (2026-07-30 review, §6b).** Item 11 is done, and the
measured head was: `expr-call-in-expr` (11.7%), `body-0x53` (7.2%), the
intrinsic 2113/2117 pair (12.1% combined, acceptance blocked on operand-type
tracking and member addressing). The near-term target with the best
match-bucket leverage is the **one-away TUs** (scan JSONL,
`fn_total - fn_in_class = 1`): at the time `body-0x53` ×4,
`assign-dst-not-formal` ×3 (needs the positive local signal — `.sy`),
`expr-call-in-expr` ×1, `expr-load-type-A64381` ×1.

**Both were taken, and the outcome separates decode reach from acceptance
sharply enough to be worth recording (measured at `b775afe`).**

| step | bucket cleared | census | fully in class |
|---|---|---|---|
| `.sy` locals | `assign-dst-not-formal` 5,534 → 5,533 | 109,501 → 109,501 | **+0** |
| lexical scopes | `body-0x53` 170,401 → **0** | 109,501 → 110,366 | **+865** |

Locals bought nothing at workload scale even though the decode is right and
byte-exact on fixtures: whole-file `.sy` refusal denied locals to every function
in any TU containing one aggregate-typed local, and the three TUs the bucket
pointed at turned out not to want locals at all — their destinations are member or
global stores, correctly out of class. The scope layer's 170,401 → 0 is the
cleanest confirmation the census works as an instrument, and its 865/170,401 yield
is the cleanest statement of the instrument's limit: those functions all advanced,
to `expr-call-in-expr` (+28,984), `body-0x29` (+10,877) and the intrinsic family
(+8,000).

**Where the leverage now is**, recounted after the scope layer (16 TUs within 3
functions of matching, none of them closed):

  * **`expr-load-type-*` gates 9 of the 16.** `864381` ×4 TUs, `A64381` ×4,
    `864383` ×3, plus `8643C0`/`8643F0`/`864382`. This is the single highest-value
    target by TU leverage and it is *not* decode-only — a narrow or floating load
    needs its own instruction.
  * **`assign-dst-not-formal` ×3** — measured as member/global stores, so this
    wants a real store with a relocation, not more decode.
  * **`body-0x29` / `expr-op-0x3A`** — labels and branches, the control-flow layer.
    `Primes.cpp` and `Sort.cpp`, the two nearest loop functions, both stop here.

So demand-driven ordering and TU-match leverage have now **diverged**: the biggest
bucket (`expr-call-in-expr`, 304,813) is not what the nearest TUs want. The
one-away list is the better instrument at this point, and every item on it needs
codegen rather than decode — which is the review's asymptotic-stall argument
showing up as a concrete change in what the next rung has to be.

**And the histogram itself was misleading (`docs/IL_LOAD_TYPES.md`, 2026-07-30).**
The `expr-load-type-XXXXXX` key is not stable: it truncates the LEB type id to one
byte, and derived-type ids are allocated per translation unit from 0x1000, so one
construct lands in different buckets in different TUs. Regrouped by family:

| family | blocked functions | what it is |
|---|---:|---|
| `A643xx` | **750,421** | const-pointer / `this` loads |
| `8643xx` | **294,810** | data-pointer loads |

**~44% of all blocked functions sit behind a pointer-typed load** — larger than
`expr-call-in-expr`, and larger than anything the histogram has ever shown, because
the instrument was splitting the biggest class into hundreds of shards. `GAPS.md` §6
now carries the general form of that mistake.

The ranked consequence, from that report's own estimate and to be verified rather
than trusted: admitting **data-pointer (kind 3) and function-pointer (kind 4)
width-4 TYPEs** through the already-in-class indirect-load leaf and identity shapes
needs *zero new instructions* — a pointer getter is `lwz r3,off(r3); blr`,
byte-identical in scheme to the int getter it already emits, and `2C` ptr→ptr emits
nothing. A full-body shape scan over 128,081 bodies (5.2% of the corpus) found 6,174
complete bodies of exactly those shapes, extrapolating to **~118k functions fully in
class**, which would roughly double the census. After it: `lbz` bool/uchar getters
(~31k), then `lfs`/`lfd` getters (~18k).

Note the distinction that has made estimates here wrong before: the `float` (93,189)
and `double` (79,542) buckets proper are FP-**arithmetic** bodies, not getters, and
will behave the way W13b did — about +1 — until real FP codegen exists. "A load of
type T" and "arithmetic in type T" are different rungs.

#### The ~118k estimate, adjudicated: **+88,116, essentially dead on** (2026-07-30)

The pointer-load rung above was implemented, reviewed and graded. Measured across
all 878 TUs: census **122,487 → 210,603, +88,116**, with **832 TUs gaining, 0
losing, and mismatch 0**. Against an estimate of "~118k functions fully in class"
that was explicitly labelled an upper bound, this is the closest any estimate in
this document has come to its outcome, and the estimate's own stated reason for
being an upper bound (the scan did not verify the base binds to a formal or
`this`) is the right size to explain the rest.

It is also byte-exact where it matters, not merely admitted: a 10-function cross
of `int`/`char`/`long long` values against `int*`/`const int*`/`char*`/`void*`/
`int**` plus two pointer identities is `Port=Match`, where mainline admits 3 of
the 10.

**A false correction stood here for one commit and is worth keeping visible.** An
earlier revision of this section reported the rung as **+0** and built an argument
on top of that: the `expr-load-type-*` key really is emitted by the `parse_expr`
**operand** gate (`body/expr.rs:309`) and not by the load leaf the rung widened,
so the reasoning looked sound. The number was wrong. Both the main repo and the
rung's worktree contained a `work/dc3-workload/scan-t1.jsonl`, and a *relative*
path read the wrong one — reflinked worktrees mean identical relative paths hold
different data, and which one you get depends on the working directory a shell
tool happened to be left in. The failure mode is worth naming because this repo's
whole parallel-agent workflow rests on those worktrees: **quote absolute paths for
every measurement artifact, and check the row count and denominator of a scan
before differencing it against another.** Two scans of the same corpus agreeing on
`fn_total` proves nothing about which binary produced them.

The observation about `expr.rs:309` survives its wrong conclusion and still bounds
what remains: the pointer *operand* gate is untouched, so a pointer feeding
arithmetic or a call is still refused, and widening it is not decode-only — a
pointer in an add chain needs element-size scaling. That, and `.sy` binding on real
TUs (`param-width-undetermined`, 567,549), are the next two rungs.

> **§6a–§6m are frozen.** They are the historical rung ledger and every
> cross-reference in the repo points at these letters, so nothing here is
> renumbered or moved (`docs/ARCHITECTURE_SEAMS.md` §9.6 — history does not
> conflict, only growth does). **New rungs land in `docs/rungs/<date>-<slug>.md`**,
> one file per rung, indexed by `docs/rungs/INDEX.md` and registered by
> `crates/c2-harness/tests/rung_registry.rs`: section letters and `W`-numbers
> allocated concurrently collide silently (§6e/§6f/§6g/§6i and the tag `W23`
> were each claimed twice on 2026-07-30), and filenames collide as add/add
> conflicts git flags loudly.

## 6a. The frame audit, 2026-07-30 (D6) — the next rung is the general frame

**The strategic answer, MEASURED.** `docs/IL_CALL_IN_EXPR.md` §18 asked whether the
port is out of leaf-shaped work. Over the **2,182,551 blocked functions**, counting
CALL tokens per body (a decode-only measure that runs *outside* the grammar, so an
undecoded row is priced like any other):

| | functions | share of blocked |
|---|---:|---:|
| `calls-0` — provably no frame | 626,398 | 28.7 % |
| `calls-1` — a tail call, or a small frame | 753,498 | 34.5 % |
| `calls-2plus` — **provably a frame** | **802,655** | **36.8 %** |

36.8 % is an exact lower bound. Pricing the middle class from the code c2 actually
emits — 178,969 emitted functions across 871 workload objs, read straight off `.text`
(framed iff it saves LR or moves r1): `calls-1` is **42.8 % framed**, `calls-2plus` is
98.3 %, and the corpus as a whole is **42.3 % framed**. Applying that gives a point
estimate of **≈ 1,129,500 blocked functions, 51.8 %** — labelled an estimate, because
the split is measured on emitted code and applied to a population that is mostly not
emitted (§18.6).

**Every large `expr-call-in-expr` row is 96–100 % framed** — `chained` 45,663 (100 %),
`op-0x9B` 39,361 (100 %), `recv-object` 96.0 %, `recv-load` 99.7 %, `recv-field`
90.3 %, `recv-intrinsic` 85.7 % — which is ~199,000 framed functions in that bucket
alone. **They are not three rungs; they are one rung's first three customers.** So
item 5 of §5's phase list (frames/spills + per-function `.pdata`, W-UNW-1) moves ahead
of further grammar rungs, and its content is now measured rather than sketched:
variable frame size (96 B for one by-value temporary, 112 B for two), LR save/restore,
callee-saved GPRs allocated descending (`std r31,-16(r1)`, `std r30,-24(r1)`), a
frame-slot allocator for by-value temporaries (`stw r3,80(r1) ; addi r3,r1,80`), and a
**`.pdata` entry per framed function** with prolog lengths of 3, 4 **and 5** in one
probe TU — where `coff::build_pdata` hardcodes 3 — and no entry at all for a leaf.
Per-COMDAT, because the workload's `/O1` implies `/Gy` and `PortC2::build` refuses a
framed call there by name today.

**Two rows the hypothesis got wrong, and they are the two biggest names on the
board.** §3's G5 tables call `expr-intrinsic-base-member-addr` (2117) and
`expr-intrinsic-this-adjust` (2113) "mostly non-leaf bodies". Measured: 2113 is
**59.2 %** framed, and 2117 is **28.0 %** — **32,372 of its 122,949 issue no call at
all**. Captured, they are `lwz r3,4(r3) ; blr` / `lwz ; lwz ; add ; blr` /
`stw r4,4(r3) ; blr`: two to four instructions, no frame, no `.pdata`. That is the
largest genuinely leaf-shaped block left in the census and the one rung still takeable
*before* the frame. With `data-addr-1sym` (2,712, 100 % `calls-1`) and
`recv-object × type-ptr` (2,410, 100 % `calls-1`) it is the entire remaining local
inventory above a thousand functions: **37,494 functions, against 802,655.**

> **CORRECTION 2026-07-30 — that last sentence is false, and it was false when it
> was written.** It was computed from a histogram whose operand-type key embedded a
> per-TU type id and so split one construct across 256 names (`GAPS.md` §6). With
> the key de-sharded, the call-free blocked inventory is **585,777 functions**, of
> which **381,810 are a single construct** — a 4-byte data-pointer operand,
> `expr-load-type-A643` (298,770 `calls-0`) and `-8643` (83,040). The frame rung is
> still the biggest piece of work; the enumeration of what is local was wrong by
> 10×. `docs/IL_CALL_IN_EXPR.md` §20 has the corrected ranking, the exact-partition
> check, and the next rung (+14,038, measured by counterfactual).

**Bundled measurement-integrity fix.** §16.4's chain undercount is closed: 37,662
functions re-key from `recv-load`/`recv-deref`/`recv-intrinsic`/`recv-field`/
`recv-call`/`recv-object` into `chained`, an exact partition with the
`expr-call-in-expr` total (268,140) and the non-`mcall` keys (1,914,411) unchanged to
the function. The chain population is **8,001 → 45,663, a 5.71× undercount** rather
than the 4.4× §16.4 estimated. Acceptance untouched; census stays 280,020 / 2,462,571
= 11.37 %, mismatch 0.

**One thing this rung found and did not close, recorded because it bounds every
census number above.** `src/lazer/meta_ham/HamUI.cpp` has **9,551 function bodies in
its `.ex` and c2 emits 350 functions**; corpus-wide it is 2,462,571 IL bodies against
**178,969 emitted, 7.3 %**. The census denominator is IL bodies, which is the right
denominator for the port's all-or-nothing per-TU gate — but it is not the denominator
of the emitted code, and the two should never be mixed without saying so. The port
**fails closed** on the gap (`bundle.rs`'s `bound.len() != segs.len()`; probe
`work/fa/probes/p5.cpp` censuses 2/2 in class, the reference emits one function, the
port returns `NotImplemented`), but incidentally rather than by design.

## 6b. Independent review, 2026-07-30 (HEAD `2724ca5`)

An adversarial re-measure of every headline claim, by a session that did not
write any of the code. Every number below is from a command run that day, not
from this document.

**Confirmed by measurement:**

- Mode lanes (`scripts/mode_lane.sh`, 90 fixtures): `/Ox` **32** match, `/O1`
  **28**, `/O2` **28**, `/Ox /Gy` **28** — **0 mismatch in all four lanes**.
- Real workload (`c2rs gap`, 878 TUs, 41.4 s): **match 6, mismatch 0,
  codegen-gap 0, vocab-gap 865, capture-fail 7**; census **109,501 /
  2,462,571 (4.45%)**.
- Fail-closed: 0 mismatch across all of the above **plus** the generated
  sweep (`scripts/expr_sweep.sh`: 2,589 cases, 0 mismatch) and a green
  `cargo test --workspace`. No known wrong-bytes emit at HEAD.
- Nine TUs are exactly one function from matching (scan JSONL,
  `fn_total - fn_in_class == 1`). **Correction to the working claim**: their
  blockers are `body-0x53` ×4 and `assign-dst-not-formal` ×3 — but also
  `expr-call-in-expr` ×1 (`Main.cpp`) and `expr-load-type-A64381` ×1
  (`xboxheap.cpp`). Two features cover seven of the nine, not all nine.

**Strategy verdict.** The decode/statement-layer turn is the right one, and
the evidence is this repo's own ledger: the histogram-driven rungs (R2, the
statement layer, `/O1` support) bought the census jumps (0.32→3.17%,
3.24→4.45%) while four consecutive staged-fixture codegen rungs bought ≈0
(§G5, `GAPS.md` §2b). The grammar should be driven to parse-complete on the
workload *ahead* of acceptance — every "bucket was the instrument" episode
(three so far) and every mis-attributed rung was a cost of scheduling against
a partially decoded corpus. But grammar work must stay tied to acceptance
attempts the way it is now (decode gated by captures, acceptance gated by
byte-exact emission): a parse-everything pass with no emission witnesses
would entrench wrong decodings with nothing to falsify them. The
counter-argument taken seriously: TU-level match — the metric the payoff
contract pays on — has been flat at 6/878 through a ~30k-function census
gain, and the nine one-away TUs say the fastest route to real matches is
*acceptance* work on exactly two features, not more decoding. Both are true;
the resolution is that locals (`.sy`) and `body-0x53` are simultaneously the
top of the decode ranking and the blockers on the one-away TUs, so the two
orderings currently agree. Revisit when they stop agreeing.

**Honest distance to parity (ordered; ⚠ = size unknown because the format or
the rule is uncharacterized):**

1. Locals — a positive local signal (`.sy`) so `assign-dst-not-formal` can
   admit stores to locals without risking the static-store mis-emit.
2. Statement grammar past the leading expression: `body-0x53` compounds,
   `body-0x29`, `body-0x9B`, early returns.
3. ⚠ Control flow (W8): branch tokens, block layout order, compare/branch
   fusion — forces the block IR and the first real scheduling exposure.
4. ⚠ General register allocation + scheduling. Every accepted class so far
   *caps itself below* c2's allocator (r9 floor, 4-term chains, depth-2
   trees, one FP constant). Parity means modeling the allocator and the
   scheduler in general, on both `/Ox` and `/O1`; nothing measured yet bounds
   that work, and it is the single largest unknown on the board.
5. Frames/spills + per-function `.pdata` counters (W-UNW-1).
6. Generalized calls (W11), then member/indirect calls — needs `this`/vtable
   and memory addressing (W12); `.sy` becomes load-bearing.
7. ⚠ Intrinsic *acceptance* — per-id, argument-literal-constrained allow-list;
   the id table cannot be enumerated from IL.
8. Data sections, globals, strings (W14); 2+ FP constants (needs c2's
   constant evaluator *and* scheduler); switch tables, 64-bit, unsigned long
   tail.
9. ⚠ The `/EHsc`/`/Oi`/inlining flag regime on real TUs — the bundle does not
   record everything the obj depends on (`/Gy` proved it), and EH scaffolding
   on otherwise in-class bodies is unprobed.
10. The front end (Track D) — off the match-bucket critical path but gating
    the composition milestone and the >2.4× downstream regime.

Items 3, 4, 7 and 9 are the reason no completion fraction can honestly be
derived from 4.45%: the census measures decode demand, and none of those four
is a decode item.

## 6c. The census/gate disagreement, sized and closed (D13, 2026-07-30)

Roadmap item #44. The invariant is that acceptance lives in the IL parser so
`IlBundle::function_census` — the public coverage numerator — and `PortC2` cannot
disagree about what is in class. It was known to be violated
(`IL_CALL_IN_EXPR.md` §24.7) and had never been sized.

### The instrument

`IlBundle::census_functions` pairs every census row with the emitter's own
function record, built by `shape_to_function`, which `IlBundle::functions` also
calls — one locator for the shape→function mapping. `codegen::function_gate` runs
`PortC2`'s per-function selector, dispatched by the new `codegen::select_function`
that both the packed and the COMDAT emitters now go through, so the cross-check
cannot drift from the emitter. `c2rs gap` and `c2rs census` print the
disagreement **in the same block as the numerator, every run**.

### The size, and what it was actually made of

**9,230 of 411,934 — 2.24 % of the numerator**, on the 878-TU workload at `/O1`.

| functions | cause | where the gate was |
|---:|---|---|
| 9,028 | a generated empty destructor whose callee token has no `.gl` symbol | `shape_to_function` (per function, invisible to the census) |
| 202 | an optimization word the port does not emit under (`00200001` ×136, `00200101` ×66) | `PortC2::build` |
| 0 | `return a + b*c` — the shape §24.7 characterized | `codegen::select_text` |

Plus **14 in the fixture corpus** that the workload does not contain: the §24.7
depth rule (9), `==`/`!=` against a large unsigned (4), FP scratch exhaustion (1).

**The characterized case was 0 % of the real total.** §24.7 was written from three
probes and named the straight-line depth rule; the workload has 62,813
straight-line functions and not one whose operand stack goes past depth 2. Every
one of the 9,230 was a cause nobody had looked for. `GAPS.md` §6 carries the
general form.

### What moved, and where to

Four gates, each with its own census key on the way in:

| gate | now lives in | key |
|---|---|---|
| callee resolves through `.gl` | `census_functions` (post-parse) | `callee-unresolved-{tail-call,dtor-delegation,framed-call}:eof` |
| the optimization word names a mode | `census_functions` (post-parse) + `c2_il::opt_word_mode` | `opt-mode-<word>` |
| serial-chain depth / the depth-2 tree shape | `c2_il::chain_form`, gated in `straight_line_out_of_class_ctx`, consulted by `select_text` | `expr-out-of-class-tree-depth` |
| the difference spine's three literal rules | `CompareLeaf::out_of_class_ctx` | (falls through to `expr-cmp-*`) |

Both census-side gates are applied **last, to an otherwise-in-class function
only**. Gating them up front would relabel every blocked function whose real
problem is elsewhere and destroy the histogram that ranks this roadmap — which is
the objection `IlBundle::opt_words`' doc comment raised against gating the mode
at all, and it is answered by the ordering rather than by leaving the numerator
wrong.

Two of the four were already spelled out **twice** (the comparison leaf's
wide-literal and `i16::MIN` rules, in the parser and in `compare_leaf_text`), and
the third rule of that same family was in codegen alone — which is the one that
leaked. Partial duplication is worse than none: it makes the seam look handled.

### The corrected census

**411,934 → 402,704 (16.73 % → 16.35 %), −9,230.** It is meant to go down. The
old number counted functions the port refuses; a numerator with an unmeasured
error term is not a benchmark.

The accounting is exact: the 9,230 appear under three new keys
(`callee-unresolved-dtor-delegation:eof` 9,028, `opt-mode-00200001` 136,
`opt-mode-00200101` 66) with **zero** movement in any pre-existing key, **zero**
TUs changing class, and mismatch 0. Residual disagreement: **0** on the workload,
**1** on the fixtures.

### Gate evidence (this change)

| lane | result |
|---|---|
| `cargo test --workspace --release` | 366 passed, 0 failed |
| `c2rs bench` | **123 pass, 0 fail, 0 error** (121 before; +2 fixtures) |
| `scripts/mode_lane.sh /Ox` | 51 match, **0 mismatch**, 0 codegen-gap |
| `scripts/mode_lane.sh /O1` / `/O2` / `/Ox /Gy` | 48 match, **0 mismatch**, 3 codegen-gap each |
| `scripts/expr_sweep.sh` | checked=**4343**, mismatches=**0** |
| 878-TU scan | match 6, **mismatch 0**, census 402,704/2,462,571, 584 keys, disagreement **0** |
| `census fixtures/cpp/w21_census_gate.cpp` | **9/9 in class**, `Port=Match` |
| `census fixtures/cpp/w21_census_gate_neg.cpp` | **0/8 in class**, `Port=NotImplemented`, no disagreement |

`crates/c2-harness/tests/census_gate.rs` runs the cross-check over the whole
fixture corpus on every `cargo test` and asserts the disagreement equals its
recorded value, so a gate landing in codegen instead of the parser fails a test.

### Found and not taken, ranked, with the frame axis applied

1. **`callee-unresolved-dtor-delegation:eof` — 9,028 functions, all `calls-1`,
   all grammar-complete** (`:eof`). Almost certainly a **gate-side decode gap
   rather than a real out-of-class construct**: all 826 TUs that have one *also*
   have resolved delegations (26,918 of them are in class right now), so it is
   not a property of the shape — `gl_symbol_index` is missing specific tokens.
   Recovering them is a `.gl` **binding** change, and §6's `.sy` bullet is the
   governing precedent: the oracle cannot grade a correspondence, so it must be
   graded on the binding's own invariants, not on a green differential. This is
   the largest single named item on the board that needs no new instruction.
   > **TAKEN — §6e (D14).** The prior was right and the mechanism was one byte:
   > `.gl` has a **second record separator**, `26`, that the NUL-anchored name
   > scan could not see. **+9,027** (the 9,028, less one function the new
   > fail-closed ambiguity rule costs), 1:1 into the three `empty-dtor-*` shapes.
2. **`opt-mode-00200001` — 202 functions in 2 TUs** (`HamRibbon.cpp`,
   `Ribbon.cpp`; 136 `calls-0` + 66 with the ctor/dtor bit). `00200001` is
   `/O1`'s `00200005` without bit `0x4`, and `docs/OPT_MODE.md` explicitly leaves
   that bit unexplained. One controlled capture would settle whether it is a mode
   the existing codegen already targets. Small, but it is the *whole* of the
   second cause.
3. **FP scratch exhaustion — 1 fixture function, 0 workload functions.**
   `w13_fscratch.cpp`'s `fm13` (13 `float` parameters, 12 multiplies) is refused
   because `float_leaf_text` never retires a parameter from its live set, so 13
   parameters leave exactly one free pool slot. The fixture's own comment says
   the cursor should "wrap twice", so the model and the code disagree — and the
   fixture has never graded it, because its TU has four out-of-class siblings.
   Moving this gate means lifting the FP register allocator into `c2-il`, which
   is byte-visible; not worth it at a measured cost of 0 workload functions, but
   it is a *second* instance of §6's "a fixture that states the rule and carries
   the failing case can still grade nothing".
4. **`eat_int_like`'s four-triple whitelist (roadmap #43), 5,684** — untouched
   here. Its caution stands: that number is a decode ceiling attributed from key
   names, not a whole-body-completeness measurement, and the locator is shared by
   three graded shapes.

## 6d. W22 — the int-like operand type by spelling (roadmap #43, 2026-07-30)

`eat_int_like` matched an exact four-triple whitelist (`86 41 74` int,
`86 42 75` unsigned, `86 41 12` long, `86 42 22` unsigned long). A width-4
integer carrying a **per-TU type id** — an `enum`, a `typedef`, a `const` or
`volatile` qualification — has a different third byte and refused, even though
`is_int4_type` admits it on the tag/kind nibbles and c2 emits the identical
instruction. It now falls through to that predicate.

### Re-measured before building, and the recorded estimate was wrong by 2.8×

`IL_CALL_IN_EXPR.md` §24.6 recorded the over-refusal as **5,684**, with the
caution that it is "a decode ceiling attributed from key names, not a whole-body
-completeness measurement" and that the takeable number should be **smaller**.
A counterfactual scan says otherwise:

| | functions |
|---|---:|
| released (blocked keys that fell) | 20,779 |
| advanced to a different blocker | 4,855 |
| **whole bodies gained** | **+15,924** |

The bias direction was called wrong, and the reason is `GAPS.md` §6's own rule
about estimating the fix rather than the finding: **`eat_int_like` has five call
sites**, the key-name estimate covered the attribution of three
(`8642`/`A641`/`A642`), and the realized yield is dominated by a *different*
row — `expr-op-0x27` fell **12,637**, an ordinary member getter whose member is
an `enum`. `expr-load-type-8641` (5,221) and `expr-lit-type-8641` (1,936) went to
**0**. This is the third time a single-site estimate has under-counted a
multi-site fix; the rule now has a corollary: **when the estimate comes from key
names, the sites that fix are not the sites that were counted.**

### Census

**402,704 → 418,628 (16.35 % → 17.00 %)**, mismatch 0, no TU changing class,
census/gate disagreement still **0** (the port accepts every function admitted —
checked by the D13 cross-check, which is what makes "the decode widened" and "the
emitter agrees" one measurement instead of two).

### Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | 366 passed, 0 failed |
| `c2rs bench` | **125 pass, 0 fail, 0 error** |
| `scripts/mode_lane.sh /Ox` | 52 match, **0 mismatch** |
| `/O1`, `/O2`, `/Ox /Gy` | 49 match, **0 mismatch**, 3 codegen-gap each |
| `scripts/expr_sweep.sh` | checked=**4343**, mismatches=**0** |
| 878-TU scan | match 6, **mismatch 0**, 418,628/2,462,571, 569 keys |
| `census fixtures/cpp/w22_int_spelling.cpp` | **13/13 in class**, `Port=Match` |
| `census fixtures/cpp/w22_int_spelling_neg.cpp` | **0/7 in class**, `Port=NotImplemented` |

`w22_int_spelling.cpp` grades the widening across all three shapes the locator is
shared by — the member getter (`27` + `30`), the identity leaf and the `41`
result annotation — plus arithmetic, over `enum` (both signed and unsigned
underlying), `typedef`, `const` and `volatile` spellings.

`w22_int_spelling_neg.cpp` holds what must keep refusing, and it is the more
load-bearing file: `is_int4_type` requires the tag's width nibble to say **4-byte
alignment** *and* the kind's high nibble to say **4-byte size**, so the narrow
typedefs, `long long`, and a 4-byte `int` under `#pragma pack(1)` all refuse. The
`pack(4)` `long long` member in it is the discriminating case for `GAPS.md` §6's
third wrong-bytes emit (tag carries alignment, kind carries size — equal for
every naturally-aligned type), and the corpus had never carried it as a fixture.

## 6e. W-UNW-1 closed: per-function `.pdata` (2026-07-30)

The prerequisite for #35 (general non-leaf lowering) was that the port emit
byte-exact unwind data for framed functions. It does now, in both sectioning
modes, and the label counter that blocked it is derived rather than pinned.

**What was established from c2's own output** (`OBJ_FORMAT_MVP.md` §7): the X360
record is 8 bytes, `BeginAddress` (reloc-patched, addend 0) plus a packed word
`PrologLen[7:0] | FuncLen[29:8] | ThirtyTwoBit[30] | ExceptionFlag[31]`, both
big-endian. There is **no `.xdata` and no unwind-code array** — the whole thing
is those 8 bytes, which is not the x64 shape and was not assumed from it.
`FuncLen` and `PrologLen` are exactly the two `$M` label values over four.
Prologue lengths of 3, 5, 6 and 7 all occur in ordinary code; the old emitter
hardcoded 3. A leaf gets no record at all — including a leaf with a 400-byte
local array, which lives in the red zone; grow it to 70,000 bytes and the record
appears, so the predicate is "does this function move `r1`", which the emitter
knows by construction.

**What was implemented.** `Function::frame` carries the two lengths; `emit_obj`
appends one `.pdata` holding every framed function's record with an ADDR32 per
record, and `emit_comdat_obj` gives each framed function its own `.pdata` COMDAT
immediately after its `.text` COMDAT, `SELECT_ASSOCIATIVE` with the aux `Number`
naming that `.text`. The third whole-obj emitter (`emit_framed_obj`, hardcoded to
one function and to `$M2545/$M2546/$T2547`) is deleted — this file already
carried two bugs whose whole cause was one rule implemented in two emitters and
fixed in one.

**Where the gate lives.** A framed function may not share a TU with a
comparison or floating-point leaf, because those consume 3 and 2 label slots
against the 1 every emitted class consumes. That is a TU-level acceptance
question, so under §6c's invariant it sits in `c2_il::IlBundle::functions`
beside the other TU-level gates, not in codegen — and `function_gate` lost its
`Selected::Framed if fn_level_linking` arm, which after this rung refused what
the emitter emits and would have made the disagreement counter wrong in the
*under*-claiming direction.

**Gate evidence, on the merged tree** (c2-rs `ae0467b`, workload tree
`dc3-decomp` at `05ca6d09`, both scans taken against that same corpus HEAD):
130 fixtures × 4 mode lanes, **0 mismatch** everywhere — `/Ox` 56 match (master
52), `/O1` 54 (49), `/O2` 54 (49), `/Ox /Gy` 54 (49). `c2rs bench` 130 pass / 0
fail / 0 error; `cargo test --workspace` 370 pass. 4,706 generated sweep cases,
**0 mismatches**; the 363 new W-UNW cases graded separately in all four modes at
342 match / 0 mismatch / 12 honest refusals each. 878-TU scan: match 6,
**mismatch 0**, capture-fail 7, replay 36/36, **census/gate disagreement 0**.
Census **418,628 / 2,462,571 = 17.00 % before and after, 0 TUs changed** — this
is groundwork and it moved the census by exactly zero, as intended. The
fixture-lane disagreement fell 11 → 9 (`/O1`) and 10 → 9 (`/Ox /Gy`); the
remainder is master's pooled-FP-constant-under-`/Gy` refusal, untouched here.

**One live wrong-bytes emit found and fixed**, unreachable before the
single-function gate came off: the framed `bl` displacement was the literal
`4BFFFFF5`, right only for a function at `.text` 0.

### What still blocks #35

Everything below is *codegen*, not obj format. The unwind side is done for any
frame the codegen can describe, because `Frame { prolog_len, func_len }` is all
the record needs and both are byte offsets the emitter already computes.

1. ~~**The frame itself is one hardcoded shape.**~~ **CLOSED 2026-07-30 — see
   §6f.** The model is measured and implemented
   (`c2_core::codegen::FrameLayout`); the three helper shapes are refused by
   name with their thresholds pinned by paired captures. Two of this item's own
   premises were wrong: "96 B for one by-value temporary, 112 for two" is really
   the *callee-saved register count*, and `_RtlCheckStack12` does not arrive
   "past a page" — inline `ld` probes cover the first four and the call starts
   at five.
2. **More than one call per body.** The accepted shape is one `bl` with the
   result consumed by one `addi`. Every large blocking row is 96–100 % framed
   *and* multi-call, so #35's customers all need call sequencing, argument
   registers r3–r10 with stack spill, and a live-range model across calls.
3. **The label stride of every class #35 will admit.** Partly closed (§6g): the
   comparison leaf's stride is now measured per relation over the whole 60-point
   grid and the gate asks one three-valued predicate
   (`IlFunction::label_slots`), so an unmeasured class refuses rather than
   defaulting. What remains: the FP leaf (2, or 4/6 with pooled constants) and
   — the live risk for step 2 — a framed function using the `__savegprlr_N`
   pair, whose `/Gy` stride is **7, not 5**, with the two extra slots allocated
   *before* its own `$M` pair (`CODEGEN_FRAMED_CALLS.md` §4.4). The helper
   codegen and that correction must land together. Any new class needs one
   compile, `<class> ; int F(int a){return g(a)+1;}`, differenced against
   `.gl+7+9`.
4. **EH is out of class and visibly so.** Bit 31 of the unwind word, and a
   function with a `try`/`catch` produces **several** records (the catch funclet
   first, with a non-zero `BeginAddress` addend). The workload compiles
   `/EHsc`, so this arrives with the first real body that needs it.
5. **`.pdata` beside `.rdata`.** No captured TU has both a pooled FP constant
   and a framed function, so the section order between them is unknown; the
   combination is refused rather than guessed.
6. Unchanged and unrelated to unwind: the whole-TU all-or-nothing gate, and the
   fact that the port has no model of *which* bodies c2 emits (§6a).

## 6f. D14 — the `.gl` record form the symbol index could not see (2026-07-30)

Roadmap item from §6c's ranked handoff, #1: `callee-unresolved-dtor-delegation:eof`,
**9,028 functions in 826 TUs**, all `calls-1`, all grammar-complete. A generated
empty destructor whose delegation callee has no `.gl` symbol, so `shape_to_function`
refuses. The prior was that this is a decode/binding gap rather than an out-of-class
construct, because all 826 of those TUs *also* hold resolved delegations.

### What the tokens actually denote, and how that was established

They denote exactly what a resolving one denotes — a `.gl` symbol record — in a
**second record form the index could not reach**. A record is

```text
80 <LE32 type id>  <2 bytes>  <kind>  <operand token>  <SEP>  <name> 00  <TYPE> …
```

and `SEP` takes two values. Transcribed from `src/system/jpeg/Jpeg.cpp`, two
adjacent records of the same class, same `04` kind byte, byte-identical framing on
both sides:

```text
80 75 14 00 00  00 00  04  84 30  00  ??YString@@QAAAAV0@PBD@Z  00 86 03 04 04 …
80 85 14 00 00  00 00  04  c2 30  26  ??_GString@@UAAPAXI@Z     00 86 03 04 04 …
                           \_tok/ \sep/
```

That identity is the argument, and it is a **container** argument rather than an
oracle one: the token field's position is fixed by the framing, so if the two bytes
are the operand token in the `00` form — which 2,323 of 2,323 resolving call sites
already say — they are the operand token in the `26` form too. `gl_symbol_index`
anchored on "a name is the run right after a NUL", which cannot see past a
separator that is not NUL; anchoring on the separator is what recovers them.

Measured over eight real TUs: the byte before a `?`-mangled name takes exactly
**two** values, `00` (20,336) and `26` (12,505), and nothing else. What `26`
*means* is deliberately not named — every witness carrying it is a
compiler-generated or header-inline symbol (`??_G`, `??_E`, `??_7`, `??_R*`,
`_CT`/`_TI`, and `??1logic_error@stlpmtx_std@@UAA@XZ`) while an out-of-line
`??1String@@UAA@XZ` carries `00`, but that is a correlation over one corpus and
`GAPS.md` §6's rule against guessed names applies. Nothing branches on the value.

### The wrong-but-green failure mode, stated and then closed

**A green differential cannot grade a correspondence** (`GAPS.md` §6, the `.sy`
bullet). For this binding to be wrong-but-green, the two bytes before the `26`
separator would have to be something other than the operand token — a type id, a
vftable slot, a neighbouring symbol's token — while still producing names that
happen to be right wherever an obj was compared. Four measurements close it, none
of which is an obj compare:

1. **Framing identity.** The field is at the same offset in both forms, in records
   whose every other byte agrees. A different field would have to occupy the same
   position in the same record layout.
2. **The shape's own semantics, over the whole workload.** A generated empty
   destructor delegates to a sub-object's destructor, so its callee is a destructor
   by construction of the *source*. All **35,946** in-class generated destructors on
   the 878-TU workload resolve to a `??1` mangling; `??_G`, `??_E`, `??_D` and
   "something else" are **0**. A misread field would produce arbitrary symbols.
3. **Injectivity.** `.gl` assigns one token per symbol. Tokens two records disagree
   about are **dropped**, not resolved to the first — the third value that refuses.
   The workload's residual is 7, all in one record form this reader does not model
   (`$…$initializer$` local statics), and their measured cost is **0 functions**.
4. **The counterfactual.** Running the identical binary with `26` removed from the
   separator set and everything else in place gains **0** functions. So the whole
   +9,028 is this record form and nothing else in the rewrite.

What is **not** closed, and is stated rather than implied: no fixture grades a
`26`-form binding through the emitter, because the form could not be reproduced in
a controlled TU (eleven probes over virtual, inline, template, `throw()` and
EH-referenced destructors all produced `00`). The obj-level evidence for this rung
covers the *rewrite* — every other rule changed here — and the `26` form rests on
the four measurements above.

### The rewrite the recovery needed, and what it cost

Anchoring on the separator is not enough on its own, because a record's token bytes
are frequently printable and run together with its name: `c2 30 26 ??_GString@@…`
reads as one graphic run, `0&??_GString@@…`. Four rules, each measured:

| rule | why | measured |
|---|---|---|
| the name is the **rightmost** separator-preceded start of its run | leftmost glues the record's own token bytes onto the name — which the old scan was doing whenever a kind byte was `00` (`b[&??_R0?AVFixedString@@@8`, 5 live entries in `Memory_Xbox.cpp` alone) | — |
| the record **kind** must be one of `00 04 0E 10` | `.gl`'s type table puts a type id where the token is | exactly this set over 32,898 `?`-mangled records |
| the name is spelled in `[A-Za-z0-9_$?@]` | separates a symbol from a path or a template-id | — |
| a whole mangled name **outranks** a bare one on the same token; equal rank disagreeing → dropped | the type table's residue is bare, and a bare run is never a callee | 44 collisions → 0 |

Cost of the rewrite, measured over eight real TUs (24,281 index entries before,
34,208 after): **zero** `?`-mangled bindings change name, **zero** are lost except
one that was itself a junk read, and **zero** conflicts involve a mangled name. On
the workload it costs exactly **one** function (`callee-unresolved-tail-call:eof`
0 → 1) and gains none — that is what the counterfactual scan measures.

### The estimate

Recorded before building: **+9,028, biased low**, cause named — tokens whose record
carries a separator or kind outside the measured sets, plus the newly-dropped
ambiguous tokens. Realized: **+9,027**. The single missing function is exactly the
named cause, and is at the instrument's one-function noise floor.

### Census

**418,628 → 427,655 (17.00 % → 17.37 %)**, mismatch 0, no TU changing class,
census/gate disagreement still **0**, 569 keys → 569.

The accounting is exact and 1:1: `callee-unresolved-dtor-delegation:eof` **9,028 →
0**, landing in `empty-dtor-delegation` (+7,196), `empty-dtor-member` (+1,694) and
`empty-dtor-member-adjusted` (+138) — 9,028 — against `callee-unresolved-tail-call:eof`
+1 and `multiarg-tail-call` −1. **No other key moved at all.** All of it is
`calls-1`, as §18's frame axis requires for this shape.

### Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | 372 passed, 0 failed |
| `c2rs bench` | **127 pass, 0 fail, 0 error** |
| `scripts/mode_lane.sh /Ox` | 53 match, **0 mismatch**, 0 codegen-gap |
| `/O1`, `/O2`, `/Ox /Gy` | 50 match, **0 mismatch**, 3 codegen-gap each |
| `scripts/expr_sweep.sh` | checked=**4367**, mismatches=**0** |
| 878-TU scan | match 6, **mismatch 0**, 427,655/2,462,571, 569 keys, census/gate disagreement **0**, `.gl` binding violations **0** |
| `census fixtures/cpp/w23_gl_callee_bind.cpp` | **5/5 in class**, `Port=Match` |
| `census fixtures/cpp/w23_gl_callee_bind_neg.cpp` | 3/3 in class, `Port=NotImplemented` (two TU-level gates) |

Two invariants now print on **every** `gap` and `census` run, next to the numerator,
for the same reason the census/gate disagreement does: the count of tokens `.gl`
claims twice, and the mangling class every generated destructor's callee resolves
to. A binding cannot be graded by the oracle, so the grader has to be permanent.

### Found and not taken, ranked, with the frame axis applied

1. **`expr-intrinsic-this-adjust` — 141,800 functions, of which 83,891 are
   `calls-2plus`** and therefore need a frame. The takeable half is the ~57,909
   `calls-0`/`calls-1` remainder, and it has no `-whole` bit, so `GAPS.md` §6's rule
   applies: spend one counterfactual scan before scheduling against it.
2. **The census's per-function `name` is bound POSITIONALLY** — `census_functions`
   zips `mangled_names` onto segments when the counts happen to be equal, and
   `mangled_names` drops every `??`-prefixed name, so on
   `fixtures/cpp/w23_gl_callee_bind.cpp` the destructors are reported under the
   *tail calls'* names. Diagnostic-only today, but the **varargs gate reads the same
   positional name**, so a TU where the wrong name ends `ZZ` would refuse a
   non-variadic function or admit a variadic one. `gl_defined_names` is the correct
   locator and is already in the crate — this is roadmap #14's exact shape, one seam
   over.
3. **TU-level gates are outside the census/gate cross-check.** A function-template
   callee makes c2 splice `/alternatename:…` into `.drectve`, so
   `drectve_is_boilerplate` refuses the TU while the census reads 1/1 in class
   (`w23_gl_callee_bind_neg.cpp` holds it). §6c's cross-check runs per function and
   cannot see it; the numerator's error term is therefore still an upper bound by
   the TU-level gates, unmeasured.
4. **The `$…$initializer$` record form**, 7 ambiguous tokens on the workload, cost
   0 functions. Its name starts with `$`, which no name test here admits, so the
   scan reads the token bytes as the name's head. One capture of a local static with
   a dynamic initializer would settle whether `$` is a third separator or a name
   character — the two readings are one byte apart.
## 6g. The frame model (#35 step 1), 2026-07-30

**Byte evidence: `docs/CODEGEN_PPC_MVP.md` §"The frame model".** Established from
44 reference objs, and then reconciled against `docs/CODEGEN_FRAMED_CALLS.md`,
which was produced independently and in parallel from 480 designed compiles per
mode. **The two derivations agree** — neither knew the other's probes, which is
the strongest evidence either could have. Headline results, ordered by how much
they should change what the next person does:

1. **A live wrong-bytes emit, found before the model was written.**
   `framed_call_text` emitted one byte-constant 0x24-byte body; the parser
   required the call's argument to be *a* formal and then dropped the formals
   list, so the emitter assumed it was the formal already in r3. c2 emits
   `or r3,rN,rN` first whenever it is not, and the `.pdata` `FuncLen`, both `$M`
   label values and the REL24 site all followed it wrong. **37 of 47 probes
   around the accepted class mismatched** — every argument at a non-zero formal
   position, every member function (`this` takes r3), and every free function
   with a leading `float`/`double`/`long long`/pointer/8-byte aggregate
   parameter. Four mode lanes, a 4,706-case sweep, an 878-TU scan and a green
   test suite were all green over it, because every framed fixture and all 363
   generated framed cases have exactly one parameter. `GAPS.md` §6 instance 8.
   The *other* capture agent's 87 probe TUs found no mis-emit; this one was
   inside the accepted class, which only a probe of the class's own neighbours
   reaches.
2. **The frame size is a function, and its shipped constants were its all-zero
   case.** `align16(80 + locals + 8 + 8·nSaved)` for `nOutSlots ≤ 8`, exact on
   all 44 witnesses; the general form with the `nOutSlots` term is
   `CODEGEN_FRAMED_CALLS.md` §1.2 and `FrameLayout` implements it, carrying four
   rows of that sweep as cross-check assertions. This item's own premise — "96 B
   for one by-value temporary, 112 for two" — was the **saved-register count**
   misread as a temporary count.
3. **`_RtlCheckStack12` is not "past a page".** A frame under `0x5000` is probed
   inline with `ld r12,-4096k(r1)`, one per page boundary *crossed*
   (`floor((F−1)/4096)`, so `F = 4096` probes nothing); the call arrives at five
   pages, `li r12,−F` / `bl _RtlCheckStack12` / `stwux r1,r1,r12`. Boundary
   pinned by the pair `F = 20464` (inline) / `F = 20480` (call). This axis is not
   in the other document — its `localsBytes` tops out at 132 and this one's at
   200,000.
4. **The two save-helper thresholds are different numbers**: `__savegprlr_N` at
   3 saved GPRs, `__savefpr_N` at 4 saved FPRs, each pinned by the pair either
   side of it. Independently confirmed by §2.3/§2.4 of the other document. The
   GPR helper also carries the LR and the epilogue *tail-branches* into
   `__restgprlr_N` with no `blr` at all; the FPR helper does not, and its restore
   is an ordinary `bl`. The **mixed inline** case is this rung's alone: with GPRs
   and FPRs both saved inline, the prologue stores GPRs then FPRs while the
   epilogue restores in ascending slot address — so the two lists are **not**
   mirror images.
5. **The label-stride gate keyed on the wrong thing.** The comparison leaf's
   stride is 1 or 3 *by relation*, measured over the whole 60-point grid
   (relation × literal × signedness) against a seed read out of `.gl`
   (`OBJ_GY_SHAPES.md` §3.6a). The old sizing of the over-refusal — "6 of 21
   sweep cases" — was wrong in **both** directions: only 3 of 21 were this gate's
   doing (two of the named refusers do not decode as comparison leaves at all and
   are refused by the class gate either way), and the correct relaxation admits
   far more than the sweep samples — 39 newly admitted probe TUs byte-exact, 24
   neighbours still refusing, 0 mismatch. The gate now asks one three-valued
   predicate (`IlFunction::label_slots`) so an unmeasured class refuses rather
   than defaulting to 1.
6. **A census/gate disagreement in the under-claiming direction, introduced by
   this rung and closed inside it.** Reusing `select_text` for the argument setup
   inherited its `params.len() > 8` refusal, so
   `int f(int a,…,int i){ return g(a) + 1; }` censused 1/1 in class while the port
   returned `NotImplemented`. Nothing tests that direction. Found by probing for
   it; closed by moving the gate into the parser
   (`framed-arg-over-eight-formals`) and sized at **zero functions** on the
   workload. The genuinely-needed half of it is real: past the eighth formal the
   argument setup is `lwz r3,180(r1)`, which the old emitter answered with no
   instruction at all.

**Scope kept narrow on purpose.** `FrameLayout` builds only layouts needing no
external helper and no stack check; the three helper shapes refuse by name,
because each puts a second REL24 site in the prologue that `coff::Function` does
not model, and past 17 saved registers the sizing rule itself stops being exact.
Which live value gets which callee-saved register is **not** modeled — with two
the assignment is monotone in source order, with three or more it is not — and
that is step 2's problem, which both documents independently call the expensive
half.

**Census: 0.** This rung's isolated effect, measured pre-merge against its own
baseline binary on the same corpus HEAD (`dc3-decomp` `05ca6d09`):
**418,628 → 418,628**, 878 rows and `fn_total` 2,462,571 both times, **0 TUs
changing class, 0 blocker keys moving**. Post-merge the numerator is master's
**427,655 / 2,462,571 = 17.37 %**, unchanged by this branch. Predicted 0 before
the code was written — the shape the widening admits (`call-postop-0xB9`) has
**zero** occurrences on the workload — and it was 0. This rung is correctness,
not coverage.

**Gate evidence** (merged tree, workload tree `dc3-decomp` at `05ca6d09`):
`cargo test --workspace` **380 pass / 0 fail**; `c2rs bench` **138 pass / 0 fail /
0 error**; mode lanes over 138 fixtures `/Ox` **61**, `/O1` **59**, `/O2` **59**,
`/Ox /Gy` **59**, **0 mismatch in all four** (this branch adds 6 fixtures: 4
matching in every lane, 2 refusing); the generated sweep **4,829 cases, 0
mismatches**; 878-TU scan match 6, **mismatch 0**, codegen-gap 0, port-error 0,
capture-fail 7, **census/gate disagreement 0** — checked in the under-claiming
direction too, which is where this rung's own defect was.

### The handoff for #35 step 2, ranked and sized

`CODEGEN_FRAMED_CALLS.md` §7 has the rung order and this rung endorses it. What
this rung changes about it:

1. **Class A, many calls, no saved registers** — unchanged as the first rung, and
   now cheaper: `FrameLayout::default()` already *is* the Class A frame, the
   prologue/epilogue are derived rather than spelled, and `.pdata`/`$M` follow
   the emitted length. What it still needs is a second REL24 site per function
   in `coff::Function` (today one `Option<Call>`) and §4.1's symbol order.
2. **Argument marshalling** — unchanged; and note that the *one-argument* case is
   now done, including the register move this rung was missing, so the
   generalization has a correct base case to extend.
3. **`nOutSlots > 8` and addressed locals** — `FrameLayout` already computes
   both; the missing half is deciding `out_slots` and `locals` from the IL, which
   is a parser question, not a codegen one. **Add the stack-probe rules to
   whatever gate that lands behind**: a frame past four pages is not a `stwu` at
   all, and the port has no `_RtlCheckStack12` external today.
4. **Class B (1–2 saved GPRs)** — the frame arithmetic and the exact prologue and
   epilogue words are done and unit-pinned; the whole remaining cost is the
   liveness answer plus the register-assignment order, which is measured for
   `n = 2` (first live value → r30, second → r31) and **is not monotone at
   `n ≥ 3`**.
5. **Class C (≥3 saved GPRs)** — do not attempt without the `/Gy` stride
   correction (7, not 5, with the extra two slots *before* the function's own
   `$M` pair). It is latent today only because `FrameLayout` refuses these
   frames, and it is six wrong bytes per label the moment one is admitted.
6. **Classes D/E/F (FPRs)** last. The FPR-helper label stride is predicted +4 by
   the same reading and is **not captured**; one TU pairing an FPR-helper
   function with a following function settles it.

## 6h. Instrument hardening (roadmap #15, #46, #47, #48 — 2026-07-30)

Four instrument failures surfaced in one day, all of the same family: a
measurement that was wrong, or expensive enough not to be re-taken, without
anything in its own report saying so. This section records the four fixes and
what measuring them turned up. **None of it moves the port**: the census reads
**418,628 / 2,462,571 (17.00 %)** with **census/gate disagreement 0** and
**mismatch 0** before and after, and every per-TU JSONL row is byte-identical
across an uncached scan, a cold cached scan and a warm one.

**Measured at** c2-rs `b36a046` + this change, workload `dc3-decomp` `05ca6d09`
(one tracked file modified), wibo `1.0.1-23-g4a9dd6f`, XDK `16.00.11886.00`,
`--jobs 16`.

### #15 — the capture cache: 36.5 s → **0.9 s**

`c2rs gap` re-ran `cl.exe` under wibo under strace for all 878 TUs on every
invocation. Captures are pure in their inputs, so they are now cached
content-addressed under `work/capture-cache` (gitignored; `C2RS_GAP_CACHE` or
`--cache DIR`, `--no-cache` to bypass). Full accounting of the key, the
collision handling and the validator: `crates/c2-harness/src/capture_cache.rs`.

| run | wall | CPU | cache |
|---|---|---|---|
| uncached baseline (master) | 36.5 s | 446 s | — |
| cold (fills the cache) | 46.6 s | 518 s | 0 hit / 878 miss |
| **warm** | **0.9 s** | 6.0 s | 871 hit / 7 miss |
| warm + `--validate-cache 50` | 2.4 s | 15.0 s | 17 re-captured, 0 poisoned |
| warm + `--replay-every 25` | 1.5 s | 11.8 s | replay 36 checked / 0 diverged |

**39× against the cold run, 30–41× against the uncached baseline.** The cold run
costs ~28 % more than an uncached one (writing 2.2 GB of bundles), which is the
honest price of the first scan. The 7 misses on every warm run are the 7
`capture-fail` TUs: a failure is deliberately not cached, because a cached
failure is indistinguishable from a real one.

**Estimate vs outcome.** Predicted 1.5–3 s (12–25×) from a 20-TU sample, with a
stated bias that bundle re-read I/O would dominate; outcome 0.9–1.2 s (39–52×).
The bias direction was right and the magnitude was wrong by ~2× — 2.2 GB stays
in page cache, and the census walk over 2.46 M function bodies costs ~6 s of CPU
against the captures' ~440 s. **Capture is 98.7 % of a scan**, which is the
number to reuse next time.

### #46/#48 — provenance, and the loader

Every scan and self-test now prints (and `gap` records as JSONL record 0,
`"record":"provenance"`) the workload tree's git HEAD + dirty flag, the c2-rs
HEAD + dirty flag, the resolved `wibo`/`cl.exe`/`c2.dll`/`c1xx.dll`/`strace`/
`mingw` paths, and wibo's `--version` — warning loudly when it parses older than
`WIBO_KNOWN_GOOD` (`1.0.1-23`). Everything degrades to `unknown` when git or the
loader will not answer; nothing here can fail a run. Per-TU rows are unchanged,
so two scans' rows stay byte-comparable. Rationale and the two failures that
forced it: `docs/GAPS.md` §6.

### #47 — the census/gate invariant, in both linkage modes

`tests/census_gate.rs` asserted the invariant with `fn_level_linking=false`
only — i.e. in the mode the fixtures capture in, and not in the mode the entire
878-TU workload compiles in (`/O1` implies `/Gy`). Both lanes are now pinned,
with their causes named rather than just their totals: **`/Ox` 1** (the
`w13_fscratch.cpp` FP-scratch refusal) and **`/Gy` 9** (that one plus **8**
pooled-FP-constant refusals — `emit_comdat_obj` does not place the `.rdata`
COMDAT a W13b body needs). Both cost **0 functions on the workload**. A residual
that stays at 9 while one refusal is traded for another now fails too. Moving
the pooled-FP gate is a `c2-core` change and is **not** taken here.

### What the validator found on its own control case

Both facts came from the bypass-and-compare *failing where it was supposed to
agree*, which is the argument for having a control group at all:

1. **The bundle base is a per-invocation nonce.** `cl.exe` names the IL bundle
   `_CL_<hex>` freshly each run, so two captures of one TU differ in their file
   names and in the `-il` value of the `/Bd` argv echo — and nowhere else.
2. **The reference obj's COFF `TimeDateStamp` is wall clock, not pinned.** One
   cold scan's 878 objs carry **58 distinct** stamps, monotone across the scan's
   5-minute window. 51 of 51 sampled re-captures differed in bytes 4..8 and were
   byte-identical everywhere else. The project's criterion zeroes those four
   bytes by definition so nothing moves, but `c2-reference`'s standing
   "RAW-identical, wibo pins it" note is measured **back-to-back within one
   second** and is corrected in place.

## 6i. W25 + W26 — the store leaf and the one-byte-unsigned value class (2026-07-30)

`void f(S* s, int v) { s->m = v; }` is one `stb`/`sth`/`stw`/`std` at a folded
displacement, and it is the **third** consumer of the sub-object designator the
indirect-load leaf (`lwz`) and the address leaf (`addi`) already share. Full
write-up, with every captured word and every counterfactual, in
`docs/IL_STORE_LEAF.md`.

**The three candidates were measured before anything was implemented**, and the
ranking the row sizes suggested was not the one the measurement produced:
`expr-load-type-8885` (82,810 blocked) completes **0** bodies under a full type
widening; `expr-load-type-8645` (98,813) completes **1,004**, which is the
already-named FP `fmr` rung; `expr-intrinsic-base-member-addr` (118,331, the row
the rung was commissioned against) completes **740**. What made it worth doing is
what those 740 bodies *are* — a store — and that the same production through the
*plain* designator is **29x bigger** and sits in `expr-op-0x27`, the #1 row on the
board. `IL_CALL_IN_EXPR.md` §19.3's lesson at a bigger ratio: grep for every site
that implements the rule you are changing.

**It also corrects a measurement in `GAPS.md` §6.** `expr-op-0x27` was written up
as "measured to the bottom, twice: 505,122 released, 685 whole bodies, 0.14 %".
That counterfactual admitted the **token** `27` inside `parse_expr`, so it could
only count bodies that finish as an *expression*; half the row is a *statement*
that fails one token later at the `32` store. This rung took **22,095** functions
out of it. A counterfactual measures what the surrounding grammar can already
finish — "admit this token" and "admit this production" are different questions.

### Census

**418,628 → 442,273 (17.00 % → 17.96 %), +23,645**, mismatch 0, no TU changing
class, **0 new census keys**, census/gate disagreement still **0**. The sum of
every blocker key's delta is −23,645 — the bucket drop equals the gain to the
function, for the sixth rung running. Estimate was **+22,821, biased LOW** (the
counterfactual); the +824 residual is the class-preserving `2C` on the stored
value, the one rule that changed between the counterfactual build and the shipped
one. All 23,645 admitted bodies read `calls-0`, which is the standing control
group: this production cannot describe a body containing a call.

### Gate evidence

Corpus `dc3-decomp` at **`05ca6d09`**; baseline re-taken in this worktree and
reproducing master `b36a046` to the function.

| lane | baseline | W25 |
|---|---|---|
| `cargo test --workspace --release` | 370 pass | **372 pass**, 0 fail |
| `c2rs bench` | 132 pass / 0 fail / 0 error | **132 pass / 0 fail / 0 error** |
| `mode_lane.sh /Ox` | 56 match, **0 mismatch**, disagreement 1 | **57 match, 0 mismatch**, disagreement 1 |
| `/O1` · `/O2` · `/Ox /Gy` | 54 match, **0 mismatch**, 2 codegen-gap, disagreement 9 | **55 match, 0 mismatch**, 2 codegen-gap, disagreement 9 |
| `scripts/expr_sweep.sh` | checked 4,706 | checked **4,900** after W26, mismatches **0** |
| 878-TU scan | match 6, mismatch 0, 418,628/2,462,571, 569 keys, disagreement 0 | match 6, **mismatch 0**, **442,273**/2,462,571, **569 keys**, disagreement **0** |
| `census fixtures/cpp/w25_store_leaf.cpp` | — | **41/41 in class**, `Port=Match` |
| `census fixtures/cpp/w25_store_leaf_neg.cpp` | — | **0/15 in class**, `Port=NotImplemented` |

### W26 — the one-byte-unsigned value class, taken in the same session

The rung W25's own measurement ranked next, and it landed at the size the
counterfactual gave it. `bool` and `unsigned char` share the operand TYPE
`82 12`, and **inside** the class a value costs no instruction at all —
`return false;` is `li r3,0`, `return b;` is a bare `blr`, and from any other
argument register it is the W18 register move — so this is the second pure decode
widening in this project with **no emitter change**. **Out of** the class it is a
real `rlwinm r3,r3,0,24,31`, arriving on the same `2C … 00` token that is free
between the two width-4 classes, which is why `ValueClass::Int1u` is its own class
and the `41` result annotation is required to restate it.

**442,273 → 464,584 (17.96 % → 18.87 %), +22,311**, mismatch 0, disagreement 0,
+1 key. Estimate **+23,122 biased HIGH** with the cause named in advance — the
counterfactual widened `eat_return_head`'s `41` gate globally and the shipped rung
does not, so the 809 `calls-1` bodies (a `bool`-returning tail call) were expected
to be lost; realized **+22,311**, two functions off the predicted `calls-0` half.

`w26_bool_value.cpp` **15/15 in class, `Port=Match`**; `w26_bool_value_neg.cpp`
**0/10**. Lanes `/Ox` 58, others 56, **0 mismatch**, disagreements unchanged at
1/9/9/9. `bench` **134 pass**, workspace **373 pass**, sweep **4,900 cases, 0
mismatches**. The two new guards (`expr-int1u-arith`, `expr-int1u-mixed`) cost
**0 functions** on the whole workload — every `bool` arithmetic in 2.46 M
functions converts first.

Full write-up, including what the refusals cost (the mask 4,947, the
`char`/`signed char` class 1,646, the `bool` tail call 809), in
`docs/IL_STORE_LEAF.md` §9.

**One census/gate hole was found and closed on the way**, by probing W25's own
boundary rather than by a test: the parser admitted any literal as a stored value
while `emit_load_imm` refuses a wide *negative* one, so
`void f(S* s){ s->a = -70000; }` censused in class against a
`Port=NotImplemented`. The straight-line class had gated it in the parser since
W5; the new shape reached the same literal by a second route and did not — the
fifth instance of `GAPS.md` §6's "one fact, two locators". Fixed in the parser,
cost 0 functions on the workload, pinned by `n_negwide` and 2 sweep cases.

### Merged against master `7011b49`

Both rungs were developed against `b36a046`; master advanced four times in
flight (D14, the ground-truth docs drop, instrument hardening, the frame model).
**Merged census 473,611 / 2,462,571 (19.23 %)** — and additivity was *measured*,
not assumed: differencing this rung's own tree against the merged scan moves
exactly two keys, `callee-unresolved-dtor-delegation:eof` (−9,028) and
`callee-unresolved-tail-call:eof` (+1), which is D14's population and nothing
else. Interaction term **0**.

Merged-tree gate: workspace **398 pass**, `bench` **142 pass / 0 fail / 0 error**,
lanes `/Ox` **63** and `/O1`·`/O2`·`/Ox /Gy` **61**, **0 mismatch** in all four,
`census_gate.rs` passing at its recorded per-lane values (**1** packed / **9**
`/Gy`) with its named causes unchanged, sweep ** 5,023 cases / 0 mismatches**,
878-TU scan match 6 / **mismatch 0** / capture-fail 7 / disagreement **0** / 570
keys, corpus `dc3-decomp` **`05ca6d09`** (carried in provenance record 0). Both
positive fixtures still N/N `Port=Match`, and so do the frame rung's `wfr_*`.

Two merge resolutions are recorded in `IL_STORE_LEAF.md` §10.3: a duplicate
`encode_std` (byte-identical, two independent captures — the frame side's copy
kept untouched, this rung's removed), and the rung tag `W23` having been taken
twice, which is why this section is §6i and the fixtures are `w25_`/`w26_`.

## 6j. Class A many-calls (#35 step 2, rung 1), 2026-07-30

The first rung of §6g's handoff: a framed body with **more than one call and
nothing live across any of them**. Byte evidence in
`docs/CODEGEN_PPC_MVP.md` §"Class A many-call bodies"; the shapes and their
neighbours are `fixtures/cpp/mvp_call_seq.cpp` (10/10 in class) and
`mvp_call_seq_neg.cpp` (0/4, must refuse).

**Two live wrong-bytes emits found, both on mainline, neither about this rung.**
That is the headline; the coverage is not.

1. **`int f(int a,int b){ int z = g(b + a); return z; }` emitted
   `add r3,r4,r3` for `add r3,r3,r4`.** The call-argument validation existed in
   **two copies** — the direct `return g(…)` form and the bound-to-a-local form —
   and each was missing a gate the other had. The bound copy never asked
   `leaves_ascending` (c2 canonicalizes a commutative argument's leaves, so
   `g(a+b)` and `g(b+a)` are the same obj), and it never got the
   `call-arg-outer-formal` gate either, so `int z = g2(a, c); return z;`
   **panicked** `c2rs census` with the same out-of-bounds index that was fixed in
   the other copy when it was found. `GAPS.md` §6 instance 9. Both closed by
   making the two copies one (`tail_call_shape`).
2. **A multi-argument permutation with a cycle longer than three.** The
   single-temp cycle walk is right at cycle length 2 and 3 and wrong past it:
   measured over **complete** grids — all 24 four-argument permutations and all
   84 single cycles of length 2–5 in a five-argument call — 0 of 30 wrong at
   lengths 2–3, **10 of 30 at length 4 and 16 of 24 at length 5**. Past three c2
   hoists a second save into r10 and reorders the writes.
   `int f(int a,int b,int c,int d){ return a4(c,d,b,a); }` was
   `Port=Mismatch @ 8` on mainline, in the plain tail call, with nothing framed
   about it. `GAPS.md` §6 instance 10. Gated at the measured edge
   (`call-arg-long-cycle`), because what c2 does past three is described by the
   grid and not explained.

Both were found by the practice §6g named: **compile the neighbours of the class
you are about to widen.** 613 generated probe TUs, graded in both `/O1` and
`/Ox`, now report 0 mismatch and 0 census/gate disagreement.

**A third defect, in the census/gate direction that nothing tests.** A call
argument is computed into r3 by `select_text`, so it is subject to exactly the
same out-of-class rules as a leaf body — and those lived only in codegen for this
position. `int f(int a){ return g(a * 5); }` censused 1/1 while the port refused
(a constant multiply strength-reduces). Moved into the parser; zero functions on
the workload, which is why the scan's disagreement counter never saw it.

**What the rung admits**, each with byte evidence:

* `void f(){ g1(); g2(); }` and any number of statement calls after it;
* arguments per call through the *same* `select_text` / `permute_args_text`
  locators every other call shape uses — a dying formal (`g1(a); g2();`), a
  literal (`g1(1); g2(2);`), a computed value (`g1(a+1); …`), a permutation
  (`g2(b,a); h();`);
* three tails: nothing, `return <literal>;`, and the last call's value with the
  optional `+ k` post-op the single framed call already carried;
* an explicit trailing `return;`, which c2 records as a **second `3A <label>`
  branch to the same label** and emits nothing for — the two objs are
  byte-identical (1090 B each, compared whole). The label compare is the gate:
  a real early return branches elsewhere.

**What it refuses, by name.** `callseq-value-live-across-call` — a formal read
after the first call has to survive one, and c2 answers with `r31` behind a
`std`/`ld` pair. That is Class B and it is the next rung. Also
`callseq-over-eight-formals`, `callseq-tail-lit-wide`, and (from the shared
locator) `call-arg-long-cycle`.

**Two boundary facts that would each have been a mis-emit if assumed:**

* **A lone statement call IS tail-called.** `void f(int a){ g(a); }` is a bare
  `b ?g` — five sections, no frame, no `.pdata`. So the class boundary is "is
  there anything after the call", not "are there two calls":
  `int f(int a){ g(a); return 5; }` is framed on **one** call.
* **The last call of a framed body is NOT tail-called.** `int f(){ g1(); return
  g2(); }` ends `bl ?g2 ; addi r1,r1,96 ; … ; blr`. The transform is off the
  moment the function is framed.

**Symbol order and label stride, both confirmed against captures rather than
carried over.** A function's new callees are emitted in **reverse
first-reference** order (`g1();g2();g3();` → `?g3 ?g2 ?g1` at indices 15/16/17,
and the mirrored source refutes alphabetical and declaration order); a repeat
introduces no second symbol and relocates against the first
(`g1();g2();g1();` → two symbols, three REL24s); a callee an earlier function
already introduced is not re-emitted. **The same order holds packed**, which
`CODEGEN_FRAMED_CALLS.md` §4.1 measured only under `/Gy`. The label stride is
**unchanged at 4 packed / 5 `/Gy`** — the call count does not enter the counter,
framedness does: two two-call bodies in one TU are `$M2553`/`$M2558` against a
`.gl+7` seed of 2538, and 2547/2551 packed.

**Census: 427,655 → 428,147 (+492), 17.37 % → 17.39 %.** Estimated **0–800**
before the code was written, biased high, upper-bounded at 4,503 (the
`call-postop-0x4B` bucket — every function that reaches a discarded-result call
before dying). Outcome 492, inside the band. The bias was the stated one: the
census names only the *first* blocker, so every one of the 4,503 had at least one
more construct to survive. 12,000+ functions moved between buckets for the 492
that landed in class — `call-postop-0x4B` 4,503 → 0 and
`call-multiarg-postop` 20,491 → 13,425 both drain into `callseq-tail-lit`
(7,771) and `call-ref-0x3A` (5,335), which are now the two largest
statement-call blockers and are the top of the handoff below.

**Gate evidence** (workload tree `dc3-decomp` at `05ca6d09`, c2-rs at this
branch, wibo `1.0.1-23-g4a9dd6f`, XDK `16.00.11886.00`): `cargo test
--workspace` **398 pass / 0 fail**; `c2rs bench` **141 pass / 0 fail / 0
error**; mode lanes over 141 fixtures `/Ox` **64**, `/O1` **62**, `/O2` **62**,
`/Ox /Gy` **62**, **0 mismatch in all four**; the generated sweep **4,829 cases,
0 mismatches**; 613 generated probe TUs in two modes, **0 mismatch, 0
disagreement**; 878-TU scan match 6, **mismatch 0**, codegen-gap 0, port-error 0,
capture-fail 7, **census/gate disagreement 0** — checked in the under-claiming
direction too.

### The handoff for the next rung, ranked and sized

1. **`callseq-tail-lit` — 7,771 functions, the largest single statement-call
   blocker.** The tail after the last statement call opens `33` (a literal) but
   is not the plain `return <literal>;` this rung admits — a literal-anchored
   *expression*. Sample the bytes at the blocking sites and group them by
   production before sizing it further; it is one bucket holding several shapes,
   exactly the thing §6 warns about.
2. **`call-ref-0x3A` — 5,335.** The tail is a branch record the void return
   plumbing does not accept. The *simplest* member of this family — an explicit
   `return;`, recorded as a second `3A <label>` to the same label — is admitted
   here and gained **0**, so the workload's 5,335 are a different shape: two
   branch records to **different** labels, i.e. a real control transfer. Likely
   an early return, which is the control-flow rung, not this one.
3. **Class B (1–2 saved GPRs)** — unchanged from §6g's ranking and now with a
   named refusal key to measure against (`callseq-value-live-across-call`, 2
   functions on the workload as a *first* blocker, which understates it badly:
   the shape is common and usually blocked earlier). The frame arithmetic and the
   exact prologue/epilogue words are done and unit-pinned; the cost is the
   liveness answer plus the register-assignment order, measured for `n = 2` and
   **not monotone at `n ≥ 3`**.
4. **Multi-argument calls with literal arguments.** c2 emits them in **descending
   destination order** (`void f(){ g2(1,2); … }` → `li r4,2 ; li r3,1`), which is
   captured; mixing a formal with a literal is **not**. Refused today as
   `call-arg-computed`. Cheap, and it needs one capture grid.
5. **The permutation order past a three-element cycle.** Instance 10's gate is
   drawn at the measured edge. The full 24-row four-argument grid and the 84-row
   five-argument one are in the working notes; what splits the four-cycles
   four/two is visible in the data and unexplained. Whoever closes it should
   enumerate cycle lengths, not add fixtures.
6. **A wide literal tail.** `int f(){ g(); return 70000; }` refuses
   (`callseq-tail-lit-wide`) although the straight-line class already emits
   `lis`+`ori` for a bare wide constant. Under-claiming, not a gap — one
   capture settles it.

## 6k. W27 + W28 — the FP argument register file, and the 167k claim measured (2026-07-30)

Full write-up, with every captured word and every counterfactual, in
`docs/CODEGEN_FP_ARGS.md`. Two rungs, one fact: **two register numberings run
over one parameter list and neither is the formal's index.** An FP parameter
takes `f<j>` counting FP parameters *alone*; every other scalar takes
`r<2 + slot>`, and an FP parameter consumes a slot while filling no register. So
the two disagree in opposite directions, which is why `int t6(int a, float b,
float c){ return gffi(b,c,a); }` emits exactly one `mr r5,r3` and no `fmr`.

### The commissioning claim, measured — and halved

`IL_STORE_LEAF.md` §7.1 recorded the **167,021** `calls-1` functions behind
`expr-load-type-8645`/`-8885` as FP tail calls "converging on the FP
argument-register item". That was read off a counterfactual's residue. A
whole-body counterfactual — the FP type admitted at the LOAD, LIT, `2C` target,
`55` call-end and `41` result annotation *at once*, so it is an upper bound —
says **85,231 of the 167,021 (51.0 %) become whole-body complete**: 59,095
single-argument tail calls and 26,136 permutations, **0** of it `calls-2plus`.
Confirmed in kind, halved in size, and the other 81,790 are not reachable by any
FP rung. Admitting the FP *literal* classes as well releases 10,665 more blocked
functions and completes **0** additional bodies — a clean refutation worth
keeping, since the FP constant machinery is the expensive part.

### What was taken

**W27, the `fmr`** — `float f(float a, float b){ return b; }` is `fmr f1,f2`,
and `float mixfp(int a,float b,float c){ return b*c; }` is `fmuls f1,f1,f2`.
Both were live wrong-bytes emits (`GAPS.md` §6 (6) and (7)); the blunt gate that
closed them — every formal must be an FP operand — cost a MEASURED 1,005
(§23.1). Replaced with the real numbering, read from `.sy`'s type **kind** (05 =
"real"), which is the right key because the `<tid>` is per-TU: `float` is `40`
but `const float` is `0x1002`, and a `const float` is still passed in an FPR.

**W28, the FP store leaf** — `void f(S* s, float v){ s->f = v; }` is one
`stfs`/`stfd`, the **fourth** consumer of the sub-object designator three other
leaves already share. Sized at **7,984** by counterfactual before it was built;
`IL_STORE_LEAF.md` §6 had it as "measured and not implemented" with the FP
numbering named as what stopped it, and §7 (3) ranked the pair together.

### The alarm this rung found — the ninth live wrong-bytes emit

`coff::Function::is_float` was answering two questions with one field: "this body
does FP arithmetic (label stride 2)" and "this TU needs `_fltused`". An FP store
is the first construct that satisfies only the second, and the port emitted
**all fourteen** positive objs one symbol short. `GAPS.md` §6 (9).

### Census

**473,611 → 482,542 (19.23 % → 19.60 %), +8,931**, mismatch 0, disagreement 0,
**0 new census keys** (570 → 570), no TU changing class. The sum of every blocker
key's delta is exactly −8,931 over exactly **two** keys — `expr-load-type-8645`
−1,004 and `expr-op-0x27` −7,927 — the eighth rung running where the bucket drop
equals the gain. All 8,931 are `calls-0`.

Estimate **+1,005 / +7,984 = +8,989**, realized **+8,931**, biased HIGH by 0.6 %:
1 from a clause that deliberately holds the pooled-FP-constant population fixed
(it is refused under `/Gy` in codegen only, and widening the parameter model put
one such body in class, taking the scan's disagreement 0 → 1), and 57 from the
FP-literal and conversion refusals the counterfactual did not gate.

Note that **neither rung's gain came out of the row that named it**: the FP store
fell out of `expr-op-0x27`, the #1 row, because a store's parse blocks at the
`27` offset-add long before it reaches the value's type. `GAPS.md` §6's
unstable-attribution rule, paying off in the predicted direction, and the reason
both estimates were taken from counterfactuals rather than from row sizes.

### Gate evidence

Corpus `dc3-decomp` at **`05ca6d09`**; baseline re-taken in this worktree and
reproducing master `473c6a4` to the function (473,611 / 570 keys /
disagreement 0).

| lane | baseline | W27 + W28 |
|---|---|---|
| `cargo test --workspace --release` | 398 pass | **401 pass**, 0 fail |
| `c2rs bench` | 142 pass / 0 fail / 0 error | **146 pass / 0 fail / 0 error** |
| `mode_lane.sh /Ox` | 63 match, 0 mismatch | **66 match, 0 mismatch**, 0 codegen-gap |
| `/O1` · `/O2` · `/Ox /Gy` | 61 match, 0 mismatch | **64 match, 0 mismatch**, 2 codegen-gap |
| `scripts/expr_sweep.sh` |  5,023 cases | **5,868 cases**, mismatches **0** |
| 878-TU scan | 473,611, 570 keys, disagreement 0 | **482,542**, **570 keys**, disagreement **0**, mismatch 0 |
| `census fixtures/cpp/w27_fp_reg.cpp` | — | **33/33 in class**, `Port=Match` |
| `census fixtures/cpp/w28_fp_store.cpp` | — | **14/14 in class**, `Port=Match` |
| `census fixtures/cpp/w28_fltused_order.cpp` | — | **5/5 in class**, `Port=Match` |
| `census fixtures/cpp/w28_fp_store_neg.cpp` | — | **0/11**, `Port=NotImplemented` |
| `census fixtures/cpp/w13_fparam_neg.cpp` | 0/19 | **0/3**, `Port=NotImplemented` |

`w13_fparam_neg.cpp` shrank from 19 functions to 3 because **16 of its negatives
are now positives**: they moved verbatim into `w27_fp_reg.cpp` and are emitted
byte-exact. A case crossing from a negative fixture to a positive one is the only
direct evidence that a rung took what its refusal cost.

`census_gate.rs` passes at its recorded per-lane values (1 packed / 9 `/Gy`),
causes unchanged.

### The generator trap, hit again

`scripts/expr_sweep.sh` grew an FP block whose loop variable was `n` — the
generator's own file counter — and the sweep reported **3,344 cases where the
last recorded run was 5,023**, silently overwriting. That is the exact failure
`GAPS.md` §6 records, in the same file, and the only tell was the printed count
*falling*. Fixed (`nfp`), and it is the second instance: the rule "a generated
corpus must report its own size on every run, and that size must be compared
against the last one" earned its keep a second time within a week.

### Also taken: `fp_contract`, and the varint the reader was not

`OPT_MODE.md` §6.4/§6.5. `#pragma fp_contract(off)` clears bit `0x4` of the
per-function optimization word, the port compared the word whole, and the whole
of two census keys — **206 functions, 188 `calls-0`** — was refused on that
ground alone. The bit's only effect on emitted bytes is FMA contraction, which
`try_parse_float_leaf` already refuses; measured at corpus scale in **both**
modes rather than inheriting one from the other (129/1 at `/O1`, **145/1** at
`/Ox`, the differing fixture being `w13_fneg` both times, and refused).

Roadmap **#52 audited on the way through**: `opt_word_at` required the `80`
escape and the word is a varint. Fail-closed, so never wrong bytes — but a
short-form word censused as `opt-mode-00000000`, a key asserting the word is
*zero* when it is unread. Fixed; **0 functions** on this workload. One existing
test used `4F 1F 11` as its "unreadable prefix" case and `11` is the readable
short-form word 17, which is the finding in miniature.

**482,542 → 482,748, +206 exact**, and the two `opt-mode` keys vanish entirely
(**570 → 568**).

### The session total

**473,611 → 482,748 (19.23 % → 19.60 %), +9,137** over three rungs, mismatch 0,
census/gate disagreement 0, **two keys fewer** and none added. Final gates:
workspace **403 pass**, `bench` **148 pass / 0 fail / 0 error**, lanes `/Ox`
**68** and `/O1`·`/O2`·`/Ox /Gy` **66**, **0 mismatch** in all four,
`census_gate.rs` at its recorded 1 packed / 9 `/Gy`, sweep **5,868 cases / 0
mismatches**, 878-TU scan match 6 / mismatch 0 / capture-fail 7 / disagreement 0,
cache validator 17 re-captured and agreed / 0 poisoned, corpus `dc3-decomp`
**`05ca6d09`** and wibo **1.0.1-23** from provenance record 0.

### Merged against master `9ec4871` (Class A many-calls, #35 step 2)

Developed against `473c6a4`; master advanced once in flight. **Merged census
483,240 / 2,462,571 (19.62 %)** — and additivity was *measured*, not assumed,
because the brief flagged an interaction risk: this rung's store leaf and step
2's `call-postop` changes both touch `expr-op-0x27`-adjacent attributions.

Differencing this rung's own tree against the merged scan moves **+492**, which
is exactly master's own gain (474,103 − 473,611), so the **interaction term is
0**. Twenty-four keys move and every one of them is step 2's: `+7,771`
`callseq-tail-lit:eof`, `+5,335` `call-ref-0x3A`, `−7,066`
`call-multiarg-postop:eof`, `−4,503` `call-postop-0x4B`, `−832` `fn-tail-0xB9`.
The keys this rung owns are **bit-identical across the merge**:

| key | `473c6a4` | this rung | merged |
|---|---:|---:|---:|
| `expr-op-0x27` | 469,713 | 461,786 | **461,786** |
| `expr-load-type-8645` | 98,813 | 97,809 | **97,809** |
| `calls-0\|store-leaf` | 23,645 | 31,574 | **31,574** |
| `calls-0\|float-leaf` | 0 | 1,004 | **1,004** |
| `opt-mode-00200001` / `-00200101` | 140 / 66 | 0 / 0 | **0 / 0** |

### The merge found a live wrong-bytes emit that neither branch could contain

**The highest-value result of this merge, and it is not a number.** Before
trusting the merged tree, the cross product of the two rungs was compiled —
this rung's FP store and FP leaf beside step 2's Class A many-call body, in both
orders. It **mismatched**: `$M2564/$M2563/$T2565` against the reference's
`$M2565/$M2564/$T2566`.

`IlFunction::label_slots` still read `float_leaf`, so the FP **store** leaf got a
compiler-label stride of 1 where c2 gives it 2 — and a framed function's labels
come from a counter every function in the TU consumes, so the framed function
downstream was six bytes wrong in an obj that still links. This is
`GAPS.md` §6 instance **12**, and it is instance **11's own field one consumer
later**: splitting `is_float` into `touches_floating_point` fixed the `_fltused`
reader and left the stride reader behind. Instance #2 in its purest form — *fixed
in the one shape where the bug had been found* — with the tell available for
free, since splitting a field means auditing every reader and `grep float_leaf`
showed two.

MEASURED as the three-way capture that separates the candidates (one leaf ahead
of one framed function, reading the framed function's labels):

```text
  void lead(S* s, int v)      { s->i = v; }     $M2558 $M2559 $T2560
  void lead(S* s, float v)    { s->f = v; }     $M2559 $M2560 $T2561
  float lead(float a, float b){ return a * b; } $M2559 $M2560 $T2561
```

The stride goes with the **register file**, not with the body shape. Costs 0
functions on the workload; the TU-level gate now refuses the pair honestly,
because `coff::plan_labels` advances by 1 per non-framed function — admitting it
is a change to the framed side's label model and is ranked, not taken here.

**The practice that generalizes: a merge of two independently-green branches is a
new corpus, and the shapes only it contains have never been graded by anyone.**
The counter has an observable effect only when a framed function follows, so
before step 2 landed there was no framed shape that could share an in-class TU
with an FP store — this rung's fixtures have no framed function and step 2's have
no floating point. `scripts/expr_sweep.sh` now *generates* that cross product
(six leaf kinds × three call bodies × three orderings, +54 cases) instead of
relying on someone thinking of it again.

Three merge resolutions worth recording:

* `coff::Function::call: Option<Call>` became step 2's `calls: Vec<Call>` in the
  same hunk where this rung rewrote the `is_float` doc comment. Master's field,
  this rung's comment; the two `coff::Function` construction sites in
  `c2-core/src/lib.rs` take `calls` **and** `touches_floating_point()`.
* `IlFunction` gained `touches_floating_point` here and `is_framed`/`callees`
  there, at the same offset, and the conflict ate this side's **closing brace** —
  the shared-closing-brace hazard. Closed explicitly before splicing rather than
  letting the trailing `}` after the marker do double duty.
* `§6j` and the `GAPS.md` §6 mis-emit list were both taken. This section is
  **§6k**; the `_fltused` find is instance **11**, after step 2's 9 and 10, and
  the list header goes to *ten* wrong-bytes emits and two panics.

`scripts/expr_sweep.sh` was flagged as a collision and **is not one** — master
did not touch `scripts/` at all (`git diff --stat 473c6a4 master -- scripts/` is
empty), so the merged sweep starts from this rung's 5,868 rather than from 5,868
plus a step-2 block. Verified the way the trap demands — the printed count, the
generated count and the `.cpp` count on disk all compared, not assumed equal:
**5,868 / 5,868 / 5,868** at the merge, then **5,922** once the FP-beside-framed
cross product was added.

### Merged-tree gate

Corpus `dc3-decomp` **`05ca6d09`**, wibo **1.0.1-23**, `cl.exe`/`c2.dll`
16.00.11886.00 — all from provenance record 0 of
`work/dc3-workload/scan-merged2.jsonl`.

| lane | master `9ec4871` | merged |
|---|---|---|
| `cargo test --workspace --release` | 398 + step 2 | **407 pass**, 0 fail |
| `c2rs bench` | 148 | **152 pass / 0 fail / 0 error** |
| `mode_lane.sh /Ox` | 68 | **71 match, 0 mismatch**, 0 codegen-gap |
| `/O1` · `/O2` · `/Ox /Gy` | 66 | **69 match, 0 mismatch**, 2 codegen-gap |
| `scripts/expr_sweep.sh` | 5,868 | **5,922 cases** (printed = generated = on disk), mismatches **0** |
| 878-TU scan | 474,103 | **483,240**/2,462,571 (**19.62 %**), mismatch 0, disagreement **0**, 578 keys |
| cache validator | — | 17 re-captured and agreed, **0 POISONED** |
| `census_gate.rs` | 1 packed / 9 `/Gy` | **unchanged**, causes unchanged |

Per fixture, N/N with the verdict quoted: `mvp_call_seq` **10/10 Match**,
`mvp_call_twice` **1/1 Match**, `mvp_call_seq_neg` **0/4 NotImplemented**,
`il_call_bound_neg` **0/2 NotImplemented**, `il_call_multi` **0/7
NotImplemented**, `w27_fp_reg` **33/33 Match**, `w27_fp_reg_qual` **10/10
Match**, `w28_fp_store` **14/14 Match**, `w28_fltused_order` **5/5 Match**,
`w29_fp_contract` **16/16 Match**, `w28_fp_store_neg` **0/11 NotImplemented**,
`w28_fp_store_framed_neg` **4/4 in class, TU NotImplemented** (the label gate),
`w13_fparam_neg` **0/3 NotImplemented**.

### Found and not taken, ranked, with the frame axis applied

1. **The FP tail call — 85,231 measured, 0 of it `calls-2plus`.** The largest
   takeable item this measurement found, and it is a *leaf* rung, not a frame
   one. The lowering is captured (`CODEGEN_FP_ARGS.md` §1): the argument moves
   are `fmr` into f1..fN, the cycle scratch is **f0** exactly as the GPR file's is
   r11, and the shapes match the existing `permute_args_text` one for one. Two
   things stop it from being a line of code. First, the parser seam is
   `parse_call_shape`'s argument region (`parse_expr(…, 0x55)` plus the `55` and
   `41` type gates), which has been a concurrently-running rung's file twice
   running — step 2's, and now Class B's (values live across calls, 1–2 saved
   GPRs). **This is a rung to take once that seam is free, not to interleave
   with it**; step 2 has already rewritten the argument validation into one
   locator (`tail_call_shape`), which is the right place for the FP class to
   enter and is strictly easier to extend than the two drifted copies it
   replaced. Second, `int both(int a,int b,float c,float d){ return gif2(b,a,d,c); }` shows
   the two files' move sequences **interleaved** on a schedule no per-file solver
   reproduces — so the shippable first cut is *one* non-identity file at a time,
   which the single-argument 59,095 satisfies by construction.
2. **`2C` float→double, free everywhere it has been captured** — a bare `b` at a
   call boundary and a bare `stfd` at a store. Its narrowing twin is a real
   `frsp` through f0, and the IL spells both with the same `2C <TYPE> 00`, so
   this is one more instance of the standing shape and needs the direction
   decided from the two type triples. Not separately sized.
3. **The pooled FP constant under `/Gy`.** 1 function on the workload today, but
   it is a *census/gate split* rather than a gap — the refusal lives in
   `function_gate` because the linkage mode is a TU flag the parser cannot see —
   and every future FP rung will re-expose it. Modelling the `.rdata` COMDAT
   association deletes the clause W27 had to add.
4. **`__vector`/VMX128 formals — a third register file, unmeasured.** `.sy` class
   `D` is it (`vSrc`, 16 bytes, `src/App.cpp`), `arg_classes` refuses it under
   `param-kind-unknown`, and `ABI_EDGES.md` §5 has it unprobed. The workload uses
   it. Nothing can be numbered correctly in a function that has one.
5. **The general frame**, unchanged and still first by size: 802,655
   `calls-2plus`, which no leaf rung reaches and which this rung's 8,931 do not
   touch at all.
## 6l. W30 + Class B — the call-tail literal, and values live across calls (2026-07-30)

Two rungs in the framed/call seam, taken in the order the *measurement* ranked
them rather than the order the handoff did. `docs/ROADMAP.md` §6j ranked Class B
third and `callseq-tail-lit` first; sizing both by counterfactual before writing
any code put them 3,900× apart, and the session took the big one first and said
so.

### Sizing first, and the handoff's guess about `callseq-tail-lit` was wrong

Three counterfactual scans against master `9ec4871` (census 474,103 / 2,462,571,
workload `dc3-decomp` at `05ca6d09`), each a ~1 s warm scan:

| counterfactual | census | delta |
|---|---|---|
| lift `callseq-value-live-across-call` (Class B) | 474,105 | **+2** |
| lift the tail literal's exact-`int` type gate | 481,874 | **+7,771** |
| both | 481,876 | **+7,773 — exactly additive** |

The third row is the one that matters: the tail-literal check fires *before* the
live-across validation, so it could have been masking Class B's real size. It was
not. **Class B's whole-body-complete population on this workload is 2 functions**,
an exact ceiling rather than an estimate, and the realized outcome was 2.

§6j's handoff had ranked `callseq-tail-lit` first "by size" and guessed it was
"one bucket holding several shapes, exactly the thing §6 warns about". It is one
bucket holding **one** shape: every one of the 7,771 is a literal whose TYPE is a
width-4 integer that is not the exact `86 41 74` triple. The bucket's own
warning was right in general and wrong here, and the cheap way to find out was
the counterfactual, not the sampling.

### W30 — a rule with three implementations, two of them narrower

`33 <TYPE> <k>` is read at three places in the call productions — the sequence's
`return <literal>;` tail, the single framed call's `+ k` post-op, and the
sequence's value-call post-op — and each required the TYPE to be *exactly* `int`.
The emitted word is `li r3,k` / `addi r3,r3,k`, which is a function of the value
alone, so `unsigned`, `long`, `unsigned long`, an enum, a `const int` and a
`volatile int` were refused for carrying a per-TU type id in their third byte.
All three now go through `eat_int_like` — the locator `2C`, `41`, `30` and W22's
operand positions already agree through. The two post-op positions are worth
**0** on the workload and were widened anyway: one rule on two different gates is
`docs/GAPS.md` §6 #9's shape, and it costs nothing to close while the file is
open.

The workload's dominant spelling is `86 41 08`, a width-4 signed type whose id no
probe reproduced (`int`, `long`, `__int32`, `signed`, `size_t`, `ptrdiff_t`,
`__w64 int`, a namespace/class/anonymous enum and both cv-qualifications were all
tried). It is admitted on `is_int4_type`'s nibbles, which is what four other
positions admit it on, and the doc comment says so rather than implying a
capture. **Those 7,771 bodies are read and not emitted** — `JointUtl.cpp`'s
reference obj contains none of them — so this is a census gain under the census's
own denominator (IL bodies), which `docs/GAPS.md` §6 already distinguishes from
emitted functions.

**Census 474,103 → 481,874 (+7,771), 19.25 % → 19.57 %.**

### Class B — the liveness rule, closed by refutation

`docs/CODEGEN_FRAMED_CALLS.md` §6 lists "which values become callee-saved, and in
what order" as the half that refused to yield a rule: §3.1 *describes* an
allocator (descending from r31, parameters in order then results, reuse on death)
and `nSaved` is an input to the frame formula every other claim is conditional
on. For the call-sequence body it is now a rule, established over a 12-capture
ladder and stated as one sentence:

> **A formal read by any call after the first is copied into a callee-saved
> register, and the file is allocated descending from r31 in PARAMETER order.**

The ladder, each row a capture at `/O1 /GS- /c`:

| probe | body | what it pins |
|---|---|---|
| L1 | `v1(a); v2(b);` | 1 save, `F = 96`, 4-word prologue, `mr r31,r4` |
| L2 | `v1(a); v2(b); v3(c);` | 2 saves, `F = 112`, 5-word prologue, r31←b r30←c |
| L3 | `int r=i0(); v0(); v2(r);` | a call RESULT takes the next register — **not reachable** in this grammar (a result is discarded or is the tail; a bound one is a different production) |
| L4 | `int r=i1(a); v0(); v2(r); v3(a);` | parameters first, then results — §3.1 confirmed |
| L5 | 3 live formals | `bl __savegprlr_29`, tail-branch epilogue, no `blr` → REFUSE |
| **R1** | `v1(a); v2(c); v3(b);` | **parameter order, refuting first-use order** |
| R2 | `v1(a); v2(b); v3(c);` | R1's control — byte-identical prologue and saves |
| R3 | `v1(a); v2(a);` | a formal the first call reads too is *still* saved |
| R5 | `v1(a); g2(c,b);` | a later call marshals out of r31/r30, highest destination first |
| S4 | `v1(a); v2(5); v3(b);` | a literal argument needs no register of its own |
| S6 | `void f(float x,int a,int b){…}` | the FP-formal/GPR-index transfer, in a **framed** prologue |
| S7/U4 | `return i1(b)+1;` / `return 7;` | both tails beside the saves |

R1 is the load-bearing row: every probe in §3.1 has parameter order and first-use
order coinciding, so the description could have meant either. R1 and R2 emit
byte-identical prologues and byte-identical `mr r31,r4 ; mr r30,r5` pairs and
differ only in which one each later call reads back. The allocator walks the
parameter list.

**The `/Gy` label stride stays 5** — saved registers do not enter the counter
(two Class B functions in one TU are `$M2571/$M2572/$T2573` then
`$M2576/$M2577/$T2578`). §4.4's +2 belongs to the *helper* class, which is
refused. The step-2 handoff's warning not to assume this was worth heeding and
the answer happened to be "unchanged".

**Census 481,874 → 481,876 (+2), the estimate exactly.**

### The prediction that failed, and the mis-emit that followed it

Where the save moves go when the first call *also* marshals arguments is
measured, and the first model of it was wrong twice.

*Failure 1 — "as late as possible".* `S2` (`g2(a,c); v3(b);`) puts the save
**before** the marshalling and `S1`/`R4` put it **after**, which fits "emit each
save immediately before the first instruction that destroys its source". `U3`
(`g2(a,d); v1(b); v2(c);`) splits it perfectly — `mr r31,r4 ; mr r4,r6 ;
mr r30,r5` — and `U1` (`g3(a,d,e); v1(b);`) refutes the lazy reading: `mr r31,r4`
precedes **both** marshalling moves although only the second touches r4. So the
hoist clears the whole marshalling, not just the writer.

*Failure 2 — the r11 finding, and it was a live mis-emit for the length of one
probe run.* With hoist/trail implemented, a 17-TU grid over first-call
permutations came back **11 mismatches of 17**. A non-identity permutation beside
a save is not this interleaving at all: when a permuted argument's value is also
callee-saved, **c2 breaks the cycle through the callee-saved register and emits
no `r11` whatever**, because the save has to happen anyway.

```text
  void f(int a,int b){ g2(b,a); v1(a); v2(b); }           a->r31, b->r30
    mr r30,r4 ; mr r31,r3 ; mr r4,r3  ; mr r3,r30  ; bl ?g2
  void f(int a,int b,int c){ g2(b,a); v1(a); v2(c); }     a->r31, c->r30
    mr r31,r3 ; mr r3,r4  ; mr r4,r31 ; mr r30,r5  ; bl ?g2
  void f(int a,int b,int c){ g3(a,c,b); v1(a); v2(b); }   a->r31, b->r30
    mr r30,r4 ; mr r4,r5  ; mr r5,r30 ; mr r31,r3  ; bl ?g3
```

Refused at the measured edge — which saved register serves as the temp when
several are saved is not determined by three captures. The generalizable bit is
the *third* instance of "a rule fitted to the shapes the corpus happened to
contain" (`GAPS.md` §6 #10): the hoist/trail model was derived from captures, fit
every one of them, and was wrong on a cell none of them entered. What separated
it was enumerating the permutations — all 2 of two arguments and all 6 of three —
rather than adding one more hand-picked case.

### What the class admits and refuses, by name

Admits: 1–2 formals live across calls; any number of later calls; a first call
that marshals a single argument (`mr r3,rN`, `li r3,k`) or the identity
permutation; later calls reading formals straight out of the saved file, singly
or as a multi-argument set; literal arguments; all three tails; an FP formal in
the parameter list.

Refuses, each by name and each with the capture that would settle it in the
fixture comment: `callseq-three-plus-saved` (the `__savegprlr_29` class),
`callseq-saved-with-first-call-setup` (a non-identity permutation — the r11
finding — or a computed argument, whose write set reaches the callee-saved file
under `/Ox`), `callseq-saved-computed-arg` (`addi r3,r31,1`, the operand stream
rebased onto a saved register).

### Gate evidence

Workload tree `dc3-decomp` at `05ca6d09`, c2-rs on `wt-class-b` from master
`9ec4871`, wibo `1.0.1-23-g4a9dd6f`, XDK `16.00.11886.00`.

* `cargo test --workspace --release` **403 pass / 0 fail** (was 401);
* `c2rs bench` **149 pass / 0 fail / 0 error** (was 145);
* mode lanes over 149 fixtures: `/Ox` **68**, `/O1` **66**, `/O2` **66**,
  `/Ox /Gy` **66**, **0 mismatch in all four** (was 66/64/64/64);
* `scripts/expr_sweep.sh` **5,023 cases, 0 mismatches**;
* generated probes: **70 TUs** for W30 (55 match, 0 mismatch), **329 TUs** for
  Class B over complete small grids in **two modes** (311 match, 0 mismatch,
  18 refused under the two named gates), **17 TUs** for the permutation cell
  (6 match, 11 refused, 0 mismatch). **0 census/gate disagreement everywhere**;
* 878-TU workload scan: match 6, **mismatch 0**, codegen-gap 0, port-error 0,
  capture-fail 7, **census/gate disagreement 0**, `.gl` binding invariants 0
  violations, cache validator **17 re-captured and agreed, 0 poisoned**;
* fixtures, per `c2rs census`: `w30_callseq_tail_intlike.cpp` **21/21**,
  `_neg` **0/13**; `mvp_call_seq_b.cpp` **18/18**, `_neg` **0/7**;
  `mvp_call_seq_neg.cpp` **0/2** after its two Class B rows moved into the
  positive fixture rather than being left as decoration.

**Census 474,103 → 481,876 / 2,462,571, 19.25 % → 19.57 %.** Differenced against
this branch's own base, master `9ec4871`; the concurrent FP branch (W27/W28/W29,
+9,137) is disjoint from these keys and its own delta is against the same base.

### The handoff, ranked and sized

1. **`call-ref-0x3A` — 5,335**, now the largest statement-call blocker. §6j
   already established the simple member (an explicit `return;`) gains 0, so the
   workload's are two branch records to **different** labels: a real control
   transfer, i.e. the control-flow rung.
2. **`call-multiarg-postop` — 13,425.** The largest call-family bucket overall
   and untouched by either rung here; it drained *into* from `call-postop-0x4B`
   last rung, so its composition has not been sampled since.
3. **`call-arg-computed` — 4,447.** Mixing a formal with a literal in a
   multi-argument call. Still uncaptured; §6j ranked it 4th and it has not moved.
4. **The r11-through-a-saved-register lowering.** 0 on the workload, but it is
   the only *characterized-but-unmodeled* thing this rung leaves behind and the
   three captures above are most of a rule. The missing capture: which saved
   register is the temp when two are saved and both are permuted.
5. **A computed argument out of a saved register** (`addi r3,r31,1`), and a
   computed first-call argument. Both need `select_text` to accept a base
   register other than the formal's entry one — one parameter, and it would close
   `callseq-saved-computed-arg` and half of
   `callseq-saved-with-first-call-setup` together.
6. **Class C (≥3 saved GPRs).** Unchanged from §6j's ranking: the helper
   externals, the tail-branch epilogue and §4.4's +2 label stride all at once,
   and it is the first rung where the symbol table is as much work as the code.

## 6m. The merge gate: the cross product, and the stride rule it refuted (2026-07-30)

`wt-class-b` (§6l) merged against master `5dc991d` (§6k, the FP register file).
One textual conflict — both branches appended a section before §7 — and the code
merged clean. The mandatory part was not the merge.

### What the merge made gradable, and what that cost

**mis-emit #12 was found by the previous merge, in the cross product of two
individually-green branches.** The same configuration existed again here: this
branch's shapes are *framed bodies*, the FP rung's are *FP leaves and stores*,
and nothing had ever graded the pair. So the merge gate generated **168 TUs**
pairing every FP shape (store leaf at both widths, arithmetic leaf, `fmr`,
pooled constant, mixed parameter list) with every framed shape (Class A, Class
B at one and two saves, W30's tail literal, the single framed call, and an int
store leaf as the control), **in both orders**, graded at `/O1`, `/Ox` and
`/Ox /Gy`.

At first grading: **10 match, 0 mismatch, 156 refused.** The refusal is the
label counter — `w28_fp_store_framed_neg.cpp`, the FP rung's own fixture, which
says in its comment that admitting the pair "is a change to the framed side's
label model, not to the FP classes". That is this seam, so it was taken.

### Capturing first refuted the rule that had just landed

mis-emit #12 was repaired with:

> anything that touches floating point consumes 2 — the stride goes with the
> register file, not with the body shape.

That is right for **one** FP function, which is what its capture had (one leaf
ahead of one framed function). It is wrong from two on: it predicts 4 slots for
two FP functions where c2 gives 3, and 6 for three where c2 gives 4.

The first attempt to measure this was unreadable for an instructive reason — the
`$M` numbers are seeded from `.gl`, so two TUs are only comparable if their
mangled names are the same *length*, and `?ints@@YAXPAUS@@H@Z` against
`?fps@@YAXPAUS@@M@Z` is not. (A second reason: **zsh does not word-split an
unquoted parameter expansion**, so a `$flags` variable holding `/Ox /GS- /Gy /c`
reached `cl.exe` as one argument and both "packed" and "`/Gy`" rows were
silently the same capture. Both rows agreeing exactly is the tell.)

The fix is to measure **seed-free**: put *two* framed functions in one TU and
read the **difference** between their labels. The seed cancels, the names never
have to match, and the leaf slots fall straight out. Eleven rows, `/Ox /GS- /c`,
every row `+1` under `/Gy`:

```text
  fr1;                      fr2    delta 4    leaf slots 0
  fr1; int_store;           fr2    delta 5    leaf slots 1
  fr1; fp_store;            fr2    delta 6    leaf slots 2
  fr1; fp_store fp_store;   fr2    delta 7    leaf slots 3   <- not 4
  fr1; int_store fp_store;  fr2    delta 7    leaf slots 3
  fr1; fp_store int_store;  fr2    delta 7    leaf slots 3
  fr1; int_store int_store; fr2    delta 6    leaf slots 2
  fr1; fp_arith;            fr2    delta 6    leaf slots 2
  fr1; fp_arith fp_arith;   fr2    delta 7    leaf slots 3
  fr1; fp_store fp_arith;   fr2    delta 7    leaf slots 3
  fr1; fp_store x3;         fr2    delta 8    leaf slots 4   <- not 6
```

> **Every function consumes 1 label slot (a framed one 4 packed / 5 under
> `/Gy`), plus ONE extra slot for the translation unit if any function touches
> floating point.**

The extra slot is `_fltused`, and the two facts `Function::is_float` carries —
where `_fltused` is emitted and where the extra slot goes — are now one fact
rather than two readers of one field, which is the third time that shape has
bitten this file.

> **The generalization this section drew from it is REFUTED (2026-07-31,
> `docs/LABEL_COUNTER.md` §2.1).** It read: *the extra slot is the one TU-level
> external an FP-touching function introduces, so the rule is **one slot per
> TU-level external**, the same rule `CODEGEN_FRAMED_CALLS.md` §4.4 measured as
> **two** for the `__savegprlr_N`/`__restgprlr_N` pair.* The two measured
> numbers above are unaffected — `_fltused` is +1 and the GPR helper pair is +2,
> both still exact, and the FPR pair the rule predicted at +2 does measure +2.
> What is wrong is the *reason*, in both directions: a newly pooled FP constant
> costs **+2** and introduces no external at all, a string literal costs **0**
> while introducing one, a materialised signed relational costs **+2** and mints
> nothing, and a `do/while` costs 1 where a `while` costs 2. The surcharge table
> that does fit is `LABEL_COUNTER.md` §1.1 — read it before extending the
> counter to any class not already in that table, because this rule would have
> licensed a widening that under-counts by 2 per pooled constant.

The `/Gy` pre-pass is confirmed at exactly `3 × funcs.len()` on all eleven rows,
unaffected by floating point.

**Why it could not be stated where it lived.** `IlFunction::label_slots` is a
*per-function* method and the `+1` is a *per-TU* quantity. No value it returns
can be right for both the first FP function and the second. It now returns the
per-function stride and `coff::plan_labels`, which has the whole function list,
applies the extra slot. That is also why the wrong rule looked so plausible: at
`n = 1` the two formulations are indistinguishable.

After the fix: **62 of 168 match, 0 mismatch, in all three modes.**

`w28_fp_store_framed_neg.cpp` is promoted to `w28_fp_store_framed.cpp` (5/5 in
class, `Port=Match`, now carrying a Class B function beside the Class A one).
Its own comment predicted the promotion, and a negative fixture whose rows are
admitted and byte-exact is decoration — the same accounting `mvp_call_seq_neg`
got in §6l. The new `_neg` holds the half still refused: an FP **arithmetic**
leaf, whose stride `label_slots` cannot report because `IlFunction` does not
carry whether the leaf pooled a constant. The eleven-row table says a
constant-free one is 2 and could be admitted; the record it needs is the FP
seam's, not the framed side's to restructure inside a merge, so it is a handoff.

### Census, key-by-key

**483,240 → 491,013 / 2,462,571 (19.62 % → 19.94 %)**, workload `dc3-decomp` at
`05ca6d09`. Additive to the function, and the two branches' deltas are **key
disjoint** — measured, not assumed:

| | key | delta |
|---|---|---|
| §6l (this branch) | `callseq-tail-lit` | **−7,771** |
| | `callseq-value-live-across-call` | −2 |
| | `call-postop-0x86` → `call-postop-op-0x33` | −1 / +1 (one body reaching one token further; net 0) |
| §6k (FP) | `expr-op-0x27` | **−7,927** |
| | `expr-load-type-8645` | −1,004 |
| | `opt-mode-00200001` / `opt-mode-00200101` | −140 / −66 |

`474,103 + 7,773 = 481,876`, `+9,137 = 491,013`. **Zero keys touched by both.**

### Gate evidence (merged tree)

`cargo test --workspace --release` **409 pass / 0 fail** · `c2rs bench` **157
pass / 0 fail / 0 error** · mode lanes over 157 fixtures `/Ox` **74**, `/O1`
**72**, `/O2` **72**, `/Ox /Gy` **72**, **0 mismatch in all four** ·
`scripts/expr_sweep.sh` **printed 5,922 = generated 5,922 = on disk 5,922, 0
mismatches** · cross-product grid **168 TUs × 3 modes, 0 mismatch** · 878-TU
scan: match 6, **mismatch 0**, codegen-gap 0, port-error 0, capture-fail 7,
**census/gate disagreement 0**, `.gl` invariants 0 violations, cache validator
**17 re-captured and agreed, 0 poisoned** · `census_gate.rs` unchanged at its
recorded **1 packed / 9 `/Gy`**.

Per-fixture `c2rs census`, both seams: `w27_fp_reg` 33/33, `w27_fp_reg_qual`
10/10, `w28_fp_store` 14/14, `w28_fltused_order` 5/5, `w29_fp_contract` 16/16,
`w28_fp_store_framed` 5/5, `mvp_call_seq` 10/10, `mvp_call_seq_b` 18/18,
`w30_callseq_tail_intlike` 21/21; negatives `w28_fp_store_neg` 0/11,
`mvp_call_seq_neg` 0/2, `mvp_call_seq_b_neg` 0/7,
`w30_callseq_tail_intlike_neg` 0/13, `w13_fparam_neg` 0/3, and
`w28_fp_store_framed_neg` 2/2 in class with the TU `NotImplemented` (the label
gate is per TU; `census_gate.rs` asks the per-function gate, which both pass).

### Two things measured and left alone

* **`census/gate` is 0 on the workload and 28 on a constructible grid.** All 28
  are the pooled-FP-constant-under-`/Gy` refusal, which lives in `c2-core`'s obj
  layout and has no parser counterpart. Verified pre-existing by building master
  `5dc991d` in a scratch worktree and running the identical grid: **the same 28**,
  while the per-function numerator rose 336 → 432. This branch's rungs add
  **zero** disagreement. It is the FP/obj seam's, and `census_gate.rs` already
  records it as `KNOWN_DISAGREEMENTS_GY`.
* **`c2_il::func::sy::gpr_reg_of` is dead code** (a build warning on master).
  It states `base + ix` with `base` = r4 for a member function, and every
  formal-to-register site in `c2-core::codegen` — including this branch's Class B
  save moves — instead uses `ARG_REGS[position in func.params]`. The two agree
  today because `this` is carried *in* `params`. But "a locator nobody consults
  is not shared" is `GAPS.md` §6 #2, and it has now produced two mis-emits;
  wiring it is the FP seam's call and is flagged rather than done.

## 6n. 2026-07-31 — the per-rung docs take over, and what the session established

**Census 491,013 → 655,245 (19.94 % → 26.61 %), +164,232**, across seventeen
merges, with **mismatch 0 and census/gate disagreement 0 at every intermediate
state**. Corpus `dc3-decomp` `05ca6d09`.

This section deliberately does **not** narrate those rungs. From this session on,
each rung carries its own document under `docs/rungs/`, indexed by
`docs/rungs/INDEX.md` (generated — `scripts/gen_rung_index.sh`) and enforced by
`crates/c2-harness/tests/rung_registry.rs`, which requires a positive and a
negative fixture graded N/N and 0/N. **A measurement that admits nothing is not a
rung** and belongs in `docs/` proper; that test rejected a merge for exactly this
and was right to.

What generalizes, and is not recorded anywhere else:

**A large blocking row is one of five things, and a first-blocker histogram
distinguishes none of them — they all stop at the same byte.** Check in this
order, cheapest first:

1. **A private limit inside a recognizer that already exists.** W35 (76 % of the
   head row), W38 (81 %), WSL. Three rungs running.
2. **A production misfiled under an opcode.** W36 — `p->m();` reached the
   assignment parser because the dispatch keys on a byte a member call lacks.
3. **Real, but far smaller than its size.** W37 eliminated 134,763 functions —
   7.2 % of everything blocked — **for free**, by crossing the row against two
   axes *already in the baseline scan*.
4. **Unmeasurable, because the instrument has no production for it.** A blocker
   with no production stops the completeness walk dead, so no such row can ever
   carry a `-whole` bit. Indistinguishable from (2) from outside; the repairs are
   opposite.
5. **Mis-described.** W41's row was scheduled as "the member call preceded by
   assignment statements" and contained zero of those.

**The standing locator check, in both directions.** W35 found a *private copy*
that refused **more** than its siblings; W38 found a *shared locator* that
**nobody else asked** (`eat_ctor_this_epilogue`, one caller since W19, worth
42,238 from a second production). **Neither is visible to any gate here**: they
emit nothing, so no byte compare sees them, and they agree with census by
construction, so no disagreement check does either.

**Ranking numbers, measured.** Row → realized: 67×, 67.8× (first-blocker rows),
2.62× (a `-whole` first-blocker key), 1.45× (a counterfactual successor),
1.0002× (a counterfactual *of the production being widened*). The last is the
rule: **when the instrument is a counterfactual of the production you are
widening, the ceiling IS the estimate** — all that remains is counting the
independent refusals between it and the emitter.

**Estimates missed seven times running.** Three causes, all recorded: borrowing a
rate across populations (W36 2.99×; the relational measurement missed by two
orders of magnitude doing this, with the lesson available); enumerating
sub-shapes when the winner is not among them (W35, W38, W41); and multiplying a
ceiling by a previous rung's realized fraction without asking what produced it
(WSL). The two methods that worked: **instrument the production**, and **cross
the row with the frame class and the control-flow class** — both free, both from
axes already in every scan.

**Generated sweep axes found six live wrong-bytes emits; hand-written fixtures
found none.** All six vary something that changes no operator and no shape —
cv-qualification on a formal, a callee's *return type*, `const` on a
copy-assignment source — and are therefore invisible to review.

**Five instrument defects, one shape: a shared artifact consulted or rewritten
under an assumption nothing enforces.** A gate binary never rebuilt (47 phantom
mismatches; the false-*green* direction is the hazard), an in-place mingw stub
(a "flake" that was a half-written PE), one shared `/tmp` dir across four
concurrent lanes, a **non-relocatable capture cache** (embeds its own absolute
path, faked a 6-TU port regression), and two sweeps in one outdir deleting each
other's cases mid-grade. All five now fail closed or are refused outright.

**Addendum 2026-07-31 (WRD) — reproduce the key before believing the name, and
the ranked item that retires.** Category (5) *mis-described* has a cheap
detector nobody had named: **write a probe TU that reproduces the census key from
hand-written source, before estimating.** It costs one capture. Applied to the
5,188-function row WCH *and* WCL both ranked **second** and both described as
*"this rung's body with the `B9 <formal>` swapped for a designator"*, the first
probe produced a key that occurs **zero** times on the workload — and that, not
any cross, is what said the row was something else. It is not a chain at all: it
is the compiler-generated **destructor with one sub-object plus one body
statement**, and `Blocker::ChainBind` is that body statement's own `99`. At the
workload's `/EHsc` the family mints a `__CxxFrameHandler` / `__ehfuncinfo$`
prefix, a second `.pdata` and an unwind funclet, so **0 of 5,188 is reachable**
(`docs/EH_RECORDS.md` §6). Two things generalize past the row:

* **Crossing with the operand type and the frame class is necessary and not
  sufficient.** Both crosses came back degenerate and reassuring here. They are
  axes *of the port's model*; a blocker outside the model passes every one.
* **A ctor/dtor capture taken at the fixture profile understates the workload's
  by a phase.** One sub-object statement and nothing else is a bare branch; a
  second sub-object, or one plus any other statement, is the whole of
  `docs/EH_RECORDS.md` §1–§5. `cf-expr-0x5C` — 309,804 functions, **17.4 % of
  everything blocked** — sits on that boundary.

And WCL's closing warning is **discharged**: both of its candidate rules for the
ascending link order are refuted, each by its own capture, and the rule that fits
all nine is *whether argument slot 0 is marshalled* — `docs/CODEGEN_ARG_PERM.md`
§7.

## 6o. The next phase is EH, and no histogram was ever going to say so

Measured 2026-07-31 (`docs/EH_RECORDS.md` §7), census delta **0** — an axis, not
a rung.

**233,526 functions — 13.1 % of everything blocked — sit behind the C++
exception-handling model**, and they are invisible to every ranking instrument
this project has. Of the 310,371 bodies carrying an EH marker, **75.2 % are on
the EH side** (`plus-stmt` 160,944 + `partial` 44,688 + `multi` 27,894) against
**24.8 % cheap** (`eh-bare` 76,845, of which 35,964 are already in class). Of the
blocked residue: **85.1 % behind EH.**

**Why no first-blocker histogram sees it.** The stock is spread across rows that
each look like ordinary expression work, and **the same census key straddles the
boundary** — `Ct1::Ct1(){}` and `Ct2::Ct2(){Init();}` are both
`expr-intrinsic-this-adjust`, the #2 blocker at 141,800, on opposite sides.
Crossed: `expr-bit-and` and `…-recv-object-then-branch-brtrue` are **99.9 %**
behind EH; `expr-intrinsic-base-member-addr` 62.5 %; while the #1 row
`expr-op-0x27` is only **8.2 %** and `body-0x9B` is 61.8 % cheap. **No expression
rung retires the EH stock.**

It is not the *cheapest* next thing — the cheap side's 40,881 blocked functions
are — but it is the largest thing that was hidden, and every expression rung from
here on should be sized against its EH share before it is scheduled.

Two corrections that came with it. `5C` is **not** a ctor/dtor trailer: it ends
**any** statement in which an object with a destructor became live, so
`int userfn(int a){ MemA s; g(a); return a+1; }` carries one with no sub-object
anywhere. And the cheap/EH split is **the count of such statements**, not the
kind of function. Both were caught in one capture by the rule §6n now carries:
*reproduce the census key from hand-written source before believing a sizing.*

**Larger than either side and unmeasured: `eh-unknown` = 288,072** bodies that
stop decoding before any marker, so the axis says nothing about them. Establishing
`0x64` (145,237) and `0x67` (45,631) is what shrinks that.

> **Both numbers above are BOUNDS, not sizes — corrected 2026-07-31 the same day
> they were written** (`docs/EH_RECORDS.md` §9.4). The split above was computed
> from a **statement count**, and that predicate is refuted from bytes:
> `int P(int a){ SE s; return a+1; }` has "another statement beside" the object
> and gets **no prefix, one `.pdata`, no `.rdata`, no funclet, `Value = 0`** — it
> is cheap. **The predicate is `maxState >= 1`**, where a state indexes the
> distinct sets of *live destructible objects* observed at an outbound control
> transfer. Statement count does not enter it.
>
> So **`eh-plus-stmt`'s 160,944 is an UPPER bound and the cheap side's 40,881 is
> a LOWER bound.** The direction is known; the magnitude is not, because
> re-scanning on `maxState` is a harness change. **Re-derive before scheduling
> either side.** The phase conclusion — that EH holds the stock and no
> first-blocker histogram sees it — is unaffected, since it rests on the marker
> count rather than on the split.
>
> `eh-unknown` was separately measured down to **137,187** (−52.4 %) by
> establishing `0x67`/`0x9A`/`0x64`, and **96.4 % of what left it carried no EH
> marker at all** — so that risk closed in the favourable direction.
>
> **The re-derivation is done — see §6r.** Every split figure in this section is
> superseded; the phase conclusion is not.

## 6p. The 288,072 were not hidden EH stock — WDR, 2026-07-31

Measured in `docs/IL_DECODE_REACH.md`, census delta **0** — an axis, not a rung.

§6o closed by naming its own largest open risk: **`eh-unknown` = 288,072**, bodies
that stop decoding before any marker, *"larger than either side"*, and it named
the lever — establishing `0x64` (145,237) and `0x67` (45,631). Both are now
decoded, from probes that reproduce their census keys from hand-written source
first.

> **Decode reach 86.5 % → 94.2 % (+188,794 bodies). `eh-unknown` 288,072 →
> 137,187, −52.4 %. And of the 150,885 bodies that left it, at least 96.4 % carry
> no EH marker at all** — the EH side grew by **5,197, 2.2 %**, on a population of
> 188,794 newly legible bodies.

So §6o's phase conclusion stands and its risk is closed in the *favourable*
direction: the unmeasured stock was ordinary expression work — virtual dispatch
and returning a class by value — not more EH.

Four things generalize past the rows.

* **`67` alone is worth ZERO, and that was predicted before the scan.** The
  45,631-body row becomes a 45,631-body `cf-expr-0x9A` row two tokens later; the
  decode reach comes back at the baseline **to the function**. `cf-expr-0x9A`'s
  own first-blocker row was 222. §6n's *"a large blocking row is one of five
  things"* now has a case where the estimate said so **in advance**, in
  `work/WDR/ESTIMATE.md`, and the scan confirmed it rather than discovering it.
* **Two opcodes measured separately under-price the pair by 17 %.** `67`+`9A`
  alone is +37,639, `64` alone is +119,547, together **+188,794** — 31,608 bodies
  carry both and neither opcode alone moves one of them. Rows are not additive
  even when the productions are independent.
* **The ceiling taken neat was right to 1.1 %; both discounts were wrong.**
  190,868 predicted, 188,794 realized. Fourth recorded instance.
* **A width below its escape boundary is not a measured width.** `67`'s vtable
  offset was `00 04 08 0C 34 38` at every witness anyone had — all below `0x80`,
  where a plain byte and a signed varint are *the same bytes*. It took a
  hand-written class with forty virtuals to separate them, and the plain-byte
  reading costs 926 bodies on this corpus. Any field whose witnesses all sit on
  one side of an encoding boundary is undetermined, however many witnesses there
  are.

And the block IR is **still worth exactly 718 functions**: the `+expr-modeled`
column is unchanged to the function in every shape row. 190k more decoded bodies
moved it by nothing, by construction.

## 6q. The width that was two bytes short — WVB, 2026-07-31

Measured in `docs/IL_TYPE_WIDE_TAG.md`, census delta **0** — an axis, not a rung.

§6p closed by ranking one open item: the 129 bodies filed `cf-vbind-type-cflow-jump`,
*"the only row in this work that says a width might be incomplete rather than
merely unimplemented"*. A width was incomplete. **It was not `9A`'s.**

> **`read_type` read a five-byte `.ex` type as three.** A tag with **bit 6** set
> carries one extra byte before the kind — `.sy`'s reader has required it since it
> first bound on a real TU, and the `.ex` reader never had it. Decode reach
> **94.2 % → 97.2 % (+75,733)**; the undecoded residue's distinct keys **384 → 8**;
> census **685,882, +0**, `fn_blockers` **719 keys, every delta zero**.

Five things generalize past the row.

* **A first-blocker histogram cannot tell a construct from a resync**, and no
  cross with the frame, control-flow or EH class can either — a resync passes
  every axis, because every axis is computed after it. **376 of the 384 rows on
  this axis were resync artifacts**, including `cf-expr-0x82` (23,254), which
  §6p's own table ranked second and described as *"in §13's residue list"*.
* **`9A` is single-form, and the row named after it contained none of it.** The
  `9A` the walk stopped on is the fourth byte of the preceding type. The 69,246
  do not re-split: with the bodies around them legible the separation *grows* to
  **70,206**, and `9A <TYPE> <varint>` still decodes nothing `9A <TYPE>` does not.
* **"Honest refusal, not desync" is a claim about where a walk started going
  wrong, and the falsification test does not answer it.** Landing off the tail is
  sound evidence that something is wrong and no evidence at all about which token
  it was. §6p asserted the stronger reading for 183 bodies and all 183 were
  desyncs.
* **One rule, two locators, and only one of them had it.** Nothing compares
  `.sy`'s type reader with `.ex`'s; they read different containers and agree by
  construction on every type that is not wide. The `.sy` side had the rule
  *documented, measured and load-bearing* for months.
* **A constant that never varies at three witnesses can still be a field.**
  `.sy` requires the wide mark to be the literal `81`; `.ex` has `84` too, 106
  times, and requiring `81` refuses 36 real bodies. The same shape as `CA 81 0D`
  refuting the literal `C6 81` prefix one container over — twice now, on the same
  field.

And the EH axis moves the *other* way from §6p's: these 75,733 newly legible
bodies are **54 % on the EH side** (against WDR's ≥ 96.4 % `eh-none`), because a
wide tag means a class with a vtable. **EH stock 238,723 → 276,809, +16.0 %** —
§6o's phase conclusion is reinforced, and it was under-counted while this width
was short. The block IR is **still worth exactly 718 functions**: the
`+expr-modeled` column is unchanged to the function in every shape row, a fifth
time.

## 6r. The EH split, measured — and the cheap side is not a phase (EHMS, 2026-07-31)

Measured in `docs/EH_RECORDS.md` §10, census **691,744 / 2,462,571 = 28.09 %**,
delta **0**, mismatch **0**, disagreement **0** — an instrument, not a rung.

§6o's split was computed from a **statement count**, refuted from bytes the same
day (§9.4), and left as *"`eh-plus-stmt`'s figure is an UPPER bound and the cheap
side's is a LOWER bound; the direction is known, the magnitude is not."* Both
bounds are now measured, and **the cheap side was never a lower bound at all.**

> **EH stock 276,810 → 237,180 (−14.3 %). Cheap side 77,836 → 117,463 (+50.9 %),
> and 41,865 → 81,492 blocked (+94.7 %). `eh-partial` 4,375 → 3.** The predicate
> is `maxState >= 1` — a call while a destructible object is live — read at the
> `4C` that closes an argument list, not at the `BD` that opens the call.
> Graded against 46 hand-written functions' own objs: **46/46**, against the
> statement rule's **35/46**.

**The aggregate hides the error.** `93,075 of 354,646` marker-carrying bodies —
**26.2 %** — were on the wrong side: 66,336 filed expensive that are cheap,
26,724 filed cheap that carry the whole record set. The two errors partly cancel,
so a predicate wrong on a quarter of the population moved the headline by 14 %.
That is the worst shape an error can take, because the small aggregate invites
you to call the correction a refinement and keep the row-level table.

**Do not keep the row-level table — two of its rows INVERT.**

| census key | §6o `%EH` | measured | what §6o said |
|---|---:|---:|---|
| `body-0x9B` | 0.1 % | **62.5 %** | *"61.8 % `eh-bare` — 16,738 functions that need no EH model at all"*. Measured: **one** is cheap. |
| `expr-intrinsic-base-upcast` | (cheap, 8,277) | **42.5 % EH** | **zero** cheap. |
| `expr-intrinsic-base-member-addr` | **62.5 %** | **26.0 %** | *"the board's #3 row is behind EH too"*. It is the **largest cheap row**, 41,678. |
| `expr-intrinsic-this-adjust` | 25.7 % | **40.4 %** | under-stated by half |
| `expr-op-0x27`, `expr-bit-and`, `…-brtrue`, `body-cflow-label` | | unchanged to ±2 pts | these four stand |

**RETIRED: "the cheap side is the cheapest next thing."** §6o and §7.5 both said
EH is not the cheapest next thing because *"the cheap side's 40,881 blocked
functions are"*. There is **no cheap-side rung to schedule**: 88.4 % of its 81,492
is three general expression rows already on the board
(`expr-intrinsic-base-member-addr` 41,678, `expr-intrinsic-this-adjust` 16,051,
`expr-op-0x27` 14,308), and in each the EH-marked bodies are a minority slice —
36.6 %, 11.8 %, **3.5 %**. Widening any of those rows retires its cheap-EH slice
**for free**, because `maxState = 0` means `/EHsc` costs those functions nothing.
They should never have been counted as EH work. The two rows §6o nominated for
the job are on the other side entirely.

**§6o's phase conclusion stands, on a better number.** 237,180 functions —
**13.4 % of everything blocked** — need the whole of §1–§5, spread across rows
that each look like ordinary expression work. And the control group is now exact
rather than argued: **every one of the 237,180 is blocked, and every one of the
35,971 accepted marker-carrying functions has `maxState = 0`.** The port has
never accepted a function that needs an EH record, and the axis says so without
being told.

Three things generalize past the rows.

* **A predicate that is refuted carries no information about the size of its own
  error.** The estimate written before this scan
  (`work/EHMS/ESTIMATE.md`) put both headline numbers inside their intervals and
  within 7 % — but every miss was the same miss, *"I under-estimated how wrong the
  old axis was"*, and the single worst one was the prediction that copied the old
  table's **row identities** forward. Correcting a magnitude off a refuted prior
  is fine; correcting a *ranking* off one is not.
* **`return` carries no `4B`, and that hole was documented and still cost 26,724
  functions.** `EhMarkers::other_stmts` says in as many words that a body whose
  only extra statement is a `return` reads as bare. `SE s; return gp(a);` is that
  body, it carries the whole record set, and the caveat sat one scroll above the
  number it invalidated. A named limitation in a doc comment is not a measurement.
* **The instrument printed a control group as the top blocker.** The EH cross-tab
  spelled accepted shapes and blockers into one namespace, so
  `eh-bare|empty-dtor-delegation` — **27,501 functions the port already emits** —
  sorted above every real blocker on the board and was within a step of being
  scheduled. Fixed: rows name their population and there is a `|BLOCKED`
  subtotal. Second instrument defect in three sessions where the *shape of the
  key* was the bug.

## 6s. The wave that measured its own instruments — 2026-07-31

Four lanes, **census delta 0**, and that is the finding rather than a
disappointment: two rungs were declined on measurement and the third was an
instrument repair. What the wave actually produced is a corrected board and
**four more instruments in the "absence read as success" class**.

### The ranking premise was false

The wave was ranked by key *position*, not row size — the rule §6n earned. A key
ending `:eof` is a refusal raised *after* the parse reached the end of the
segment, so every function under it is grammar-complete by construction. That is
real: it is how `expr-out-of-class-bare-nonfirst-formal:eof`'s 43,319 produced an
estimate that landed inside ±700.

**But `Block::feature` prints `<ctx>:eof` for ANY block with `byte: None`**, and
at least one site hardcodes that at a mid-segment offset. The suffix is a
*rendering*. No argument was needed to see it — **4,466 of
`assign-dst-not-formal:eof`'s 13,887 bodies are `cflow-loop` bodies**, which
cannot be at the end of anything. The row measured **+0 twice** (delete the gate;
delete the gate *and* the check behind it), and it had already measured +0 once
in the 2026-07-30 `.sy` review. It re-entered the ranking on size alone through a
found-and-not-taken table.

The defect is **general and named but not fixed**: `assign-subst-overflow`,
`assign-ret-nonformal`, `expr-repeated-leaf`, `fn-varargs`, `lo-marker`,
`param-width-undetermined`, the `callee-unresolved-*` family, `opt-mode` and
everything `straight_line_out_of_class_ctx` returns all print `:eof` on the same
terms. Some genuinely *are* at the segment end; the renderer cannot say which.
Repairing it means giving `Block::feature` the segment length — one shared
renderer touching every recorded key, so it is a **serial** merge, never a
parallel seam. **Until then no `:eof` row may be scheduled on its position.**

#### …and the repair, with the board it corrects — 2026-07-31

Done, census delta **0**, and the `:eof` rows may be scheduled again — the ones
that are still `:eof`. `Block` now carries `seg_len` beside `off`; the renderer
earns `:eof` from `off == seg_len` and prints `:mid` otherwise. Both routes to
that offset are exact rather than approximate: `blk` reads `seg.get(p)` at the
live cursor, and the two post-parse gates (`callee-unresolved-*`, `opt-mode`)
state their fact positively with `Block::at_end`, which is sound because
`eat_fn_tail` returns `Ok` *only* at `p == seg.len()`. `Block::refuse(seg, off,
ctx)` derives the length from the segment, so it cannot be typed wrong, and
adding the field turned all 98 construction sites into compile errors — the
enumeration was obtained from the compiler, not remembered. Full write-up and
the per-key table: `docs/GAPS.md` §6.

**63.4 % of the signal was false.** Of 26,935 functions under a `:eof` key,
9,848 are genuine and **17,087 are not**. The rows this section put on the board
as "sitting on an unverified `:eof`" resolve as:

| ctx | claimed | genuine `:eof` | now `:mid` |
|---|---|---|---|
| `param-width-undetermined` | 6,974 | **0** | 6,974 |
| `call-arg-computed` | 5,544 | **5,537** | 7 |
| `expr-out-of-class-bare-nonformal` | 4,127 | **4,127** | 0 |
| `call-args-none` | 3,299 | **0** | 3,299 |
| `this-undetermined` | 2,568 | **0** | 2,568 |
| `param-multi-reg` | 1,851 | **0** | 1,851 |
| `expr-ptr-arith` | 1,678 | **0** | 1,678 |
| `call-arg-outer-formal` | 695 | **1** | 694 |
| `expr-out-of-class-formals9` | 125 | **125** | 0 |
| `module-end` | 48 | **48** | 0 |
| `formals-marker` | 16 | **0** | 16 |
| `call-arg-nonformal` | 8 | **8** | 0 |
| `mcall-framed-args` · `callee-unresolved-tail-call` | 1 · 1 | **1 · 1** | 0 |

So the largest `:eof` row on the board is now `call-arg-computed:eof` at 5,537 —
a *statement-position* call with a computed argument, whole body parsed — and
`expr-out-of-class-bare-nonformal:eof` at 4,127 is confirmed genuine, which is
what the §6n premise was originally earned on. Six of the seven rows this
section listed as unranked are **pre-parse** refusals and were never eof at all:
a gate that runs before the body is read cannot be at the end of it.

Two keys are **both** — `call-arg-computed` and `call-arg-outer-formal` reach
one predicate from a statement call (plumbing already consumed → `:eof`) and
from a value call (plumbing still ahead → `:mid`). That is the split being a
real property of the parse rather than a relabelling, and it is why the
complement had to be its own bucket instead of a merge into `<ctx>-0xNN`.
Reproduced from hand-written source through the live toolchain before the table
was believed. Pinned in-tree by
`body::tests::the_eof_suffix_is_earned_by_reaching_the_segment_end`, which
asserts the positive and the negative — a renderer that printed `:eof` for
everything passes the positive alone.

The D6 frame axis corroborates for free, used to **refute** rather than to rank:
a genuine `:eof` row has had its whole body consumed, so every function under it
must carry a call count the grammar can produce. `expr-out-of-class-bare-nonformal:eof`
is `calls-0` on 4,127 of 4,127 and `call-arg-computed:eof` is `calls-1` on 5,537
of 5,537; 9,846 of the 9,848 agree overall. Every `:mid` row is a mixture
instead, and 2,883 of `param-width-undetermined:mid`'s 6,974 are `calls-2plus` —
the same tell that caught `assign-dst-not-formal` through `cflow-loop`, one
query against a scan already on disk.

The `-whole` half of the signal survives and is now proven: WDA established by
two controlled pairs (a pointer formal moves the suffix; a whole extra string
does not) that `-whole{,2,3,4}` counts **distinct granted constructs**, not
occurrences. So the unit of work is `{form} ∪ granted` — *not* the receiver form,
which was the coordinator's hypothesis; `recv-load` alone spans 47 construct
sets. `fn-tail-0xNN` survives too, for the original reason: `eat_fn_tail` is what
every accepted shape reaches last.

### Ranking off a refuted axis reproduces the refutation

§6r re-derived the EH split on `maxState`. Two rows on the board **invert**:
`body-0x9B`, sized at 16,738 cheap, has exactly **one** cheap function;
`expr-intrinsic-base-upcast`, sized at 8,277, has **zero**. Both numbers came
from the statement-count axis *after* it had been refuted, and were then written
into a lane brief as "the cheap side's head rows, 76 % of it". Measured: 19.7 %.

The general statement, which cost a wave: **a ranking derived from an axis whose
predicate is known-refuted reproduces the refutation, and looks like ordinary
data while doing it.** The conclusion drawn from those rows survived; every
number under it did not.

Related, from the same measurement: the predicate was wrong on **26.2 % of the
population in both directions** and the errors partly cancel, moving the headline
by only 14 %. **Agreement in aggregate is not evidence a classifier is right.**

### Four more instruments in the absence-read-as-success class

1. **The capture-cache key omitted its own root.** A cached reference obj embeds
   the capture directory's path (c2 is invoked `-Fo` into it), so a relocated
   cache served foreign bytes *as a `mismatch`* — an alarm pointing at the port
   while the port was fine. Proof of the fix is the strong form: the change
   invalidates every entry, so the verification scan ran **0 hit / 878 miss**,
   re-capturing every TU cold, and reproduced the baseline exactly.
2. **The EH cross-tab** (§6r) — a control group at the top of a ranking table.
3. **The gap scan had no binary identity**, only tree identity, while every sweep
   lane pins a run-private copy and prints a content sha. It is the command that
   produces the census figure this project publishes. Now prints a digest.
4. **`sweep_mode.sh` reported a green on a run that graded nothing** — and this
   one was written *by the coordinator, after* the guards for the other three. A
   relative outdir produces `z:work\…` paths cl.exe cannot open; capture-fail was
   13707/13707; and every check `sed`s a number that is absent and parses it as
   zero. The pre-flight `SKIP` check does not cover it, because SKIP means the
   toolchain is *absent* and this was the toolchain *present and refusing
   everything*.

**The fix that generalizes is a POSITIVE check — "the run must have GRADED
something" — never an enumeration of the ways a run can be empty**, because the
next empty run will be empty in a way nobody enumerated.

Two corollaries worth carrying. **A table test proves completeness only over the
list it was written from**: the cache-key test builds `CaptureCache` with a
hand-written context and never calls `new()`, so every *documented* component had
a case and the undocumented one had nowhere to fail. And **tmpfs inode
exhaustion presents as `ENOSPC` with tens of GB free** — the sweep lanes exhaust
`/tmp`'s fixed inode count, and two independent agents misread it as disk space.

### What the generated corpus was hiding

`scripts/sweep_mode.sh` runs the generated cases through `c2rs gap --flags-file`
at an arbitrary mode. The intersection *generated case × `/EHsc`* had been empty:
`expr_sweep` drives `c2rs diff`, which hardcodes `/Ox /GS- /c`; `mode_lane` takes
flags but grades fixtures. So the axes that have found **four live mis-emits the
hand-written corpus never found** had only ever compiled with exceptions off.

First run: **mismatch 0**, but **census/gate disagreement 155** — an invariant
that reads 0 on the workload and 0 on fixtures and had never been evaluated here
at all, because `expr_sweep` greps `c2rs diff`'s per-case verdict for `*Mismatch*`
and the disagreement check lives only on the `gap` path.

The first run varied **two things at once** (`/EHsc` *and* `/Ox`→`/O1`) and was
nearly filed as an EH finding. Separated over identical cases: `/Ox` 155,
`/Ox+/EHsc` 155, `/O1` 158, `/O1+/EHsc` 158. **`/EHsc` contributes zero.** The 155
sat at the profile `expr_sweep` had been running all along.

153 of them were one off-by-one: `chain.rs`'s `IlOp::Mul if rhs_lit || (i == 1 &&
lhs_lit)` — ops are postfix and a two-leaf chain puts the operator at index **2**,
which the `Sub` arm on the *adjacent line* already knew. `return 3 * a;` censused
in class while the port refused it; `return a * 3;` was correctly refused by both.
No test caught it because the table covers `[Load, Lit, Mul]`, the form the rule
was derived from. Fixed; workload cost **verified** at 0 across all 722 keys.

Disagreement is now **7**, fully characterized: **4** are a local variable
spanning statements (its *flat* form is generated by no fragment and was found
only by hand, so that class is under-counted by an unknown factor), and **3** are
an FP leaf beside a framed int function under `/O1` — the refusal frontier of
§WEC appearing as a census **over-claim** rather than a refusal, the first time
that seam has been measurable instead of merely absent from coverage.

### Also standing open

~~The `/EHsc` mode lanes **work and are green, but are not standing lanes** —
nothing enumerates them, and the four lanes recorded throughout these docs are
`/Ox`, `/O1`, `/O2`, `/Ox /Gy`, none of which compiles `/EH`, on a workload that
compiles `/EHsc` on every TU.~~

**CLOSED 2026-07-31 (WGATE).** The lane list is data — `scripts/lanes.txt`, 12
lanes, six code-shape configurations crossed with the EH axis — `scripts/gate.sh`
is the one command that runs it, and `crates/c2-harness/tests/lane_registry.rs`
fails if the shipped registry stops carrying an `/EH` lane, stops *varying*
`/Oi`, or loses `/O1 /EHsc` by name. **Adding a lane never closed this; only
enumerating them did** (`docs/GAPS.md` §7).

## 6t. The gate was simulating the wrong machine — WAFF, 2026-07-31

The second census over-claim (board #103) turned out to be **one variable read at
three arms**, plus one producer that never called an emitter that already
existed. Both halves are worth recording, because they are opposite defects that
presented as a single row.

### The refusal, named

`c2rs census` already prints the port's own reason, and for the four filed cases
it printed **two** different ones:

```
int f(int a,int b){int x=a+1;int y=x*b;return y;}     multiply by a constant strength-reduces
int f(int a,int b){int x=a+1;x=(x)*b;return x;}       multiply by a constant strength-reduces
int f(int a,int b){int x=a+1;{return x+b;}}           reg+reg add with a pending immediate offset
int f(int a,int b){{int x=a+1;{int y=x+b;return y;}}} reg+reg add with a pending immediate offset
```

Two messages, one cause. `select_text` is **affine**, not a stack machine: an
operand is a register plus one immediate it still *owes*
(`Operand::RegOff { base, off }`), and there is no way to materialize that
immediate before a reg-reg instruction fires. `combine` therefore has three
`off != 0` refusal arms — `Add`, `Sub`, and the `Mul` **catch-all**, whose
message says "multiply by a constant" and is simply wrong here: in `(a+1)*b`
both operands are registers. The misattribution is why the row was never read as
a gap.

The gate, meanwhile, is `chain_form`, which simulates a generic **two-deep
operand stack**. That class is strictly wider than the affine one, so the
disagreement is not a bug in either — it is two different machines, and the
census was reporting the wrong one's answer.

### Which verdict — and it is BOTH, split by canonicalization

The hand-written flat controls are what separate them. Neither has a local at
all, so the locals production cannot be implicated:

```
int f(int a,int b){ return a+1+b; }    -> census in class, Port=Match
int f(int a,int b){ return (a+1)*b; }  -> census in class, Port=NotImplemented
```

* **Additive-only streams: the port was UNDER-IMPLEMENTED.** `canonicalize_chain`
  folds the literal to the end (`a+1+b` → `(a+b)+1`), which removes the pending
  offset entirely, and the result is byte-exact — `return a+1+b;` has always been
  `Match`. `body/mod.rs` called it; `shapes/assign.rs` did not, running only the
  pre-canonicalizer fallback checks. So one and the same resolved stream
  `[a, 1, Add, b, Add]` was byte-exact when written flat and refused when it
  arrived through a local. **The emitter existed, so by the ceiling rule the
  ceiling was the estimate** — this is not a rung, it is a producer that did not
  call the locator. `canonical_chain_for_codegen` is now the single decision and
  both producers call it.

* **`*` mixed with `+`: the census was OVER-CLAIMING.** `canonicalize_chain`
  declines these by design (`mul && addsub`), so the pending offset survives to
  the `Mul` and nothing rewrites it. No emitter exists and none is implied.
  `affine_serial_ok` now reproduces `combine`'s operand algebra in the parser, so
  the census refuses exactly what codegen refuses.

The ordering is load-bearing: the affine gate runs **after** canonicalization, on
the stream codegen actually sees. Run before it, it refuses `return a+1+b;`,
which is a shape the port emits byte-exactly.

### Numbers

Generated corpus at the `c2rs diff` profile (`/Ox /GS- /c`), **13,707 cases
submitted, 13,618 graded** — the 89 ungraded are const-member stores the *front
end* rejects, a constant of the corpus and unchanged on both sides:

| | before | after |
|---|---|---|
| census/gate disagreement (cases) | 4 | **0** |
| mismatches | 0 | **0** |
| `Port=Match` | 9,577 | 9,583 |
| in-class functions | 11,623 / 15,619 | 11,625 / 15,619 |

Six cases moved `NotImplemented` → `Match` and **none lost `Match`**. The
in-class net of +2 is +4 widened and −2 withdrawn, and the two directions are
reported separately on purpose: a net figure would hide the withdrawal, which is
the half that makes the number honest.

Four of the six gains are not the pending-immediate axis at all — they are
`int x=b+a; …`, which `assign.rs` used to refuse as `assign-noncanonical-order`
and the shared canonicalizer now rewrites. Sharing the locator paid twice.

At the profile the board tracks the number at — `scripts/sweep_mode.sh /EHsc`,
i.e. `/EHsc /O1 /GS- /c`, 13,949 TUs, 9,666 match, **mismatch 0**, 13,860 graded
— the disagreement is **7 → 3**. The residue of 3 is *not* this class and is
already characterized: an FP leaf beside a framed int function
(`81-fp-beside-framed.py`), the §WEC refusal frontier §6s named. Confirmed by
reading `fn_gate_refusals` out of the scan rather than by subtracting counts.

**On the real workload the change costs and gains nothing.** The 878-TU dc3 scan
reads **691,744 / 2,462,571 in class (28.09 %), mismatch 0, census/gate
disagreement 0** — byte-identical to the same scan run with the pre-change
binary, which is how it was checked rather than by comparing against a number in
an older log. The class is a generated-corpus phenomenon there, exactly as the
`i == 1` off-by-one in §6s was. That is the argument for the generated sweep
existing at all: it is the only instrument that reaches these shapes, and a
workload histogram would have ranked this at 0 forever.

### The sweep was under-counting a class it could not reach

The flat form — a local defined by one statement and consumed by a **compound**
expression in the next, with no brace between them —

```
int f(int a,int b){ int x=a+1; return x+b; }
```

is generated by **no fragment**. `43-locals-scopes.py` sweeps the neighbourhood
densely, but its flat arm returns a *bare* local (`int x=a+1;return x;`) and
every compound return it produces is wrapped in a scope
(`int x=a+1;{return x+b;}`). The shape was reachable only by writing it out by
hand, which is how it was found — so the class's disagreement count was
**unknown, not zero**.

`scripts/sweep.d/47-flat-locals.py` closes it: **242 cases**, 242 graded, 83 in
class, 0 mismatches, **0 disagreements**. Both operand orders are enumerated for
every operator, because a rule derived in one order and graded only in that order
is the failure `10-int-chains.py`'s commutation pairs already record — and the
same one that hid the `i == 1` off-by-one in §6s. Corpus **13,707 → 13,949**.

### What is asserted now that was not

`affine_serial_ok` is tested in the **under-claiming** direction as well as the
over-claiming one — every stream `c2_core::codegen::straightline`'s own
byte-graded tests prove it lowers is asserted to survive the gate. That direction
had nothing testing it, and it is the invisible one: a gate that refuses too much
reports a smaller numerator and every differential still passes, because a
refused function is never graded.

## 6u. The `-whole` family, decomposed — WRANK, 2026-07-31 (measurement only, nothing merged)

71,767 functions across 96 keys had been *confirmed* and never *decomposed*. The
lane that decomposed them merged no code and shipped no rung; its output is a
ranking, and three of its findings change how the rest of the board reads.

### The instruments came first, and one of them failed

**Distinct-source-function attribution is not measurable on this workload** —
stated positively, because the failure mode is that it looks measurable. A
per-function name side-channel returned **2 distinct names for 81,615 blocked
instances** (81,614 `(unnamed)`). `Bindings::positional` only reports names when
`names.len() == segs.len()`, and every real TU is unpaired — `src/App.cpp` has
3,752 `.gl` names against 9,033 segments. Read naively the table said "distinct
= 1" for an 18,926-function row, which is precisely the artifact-shaped answer.
A hex-window hash is token-polluted (6,494 distinct windows for 6,495
instances). What is needed is a **body hash with the per-TU token fields
masked**. `seg_len` gives a clean *lower* bound only, and it is a striking one:
three rows are a single byte length at 100 % across 700+ TUs — near-certainly
**one header inline apiece, i.e. one refusal, not seven hundred**.

**The `-whole{k}` suffix over-counts by one on ~27,600 functions.** `mcall.rs`'s
completeness walker charges a `Blocker::Type(Ptr)` grant for a pointer argument,
while the production it is measuring already accepts one — and so does the
walker's own sibling `eat_admitted_type`, **in the same file**. `eat_int_like_or_admitted`
is the only one of the three that refuses a pointer: category (1) of §6n in its
*shared-locator-nobody-else-asks* form. Verified positively from hand-written
source rather than inferred. The counterfactual moves 217 keys net 0 and leaves
census, blocked total and disagreement identical — and the board's **top row**,
`data-addr-2sym-then-plain-call-and-type-ptr-whole2`, becomes `…-plain-call-whole`:
need = 1, not 2. **Its "second construct" was never a construct.**

### The 55.4 % data-symbol claim: verified, and its attribution refuted

Directly measured rather than summed from three named groups: **40,871 of
71,767 = 56.95 %** of the family renders at least one data designator — a slight
*undercount* in the original, which omitted 1,089 functions in seven smaller
keys. But it is not one seam, and the split is almost exactly half:

| | functions | share | unruled blockers? |
|---|---:|---:|---|
| single symbol | **20,505** | 50.2 % | **no** — one REFHI/REFLO+PAIR quad, the shape `coff.rs` already emits for pooled FP constants |
| two or more | **20,366** | 49.8 % | **yes** — §17.3(b) anchor choice and (c) argument scheduler, both fitted hypotheses with no derived rule |

§17.3(b)/(c) are properties of the **multi-symbol case only**. Quoting them
against the whole 55.4 % makes half the population look unbuildable when it is
not. **The takeable half is 20,505 — not 0, and not 40,871.**

### The instrument that actually decomposes the family

A **production first-blocker** tag at every non-committal bail in the three
member-call productions says *which limit inside the shipped recognizer refused*.
No census key does that, and §6n's category (1) — a private limit inside a
recognizer that already exists — is by far the most common answer to "what is
this big row", six rungs running. **It has now been built and thrown away twice.**

| production first blocker | functions | share |
|---|---:|---:|
| none of the three productions entered | 30,475 | 42.5 % |
| `eat_call_args` — an argument the port cannot spell | 14,621 | 20.4 % |
| `mcall_chain`: receiver not a plain `B9` load | 11,877 | 16.5 % |
| framed post-op | 8,062 | 11.2 % |
| body does not END at the call | 3,326 | 4.6 % |
| inner call-args | 2,214 | 3.1 % |
| tail: receiver not a plain `B9` load | 557 | 0.8 % |
| tail: return plumbing (value) | 409 | 0.6 % |
| everything else | 226 | 0.3 % |

The 30,475 splits into **21,666 correctly not applicable** (the `data-addr`
family) and **8,809 that are member-call forms and reach no production at all** —
so no widening inside any of them can move one function. That 8,809 was **not
sized and not reported as 0**; it needs a dispatch-ladder tag.

All five top rows were reproduced from hand-written source through the live
toolchain **before** the table was believed.

### Ranked, with the framing corrections that fell out

1. **Single-symbol data address — 15,583**, three shapes behind **one** unbuilt
   emitter: `recv-load-then-call-data-addr-1sym` 10,540, `data-addr-1sym-then-plain-call`
   2,718, and a bare global read 2,325 split out of `expr-out-of-class-bare-nonformal:eof`.
   The ceiling is the estimate: for the 10,540 the member-call emitter **already
   exists** and 100 % bail at `eat_call_args` — one variable, one place. This
   **corrects** the earlier framing that the row sat behind "the member-call
   emitter *and* the data-symbol emitter, both unbuilt". It is behind one.
2. **The statement sequence with a member call — 3,326.** Its largest key,
   `recv-load-then-type-ptr-whole` (2,107), is **mis-described** — category (5).
   The name says "a pointer"; the construct is a statement sequence, and 2,106 of
   2,107 bail at *body does not end at the call*. Both halves are built.
3. **The `-whole{k}` repair** — census delta 0 by construction, but ship it
   **with** a production first-blocker key or it merges (2) into `recv-load-whole`
   and creates a conflated 8,602 bucket.
4. **Class B, a value live across a call** — 8,062 at the framed post-op, plus
   3,197 and 8,656 behind it. A **frame class, not a rung**; schedule as a phase.
   `recv-load-whole` (6,495) is category (6) and refuted as a rung on the same
   grounds: 99.5 % of it is this.
5. **Tag the body-dispatch ladder** — an instrument, and 8,809 functions cannot
   be ranked at all until it exists.
6. **`bool`/`char` argument to a member call — 786.** `parse_expr_classed`
   already computes the class; `eat_call_args` calls the class-discarding
   `parse_expr`. Category (1), small and cheap.

**Not scheduled:** the two-symbol phase (20,366), `fn-tail-0x26` (refuted at
zero — all `calls-2plus`), and `call-arg-computed:eof` (owned by a live lane).

## 7. Invariants (do not break)

- **Real c2 is the sole judge** — `port(IL) == c2(IL)` byte-exact, timestamp
  zeroed. The port never grades itself; no mocks, no fuzzy gates.
- **Fail closed** — outside the accepted class: `NotImplemented`, never a
  mis-emit. Negative-neighbor fixtures are as load-bearing as positive ones.
- **Coverage-bounded honesty** — a green run claims the tested corpus only;
  the census number is the public claim.
- **std-only, zero external crates**; toolchain-absent degrades to clean SKIP.
- **Nothing binary committed** — no IL (`_CL_*`, `*.il`), no objs, no machine
  paths, no MS binaries; only fixture `.cpp` and docs.
- **Commit small and often**, verified; never push unless asked.

## 8. The plan to parity, re-derived — 2026-08-01

An independent audit re-measured the plan against HEAD `829517a` and refuted its
**steering metric**, not its engine. The engine — counterfactual-sized rungs
behind a fail-closed differential, with a zero-mismatch record — is working and
does not change. What changes is what we publish as distance-to-done.

### 8.1 "Census → 100 %" is the wrong target, and the reason is measurable

**The denominator counts work that produces no bytes.** The audit parsed all 871
cached reference objs and counted `.text` COMDATs: **178,097 emitted functions
against 2,462,571 IL bodies — 7.23 %**, independently confirming §6's 7.3 %
figure (0.5 % apart, consistent with corpus-HEAD drift). The median TU carries
1,509 blocked bodies against **139 emitted**. A body c2 never emits needs two
things from the port — to be *skipped*, and for the skip decision to be *right*.
It does not need a byte-exact lowering.

**The numerator's overlap with emitted code was unmeasured, and that was the
softest number in the project.** Bounded from the scan plus per-TU emitted
counts, the in-class ∩ emitted overlap lay in **[22, 173,149] of 178,097** —
nothing measured whether 28.31 % covered 0.01 % or 97 % of the code c2 actually
emits.

> **MEASURED 2026-08-01 (WEMIT): 34,083 of 178,968 emitted functions in class =
> 19.04 %**, true value in **[19.04 %, 28.94 %]** — an interval 3.4 % of the
> denominator wide instead of 97 %. The fear is refuted in **both** directions:
> not 0.01 %, and not 97 %. Pre-registered before measuring, in its own commit,
> at 34,000 ± [30,000, 40,000] — **0.24 % out**.
>
> It explains the flat TU-match count with nothing else: at 19 % per emitted
> function, a TU with the median 141 emitted has no chance of being byte-exact.
> And **the widening order over emitted code is not the order over bodies** —
> `expr-op-0x27` falls 23.4 % → 18.0 %, while `body-cflow-label` and
> `expr-intrinsic-this-adjust` rise to #2 and #3. `c2rs gap` now prints both.
> Per-TU table and the binding's invariants: `docs/GAPS.md` §8.7. Given what the in-class classes are — 102,056 empty bodies, 35,964
generated destructor delegations, 177,262 indirect-load leaves, largely
header-inline shapes — the true emitted share is plausibly far below 28 %.

**The corroborating fact is stark: TU match has been flat at 6/878 across a
census run from 4.45 % to 28.31 %.** A 6.4× numerator gain bought zero TU
matches. 24 TUs are within 1 function of matching, 46 within 100, and **832 are
more than 100 blocked functions away.**

**And a caveat that had never been written down:** for a never-emitted body,
"in class" is a *parser-only* claim. The differential grades whole objs, which do
not contain those bodies, so no byte compare has ever graded them and none ever
can. The recorded precedent that this direction can be green-and-wrong is the
`.sy` positional relaxation — census +2,981, mismatch 0, wrong on 62 % of
bindings.

### 8.2 What to steer by instead, in rank order

1. **TU match count** — the payoff metric, currently **6/878** — with the
   TU-distance distribution (≤1: 24, ≤100: 46) as its leading indicator. The
   honest terminal target is **871/878 byte-exact at the workload's own flags**
   (`/O1 /Oi /EHsc`); the 7 capture-fails are workload-manifest issues, not port
   gaps.
2. **Emitted-function census** — in-class ∩ emitted / 178,968. **Built
   2026-08-01 and printing on every scan: 19.04 %**, with a named residue.
   Narrowing the interval further is a `.gl` record-shape job (152,941
   `nameless` records), not a census one.
3. **The per-function census, explicitly demoted to *driver*.** It ranks rungs
   superbly and its gate discipline is the best thing in this project. It is not
   the target, and "census 100 %" is retired as a stated goal.

The route question this opens — *lower everything* versus *model which bodies c2
emits* — is genuinely open and must not be pre-decided. A wrong emit-set model
converts refusals into missing or extra COMDATs, i.e. **mismatches**, and its
inliner half is the least-derivable model in the program (`LABEL_COUNTER.md`
§6.15.3: the `/O1` inline-decline schedule is measured exactly and *generated by
no formula*). What is **not** defensible is continuing to publish a numerator
whose overlap with emitted code is unknown. The decisive experiment is cheap:
build the binding, read out the emitted census once, re-derive the plan's cost
from what c2 actually emits.

### 8.3 Phase order the evidence supports

**Phase 0 — instruments, before any further wave is ranked.** The emitted-census
binding; production-tag completion (`prod-entered-untagged` = 731,921, target 0 —
until then every `prod-*` row is a lower bound); dispatch routing for the 8,807
unroutable member-call forms; the `-whole{k}` over-count repair; pricing the
63,858 `eh-unknown`; and converting `cross_sweep` to `scripts/lanes.txt` — the
last surviving un-enumerated lane, which has **never compiled `/EH`** on a
workload that is 100 % `/EHsc`.

**Phase 1 — leaf/expression rungs behind existing recognizers** (parallel).
Single-symbol data address (~15.6k, ceiling-is-the-estimate); the comparison
spine — small by census but **8 of the 17 TUs within 3 functions of matching
block on a `cmp` row**, the highest match-bucket leverage anywhere on the board;
the cheap intrinsic slices (`base-member-addr` 41,678 at `maxState=0` is the
largest cheap row), which retire *for free* inside ordinary widenings and must
not be scheduled as EH work. Note `expr-op-0x27` (412,797, 23.4 %) is a
**reservoir, not a rung** — measured three times at 0.14–2.5 % completion.

**Phase 2 — Class B/C frames.** The serial spine (~26k direct, plus it gates
every framed member-call receiver). Serial because the *evidence* is serial.

**Phase 3 — the FP arithmetic program** (parallel). Note the 2+-pooled-constant
scheduler is explicitly **not implementable from the existing captures**.

**Phase 4 — member calls with frames, and multi-symbol data addresses** (20,366).
The two governing rules there are **fitted hypotheses with no mechanism**; the
cheapest next step is a *designed, enumerated* capture grid, not implementation.

**Phase 5 — EH**, the largest single phase (237,180 plus an unpriced share of
`eh-unknown`). Order per `EH_RECORDS.md` §8.7: groundwork (census 0 by
construction) → the no-try unwind rung → the state model → try/catch.

**Phase 6 — control flow, demand-gated.** The old "restructure early" plank is
**refuted**: the lowering counterfactual has said **718 functions, five scans
running**, because branchy bodies are also framed, call-bearing and EH-bearing.
Re-run that counterfactual after each Phase-2/5 rung — it is one warm scan.

**Phase 7 — TU assembly and the emit-set model.** Where the route question gets
decided, gated on Phase 0's read-out.

**Phase 8 — VMX128, long tail, mode generality.** VMX128 is sized and favorable
(a genuinely independent third register file) but its **demand is invisible** —
no census key fires — so it needs an axis before it can be ranked. `/Od` (1/201
fixtures) is not on the workload's critical path at all.

### 8.4 Where 28.31 % is softest — what a hostile reviewer attacks first

1. **The headline counts functions the compiler never compiles** (§8.1). Missing
   check: the census↔obj-symbol binding.
2. **In-class for never-emitted bodies is graded by nothing**, and no obj-level
   check exists even in principle — itself an argument for the emit-set route,
   where the claim being made ("skipped") *is* checkable against the symbol table.
3. **The `26`-form `.gl` binding is emitter-ungraded** — 35,946 in-class
   generated destructors rest on four container measurements and zero
   fixture-graded objs.
4. **Instruments never shown to fail**: the `maxState` axis is graded on 46
   hand-written functions once, at `/O1 /EHsc` only; the frame axis has a
   measured 1.3 % error and its control group is the in-class population; and
   `cross_sweep`'s green 111,824 gradings are silent about `/EH`.
5. **TU-level gates sit outside the census/gate cross-check** (§6f #3). Verified
   at cost 0 whole TUs today — the 15 gap-0 TUs reconcile exactly as 6 match + 7
   capture-fail + 2 deliberately-refused empty TUs — but the check is
   per-function only and will not stay free as the class widens.
6. **The general allocator/scheduler is the largest unbounded unknown and has no
   instrument.** Every accepted class caps *below* c2's allocator; nothing
   measures how much of the blocked population needs scheduling the port has
   never attempted. A crude two-bucket axis would convert a rumour into a number.

### 8.5 Effort shape

Serial by necessity: the frame spine (each rung's captures exist only on the
previous rung's byte-exact model); `coff.rs`, always single-occupancy; the merge
funnel with a full re-gate per merge (a merged tree is a new corpus); and
instrument-before-wave, because a wave ranked off a broken signal reproduces the
break.

Parallel, 3–5 lanes on non-overlapping seams: FP/leaf; decode/instrument;
**ground-truth capture (docs and probes only — zero collision, and the cheapest
high-value lane on the record)**; the one-away/match lane, whose success is the
only thing that currently moves the payoff metric; and the front-end track.

The throughput bound is no longer file contention. It is the serial spine, the
serial merge funnel, and the estimate discipline — which is an asset worth
protecting: eight consecutive estimate misses were cured by the ceiling rule, and
every discount ever applied has been wrong five times in six.

### 8.6 The two orders side by side — and control flow is not where the body count put it

Both columns now print on every scan. They do not agree, and the disagreement is
structural rather than noise: header-inline trivia has no branches, and real
emitted functions do.

| key | bodies (1,765,320) | rank | **emitted (127,179)** | **rank** |
|---|---:|---:|---:|---:|
| `expr-op-0x27` | 412,797 · 23.4 % | 1 | 22,831 · 18.0 % | 1 |
| **`body-cflow-label`** | 48,102 · 2.7 % | **6** | **14,947 · 11.8 %** | **2** |
| `expr-intrinsic-this-adjust` | 135,941 · 7.7 % | 2 | 8,790 · 6.9 % | 3 |
| `expr-intrinsic-base-member-addr` | 113,981 · 6.5 % | 3 | 6,472 · 5.1 % | 4 |
| `expr-call-in-expr-recv-load-then-bit-and-and-branch-more` | 102,374 · 5.8 % | 4 | — | **out of the top 20** |
| `expr-load-type-8645` | 55,679 · 3.2 % | 5 | 2,267 · 1.8 % | 8 |
| `expr-brfalse` | 26,507 · 1.5 % | 12 | 3,102 · 2.4 % | 7 |
| `return-scope-close-cflow-label` | — | — | 1,817 · 1.4 % | 15 |

**`body-cflow-label` is 4.4× enriched in emitted code** — rank 6 becomes rank 2.

> **CORRECTION, same day (WCFLOW).** The sentence that stood here — "taken with
> `expr-brfalse` and `return-scope-close-cflow-label`, control-flow keys are
> 19,866 of 127,179 = 15.6 % of blocked emitted against 4.2 % of bodies" —
> **counted 3 of the ~69 control-flow census keys.** The full family is
> **22,471 emitted (17.68 % of blocked emitted) against 238,001 bodies (13.53 %
> of blocked bodies)**: family-level enrichment **1.31×, not 3.7×**. The 4.4× on
> `body-cflow-label` is real, but it is a fact about **where a first blocker
> lands**, not about control flow's share. Picking the three keys that moved and
> summing them is selection on the outcome — the same shape as the four ranking
> artifacts §6n records.

**And the counterfactual has now been run on the emitted column: it is 10.**
Five `cflow-if-1`, five `cflow-switch`, against 718 on bodies. **The picture
does not invert — it gets worse.** A random IL body is emitted at 7.27 %; a
control-flow-only-blocked body is emitted at 10/718 = **1.39 %**, i.e. **5.2×
*less* likely than average**, in the opposite direction from the key enrichment.

The mechanism is measurable and it is not subtle. The counterfactual's residue
predicate fires on **14.18 %** of *straight* emitted bodies and on **0.031 %** of
*branchy* ones — a **457× gap** that swamps the 4.4×. Branchy bodies are not
merely also-framed and also-EH-bearing; their **expressions are harder**, so a
body blocked only on control flow is a rare accident rather than a population.
§8.3's demand-gated placement for Phase 6 is confirmed on the very population it
was said to be unknown for, and `ARCHITECTURE_SEAMS.md` §7 — the oldest recorded
reading, "the codegen half forces the block IR, lowering waits" — was right all
along.

Symmetrically, the row that was #4 by bodies — 102,374 functions, already known
to be 100 % `cflow-if-1` ∧ `calls-2plus` — **does not appear in the emitted top
20 at all**. Ranking it was always going to be wasted work; now that is visible
rather than inferable.

### 8.7 Control flow is a PHASE worth 10, and the emitted board's ceiling is enumerable

Three positive statements, each independently checkable:

1. **34,169 of 34,169** in-class bound emitted rows read `cflow-straight`
   (21,205 + 12,964, summing exactly). **Zero branching bodies accepted**, on
   emitted code, at this HEAD.
2. `codegen/encode.rs` has **46 encoders and exactly one is a branch** (`blr`),
   plus one raw unconditional `b` word in the tail-call path. No `bc`, no label,
   no fixup, no block IR. **There is no emitter to widen** — this is new
   machinery, not a private limit, so §6n category (1) does not apply.
3. The control-flow stock decomposes as **1,591 (7.1 %) standalone**, 9,034
   (40.2 %) gated on the frame phase, **11,846 (52.7 %) gated on the EH phase** —
   where the port's measured acceptance is **0 of 30,254**.

Bracket for "lower control flow, change nothing else, on emitted code":

| reading | value |
|---|---:|
| direct counterfactual | **10** |
| cell-weighted model, cells the port already serves | ≤ 480 |
| raw first-blocker stock in those cells | 1,591 |
| raw stock, all cells (needs Phase 2 **and** Phase 5 first) | 22,471 |

The cell-weighted 480 assumes a branchy body's expressions are as acceptable as
a straight body's in the same cell — **measurably false by 457×** — so it is an
optimistic ceiling, not an estimate.

**Three archaeology corrections, each worth more than a new number.**

* **`expr-op-0x27`'s "0.14–2.5 % completion" band is a mis-attribution** (quoted
  at `ROADMAP.md` §6-era and `GAPS.md` §…): 1.4 % is the *pointer-type* row and
  2.47 % is `expr-convert`. The three real `0x27` measurements are 0.14 %, a
  production rung that took 22,095, and **1.48 % (6,816/461,786)**. At 1.48 % the
  row is worth **~337 emitted**.
* **`base-member-addr`'s "41,678, the largest cheap row"** is a `maxState = 0`
  *slice*, not a completion count. Its completion counterfactual is **740
  bodies**, measured and **already realized as −740**. No document that ranks the
  row mentions the 740.
* **`expr-intrinsic-this-adjust` has never had a completion counterfactual.** The
  10,469 `-whole` figure belongs to a *different key* and is itself labelled an
  unverified claim. At 8,790 emitted it is the **largest never-measured row on
  the board**, and one scratch build plus one warm scan settles it.

**Rows that die on the emitted column**, with the body count they were ranked on:

| row | bodies | emitted | clean |
|---|---:|---:|---:|
| `…recv-load-then-bit-and-and-branch-more` | 102,374 | **9** | 0 |
| `…recv-object-then-branch-brtrue` | 23,633 | 431 | **0** |
| `expr-bit-and` (already declined at 0) | 32,382 | 1,824 | **1** (99 % EH) |
| `fn-tail-0x26` (already refuted) | 4,663 | **0** | 0 |

**The concentration is the good news.** `clean` = `cflow-straight` ∧ `eh-none` ∧
`calls<2`, a hard ceiling on what widening a key alone is worth. The top 28 rows
by clean ceiling total **39,946 of 44,932** clean emitted — the emitted board's
ceiling is concentrated and enumerable, unlike the 726-key body tail. But **only
35.35 % of blocked emitted is clean at all**: the other **82,161 need the frame
phase, the EH phase, or both**, which is the honest shape of the remaining work.

Best-founded next rungs on this column: `expr-intrinsic-this-adjust` (8,790,
**measure it, do not build it**); `expr-call-in-expr-recv-object-then-type-ptr-whole`
— 1,380 emitted, **clean 1,380 = 100 %**, zero `calls-2plus`, zero EH, **7.93×
enriched** and the only large row that is entirely clean; `expr-intrinsic-memset`
(3,752 / 2,042); and the single-symbol data address at **1,548 emitted against
15,583 bodies — a 10× discount** on the figure it was scheduled with.

## 9. The listing seam — c2's own account of its output (WTRACE, 2026-08-01)

A white-box recon pass (Ghidra / rizin / live tracing) returned its highest-value
finding from the **black box**: `c2.dll` in this XDK is **not a stripped build**,
and `cl /FAsc … /c` appends **`-FAasc -Fa <file>`** to c2's own argv, driving it
to write a complete **assembly listing** (`.cod`). Confirmed on the c2 command
line printed by `/Bd`:

```
c2.dll -il …_CL_f6979fdd -typedil -Fowt_add3.obj … -Bd -Og -Ob2 -FAasc -Fa wt_add3.cod
```

This is an **output of the black box, not its disassembly** — §9.5.

### 9.1 The two validity results (measured here, not by the lane)

Neither was in the lane's report; both decide whether the instrument is usable.

1. **The listing does not perturb the obj.** Same TU compiled with and without
   `/FAsc` at `/O1 /Oi /EHsc /GS-`: **1,956 bytes each, byte-identical** with
   `TimeDateStamp` zeroed. The instrument is non-perturbing, so a `.cod` may be
   captured beside the very obj the differential grades.
2. **"Byte-faithful" is overstated, and the overstatement is exactly one class.**
   Row-by-row against the obj `.text`: **37 of 44 identical, 7 differ, and all 7
   are relocated branches** — the listing prints the canonical unrelocated word
   (`bl` → `48000001`, `b` → `48000000`) and names the target symbol, where the
   obj carries a real displacement (`4bfffff5`, …). For our purposes the name is
   *more* useful than the displacement, but **a lane that treats `.cod` as raw-byte
   ground truth will be wrong at every call site.**

   The lane's positive control was `add3` — a tail-call chain of `7d632214 /
   7c6b2a14 / 4e800020` with **no relocated branch in it**. This is the
   absence-read-as-success shape again in its subtlest form: the control was run
   where the discrepancy *cannot appear*. Twelfth instance.

### 9.2 What the listing carries

Verified on a framed + EH + loop + division body at the workload's `/O1 /Oi /EHsc`:

* exact bytes, offsets, and PPC mnemonics, with **source-line correlation**;
* **section/COMDAT emission order** — `.XBLD$W`, `.pdata`, `.pdata`, `.rdata`,
  `.code`, each with its `SEGMENT` directive (board #120, #62);
* every relocation **target by name** (`bl ?make@@YAHH@Z`, `DCD |__ehfuncinfo$…|`);
* the per-function optimization word, spelled — `; Function compile flags: /Ogsu`
  (the `4F 1F 80 <LE32>` word of board #19/#52);
* frame slot assignments (`s$ = 80 ; size = 4`);
* EH records with field boundaries — `__unwindtable$`, `__ehfuncinfo$ DD
  019930522H`, the `$T` state table, the `__unwind$` funclet (board #121, Phase 5);
* **c2's internal label counter in allocation order.**

### 9.3 The label counter is allocated in a phase order that is NOT text order

Read directly off the verified listing, ascending:

| label | where it lands | text offset |
|---|---|---:|
| `__unwind$2568` | funclet body | 0x88 |
| `$M2577` / `$M2578` / `$M2579` | EH state transitions | 0x38 / 0x78 / 0x84 |
| `$T2580` | `.rdata` state table | — |
| `$M2582` / `$M2583` | prolog end / body end | 0x18 / 0x88 |
| `$T2584` | `.pdata` #1 | — |
| `$M2585` / `$M2586` | funclet markers | 0x98 / end |
| `$T2587` | `.pdata` #2 | — |

The funclet is allocated **first** and emitted **last**; the `$M` block is split
around the `$T` tables. This is precisely the semantics whose mis-modelling
produced *both* historical six-wrong-byte defects in `coff.rs` (#5), and it has
never been observable before. It is now a transcription, not a fit.

### 9.4 Standing instrument, and what stays one-shot

**Adopt (small, durable):** a `c2rs listing <cpp> [flags]` seam in `c2-reference`
— the existing replay with `-FAasc -Fa <tmp>` appended, returning `(obj, cod)`.
The oracle is **unchanged**: the obj byte-compare remains sole judge and the
listing is a **decode aid, never a gate**. A `--qxstalls` variant appends
`/QXSTALLS`, which annotates the listing with `Possible load-hit-store penalty`,
`Dependency stall`, `PX Dispatch Groups`, `Estimated block IPC` — the first
scheduling-demand instrument the project has ever had (#119, §8.4 item 6).

**Keep one-shot:** true disassembly. Only one item needed it (the inliner's
"too big" threshold, referenced at `0x10ba23b3`) and that has a black-box route
too — a size-graded family compiled against the emitted set. Recommendation:
**take on no white-box debt.** Note for any future static lane: Ghidra headless
**refuses any path containing a dot**, so it cannot run from `.claude/worktrees/…`.

### 9.5 What it does NOT give — the honest boundary

**There is no switch that dumps c2's parsed IL.** The hoped-for jackpot is
refuted with a positive control: `-d2il<base>` is an *input* override (it tries to
**read** `<base>gl` and dies `C1083 … 'ZZZgl': No such file` with nothing created),
the same role as `-il`. ~25 candidate flags returned `C1007 unrecognized flag …
in 'p2'`. **The IL decoder stays hand-fit**; the listing accelerates the *codegen*
decoders only.

The **emit-set predicate (§8.1) is named but not formula-ised.** c2's strings
enumerate its disjuncts — `globally unreferenced`, `has linear flow`, `is a
redirector function`, `won't be inlined (too big)`, `inlining prohibited`,
`InlBadCandidate` — and the listing shows the decision per TU, but the `/O1`
inline-decline schedule that `LABEL_COUNTER.md §6.15.3` calls "generated by no
formula" is unmoved. One genuine null result bounds it: an `strace` diff of an
in-class against an out-of-class TU shows the opened-file sets differing **only**
in bundle hash and filenames — same five `_CL_*` bundles, **identical mmap count
(124)**. c2 consults **no external table** for the shapes the port refuses; the
predicate is entirely in code plus bundle.

Also inert, recorded so nobody re-schedules it: the MS build provenance
(`…\vctools\compiler\be\p2\c2\obj\i386\c2.pdb`, MS calls this component **"p2"**),
and the incidental fact that c2 loads `msdisXXX.dll` to disassemble and
`msobjXX.dll` to serialize COFF — which is why the container is so standard.

### 9.6 Price, in the honest units

**The listing moves neither the census nor TU match by itself.** It is an
instrument that lowers the RE cost of the remaining phases, and it is worth
ranking only because of *which* phases: EH (Phase 5, ~237k bodies; 11,846 emitted
control-flow bodies gated behind it), frames (Phase 2), the label counter, and
section order. §8.7 measured that **82,161 of blocked emitted need the frame
phase, the EH phase, or both** — and those two phases are exactly the ones whose
structure the listing prints symbolically. It is the "cheapest high-value lane"
§8.5 already called for, now with a concrete tool.

### 9.7 New board items

* **#132** — the `c2rs listing` seam (Phase 0 instrument).
* **#133** — transcribe the EH record layout from `.cod` into `EH_RECORDS.md`
  (Phase-5 groundwork; census 0 by construction).
* **#134** — `/QXSTALLS` scheduling-demand axis; answers #119.
* **#135** — model the label counter from `.cod` allocation order (§9.3); retires
  the bug class behind #5.
* **#136** — reconcile the per-TU `.cod` `PUBLIC`/`PROC` set against the obj
  COMDAT scan as a second, **name-carrying** source for the emitted census (§8.2).

### 9.8 README wording

The listing, the diagnostic strings, `/QXSTALLS`, the emitted set, and the trace
diff are all **observable outputs of the black box** — the category the README
already blesses — so the clean-room claim survives and can be *sharpened*:

> The original binary is treated as a black box and its observable output — the
> obj, and c2's optional `/FAsc` assembly listing and diagnostic output — is the
> spec. No decompiled c2 source informs the port.

If a disassembly-derived constant is ever adopted, that blanket claim must weaken
to per-finding disclosure, naming the site in the relevant `docs/` file. On the
recommendation of §9.4 we take on none, so the sharpened sentence stands.

### 9.9 WLISTING — pre-registration (written before any of the three was run)

Lane `w-listing`, 2026-08-01, board items #132 / #134 / #136. Registered here
**before** the seam existed and before either scan ran, so the scores below can
be graded rather than retrofitted.

**#132 — the seam.**

* **E1** c2 accepts `-FAasc -Fa <Z:path>` appended to a *standalone* replay argv
  (not just to a `cl` driver line) and writes the `.cod`. *Refuted by* `C1007
  unrecognized flag … in 'p2'` or no file.
* **E2** The listing does not perturb the obj: replay-with-listing and
  replay-plain are byte-identical with `TimeDateStamp` zeroed, **at the same
  `/Fo` path**. *Refuted by* any differing byte.
* **E3** `.cod` instruction rows equal the obj `.text` word at the same
  COMDAT-relative offset EXCEPT at relocated `b`/`bl`, where the `.cod` prints
  `48000000` / `48000001`. *Refuted by* any differing row whose mnemonic is not
  `b`/`bl`.
* **E4** (the residual §9.1 could not see) A data-address relocation
  (`lwz r3,?g@@3HA(r11)`) does **not** differ, because c2 leaves the
  displacement 0 in both artifacts. *Refuted by* a differing non-branch row.
* **E5** The `.cod` `PROC` set equals the obj `.text` COMDAT set exactly on the
  fixture corpus. *Refuted by* a non-empty residue either way.

**#134 — `/QXSTALLS` demand.** Predicted **85 %** (interval [75 %, 95 %]) of
*blocked emitted* functions carry a stall annotation. *Refuted by* < 50 %.

* **The control that decides whether the number means anything:** the same
  fraction over **in-class emitted** functions — the ones the port already
  reproduces byte-exact with no scheduler at all. Predicted **≤ 35 %**, i.e. the
  annotation discriminates. **If in-class ≈ blocked, the instrument is
  uninformative and #134 must report that, not the headline number.** This is
  the positive question §9.1's twelfth instance demands: a scan that reports only
  the blocked fraction would go green on a signal that is present everywhere.
* Second honest bound registered up front: an annotation says the *emitted
  schedule stalls*, not that c2 *reordered* anything. The number is an **upper**
  bound on scheduling demand.

**#136 — the second, name-carrying census source.**

* **Injectivity** — no mangled name appears twice in one TU's `PROC` set.
  Predicted residue **0**.
* **Totality** — `PROC` set == obj `.text` COMDAT set per TU. Predicted **small
  but non-zero on the real workload** (0 on fixtures), concentrated in
  compiler-generated names.
* **Agreement on the 6 byte-exact TUs** — predicted **exact**.
* **The error term on 19.09 %** predicted **< 1 percentage point**.

### 9.9.1 #132 — the seam, and the design that was measured to death first

`Toolchain::capture_listing[_with]` is the existing capture with `/FAsc` (and
optionally `/QXSTALLS`) appended, returning `(CapturedReference, cod)`;
`c2-reference::cod` reads the listing; `c2rs listing <cpp>` exposes it. The
oracle is unchanged — the obj byte-compare is still the sole judge and the
listing is a decode aid, never a gate.

**The cheaper design does not work, and it was measured rather than assumed.**
Appending `-FAasc -Fa` to a *standalone* c2 replay would have bought a listing
per **cache hit** instead of per capture, which over 878 TUs is the whole cost.
It fails under wibo: `-FAasc` is the only thing that makes c2 load
`msdisXXX.dll`, which under `cl.exe` resolves from the driver's own directory
and under `c2host.exe` does not. Resolving it removes the `SIGABRT` on the
stubbed `?PdisNew@DIS@@SGPAV1@W4DIST@1@@Z`; c2 then `SIGSEGV`s inside the
disassembler after reading the source for line correlation, and it still does so
with `msvcp100.dll`, `msobjXX.dll`, `pgodb100.dll` and `tlbref.dll` **all**
resolvable and **no missing import left in the wibo trace**. The fault is inside
`msdisXXX.dll` under wibo and is not ours. Recorded so nobody re-attempts it;
`msobjXX.dll` in particular was deliberately *not* left beside the host, because
making it resolvable changes the oracle's environment for every replay to buy
nothing.

**The two standing tests** (`crates/c2-reference/tests/listing.rs`) run on
`il_call_return.cpp`, which contains **10 relocated branches (3 `b`, 7 `bl`)**,
and that quantity is pinned by its own assertion phrased over the *mnemonic* —
a fixture property, not a classifier property, so a broken classifier goes red
instead of making the later assertions unreachable. Seven assertions, each with
a distinct message; four were verified red by mutation:

| mutation | assertion that fired |
|---|---|
| byte compare offset by one | (d) 52 non-branch rows differ |
| `is_relocated_branch` mnemonic broken | (e) classifier missed 10 branches |
| `PROC NEAR` matched with a space only | (a) 0 of 10 relocated branches |
| listing word XOR 0x10 | (c) neither 48000000 nor 48000001 |

§9.1's byte claim is **confirmed and its class is now bounded from the other
side**, on 204 fixtures rather than one: **9,430 rows identical, 1,024
differing, every one a `b` or a `bl`.** The half §9.1 could not see is that
**non-branch relocations do not differ** — a data-address row
(`lwz r31,?g_i@@3HA(r11)` = `83eb0000`) carries a relocation and matches
exactly, because c2 leaves the displacement 0 in both artifacts. So the class is
`{b, bl}`, not "anything relocated", and a lane may trust every other row.

One instrument defect worth recording because it is the §9.1 shape again: the
first `PROC` pattern required a **space**, and c2 tab-aligns some definitions.
It silently dropped 5 of 7 functions on `il_call_perm.cpp` and then reported
"all differing rows are branches" over the rows it had not read. Caught by
comparing the `PROC` count against the obj COMDAT count, which is now assertion
(g) and board #136's invariant 2.

### 9.9.2 #134 — `/QXSTALLS` is NOT a scheduling-demand axis, and #119 still has no instrument

Full workload, 871 TUs captured of 878 (7 capture-fails, the known
workload-manifest issues), in **emitted-function units**:

```
BLOCKED  emitted: 115,877/127,093 carry a stall annotation   91.17 %
IN-CLASS emitted:   5,100/ 34,169 carry a stall annotation   14.93 %   <- control
discrimination: +76.25 pp
```

Taken at face value this is the headline #134 asked for. **It does not survive
its own control.** A blocked emitted function is far longer than an in-class one
— the port's class is leaves and tail calls — and a longer body has more chances
to stall. Stratified by exact instruction count:

| rows | blocked stalled/total | in-class stalled/total |
|---:|---|---|
| 1 | 0/5,700 (0.00 %) | 0/12,475 (0.00 %) |
| 2 | 865/5,613 (15.41 %) | 0/16,562 (0.00 %) |
| 3 | 10,186/10,276 (99.12 %) | 2,012/2,012 (100.00 %) |
| 4 | 7,172/7,234 (99.14 %) | 895/895 (100.00 %) |
| 5 | 6,247/6,411 (97.44 %) | 237/237 (100.00 %) |
| 6 | 4,764/4,766 (99.96 %) | 548/548 (100.00 %) |
| 7 | 3,014/3,171 (95.05 %) | 297/329 (90.27 %) |
| 8 | 2,869/2,869 (100.00 %) | 77/77 (100.00 %) |
| 9-16 | 16,480/16,861 (97.74 %) | 821/821 (100.00 %) |
| 17-32 | 32,872/32,873 (100.00 %) | 212/212 (100.00 %) |
| 33+ | 29,408/29,408 (100.00 %) | 1/1 (100.00 %) |

**At every length of 3 instructions or more the two populations are
indistinguishable — 95-100 % on both sides.** The annotation is very nearly a
function of body length: any body of 3 or more instructions carries one, whether
or not the port already reproduces that body **byte-exact with no scheduler at
all**. The +76.25 pp is a length effect: **84.98 % of in-class emitted functions
(29,037/34,169) are 1-2 instructions**, against **8.90 % of blocked
(11,313/127,093)**.

The one genuine within-length signal is at 2 instructions — blocked 15.41 %
against in-class 0.00 %, 865 functions. It is real, it is small, and it is not a
basis for sizing a scheduler.

**The load-hit-store sub-signal cannot be evaluated at all.** It fires on 616
blocked emitted functions (0.48 %) and 0 in-class — but **582 of the 616 are in
`rows-33+`, where the in-class population is exactly one function.** There is no
size-matched comparison to make. What can be said is a ceiling: at most 616 of
127,093 blocked emitted functions carry the one annotation that names a
*scheduling* remedy rather than a latency.

**So the answer to #119 is a refusal, not a number.** `/QXSTALLS` measures the
emitted schedule's stalls; it does not distinguish code that needs scheduling
from code the port already emits correctly without any. The general
allocator/scheduler remains the largest unbounded unknown **and remains without
an instrument**. Do not schedule scheduler work off the 91.17 %.

The negative control is intact and was run: the same scan **without**
`--qxstalls` reads 0/127,093 and 0/34,169. The reader is also unit-tested to
refuse a bare `; [I 11A]` issue-cycle marker, which every annotated function
carries and which alone would have made the fraction 100 % by construction.

### 9.9.3 #136 — the second source is *exactly* the first source, and that is the result

Per TU, `.cod` `PROC` set against obj `.text` COMDAT set, over the same 871 TUs:

```
.cod PROC set                       178,968
obj .text COMDAT set                178,968
invariant 1  injectivity            0 duplicate PROC names          PASS
invariant 2  totality               0 cod-only, 0 obj-only          PASS
invariant 3  the 6 byte-exact TUs   6/6 reconcile exactly           PASS
ERROR TERM on the emitted census    0 of 178,968  =  0.0000 pp
```

178,968 is §8.1's denominator, now **independently confirmed by a second
artifact c2 writes, in mangled names**. And the census read out through this
scan's own fresh `/FAsc /QXSTALLS` captures is **34,169/178,968 = 19.09 %**,
byte-for-byte the figure a `c2rs gap` run over the *cached, un-listed* captures
prints — two capture paths, two code paths, the same number, which is also a
population-scale confirmation that `/FAsc` and `/QXSTALLS` do not perturb.

**Scope this honestly.** #136's 0.0000 pp is an error term on the emitted
census's **denominator**, not on its numerator. The `.cod` gives names, and the
obj already gave names; the `PROC` set therefore cannot say anything about the
`.gl` record → census row binding, whose residue — **17,706 emitted symbols
(9.89 % of the denominator) that no census row claimed** — is untouched by this
instrument and still needs a different one. The one-line answer to board #118:
*the denominator is now watched and is exact; the numerator's binding residue is
not, and #136 was never able to reach it.*

**The instrument can go red, and that was checked rather than assumed** — a
0-residue result over 178,968 items is exactly the shape this project reads as
success when it is really absence. Two falsification runs:

* break the `PROC` parser outright → `obj-only 1,125 of 1,125`, error term
  **100.0000 pp**;
* corrupt **one** `PROC` name per TU → `cod-only 5, obj-only 5` out of 1,125,
  error term 0.8889 pp — while the two **counts stayed equal at 1,125 = 1,125**.
  That last part is the point: a reconciliation that compared totals would have
  passed this. The comparison is over sets.

### 9.9.4 Pre-registration scores

Registered in §9.9 before any of it ran, in its own commit.

| | registered | measured | |
|---|---|---|---|
| #132 E1 | standalone replay takes `-FAasc` | **crashes in `msdisXXX.dll`** | **MISS** |
| #132 E2 | listing does not perturb the obj | identical, `/FAsc` and `/QXSTALLS` | HIT |
| #132 E3 | differing class = `b`/`bl` | 1,024 of 1,024 are `b`/`bl` | HIT |
| #132 E4 | data-address rows do not differ | 0 of them differ | HIT |
| #132 E5 | fixture `PROC` set == obj COMDAT set | 204/204 equal | HIT |
| #134 | 85 % blocked, interval [75, 95] | **91.17 %** | HIT (6.2 pp out) |
| #134 control | in-class at most 35 % | **14.93 %** | HIT |
| #136 injectivity | residue 0 | 0 | HIT |
| #136 totality | small but non-zero on the real workload | **exactly 0** | **MISS** |
| #136 byte-exact | exact agreement | 6/6 | HIT |
| #136 error term | under 1 pp | 0.0000 pp | HIT (vacuously — totality was 0) |

**9 of 11.** Both misses are worth more than the hits: E1 killed the cheap design
and is now documented so nobody re-attempts it, and the totality miss is the one
that makes the #136 result *stronger* than registered.

And the score flatters the #134 line. The registered control asked the right
question — "if in-class is close to blocked, the instrument is uninformative and
#134 must report that" — and the **raw** comparison passed it (14.93 % against
91.17 %). The size stratification was **not** pre-registered; it was added
because a blocked-vs-in-class comparison is a comparison of long bodies against
short ones, and it overturned the reading. Registering a control is not the same
as registering the *right* control.

### 9.9.5 Gate evidence

At `dc1bd9b`, worktree configured against the shared toolchain
(`scripts/configure_existing_worktree.sh`):

* `cargo test --workspace` — **571 passed, 0 failed, 1 ignored** across 24 test
  binaries (the ignored one is pre-existing). Includes 6 new `cod` unit tests,
  3 new `listing.rs` integration tests, 3 new `c2-harness::listing` unit tests.
* `c2rs selftest` — **204/204 PASS**, 0 fail, 0 skip.
* `scripts/gate.sh --jobs 6` — **GATE: PASS, 12/12 lanes ran**, 2,448
  fixture-verdicts, **0 mismatch** in every lane. `scripts/gate.sh --selftest` —
  PASS, 15 cases.
* `c2rs gap` full workload — 6 match, 0 mismatch, 865 vocab-gap, 7 capture-fail;
  emitted census 34,169/178,968 (19.09 %), unchanged by this lane, which wrote
  no port code and did not touch `crates/c2-core`.

### 9.10 WR1's ordering rules are pinned only where the toolchain is — board #137

Not a correctness alarm. The rung is verified by the strongest judge the project
has: `cross_sweep` **512,628 gradings**, gate **2,472 fixture verdicts**,
`expr_sweep` **14,399**, all **0 mismatch**. This is a **durability** gap, and it
is recorded because the file it sits in has produced the same defect class twice.

WR1 landed ~1,500 lines including **150 in `crates/c2-core/src/coff.rs`**, and
the `#[test]` block count **did not move**: 557 at its base `6b07500`, 557 at
`wt-w-r1`. Its own gate evidence states this plainly — it reported
`cargo test --workspace 557 passed`, which *is* the base count. The number was
reported honestly and read by nobody, including the coordinator on first pass.
**A test total that does not move across a 1,500-line rung is a finding; compare
it to the base rather than to zero failures.**

The rule at risk is the one WR1 itself found as an ALARM: **the address `addi` is
emitted LAST**, not at its slot's turn in the descending walk. Descending and
address-last **agree on every body with the symbol at slot 0** and disagree only
when a literal sits at a *lower* slot. That discriminating arrangement now exists
in `fixtures/cpp/wr1_sym_addr.cpp`. The same holds for the other rule WR1 got
wrong on its first differential: the REFHI/REFLO quad's halves are **not
adjacent**, because the `lis` is hoisted, so REFLO is not at `hi_off+4`.

> **AMENDED 2026-08-01 by WLABEL — the paragraph above originally said the gap
> was the *portable* lane, implying the toolchain lane caught it. It does not.**
> Measured by mutating the **base** tree, before any new test existed:
>
> | mutation | portable `cargo test` | **toolchain** `cargo test` | `gate.sh` |
> |---|---|---|---|
> | `addi` at its slot's turn | 571/0 | **571/0** | red |
> | REFLO at `hi_off+4` | 571/0 | **571/0** | red |
> | `lo_off = base+4` | 571/0 | **571/0** | red |
>
> `cargo test --workspace` **never grades `wr1_sym_addr.cpp` in either lane** —
> `differential.rs` names three fixtures and that is not one of them; the string
> appears in `crates/` only inside comments. Only `scripts/gate.sh` goes red.
> This also refutes WLABEL's own registered control, and it is the more useful
> result.

**Standing rule (as amended):** a rung that touches `coff.rs` must add a
**portable** assertion for each ordering rule it establishes, **and the evidence
that an ordering bug is caught at all is `scripts/gate.sh`, not
`cargo test`.** Adding a fixture does not put it in the test lane. The
differential catches an ordering bug only where some fixture happens to arrange
the discriminating case — precisely the coverage argument WR1's own ALARM
refuted, since its hand-written fixture had three literal cases and **all three
put the symbol at slot 0**. WLABEL's pins therefore keep a **slot-0 control that
stays green** under the address-last mutation: a test that goes red everywhere
identifies nothing.

> **The `#[test]`-count metric introduced above is itself inflatable.**
> `git grep -c '#\[test\]'` cannot distinguish an attribute from a comment, so
> prose *about* the count changes the count — WLABEL's first tally read 580
> against a runner total of 579 for exactly that reason. Reconcile grep against
> the runner's own total before quoting either.

### 9.11 A re-key that corrupts `-whole` tables — feeds #110

WR1 moved **39,967** functions out of `expr-call-in-expr-data-addr-*` into
`call-arg-multi-sym`, plus **12,327** into keys naming their next blocker
(`expr-op-0x9B` +6,670, `call-ref-cflow-jump` +5,657). Totals reconcile and the
new names are truthful, but **the `-whole`/`-more` grammar-completeness suffix is
lost on that family**: a table built by grepping `-whole` now **under-counts by
18,931**. The `:eof`/`:mid` suffix preserves the distinction in a different
field. Board **#110** already tracks a `-whole{k}` over-count on ~27,600
functions; these are two different corruptions of the same ranking input and
should be repaired together.

# DRAFT for `docs/ROADMAP.md` §9.12 — paste verbatim, then delete this file.

Kept out of `ROADMAP.md` on purpose: that file is the recorded add/add conflict
site for concurrent lanes (`docs/rungs/README.md`), the coordinator lands §9.12
serially, and this lane was told not to touch §1–§9.11. Everything below is the
section text.

---

### 9.12 W-LABEL — the pin §9.10 asked for was smaller than §9.10 thought, and the label counter is an ORDINAL rule (2026-08-01)

Lane `w-label`, board **#137** then **#135**. Pre-registration in
`docs/rungs/_2026-08-01-w-label-prereg.md`, committed at `6e3e9d3` before the
first mutation ran and before the first `.cod` was captured.

#### 9.12.1 #137 — `cargo test` pinned WR1's ordering rules in NEITHER lane

§9.10 stated the gap as *portable*: the fixtures are toolchain-gated, so on the
portable lane nothing pins the two rules. **The gap is one column wider than
that.** Three mutations, each a one-site edit implementing the rule WR1 got
wrong on its first differential, run against the **base tree before any new test
existed**:

| mutation | portable `cargo test --workspace` | toolchain `cargo test --workspace` | `c2rs diff wr1_sym_addr.cpp` |
|---|---|---|---|
| **M1** the address `addi` at its slot's turn in the descending walk | 571 / 0 | **571 / 0** | Mismatch @ obj 821 |
| **M2** REFLO written at `hi_off + 4` (both emitters) | 571 / 0 | **571 / 0** | Mismatch @ obj 1552 |
| **M3** `lo_off` derived as `base + 4` instead of searched | 571 / 0 | **571 / 0** | Mismatch @ obj 1552 |

The toolchain column is the surprise, and it refutes the lane's own registered
control (P0′, which predicted the toolchain lane would catch each one).
`crates/c2-harness/tests/differential.rs` runs the port against the reference on
**three named fixtures** — `add3.cpp`, `il_bool_materialization.cpp`,
`il_call_return.cpp` — and `wr1_sym_addr.cpp` is not among them. So
`cargo test --workspace` **never grades that fixture at all**, with or without a
toolchain, and the two lanes' totals are equal *because the integration tests
report `SKIP` and still count as `ok`* — no count distinguishes them and only a
mutation can. The single judge that went red is `scripts/gate.sh`: under M3,
**GATE: FAIL, 10 of 12 lanes MISMATCH**, 2,472 fixture-verdicts.

Restate §9.10's standing rule with the correction: a rung that touches `coff.rs`
must add a **portable** assertion for each ordering rule it establishes, because
the differential that catches it is `scripts/gate.sh`, **not** `cargo test`, and
a contributor who runs the workspace suite sees green either way.

**Eight tests, all portable, no toolchain**, in the three files where the two
rules actually live:

* `codegen/calls.rs` — the address `addi` is emitted LAST with a literal at a
  **strictly lower** slot (`s->m3(7, &gI)`, symbol at slot 2), plus the
  **symbol-at-slot-0 control** (`gsp(&gI, 7)`) which must stay **green** under
  M1. WR1's hand fixture had three copies of the control and none of the
  discriminator.
* `lib.rs` — `data_refs_of` **searches** the body for the low-half `addi`
  instead of assuming `hi_off + 4`, rebases both halves by the function's
  `.text` offset, and refuses four bodies it cannot read.
* `coff.rs` — the emitted quad's REFLO lands at `lo_off` in **both** emitters,
  the records are ascending-VA with REFHI ahead of its PAIR, the pooled-FP quad
  **is** adjacent (the negative that says the two quads are genuinely
  different), and the label triple's three slots bind `$M(n)`→prologue length,
  `$M(n+1)`→function length, `$T(n+2)`→`.pdata`, with the two `$M` written to
  the symbol table in the **opposite** order and the callee external between
  them.

**Mutation evidence — seven mutations, seven distinct messages**, each red on
the portable lane:

| mutation | site | assertion that fired |
|---|---|---|
| M1 descending walk | `sym_slots_text` | (c) `addi` must come LAST — and the slot-0 control stayed **green** |
| M2 REFLO at `hi_off+4` | `coff.rs`, both emitters | (d) packed, (h) COMDAT |
| M3 `lo_off = base + 4` | `data_refs_of` | (d) derivation |
| M4 PAIR emitted before REFHI | `coff.rs` | (f) record order |
| M5 the low PAIR dropped | `coff.rs` | (b) record count |
| M6 the two `$M` swap meaning (packed) | `coff.rs` | (n) `$M(n)` is the prologue length |
| M7 the two `$M` emitted in numeric order | `coff.rs` | (o) `$M(n+1)` is written first |

**M6 is the one worth reading.** The first draft of that test called only
`emit_comdat_obj`; swapping the two `$M` inside `emit_obj` under it left
`cargo test` at **85 passed / 0 failed**. One rule in two emitters, pinned in
one, is exactly how this file's `.pdata`-ordering bug survived. The shipped test
asserts both emitters.

Two smaller instrument facts, recorded because both are the
absence-read-as-success shape:

* `c2rs bench` is the **oracle self-test**, not the port differential. It prints
  `206 pass, 0 fail` under M1. `scripts/configure_existing_worktree.sh`
  advertises it as "every fixture, the correctness gate", which is how the lane
  nearly read a green bench as evidence that M1 was harmless.
* Writing the FP-adjacency test against `emit_comdat_obj` read **0 relocation
  records**: the COMDAT emitter carries no constant-pool code, because
  `PortC2::build` refuses a pooled constant under `/Gy`. It was moved to
  `emit_obj` rather than shipped as a control run where the effect cannot appear.

#### 9.12.2 #135 — the allocation order, widened from one body to 80 listings

`scripts/gt_label_cod.py`: 20 shapes × 4 flag sets (`/O1 /Oi /EHsc`, `/O1`,
`/O2 /EHsc`, `/Ox`), **80 of 80 listings captured**. Five shapes are the
**fitted** set; the other fifteen were held out and not looked at until the rule
was written.

**The rule.** In allocation order (ascending label number) the counter is
consumed **per function, in `.text` order**, and within one function:

1. one **funclet-entry** label per funclet the function needs (`__catch$k` /
   `__unwind$k`), **first**, before any of that function's `$M`/`$T`;
2. the function's **EH state-transition `$M`** block, ascending;
3. the **state table's own `$T`**, in `.rdata`;
4. then **one triple per emitted body** — the main body first, then each funclet
   in emission order — each triple exactly `$M(n)` prologue end · `$M(n+1)` body
   end · `$T(n+2)` `.pdata` record, consecutive, and the triples of one function
   consecutive **with each other, stride 3**.

Steps 1–3 are empty for a function with no EH, which collapses the rule to the
single triple `coff::plan_labels` already ships.

Eleven ordinal predicates, graded per (probe, mode):

```
                                      FITTED (5)   HELD OUT (15)
P1  every .pdata $T closes a triple      16/16         40/40
P2  $M(n) prologue < $M(n+1) end         16/16         40/40
P3  one triple per emitted body          16/16         40/40
P4  funclet allocated first                6/6         26/26
P5  …and emitted last                      6/6         26/26
P6  state table below the triples          6/6         26/26
P7  the $M block splits (EH)               6/6         26/26
P7b …and does not, without EH            10/10         20/20
P8  a function's triples stride 3          6/6         26/26
P9  functions in .text order                 —         16/16
P10 the main body's triple first           6/6         26/26
TOTAL                                    94/94       312/312  = 100.0 %
```

**Held-out accuracy: 312 of 312, 100.0 %,** on shapes the rule was not fitted on
(loop, switch, nested try, two catches, EH beside plain, two EH functions in one
TU, ctor/dtor, virtual, FP leaf, relational comparator, five leaves,
leaf-then-framed, many locals).

**The control that decides whether that is news.** A predicate that restated the
shipped model would go green on the whole in-class population by construction:

```
`coff::plan_labels` accounts for EVERY label in the TU
  non-EH rows   24/24   100.0 %
  EH rows        0/32     0.0 %
```

The shipped model is **complete on every non-EH row and complete on no EH row**.
The gap is entirely EH, the new rule closes all of it, and — stated honestly —
**the new rule adds nothing on non-EH bodies**: P4–P8 and P10 are `n/a` there
and what remains is the shipped triple.

**Falsification, because 312/312 is the shape this project reads as success when
it is really absence.** Seven mutations of the parsed allocation, each of which
must turn its predicate red:

| mutation | went red |
|---|---|
| every `.pdata` `$T` one higher | P1 (56) |
| funclet allocated last | P4 (32) |
| `$M(n)` / `$M(n+1)` offsets exchanged | P2 (56) |
| state table allocated above the triples | P6 (32) |
| the funclet's triple ahead of the main body's | P10 (32), P5 (32), P6 (32), P8 (32), P4 (20), P1 (2), P3 (2) |
| functions allocated in reverse `.text` order | P9 (16) |
| triples spaced 4 apart instead of 3 | P1 (56) |

**Two predicates survive every mutation: P7 and P7b.** They are §9.3's headline
— "the `$M` block splits around the `$T` tables" — and they are **entailed** by
P1 + P6 + P8 + P10, not independent evidence. Once a function has more than one
triple a `$T` necessarily sits between two `$M`; the split is a consequence of
"one (M,M,T) triple per emitted body, main first, then funclets", which is the
load-bearing statement. §9.3's phrasing is true and is not the finding.

**Two corrections to §9.3's wording**, both TU-versus-function scope:

* "the funclet is allocated **first**" is a **per-function** statement, not a
  per-TU one. At TU scope it is **false on 2 of 26 EH cells** — `eh_loop_two_fn`
  at `/O1` and at `/Ox`, where a first function's labels are allocated before a
  second function's funclet. Per function it is 32/32.
* the same for "the state table sits below the triples".

**What is NOT MODELLED, and the round refuses to guess it.** The rule above is
**ordinal**. The counter also consumes slots it never emits, and those gaps are
**not constant**:

```
gap: last funclet label  →  first EH-state $M    2, 3, 4, 5, 7, 8, 9, 10, 11
gap: state table $T      →  first triple         0, 1, 2, 3
```

So the *numbers* cannot be predicted from the shape, only their **order**.
Registered prediction B5 — "≥ 90 % of held-out label numbers predicted exactly
from the TU's first label" — is **REFUTED**, and that is the round's most useful
negative: `plan_labels` needs cardinal numbers, so **#135 ships no
`plan_labels` change**. A wrong stride is a wrong `$M` number and a wrong `$M`
number is a wrong-bytes obj. What ships is the instrument, the ordinal rule, and
the portable pin on the triple's slot binding (§9.12.1), which is the half of
#135 that *is* transcribable today.

A worked transcription, `eh_two_catch` at `/O1 /Oi /EHsc`, one function, three
bodies:

```
__catch$2553  funclet entry   text 0x5c     allocated first
__catch$2554  funclet entry   text 0x84
$M2564        EH state        text 0x24
$T2565        state table     .rdata
$M2568/$M2569/$T2570   main body      0x24 / 0x54 / .pdata
$M2571/$M2572/$T2573   __catch$2553   0x64 / 0x7c / .pdata
$M2574/$M2575/$T2576   __catch$2554   0x8c / 0xa4 / .pdata
```

Three instrument defects, all found before a verdict was read, all recorded
because each printed a plausible row:

1. a body-end `$M` took the **next** function's first offset — under `/Gy` that
   restarts at 0, so every multi-function TU read "end 0 < prologue 12" and
   **15 sound cells went red**;
2. `/Ox` names its sections with a bare `.rdata` directive rather than
   `NAME SEGMENT`, so every `/Ox` `$T` was attributed to `.XBLD$W` and the
   state-table predicate scored **`n/a` on all 20 `/Ox` rows** — which prints
   exactly like a predicate that passed;
3. a `.pdata` `$T` row sits **outside any `PROC`** and names its body in its
   `DD` operand; binding by position put all 56 of them in `fn_ix = -1` and the
   triple predicate read **0 of 56**.

#### 9.12.3 `LABEL_COUNTER.md` §6.15–§6.19 — the `.cod` evidence leaves every one of them UNTOUCHED

Not "widens some, refutes none". **Untouched, all of them, and for an
instrumental reason rather than a numerical one.**

§6.15–§6.19 do not measure the label counter. They measure the **inline-decline
decision** — how many of `N` call sites survived at a given callee size `s` —
through `scripts/gt_inline_decline.py`, which reads **REL24 relocation counts
and `bcctrl` counts** and reads **zero label symbols** (`grep -c` for `$M`,
`$T` and `first(` over that script: **0**). §6.15 says so itself: *"§6's law is
graded on label strides"*, distinguishing itself from the rounds that follow.
And §9.5 already recorded that the listing leaves the `/O1` inline-decline
schedule *unmoved*.

A `.cod` allocation order is a statement about the ordering of the counter
**within** a body. It is commensurable with §1 and §6.0–§6.14 (the counter, law
L′) — that is board #135 — and with nothing in §6.15–§6.19.

**The two vacuous ones specifically, and why `.cod` is not their remedy either.**
They are already named by the document's own §6.20 audit, so this lane did not
find them:

* **§6.15.2 — "dead locals move the decline by zero."** Vacuous because a
  `deadloc` ladder moves `s` by **zero by construction** — that is the point of
  the probe — so all twenty rungs sit at one index, 16 bytes below the nearest
  band edge, every cell saturated. A listing changes the *readout* of the
  decline, not the ladder's index range. **Untouched.**
* **§6.17.8 — "`/Ox`: there is no linkage split at all."** Vacuous because the
  sweep was `range(0, 9)` hardcoded and stops 28 bytes short of the only `/Ox`
  threshold there is; 36 cells rested on six, and the two rung kinds the section
  names first contributed none. Again a range-design fault. **Untouched.**

The one route by which `.cod` could ever touch these rounds is as a **second,
name-carrying source for the site count** — the listing prints every surviving
call by callee name, where the relocation table prints it by index. That is the
#136 relationship (a second instrument for the same observable), not new
evidence, and it is not scheduled here.

#### 9.12.4 Pre-registration scores

Registered at `6e3e9d3`, before the first mutation and the first capture.

| | registered | measured | |
|---|---|---|---|
| **P0** nothing portable pins either rule | portable stays 571/0 under M1–M3 | 571/0, three times | HIT |
| **P0′** each mutation is caught by the toolchain lane | toolchain `cargo test` goes red | **571/0, three times** | **MISS** |
| P1 `addi` last with a literal at a lower slot | passes; red under M1 | red, message (c) | HIT |
| P1′ the slot-0 control stays green under M1 | green | green | HIT |
| P2 `lo_off` searched, not `hi_off+4` | red under M3 | red, message (d) | HIT |
| P3 REFLO at 8 in the emitted obj | red under M2 | red, (d) and (h) | HIT |
| P4 the test-block total moves 571 → 577, [575, 580] | — | **571 → 579** | HIT |
| B1 non-EH triple contiguous and in order, 100 % | 100 % | 100 % (56/56) | HIT |
| B2 allocation order is text order, non-EH, 100 % | 100 % | 100 % (16/16 held out) | HIT |
| B3 funclet allocated first / emitted last, 100 % | 100 % of EH bodies | 100 % **per function**; 2 of 26 fail at TU scope | HIT, with a scope correction |
| B4 the `$M` block splits around the `$T`, 100 % | 100 % of EH bodies | 100 % — and **entailed, not independent** | HIT, vacuously |
| B5 ≥ 90 % of held-out label NUMBERS predicted exactly | ≥ 90 %, refuted below 70 % | **not modelled at all** | **MISS** |
| B2′ control: `plan_labels` < 50 % on refused shapes, new rule ≥ 90 % | discriminates | **0.0 % vs 100 %** | HIT |
| C `.cod` touches at most one §6.15–§6.19 negative | ≤ 1 | **0** | HIT |

**13 of 15.** Both misses are worth more than the hits. **P0′** changes a
standing rule: the lane predicted the toolchain lane would catch the mutations
and it does not, because `cargo test` never grades that fixture — so §9.10's
"portable lane" framing understates the gap by one column, and the sentence a
contributor needs is "run `scripts/gate.sh`", not "run the tests". **B5** is the
one that stops a bad change landing: without it this lane would have had an
ordinal rule and an invitation to fit a stride to five samples, and the gaps say
the stride is not there.

Two registered predictions were graded and found **vacuous rather than wrong**,
and are called out rather than counted as evidence: **B4** (entailed by B1/B3
once a function has two triples — it survives every one of the seven
falsification mutations, which is the definition of measuring nothing) and
**P9's fitted column** (0 of 0 cells: no fitted shape has two labelled
functions, so that column could not have failed; the predicate is graded on the
held-out column alone, 16/16).

#### 9.12.5 Gate evidence

Test blocks: **571 at the merge base `33d0049`, 579 at the tip** — the diff
§9.10 asks for, quoted at both ends. **Eight** new blocks, every one portable.

The first tally read **580**, and the extra one was a `#[test]` literal inside a
comment this lane wrote *about* the count. `git grep -c` cannot tell a comment
from an attribute, so §9.10's own metric is one a rung can inflate by writing
about it. The comment now spells the attribute out in prose and the grep and the
runner agree at 579.

* `cargo test --workspace` — **579 passed, 0 failed, 1 ignored** (the ignored
  one is the pre-existing doc-test), and the same **579 / 0 / 1** on the
  **portable** lane (`C2RS_WIBO=/nope C2RS_CL_EXE=/nope C2RS_C2_DLL=/nope`) —
  the eight new blocks are in both numbers.
* `c2rs selftest` — **206/206 PASS**, 0 fail, 0 error.
* `scripts/gate.sh --jobs 6` — **GATE: PASS, 12/12 lanes ran and every one
  graded a corpus**, 2,472 fixture-verdicts, 206/206 in every lane, 0 mismatch.
  `scripts/gate.sh --selftest` — PASS, 15 cases.
* No port behaviour changed: this lane added tests, one tooling script and this
  section, and wrote no emitter code.

DRAFT for `docs/ROADMAP.md` §9.13 — written by lane w-adjust, to be landed by
the coordinator. Nothing in §1–§9.12 is touched. Full record:
`docs/rungs/2026-08-01-w-adjust.md`.

---

### 9.13 W-ADJUST — the largest never-measured row is worth 472, and the only clean one is worth 1,385

§8.7 named two rows and told this lane to treat them differently: **measure
`expr-intrinsic-this-adjust`, do not build it**; build
`expr-call-in-expr-recv-object-then-type-ptr-whole` only if its counterfactual
holds up. Both were measured the same way — one scratch sink at the row's own
refusal site, one warm scan, Δ`emit-in-class` against 34,674 — and the second was
built. The emitted census is **19.37 % → 20.15 %**.

| run | bodies | emitted | Δ emitted |
|---|---:|---:|---:|
| base `33d0049` | 703,875 | 34,674 | — |
| sink disabled (control) | 703,875 | 34,674 | **0** |
| #127, adjust offset 0 only | 708,193 | 35,108 | **+434** |
| #127, any adjust offset | 708,231 | 35,146 | **+472** |
| #128, named object receiver | 706,402 | 36,059 | **+1,385** |

The control is the row that matters: the *instrumented* binary with the sink
disabled reproduces the base scan on all five of its published numbers. An
instrument whose inertness is asserted rather than run is the twelfth instance of
this project's dominant failure, and it was cheap to avoid.

**#127 is 472 emitted, 5.4 % of the row, and 92.0 % of that is free.** 434 of the
472 are at adjust offset 0, where the receiver's true operand stream is
`[Load(this)]` and no new codegen is needed. The 10,469 `-whole` figure §8.7
already flagged is confirmed to belong elsewhere, and **8,790 is not the row's
worth by a factor of 18.6**.

**The row's name never said what it is.** 135,926 of its 135,941 bodies decline
at `eat_receiver_this` — the member-call production bails non-committally, the
assignment parser then runs `parse_expr`, and `parse_expr` stops on the
intrinsic. **The census key names the second reader's stop, not the first
reader's refusal.** That is also why the row has no `-whole` bit: its `Block` ctx
is `expr-intrinsic`, not `CALL_IN_EXPR`, so `whole_body_is_one_value` never runs
on it. §8.7's "largest never-measured row" is a consequence of the attribution,
and the same mechanism hides the completeness of **7,712 further clean emitted
functions** at the receiver sites.

**What it means for #131.** The two arms measured convert at **5.4 %** and
**100.4 %** — 19× apart — so no rate transfers, and #131 must be sized off stock.
Measured in emitted units for the first time (per-row dump joined to the obj's
`.text` COMDAT leaders):

| the three receiver-designator sites | emitted blocked | clean | clean ∧ complete |
|---|---:|---:|---:|
| `tail-recv-not-a-plain-b9-load` | 23,158 | 7,670 | 19 |
| `chain-recv-not-a-plain-b9-load` | 13,896 | 1,441 | 1,380 |
| `cmp-second-recv-not-a-plain-b9-load` | 6 | 0 | 0 |
| **total** | **37,060** | **9,111** | **1,399 (+3)** |

37,060 is **29.3 % of all blocked emitted** — #131 is the largest single site on
the emitted board, larger than any census key — and its honest worth is
**≈ 2,600 emitted (1.4 pp)**: 1,385 taken here, 472 from #127, and ~710 if the
remaining clean-not-whole stock converts at #127's own 15.3 %-of-clean rate.
**The raw stock overstates the site by about 14×.** The optimistic ceiling, every
clean row converting, is 9,111.

§9.11's `-whole` corruption was **verified against `:eof`/`:mid` rather than
trusted**: re-counted with the suffix, the clean-and-complete stock at these three
sites gains **3** functions (1,399 → 1,402). The corruption is real and it is
0.2 % of this table, which matters as a negative result — the 7,712 clean residue
is genuinely unmeasured, not merely mis-suffixed.

**#128 converts 100 % and its key's second half was never a blocker.** All 1,380,
plus 16 from three neighbouring `recv-object` rows, against 11 re-filing under
named codegen gates. The key reads `-then-type-ptr`, i.e. "the receiver form *and*
a pointer-typed operand", so it looked like two widenings. It is one: the `-whole`
measure's operand vocabulary is `eat_int_operands` → `eat_int_like`, while the
**shipping acceptance path** is `eat_call_args` → `parse_expr` with
`eat_int_like_or_ptr4`, which has admitted width-4 pointers since W22. **The
census measure is narrower than the emitter, and the difference is printed as a
second construct.** On the emitted board that mis-describes a further **7,983**
functions (`…-and-call-more` 5,663, `…-and-deref-load-more` 1,462,
`…-and-plumbing-more` 449, `…-and-op-more` 409). Repairing `eat_int_operands`'s
type gate is a small instrument change and belongs with board #110's `-whole{k}`
over-count and §9.11's lost suffix: **three corruptions of the same ranking
input.**

The 1,380 are **four distinct mangled names**, three of which are 1,379 of them —
`??6DebugFailer@@QAAXPBD@Z` (759), `??6DebugNotifier` (604),
`??6DebugWarner` (16) — one header-inline `TheDebug << s;` forwarder emitted once
per TU across 803 TUs. The emitted census counts COMDATs and 1,385 is 1,385, but
the *differential coverage* behind the rung is one source shape, which is why the
generated axis carries more weight here than the fixture.

The rung **reconciles to the unit**, which is the control that matters for a
change that re-routes bodies between productions. `chain-recv-not-a-plain-b9-load`
falls 94,948 → 30,183 and the 64,765 re-routed bodies account for themselves
exactly (2,537 accepted + 24,874 `tail-object-receiver-is-not-a-tail-call` +
28,300 "does not end at the call" + 9,046 argument-vocabulary + 8); the 2,539 that
changed dispatch arm resolve as 2,527 in class + 10 refused one layer later by the
`.gl` linkage gate + 2 committed refusals. **One in-class shape label moved and
nothing shrank** — `multiarg-tail-call` 27,868 → 30,395 — so no previously
accepted body changed production, changed shape, or fell out of class. Stated
positively over 2.46 M bodies rather than as the absence of a complaint.

#### 9.13.1 ALARM — WR1's ordering rule was wrong from two setup words up, and it was live on mainline

The new fixture mismatched at first build. Bisected to a body with **no receiver
in it at all**:

```cpp
extern int gI;  void gs3(int*, int, int);
void b3() { gs3(&gI, 3, 4); }      // pure WR1: a data symbol as a call ARGUMENT
```

At `33d0049`, with none of this lane's code present, that body is **1/1 in class
and the port emits it wrong** (`Port=Mismatch @ offset 545`) — verified by
checking out `33d0049 -- crates`, rebuilding and diffing. WR1's rule was *"the
address `addi` is emitted LAST"*. c2's own `.cod` listing says it goes **SECOND**,
after exactly one word of the descending non-address walk:

```
    3d600000  lis  r11,?gI@@3HA
    38a00004  li   r5,4        <- one word of the descending walk
    386b0000  addi r3,r11,?gI  <- the address, SECOND
    38800003  li   r4,3        <- …and the rest of the walk follows
    48000000  b    ?gs3@@YAXPAHHH@Z
```

**At one setup word the two readings are the same sequence.** Eleven cells now pin
the rule — walks of length 0 to 4, the address at slot 0 and at a middle slot,
literals and in-place formals in the walk, free and member callers — and it
*subsumes* WR1's rule rather than contradicting it: address-last is the n ≤ 1
case.

Three consequences, each larger than the fix:

1. **§9.10's standing rule is attached to the wrong thing.** It says a rung that
   touches `coff.rs` must add a portable assertion for each ordering rule it
   establishes. This rule lives in `codegen/calls.rs`, had no unit test either,
   and failed the same way. **The rule belongs to the ordering rule, not to the
   file.** `the_data_address_addi_is_emitted_second_not_last` is now that
   assertion and it runs with no toolchain.
2. **The generated sweep was green over a wrong rule because the axis did not
   exist.** `53-data-symbol-addr.py`'s WR1 block emitted 70+ cases varying the
   address's slot, its destination register, the literal's value, the object's
   type and the mangled name's length — and never the **count**. Every case had
   ≤ 1 literal. Generated axes find what hand fixtures structurally cannot *only
   where the generator has the axis*; an axis a fragment does not vary is exactly
   as invisible as a fixture that does not arrange the case.
3. **A fixture's own blind spot is worth writing into the fixture.**
   `wadjust_obj_recv.cpp` states in its header that it cannot discriminate any
   slot-dependent rule, because the receiver is argument zero by construction —
   the sentence WR1's ALARM had to be discovered to produce.

#### 9.13.2 Pre-registration score — 4 of 8, and three of the misses are the findings

| | registered | measured | |
|---|---|---|---|
| E1 | #127 bodies 14,000, [5,000 , 30,000] | **4,356** | **MISS**, below the floor |
| E2 | #127 emitted 1,000, [300 , 3,000] | **472** | HIT (2.1× high) |
| E3 | ≥ 60 % of E2 at adjust offset 0 | **92.0 %** | HIT |
| E4 | control: gate disagreement goes non-zero | **0** | **MISS** |
| E5 | #131 ≤ 4× #127's realized | **5.5×** | **MISS** |
| E6 | #128 emitted 1,380, [600 , 1,380] | **1,385** | HIT on the point, ceiling wrong |
| E7 | #128 is ≤ 10 distinct names | **4** | HIT |
| E8 | receiver alone converts < 100 | **1,385** | **MISS**, 14× |

* **E1/E2 are the WR1 lesson repeating**: both estimates came from the same
  body-column anchor and both were high, 3.2× and 2.1×. The emitted number landed
  inside its interval only because the interval had been widened on WR1's
  precedent. A body-column anchor is not a source of an emitted estimate *even
  when it is transparently discounted*.
* **E4 registered the wrong control**, which is §9.9.2 again. The sink hands
  codegen `[Load(obj)]` where the true stream at `k != 0` is `[Load, Lit, Add]`,
  so it *does* over-claim — and `census/gate disagreement` cannot see it, because
  the port **accepts** the wrong stream and would emit wrong bytes rather than
  refuse. A gate-agreement counter separates "census accepted, port refuses"; it
  is silent on "census accepted, port would get it wrong". The control that works
  is the one run for #128 and not for #127: build it and put it in front of the
  differential — which is exactly how §9.13.1 was found.
* **E6's hit hides a wrong assumption**: the interval's ceiling was the row's own
  emitted count, on the reasoning that a row cannot convert more than itself. It
  took 16 functions from three neighbours. **A census row is not a unit of work.**

#### 9.13.3 Gate evidence

At `be797bf`, worktree configured against the shared toolchain:

* `cargo test --workspace` — base `33d0049` **571 passed, 0 failed, 1 ignored**
  → tip **576 passed, 0 failed, 1 ignored** (the ignored one is pre-existing).
  Both totals measured, not inferred: the base was rebuilt from
  `git checkout 33d0049 -- crates fixtures` and re-run.
  **`#[test]` count over `crates/` 571 at the merge-base `33d0049` → 577 at tip**.
  Five of the six new grep lines are real tests and the sixth is the literal
  `#[test]` inside a doc comment — `git grep -c` counts lines, and a whole-tree
  grep is additionally polluted by prose in `docs/`. Five new portable tests: two pinning the ordering rule and its refusals, two pinning
  both directions of the `26`/`26` receiver-vs-chain discriminator.
* `c2rs selftest` — **208/208 PASS**, 0 fail, 0 skip.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,496 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — 42,719 configurations × 12 lanes =
  **512,628 gradings, 512,628 graded, 0 mismatches**; 406 of 406 declared family
  pairs reached and emitted, refusal-frontier residue 0.
* 878-TU workload scan — 6 match, **0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies 706,402/2,462,571 (28.69 %); **emitted 36,059/178,968 (20.15 %)**;
  census/gate disagreement **0**.
* Fixtures — `wadjust_obj_recv.cpp` 21/21 in class and `Port=Match`;
  `wadjust_obj_recv_neg.cpp` 0/11 and `Port=NotImplemented`;
  `wr1_sym_addr.cpp` 27/27 and `Port=Match` with its six new arity cells.

#### 9.13.4 New board items

* **#139 — repair `eat_int_operands`'s type gate to match the emitter's.**
  **Re-attributes** 7,983 emitted functions whose keys name a `-then-type-ptr`
  second construct the emitter does not refuse. Most of them carry `-more` and so
  would not convert on that widening alone — the claim is about where the ranking
  says the work is, not about free functions. Instrument, not a rung; goes ahead
  of any ranking taken off those rows. Sits with #110 and §9.11.
* **#140 — `expr-intrinsic-this-adjust` at adjust offset 0, 434 emitted.**
  Measured end to end here; the sink is 30 lines and is in `db812f7`. Needs the
  receiver designator to return an operand form richer than a token, which is the
  refactor #131 needs as a whole. **Schedule it at 434, not at 8,790.**
* **#141 — `call-arg-sym-permuted`: the data address beside a formal that has to
  move.** Refused by WR1 on one probe; it blocks every free-function caller of
  the #128 shape and is the largest single refusal inside the row this lane took.
  c2 pre-saves into r11 and moves the `lis` to r10 at two shifting formals — a
  designed capture grid over (formals moving) × (walk length), for which this
  lane's `q1`/`q3` listing probes are the template.
* **#142 — the other clean-not-whole receiver arms** (7,712 emitted, none with a
  completeness bit): `expr-op-0x27` 5,629 at this site, `expr-brfalse` 1,484,
  `assign-store-type-0x86` 1,138, `expr-intrinsic-dynamic-cast` 1,003. Each needs
  its own counterfactual; the two run here differ by 19×, so **no rate may be
  borrowed between arms**.

DRAFT for `docs/ROADMAP.md` §9.15 — written by lane `w-eh`, to be landed by the
coordinator. Nothing in §1–§9.14 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-eh-prereg.md`, committed at `689ba57` before the first
capture. Full record: `docs/EH_RECORDS.md` §11.

---

### 9.15 W-EH — the EH records by name, and the label gaps are the §1.1 surcharge block (2026-08-01)

Lane `w-eh`, boards **#133**, **#121**, **#138**. **Measurement and
transcription only: no port code, no `crates/` change, census moves by 0 by
construction.** That was the registered expectation and it is the outcome.

Two corrections to the brief this lane was given, both found before any
measurement and both worth recording:

* **`docs/EH_RECORDS.md` already existed** — 1,711 lines, §1–§10, derived from
  **obj bytes**. It was not this lane's to create. That is a better starting
  position than a blank page, because it makes the byte model a **control that
  can go red**: #133 becomes a second, name-carrying source for a layout already
  fitted, which is the #136 relationship (§9.9.3) rather than a transcription.
* **#121 as the brief states it is not the board item §9.2 names.** §9.2 attaches
  #121 to the EH records; the brief describes it as
  `codec::gl_offset_framed`'s over-fit (`GAPS.md` §8.2). Both were addressed and
  they are unrelated artifacts — see §9.15.2.

#### 9.15.1 #133 — the layout, from 21 shapes rather than one

`scripts/gt_eh_cod.py`, **110 listings, 110 captured** — 15 EH shapes × 4 flag
sets (`/O1 /Oi /EHsc`, `/O1 /Oi /EHa`, `/O2 /EHsc`, `/Ox /EHsc`), plus 5
held-out `maxState` shapes, 5 held-out gap combinations and 40 single-axis gap
probes. The axes are **structural counts**, which is the §9.13.1 lesson
applied rather than quoted: try blocks 0–4, nesting depth 0–4, catches per try
1–4, destructible objects 0–5, functions per TU 1–3, and every catch form
(value, `&`, `const&`, pointer, ellipsis). Two probes fitted; the rest held out
with their counts registered in the script before capture.

The full field-by-field layout is `docs/EH_RECORDS.md` §11. What is new against
the byte-derived §8.3:

* **§8.3's `FuncInfo` is confirmed 9 of 9** — no field moved, none added, still
  no `dispUnwindHelp`. The control could have gone red and did not.
* **`maxState` = (destructible objects) + 2 × (lexical `try` blocks).** A try
  block is worth **two** states. Every A2 miss was this cell and all in one
  direction. Registered and graded on **five shapes it was not fitted on**,
  including a four-deep nest and a four-block sequence — the two arrangements
  that separate "per try block" from "per nesting level" — **10 of 10 exact.**
* **Try blocks are emitted INNERMOST FIRST**, with the enclosing block's
  `tryLow..tryHigh` spanning the inner one. §8.3 never fixed that order; a table
  built in source order is wrong on every nested function.
* **The 8-byte pad is printed, not inferred.** §8.3 *proved* the 9-dword
  `FuncInfo` from two symbol offsets; the listing emits a literal `ORG $+4`.
  Both pad values occur (0 on 13 probes, 4 on 50).
* **`/EHa` is accepted, and it is a different state model — not a two-dword
  variation.** `EHFlags` `01H` → **`00H`** on 15 of 15, and the `catch(...)`
  `adjectives` `040H` → **`00H`**, so §8.3's "1 on all 21" and "`0x40` ellipsis"
  are `/EHsc`-scoped. **This lane's first draft then said "everything else is
  byte-identical", and that was wrong** — measured across all 15 EH shapes with
  label numbers normalised, `/EHa` differs in **44 of 546 data slots on 15 of
  15 probes**: it *grows* `nIPMapEntries` and the ip-to-state array on **10 of
  15** (`h_try1` 1 → 3, `h_try3seq` 7 → 13, `h_nest3` 3 → 9), and on `h_catch4`
  moves `maxState` 2 → 3 and `catchHigh` 1 → 2. Only the no-try
  destructor shapes differ in `EHFlags` alone. **The layout is mode-independent;
  every count in it is not.** The wrong sentence survived until the comparison
  was actually run rather than asserted from the two fields the lane went
  looking for — §9.1's shape, caught before landing.

**The residue, named, because a correspondence graded on totality needs one.**
`__catchsym$F$k` — the `$k` suffix is **NOT MODELLED**. It is a `STATIC` symbol
whose name reaches the obj string table, so a wrong `$k` is a wrong-bytes obj.
On a sequential-try ladder the first `$k` equals `maxState` and the rest ascend;
**`h_catch4` refutes that as a law** (`maxState` 2, `$k` 6), and `h_2fn` shows it
is **per function** — two functions in one TU both get `$2`. Phase 5 needs this
and does not have it. Also open: `nIPMapEntries` for try shapes (§9.7 already
refuted the no-try rule there, and this lane **declined** those nine cells rather
than guessing, scoring them zero), and `adjectives` `0x02`.

**Totality, and why the headline number is not the evidence.** Every datum
claimed by a named field: **598/598 fitted, 2,920/2,920 held out, residue 0.**
That is exactly the shape this project reads as success when it is absence — and
here the failure mode is concrete. c2 **run-length-encodes**: `DD 2 DUP(00H)`
carries `nTryBlocks` *and* `pTryBlockMap` in one operand. The first version of
this instrument read `__ehfuncinfo$` as **8 dwords, residue 0, every field
claimed**, with `pIPtoStateMap` decoded onto `nIPMapEntries`.

So totality is graded beside an **arity** check that predicts each record's
length from a count field in a *different* record: **377/377 consistent.** Three
falsifications:

| mutation | totality | arity |
|---|---|---|
| the `DUP` expansion removed — the bug that really happened | **residue 0, SILENT** | **22 red**, `FuncInfo got 8 want 9` |
| `FuncInfo` truncated to 8 named fields | residue 8 / 70 | — |
| `HandlerType` read as 5 dwords (x86's `copyFunction`) | residue 36 / 281 | — |

**The first row is the finding.** The mutation that actually occurred is
invisible to the residue metric. *A totality count cannot see a short read* —
it needs a length predicted from somewhere else.

#### 9.15.2 #121 — NOT settled, and the number in `GAPS.md` §8.2 is 38, not 34

**Verdict: the listing does not settle #121, and it cannot.** `.cod` is an
artifact of c2's **output**; `codec::gl_offset_framed` frames records in c2's
**`.gl` input** bundle. §9.5 already refuted the existence of any IL dump with a
positive control, and nothing in 110 listings names a `.gl` offset.

That statement is unfalsifiable on its own, so it was given a number. The `.cod`
names every **emitted** function and #136 proved that set equals the obj COMDAT
set exactly — so the listing *can* adjudicate the emitted subset, and only that.
On `src/App.cpp`, where the over-fit bites: **158 emitted functions against
6,069 framed records = 2.6 %.** The listing is silent on the other 97.4 %,
because they are bodies that never reach an obj. Registered ≤ 5 %; measured
2.6 %. (The 158 is `GAPS.md` §8.2's figure carried through #136's proven
identity, not re-measured here.)

**Re-measuring the three figures rather than quoting them changed one of them.**
Directly over the cached `.gl`/`.ex` for `src/App.cpp` (`.gl` 1,512,566 B, `.ex`
2,552,214 B):

| `GAPS.md` §8.2 | re-measured |
|---|---|
| loosened predicate finds **6,069** | **6,069** — exact |
| of which **6,068** land on a `4F 1F` start | **6,068** — exact (the one miss is `0x0B0004F5`, far past the end of `.ex`) |
| shipped predicate finds **34** | **38** |

**34 is not the framing count.** The framing predicate hits **38**; the reader's
32-byte name bound then drops 4 as `records_nameless`, leaving 34.
`GAPS.md` line 2592 says "the gate's *reader* finds 34" and is correct; the
doc comment at `crates/c2-il/src/func/bind.rs:84` says "the gate's **framing**
therefore finds 34" and is **wrong by 4**. Two further corrections fall out:

* only **31** of the 34 pass `looks_mangled`, and `gl_defined_names` is
  all-or-nothing — it returns empty on the *first* framed hit with no nearby or
  non-mangled name. So `Bindings::per_record` binds **0** functions on
  `App.cpp`, not 34. "34" is *framed records the reader could name*, not
  *records that bind*.
* the `.ex` carries **9,196** `4F 1F` markers, not 9,033 — a different quantity
  (the census anchors on the `LO` marker) and easy to conflate. The loosening
  recovers 6,069 / 9,196 = **66.0 %**.

**So #121 stands open and needs a different instrument.** The over-fit is real
and confirmed at the two figures that matter; the listing is not its remedy.
`crates/c2-il` was not touched — lane `w-rerank` owns it — and the correction
above is a doc-comment fix for whoever does.

#### 9.15.3 #138 — the gaps are the §1.1 surcharge block, and they ARE additive

§9.12 measured `last funclet → first EH-state $M` at **2–11** and `state table
$T → first triple` at **0–3**, and refused to model them. The refusal was
correct. The reason is now measured, and it is **not** what the brief's three
candidates proposed.

**The leading registered hypothesis was wrong and it was cheap to kill.** C1
predicted ≥ 90 % of the gap slots would turn out to be labels the §9.12 parser
never read, under a prefix like `$LN`. Those labels **do** exist —
`$LN12@f`, `$LL3@f` — and they are a **separate, small, per-function** space
(observed 1..17) with no relation to the TU counter (25xx). **0 % of the gap
slots are named anywhere in the listing.** REFUTED.

**What governs them.** Holding the EH shape fixed at one destructible local and
moving **one axis per probe**:

> **G = 2 + 2 × [`f` is the FIRST emitted function in the TU] + Σ(`f`'s own
> `LABEL_COUNTER.md` §1.1-style surcharges)**

| axis moved | ΔG | note |
|---|---:|---|
| a **string literal** (an `.rdata` COMDAT + a `??_C@` symbol) | **+0** | **THE CONTROL.** §2.1 measures it at 0 slots; if G had moved, the model was dead |
| k **discarded** unreferenced statics, k = 1,2,4,8 | **+0** | 5 cells |
| a signed relational over two call results | **+2** | §1.1's exact integer, and it mints nothing |
| `_fltused` + a newly pooled FP constant | **+3** | §1.1's 1 + 2 |
| a loop | **+4** | not in §1.1's table |
| ≥ 1 extra call to a function declared elsewhere | **+2**, **flat** in k = 1..4 | |
| a try/catch instead of a bare destructor | **+3** | |
| **each body inlined into `f`** | **+3**, exactly linear | |
| a preceding emitted function | **−2** | see below |

**The −2 needed a discriminator and got one.** A ladder of k preceding emitted
leaf functions drops G from 4 to 2 and then **saturates** — which is consistent
with both "the first emitted function in the TU pays 2" and "a TU with more than
one function is different". The same leaf functions placed **after** `f`
(`x_trail1..4`) leave G at **4**. So the charge is paid by the **first emitted
function**, and ladder A alone was a control run where the discrepancy could not
appear.

**Graded on combinations it was not fitted on.** Five probes combining terms
(loop + 2 inlined + led; relational + 2 inlined; string + loop; led + 2 external
calls; led + pooled constant), predictions registered before capture: **5 of 5
exact.** The terms **add**.

**The answer to the brief's three candidates, separately:**

1. **Per-TU vs per-function counter resets — refuted.** The counter is monotone
   across every function boundary and no number is reused. Registered as
   expected-inert and it was.
2. **Labels consumed by bodies inlined away — CONFIRMED, at +3 each, and this
   nearly went unmeasured.** The first ladder used `static` callees and the
   second `__forceinline`, and **c2 emitted every one of them as its own
   COMDAT** — checked by `PROC` count rather than assumed. Both ladders moved
   *two* axes at once (bodies inlined into `f`, and functions added to the TU).
   The isolated term comes from contrasting them against the ladder where `f`
   does **not** call the leading functions: `x_fi_k − x_lead_k` = 3, 6, 9, 12.
3. **Labels allocated by phases that emit nothing — refuted *at G*, confirmed
   *elsewhere*, and the distinction is the point.** Discarded statics move G by
   **0** on all five cells. They do consume the counter: each one advances the
   TU's first label by exactly **3**, outside the block G measures. So "labels
   that reach no obj" is real and measurable — it is simply not the mechanism
   behind the inter-stage gaps.

**So: are the gaps predictable?** The honest answer is a third branch the
pre-registration did not offer. **G is governed by an additive law whose terms
are measured integers, and it predicts held-out combinations 5/5.** It is not a
compiler mystery, and §9.12's "not predictable from the shape" is precisely
right — G is not a function of the **EH shape** at all. It is the ordinary
`LABEL_COUNTER.md` §1.1 surcharge block, which §2.2 already established is
allocated **ahead of** a function's own `$M` pair (`extra == stride − base` on
all 21 framed rows). In a non-EH function that block sits before the pair and
nobody called it a gap; in an EH function the funclet labels are allocated first,
so the same block becomes **visible between the funclets and the ip2state `$M`s**.
§9.12 measured it across TUs whose surcharge content differed and correctly read
the spread as unmodelled.

**This does NOT license a cardinal `plan_labels`, and no `plan_labels` change
ships.** Two reasons, both load-bearing:

* One input is the **set of bodies c2 chose to inline**, at +3 each.
  `LABEL_COUNTER.md` §6.15.3 records that the `/O1` inline-decline schedule is
  *"generated by no formula"*, and §9.5 records that c2's strings **name** the
  emit-set predicate's disjuncts without formula-ising it. The **per-body cost is
  constant; which bodies is not predictable**. That is the precise sense in which
  the gaps are an inlining artifact — and it is a sharper statement than "they
  are unpredictable".
* Two terms are outside §1.1's measured table entirely (a loop at +4, the extra
  external call at +2 flat), and `EH_RECORDS.md` §9.8's own `G = 4 + Σmint` is
  now explained rather than repaired: its base **4 is `2 + 2`** — the true base
  plus the first-emitted-function charge — and **its 27 probes never varied
  which function came first**, so a two-term constant read as one. Its `qLOOP`
  miss (8 against an expected 6) is exactly this lane's loop term, which
  `Σmint` cannot see because a loop mints nothing (§2.1).

A wrong `$M` number is a wrong-bytes obj (§9.12.2). The rule stays **ordinal**.

#### 9.15.4 Pre-registration scores

Registered at `689ba57`, before the first capture; the `maxState` law and the
gap decomposition were each re-registered in their own commit before the
held-out round that graded them.

| | registered | measured | |
|---|---|---|---|
| A1 totality | residue 0 fitted and held out | 0 and 0 | HIT, and **near-vacuous alone** — see A1b |
| A1b arity | *(not registered — added when `DUP` was found)* | 377/377; catches what A1 cannot | — |
| A2 structural counts | ≥ 85 % exact, refuted < 60 % | **79.5 %** (62/78) | **MISS**, not refuted |
| A2′ the corrected `maxState` law | held out ≥ 85 % | **100 %** (10/10), **`/EHsc` only** | HIT, scoped |
| A3 `.cod` vs §8.3 `FuncInfo` | 9/9 agreement | 9/9 | HIT |
| A4 `/EHa` accepted, `EHFlags` ≠ 1 | accepted, flag moves | accepted, `01H`→`00H` on 15/15 | HIT — **and the axis was far bigger than registered**, see §9.15.1 |
| A5 `adjectives` by clause | `00`/`09`/`08` | exactly that | HIT |
| A6 structural-count law | counts are a function of the axis; `maxState` rises with (dtors + try) | `nTryBlocks` and the arrays exact; **`maxState` weighs a try DOUBLE** | **MISS** on the stated form |
| B1 #121 settled? | NOT settled, in principle | not settled | HIT |
| B2 fraction the `.cod` can adjudicate | ≤ 5 % | **2.6 %** | HIT |
| B3 re-verify 34 / 6,069 / 6,068 | all three exact | 6,069 ✓, 6,068 ✓, **34 → 38** | **MISS** |
| C1 gap slots named under an unparsed prefix | ≥ 90 %, refuted < 50 % | **0 %** | **MISS**, refuted |
| C2 counter per TU, monotone | no reset, no reuse | none | HIT (registered inert, and inert) |
| C3 inlining moves the gap | it does | **+3 per inlined body, linear** | HIT |
| C4 phases that emit nothing | moves the residue | **+0 at G, +3 each outside it** | **SPLIT** |
| C5 the verdict | branch (a) *or* branch (b) | **neither — a third branch** | **MISS** |

**9 of 15, with one split.** The misses carry this round:

* **C1** was the lane's *leading* hypothesis and it was refuted in the first ten
  minutes by one `grep` for `$`-prefixed labels. Killing it early is what left
  time for the ladders that actually answered #138.
* **C5** is the more interesting failure: the pre-registration offered a
  two-branch disjunction ("accountable from the listing" *or* "an inlining
  artifact, stop") and reality took a third — accountable from the **surcharge
  table**, with inlining as one unpredictable *input* rather than the whole
  story. **A disjunction registered as exhaustive was not**, which is worth more
  than either branch would have been.
* **B3** is the reason re-measuring beats quoting: `34` is a reader count, not a
  framing count, and a shipped doc comment says otherwise.
* **A2/A6** both missed on the same cell, and the miss produced the section's
  best result — a law graded 10/10 on shapes it was not fitted on. An estimate
  that is wrong in a single consistent direction is a law waiting to be written.

Two registered items are called out rather than counted as evidence: **A1**,
which is vacuous without the arity check that was *not* registered (it was added
mid-round when `DUP` was found, so it is an unregistered strengthening, not a
scored prediction); and **C2**, which was registered as expected-inert and is
inert — §9.12's P9 already implied it.

#### 9.15.5 Gate evidence

This lane wrote **no port code**: `docs/` (two files), `scripts/gt_eh_cod.py`,
and nothing under `crates/`. The gate is quoted to show it did not move, not to
claim it as evidence for anything above.

* `cargo test --workspace` — **584 passed, 0 failed, 1 ignored**. This **is**
  the merge-base count: `git diff --stat 99ed418..HEAD` is `docs/EH_RECORDS.md`,
  `docs/rungs/*`, `scripts/gt_eh_cod.py` and nothing else, so no test was added
  or removed. (§9.10's standing metric asks for the diff at both ends; here the
  diff is empty by construction, which is the honest form of it. Note the number
  is **584**, not the 579 of §9.12.5 or the 576 of §9.13.3 — those were
  different merge bases, and quoting a stale total as "unchanged" is exactly the
  §9.10 trap.)
* `c2rs selftest` — **208/208 PASS**, 0 fail, 0 skip.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes in the registry,
  **2,496 fixture-verdicts across all lanes, 0 mismatch in every one**
  (208/208 graded per lane; `/O1` 97 match, `/Ox` 99, `/Od` 1).
* Census, emitted census, and TU match are **unchanged by construction** — no
  acceptance path, no emitter and no census key was touched.
* `scripts/gt_eh_cod.py` — 110/110 listings captured across 4 flag sets.

#### 9.15.6 New board items

* **#143 — `__catchsym$F$k`, the per-function symbol ordinal.** The one piece of
  the EH record set §11 could not model, and it is a *name* that reaches the obj
  string table. Blocks a byte-exact Phase-5 emitter on any function with a try
  block. The `$LN`/`$LL`/`e$NNNN` numbers look like the same space and are the
  place to start.
* **#144 — `nIPMapEntries` for try/catch shapes.** §9.7 refuted the no-try rule;
  this lane declined to guess. `h_try1` 1, `h_try2seq` 4, `h_try3seq` 7,
  `h_nest3` 3 — not a function of any count in `FuncInfo`.
* **#145 — fix the `bind.rs:84` doc comment (38, not 34) and record that
  `Bindings::per_record` binds 0 on `App.cpp`.** One-line doc change plus a
  measured note; belongs to whoever holds `crates/c2-il` after `w-rerank`. Feeds
  #121, which is **still open** and still needs an instrument that reads the IL
  container, not the listing.
* **#146 — repair `EH_RECORDS.md` §9.8's `G = 4 + Σmint`.** The base is `2 + 2`
  (the second 2 being the first-emitted-function charge its 27 probes never
  varied), and `Σmint` should range over **all** §1.1-style surcharges, not the
  minting ones — which is what its `qLOOP` miss already was. Instrument
  correction, not a rung.

# DRAFT for `docs/ROADMAP.md` §9.14 — paste verbatim, then delete this file.

Kept out of `ROADMAP.md` on purpose: that file is the recorded add/add conflict
site for concurrent lanes (`docs/rungs/README.md`), the coordinator lands §9.14
serially, and lane `w-eh` is live in `docs/` at the same time. Everything below
is the section text.

---

### 9.14 W-RERANK — three corruptions of one ranking input, and two of them were one defect (2026-08-01)

Lane `w-rerank`, boards **#139**, **#110**, and §9.11's lost suffix. Instrument
work: **the census numerator does not move by one function**, and the emitted
board's *size* does not change either — 125,203 blocked emitted, 43,042 of them
clean, before and after. Only the attribution moves, and it moves 13,321 emitted
functions off keys that named a construct which was never a blocker.

#### 9.14.1 Pre-registration (written and committed before any measurement)

Committed at `f96c2d0`, the lane's first commit, before the base scan was run.

| | registered | refuted if | measured | |
|---|---|---|---|---|
| **P1** | emitted on `type-ptr` keys falls to ≤ 300 | > 1,500 remain | **13,521 → 200** | **HIT** |
| **P2** | the numerator is unchanged **to the unit** | any Δ ≠ 0 | 706,402 / 36,059, thrice | **HIT** |
| **P3** | #110 and #139 are one defect; ≥ 90 % of the `-whole{k≥2}` over-count goes | drop < 5,000 | **−3,761** | **MISS** |
| **P4** | 0–8,000 `-more` bodies become measurable | > 30,000 | wrong **direction** | **MISS** |
| **P5** | the completeness repair is total, residue named, agreement 100 % | any hole or disagreement | 2,462,571/2,462,571; 466,553 agree / **0** disagree | **HIT** |
| **P6** | the guard goes **red on the base measure** at the ptr class | it passes at base | **9 of 16,352 TYPEs**, ptr among them | **HIT** |
| **P7** | 2–6 rows move in the top 25; ≥ 1 dies; ≥ 1 appears | nothing moves | 4 die, 3 appear, **8** change rank | **HIT** |
| **P8** | the guard finds ≥ 1 **further** disagreement | it finds none | **four** further classes | **HIT** |
| **P9** | `gate.sh` PASS, 0 mismatch | any mismatch | 12/12, 2,520 verdicts, 0 | **HIT** |
| **P10** | a fixture in the refused shape is `Port=Match` | refusal or mismatch | 8/8 in class, byte-exact | **HIT** |

**8 of 10, and both misses are worth more than the hits they sit beside.**

* **P3 was registered against a stale denominator, which is the WR1 lesson in a
  new costume.** The "~27,600 `-whole{k}` over-count" comes from §6u
  (2026-07-31), *before* WR1 and W-ADJUST re-keyed the family. At this HEAD the
  entire `-whole{k≥2}` population is **15,773 bodies**, so a ≥ 20,000 drop was
  arithmetically impossible and a ≥ 90 % drop was never what #110 claimed. The
  number was copied out of a board item without re-measuring the thing it was a
  number about. §9.13's E1/E2 note says a body-column anchor is not a source of
  an emitted estimate *even when transparently discounted*; this is the same
  failure with the axis held fixed and the **date** moving.
* **P4 got the sign wrong.** `-more` did fall, by 27,595 bodies — inside the
  registered interval — but almost none of that became *measurable*. UNMEASURED
  rose by 16,927 in the same scan. Narrowing the measure to its emitter (the
  `55` annotation, the pointer-arithmetic rule) means the greedy chain now stops
  on constructs with no production, which is a **more honest** reading and the
  opposite of the one registered. A magnitude landing inside an interval for the
  wrong mechanism is not a hit.

#### 9.14.2 #139 and #110 are the same defect, and it was wrong in BOTH directions

The brief named three independent corruptions. Two of them are one line.

`mcall.rs`'s completeness walker read a call argument's operand TYPE through
`eat_int_like` — width-4 integers only — while the shipping path
(`eat_call_args` → `parse_expr` → `eat_operand_type`, which all three member-call
productions route through) has admitted 4-byte pointers there since W22. The
greedy walker charges the difference as a granted `Blocker::Type(Ptr)`, and that
one grant produces **both** published symptoms at once:

* the key prints `-then-type-ptr`, a second construct that was never a
  blocker — **#139**;
* the grant count is one too high, so `-whole{k}` over-counts — **#110**.

They were tracked as two board items and repaired by one locator.

**#139 also under-sized itself by 1.69×, and the four rows it named reconcile
exactly.** The board item says "re-attributes 7,983 emitted functions" and lists
four `recv-object` keys. Three of the four reproduce to the function here
(5,663 / 1,462 / 449) and the fourth reads 402 against 409 — a 7-function drift
between §9.13's lane tip `be797bf` and this HEAD. But the four are one family
out of **70 keys naming `type-ptr`, totalling 13,521 emitted**. The item was
sized by listing the rows someone had already looked at, which is selection on
the outcome — the same shape §8.6 records for the three control-flow keys.
Measured over the key space instead of over the shortlist, the re-attribution is
**13,321** functions.

**And the measure was not merely narrow.** The guard was run against the base
measure by mutating the four repaired positions back one at a time — the operand
gate to `eat_int_like`, the stream rules off, the `2C` arm off, the `55`
annotation back to `eat_type` — rather than against the base *source*, because
the test hook does not exist there. That is a reproduction and is labelled one;
what it reproduces is the acceptance behaviour, which is all the guard reads. It
reports **9 of 16,352 TYPEs** disagreeing, in two directions at once:

| `(tag, kind)` | class | emitter | base measure | |
|---|---|---|---|---|
| `86 43`, `86 44`, `A6 43`, `A6 44` | pointer, plain and `const` | admits | **refuses** | #139 / #110 |
| `82 12` | one-byte-unsigned | admits | **refuses** | never reported |
| `96 41`, `96 42`, `B6 41`, `B6 42` | **`volatile`** int / unsigned | **refuses** | admits | never reported |

The `volatile` row is the dangerous one and it had no board item. A measure
*wider* than its emitter manufactures phantom **completeness** — a row reads
`-whole` and the shipping path refuses it outright — and §9.13's E4 is the
record that this direction is invisible to `census/gate disagreement`, because
nothing refuses and nothing mis-emits. `volatile` at an operand position is not
a nicety either: admitting it in the *emitter* was a live wrong-bytes emit
across five shapes (W32), which is why `eat_operand_type` gates it and
`eat_int_like_or_ptr4` does not.

Three further positions were out of correspondence and are now in it:

| position | emitter | measure, before |
|---|---|---|
| the `55` call-end annotation | `eat_int_like_or_ptr4` | `eat_type` — **any** TYPE |
| pointer arithmetic in an argument | refused (`p + 1` is `addi r3,r3,4`) | admitted |
| one-byte-unsigned arithmetic / mixing | refused | admitted |
| a class-preserving `2C` conversion | admitted, emits nothing | refused |

The `55` gate alone is **2,925 of 13,500** enumerated operand streams, all in
the over-claiming direction.

#### 9.14.3 The repair reconciles to the unit, and one row is an exact identity

Blocked bodies **1,756,169 → 1,756,169** and blocked emitted **125,203 →
125,203**, both exactly. 724 distinct keys become 673. Nothing entered the
census and nothing left it; rows were renamed.

The cleanest single control is a row §6u predicted by name and by number:

```
expr-call-in-expr-recv-load-then-type-ptr-whole   2,107 -> 0
expr-call-in-expr-recv-load-whole                 6,495 -> 8,602      (= 6,495 + 2,107)
```

The row whose key said "the receiver form **and** a pointer type" loses its
second construct entirely and its bodies land on the **form-alone** key, to the
unit. §6u wrote, before any of this was built, that the repair would "merge (2)
into `recv-load-whole` and create a conflated 8,602 bucket". It is 8,602.

The `-and-type-ptr` rows behave the same way — the phantom is stripped and what
was the *third* construct becomes the second:

```
…recv-load-then-type-ptr-and-off-add-more  22,570 -> 0
…recv-load-then-off-add-more                    0 -> 22,564
…recv-object-then-type-ptr-and-call-more   19,651 -> 0
…recv-object-then-call-recv-object-more         0 -> 18,912
```

#### 9.14.4 §9.11's lost suffix: completeness is a FIELD now, not a substring

WR1 moved 39,967 functions from keys carrying `-whole`/`-more` into keys
carrying `:eof`/`:mid`. Nothing was lost and every new name is truthful, but the
two encodings live in different halves of the rendered key, so a ranking table
built by grepping `-whole` under-counts that family — and a ranking *is* such a
table. §9.13 had to re-derive the join by hand to re-check a 1,399-row figure.

`Complete` is that fact's home: a closed seven-value vocabulary, computed from
the block's own state and **never from the rendered string** (grepping the key
was the defect; a better-informed grep is the same defect), carrying its
provenance so the two producers stay separable. It is a fifth census axis beside
`cflow`, `eh`, `dispatch` and `prod`, for the reason all four of those are
axes: an orthogonal fact goes beside the key rather than into its name.

**The oracle cannot grade a correspondence**, so it is graded the three ways one
can be:

| | check | result |
|---|---|---|
| agreement | against `feature()`'s own rendering, whole enumerated key space, and on the 878-TU workload | **466,553 agree, 0 disagree** |
| totality | every body gets a reading | **2,462,571 / 2,462,571**, and the residue is the *named, printed* row `complete-none` (1,243,453) |
| injectivity | seven readings, seven distinct `complete-`prefixed names | holds; no two can be summed into a double count |

The workload row that matters: **1,289,616 rows carry no suffix at all**. Those
are exactly what a `-whole` grep silently scores as "not whole".

Reconciled against §9.11's published figures to the unit:

| | §9.11 | measured here | |
|---|---:|---:|---|
| `call-arg-multi-sym:eof` | 18,931 | **18,932** | +1 |
| the family total | 39,967 | **39,968** | +1 |

and the `+1` is not slack — it is W-ADJUST's own recorded delta
(`docs/rungs/2026-08-01-w-adjust.md` line 166, `+1 call-arg-multi-sym:eof`).

The report now prints the join both producers answer:
**83,543 blocked bodies are grammar-complete**, of which the `-whole` grep can
see 57,533.

#### 9.14.5 The re-ranked emitted board

Totals identical on both scans: bound-emitted 161,262 = 36,059 in class +
125,203 blocked; clean 43,042 (34.38 %). `clean` = `cflow-straight*` ∧ `eh-none`
∧ `calls<2`.

**Rows that DIE** (all four were in the base top 25; all four are `type-ptr`):

| row | base emitted | tip | clean |
|---|---:|---:|---:|
| `…recv-object-then-type-ptr-and-call-more` | 5,663 (rank 5) | **0** | 0 |
| `…recv-load-then-type-ptr-and-op-more` | 1,598 (rank 16) | **0** | — |
| `…recv-object-then-type-ptr-and-deref-load-more` | 1,462 (rank 18) | **0** | — |
| `…recv-load-then-type-ptr-and-off-add-more` | 1,043 (rank 24) | **0** | — |

…and below the cut, `chained-then-type-ptr-and-op-more` (586),
`recv-object-then-type-ptr-and-plumbing-more` (449),
`recv-field-off0-then-call-nested-call-and-type-ptr-more` (419),
`recv-load-then-type-ptr-and-deref-load-more` (351),
`…-and-branch-more` (316), `chained-…-and-off-add-more` (231). **13,321 emitted
functions in total leave `type-ptr` keys.**

**200 remain, and they were checked rather than waved through.** All 200 bail at
`tail-void-body-does-not-end-at-the-call`, i.e. the pointer TYPE they name is
reached at a *statement-layer* position and not at a call argument — outside the
region this repair's correspondence covers, where a `Blocker::Type(Ptr)` is a
truthful name. A residue that is merely small is not thereby explained; this one
is 200 rows of one production, and it says so.

**Rows that APPEAR:**

| row | tip emitted | rank | clean | what it actually is |
|---|---:|---:|---:|---|
| `…recv-object-then-call-recv-object-more` | 5,610 | **NEW at 5** | **0** | 100 % `calls-2plus` — a frame phase, not a rung. 1,139 distinct names |
| `…recv-object-then-deref-load-more` | 1,465 | 316 → **18** | 1 | likewise phase-gated |
| `…recv-load-then-off-add-more` | 1,038 | **NEW at 24** | **851** | 1,008 of 1,038 bail at `tail-argument-not-in-the-operand-vocabulary` — §6n **category (1)**, a private limit inside a production that already ships. 267 distinct names |

**Rows that MOVE:** eight of the top 25 change rank.
`recv-load-then-intrinsic-call` 11 → 8 (+805); `recv-load-whole` **32 → 17**
(+777); `…call-recv-load-and-deref-load-more` 13 → 11; `expr-op-0x9B` 17 → 16;
and four displaced downward by the risers (`expr-load-type-8645` 8 → 9,
`body-0x9B` 9 → 10, `expr-cmp-eq` 10 → 12, `expr-intrinsic-0xDF` 12 → 13).

Ranked by **clean ceiling** instead, two rows are new in the top 25 and one
jumps ten places:

| | clean | was | emitted | row |
|---|---:|---:|---:|---|
| 6 | 1,485 | 712 (16) | 1,506 | `expr-call-in-expr-recv-load-whole` |
| 12 | **851** | — (NEW) | 1,038 | `…recv-load-then-off-add-more` |
| 23 | 459 | — (NEW) | 560 | `…recv-load-then-type-int1-more` |

**And the biggest riser is not a rung — the `prod` axis says so.**
`recv-load-whole` reads 1,485 clean of 1,506 (98.6 %) and looks like the find of
the session. It is not: 792 of it bails at
`tail-void-body-does-not-end-at-the-call` and 711 at
`framed-result-not-consumed-by-a-literal-post-op`. It is the **statement/frame**
population — §6u's category (6) and §27.4's "not an argument question" — and
`clean` cannot see that, because `calls-1` ∧ straight ∧ `eh-none` is true of a
body that simply does not end at its call. §8.7 already says `clean` is an
optimistic ceiling and not an estimate; this is the sharpest instance yet, and
it is the reason the production axis is printed beside the joint.

**The one genuinely new candidate the corrupted input was hiding** is therefore
`…recv-load-then-off-add-more`: 1,038 emitted, 851 clean, 267 distinct mangled
names, and 97 % of it inside one shipping recognizer's argument vocabulary. Its
old name was `…-then-type-ptr-and-off-add-more`, which said the work was "a
pointer type **and** a byte-offset add". The pointer half was never there.

#### 9.14.6 The generalized guard, mechanized

> **When a census key names a construct, the measure's acceptance vocabulary
> must match the emitter's.** A measure narrower than its emitter manufactures
> phantom rungs; a measure wider than its emitter manufactures phantom
> completeness.

This is mechanically checkable and is now checked, by two portable tests that
need no toolchain:

* `a_measure_and_its_emitter_admit_the_same_types` — every `(tag, kind)` in
  `0x80..=0xFF × 0x00..=0xFF` that `read_type` parses (**16,352** TYPEs);
* `a_measure_and_its_emitter_admit_the_same_operand_streams` — the full cross of
  two operand classes × five operator shapes (none, `+`, `-`, `*`, `2C`) × the
  `55` annotation type (**16,875** streams).

Three properties make them controls rather than restatements:

1. **Both sides are driven end to end through their own entry points** over the
   same bytes — `shapes::calls::eat_call_args` for the emitter, the completeness
   walker's own argument region for the measure. A test that asserted a property
   of a *shared helper* would pass no matter how far `parse_expr` drifted from
   it, which is precisely the drift that produced #139.
2. **The domain is enumerated, not sampled.** A witness list would have missed
   the class that was wrong, because that class had witnesses on the emitter
   side and none on the measure side.
3. **They have been observed red, four times**, each on a class not yet
   repaired: 9/16,352 against the reproduced base gate; 1/16,352 for
   one-byte-unsigned; 2,925/13,500 for the `55` annotation; 1,053/13,500 for the
   stream rules; and 333/16,875 under a deliberate mutation removing the `2C`
   arm.

**The guard found more than it was built for, twice, and both are recorded
because both were nearly published as reasoning instead.**

* The first version of the shared locator returned `Int4 | Ptr4`, argued from
  the two gates' definitions: the `55` annotation is `eat_int_like_or_ptr4`, so
  a one-byte-unsigned value "would be refused one token later". That confuses
  the *formal's* declared type with the *argument expression's* type — `f(int)`
  called with a `bool` annotates `55 86 41 74` over an `82 12` operand, and the
  emitter takes it. The enumerated guard found the single excluded pair on its
  first run. **A correspondence argued from definitions is a claim; a
  correspondence enumerated over the domain is a measurement.**
* The `2C` conversion was sized at **0** and documented as a deliberate, bounded
  omission — measured on the *base* tree. Both halves were wrong: the key is
  spelled `…-then-convert`, and repairing the operand TYPE let the walk reach
  past the pointer it used to stop at, so the row went **829 → 13,325 bodies and
  26 → 1,144 emitted in one scan**. **A residue sized before the repair that
  exposes it is not sized.**

**What the guard does not cover, stated with its reason.** It is scoped to the
call-argument region, where the correspondence is exact and the emitter is
`eat_call_args`. `Vocab::IntrinsicRecv` is deliberately excluded and kept at the
old int-only vocabulary: nothing in the intrinsic family is lowered at all, so
there is no emitter to correspond to, and widening it to match a production that
does not exist would be a claim. That is the same honesty gate `form_is_measured`
applies, and naming the position in an enum is what makes the exclusion visible
instead of accidental.

#### 9.14.7 The repair's own fix reintroduced the disease, and a fixture caught it

The pointer-arithmetic refusal filed as **`…-then-op-0x55`** — the byte the
operand run stopped *in front of*, not the construct — which is exactly what
#139 exists to cure. `Fail::note` gives ties to the first note and the same loop
records a `FailKind::Value` at that offset one line earlier, so the construct
name was silently discarded. Named now (`…-then-ptr-arith`,
`…-then-int1u-misuse`) via `note_forcing`, because a stream refusal is a
property of the whole run rather than of one byte.

It was caught by `fixtures/cpp/wrr_arg_vocab_neg.cpp`, which exists because the
repair's *premise* is gradable even though the repair is not. `mark_whole` is
diagnostic — its `Err` stays an `Err` — so no byte moves and
`census/gate disagreement` is structurally blind to the whole change. §9.13's E4
is the record of registering a control that cannot see the failure mode. The
control that works is the differential:

* `wrr_arg_vocab.cpp` — **8/8 in class, `Port=Match`**, byte-exact: a pointer
  argument, a cv-qualified pointer, a class-preserving `int*`→`void*` convert, a
  pointer beside an int in one operand run, member and free-function callers.
* `wrr_arg_vocab_neg.cpp` — **0/5 in class, `Port=NotImplemented`**: pointer
  arithmetic, a `double` argument, a `long long` argument, a cross-class
  reinterpret. Without the negative half the positive cannot tell a correct
  vocabulary from one that admits everything — and "admits everything" is the
  direction that now propagates into the census.

Both are in `fixtures/cpp/`, so both are in **every** gate lane. That is the
distinction §9.13's brief draws: `differential.rs` grades a fixed list of three
fixtures and adding a fixture does not put it in that lane; `scripts/gate.sh`
walks the corpus, and the verdict count went **2,496 → 2,520**, which is
exactly 2 fixtures × 12 lanes.

#### 9.14.8 An environment hazard that cost this lane an hour, and reads as an ALARM

The first base scan reported **6 mismatch, 0 match** on untouched `master`, on
the six TUs §9.13 published as matching — with the census reproducing §9.13
exactly to the function. It is not a regression. **The capture cache is not
portable across worktrees by PATH LENGTH.**

The reference obj embeds its own output path, and the cache captures into the
entry's directory. Reaching the shared cache through a worktree symlink
(`…/.claude/worktrees/<name>/work/capture-cache`, 90 chars) serves objs that
were captured under the main repo's path (48 chars), so the port's obj is 42
bytes longer and the compare diverges at COFF offset 8 — `PointerToSymbolTable`
— by one section header's worth of shift. Addressing the same bytes by their
literal path restores 6/6 match.

Two things follow, and the second is the one worth keeping:

1. `scripts/configure_existing_worktree.sh` links `compilers/` and copies
   `work/dc3-workload/` and does **not** touch the capture cache. That is
   correct, and the reason should be in the script: a lane that "helpfully"
   symlinks it gets six phantom mismatches.
2. **`--validate-cache N` cannot see this.** It re-captures *in place*, in the
   long-path directory, then compares and self-heals — so it reports
   "6 re-captured and agreed, 0 POISONED" and returns the fresh obj, turning a
   loud failure into a silent pass. An instrument that repairs the condition it
   is supposed to detect reports the absence of the thing it just removed.

Use `C2RS_GAP_CACHE=<main-repo>/work/capture-cache` verbatim from a worktree.

#### 9.14.9 Gate evidence

At `15ed8aa`, worktree configured against the shared toolchain, cache addressed
by its canonical path.

* `cargo test --workspace` — base `99ed418` **584 passed, 0 failed, 1 ignored**
  → tip **589 passed, 0 failed, 1 ignored**. Both measured, not inferred: the
  base was rebuilt from `git checkout 99ed418 -- crates` and re-run.
  **`#[test]` grep over `crates/` 585 at base → 590 at tip.** Grep and runner
  agree at both ends once the one `#[ignore]`d test is added to the runner's
  passed count (585 = 584 + 1; 590 = 589 + 1), so no grep line is prose or a
  doc-comment here — the whole-tree grep, which *is* polluted by `docs/`, reads
  594 at base and is not the number quoted.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP
  / 0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip** (208 at §9.13 plus this lane's
  two fixtures).
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — 42,719 configurations × 12 lanes =
  **512,628 gradings, 512,628 graded, 0 mismatches**; 406 of 406 declared family
  pairs reached *and* emitted; refusal-frontier residue **0**. Run because this
  lane touched decode, even though every line of it is on the census's
  diagnostic path.
* 878-TU workload scan — **6 match, 0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies **706,402 / 2,462,571 (28.69 %)**; emitted **36,059 / 178,968
  (20.15 %)**; census/gate disagreement **0**. Identical to base on all of them.
* Fixtures — `wrr_arg_vocab.cpp` 8/8 in class and `Port=Match`;
  `wrr_arg_vocab_neg.cpp` 0/5 and `Port=NotImplemented`.

#### 9.14.10 Board items

* **#139 — CLOSED.** With **#110**, which was the same defect.
* **#143 — `…recv-load-then-off-add-more`, 1,038 emitted / 851 clean.** The one
  new candidate this re-rank exposed. 1,008 of 1,038 bail at
  `tail-argument-not-in-the-operand-vocabulary` — category (1), a private limit
  in a shipping recognizer — across 267 distinct mangled names. Size it off its
  own counterfactual: §9.13 measured two arms of one family converting **19×
  apart**, so no rate transfers.
* **#144 — the `volatile` operand class was admitted by the measure and refused
  by the emitter, and had no board item.** Repaired here. The general form is
  the guard in §9.14.6; the specific worry is that `eat_operand_type`'s
  `volatile` gate has **one** call site by design (W32), and any second reader
  of an operand position is a candidate for the same divergence.
* **#145 — `scripts/configure_existing_worktree.sh` should say why it does not
  link the capture cache**, and `--validate-cache` should report a path-length
  mismatch instead of self-healing it (§9.14.8).
* **#146 — extend the correspondence guard beyond the call-argument region.**
  The pattern generalizes to every place a census measure shadows a shipping
  production; the argument region is simply the one #139 was about. Each new
  pair costs one enumerated test and, on this evidence, finds something.

# 9.16 W-TU — #122 never moved the metric, and the metric was measuring the wrong population (2026-08-01)

Lane `w-tu`, board **#122**. Measurement and instruments; **no codegen change,
no TU converted, and none was convertible.** Base and tip `1f3e00e` +
this lane's two commits.

Headline: **TU match 6 → 6.** Three findings, in descending order of how much
they change what to do next:

1. **#122's "6 → up to 15" was the item's own ceiling restated as an outcome.**
   The string has never existed in this repository.
2. **The leading indicator counts the wrong population, and a third constraint
   binds harder than either.** The port emits one `.text` COMDAT per `.ex`
   function segment and has **no emit-set model**, so **only 25 of 871 graded
   TUs can ever be byte-exact** at any codegen quality. Six already are. TU
   match is ceilinged at **25/878** until Phase 7 exists, against a terminal
   target of 871.
3. **There is no one-away lever, and the ten are not ten different things —
   they are one thing.** **17 of the 19** reachable near-match TUs block on
   control flow. Exactly **one** (`xboxheap.cpp`) is free of both control flow
   and EH, and it is **three** independent refusals away, not the two the board
   records.

---

## 9.16.1 #122 — the number never moved, and the "15" is arithmetic, not measurement

**Verdict: the projection branch.** The nine were never converted; nothing
regressed; the TUs did not convert. The completion record carried the board
item's *ceiling* into the past tense.

The evidence, each piece of which could have come out the other way:

* **Master's own merge commit says so in its subject line.** `6b07500`,
  *"Merge: WLR — nine TUs one function from byte-exact, and none of them was a
  rung"*, opens its body with **`TU match 6 -> 6`** and continues: *"The
  pre-registered estimate, committed before any code and stated in TUs, was 0
  conversions of 9. Actual 0."* The lane that owned the item reported the miss
  correctly and in the right unit. The board did not read it.
* **The string `15/878` has never existed in this repository.** `git log --all`
  with `-S"15/878"`, `-S"15 of 878"`, `-G"TU match 1[0-9]"` and
  `-G"match +(7|8|9|1[0-5]) +[0-9]" -- docs` all return **zero commits**, over
  the whole DAG — every branch, merged or not, so a lane that moved the number
  and never merged would still have shown. A grep of all 15,849 lines of commit
  message across all refs for a TU match other than 6 returns nothing.
* **Every recorded statement of the metric says 6.** Ten distinct sentences
  across `ROADMAP.md`, `GAPS.md` and the rung docs; the values are
  `6`, `6/878`, `6 → 6`, *"6 before and 6 after"*, *"flat at 6/878"*.
* **The scan reproduces 6 at `1f3e00e` today**, warm cache, 871 hits.

**Where "15" comes from.** It is `6 + 9`: the current match count plus the size
of the bucket the item was scoped to. `GAPS.md` §8.7 closes with *"Nine of them
are one blocked emitted function away from a whole byte-exact TU, which is the
cheapest thing on the board that moves the payoff metric."* That sentence is the
item. "Up to 15" is its ceiling, and a ceiling is what you write **before** the
work, not after. (It is *not* the other 15 in that section — §8.7's published
`≤0` bucket is also 15, but that is a distance bucket including 14 TUs with no
functions, and nobody would phrase it "→ up to 15/878".)

**Why this is the most dangerous artifact on the board, stated plainly.** §9.9.2
and §9.13 record controls that passed while measuring the wrong thing. This is
worse than that class, because there was no measurement at all: a projection was
promoted to a result by the act of closing the item. The specific mechanism is
that **the board's payoff field and its outcome field were the same field**, so
"what this would buy" and "what this bought" are indistinguishable once the
status flips. Anything that records an estimate and a status in one place has
this defect.

The remedy is the one this project already uses everywhere else and did not
apply to its own board: **pre-register the estimate in its own artifact, score it
separately, and never let the estimate be the record of the outcome.** The lane
did exactly that (`GAPS.md` §9.1, estimate 0 of 9, actual 0, scored exact) and
the board overwrote it with its own guess.

## 9.16.2 The leading indicator counts `.ex` bodies; the goal is written in emitted functions

`gap.rs::near_match_tus` measures distance as `fn_total - fn_in_class` — blocked
**IL bodies**. A byte-exact TU is a claim about its **`.text` COMDATs**. §8.1
already established these are wildly different populations (2,462,571 against
178,968); nothing had crossed them per TU. Both distances now print on every
scan:

| bucket | blocked **bodies** (published) | blocked **emitted** (new) |
|---|---:|---:|
| ≤ 0 | 1 | **2** |
| ≤ 1 | 10 | **19** |
| ≤ 10 | 25 | **82** |
| ≤ 100 | 32 | **399** |
| ≤ 1000 | 210 | **857** |

The two disagree by 12× at ≤100 and they **rank differently**, which is the part
that matters for steering: `src/system/math/Rand2.cpp` is 8 blocked bodies but
**2** blocked emitted functions; `src/system/net/JsonMemory.cpp` is 7 and **3**;
`src/system/math/vec.cpp` is **565** blocked bodies and **zero** blocked emitted.
Ranking by bodies puts real work and bookkeeping in the same bucket.

**And a correction to the published band that costs one TU.** `≤1: 10` is
**cumulative**, and its first member is `src/system/utl/Spew.cpp` at distance
**0**, which already matches. The bucket holds **nine** one-away TUs and one
already-converted one. Every brief that has said "ten TUs are one function from
byte-exact" has been counting a TU that is zero functions from it.

## 9.16.3 The emit-set ceiling — 25 of 871, and it is the binding constraint

Neither distance is distance-to-match, because a third condition binds before
either:

> `PortC2::build` takes `il.functions()` — **one entry per `.ex` function
> segment** — and under `/Gy` pushes exactly one `.text` COMDAT per entry
> (`crates/c2-core/src/lib.rs:192` and the `fn_level_linking` loop).
> **There is no emit-set model anywhere in the port.**

So when a TU's `.ex` segment count differs from its reference obj's `.text`
COMDAT-leader count, the port writes the wrong number of sections and the obj
diverges however good the codegen is. `emit-emitted` is that leader count and
`fn_total` is that segment count, so the predicate is a comparison of two
numbers every scan already had:

| | TUs |
|---|---:|
| `.ex` segments **==** obj `.text` COMDATs — reachable in principle | **25** |
| `.ex` segments **>** COMDATs — port would emit **spurious** COMDATs | **842** |
| `.ex` segments **<** COMDATs — port would **miss** COMDATs | **4** |

**TU match cannot exceed 25/878 before Phase 7**, and six of the 25 are the
current matches. The terminal target is 871. So §8.3's Phase 7 is not the last
phase in the plan — it gates **846 of the 871**, and no amount of Phase 1–6
widening touches them.

`src/system/math/vec.cpp` is the clean demonstration and it is live: **zero**
blocked emitted functions, both emitted functions in class, and it is
`vocab-gap` — 802 `.ex` bodies against 2 emitted COMDATs.
`src/system/synth_xbox/MeterEffect.cpp` fails in the other direction: 10 bodies
against **13** COMDATs, so three of c2's emitted functions have no IL body at all
and no widening can produce them.

### The control, because a ceiling asserted is not a ceiling measured

The reading is that `fn_total` counts `.ex` segments and `emit-emitted` counts
`.text` COMDAT leaders. If that is wrong the ceiling is void. The invariant that
can go red: **no `match` TU may violate it** — a byte-exact obj cannot carry a
different number of `.text` COMDATs than the port wrote.

* On the workload: **0 violations**, printed on every scan beside the ceiling.
* The base rate makes it a real test rather than a tautology: agreement holds for
  25 of 871 = **2.9 %**, so six matching TUs all agreeing by accident is ~10⁻⁹.
* The unit test does not stop at "it is zero". Per **#145** — a validator that
  cannot see the defect it exists for is worse than none — it mutates a
  **matching** TU into a violation (5 segments, 2 COMDATs) and requires the count
  to go to 1, plus asserts the mutation did not change the `match` count, so the
  control tests the emit-set reading and not the class filter.

## 9.16.4 The near-match band, per TU, by the byte

All 25 in the ≤10-by-bodies band, censused one capture each at the workload's own
`/O1 /Oi /EHsc`. `reach` = the emit-set condition of §9.16.3.

| dist | emitd | reach | TU | the blocked function(s) | key | cflow / EH | what must actually fall |
|---:|---:|:--:|---|---|---|---|---|
| 0 | 0 | ✅ | `system/utl/Spew.cpp` | — | — | straight | **matches** |
| 1 | 1 | ✅ | `Main.cpp` | `?Run@App@@QAAXXZ` 222 B | `param-width-undetermined:mid` | straight / **eh-state1** | **Phase 5** — the whole EH record |
| 1 | 1 | ✅ | `system/math/Primes.cpp` | (unnamed) 294 B | `expr-jump` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `system/math/Sort.cpp` | `?HashString@@YAHPBDH@Z` 261 B | `assign-store-type-0x86` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/osfinfo.cpp` | (unnamed) 445 B | `expr-cmp-ge` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/undname.cpp` | (unnamed) 532 B | `expr-cmp-ne` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/LIBCMT/vswprnc.cpp` | (unnamed) 508 B | `expr-cmp-eq` | **if-n** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/nuispeech/xboxheap.cpp` | `CXboxHeap::CXboxHeap` 404 B | `expr-op-0x27` | straight / none | **3 refusals**, one of them Phase 4 — §9.16.5 |
| 1 | 1 | ✅ | `xdk/xjson/jsonwriter.cpp` | `?GetBuffer@JsonWriter@@QAAJPAGPAK@Z` 1349 B | `expr-brfalse` | **loop** | **Phase 6** |
| 1 | 1 | ✅ | `xdk/xlrc/xlrcimpl.cpp` | `?CreateClient@CXLrcImpl@@…` 519 B | `assign-rhs-call-0x26` | **if-n** | **Phase 6** |
| 2 | 1 | ❌ | `ChecksumData_xbox.cpp` | 192 B + the **data object** `?gFileChecksums@@3PAUFileChecksum@@A` 152 B | `expr-op-0x27`, `data-sym-not-extern:eof` | straight | **Phase 7** (3 bodies / 1 COMDAT) |
| 2 | 2 | ✅ | `system/negate_test.cpp` | 2 × (unnamed) 388/396 B | `assign-store-type-0x86` ×2 | **if-n** ×2 | **Phase 6** |
| 2 | 2 | ✅ | `system/synth_xbox/Biquad.cpp` | 838 B, 162 B | `expr-cmp-eq`, `…recv-load-then-plumbing-0x3A` | **cf-expr-0x05 / eh-unknown**, straight | **Phase 6** + a member-call production |
| 2 | 2 | ✅ | `xdk/LIBCMT/vsnprnc.cpp` | 536 B, 181 B | `expr-cmp-eq`, `call-arg-lit-permuted:mid` | **if-n**, straight | **Phase 6** + arg permutation |
| 3 | 3 | ✅ | `system/rndobj/wordwrap.cpp` | 97 / 502 / 2661 B | `expr-jump`, `expr-bit-and`, `expr-cmp-eq` | straight, **if-n**, **cf-expr-0x05** | **Phase 6** |
| 3 | 3 | ✅ | `system/utl/Pool.cpp` | 431 / 234 / 230 B | `expr-op-0x27` ×2, `expr-brtrue` | **cf-expr-0x05**, **if-1** ×2 | **Phase 6** |
| 3 | 1 | ❌ | `xdk/nuiapi/nuidetroit.cpp` | 155 / 187 / 874 B | `expr-ptr-arith:mid`, `param-multi-reg:mid`, a member-call chain | straight ×2, **cf-expr-0x08** | **Phase 7** (3 bodies / 1 COMDAT) |
| 3 | 3 | ✅ | `xdk/nuispeech/mmio.cpp` | 286 / 419 / 512 B (8 of 11 already in class) | `expr-cmp-eq` ×3 | **if-2**, **if-n** ×2 | **Phase 6** |
| 4 | 4 | ✅ | `system/synth_xbox/IPP_basicmath_xbox.cpp` | `?Add_InPlace@IPP@@…`, `?MulConstant_InPlace@…`, `?Mul_InPlace@…`, `?Mul@IPP@@…` | `expr-cmp-eq` ×4 | **loop** ×4 | **Phase 6** |
| 4 | 4 | ✅ | `system/utl/EncryptXTEA.cpp` | 191 / 244 / 492 / 478 B | `expr-intrinsic-memcpy`, `expr-op-0x27` ×2, `expr-load-type-8882` | straight ×2, **loop** ×2 | **Phase 6** (2 of 4) |
| 4 | 4 | ✅ | `xdk/nuispeech/xboxmem.cpp` | `?GetXAllocAttributes@…`, `?MemAlloc@…`, `?MemFree@…`, `?MemSize@…` | `expr-cmp-ne`, `expr-cmp-eq` ×3 | straight, **if-1** ×3 | **Phase 6** (3 of 4) |
| 7 | 3 | ❌ | `system/net/JsonMemory.cpp` | 7 bodies, **all `cflow-straight`, all `eh-none`** | `expr-op-0x27`, `call-ref-cflow-jump` ×3, `call-arg-multi-sym:mid`, `call-bound-store-0x86` ×2 | straight ×7 | **Phase 7** (11 bodies / 3 COMDATs) |
| 8 | 2 | ❌ | `system/math/Rand2.cpp` | 8 bodies | `expr-op-0x27` ×3, `call-ref-cflow-jump` ×4, … | straight ×6, **cf-expr** ×2 | **Phase 7** (13 bodies / 2 COMDATs) |
| 8 | 6 | ❌ | `system/oggvorbis/VorbisMem.cpp` | 8 bodies, one **eh-state1** | `call-ref-cflow-jump` ×3, `expr-op-0x27`, … | straight | **Phase 7** (12 bodies / 7 COMDATs) |
| 8 | 12 | ❌ | `system/synth_xbox/MeterEffect.cpp` | 8 bodies | `expr-intrinsic-this-adjust` ×2, `expr-op-0x27`, … | **loop** ×2, **if-1** ×3, **if-2** | **Phase 7** (10 bodies / **13** COMDATs) |
| 18 | — | ✅ | `keygen_xbox.cpp` (20th reachable, outside ≤10) | 18 of 20 | `expr-jump` ×8, `assign-store-type-0x86` ×2, … | **loop** ×11, **if** ×2 | **Phase 6** |

**Read the `reach` column first.** Six of the 25 in the published band —
`ChecksumData_xbox`, `nuidetroit`, `JsonMemory`, `Rand2`, `VorbisMem`,
`MeterEffect` — can never be byte-exact by widening. They are in the near-match
band because they have few blocked *bodies*, and they are unreachable because
their body count is not their COMDAT count. The sting is `JsonMemory.cpp`: it is
the **only** TU in the 4–10 band whose every blocked body is `cflow-straight` and
`eh-none` — the one clean widening target in the band — and the emit set puts it
out of reach anyway.

## 9.16.5 The key names, taken to the byte — and `expr-op-0x27` is the second reader's stop

The brief's warning held: **the key name names the blocker in 0 of the 9.**

* For **seven**, the body is `cflow-if-n` or `cflow-loop`. `c2-il::func::census`'s
  `every_in_class_row_is_a_single_basic_block` asserts every in-class row is
  `cflow-straight`, and it holds over all readable in-class bodies on the
  workload. The named `cmp`/`jump`/`brfalse` is a real construct in a real body
  and removing it converts nothing — §9.3's refutation, re-confirmed at this HEAD.
* For **`Main.cpp`**, `param-width-undetermined:mid` is a distraction; the body is
  the only one of the nine with `maxState ≥ 1`.
* For **`xboxheap.cpp`**, the key does not merely name the wrong construct — **its
  byte pointer is provably not the cause**, and that was shown rather than argued.

### The `xboxheap` ladder, rebuilt at this HEAD

§9.4's probe ladder, reconstructed from the real source
(`mSize = size; mFreeHead = this; mCount = 0; mUsedHead = this; auto& listHead =
mListHead; …; AllocatePageBlock(initSize);`) with each refusal isolated:

```
  L1  mSize=size; mFreeHead=this; mUsedHead=this;              1/1 in class  store-run
  L2  …the same run plus `mCount = 0;`                         0/1  expr-op-0x27
  L3  mSize=size; BLOCK& lh=mListHead; lh.mNext=&lh; …         0/1  expr-op-0x27
  L4  mSize=size; AllocatePageBlock(initSize);                 0/1  expr-op-0x27
```

Three structurally unrelated constructs, **one key**. Taken to the byte, with the
`.ex` segments dumped (`census --keep-il`) and compared:

```
  blocking byte reported for L2, L3, L4:   segment offset 96, all three
  segment bytes 88..104, ALL FOUR probes:  43 81 20 33 86 41 74 00 >27< a6 43 f5 08 b9 fd 09
  first byte at which each differs from L1 (the IN-CLASS control):
      L2 vs L1   offset 159   — AFTER the reported blocking byte
      L3 vs L1   offset  54   — BEFORE it
      L4 vs L1   offset 112   — AFTER it
```

**`L1` is in class and contains the identical byte at the identical offset behind
an identical 96-byte prefix.** So byte 96 is admissible, and the bracket the
census prints as *"the byte that blocked the parse"* is pointing at a byte that
demonstrably does not block. What `expr-op-0x27` records here is **where the
second reader stopped after the first reader declined** — §9.13's `this-adjust`
pathology exactly, now on the **largest row on the board**: `expr-op-0x27` is
407,016 bodies (23.2 %) and 22,759 emitted (18.2 % of blocked emitted), and the
roadmap has measured it three times at 0.14–2.5 % completion without a mechanism
for why. This is the mechanism. It is not a construct; it is a fall-through.

**Consequence: every "go to the byte" investigation that started from an
`expr-op-0x27` window has been reading the wrong bytes**, and the row's ranking
in both censuses is a ranking of a residue, not of a rung. Fixing the census to
report the *first reader's* refusal reason for these bodies is the highest-value
instrument job on the board and it is not this lane's seam.

### And `xboxheap` is still THREE refusals away, not two

`GAPS.md` §9.4 lists three and marks refusal (1) — *"a literal value in a store
run of more than one (`mCount = 0;` among seven formal/`this` stores)"* —
**"Taken — see §9.5"**. It was not taken. WLR admits a run in which **every**
statement stores the *same* literal; `xboxheap`'s run stores formals **and** one
literal, and WLR's own doc refuses that case explicitly (*"The mixed
literal/formal run is refused for the same reason and a second one"*). The two
statements contradict each other inside one document set.

Measured, with a control that could have failed:

```
  A  h->mSize=9;  h->mCount=9;  h->mX=9;          ok   store-run      (WLR's own shape)
  B  h->mSize=size; h->mCount=0; h->mX=x;         GAP  expr-op-0x27   (xboxheap's shape)
  C  h->mSize=size; h->mCount=0;                  GAP  expr-op-0x27   (the minimum of B)
  D  h->mSize=size; h->mCount=n;                  ok   store-run      (CONTROL: no literal)
```

**D is the control and it passed**: strip the literal and the same two stores are
in class, so the literal is what refuses and the reading is not an artifact of
the formals. Had D been refused, the whole attribution would have been wrong.

So `xboxheap` needs (1) the **mixed** literal/formal store run — which WLR
measured and declined on evidence, because at length 2 c2 returns the stores in
the *opposite* order to the source and the literal's position across lengths 2–6
is a two-queue schedule with a ready-time; (2) the interior sub-object reference
bind; (3) a framed member call on `this` with an argument, which is §8.3 **Phase
4**, whose governing rules are recorded as fitted hypotheses with no mechanism.
**The one TU in the band that is neither control flow nor EH still contains a
Phase-4 item.**

## 9.16.6 Is there a one-away lever? No — and the ten are not ten different things

The brief's fourth question offered two answers and the measurement gives a
third, which is worse than either.

* **No single rung converts even one TU**, let alone two. The 25 reachable TUs
  are 6 already matching and **19 not**. Of those 19: **17 block on control
  flow** (`cflow-if-*`, `cflow-loop` or an undecoded `cf-expr-*` in at least one
  blocked body), **1** on the whole EH record (`Main.cpp`), and **1** on three
  refusals including a Phase-4 item (`xboxheap.cpp`). That is the complete
  partition — every reachable TU is in exactly one of the three.
* So it is **not** "ten TUs each needing a different thing". It is **one thing,
  needed by seventeen of them**: Phase 6. The distance metric was not misleading us
  about diversity — it was hiding a *concentration*, which is the opposite error
  and points the other way.
* §8.3 currently has Phase 6 **demand-gated and last but one**, on the
  counterfactual that has said "718 functions, five scans running". That
  counterfactual is measured in **functions**. Measured in **TUs at the near
  edge**, control flow is the single largest item on the board: it is the sole
  blocker of 17 of the 19 TUs that can be reached at all. **Both readings are
  correct and they rank Phase 6 at opposite ends**, because one counts body mass
  and the other counts payoff. §8.2 says the payoff metric is TU match.
* The cheapest honest re-plan is therefore: **Phase 7 (emit-set) and Phase 6
  (control flow) are the whole remaining program for TU match, in that order** —
  Phase 7 because it gates 846 of 871 and nothing else touches them, Phase 6
  because it gates 17 of the 19 that Phase 7 does not. Phases 1–4 as currently
  ranked convert **zero** TUs at the near edge; they are census work.

## 9.16.7 Pre-registration, scored

Registered in `docs/rungs/_2026-08-01-w-tu-prereg.md`, committed at `3db930a`
before any per-TU measurement. Declared bias: pessimistic and *borrowed* (§9 was
read first), with E5/E6/E8 flagged as the ones that could go wrong. They are
exactly the ones that did.

| # | claim | est | interval | actual | score |
|---|---|---|---|---|---|
| E1 | #122 is the **projection** branch of the four | projection | one of four | projection | **HIT** |
| E2 | commits recording a TU match ≠ 6, all branches | 0 | [0, 2] | **0** | **HIT** |
| E3 | of the 9, converted by removing only the named first blocker | 0 | [0, 1] | **0** | **HIT** |
| E4 | most of the 25 any **single** change converts | 1 | [1, 3] | **0** | **MISS** — below the floor; not even one |
| E5 | of the 7 at distance 4–10, all-straight and `maxState == 0` | 2 | [0, 5] | **1** (`JsonMemory`) | **HIT** (inside, 1 off the point) |
| E6 | of the 9, key names that do **not** name the real blocker | 2 | [0, 5] | **9 of 9** | **MISS**, badly — above the ceiling |
| E7 | TUs converted by this lane | 0 | [0, 1] | **0** | **HIT** |
| E8 | `xboxheap` refusals remaining after WLR | 2 | [2, 3] | **3** | point **MISS**, inside the interval |

**5 of 8 on the point, and the three misses carry the lane's value.**

* **E6 is the important miss.** I registered 2 of 9 key names as misleading and
  the answer is **9 of 9** — I under-predicted a failure mode I had been
  explicitly warned about three times in the brief, because I was implicitly
  treating "the key names a real construct in the body" as "the key names the
  blocker". For the seven control-flow TUs both are true of the construct and
  false of the blocker, and I had already written down the reason (the
  single-basic-block invariant) before making the estimate. The prediction was
  refuted by evidence I already possessed.
* **E4 low is the useful direction.** I allowed that *something* might convert
  one TU. Nothing does. That is what makes §9.16.6 a re-plan rather than a
  ranking tweak.
* **E8** was the borrowed-prior failure: I took `GAPS.md` §9.4's "Taken" at its
  word for one number instead of re-running the probe, which is §9.14's
  *"a board item's quantity ages"* applied to a refusal count rather than a
  denominator.

## 9.16.8 The absence-read-as-success instance this lane produced, and caught

A `cargo test --workspace --release` run launched while an incremental rebuild
was in flight reported **`ok` for every target, 0 failed** — and produced **20**
result lines summing to **422 passed**, against the true **24** lines and **591**.
**169 tests did not run and the run reported success.** Nothing in the output
said "fewer targets than usual"; every line that existed was green.

It was caught only because the base-vs-tip comparison the brief mandates put 589
next to 422 and the difference was not +2. **A tip-only reading would have gone
into this document as the tip total.** Thirteenth recorded instance, and the
first where the *test runner itself* was the instrument that read absence as
success. The mitigation is the one already in use for gate lanes — compare a
count, never a status — and it should extend to test totals: **record the number
of test targets, not just the number of tests**, because a lost target is
invisible in the sum.

## 9.16.9 Gate evidence

| lane | base `1f3e00e` | tip |
|---|---|---|
| `cargo test --workspace --release` | **589 passed, 0 failed, 1 ignored, 24 targets** | **591 passed, 0 failed, 1 ignored, 24 targets** |
| `#[test]` grep over `crates/` | **590** | **592** (+2, both new) |
| `scripts/gate.sh --jobs 4` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **2,520 fixture-verdicts** |
| `c2rs selftest` | — | **210 PASS, 0 FAIL** |
| 878-TU workload scan | match **6**, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 | **identical** |
| census | 706,402 / 2,462,571 (28.69 %) | **identical** |
| emitted census | 36,059 / 178,968 (20.15 %) | **identical** |
| census/gate disagreement | 0 | **0** |
| distance (bodies) | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 | **identical** |
| distance (**emitted**) | not measured | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 399, ≤1000: 857 |
| emit-set ceiling | not measured | **25 of 871**, violations among matching TUs **0** |

`cross_sweep` not run: **no codegen was touched.** The diff is `gap.rs`
(three read-only report methods and two tests), the report block in `main.rs`,
and three scripts. `PortC2`, `codegen` and every recognizer are untouched, so
there is no lowering whose cross product could have moved.

**Caveat on the environment, stated because it is printed on every scan and
should not be silently inherited:** this box's `wibo` is `1.0.1-7`, older than
the known-good `1.0.1-23`. The scan warns that this makes the *replay* column a
fake divergence alarm while *census and mismatch counts stay byte-identical*.
This lane reports no replay number and used `--replay-every 0`, so nothing above
depends on it — but the next lane to quote a replay figure from this machine
must upgrade first.

## 9.16.10 Found and not taken, ranked

1. **The `expr-op-0x27` attribution** (§9.16.5) — the board's #1 row by both
   censuses is a second-reader fall-through, not a construct. Making the census
   report the *first* reader's refusal for these bodies would re-rank the top of
   the widening order. Largest instrument job on the board; not this lane's seam.
2. **Phase 7, sized for the first time**: 842 TUs where the port would emit
   spurious COMDATs, 4 where it would miss them. The 4 are the cheaper end and
   include two license TUs (`TomCryptLicense`, `ZlibLicense`) with **zero** `.ex`
   bodies and **one** emitted COMDAT each — the smallest possible instance of the
   emit-set problem, and a much better first probe than anything with 802 bodies.
3. **The census names the callee, not the function, for any body containing a
   call** (`GAPS.md` §9.6, unfixed). Confirmed again here: probe `L4`, whose only
   function is a constructor, is reported as
   `?AllocatePageBlock@L4@@QAAPAXI@Z`. Every function name in the near-match
   table for a call-bearing body is wrong.
4. **`GAPS.md` §9.4's "Taken — see §9.5" on refusal (1) is incorrect** and should
   be corrected in place when §9 is next edited, together with the refusal count
   for `xboxheap` (3, not 2).
5. **The `.ex` carries data objects.** `ChecksumData_xbox.cpp`'s third census row
   is `?gFileChecksums@@3PAUFileChecksum@@A`, 152 B, keyed
   `data-sym-not-extern:eof` — a *data* symbol occupying a function segment. That
   is worth a line in the emit-set model, because it means `fn_total` is not
   purely a function count and the ceiling of §9.16.3 may be slightly
   conservative in TUs of this shape.

DRAFT for `docs/ROADMAP.md` §9.17 — written by lane `w-arms`, to be landed by the
coordinator. Nothing in §1–§9.16 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-arms-prereg.md`, committed at `2db819c` before the
first scan.

---

### 9.17 W-ARMS — the largest site on the emitted board is 8 % assignments, and the biggest blocker on it is worth 6 (2026-08-01)

Boards **#142** (the clean-not-whole receiver arms) and **#143**
(`…recv-load-then-off-add-more`). **No rung.** Both were measured and both
declined, and the two measurements are worth more than a rung of this size would
have been:

* **#142 is decomposed for the first time**, into 27 named receiver constructs
  in emitted-function units — the site was one undifferentiated bucket *by
  construction*, because all three member-call productions threw away the
  `Block` that says what stopped them.
* **#143's `-more` is an arity artefact.** Every one of its 1,038 emitted
  functions is discounted by a suffix that means "the same construct twice", not
  "a second construct".
* **`expr-op-0x27` — the #1 blocking feature on the emitted board, 22,759
  emitted and 407,016 bodies — converts 6 emitted functions** when its named
  token is granted.

---

#### 9.17.1 The site had no instrument, and that was a property of the source

§9.13 sized the three receiver-designator `prod` sites at 37,060 blocked
emitted, "the largest single site on the emitted board, larger than any census
key", and could say nothing about what is *in* them. That is not an oversight of
the analysis. `mcall_{tail,chain,cmp}` each call `eat_receiver_this`, which
returns an `Err(Block)` carrying the refusal context **and the byte**, and each
maps it to a flat `prod_tag("…-recv-not-a-plain-b9-load")` — discarding both. No
scan of any existing axis could have decomposed the site.

The tag is refined in place, keeping the published site name as a **prefix**
(`<old site>/<construct>`), so every figure keyed on the old string is recovered
by a prefix test. It costs nothing in the harness: `prod` is already a census
axis and already a row-dump column.

**It names the construct, never the position** — [`mcall::Fail::blocker`]'s own
rule, and the same vocabulary (`off-add`, `deref-load`, `store`, `plain-call`,
`virtual`, `temp-bind`, `convert`, `ternary`, `call-in-expr`), so the two axes
cross without a translation table. **Five** portable tests, no toolchain — the
five that take `#[test]` over `crates/` from 590 to 595:

* `every_receiver_refusal_has_a_name` — the domain **enumerated**, 8 contexts ×
  256 bytes × EOF × 3 arms, with the residue asserted **inside** the loop so a
  failure names its witness rather than reporting a count;
* `the_receiver_vocabulary_is_injective` — the designator and bind positions are
  disjoint sets of names, so no two rows of the decomposition can be summed into
  a double count;
* `the_intrinsic_receiver_arm_separates_by_selector` — the first **arity** axis:
  2113…2119 must give seven distinct names, and the test's own encoder is checked
  against the captured `80 41 08 00 00` so the test cannot be right about an
  encoding the tree is wrong about;
* `a_literal_behind_an_offset_add_is_named_for_the_add_not_the_byte` — the second,
  varying the token *behind* the opcode and the literal's own varint width;
* `an_indirect_store_at_the_bind_position_is_named_a_store`.

The last three exist because **totality residue 0 is not a control** (#144): a
table that gave every byte the same name would pass the first two. §9.17.2 is
what happens when only the first two exist.

**Read-only over the census, run rather than argued.** The 878-TU scan with the
refined tag reproduces the aggregate report **line for line** (the only
difference is the wall clock) and **161,262 of 161,262** dump rows are identical
outside the tag column. The tag-coverage residue stays **0** — no body enters a
production, declines, and reaches an untagged bail. `c2rs perf` geomean 540× at
base, 541×/547× at tip.

**And it lands on two published figures it was not fitted to.** §9.13 derived,
by hand and from a different join, that 3,062 emitted rows of
`expr-intrinsic-this-adjust` are clean, and that **135,926** of that row's
135,941 bodies decline at `eat_receiver_this`. The new axis reads **3,062** and
**135,923** — the second differing by 3 bodies across the two HEADs. A
decomposition agreeing to the unit with a number computed a different way is what
separates this from a relabelling.

#### 9.17.2 The first version of the axis repeated §9.14.7's disease, and the workload caught it

The table's first run filed **5,806 emitted functions** under
`then-op-0x33` — an honest hex bucket, so the totality test passed. The bytes say
otherwise:

```text
53 · 26 79 51 · b9 18 5b a6 43 d5 37 · 33 86 41 74 00 · 27 a6 43 d0 34 · 99 …
     the method   the base pointer      the literal      the OFFSET ADD    bind
```

`33 <int-like> <k>` behind a `27` is a **byte-offset add on the designator**
(`p->f.m()`); the literal only feeds it. Naming the byte the run stopped in front
of is precisely the defect §9.14.7 records for `op-0x55` and the one #139 exists
to cure — reproduced here, in the instrument written to avoid it, one commit
after reading that section. A two-token lookahead names it `then-off-add`, and
the arity test above is what keeps it named.

**After the repair, zero rows land in a hex bucket at all.** All 27 arms carry a
construct name. That is a stronger residue statement than the pre-registration
asked for, and it is a *measured* zero rather than a designed one.

#### 9.17.3 #142, decomposed — and the site is 8 % assignments

At `1f3e00e`, 878-TU dc3 workload, emitted-function units.

| the three receiver-designator sites | emitted | clean | clean ∧ complete |
|---|---:|---:|---:|
| `tail-recv-not-a-plain-b9-load` | 23,158 | 7,824 | 0 |
| `chain-recv-not-a-plain-b9-load` | 2,490 | 18 | 2 |
| `cmp-second-recv-not-a-plain-b9-load` | 6 | 0 | 0 |
| **total** | **25,654** | **7,842** | **2** |

**The denominator aged by 11,406, and not through the rung that took it.** §9.13
read 37,060 / 9,111 / 1,399; #128 converted 1,385. The other 11,406 emitted
functions left the *chain* arm because #128 **re-routed** them to other
productions — §9.13 published that re-route in the body column
(`chain-recv…` 94,948 → 30,183) and never restated it in the emitted one
(13,896 → **2,490**). Pre-registering against 37,060 minus a conversion count
was a MISS by 28 %. §9.14's P3 records a denominator ageing by *date*; this one
ages by a **neighbouring rung's re-routing**, which is invisible in the number
the board item carries.

The clean stock is intact: 7,842 against §9.13's 7,712 residue. **7,741 of it
(98.7 %) reads `complete-none`.**

| receiver construct | emitted | clean | names | `calls-0` | walker |
|---|---:|---:|---:|---:|---:|
| `no-b9-this-adjust` (intrinsic 2113) | 9,653 | **3,063** | 1,290 | 0 | 881 |
| `then-off-add` (`base + k`) | 5,803 | **2,856** | 1,270 | 759 | 252 |
| `b9-not-a-ptr4` | 2,278 | 174 | 34 | 519 | 12 |
| `no-b9-literal` | 1,125 | 107 | 89 | 145 | 206 |
| `then-store` | 1,100 | 25 | 17 | 31 | 0 |
| `then-operand-load` | 1,090 | 292 | **4** | 326 | 0 |
| `no-b9-plain-call` | 1,026 | 10 | 10 | 0 | 197 |
| `no-b9-base-member-addr` (2117) | 706 | 153 | 60 | 0 | 21 |
| `no-b9-base-downcast` (2115) | 598 | 221 | 104 | 0 | 6 |
| `then-dynamic-cast` (2119) | 543 | **542** | 115 | 0 | 0 |
| `no-b9-dynamic-cast` | 460 | 1 | 1 | 1 | 0 |
| `no-b9-convert` | 335 | 200 | 95 | 96 | 111 |
| …15 further named arms | 937 | 198 | — | 183 | 292 |
| **total** | **25,654** | **7,842** | — | **2,060** | **1,978** |

**Three arms are 82.4 % of the clean stock** and they are three different orders
of work: an intrinsic with no production at all (3,063), a designator offset add
(2,856), and a `dynamic_cast` receiver (542).

**8.0 % of the site — 2,060 emitted functions — has no receiver in it, and that
is a byte fact rather than a judgement.** `calls-0` is a body with **no CALL
token anywhere**, so it cannot contain a member call. The body dispatch offers
*every* statement-head `26` to the member-call productions, and an assignment
whose destination is a symbol opens on the same byte:

```text
26 d5 bd 04 00 · b9 5b 0a 86 43 83 20 · 32 86 43 83 20 · 4b     *dest = src;
26 29 0a       · 33 86 41 74 13        · 0f 86 41 74    · 4b     x <op>= 0x13;
```

`32` is `mcall`'s own `Stop::Store`. `then-store` (1,100), `then-operand-load`
(1,090) and `no-b9-literal` (1,125) are assignment statements the production was
offered and declined. Any ranking that reads 25,654 as "receiver work" is over by
at least 2,060 and by more than that on the arms whose `calls-0` share is
partial.

#### 9.17.3a #142's own four keys reconcile — and three of them are not the stock they were listed as

The board item names four keys and their sizes. Three reproduce **to the
function** at this HEAD and the fourth ages by 80:

| key | #142 said | at the site | **clean ∧ ¬complete** | anywhere |
|---|---:|---:|---:|---:|
| `expr-op-0x27` | 5,629 | **5,709** | **2,864** | 22,759 |
| `expr-brfalse` | 1,484 | **1,484** | **0** | 3,102 |
| `assign-store-type-0x86` | 1,138 | **1,138** | **25** | 1,138 |
| `expr-intrinsic-dynamic-cast` | 1,003 | **1,003** | **543** | 1,235 |
| total | 9,254 | 9,334 | **3,432** | 28,234 |

That resolves an arithmetic oddity in the item itself: it describes a **7,712**
population and then lists four keys totalling **9,254**. The lists are of
different things — the four keys' site totals include rows that are not clean —
and in the units the item is actually about they come to **3,432, i.e. 44 % of
the 7,840**.

The per-key decomposition says where each really is:

* **`expr-brfalse` contributes 0.** All 1,475 of its site rows are
  `b9-not-a-ptr4`, and a `brfalse` body is not `cflow-straight`, so none of it
  was ever in the clean stock it was listed under.
* **`assign-store-type-0x86` contributes 25**, and 1,100 of its 1,138 are
  `then-store` — the assignment population of §9.17.3, not member calls.
* **`expr-op-0x27` contributes 2,864**, 5,545 of it the `then-off-add` receiver.
* **`expr-intrinsic-dynamic-cast` splits 543 / 460** between the two positions
  (`then-` and `no-b9-`) and the split sums to the board's 1,003 exactly.

The largest single arm of the clean stock — `no-b9-this-adjust`, 3,062 —
**appears in none of the four**, because it files under `expr-intrinsic-this-adjust`
and that row is already boards #127/#140. So the board's own shortlist covered
44 % of the population and omitted its largest member. Selecting the rows someone
had already looked at is the same shape §9.14.2 records for #139's 1.69×
under-sizing and §8.6 records for the three control-flow keys.

#### 9.17.4 The blocker names ARE trustworthy — and §9.14's repair is not why

The brief asked whether the repaired completeness walker made the names
trustworthy. It is answerable and the answer is a measurement, not an argument:
cross the census key against the **measured** receiver construct, per row, over
the 7,840 clean-not-complete rows. The noun map is deliberately generous to the
"trustworthy" hypothesis and every arm has one, so nothing is absorbed into an
undecidable bucket.

| | rows | |
|---|---:|---|
| the key **names the construct at the receiver position** | **7,421** | **94.7 %** |
| the key names something else | 419 | 5.3 % |
| undecidable | **0** | — |

**The undecidable row is 0 because the first version of it was 4,651**, and that
is a dead end worth naming. The noun map was written from the arm names this
lane *expected* rather than the ones it *measured*, so no intrinsic arm matched
and 59 % of the population fell into a bucket the control could not judge. It
printed 40.6 % agreement, which is a number, looks like a result, and is an
artefact of the map. **A control whose denominator quietly absorbs the cases it
cannot judge is not a control** — the undecidable count is printed on every run
for exactly that reason, and it is what caught this.

**Registered 55 %, interval [25 %, 85 %]. Measured 94.7 % — a MISS above the
ceiling, and it corrects a published sentence.** §9.13 wrote that these rows'
census key "names the second reader's stop, not the first reader's refusal",
which is true of the *mechanism* and reads as an indictment of the *name*. The
name is right 19 times in 20, because the two readers stop on the same construct
one token apart: the production bails at the `33` literal, `parse_expr` walks it
and stops at the `27`, and the key says `expr-op-0x27`. Two independent readers
also agree to the unit on the biggest arm — 3,062 clean rows keyed
`expr-intrinsic-this-adjust` against §9.13's independently derived **3,062**.

**§9.14's repair is almost entirely out of reach of this site, and the control
that says so could have said the opposite.** The repair is inside `mcall`'s
completeness walker, and only that walker mints `expr-call-in-expr-*` keys. So
the question "how much of the site was in the repair's blast radius" is a
countable one:

| at the three sites | emitted rows | |
|---|---:|---|
| key minted by the **completeness walker** | 1,978 | 7.7 % — the population §9.14 *could* have moved |
| key minted by another reader | 23,676 | 92.3 % |
| keys still naming `type-ptr` | **0** | the repair's own success criterion, reproduced here |
| `complete-whole:grammar` over the whole site | **72** | of 25,654 |

and **7,741 of the 7,842 clean rows (98.7 %) read `complete-none`**, which is by
definition a refusal the walker never produced.

The pre-registration said the repair moved **0** of these, which was too strong:
1,978 were reachable. What is measured is the substance — 92.3 % of the site's
keys come from a reader §9.14 did not touch, and the site has 72 completeness
readings across 25,654 emitted functions.

So what #142 is missing is **not truth, it is a completeness bit**. Every arm's
name is right and no arm has a widening estimate attached, which is exactly why
§9.13 called the residue "genuinely unmeasured" — and the reason is structural:
`Block::completeness` returns `NoSignal` for any keyed byte refusal whose `ctx`
is not `CALL_IN_EXPR`, and these refusals are minted by the statement and
assignment layers.

The 419 disagreements are the real second-reader stops and they concentrate:
291 `then-operand-load` rows keyed `expr-convert-target-8642` / `-A641` /
`expr-ternary`, and 87 `no-b9-literal` rows keyed
`expr-call-in-expr-recv-intrinsic-this-adjust-then-intrinsic-call`.

#### 9.17.5 #143 — the `-more` is a COUNT, and the row is worth 6 here and 356 elsewhere

The row reproduces §9.14's figures exactly: **1,038 emitted, 851 clean, 267
distinct names**, and 1,008 of 1,038 bail at
`tail-argument-not-in-the-operand-vocabulary`. **All 1,038 read
`complete-more:grammar`; none reads `-whole`.**

The shape, from a probe whose census key is the row's own
(`work/warms/probe_offadd.cpp`): a **byte-offset add in a call argument**,
`p->one(&t->s.k)`. c2's listing gives the lowering directly:

```text
?a1@@YAXPAUS@@PAUT@@@Z    38840008  addi r4,r4,8
                          48000000  b    ?one@S@@QAAXPAH@Z
?a3@@YAXPAUS@@PAUT@@H@Z   7c8b2378  mr   r11,r4
                          7ca42b78  mr   r4,r5
                          38ab0008  addi r5,r11,8
                          48000000  b    ?three@S@@QAAXHPAH@Z
```

**The `-more` is an arity artefact.** `&t->s.k` is **two** `27` off-adds, one per
designator step — from the probe's own `.ex`:

```text
b9 01 0a 86 43 89 20      the base pointer
33 86 41 74 00 · 27 …     + 0      (t->s)
33 86 41 74 08 · 27 …     + 8      (.k)
55 86 43 f4 08 · 4c       the formal's type, apply
```

`Admit` holds construct **classes**, so the second off-add takes the
`adm.holds(blk)` arm: `need = NEED_MORE`, `broke_on` never set, and the key
renders `-then-off-add-more` **with no `-and-<kind>` third construct**. The
walker's own comment on that arm reads *"a construct that repeats means its
production did not consume the thing the classifier named — a bug, not a body"*.
Here it is a body, and the construct legitimately repeats. Every one of the 1,038
carries a discount that means "twice", not "and something else" — and a one-step
recognizer prices the row at a fraction of itself (the first version of this
lane's sink fired on 1 of the probe's 4 witnesses).

**Four sinks, one base, measured on the same binary with the sink disabled.**

| sink | grants | Δ bodies | Δ emitted | graded against the oracle |
|---|---|---:|---:|---|
| **off** (control) | — | 0 | **0** | reproduces every published number |
| `zero` | an off-add run summing to **0** | 0 | **0** | `Port=Match` (`pz.cpp`) |
| `honest` | the run as `[Load, Lit(sum), Add]` | +5 | **+5** | `Port=Match` (`ph.cpp`) |
| `expr` | `27` as an operator in **all** of `parse_expr` | +6 | **+6** | `Port=Match` (`ph.cpp`) |
| `ceiling` | the run with the **offset dropped** | +1,471 | **+356** | **`Port=Mismatch @ 8`** (`pn.cpp`) |

The `zero` arm is the #127 analogue — a designator chain summing to 0 addresses
the base itself and c2 emits nothing for it, so no codegen is needed. #127's
offset-0 arm was 92 % of its row. **Here it is 0 emitted functions**, though it
is not vacuous: it moves 703 bodies and 136 emitted rows off their keys and
converts none of them. No rate transfers between two arms of one family, and
none transfers between two *families* either.

**Exactly ONE independent refusal separates +5 from +356, and it is named.**
`expr-op-0x27` reads **22,456 in both** the `honest` and `ceiling` scans, so the
351-function difference is entirely `tail_call_shape`'s slot path, which has no
`SlotArg` for a computed address. Every member call is multi-argument by
construction (the receiver is slot 0), so every one of them takes that path.
Registered **≥ 3** independent refusals; measured **1**.

#### 9.17.6 The row above it: `expr-op-0x27` is worth 6

`expr-op-0x27` is the **#1 blocking feature on the emitted board** — 22,759
emitted, 407,016 bodies, 23.2 % of the blocked body column. Granting its named
token in `parse_expr` converts **6 emitted functions**. The row leaves the board
entirely and **201,618 bodies re-file under `expr-op-0x30`**, the indirect load,
which was below the cut before and is now the largest row on the body axis.

The byte-offset add is a **designator-chain prefix**. What stands behind it is
the rest of the member-access chain, and the chain is the work. That is the same
finding §9.14.5 recorded for `recv-load-whole` — a row that looks like the find
of the session and is a phase — reached from the other direction, and it is the
sharpest instance of §8.7's rule that a blocking-feature count is a *position in
a queue*, not a quantity of work.

The widening is principled and is *why* the number is trustworthy: `27` is the
**byte**-offset add and `02` is the scaled one (`p + 1` on an `int*` emits
`addi r3,r3,4`), which is why `parse_expr`'s pointer-arithmetic guard refuses
`02` over a pointer and why `27` may be exempt from it. The sink separates the
two facts the old single `saw_ptr && any-arith` test conflated. Shipping it would
also oblige `mcall::eat_int_operands`'s `Vocab::CallArg` to widen in lockstep, or
§9.14.6's correspondence guard goes red.

#### 9.17.7 DECLINE, and the control that would not have caught the over-claim

**#143 is declined in this lane**, under the rule registered before the
measurement. The realizable worth here is **6 emitted functions** — §8.7's
decline size, and smaller than the 8 the strongest lane of the week declined at.
The 356 needs a new `SlotArg` variant and its **ordering rule** inside the
permutation walk, in `crates/c2-core`, which this lane may not touch. §9.13.1's
ALARM is exactly the rule at issue: `?a3` is `mr r11,r4 ; mr r4,r5 ;
addi r5,r11,8` — **one** non-address move in the walk, which is the n ≤ 1 case
where address-last and address-second agree and where a wrong rule ships green.

**The workload differential could not have caught the over-claim, and this is the
thirteenth time that shape has come up.** The `ceiling` sink emits `mr` where c2
emits `addi`. The 878-TU scan under it still reads **6 match, 0 mismatch,
census/gate disagreement 0** — because none of the six byte-exact TUs carries the
shape. Only the dedicated probe failed, `Port=Mismatch @ offset 8`, and the
`zero` arm's `Port=Match` beside it is what says the probe can also pass.
Registering "0 mismatch on the workload" as this rung's control would have been
§9.13's E4 verbatim.

#### 9.17.8 Pre-registration score — 13 of 17, and three of the four misses are the findings

| | registered | measured | |
|---|---|---|---|
| A1 | site emitted 35,700, [32,000 , 38,500] | **25,654** | **MISS**, below the floor |
| A2 | clean 7,730, [6,300 , 9,200] | **7,842** | HIT |
| A3 | clean ∧ complete 60, [0 , 900] | **2** | HIT |
| A4 | the intrinsic family is the largest, 40 % [20 , 70] | largest, **50.8 %** | HIT |
| A5 | top three ≥ 80 % of clean | **82.4 %** | HIT |
| A6 | arms with ≥ 500 clean: 4, [2 , 8] | **3** | HIT |
| A7 | 0 rows in an unnamed bucket | **0** — after a repair | HIT (see below) |
| A8 | the axis is read-only, to the unit | report identical, 161,262/161,262 | HIT |
| C1 | key/construct agreement 55 %, [25 % , 85 %] | **94.7 %** | **MISS**, above the ceiling |
| C2 | ≥ 30 % name something else | **5.3 %** | **MISS** |
| C3 | §9.14's repair moved **0** of these | **1,978 (7.7 %) were reachable**; 0 still name `type-ptr` | HIT on the substance, the **0** was too strong |
| B1 | the row ages by ≤ ±15 % | 1,038 / 851 / 267, **exact** | HIT |
| B2 | Δ emitted 60, [0 , 400] | **0 / 5 / 6 / 356** | HIT on the interval |
| B3 | ≥ 3 independent further refusals | **1** | **MISS** |
| B4 | DECLINE | **DECLINE** | HIT |
| B5 | the disabled sink reproduces the base | every number | HIT |
| B6 | the differential is the control that can fire | it **fired** | HIT |

* **A1 is a new way for a denominator to age.** §9.14's P3 records one that aged
  by *date*; this one aged because the **neighbouring rung re-routed** 11,406
  emitted functions out of the site without converting them, and the board item
  carried only the conversion count. Subtracting a rung's realized gain from a
  site it touched is not a correction — it is a guess that the rung moved
  nothing else.
* **C1/C2 are one miss and it corrects §9.13.** The prediction inherited §9.13's
  sentence about second-reader stops and read it as "the names are wrong". The
  names are right 94.7 % of the time; the thing that is missing is the
  completeness bit. A published mechanism restated as a quality judgement is how
  a wrong prior gets inherited, and the fix is that C1 was registered as a
  *number* with an interval that could have contained either answer.
* **B3's miss is the section's largest finding.** It was registered off the
  `-more` suffix, i.e. off exactly the discount the measurement then showed to be
  an arity artefact. Reading a suffix as evidence about *what* is behind a row,
  when it is computed by a set that cannot count, is #110/#139's failure in a
  third costume.
* **A7 passed in the letter and failed in the spirit.** `op-0x33` is an honest
  hex bucket, so a totality test could not see that 5,806 functions were filed
  under the byte the run stopped in front of. Totality residue 0 is not a
  control (#144); the arity test is.
* **B2 registered one number for a quantity that has four.** All four realized
  values land inside the interval, which makes the hit weak evidence: the
  question "what does granting this construct cost" has a different answer for
  each of four gates and the registration did not say which gate it meant.

#### 9.17.9 Gate evidence

At `39ae1e2`, worktree branched from `1f3e00e` (verified: the harness's own
worktree branched from `origin/master`, **587 commits behind**, and was reset
before any work — the third lane this week to meet that), cache addressed by its
canonical main-repo path.

* `cargo test --workspace` — base `1f3e00e` **589 passed, 0 failed, 1 ignored**
  → tip **594 passed, 0 failed, 1 ignored**. Both measured, not inferred: the
  base was rebuilt from `git checkout 1f3e00e -- crates` and re-run.
  **`#[test]` grep over `crates/` 590 at base → 595 at tip.** Grep and runner
  reconcile at both ends once the one `#[ignore]`d test is added to the runner's
  passed count (590 = 589 + 1; 595 = 594 + 1), so no grep line here is prose or a
  doc comment. Five new portable tests, all enumerated over their domain.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip**.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — 42,719 configurations × 12 lanes =
  **512,628 gradings, 512,628 graded, 0 mismatches**; 406 of 406 declared family
  pairs reached *and* emitted; refusal-frontier residue **0**. Identical to
  §9.14's run on every one of those numbers. Run because this lane touched
  `parse_expr` and `eat_call_args`, even though both additions are inert with
  the sink unset.
* 878-TU workload scan — **6 match, 0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies **706,402 / 2,462,571 (28.69 %)**; emitted **36,059 / 178,968
  (20.15 %)**; census/gate disagreement **0**. Identical to base on all of them,
  with the row dump armed and with it off.
* `c2rs perf` — geomean **540×** at base, **541× / 547×** at tip over the 100
  matched fixtures; 100 Match, 0 mismatch, 110 not-implemented. The two sinks add
  one `OnceLock` read each on a locator that runs per call argument, and the
  measurement is here because arguing it would not be.
* Probes — `pz.cpp` `Port=Match` under `zero`; `ph.cpp` `Port=Match` under
  `honest` and `expr`; `pn.cpp` `Port=NotImplemented` under `zero` and
  **`Port=Mismatch @ offset 8`** under `ceiling`.

**No fixture was added and no `fixtures/cpp/` entry changed**, because nothing
shipped. The probes live under `work/` and are named in this section so the
decline is reproducible; a fixture for a shape the port refuses would put a claim
in every gate lane that this lane did not earn.

#### 9.17.10 Board items

* **#149 — the off-add ARGUMENT slot, 356 emitted, in `crates/c2-core`.**
  Needs a `SlotArg` variant for `base + k` and its **position in the permutation
  walk**. Route to whoever owns codegen. The capture grid is §9.13.1's with one
  axis added, because §9.13.1's ALARM is the exact rule at issue: (walk length
  0…4) × (the address at slot 0 / a middle slot / last) × (offset 0 / small /
  past the `addi` immediate) × (free and member callers). The measured
  counterfactual, the probes and the four sinks are in `39ae1e2` behind
  `C2RS_SINK_OFF_ADD_ARG`; **do not re-derive the row's worth from its census
  size or its clean figure** — 1,038 emitted overstates the ceiling by 2.9× and
  851 clean by 2.4×, and the *realizable-without-codegen* number is 6.
* **#150 — `expr-op-0x27` is worth 6 emitted functions.** The #1 row on the
  emitted board. Behind it is `expr-op-0x30` and the rest of the member-access
  chain (201,618 bodies re-file there). The board should carry the number so
  nobody schedules 22,759.
* **#151 — the completeness walker cannot COUNT, and reads its own inability as
  a bug.** `Admit` holds construct classes, so a construct that legitimately
  repeats renders `-more` with no `-and-<kind>`, and the code comments that state
  as "a bug, not a body". **Every `-then-<x>-more` key with no `-and-` is a
  candidate for the same misreading**, and each one is a row somebody may discount
  on a suffix that means "twice". Same family as #110/#139: one measure, wrong
  about what it is measuring. An `Admit` that carried a multiplicity would name
  these `-whole2` instead.
* **#152 — the receiver-designator site is at least 8.0 % assignments.** 2,060 of
  25,654 emitted rows are `calls-0` — no CALL token in the body at all. The body
  dispatch offers every statement-head `26` to the member-call productions. Any
  sizing of #131/#142 off 25,654 is over by at least that, and the three arms
  concerned (`then-store`, `then-operand-load`, `no-b9-literal`) are named and
  countable now.
* **#153 — the three receiver arms worth ranking**, with no rate borrowed between
  them: `no-b9-this-adjust` 3,063 clean (this is #140's row, sized at **472**
  emitted end to end — the clean figure is 6.5× its measured worth),
  `then-off-add` 2,856 clean (the receiver-side twin of #143, whose
  argument-side arm converted 6 — **the twin is unmeasured and no rate may be
  borrowed from it**), `then-dynamic-cast` 542 clean over 115 names.
  Two `clean` ceilings in this family have now been measured against a realized
  number: **3,063 against #140's 472 (6.5×)** and **851 against #143's 6
  (142×)**. §8.7 says `clean` is an optimistic ceiling and not an estimate; the
  spread between those two is why it cannot be scaled either.
* **#154 — `Block::completeness` returns `NoSignal` for every refusal minted
  outside `CALL_IN_EXPR`**, which is 98.7 % of #142's clean stock. That is not a
  defect — it is honest — but it means the largest site on the emitted board can
  never be ranked by completeness while its keys come from the statement and
  assignment layers. Either the walker reaches these positions or the board needs
  a second completeness producer for them.

# 9.18 W-EMITSET — the emit set is not predictable, the ceiling is 111, and the inliner is not the reason (2026-08-01)

DRAFT for `docs/ROADMAP.md` §9.18 — written by lane `w-emitset`, to be landed by
the coordinator. Nothing in §1–§9.17 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-emitset-prereg.md`, committed at `3b9a4ae` before the
first measurement. Base and tip `74d0744` + this lane's commits.

**Headline: TU match 6 → 6.** Measurement and one instrument; no codegen, no
emit-set model shipped, and shipping one would have been wrong. Four findings,
in descending order of how much they change what to do next:

1. **The emit set is not predictable from anything the census can see.** A cell
   table over *every* feature the instrument has — binding, mangling, access
   code, census key, control flow, EH, frame class, dispatch, production,
   completeness — fitted on 432 TUs and graded on a disjoint 432, scores
   **94.938 % per body against a 93.700 % never-emit base**, and **1 of 432 TUs
   exactly right — the same 1 the base predictor gets, and it is a TU that emits
   nothing.** The model is worth **1.24 pp of bodies and zero TUs**.
2. **The binding, not the codegen and not the inliner, is the ceiling — and it is
   111 of 871.** For a TU to be byte-exact the port must reproduce the reference
   `.text` COMDAT set, and it can only emit a COMDAT for a body it has under a
   name the binding gives it. **760 of 871 TUs carry at least one emitted symbol
   no census row claims.** §9.16.3's 25 is a count comparison on a model-free
   port; 111 is the ceiling on any *model*.
3. **The `/O1` inline-decline schedule does NOT gate emission, and this is
   measured at workload scale.** §8.1 called it "the least-derivable model in the
   program" and made it the reason not to attempt Phase 7. **58.6 % of every
   function c2 emits is ≤ 64 bytes** — `LABEL_COUNTER.md` §6.15.3's *unbounded*
   band, the callees c1xx inlines at every site. The median emitted function is
   **40 bytes, ten instructions**. §6.5's fixture result generalizes: c2 emits
   the fully-inlined callee anyway. **The hardest thing on the board is not on
   the critical path.**
4. **What the residue actually is has a name, and it is the polymorphic class.**
   Of the 13,646 emitted symbols with no readable `.gl` body record, **70.0 % are
   virtual members** and 47.6 % are the `??_` synthesized family — and the
   control holds: the *bound* population is 42.1 % virtual, so "virtual" is not a
   fact about mangled names in general. Restricted like for like to non-`??`
   names: **bound 42.1 % virtual, unreadable-record 98.8 %.**

---

## 9.18.1 The ceiling on a MODEL is a different number from §9.16.3's, and lower

§9.16.3 measured `.ex` segment count against obj `.text` COMDAT count and got
**25 of 871**. That is the ceiling on the port *as it stands* — one COMDAT per
segment, no model. It is a comparison of two integers.

A model has to reproduce the **set**, and it is constrained further:

> `PortC2::build` can only ever emit a COMDAT for a body this bundle carries,
> under the name the `.gl` binding gives that body. An emitted symbol no row
> claims is a COMDAT the port cannot produce at any codegen quality and under
> any predictor.

That residue was already published — **17,706 symbols, 9.89 % of the 178,968
denominator** (§9.9.3) — but only ever as a *total*. Per TU it is the binding
constraint, and it had never been read that way:

| | TUs of 871 |
|---|---:|
| every emitted symbol binds to a census row — **reachable today** | **111** |
| would, if `bind.rs` lost none of the records it already finds | **116** |
| carries ≥ 1 emitted symbol with **no** `.gl` body record this reader can find | **755** |

Six of the 111 emit nothing at all, so the non-vacuous figure is **105**.

**And the arithmetic control (#144 — residue 0 is not a control, add an arity
check).** Counting TUs with a residue is not the same as counting the residue's
contents: median unbound-per-TU **10**, mean **20.3**, max **192**, sum
**17,706**; for the no-record half alone, median **9**, mean **15.7**, max
**127**, sum **13,646**. **60 TUs carry exactly one no-record symbol.** A ceiling
reported only as "760 TUs blocked" would have hidden that a twelfth of them
block on a single symbol.

## 9.18.2 The split that had to exist before the ceiling could be read

An emitted symbol no row claims had been one number. It is two things that need
opposite work, and the ceiling is stated over the second:

* it has a framed `.gl` body record — **the body is in this bundle and
  `EmitBinding` lost the row.** An instrument defect, closable in `bind.rs`.
* it has none — **a wall**: a segment-driven port must synthesize the COMDAT.

`c2_il::gl_body_record_names` reports every name owning a framed body-start
record, with the *same* framing and the *same* name-distance bound as
`EmitBinding::new`, deliberately — so a difference between the two answers is a
difference in the **binding**, never in the reader. Diagnostic only: the gate,
the census verdict and the emitter do not consult it, and every published number
is byte-identical with it armed (§9.18.7).

```
17,706 unbound emitted symbols
   4,060  have a body record   — instrument defect (bind.rs)
  13,646  have none            — the wall, as this reader sees it
```

**The two tests are on the cases that discriminate, per #145.** A row two
records collide on binds nothing and must still report *both* names — otherwise
the ceiling reads a binding collision as "c2 emitted a symbol with no body" when
the body is right there. And a symbol with no record must **not** be invented —
otherwise `emit-unbound-no-record` is 0 by construction, which is the
absence-read-as-success shape exactly.

## 9.18.3 The wall is the polymorphic class, and the control could have failed

**Key names lie, so this went to the byte.** `mangling_class` reports 47.7 %
`special-generated`, which is *every* `??_…` — and `??_` is `??_G`/`??_E`/`??_D`
(real synthesized functions) as well as `??_7` (vftable), `??_R0`…`??_R4` (RTTI)
and `??_C` (string literals), which are data. A decomposition that never prints
a name cannot tell those apart, and the whole reading rests on which it is. The
names, by exact prefix:

| | count | share of 13,646 |
|---|---:|---:|
| `??_G` scalar deleting dtor (synthesized) | 4,862 | 35.6 % |
| `??1` destructor | 3,370 | 24.7 % |
| `?` ordinary member/free function | 2,755 | 20.2 % |
| `??0` constructor | 794 | 5.8 % |
| `??_D` vbase dtor iterator (synthesized) | 587 | 4.3 % |
| `??_E` vector deleting dtor (synthesized) | 582 | 4.3 % |
| `??__F` dynamic atexit dtor (synthesized) | 379 | 2.8 % |
| `??` operator | 189 | 1.4 % |
| `??_H`, `??_F`, `??__E`, other | 99 | 0.7 % |
| undecorated | 1 | 0.0 % |

No RTTI, no vftables, no string literals — the `.text`-COMDAT-function filter
already excluded them, and printing the names is what established that rather
than assuming it. **6,508 (47.6 %) are genuinely synthesized**; the other 7,138
are real user functions with real bodies.

**And what those user functions have in common is virtualness.** MSVC's access
code sits immediately after the `@@` closing the qualified name; virtual is
`{E,F,M,N,U,V}`. The **bound** population is the control, and it could have
refuted this outright:

| population | n | virtual | non-virtual member | static | free |
|---|---:|---:|---:|---:|---:|
| bound (control) | 89,700 | **42.1 %** | 29.5 % | 4.9 % | 22.3 % |
| unbound, has record | 3,459 | **2.4 %** | 52.6 % | 0.3 % | 35.2 % |
| unbound, **no record** | 2,756 | **98.8 %** | 0.8 % | 0.0 % | 0.3 % |

(non-`??` names only, so the comparison is like for like.) Over the whole
no-record class, **9,553 of 13,646 = 70.0 % are virtual**. The bound control at
42.1 % says this is not a property of mangled names; the has-record column at
2.4 % says it is not a property of being unbound either. It is a property of
**this reader**, and the byte-level reason is visible in
`src/system/obj/TextFile.cpp`: a virtual member's `.gl` record carries extra
material between the name and the offset field —
`?Print@TextFile@@UAAXPBD@Z\0 82 07 05 00 00 20 01 04 02 93 45 dd 20 80 a3 22`
against a non-virtual `??0DataNode@@QAA@H@Z\0 86 03 05 04 20 00 02 01 00 80 …` —
so the `80 <LE32> 00 00` framing and the 32-byte name-distance bound lose it.

## 9.18.4 The ceiling ladder — what each repair is worth, in TUs

Stated as **ceilings under named repairs, not as achieved results.** §9.16.1
records what happens when a board's payoff field and its outcome field are the
same field; everything below the first row is a counterfactual and must never be
written back as a status.

| | TUs of 871 | delta |
|---|---:|---:|
| **measured today** — every emitted symbol binds | **111** | — |
| + repair the ROW binding in `bind.rs` (the 4,060 has-record symbols) | 116 | +5 |
| + read the virtual member's `.gl` record shape | 204 | +88 |
| + synthesize the `??_` family (no `.ex` body exists) | 238 | +27 |
| + **both** of the last two | **436** | +325 |
| after both, still blocked | 435 TUs, 1,797 symbols | |

**The +5 is the surprise and it is the useful one.** Repairing the row binding —
the residue this project has been reporting for weeks — buys **five TUs**. The
work that matters is the *record reader* (+88) and COMDAT synthesis (+27), and
they are worth **+325 together**, far more than the sum of their parts, because
most blocked TUs carry both kinds.

**What is left after both, by name**, and it is a third compiler-generated
population that no `??_` prefix marks: `??1?$_STLP_alloc_proxy@…@@QAA@XZ` (389),
`??1?$ObjDirItr@…@@QAA@XZ` (161), `??0bad_alloc@std@@QAA@ABV01@@Z` and
`??0logic_error@…@@QAA@ABV01@@Z`-shaped copy constructors (66 each). These are
**implicitly-declared special members** — an implicit copy constructor or
destructor is mangled exactly like a user-written one, so no prefix separates
them. That is the honest open end of this decomposition and the next lane's
first question.

## 9.18.5 The predictor, fitted and graded — and the arithmetic that dooms it

Split 864 TUs (the ones that emit anything) into two disjoint halves **by TU**,
432 fit and 432 grade; every threshold and every cell fitted on the fit half
only. The model is a **cell table**: partition rows by a feature cross, take each
cell's majority label on the fit half, apply to the grade half. That is the most
favourable non-parametric predictor over those features — if it cannot separate,
no simpler rule over the same features can.

| model | cells | held-out per-body | TP | FP | FN | TUs all rows right |
|---|---:|---:|---:|---:|---:|---:|
| **P0 never emit (BASE)** | 1 | **0.93700** | 0 | 0 | 80,479 | **1** |
| has-name | 2 | 0.93700 | 0 | 0 | 80,479 | 1 |
| + mangling + access code | 28 | 0.93701 | 13 | 2 | 80,466 | 1 |
| + census key | 1,527 | 0.94651 | 14,808 | 2,661 | 65,671 | 1 |
| + cflow / EH / frame | 3,319 | 0.94900 | 18,808 | 3,482 | 61,671 | 1 |
| + dispatch / production / completeness | 3,674 | **0.94938** | 18,823 | 3,013 | 61,656 | **1** |

**Per-TU exact set — predicted emitted names == reference emitted names — is 1
of 432 for every model, including the base.** That TU is `src/system/decomp_pch.cpp`
and it is the **only** held-out TU with zero emitted rows, verified by name: the
best model gets **zero TUs that emit anything**. The best model recovers 18,823
of 80,479 emitted functions (23.4 % recall) and invents 3,013.

**And the per-TU figure is scored generously, deliberately.** The reference set
it is compared against is the *bindable* emitted set — the 17,706 symbols of
§9.18.1 are not in it, because no row carries them. So the grading pretends the
ceiling problem away and the model still scores 1 of 432. Scored against the
real `.text` COMDAT set it would score 1 of 432 as well, but for two independent
reasons instead of one.

**C-leak, registered and it bit.** 1,134,139 rows (46.1 %) carry no bound name
at all, so they are `not-emitted` **by binding failure, not by c2's decision**,
and every model banks them for free. Restricted to the 1,328,432 named rows the
positive rate is 12.14 %, not 6.55 %.

**And a correction to a published headline that follows from the same fact.** §8.1's
denominator is 178,968 emitted against 2,462,571 bodies = 7.27 %. The rate a
segment-driven model can actually *see* is **161,262 / 2,462,571 = 6.55 %** —
the other 17,706 are not reachable from any segment. Any statement of the form
"the port need only decide 7.23 % correctly" is 0.72 pp optimistic, and the
missing 0.72 pp is exactly the population §9.18.1 shows is the ceiling.

**E10's arithmetic, which was registered before any of this and is the reason
per-body accuracy is not a headline.** Median 2,136 rows per held-out TU:

| per-body accuracy | expected TUs all-right, of 432 |
|---|---:|
| 0.99 | 0.00 |
| 0.999 | 51.0 |
| 0.9999 | 348.9 |
| 0.99999 | 422.9 |

At the measured **0.94938** the expectation is `432 × 0.94938^2136 ≈ 10⁻⁴⁶`.
**Three nines is the entry price and the instrument is two orders of magnitude
away.** Reporting "94.9 % accurate" without this table would be the single most
misleading sentence this lane could have written.

## 9.18.6 The inliner is not the reason — measured, with a control that could have failed

§8.1 declined to pre-decide *lower everything* versus *model the emit set*, and
the stated reason was that the emit set's "inliner half is the least-derivable
model in the program (`LABEL_COUNTER.md` §6.15.3: the `/O1` inline-decline
schedule is measured exactly and *generated by no formula*)."

**The premise is false, and the test is one line of arithmetic on the objs.**
The schedule's axis is `s`, the callee's own emitted `.text` size, and its top
band is `s ≤ 64` bytes (≤ 16 instructions) = *inlined at every site, unbounded*.
§6.5 claims on fixtures that c2 emits the callee's COMDAT anyway. If emission
were inline-gated, the `s ≤ 64` band would be the **rarest** among emitted
functions — every member of it was inlined everywhere and needs no out-of-line
copy. If §6.5 generalizes it should be one of the commonest.

Every `.text` COMDAT of 25 TUs, taken as every 36th line of the workload list so
it is a spread and not a hand-pick — **4,490 emitted functions**:

| `s` band | schedule | emitted | share |
|---|---|---:|---:|
| **≤ 64** | **inlined at EVERY site** | **2,632** | **58.6 %** |
| 68–72 | 9 sites | 145 | 3.2 % |
| 76 | 7 sites | 136 | 3.0 % |
| 80 | 5 sites | 57 | 1.3 % |
| 84–88 | 4 sites | 215 | 4.8 % |
| 92–100 | 3 sites | 165 | 3.7 % |
| 104–140 | 2 sites | 304 | 6.8 % |
| 144–256 | 1 site | 587 | 13.1 % |
| ≥ 260 | never inlined | 249 | 5.5 % |

Median emitted function **40 bytes = 10 instructions**; the commonest sizes are
**8 (578×), 4 (455×), 12 (355×), 16 (220×)** — one- to four-instruction getters,
every one of them a callee c1xx inlines at every site. Sanity control: **0 of
4,490 sizes is not a multiple of 4.**

**So the inline decision does not enter the emit predicate.** It is also
structurally impossible that it could: §6.15.1 measured that the *front end*
decides inlining, once, per (caller, callee) pair, so by the time c2 sees the IL
the inlining is done; and the schedule's axis `s` is the callee's **emitted**
size, which does not exist for a body that is not emitted — the axis is
undefined on exactly the population whose emission is in question.

**This is the lane's best news and it should be read as a re-plan, not a
reassurance.** §8.1's stated reason for leaving Phase 7 undecided is retired.
The reason to be cautious about Phase 7 is now §9.18.1's 111 and §9.18.5's
1-of-432 — both of which are about the *binding* and about *synthesis*, and
neither of which is undecidable.

## 9.18.7 What the residue does NOT decompose into, and why that is a result

The brief asked for the residue decomposed by c2's own named disjuncts —
`globally unreferenced`, `has linear flow`, `is a redirector function`,
`won't be inlined (too big)`, `inlining prohibited`, `unreferenced import`,
`InlBadCandidate`. **This lane could not do it and no lane can with the
instruments that exist.** Those strings live in `c2.dll`'s string table; §9.5
already established with a positive control that there is no switch that dumps
them (~25 candidate flags return `C1007 unrecognized flag … in 'p2'`), and the
`.cod` listing prints assembly, not inline decisions. **There is no per-body
disjunct label anywhere in this project's reach.**

What *is* measurable is a structural partition of the 2,301,309 not-emitted
bodies, and its shape is the answer to the same question:

| | bodies | share |
|---|---:|---:|
| segment with **no bound name** — the binding cannot say what it would be called | 1,134,139 | 49.3 % |
| named row c2 did not emit | 1,167,170 | 50.7 % |

**and the second half does not decompose further.** The cell table of §9.18.5
partitions it 3,674 ways on every axis the census owns and moves accuracy
**1.24 pp**. That is the registered refuter for E5 firing: *a flat decomposition
⇒ no cheap clause exists.* There is no 90 %-and-cheap clause. There is no 40 %
one either.

## 9.18.8 Fail-closed wiring — how a model would ship without ever guessing a COMDAT

Nothing here shipped. This is the design the next lane inherits, and the point of
writing it now is that **the shape is forced**, not chosen.

1. **The decision is per TU, not per body.** `PortC2::build` already emits a
   whole obj or nothing. The model must be **total**: every `.ex` segment gets
   `Emit(name)`, `Skip`, or `Unknown`, and **one `Unknown` refuses the TU**.
   There is no partial credit — a TU with one wrong COMDAT is a mismatch, and a
   mismatch outranks every other outcome.
2. **`Emit` requires a positively bound name from the GATE binding**
   (`Bindings::per_record`), never from `EmitBinding`, which is the diagnostic
   one and is deliberately looser. A body emitted under the wrong name is a
   relocation against the wrong symbol — a mis-emit, not a gap. Today
   `emitset-unnamed-segment` alone would refuse essentially every TU (46.1 % of
   segments are unnamed), and **that is correct behaviour**: it is the model
   telling the truth about §9.18.1's ceiling instead of guessing past it.
3. **Refuse on the presence of a construct, never on the absence of a symbol.**
   The port cannot check "c2 emitted a `??_G` I do not have" — it would have to
   know the name already, and it only knows the reference's names at *grade*
   time, which is the oracle. The port-side gate has to be positive: *this TU
   declares a class with a virtual destructor* / *a namespace-scope object with a
   dynamic initializer* → refuse until a synthesis phase exists. Refusing on an
   absence is the failure mode this document records twelve times.
4. **The byte compare DOES grade the emit set, and that is the strongest reason
   to build it behind the existing gate rather than behind a new classifier.** A
   wrongly skipped body shortens the obj; a wrongly emitted one lengthens it;
   either diverges at COFF offset 2 (`NumberOfSections`) or 8
   (`PointerToSymbolTable`) long before any instruction byte matters. **But
   #149/§9.17's coverage bound applies with full force**: today's 878-TU scan
   reads 0 mismatch because 865 TUs refuse before reaching the emitter, so the
   scan cannot see an emit-set defect at all. A dedicated probe is owed —
   fixtures carrying a header inline that **is** emitted beside one that is not,
   on a TU the port compiles whole.
5. **The set is not enough; the model owes an ORDER.** COMDAT section order,
   symbol-table order, and the per-TU label-counter surcharge (`LABEL_COUNTER.md`
   §1.1) are all functions of the emitted sequence. A model that gets the set
   right and the order wrong is still a mismatch. The `.cod` listing seam
   (§9.1–§9.4) is where the order is readable symbolically, and §9.3 already
   records that the label counter's phase order is **not** text order.

## 9.18.9 Pre-registration, scored — 6 of 10, and the misses are the lane

Registered in `docs/rungs/_2026-08-01-w-emitset-prereg.md`, committed at
`3b9a4ae` before the first measurement. Declared bias: **borrowed and
structural** (§6.5 was read first) and pessimistic on E4.

| # | claim | est | interval | actual | score |
|---|---|---|---|---|---|
| E1 | share of `.ex` segments with a bound name | 25 % | [10, 45] | **53.94 %** | **MISS** — above the ceiling |
| E2 | P0 held-out per-body accuracy | 92 % | [85, 99] | **93.700 %** | HIT — but see below |
| E3 | P0 held-out per-TU exact-set, of 871 | 2 | [0, 40] | **1** (of 432 held out) | HIT |
| E4 | best predictor's TUs made emit-set-reachable | 60 | [5, 300] | **~2** | **MISS** — below the floor |
| E5 | largest single disjunct's share of not-emitted | ≥ 80 % | [40, 99] | **not measurable**; nearest analogue 50.7 %, decomposition FLAT | **MISS** |
| E6 | cost of the inliner clause, in TUs | 0 | [0, 100] | **0** | **HIT** |
| E7 | unbound emitted share, re-measured | 9.9 % | [8, 12] | **9.89 %** | HIT |
| E8 | `.gl` linkage byte takes ≥ 2 values ≥ 1 % among bound records | YES | — | true on one TU; **never measured at workload scale** | **MISS (process)** |
| E9 | TUs converted by this lane | 0 | [0, 0] | **0** | HIT |
| E10 | per-TU far worse than per-body implies | YES | — | 0.94938 per body → 1 of 432 | HIT |

**6 of 10, and three of the four misses carry the lane's value.**

* **E4 is the important miss and it is low, which is the useful direction.** I
  registered 60 TUs with a floor of 5, having read §9.16.3's 25 and expected a
  model to beat a model-free port. It does not beat it — it does not beat
  *nothing*. That is what makes §9.18.6 a re-plan rather than a ranking tweak.
* **E1 was wrong by more than the interval's whole width**, and I had the
  arithmetic to get it right before estimating: the scan already printed
  `emit-records 1,515,160`, `nameless 152,941`, `row-conflicts 33,552`, which
  subtract to 1,328,432 of 2,462,571 = 53.9 %. **I registered a guess where a
  subtraction was available.** Same class as §9.16.7's E8 — a borrowed prior
  used in place of a number already in the report.
* **E2 is a hit whose refuter fired.** I registered "below the 92.77 % never-emit
  base rate ⇒ worse than nothing". The measured base is **93.700 %**, not
  92.77 %, because 92.77 % was computed from `emit-emitted`/bodies and the
  reachable positive rate is `emit-bound`/bodies = 6.55 %. P0 did not fall below
  the base — it **tied it exactly, TP = 0**. A registered refuter stated against
  a stale constant would have passed a predictor that predicts nothing.
* **E8 is a process miss and is reported as one.** I registered an estimate and
  then never built the instrument that would have graded it, because §9.18.5 made
  the answer irrelevant. *Irrelevant is not measured*, and the honest score is a
  miss.

## 9.18.10 Gate evidence

| lane | base `74d0744` | tip |
|---|---|---|
| `cargo test --workspace --release` | **596 passed, 0 failed, 1 ignored, 24 targets** | **598 passed, 0 failed, 1 ignored, 24 targets** |
| `#[test]` grep over `crates/` | **597** | **599** (+2, both new) |
| `scripts/gate.sh --jobs 6` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **2,520 fixture-verdicts**, 0 mismatch in every lane |
| `c2rs selftest` | — | **210 PASS, 0 FAIL** |
| 878-TU workload scan | match 6, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 | **identical** |
| census | 706,402 / 2,462,571 (28.69 %) | **identical** |
| emitted census | 36,059 / 178,968 (20.15 %) | **identical** |
| census/gate disagreement | 0 | **0** |
| distance (bodies) | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 | **identical** |
| distance (emitted) | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 399, ≤1000: 857 | **identical** |
| emit-set ceiling (§9.16.3) | 25 of 871, violations 0 | **25 of 871, violations 0** |
| emit-set MODEL ceiling | not measured | **111 today / 116 repaired / 755 wall** |

**Target count recorded beside test count**, per §9.16.8: 24 at base and 24 at
tip. `cross_sweep` not run — **no codegen was touched.** The diff is
`c2-il/src/func/bind.rs` (one `pub fn`, two tests), two re-exports, and
`c2-harness` (`gap.rs` accounting + a scratch dump, `main.rs` two report lines).
`PortC2`, `codegen` and every recognizer are untouched.

**Instrument inertness, asserted by running it rather than argued.** The full
878-TU scan was run three times — plain, with the wall dump armed, and with the
per-row dump armed — and census, emitted census, match, mismatch, disagreement,
both distance ladders and the §9.16.3 ceiling are byte-identical across all
three. The dumps' own totality checks are the second half of that: the wall dump
emits **178,968** lines, exactly the emitted denominator, and the row dump
**2,462,571**, exactly the census denominator.

**Environment caveat, inherited and restated:** this box's `wibo` is `1.0.1-7`,
older than the known-good `1.0.1-23`. Every scan here used `--replay-every 0` and
this lane quotes no replay number; census and mismatch counts are byte-identical
under both loaders, which is what the scan's own warning says.

## 9.18.11 Found and not taken, ranked

1. **The virtual `.gl` record shape** (§9.18.3) — worth **+88 TUs** of ceiling on
   its own and **+325 with COMDAT synthesis**, it is a *format* job with a
   byte-level witness already transcribed, and it is the largest single number
   this lane found. It is `bind.rs` and `gl.rs`, not codegen. **Rank 1 by a wide
   margin.**
2. **COMDAT synthesis for the `??_` family** — 6,508 symbols, 47.6 % of the wall,
   and `??_G` alone is 4,862. A scalar deleting destructor is
   `{ this->~T(); if (flags & 1) operator delete(this); }`, a shape small enough
   to be a fixture. It also has the cleanest possible first probe: §9.16.10
   already named `TomCryptLicense` / `ZlibLicense`, **zero `.ex` bodies and one
   emitted COMDAT each**.
3. **The implicitly-declared special members** (§9.18.4) — 1,797 symbols across
   435 TUs that no prefix marks and that this lane could only name, not classify.
   Whoever takes rung 1 will meet them immediately.
4. **§8.1's "least-derivable model" clause should be struck**, not softened.
   §9.18.6 refutes its premise at workload scale with a control that could have
   failed. Leaving it in place mis-ranks Phase 7 for the same reason §9.16.6
   found Phase 6 mis-ranked: the sentence is load-bearing and nobody re-measured
   it.
5. **The 7.23 % figure should be published as 6.55 %** where it is used to size
   the emit-set decision (§9.18.5). The difference is precisely the unreachable
   population, so quoting the larger number understates the very constraint it is
   quoted to describe.

DRAFT for `docs/ROADMAP.md` §9.19 — written by lane `w-slotarg`, to be landed by
the coordinator. Nothing in §1–§9.18 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-slotarg-prereg.md`, committed at `dbd104b` before the
first capture; the out-of-sample registration is
`docs/rungs/_2026-08-01-w-slotarg-grid3-prereg.md`, committed at `6caeddc`
**before grid 3 was compiled**.

---

### 9.19 W-SLOTARG — the +356 is real, the naive lowering is wrong, and the ordering rule survives 360 cells and dies on the 361st (2026-08-01)

Board **#149** (the off-add ARGUMENT slot, 356 emitted, `crates/c2-core`) and
**#150** (`expr-op-0x27` is worth 6). **The conversion is DECLINED.** What is
delivered instead is the diagnosis §9.17 asked for, the ordering rule measured
over three capture grids, and — the result — **the rule's refutation on the grid
it did not see.**

* **`Mismatch @ offset 8` is a LENGTH telescope, not a header defect.** One
  missing instruction word, surfacing at the first size-dependent field in the
  file.
* **WR1's address-last rule mis-emits 654 of the 728 captured cells that have a
  walk (89.8 %).** It is not a safe default for this construct.
* **A rule fitted to 360 in-sample cells mispredicts 98 of 394 out-of-sample
  cells.** Every miss is an r11 pre-save, on the one axis the first two grids
  could not vary. Had it shipped, the 878-TU differential would have read
  **6 match / 0 mismatch** over it — §9.17.7's blind spot, for the fourteenth
  time.

---

#### 9.19.1 Why offset 8 — and it is not where the bytes are wrong

§9.17.9 recorded `pn.cpp` → `Port=Mismatch @ offset 8` under the `ceiling` sink
and left it undiagnosed. Offset 8 is early enough to look like a header,
section-count or relocation consequence. It is none of those.

Reproduced at this base (`74d0744`) on a three-line probe, then both objs parsed:

| | c2 | port under `ceiling` |
|---|---|---|
| `NumberOfSections` | 5 | 5 |
| `NumberOfSymbols` | 15 | 15 |
| `.text` relocations | 1, sym 14, type `0x0006` | 1, sym 14, type `0x0006` |
| the four sections before `.text` | 132 / 152 / 16 / 16 B at 220 / 352 / 504 / 520 | **identical** |
| **`.text SizeOfRawData`** | **8** | **4** |
| `.text` words | `38840008` `4bfffffc` | `48000000` |
| **`PointerToSymbolTable`** | **554** | **550** |
| total | 891 B | 887 B |

The sink drops the offset, so codegen is handed `[Load(t)]`; `t` is formal 1,
argument slot 1 is r4, `t` is *already* in r4, and the port emits **nothing** for
the argument where c2 emits `addi r4,r4,8`. `.text` is one word short.

`.text` is the last section, so its 4-byte shortfall lands in
`PointerToSymbolTable` — and that field lives at **file offset 8..12**, ahead of
every byte of section payload (COFF header 0..20, five section headers 20..220).
**Offset 8 is simply the earliest byte in the file that can show that a function
got shorter.** The branch word also differs (`4bfffffc` vs `48000000`), but only
because it now sits at `.text` offset 0 instead of 4 — a consequence of the same
missing word, not a second defect.

Taken to the byte rather than asserted: `0..4` and `12..20` are identical, the
**316 bytes of the four sections that precede `.text` are identical**, and inside
the five section headers exactly **three** bytes differ —

| byte | field | ref → port | |
|---:|---|---|---|
| 196 | `.text SizeOfRawData` | 8 → 4 | the missing word |
| 204 | `.text PointerToRelocations` | 544 → 540 | the same 4 bytes, downstream |
| 217 | `.text Characteristics` | `0x60400020` → `0x60401020` | **not the sink** — see below |

— and the third is the `prefilter` `/Gy` confound of the note below, absent from
the differential's own emission, which is why `c2rs diff` reports offset 8 and
not 217.

The control that says the probe can also pass: the same probe with the member at
offset 0 (`pz.cpp`) reads **`Port=Match`** under the `zero` sink at this base.

**A methodological note worth keeping.** The first extraction of the two objs was
done with `c2rs prefilter --emit-obj`, and it reported a divergence at byte 217
on a body the differential grades `Port=Match`. `prefilter` derives
function-level linking from the flags (`/O1` implies `/Gy`) while `differential`
does not, so the port emitted a COMDAT `.text` (`0x60401020`) against the
reference's packed one (`0x60400020`). **`prefilter` is not a valid instrument
for byte forensics against an obj captured by another path**, and the reference's
own `S_OBJNAME` must be read out of its `.debug$S` and passed back as
`--obj-name` or the comparison measures the output path.

#### 9.19.2 The 356 does not age, and the workload still cannot grade it

Board quantities age (§9.17's A1 aged by 11,406), so it was re-measured before
anything was registered against it. 878-TU dc3 workload, at `74d0744`:

| | bodies | emitted | Δ emitted |
|---|---:|---:|---:|
| base | 706,402 / 2,462,571 (28.69 %) | 36,059 / 178,968 (**20.15 %**) | — |
| `C2RS_SINK_OFF_ADD_ARG=ceiling` | 707,873 (28.75 %) | 36,415 (**20.35 %**) | **+356** |

Identical to §9.17.5 to the function. Both runs read **6 match, 0 mismatch,
census/gate disagreement 0** — the ceiling sink provably mis-emits (§9.19.1) and
the differential is silent, which is why nothing in this section is graded on it.

#### 9.19.3 The capture grid, and the two constructs that are not the same construct

Three grids of c2's own `.cod` listing (`c2rs listing`, non-perturbing),
**754 in-domain cells**, crossed rather than sampled:
`scripts/slotarg_grid{1,2,3}.py`, read by `scripts/slotarg_read.py`, rule in
`scripts/slotarg_rule.py`.

Grid 1 (240 cells) — designator steps 1/2 × offset 0/8/0x8000/0x10000 × arity
1–5 × address slot × free/member caller. Grid 2 (120) — the base formal parked
**above** every slot the call writes. Grid 3 (402, 394 in domain) — the base in a
**middle** register, offsets straddling the 16-bit boundary from both sides,
arities 6–8, and a two-step designator straddling the boundary.

Four facts hold across all 754:

1. **Two designator steps are one `addi`.** `&t->s.k` at (0x7ffc, 8) emits a
   single add of 0x8004; the steps=1 and steps=2 cells are byte-identical
   wherever the sums agree. §9.17.5's "`-more` is an arity artefact" confirmed
   from the emitter side.
2. **A wide offset splits.** `k ≥ 0x8000` is `addis`+`addi`, and a zero low half
   collapses to a bare `addis` — the "wide literal with a zero low half" hazard,
   present and load-bearing.
3. **Offset 0 with the base already in the destination emits nothing** — #127's
   arm, reproduced here.
4. **The address is NOT emitted last.** WR1's rule agrees with c2 on **74 of the
   728 cells that have a walk (10.2 %)**; it mis-emits **89.8 %**.

And the fact that matters most for whoever takes #149:

**A computed address is not scheduled like a data-symbol address.** For the same
arrangement — the address at slot 0 under a two-word walk — c2 puts a *symbol*
address at walk index 1 and a *computed* address at index 2:

```text
  gs3(&gI, 3, 4)        li r5,4  · addi r3,r11,0 · li r4,3    <- SECOND  (§9.13.1)
  f3_0(&t->k, 11, 12)   li r5,12 · li r4,11 · addi r3,r3,8    <- its descending slot
```

`sym_slots_text` is therefore **not reusable** for the off-add, and
`a_computed_address_is_not_scheduled_like_a_data_symbol_address` is the portable
assertion that says so. Verified discriminating: mutating `sym_slots_text` back
to address-last reds it and one other test and leaves **87 green** — §9.12's pin,
a mutation that reddens everything identifies nothing.

#### 9.19.4 The rule agreed on 360 of 360, and that was worth nothing

A rule was refined against grids 1 and 2 until it reproduced all 360 cells
exactly — mnemonic, both registers and the immediate. Four refinements:
descending merge → "never the first setup word" for the wide form → "the low half
closes early when the base is clobbered" → the `mr`/`addi` vs `addis`
asymmetry. It looked finished.

**It is fitted.** Grid 3's predictions were generated and committed at `6caeddc`
*before* grid 3 was compiled, and scored **296 / 394 (75.1 %)** — below the
registered floor of 300.

**All 98 misses are the same miss.** Every one is an **r11 pre-save that the rule
did not expect**, and the rule predicted a pre-save correctly exactly once:

```text
  w_32764_3a_0_m1     (the base formal in r4 rather than r3)
    predicted   addi r3,r4,32764 · li r5,12 · li r4,11
    c2 emits    mr r11,r4 · li r5,12 · li r4,11 · addi r3,r11,32764
```

The axis is **the base formal's own register position**. Grids 1 and 2 always
parked the base at the lowest slot, so the clobbering `li` was always the *last*
walk word, and "hoist the address ahead of the walk" was never separated from
"hoist it ahead of the clobber". §9.13.1's third consequence, in a third costume:
*an axis the generator does not vary is exactly as invisible as a fixture that
does not arrange the case* — and this time the generator was written by a lane
that had just read that sentence.

It is also **the refusal `sym_slots_text` already carries**: "at two shifting
formals c2 pre-saves into r11 … which one probe does not separate". Here it fires
at **one** shifting formal, so the existing refusal's stated reason understates
its own scope.

**And it is not one refinement away — which is the part worth handing over.**
Over grid 3's 394 in-domain cells the pre-save fires 99 times, always inside the
298 cells whose base is clobbered, and one further axis nearly separates it:

| clobbered cells | pre-save | no pre-save |
|---|---:|---:|
| address destination **below** the base | **90** | 0 |
| address destination **above** the base | 9 | **199** |

Nine exceptions, not zero — and they are a coherent family: **wide offset with a
non-zero low half, destination exactly one register above the base, and the
base's own literal is the first word of the walk.**

```text
  w_32768_3a_2_m1     base r4, dest r5
    fitted rule   mr r11,r4 · li r4,11 · li r3,10 · addis r5,r11,1 · addi r5,r5,-32768
    c2 emits      mr r11,r4 · li r4,11 · addis r5,r11,1 · li r3,10 · addi r5,r5,-32768
```

So the pre-save arm is wrong twice over: about *when* it fires, and about *where
the computation goes once it does* — c2 interleaves the address into the walk
rather than appending it. **The last time this lane had a residue of 0 it was
wrong on the next grid**, and a residue of 9 on an axis discovered by the grid
that broke the rule is not a basis for a fifth attempt.

#### 9.19.5 DECLINE, under the rule registered in advance

`_2026-08-01-w-slotarg-prereg.md` registered: *if the measured rule cannot be
stated as a total function over the grid, it is refused, not fitted* — and the
grid-3 registration added *if O1 lands below its floor the rule is refused, not
patched again*. It did. **A fifth refinement against a third grid is fitting with
extra steps**, and the arithmetic says what it would have bought: the fitted rule
is right on **656 / 754 (87.0 %)** of everything captured, i.e. it would mis-emit
roughly one call in eight, silently, on a shape the workload differential cannot
see.

**So #149's stock is left unconverted: 356 emitted functions, still blocked.**

Two further constraints are recorded rather than worked around:

* **`SlotArg` is declared in `crates/c2-il`** (`func/mod.rs`, plus the
  `pub(crate)` twin in `func/body/mod.rs`), and lane `w-emitset` was live there.
  So even a proven rule could not have shipped end to end from this lane; the
  Δ emitted here is **0 by construction, not by measurement**, and it was
  registered that way before any of this was run.
* The port stays **honestly unable to represent the shape**. The exhaustive
  `match` in `the_computed_address_schedule_is_not_established_and_has_no_slot_variant`
  stops compiling the moment a variant is added, which is where the next lane
  will read §9.19.4.

#### 9.19.6 Pre-registration score — 9 of 13, and the two misses are the section

| | registered | measured | |
|---|---|---|---|
| G1 | c2 emits ≥ 200 of 240 grid-1 cells | **240** | HIT |
| G2 | ≥ 2 schedule shapes; 4, [2, 8] | **36** raw sequences in grid 1 alone | **MISS**, above the ceiling |
| G3 | ≥ 1 r11 pre-save; 30, [10, 150] | grid 1 **8**, grid 2 **0**, grid 3 **99** | HIT on the phenomenon, **MISS** on grid 1's interval |
| G4 | `k ≥ 0x8000` is not one `addi` | `addis`(+`addi`), zero low half collapses | HIT |
| G5 | `k = 0`, base in place, emits nothing | **YES** | HIT |
| G6 | the address's position differs slot-0 vs last | **YES** | HIT |
| S1 | Δ emitted from this lane | **0** — a constraint, declared in advance | — |
| S2 | address-last mis-emits ≥ 50 %; 65 %, [20 %, 90 %] | **89.8 %** | HIT, at the ceiling |
| S3 | portable assertions 5, [3, 10] | **2** | **MISS**, below the floor |
| S4 | the control stays green under every mutation | **PASS** — 87 green, 2 red | HIT |
| S5 | tip workload scan identical to base | **identical** | HIT |
| S6 | gate/selftest/sweeps unchanged | **unchanged** | HIT |
| S7 | verdict: partial — rule shipped, conversion declined | **rule REFUTED**, conversion declined | **MISS**, and it is the finding |
| O1 | out-of-sample 370/402, [300, 402] | **296 / 394** | **MISS**, below the floor |
| O2 | the failing axis is the middle-clobber one | **it is, all 98 of it** | HIT |
| O3 | boundary offsets need no new arm | **YES** | HIT |
| O4 | arity 6–8 needs no new arm | YES; arity 8 spills the base to the stack (8 cells, out of domain) | HIT |

* **O1 is the section.** It was registered at 92 % by a lane that had just
  watched its rule reproduce 360 of 360, and the honest floor it set is what
  turned a shippable-looking result into a decline. **The value of the
  out-of-sample grid was entirely in its being generated before it was seen** —
  had grid 3 been captured first, the fifth refinement would have been
  irresistible and the rule would have looked finished again.
* **G2's miss is the same error as §9.17's C1**: a shape count registered as a
  small integer when the quantity was a cross-product. The 36 raw sequences are
  reproduced by a rule with **4 ordering arms**, and 4 was the point estimate —
  the registration named the wrong noun, not the wrong number, and an interval
  on "shapes" could not have been right about either.
* **S3 under-delivered on purpose.** Five assertions were registered for a rule
  that would ship; two is what a refuted rule can honestly support, and inventing
  three more would be pinning a rule this section says is wrong.
* **S7 is a miss in the good direction.** The lane expected to ship the rule and
  decline only the conversion. It is declining both, and the reason is a
  measurement that only exists because the decline rule was written down first.

#### 9.19.7 Gate evidence

At `e9a56f5`, worktree branched from `origin/master` **609 commits behind**
(`4ea415a`) and reset to `master` `74d0744` before any work — the **fifth** lane
this week to meet that, and the third to have it be the first thing it found.
Cache addressed by its canonical main-repo path.

* `cargo test --workspace` — base `74d0744` **596 passed, 0 failed, 1 ignored**
  → tip **598 passed, 0 failed, 1 ignored**. Both measured, not inferred (base
  rebuilt from `git checkout 74d0744 -- crates`). **`#[test]` grep over
  `crates/` 597 at base → 599 at tip**, reconciling with the runner at both ends
  once the one `#[ignore]`d test is added (597 = 596 + 1, 599 = 598 + 1).
  **Target count 24 at base and 24 at tip** — no target was added, so the two new
  tests are in the lane `differential.rs` does not grade, which is why
  `scripts/gate.sh` is quoted separately below.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip**.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — 42,719 configurations × 12 lanes =
  **512,628 gradings, 0 mismatches**; 406 of 406 declared family pairs reached
  and emitted; refusal-frontier residue **0**.
* 878-TU workload scan at tip — **identical to base on every headline number**:
  6 match, 0 mismatch, 865 vocab-gap, 7 capture-fail; bodies 706,402 / 2,462,571
  (28.69 %); emitted **36,059 / 178,968 (20.15 %)**; census/gate disagreement 0.
* The two sink scans on the same binary, for the counterfactuals quoted above:
  `ceiling` emitted **36,415** (**+356**, and it mis-emits), `expr` emitted
  **36,065** (**+6**). Four 878-TU scans in total — base, tip, `ceiling`, `expr`.
* Probes — `work/pn.cpp` `Port=Mismatch @ offset 8` under `ceiling` and
  `Port=NotImplemented` under `zero`; `work/pz.cpp` `Port=Match` under `zero`.
* Grids — 240 + 120 + 402 cells, all emitted by c2; 8 arity-8 cells spill the
  base to the stack (`lwz r11,t$(r1)`) and are excluded from the rule's domain by
  name rather than dropped silently.

**No fixture was added and no `fixtures/cpp/` entry changed**, because nothing
shipped — §9.17.9's rule, and the same reasoning: a fixture for a shape the port
refuses would put a claim in every gate lane this lane did not earn.

**Reproduction.** The grids are generated by committed code
(`scripts/slotarg_grid{1,2,3}.py` → `c2rs listing <cpp> --out <cod>` → read with
`scripts/slotarg_read.py`, scored by `scripts/slotarg_rule.py`), so nothing here
depends on a scratch directory. The two one-off probes are three lines each and
are given in full rather than named, because §9.17.9's `pz/ph/pn.cpp` live under
a gitignored `work/` and could not be re-run from the section that cites them:

```cpp
// pn.cpp — Mismatch @ offset 8 under `ceiling`, NotImplemented under `zero`
struct S { void one(int*); };
struct T { int pad0; int pad1; struct { int k; } s; };
void a1(S* s, T* t) { s->one(&t->s.k); }

// pz.cpp — the control: Port=Match under `zero`
struct S { void one(int*); };
struct T { struct { int k; } s; };
void a2(S* s, T* t) { s->one(&t->s.k); }
```

#### 9.19.8 Board items

* **#149 stays open at 356, and its cost is now known.** The variant is trivial;
  the ordering rule is **not established**, and the next lane starts from
  `scripts/slotarg_grid{1,2,3}.py` (754 graded cells) plus the 98 witnesses that
  refuted the fitted rule. **Do not re-derive the rule from grids 1–2 alone** —
  they agree with a rule that is wrong one call in eight.
* **#155 — the r11 pre-save is a rule of its own, and it is under-scoped
  everywhere it is mentioned.** `sym_slots_text` refuses it at "two shifting
  formals"; grid 3 fires it at **one**, and 98 of 394 cells need it. It is the
  same object as board **#141** (`call-arg-sym-permuted`), which is sized off one
  probe. Both should be measured on one grid over (base register position) ×
  (walk length) × (wide/narrow offset), which grid 3 already is for the off-add
  half.
* **#156 — `prefilter` and `differential` disagree about function-level
  linking.** `prefilter` infers `/Gy` from `/O1`; `differential` does not. On the
  same source the two emit `.text` characteristics `0x60401020` and `0x60400020`,
  so a body the differential grades `Port=Match` reads `bytes-diverge at 217`
  through `prefilter`. Nothing shipped depends on it, but `prefilter` is the
  reject-only seam a caller is meant to trust, and one of the two is wrong about
  the workload's real flags.
* **#157 — a computed address whose base formal is passed on the stack.** Grid
  3's arity-8 cells lower it as `lwz r11,t$(r1)` and then compute from r11. Out
  of the modeled domain, 8 witnesses captured, named here so a later grid does
  not rediscover it as an anomaly.
* **#150 is closed at 6.** `expr-op-0x27` reproduces at this base to the
  function — **22,759 emitted, 407,016 bodies**, identical to §9.17.6 — and
  granting its named token converts **6 emitted functions**, re-measured here on
  the same binary (`C2RS_SINK_OFF_ADD_ARG=expr`: emitted **36,065** against the
  base's **36,059**). The board should carry **6**, not 22,759. The #1 blocking
  feature on the emitted board is also the least valuable thing on it, which is
  §8.7's rule about blocking-feature counts being queue positions rather than
  quantities of work.
