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
