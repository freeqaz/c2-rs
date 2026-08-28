# `w-s7` — the three cells that settle `sym+0x20` bit 12

Lane `w-s7`, 2026-08-28. [`../../WB_S7_FINDINGS.md`](../../WB_S7_FINDINGS.md)
§4.3; board **#3741**.

**The question.** Stage S7 (`FUN_10b7e032` @ `0x10b7e032`) gates four of its ten
passes — including both of its tuple splicers, `0x10c21b03` and `0x10b35c78` —
on `test DWORD PTR [eax+0x20],0x1000` at `0x10b7e03a`, over the **symbol**
record reached through `func+0`. **c2 never sets that bit**: the image holds 40+
sites that test it and **zero** that write it, so it arrives from the IL. These
cells identify the *source construct* that produces it.

**The discriminator is fixed by the gate's structure, not chosen after the
fact.** `after0` (`0x10b7e701`) is S7's unconditional call site, one per
function. `sched0` (`0x10b7e00c`) sits **inside** the range that `0x10b7dfea`
(`jne 0x10b7e017`) skips when the bit is **set**. Therefore:

    functions with the bit SET  ==  after0 - sched0

Both taps already exist — `DISCLOSURE.md` **W-STAGETAP-1** and **W-STAGETAP-3**.
**This grid adopts nothing and adds no site.**

| cell | what it is |
|---|---|
| `s7_ctl.cpp` | **control** — three ordinary functions, no EH construct at all. Establishes `sched0 == after0` on this grid's own compilation, so a split elsewhere is attributable to the construct |
| `s7_seh.cpp` | `__try`/`__except` and `__try`/`__finally`. **`ctl_a` is repeated verbatim from the control** so a per-function result is separable from a per-TU one |
| `s7_cxx.cpp` | C++ `try`/`catch`, also with `ctl_a` verbatim. Separates *EH* from *SEH*: this compiles at `/EHsc` and lowers real C++ EH |

## Result

Flags are the workload's own profile, `/O1 /Oi /EHsc /GS- /c` — `stage`'s
default.

| cell | functions | `sched0` | `after0` | bit set on |
|---|---:|---:|---:|---:|
| `s7_ctl.cpp` | 3 | **3** | 3 | **0** |
| `s7_seh.cpp` | 3 | **1** | 3 | **2** |
| `s7_cxx.cpp` | 2 | **2** | 2 | **0** |

1. **`__try` sets the bit** — two `__try` functions, two bits.
2. **It is per-function**, not per-compiland: `ctl_a` is byte-identical source
   in cells 1 and 2 and still reaches `sched0` in the TU where two other
   functions do not.
3. **C++ EH at `/EHsc` does not set it.** The construct is SEH.

**Obj-side, closing the loop from the read to the object:** `s7_seh.obj`
contains `__C_specific_handler`, `.pdata` and `.xdata`; `s7_ctl.obj` contains
**none of the three** — exactly what `0x10c21b03`'s body predicts
(`FUN_10c05869("__C_specific_handler")`, symbol kind `'S'` = `0x53`).

## Reproduce

```sh
for c in s7_ctl s7_seh s7_cxx; do
  cargo run --release -p c2-harness --bin c2rs -- \
      stage counts --fixtures docs/whitebox/grids/w-s7/$c.cpp
done

cargo run --release -p c2-harness --bin c2rs -- \
    compile docs/whitebox/grids/w-s7/s7_seh.cpp --keep-obj /tmp/seh.obj
strings -a /tmp/seh.obj | grep -E '__C_specific_handler|[.]pdata|[.]xdata'
```

`seh_probe` / `cxx_probe` are declared and never defined: these are `/c` cells,
never linked, and an unresolved extern is what keeps the `__try` body from being
folded away.

> **Why this grid exists at all.** The lane's first draft asserted it could not
> be built *"because no `wibo` is installed on this box"*, on the strength of
> `command -v wibo`. `wibo` **is** installed — as the sibling `../wibo` build
> that `CLAUDE.md` documents and `Toolchain::locate()` resolves — and the same
> lane's gate run had already driven the seam 7,038 times. The cells are tracked
> here rather than left in gitignored scratch so the next reader can re-run the
> measurement instead of re-deriving the claim.
