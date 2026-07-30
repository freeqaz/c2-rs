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

Adversarial review then found the likely *mechanism*, and it is one field:
`read_record` reads a record's size as a `u16` where the stream carries a **varint**
(the same form the function already reads correctly for `static` records, under a
comment warning about exactly this mistake). One `char buf[128]` anywhere in a
translation unit desyncs the record and unbinds that whole file. So the 567,549 is
not an irreducible reader gap — it is largely one encoding error, and the sibling
count of `param-multi-reg = 1` is also an artifact of it: a 16-byte class with a
copy constructor is recorded as a 4-byte *pointer* (passed by hidden reference,
**one** register) and the mis-read yields 2052, which trips the `> 8` test. That
means the key does not currently mean what its name says.

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
