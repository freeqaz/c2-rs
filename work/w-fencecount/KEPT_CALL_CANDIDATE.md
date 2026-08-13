# A second cell that fires `fence-blocks-exact`, by the KEPT-call mechanism

Lane w-fencecount, measured after `7840efeb` landed. **Not shipped** — the
control the lane ships (`fixtures/cpp/wfcnt_fence_holds_exact.cpp`) already
reads `sole 1 / exact 1 / bodies 2 / first 0`, and a second fixture asserting
the same counter cell would be a duplicate control. This is recorded because
the two cells are exact on **opposite** mechanisms, and that difference is a
fact about the counter's input, not about this file.

## The difference

| | shipped cell | this candidate |
|---|---|---|
| callee | `static int wfcnt_leaf(int a)` | `static __declspec(noinline) int wfcnt_big(int a, int b)` |
| what c2 does with the call | **INLINES** it | **KEEPS** it |
| why the port's caller is exact anyway | mechanism **I** (`c2_core::splice`) reproduces the inlined result | `splice_body_why` declines at `S7-callee-noinline`, so `Selected::Tail` emits the branch c2 emits |
| the wrapper's obj-side shape | no `REL24` | one `REL24` against a name this TU defines |

`docs/rungs/2026-08-09-w-vsnprnc.md` §1's TU is the second shape: `vsprintf_s`
tail-calls `_vsprintf_s_l` and **c2 emits the branch**. So the shipped cell
fires the same counter cell through a mechanism the originating TU did not use.
Both are sound inputs to `fence_blocks` — the counter asks only *sole cause* and
*every emitted body exact* — and neither reading is stronger, but the counter
has now been exercised on only one of the two.

## The source, and what it measured

`work/w-fencecount/probe/c3.cpp`:

```cpp
static __declspec(noinline) int wfcnt_big(int a, int b) {
    return a + b;
}

int wfcnt_wrap(int a, int b) {
    return wfcnt_big(a, b);
}
```

`static` defeats `w-fence2`'s plain-external exemption (linkage `03`, not `05`)
so the gate refuses at `locally-defined-callee`; `__declspec(noinline)` clears
bit `0x40` of the `.gl` attribute byte, which `comdat::callee_is_one_c2_expands`
asks **ahead of** the size test, so the port does not predict an expansion c2
does not perform.

Captured through `c2rs gap` at the `tests/gate_cause.rs` `/O1` profile
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`), `--jobs 1 --no-cache`:

```text
class                vocab-gap
gate_causes          ["locally-defined-callee"]     (first: same)
fn_total / in_class  2 / 2
emit-emitted         2
fnbyte-denominator   2
fnbyte-exact         2
fnbyte-exact-relocated      1
fnbyte-calltarget-agree     2
fnbyte-shape|tail|fnbyte-exact   1
fnbyte-shape|plain|fnbyte-exact  1

locally-defined-callee   sole 1  exact 1  bodies 2  first-of-multi 0
gap-metric fence-blocks-exact:locally-defined-callee        1
gap-metric fence-blocks-exact-bodies:locally-defined-callee 2
```

`fnbyte-exact-relocated 1` and `fnbyte-calltarget-agree 2` are the part the
shipped cell cannot report: the wrapper's relocation **exists** and names what
c2's names, so "c2 kept the call" is read off c2's own obj rather than argued
from a size bracket. No `docs/whitebox/` bracket is adopted by either cell.

## The SIZE route is refuted for both cells

Building a static callee over `WB_INLINE_FINDINGS` F1's `(300,308]` ceiling
fails, and it fails for a reason no spelling fixes. Two probes, same profile:

| probe | `.ex` | `gate_causes` | stops at |
|---|---:|---|---|
| 36-step multiply chain (N4's shape) | 3,720 B | `[body-out-of-class, locally-defined-callee]` | `expr-op-0x0F` |
| 90-step alternating add chain `a += b; b += a; …` | 4,312 B | `[body-out-of-class, locally-defined-callee]` | `expr-op-0x0F` |

Both add a second cause and destroy the sole-blocker premise. The compound
assignment is what the IL body parser refuses, but rewriting it as a `return`
expression does not help either: a body large enough to clear 308 emitted bytes
is outside every class `codegen::select_function` accepts, and GRID-W's port
side stops at 152 B (`guard_chain_shared_tail`, the largest class the port
lowers anywhere on the workload). **No callee can be over c2's static ceiling
and inside the port's class at the same time**, which is why both cells move
c2's decision off size — one onto the attribute, one onto the inline itself.
