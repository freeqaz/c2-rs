# Fixtures

These are the **inputs** to the differential harness. They are **C++ only**.

## Why only `.cpp`?

The whole point of c2-rs is that IL and obj are *never hand-maintained*. Both
are generated at test time by the reference toolchain (`cl.exe` + `c2.dll`
under wibo):

- The **IL bundle** (`_CL_*{ex,gl,sy,in,db}`) is captured on demand via the
  `/Bd /d2nop` early-abort trick (see the crate `c2-reference`).
- The **`.obj`** is produced by a normal `/Ox /GS- /c` compile.

Committing captured IL or obj would create a second source of truth that can
silently drift from the toolchain. So `.gitignore` excludes `_CL_*`, `*.obj`,
and `*.il`; only the `.cpp` under `cpp/` is tracked.

## Contents

`cpp/` — self-contained (include-free) translation units:

| File | Origin | Notes |
|------|--------|-------|
| `il_bool_materialization.cpp` | dc3-decomp il-fixtures corpus | signed/unsigned comparison → boolean materialization. **Ported (W6)** — `Port=Match`, 6/6 in class; see `docs/CODEGEN_W6_COMPARE.md` |
| `il_call_return.cpp` | dc3-decomp il-fixtures corpus | call / return / virtual-call shapes |
| `add3.cpp` | written here | tiny freestanding int functions; `select_max`/`shift_mask` are still out of class |
| `mvp_*.cpp` | written here | the MVP ladder: add/sub/mul chains, immediate folding, wide constants, tail calls, the framed non-leaf `g(a)+k`, and the empty TU |
| `mvp_call_seq.cpp`, `mvp_call_seq_neg.cpp` | written here | **Ported (#35 step 2, rung 1)** — Class A many-calls: a framed body with several calls and nothing live across them. Ten shapes in one TU (which also stresses the label counter); the negative half holds the Class B boundary (a formal read after the first call needs `r31`) and the multi-argument literal list. See `docs/CODEGEN_PPC_MVP.md` §"Class A many-call bodies" |
| `il_call_bound_neg.cpp` | written here | **Negative** — the call-bound-to-a-local form's two drifted gates. `int z = g(b + a); return z;` was a live wrong-bytes emit (c2 canonicalizes a commutative argument's leaves) and `int z = g2(a, c); return z;` panicked the census; one locator now (`docs/GAPS.md` §6 instance 9) |
| `mvp_empty.cpp` | written here | **R1** — a TU that defines no functions; the smallest whole-TU byte-exact target (720 B, four sections, no `.text`) |
| `w10_empty_fn.cpp` | written here | **R2** — empty *function* bodies (`void f() {}` → a bare `blr`), the `body-0x3A` census bucket |
| `w5_chain.cpp` | written here | **W5 chains** — 3+-op `*`/`-` chains. This fixture caught a live mis-emit: the port reused one scratch where c2 descends `r11→r10→r9` |
| `w5_tree2.cpp` | written here | **Ported (W5 trees, depth 2)** — `Port=Match`, all four shapes. Note the add-root register swap: with a `+` root the two children's registers are exchanged relative to every other root operator — characterized, not explained, so accepted at exactly this depth |
| `w5_tree3.cpp` | written here | **W5 trees, depth 3** — still out of class |
| `w5_tree_neg.cpp` | written here | W5 negative neighbours — every function must keep returning `NotImplemented` |
| `mvp_fmul3.cpp` | written here | **Ported (W13a)** — `float fmul3(float,float,float)`; `fmuls f0,f1,f2 ; fmuls f1,f0,f3` |
| `w13_fabi.cpp`, `w13_fops.cpp`, `w13_fscratch.cpp`, `w13_fneg.cpp` | written here | W13 characterization: the FP calling convention, per-op encodings, the `[f0, f13…f1]` scratch cursor, and the negatives (converts, contraction, spills, synthesized constants) that must keep refusing. `w13_fneg`'s `n_k_add`/`n_k_dadd` are **no longer negatives** — W13b made both byte-exact — but the file still refuses as a whole, because decode is all-or-nothing per TU. See `docs/CODEGEN_W13_FLOAT.md` |
| `w27_fp_reg.cpp`, `w27_fp_reg_qual.cpp` | written here | **Ported (W27)** — the FP argument register file, numbered over the **FP parameters alone**. `w27_fp_reg` holds the two live mis-emits this closed (`GAPS.md` §6 (6)/(7)) plus the sixteen cases promoted out of `w13_fparam_neg.cpp`; `w27_fp_reg_qual` is the `.sy`-kind-vs-tid boundary — a `const float` is still an FPR, a `float&` is **not**. See `docs/CODEGEN_FP_ARGS.md` §1
| `w28_fp_store.cpp`, `w28_fp_store_neg.cpp`, `w28_fltused_order.cpp` | written here | **Ported (W28)** — the FP store leaf (`stfs`/`stfd`), the fourth consumer of the sub-object designator. The positive file grades **both** register numberings in one instruction; the negative holds the conversion pair (`frsp` vs free) and the pooled literal. `w28_fltused_order` pins where `_fltused` goes in a MIXED TU, which is `GAPS.md` §6 instance 11
| `w29_fp_contract.cpp` | written here | **Ported** — `#pragma fp_contract(off)`, which is an optimization *word* and not a code shape. The only graded evidence that accepting `00200001`/`00a00001` compares the **port** against c2 under the pragma; the corpus-scale experiment compares c2 against c2. See `docs/OPT_MODE.md` §6.4
| `w8_cond_tail.cpp`, `w8_cond_tail_value.cpp`, `w8_cond_tail_neg.cpp` | written here | **Ported (W8)** — the port's **first conditional branch**: a two-arm conditional tail call, reduced from `?MemFree`/`?MemSize` in the frontier TU `src/xdk/nuispeech/xboxmem.cpp`. Pins the two `b` encodings (self-relative with no relocation vs section-start placeholder + `REL24`, board #191) and the fact that the epilogue is never materialized. The negative half holds the two `cflow-if-1` neighbours that must stay refused: fold **band 2** (one successor *is* the epilogue → `bnelr`, no branch at all) and the **tail-merge** (both arms calling the same callee inverts the layout, board #193). See `docs/CFG_SHAPE.md` §4 |
| `w9_rel_signed.cpp`, `w9_rel_unsigned.cpp` | written here | **W9 — the relation grid.** Every W8 fixture tests one cell: `v1 == 0` on a **pointer**, i.e. `Rel::Eq` on an unsigned operand against the literal 0. So five of `branch_sense`'s six rows, the entire `cmpwi` path and every non-zero comparison immediate were written, unit-asserted **against the port's own table**, and never graded by the real `c2` (`docs/STATUS.md` trap 5). These two files grade all twelve cells — six relations × signed/unsigned — and add the port's first oracle witness for `bt` (`BO=12`) and for the `LT`/`GT` CR bits. Lane w-frame's ranking measured `bt` as the **most-wanted construct on the whole FRONTIER** (8 of 17 TUs) and `cmpwi` second (6 of 17). All twelve came out `Port=Match` on the first differential run |
| `w9_cmp_zero_le.cpp` | written here | **W9 — the comparison ZERO folds, found by a coverage sweep** (`work/w-frame/sweep.py`), not by reading. Signed `a <= 0` (`neg`/`orc`/`srwi31`, and the only `orc` the port can emit) was the last emission production in `crates/c2-core/src/codegen/` that no graded build had ever executed; its scratch register is `11` at `/O1` and `10` otherwise, a rule fitted from `>`'s witness with none of its own. Both variants reproduce. `w6_rel_k.cpp` could not have caught it: all twenty of its bodies compare against a **non-zero** literal, so it drives the general spines and never the folds — a fixture family can be thorough on the axis it varies and blind on the one it holds fixed. The four unsigned bodies also measure that **c1xx canonicalizes `a > 0u` / `a <= 0u` / `a < 0u` / `a >= 0u` upstream**: their source spelling is in the corpus and `leaf/compare.rs`'s four `(Rel::X, unsigned)` zero-fold arms still never execute |
| `w13b_fconst.cpp` | written here | **Ported (W13b)** — the minimal one-constant witness: one `.rdata` COMDAT, `addis`+`lfs`, a REFHI/REFLO+PAIR relocation quad |
| `w13b_fdedup.cpp` | written here | **Ported (W13b)** — dedup keyed on `(bit pattern, width)` so a `float` 1.0 and a `double` 1.0 are two constants; symbol placement (each pair goes right after the symbol of the function that *first* references it); and the relocation-layout bug it caught — a section's relocations follow **that section's own** raw data, invisible while `.text` was last |
| `w13b_fpool.cpp` | written here | **W13b negative** — bodies whose IL carries 2+ FP literals. c2, not c1xx, evaluates FP constants: `a*2.0f*b*3.0f` reassociates to `(a*b)*6.0f`, `a/3.0f/7.0f` to one `fmuls` by 1/21. `ke` is also the witness that a constant claims its FP register *before* any interior temporary |
| `w13b_ffold.cpp` | written here | **W13b negative** — the identity folds. `a+0.0f`, `a*1.0f`, `a-0.0f` are a bare `blr` with nothing pooled; `a*0.0f` is **not** folded and must keep emitting. The gate is per `(operator, value)` pair, and only a fixture holding both halves separates that from the wrong rule "refuse the value 0.0" |
| `il_convert_scalar.cpp`, `il_intrinsic_call.cpp` | written here | Characterization for `2C` (the real cast) and `40` (the intrinsic call, which is **not** a cast). Both replay `ByteExact` and must keep refusing. See `docs/IL_CAST_CONVERT.md` |
| `il_intrinsic_nullary.cpp` | written here | **`0x40` negative** — the *zero-argument* intrinsics (`__debugbreak`→`twi 31,r0,22`, `_ReturnAddress`→`mflr r3`, `__mftb`). The only shape that can decide `40 <TYPE>` against `40 <TYPE> <varint>`: with no arguments the `4C` apply sits immediately after the result type, so a trailing field would swallow it. `n_notintrinsic`/`n_lwsync` are the separating negatives for "any unknown extern becomes a `0x40`" — they are ordinary calls |
| `il_intrinsic_bits.cpp` | written here | **`0x40` negative** — 17 arithmetic/bit intrinsics with their ids and exact expansions, plus four near-misses (`fabsf`, `sqrtf`, `_rotl16`, `_MulHigh`) that are *not* intrinsics. Separates "the id is per name family" (`abs` and `labs` share id 15) from "per signature", and shows why even the one-instruction cases are unlowerable: the destination register is chosen by the consumer |
| `il_intrinsic_layout.cpp` | written here | **`0x40` negative** — the class-layout family (2113…2119), **86 % of the whole bucket**. Separates 2113 from 2114 at the *same* descriptor and *same* offset literal — the difference is the null guard, one instruction against five plus a branch; shows 2115's offset is not pre-negated (the id is); and pins `0x66`'s second byte as an **arity** (`66 02` vs `66 03`), not the constant it was recorded as. `l_dyn` pins 2119 = `dynamic_cast` |
| `il_intrinsic_fold.cpp` | written here | **`0x40` negative** — c1xx does *not* fold intrinsics, and c2's rule splits by intrinsic: `abs(-5)`→`li r3,5` and `_rotl(1u,4)`→`li r3,16` fold, while `fabs(-1.5)` and `sqrt(4.0)` **do not** and pool the unfolded constant. Also the first captured *nested* `0x40` (`abs(abs(a))`), settling an open question |
| `il_intrinsic_byval.cpp` | written here | **`0x40` negative** — selectors 222/223, the two that stay UNNAMED. Pins the trigger (a non-trivial **copy constructor**, not the destructor and not `/EHsc`) and the literal (`sizeof(class)`: `04` for 4 bytes, `0c` for 12) without claiming which of the pair does what, and names the fixture that would separate them. See `docs/IL_INTRINSIC_CALL.md` §6 |

Fixture gate as measured at commit **`cebfb88`** (W13b): **21 of the 41 tracked
`cpp/*.cpp` `Port=Match`, 0 mismatch**, `cargo test --workspace --release` green
(202 tests). Concurrent sessions have since added fixtures and rungs
(`06d29b9`, `db3b5ad`, `61e0d85`), so both the numerator and the denominator have
moved — re-run `c2rs diff` rather than quoting this line as current.

**Not yet described in the table above**: `w6_rel_k.cpp`, `il_call_args1.cpp`,
`il_call_args2.cpp` and `il_call_multi.cpp`, added after `cebfb88` by the sessions
that landed W6's non-zero-literal relations and the call-argument
characterization. Left for those sessions to document rather than guessed at here.

### Negative fixtures are not optional

Roughly half the files here exist to be **rejected**. `mvp_call_submod`,
`…_mulmod`, `…_widemod`, `…_twice`, `…_then_stmt`, `…_two_framed`,
`…_plus1plus2`, `…_argframed_plusk`, `w5_tree_neg`, `w13b_fpool` and
`w13b_ffold` all pin the fail-closed boundary: each is one small step outside an
accepted class, and each must report `NotImplemented` rather than bytes.

That discipline is load-bearing rather than decorative. A green corpus is only
as strong as its ability to *separate* the candidate rules — the W5 mis-emit
survived because every fixture up to `a-b-c` had exactly one intermediate,
where the single-accumulator and descending-register rules produce identical
bytes. When adding a class, add the neighbour that would look the same under a
plausible wrong rule.

W13b produced three more instances of exactly that, worth naming because each
wrong rule matched the entire pre-existing corpus:

- **"refuse the value 0.0 or 1.0"** vs **"refuse the `(operator, value)` pair"** —
  separated only by `w13b_ffold::q5` (`a * 0.0f`), which really does pool a zero
  and multiply.
- **"allocate FP registers in emission order"** vs **"the constant allocates
  first"** — separated only by `w13b_fpool::ke`, a body with a constant *and* an
  interior temporary. Every single-operator body matches both.
- **"relocations follow all raw data"** vs **"relocations follow their own
  section's raw data"** — indistinguishable until a `.rdata` sat behind `.text`,
  i.e. until `w13b_fdedup` existed.

Include-free is deliberate: no `e:\` include roots means no `WIBO_PATH_MAP` /
`WIBO_COMPUTER_NAME` string-hash determinism knobs are needed — the capture is
reproducible with a bare toolchain.

## Adding a fixture

Drop a self-contained `.cpp` into `cpp/`. `c2rs bench` picks up every
`cpp/*.cpp` automatically. Do not add headers or include paths without also
wiring the include/path-map handling in `c2-reference` — the current capture
path is include-free by design.
