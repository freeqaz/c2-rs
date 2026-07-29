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
| `mvp_empty.cpp` | written here | **R1** — a TU that defines no functions; the smallest whole-TU byte-exact target (720 B, four sections, no `.text`) |
| `w10_empty_fn.cpp` | written here | **R2** — empty *function* bodies (`void f() {}` → a bare `blr`), the `body-0x3A` census bucket |
| `w5_chain.cpp` | written here | **W5 chains** — 3+-op `*`/`-` chains. This fixture caught a live mis-emit: the port reused one scratch where c2 descends `r11→r10→r9` |
| `w5_tree2.cpp`, `w5_tree3.cpp` | written here | **W5 trees** — `(a+b)*(c+d)` and deeper; still out of class |
| `w5_tree_neg.cpp` | written here | W5 negative neighbours — every function must keep returning `NotImplemented` |
| `mvp_fmul3.cpp` | written here | **Ported (W13a)** — `float fmul3(float,float,float)`; `fmuls f0,f1,f2 ; fmuls f1,f0,f3` |
| `w13_fabi.cpp`, `w13_fops.cpp`, `w13_fscratch.cpp`, `w13_fneg.cpp` | written here | W13 characterization: the FP calling convention, per-op encodings, the `[f0, f13…f1]` scratch cursor, and the negatives (constants, converts, contraction, spills) that must keep refusing. See `docs/CODEGEN_W13_FLOAT.md` |
| `il_convert_scalar.cpp`, `il_intrinsic_call.cpp` | written here | Characterization for `2C` (the real cast) and `40` (the intrinsic call, which is **not** a cast). Both replay `ByteExact` and must keep refusing. See `docs/IL_CAST_CONVERT.md` |

### Negative fixtures are not optional

Roughly half the files here exist to be **rejected**. `mvp_call_submod`,
`…_mulmod`, `…_widemod`, `…_twice`, `…_then_stmt`, `…_two_framed`,
`…_plus1plus2`, `…_argframed_plusk` and `w5_tree_neg` all pin the fail-closed
boundary: each is one small step outside an accepted class, and each must
report `NotImplemented` rather than bytes.

That discipline is load-bearing rather than decorative. A green corpus is only
as strong as its ability to *separate* the candidate rules — the W5 mis-emit
survived because every fixture up to `a-b-c` had exactly one intermediate,
where the single-accumulator and descending-register rules produce identical
bytes. When adding a class, add the neighbour that would look the same under a
plausible wrong rule.

Include-free is deliberate: no `e:\` include roots means no `WIBO_PATH_MAP` /
`WIBO_COMPUTER_NAME` string-hash determinism knobs are needed — the capture is
reproducible with a bare toolchain.

## Adding a fixture

Drop a self-contained `.cpp` into `cpp/`. `c2rs bench` picks up every
`cpp/*.cpp` automatically. Do not add headers or include paths without also
wiring the include/path-map handling in `c2-reference` — the current capture
path is include-free by design.
