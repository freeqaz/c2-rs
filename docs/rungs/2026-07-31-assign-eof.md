# WAE — `assign-dst-not-formal` declined at 13,887 rows and a measured worth of 0, and the `:eof` suffix that ranked it

    Tag:       WAE
    Slug:      assign-eof
    Date:      2026-07-31
    Fixtures:  wae_assign_dst.cpp wae_assign_dst_neg.cpp
    Census:    691,744 unchanged (28.09 %) — the widening is DECLINED; its measured worth is 0
    Record:    this document; work/ASSIGN/ESTIMATE.md (pre-scan), work/dc3-workload/scan-assign.jsonl

`assign-dst-not-formal:eof` was the ranked next rung at **13,887 functions** on
the post-WVB workload scan. Two things were believed about it and neither is
true: that its `:eof` suffix made every function under it grammar-complete, and
that the destination class it excludes is worth admitting.

Its measured worth is **0 functions**. Not "smaller than it looks" — zero, under
a counterfactual that deletes the gate outright and a second that deletes the
gate *and* the check behind it. What this rung ships instead is the instrument
fix: the gate now speaks **last** rather than first, so the 13,887 functions are
filed under the constructs that actually block them, and a latent mis-emit found
on the way is closed at a cost of 0.

## What it admits, and what it refuses

**Nothing new is admitted.** The rule is unchanged: an assignment destination is
folded away only on positive evidence that it is register-resident — a formal
from `.ex`'s `2D` list, or an automatic, plain, unqualified `int` local whose
address is never taken, from `.sy` (`sy.rs`'s `admissible`). A global, either
flavour of `static`, a qualified or non-`int` local, or a local whose address
escapes is refused, because folding its store is a silently dropped write.

Three things changed, all inside `crates/c2-il/src/func/body/shapes/assign.rs`:

1. **The refusal is deferred to the end of the parse.** It is recorded on the
   offending `26` push and raised only after the remaining statements, the
   returned expression, the return plumbing and the four post-substitution gates
   have each had their say. The accepted set is bit-identical by construction —
   a body that reaches the end with `deferred` set still refuses — and measured
   identical: census 691,744 before and after.
2. **The same test now applies to the right-hand-side-is-a-call route**, which
   skipped it entirely. See "the hole" below.
3. **The key is `assign-dst-not-formal-0x26`**, raised at the byte it is about,
   replacing `assign-dst-not-formal:eof`. See "the suffix" below.

## Estimate vs outcome

Written down before the measuring scan, verbatim in `work/ASSIGN/ESTIMATE.md`:

* **Hard ceiling 439** — the `cflow-straight+expr-modeled` rows, the only ones
  whose bodies are otherwise fully modelled.
* **Point estimate 200**, census 691,744 → 691,944.
* **Bias stated in advance: HIGH.**
* **Pre-registered decline threshold: a realized delta under 500.**

**Outcome: 0.** Estimate-to-realized ratio is undefined (200 → 0); the estimate
was high by its whole magnitude, in the predicted direction, and the ceiling it
was derived from was itself 439× too generous. The decline threshold was
pre-registered and is met by a factor of ∞.

The reason the ceiling was wrong is worth more than the number. `+expr-modeled`
says the *control-flow scanner's* expression model covers the body. It does not
say the recognizers do, and the two are different vocabularies — 438 bodies read
`+expr-modeled` and **not one of them** parses when the destination gate is
lifted. A cross-tab against the cflow axis bounds a row from above and is worth
computing; it is not a proxy for the recognizer, and this rung is the cleanest
counterexample the series has produced.

### Counterfactual A — delete the gate

Destination check removed, rebuild, rescan all 878 TUs:

```
  FUNCTION CENSUS (P2b): 691744/2462571 functions in class (28.09%)   <- identical
```

**+0.** And the residue is where the finding is: the key does not merely shrink,
it **empties**, with every row landing on a blocker that was always there.

```
  assign-dst-not-formal      13,887  ->  0
    8,221  assign-store-type-0x86    the very next line — a 4-byte non-integer store
    1,906  assign-store-type-0x82    a 1-byte store
    1,364  expr-jump                 a goto / break / continue / loop exit
      830  expr-op-0x27
      789  expr-call-in-expr-recv-load-then-call-op-0x64
      468  expr-call-in-expr-recv-load-then-call-data-addr-and-deref-load-more
      232  expr-call-in-expr-recv-intrinsic-this-adjust-then-intrinsic-call
       77  across 26 further keys
```

**Not one workload function has the destination as its only blocker.**

### Counterfactual B — delete the gate *and* the check behind it

A is only sound if `assign-store-type` is not just the same fact restated. It
is: a destination whose type this class will not fold also fails the store-type
check one line later. So B lifts both — any TYPE is stepped over — and rescans:

```
  FUNCTION CENSUS: 691744/2462571 (28.09%)   <- identical again, +0
    -8,221  assign-store-type-0x86     ->  +4,034 expr-jump, +3,855 expr-op-0x60,
    -1,906  assign-store-type-0x82         +809 recv-load-then-call-other, +722 expr-op-0x10
```

**Two independent refusals lifted, still +0.** That is the "count the
independent refusals between ceiling and emitter" discipline run to completion:
there are at least three, and the third is the whole expression and control-flow
layer.

### Why it was always going to be 0 — the emitter that does not exist

The refused destinations split in two, and the split decides the rung:

| destination | why it refuses | emitter |
|---|---|---|
| global, file-scope `static`, function-scope `static` | a memory object; the store needs a data-symbol relocation | **does not exist, and must not be faked.** `int f(void){ return gv; }` is out of class (`expr-out-of-class-bare-nonformal`) and `int* f(void){ return &gv; }` is out of class (`expr-op-0x41`) — the port emits no data relocation at all |
| `volatile` local, address-taken local | must not be coalesced | correctly refused, permanently |
| 4-byte register-resident non-`int` local (`unsigned`, pointer) | `.sy`'s type-id gate | **exists** — folding emits nothing, exactly as for `int` |

Only the last row is a candidate widening, and it is the one that does not
appear: a pointer local's body contains pointer expressions, which land it in
the 5,011 `cflow-straight` rows rather than the 439, and it hits
`assign-store-type-0x86` regardless. Counterfactual B lifted that too and still
converted nothing.

This had been measured once before and not carried forward. `ROADMAP.md` §6
(the 2026-07-30 ordering re-check, around line 1204) records
`assign-dst-not-formal` 5,534 → 5,533 for
**+0** and states the reason in one line — *"the three TUs the bucket pointed at
turned out not to want locals at all — their destinations are member or global
stores, correctly out of class"*. The row was then re-entered into the ranking
by `docs/rungs/2026-07-31-offset-run.md`'s found-and-not-taken table at
**+13,350**, on size alone. **A row that has already measured 0 needs a reason
before it is re-ranked, not a bigger number.**

## The suffix — `:eof` was a rendering, not a fact

`assign-dst-not-formal:eof` was ranked *because* of its suffix: a census key
ending `:eof` is a refusal raised after the parse reached the end of the
segment, so every function under it is grammar-complete by construction and
nothing can be hiding behind it. `expr-out-of-class-bare-nonfirst-formal:eof`
is the precedent where that reasoning held, at 43,319 functions and ±700.

It does not hold here, and the mechanism is worth stating exactly because it
generalizes. `Block::feature` (`body/mod.rs:565`) renders **any** block with
`byte: None` and `aux == 0` as `<ctx>:eof`. It has no access to the segment, so
it cannot tell "the parse ran out of bytes" from "a predicate said no and the
author passed `None`". This site passed `None` at `off: probe` — the byte
*after* the destination token, in the middle of the segment. The suffix was
therefore a property of the constructor call, not of the parse.

The data says so directly: **4,466 of the 13,887 rows are `cflow-loop`**, and a
loop body is not grammar-complete under any reading. Reproduced from
hand-written source before any sizing was believed (`work/ASSIGN/probe/p1.cpp`,
7 of 9 functions land in the key) — the rule that has now caught three wrong
sizings.

Fixed narrowly, inside this seam: the block carries `byte: Some(0x26)` at the
destination push. One key, not a shard per right-hand side; the hex window
centres on the `26`; and the suffix is gone rather than made honest, because
making it honest is a change to a shared renderer that every recorded census key
in `docs/` reads through.

**The general defect is NOT fixed and is left named.** Every other hand-built
`byte: None` block prints `:eof` on the same terms — `assign-subst-overflow`,
`assign-ret-nonformal`, `assign-repeated-leaf`, `assign-noncanonical-order`,
`expr-repeated-leaf`, `fn-varargs`, `lo-marker`, `param-width-undetermined`, the
`callee-unresolved-*` family, `opt-mode`, and everything
`straight_line_out_of_class_ctx` returns. Some of those genuinely are at the
segment end; the rendering cannot tell you which, and the ranking has been
reading it as if it could.

## The hole — a destination test at one of two sites

Found while characterizing, and it outranks the rung. The
right-hand-side-is-a-call route

```rust
if rhs_is_call { if env.is_empty() { return parse_call_shape(seg, &mut q, lo, Some(dst)); } }
```

hands `dst` to the call shape as a bound token **without ever asking what `dst`
is** — the one site in the file that does not consult `.sy`. So

```cpp
extern int gv;  int g(int);
int f(int a) { gv = g(a); return gv; }
```

censused **1/1 in class** as an `int-tail-call`, with the store to `gv` folded
into thin air: precisely the defect `.sy` was built to stop, at the site that
did not use it. This is the `GAPS.md` §6 shape — *one fact, one locator* — with
a second consumer that never asked.

It was never a live mis-emit. `IlBundle::functions` refuses any TU whose `.gl`
carries a mangled run that no function record claims and no body resolved as a
callee (`fixtures/cpp/il_gl_data_symbol.cpp`), and a body that stores to a
global necessarily puts that global in `.gl`. `c2rs diff` reads
`ReferenceReplay=ByteExact  Port=NotImplemented` and there is no mismatch
anywhere in this rung. **But a function-level class must not be sound only by a
translation-unit accounting rule about something else**, and the census — which
is the widening order — was counting such functions as in class.

The fix is the same deferral: parse the call first, refuse on the destination
after. Placing the check *before* the call parse was measured and rejected —
**14,454 workload functions reach that branch with a destination the class will
not fold, and 0 of them parse**, so an early check would have invented a
14,454-row census bucket naming none of its own contents, which is the exact
defect this rung exists to remove. Deferred, the cost is **0 functions**.

## Gate evidence

Every lane run from this worktree, with `cargo build --release` immediately
before each, on `wt-assign-eof` over master `032247e`. Every pinned lane reports
**`sha 510b338d7525`**, and `sha256sum target/release/c2rs` on the final tree is
the same twelve digits — so the binary the sweeps graded is the binary this tree
builds, checked rather than assumed (the last commit is a doc comment, and a
doc comment producing an identical binary is a fact to verify, not to trust).

| lane | result |
|---|---|
| `cargo test --workspace --release` | **513 pass, 0 fail** |
| `c2rs bench` | **195 pass, 0 fail, 0 error** |
| `c2rs selftest` | **195 PASS, 0 fail/error** |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **92 / 90 / 90 / 90 match, mismatch 0** in all four |
| `scripts/cross_sweep.sh` | **27,956 configurations × 4 lanes, 0 mismatches** (13,707 TUs graded, 9,577 matched) |
| `scripts/expr_sweep.sh` | **13,707 cases, 0 mismatches** |
| 878-TU workload scan | match 6, **mismatch 0**, census **691,744 / 2,462,571 = 28.09 %** (unchanged), **disagreement 0** |
| fixtures, `c2rs census` | `wae_assign_dst.cpp` **5/5**, `wae_assign_dst_neg.cpp` **0/6** |

The negative fixture's six functions all land on `assign-dst-not-formal-0x26`,
and its last one — `wae_neg_call_global` — is the closed hole: it censused
**in class** before this rung.

The under-claiming direction, which nothing else tests, is the point of
counterfactual A: the gate's over-refusal is not "small", it is exactly zero
functions on 2,462,571.

## Found and not taken

| item | size | what stops it |
|---|---:|---|
| `assign-store-type-0x86` | **+8,221** (new; a 4-byte non-integer store) | the same fact one line down. Counterfactual B lifts it for **+0** — it is a *restatement* of the destination gate, not a rung. Do not rank it |
| `assign-store-type-0x82` | **+1,906** (new; a 1-byte store) | also **+0** under B, and not free even in principle: a store to a `char` local truncates, so folding it is a value change, not a copy |
| a store to a data symbol | the whole memory-object half of the row | **no data relocation exists anywhere in the port.** `return gv;` and `return &gv;` are both out of class, and `il_gl_data_symbol.cpp` refuses the TU besides. This is a codegen rung (two instructions, two relocations, a `.data`/`.bss` section and the `.gl` accounting rule) and nothing about the assignment class shortens it |
| 4-byte non-`int` register locals in `.sy` | 0 measured | `sy.rs`'s `admissible` requires `tid == TID_INT`; widening it to `unsigned`/pointer is a two-line change and converts nothing, because the bodies that would use it block in the expression layer. Re-run counterfactual B if the pointer operand vocabulary ever lands |
| `expr-jump` | **+1,364** here, on top of its own row | the control-flow layer |
| a portable-lane regression guard for the deferral | 0 tests | `assign.rs` carries no `mod tests` and `func::test_fixtures` has no assignment segment with a non-formal destination. The behaviour is graded by two toolchain fixtures and 2,462,571 workload functions, which is stronger evidence — but the no-toolchain lane has none of it, and manufacturing a synthetic `.ex` segment for it would be a fixture nobody captured. Needs a real segment dumped out of `wae_assign_dst_neg.cpp` |
| the `:eof` rendering, everywhere else | unknown, and that is the finding | `Block::feature` cannot distinguish a real end-of-segment from a hand-passed `byte: None`. Every `:eof` key in every ranking table is one or the other and no reader can tell. Fixing it means giving `feature` the segment length — one shared renderer, every recorded key — so it is named here and left for a serial merge, not done in a parallel seam |

### The riskiest thing left unmeasured

**Whether the deferral has moved a refusal that some other rung is ranking.**
The 13,887 rows redistribute into 33 keys, four of them by more than 200, and
`expr-call-in-expr-recv-load-then-call-data-addr-and-deref-load-more` (+468)
sits in the data-address family another lane is working in this session. The
census total is provably unchanged (+0, and the movers sum to +0), so no rung's
*yield* is affected — but a row's *size* is, and any sizing taken from
`scan-post-wvb.jsonl` for those keys is now 0.02–3 % low. `scan-assign.jsonl` is
the current one.
