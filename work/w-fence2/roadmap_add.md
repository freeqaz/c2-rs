
## 10.33 W-FENCE2 — the inline fence NARROWED on the decline side; `vsnprnc.cpp` MATCHES (19 → 20), and c2 is measured never to inline a callee over 80 bytes on 7,552 workload call sites (2026-08-09)

Rung: [`docs/rungs/2026-08-09-w-fence2.md`](rungs/2026-08-09-w-fence2.md).
Board rows **#2470**–**#2482**; **#2483**–**#2499** left explicitly unminted.
PREREG `work/w-fence2/PREREG.md`, frozen at `150d96af` before the first
`crates/` change, the first probe cell and the first fixture line. Base master
**`acb151ed`**, workload **dc3 `d7a3c1aa`**, both ends scanned with their own
binary.

**§10.29's fence was right as a safety property and it refused on the wrong
question.** `w-inlfence` shipped one clause — *a callee this TU also defines, of
which the port has no model, refuses the function* — reasoning that c2 cannot
inline a body it does not have. That closed a real latent wrong-emit. But it
answers *"could c2 have inlined?"* where an obj turns on *"did it?"*, and
§10.32's own frontier carried the price: `src/xdk/LIBCMT/vsnprnc.cpp`,
`fnbyte-exact 2/2`, **zero bytes of codegen distance**, and `vocab-gap`.

    TU match       19 -> 20        mismatch 0 -> 0      codegen-gap 0 -> 0
    vocab-gap     852 -> 851       capture-fail 7 -> 7
    fnbyte-exact  35,793 -> 35,793 (+0)    fnbyte-differs 1,898 -> 1,898
    census 712,280 -> 712,280 (+0)         emitted 39,226 -> 39,226 (+0)
    gap-metric keys 256 both ends: 0 vanished, 0 appeared, 3 changed
    per-TU verdicts BY NAME over 878 TUs: 1 changed, TOWARD acceptance

**The evidence is GRID-W and it is the decline side only.** For every IL call
edge to a callee its own TU defines, over all 878 TUs, the instrument asks the
**reference obj** whether the caller's `REL24` target set names the callee —
1,101 kept, 6,451 inlined, 0 unknown. **c2 inlines nothing above 80 emitted
bytes** (955 kept, 0 inlined at 96 B and up), 64–95 B is a **mixed** band, and
the port's own shippable input separates perfectly on a population of **one**:
`vsprintf_s → _vsprintf_s_l`, `ref=152 port=152`, the only site in the workload
with a lowerable locally-defined callee whose call c2 kept.

**What ships is a narrowing plus one measured constant, and the constant is
worth zero functions today.** `IlBundle::functions` stops refusing when the
callee's `.gl` defined record carries **plain external** linkage — `05`, and the
flags byte at `name_nul + 5` clear, which is the only thing in the IL that sees
`__forceinline` (GRID-K; F4 measured `__forceinline` inlining a 980-byte callee)
— and every segment is at `/O1`. `c2_core::comdat::INLINE_DECLINE_BYTES` (128)
replaces `splice::INLINE_UNBOUNDED_BYTES` (64) at the composition seam, changing
its meaning from *"the port can prove c2 EXPANDS this"* to *"the port cannot
prove c2 KEPT this"*. `splice`'s S7 is untouched.

**Two things this lane found that are not about `vsnprnc`.**

1. **The emitter had never resolved a `REL24` against a name its own obj
   defines.** The first narrowing produced a live `Port=Mismatch @ offset 12` —
   one extra 18-byte symbol record. The budgeted unnamed refusal, and its
   general form is that a parser narrowing admitting a shape the emitter has
   never emitted is **two** changes, the second invisible from the parser.
2. **§10.29's own decline D9 turned over.** *"A `__forceinline` cell would grade
   nothing"* was true under a wholesale fence; under a narrowed one it grades a
   **wrong obj**, and both of this lane's negative fixtures are realized wrong
   emits dumped from the reference obj rather than argued.

**And §10.32's factor-model false positive is closed without touching the
model.** `A∧B∧C∧(D∨E)` read 20 against a match set of 19; at this tip both are
20 and they are the same twenty TUs. The model needed no term; the gate needed a
narrowing.

Gate: **18/18 PASS**, 331/331 graded per lane, **0 mismatch anywhere**, 5,958
fixture-verdicts, sweep 19,460 graded / 0 mismatch, cross 90,424 / 0 mismatch;
`cargo test --workspace --release --no-fail-fast` **1,410 / 0 / 38 targets**
(base 1,406 / 38); `c2rs selftest` and `c2rs bench` **331 PASS / 0 ERROR**;
`board_audit.sh` five zeros; `rung_registry` 2 passed. `hatch-red` is REFUSED
by a **pre-existing** `HATCH-DRIFT` in `body/shapes/calls.rs`, reproduced at
master with this lane's `crates/` reverted (#2482, board #1406).
