# ROADMAP — from the MVP port to a fully implemented stack

Status: living document (written 2026-07). Describes where the port stands,
what is missing, and the ordered plan to close it. The invariants at the
bottom are **load-bearing** — every phase must preserve them.

## 1. Where we are

The foundation is proven and fast; the port itself is deliberately narrow.

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
  (`c2rs diff`, 17/31 fixtures Match, 0 mismatch): straight-line integer
  add/sub/mul chains with immediate folding and wide constants (now including
  3+-op `*`/`-` chains, see the mis-emit note below), multi-function TUs of
  those, bare void tail calls, integer tail calls `return g(<arg>)`
  (passthrough / `+0` fold / arg-setup), the framed non-leaf
  `return g(a) + k` (6-section obj with `.pdata`), **the empty TU** (R1), and
  **comparison→boolean leaves** (W6: `return a <rel> k`, branchless). Everything
  else returns `NotImplemented` — fail closed, never a guess.
- **R1 — the first nonzero match bucket.** `coff::emit_empty_obj` emits the
  720-byte four-section obj for a TU that defines no functions, recognized
  *positively* (`is_empty_module`: `.ex` carries neither a `LO` body marker
  nor a `4F 1F` function start). Gap scan: **match 0 → 5 of 878 TUs**. Two of
  the seven zero-function TUs are deliberately refused — they carry a stray
  `4F 1F` after the module end — a conservative miss, taken in preference to
  relaxing a fail-closed test.
- **W6 — comparison → boolean, byte-exact.** `il_bool_materialization.cpp` is
  `Port=Match`, 6/6 functions in class. c2 lowers these **branchlessly** (no
  `cmpw`/`cmplw` at all) via carry-bit and bit-extraction idioms; the `k == 0`
  folds are mandatory and dispatched first. Full byte evidence, CONST/DERIVED
  split and fail-closed negatives: **`docs/CODEGEN_W6_COMPARE.md`**. `<`, `<=`
  and `>=` against a **non-zero** literal stay out of class — the spine's
  instruction order for a literal lhs is unresolved and guessing it would be a
  silent wrong-bytes emit.

- **W5 chains** — `*`/`-` chains past two operations, allocating temporaries
  down the `r11 → r10 → r9 …` cursor and **refusing below `r9`**; the rule and
  its eleven negative neighbours are in **`docs/CODEGEN_W5_SCRATCH.md`**. This
  is the change that fixed the mis-emit below. Expression *trees* still fail
  closed.
- **Harness + tooling**: oracle self-test, corpus generator, obj→IL retrieval
  baseline, IL-space search prototype, edit gate; `perf`/`perf-scale`
  (~200–290× per obj, ~897k obj/s at 32 threads vs ~3.1k for real c2).
- **P2 / P2b measurement** — `c2rs gap` (real-workload TU scan) plus the
  function-level census (`c2rs census <cpp>`, and the scan-wide coverage +
  blocking-feature histogram in the `gap` report; the census also prints a
  bracketed hexdump of the bytes at each blocking site, which is what turned
  guessed opcode names into measured ones). The port's real coverage is now a
  *measured* number: **78,028 / 2,462,571 functions in class (3.17%)** over the
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

- `w10_empty_fn.cpp` — **empty function bodies**, the function-level analogue
  of R1 and the `body-0x3A` census bucket (4.4% of blocked functions; by name
  mostly STL plumbing and trivial destructors). No expression to select, so it
  is reachable with no new instruction selection.
- `w5_tree2.cpp` / `w5_tree3.cpp` / `w5_tree_neg.cpp` — multi-scratch
  expression **trees** (`(a+b)*(c+d)` and deeper). Chains are done; trees
  still fail closed. The register-allocation rule, the evaluation order and
  the eleven negative neighbours are fully characterized in
  `docs/CODEGEN_W5_SCRATCH.md`.
- `select_max`, `shift_mask` in `add3.cpp` — ternary select, shifts, `&`.
- `mvp_fmul3.cpp` + `float_leaf_neighbors` test — float arithmetic leaves;
  the IL side already parses, codegen has no float registers.
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
| W5 | **Multi-scratch expressions** (tree depth > 2, e.g. `(a+b)*(c+d)`) | COLOR scratch order beyond r11; operand-stack depth limit lifted | **chains DONE** (`r11→r10→r9…`, refused below r9); trees test-pinned reject — `CODEGEN_W5_SCRATCH.md` |
| W6 | **Integer comparisons → bool** (`x!=0`, `x>7`, signed/unsigned) | branchless carry/bit-extraction spines (**no** `cmpwi`/`cmplwi`), `subfe`/`addze`/`cntlzw`/`rlwinm`, mandatory `k == 0` folds | **compare-against-literal leaves DONE**, 6/6 in `il_bool_materialization.cpp`; `<`/`<=`/`>=` vs a non-zero literal still refused — `CODEGEN_W6_COMPARE.md` |
| W7 | **Shifts + bitwise** (`<< >> & \| ^ ~`, mul-by-const strength reduction) | `slw`/`srw`/`sraw`/`rlwinm`/`andi.` (the dot!); non-commutative hazard list grows | `shift_mask` |
| W8 | **Control flow** (if/else, ternary select, loops) | multi-block `.ex` (labels, branch tokens), `bc`/`b`, block layout order, compare+branch fusion | `select_max`, `il_call_return` conditionals |
| W9 | **Division / modulo** | `divw`/`divwu`, remainder via `mullw`+`subf`; const divisor → multiply-high | — |
| W10 | **General frames + locals** (spills, local temps) | frame-size model beyond the fixed 96B, `lwz`/`stw` to frame, `.pdata` generalization; **must solve the `.pdata` label-counter shift (W-UNW-1)** | `cached_return` |
| W11 | **Calls generalized** (multi-arg, stack args, multiple calls/externals, multi-fn TUs with calls, calls in expressions) | arg registers r3–r10 + stack spill, call sequencing, `.pdata` per function (W-UNW-1 again), extern pairing beyond the single-external shortcut | `il_call_return.cpp` |
| W12 | **Memory / struct access** (pointers, member loads, stores, arrays) | `lwz`/`lhz`/`lbz`/`lha` + sign/zero extension, store forms, member-offset addressing — the `Box::Volume` float leaf lives here | float-leaf codec already typed |
| W13 | **Float codegen** (`fmul3` and friends) | f1–f13 params, `fmuls`/`fadds`/`fsubs`, `frsp`; float *constants* need `.rdata` + `lfs` + a data reloc (splits into 13a param-only leaves, 13b constants) | `mvp_fmul3.cpp` |
| W14 | **Data sections / globals** (`.data`/`.rdata`, string literals, statics, arrays) | new COFF sections, `ADDR32` relocs, symbol storage classes | — |

Long tail, census-driven only: switch/jump tables, 64-bit ints (`addc`/`adde`
carry chains), unsigned variants everywhere, virtual/indirect calls (needs
W12 + `mtctr`/`bctrl`), `__declspec`s, intrinsics. Do not schedule these
ahead of measured demand.

**Re-rank from the P2b census, now that the CALL grammar is characterized
(INFERENCE from the measured histogram of §G5 — the histogram is a
measurement, the attribution of buckets to rungs is not).** The largest
buckets are no longer anonymous: `docs/IL_CALL_GRAMMAR.md` decodes the CALL
token, the three coexisting variable-width encodings and the body/statement
grammar, and names most of the top of the histogram. Grouping the current top
10 by what the bytes actually are:

- **Genuinely out-of-class calls — these must keep failing closed, they are
  not a "fix"** (`call-token-0xB9`, **14.8%**, the single largest bucket).
  This is *not* a missing opcode: `BD` is a postfix operator applied to
  whatever the operand stream pushed, and here the callee is an **expression**
  — `b9 <tok> <TYPE>` (indirect call) or `26 <method> <obj-expr> 99 …`
  (member call) — not the direct `26 <tok>`. An indirect call has **no
  relocatable callee name anywhere**; a member call needs a `this` argument
  and possibly vtable dispatch. Per `IL_CALL_GRAMMAR.md` §6.2 both must be
  rejected *before* emission, permanently, until W11/W12 give the port real
  argument passing and member addressing. Widening the parser to accept them
  without codegen would convert a refusal into a mis-emit.
- **Type-driven blocks**, i.e. the port only lowers `int`:
  `expr-load-type-864540` (**float**, 3.4%), `expr-load-type-888541`
  (**double**, 3.1%), `expr-load-type-864383` (**void\***, 1.9%) = **8.4%**.
  These are honest **W13** and **W12** demand. Knowing how to *skip* a
  `double` (the width rule is now known) is not knowing how to lower it.
- **Casts** — `expr-cast` (`40 <target-type>`, **6.8%**), the second-largest
  bucket. Mostly W12-adjacent (pointer/integer conversion) and the cheapest
  of the large buckets to characterize next.
- **Remaining call-shaped** blocks: `call-token-0x33` (5.9%, a literal where
  a CALL was expected), `expr-call-in-expr` (4.9%), `call-token-0x26` (3.3%)
  = **14.1%** — real **W11** demand (multi-arg calls, calls nested in
  expressions, `26 dest 26 callee BD …` assign-a-call-result statements).
- **Statement/body-shaped** blocks: `body-0x3A` (**4.4%**) is now known to be
  an **empty function body** (`IL_CALL_GRAMMAR.md` §4.2) — the cheapest large
  bucket on the board and staged as `w10_empty_fn.cpp`; `body-0x53` (2.9%) is
  a body whose first statement is an `if`/compound, i.e. **W8** control flow.
- **W6 (comparisons) and W7 (shifts) still do not appear in the top 10**, and
  W6's leaf class has now landed anyway.

So the measured demand still says the W5→W6→W7→W8 order is not
demand-driven, but the shape of the demand has changed: a large part of the
top is **out-of-class by construction** (member/indirect calls) rather than
schedulable work, and the schedulable head is *empty function bodies*, then
casts, then float/double types, then generalized calls.

> **Standing caution — a census bucket may be a parser defect, not
> vocabulary.** This has now happened **twice**. The variable-token-width fix
> (§G2) deleted the `call-token-0x01…0x05` and `expr-load-type-0N00A6`
> families, which were pure misalignment. The CALL-token decode deleted the
> entire **`call-anchor-*` family** — previously ~12.4% of blocked functions
> (`call-anchor-0x00` 235,886, `-0x08` 43,269, `-0x20` 24,600 → **0**) —
> which was measuring a hardcoded 6-byte "anchor" that was never an anchor
> (§G2). Both times a large, plausible, stable bucket turned out to be the
> instrument rather than the corpus. Before scheduling work against any
> bucket, dump the bytes at the recorded offset (`c2rs census`, which now
> prints them) and confirm the parse arrived there **aligned**.

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

**Outstanding**: `crates/c2-il/src/codec.rs` still assumes a fixed 2-byte
token (`tok16`) and therefore carries the same latent defect on real TUs. It
is round-trip gated, so it fails *closed* (an opaque span) rather than
mis-decoding — no correctness exposure today — but it caps typed coverage on
real bundles and must adopt the variable-width read before the codec is
pointed at the real workload.

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
guessed opcode names into measured ones (the relational correction below), and
what exposed the true CALL token shape and the meaning of `body-0x3A` /
`body-0x53` on a real TU. Per-function lines are suppressed above 64 functions,
where only the histogram is readable.

**Baseline (2026-07-29 end of day, 878 dc3 TUs, real `/O1 /Oi /EHsc` flags,
~36 s at `--jobs 16`):**

| Class | TUs | % |
|---|---|---|
| **match** | **5** | **0.6** |
| mismatch | 0 | 0.0 |
| codegen-gap | 0 | 0.0 |
| **vocab-gap** | **866** | **98.6** |
| capture-fail | 7 | 0.8 |

**The match bucket is nonzero for the first time** — R1 (empty-TU obj
emission, §1). Five of the seven zero-function TUs; the other two are refused
on purpose (a stray `4F 1F` after the module end defeats the positive
`is_empty_module` test, and relaxing that test is not worth 2 TUs).

**Function census: 78,028 / 2,462,571 functions in class (3.17%).** Progression
today on the identical instrument:

| | in class | % |
|---|---:|---:|
| start of day | 4,154 | 0.17 |
| + variable token width (§G2, 40f767d) | 7,114 | 0.29 |
| + CALL-token decode (§G2, 2870fc1) | 7,954 | 0.32 |
| + empty function bodies (`w10_empty_fn.cpp`, a44c8f3) | **78,028** | **3.17** |

The first two steps were *decode* fixes, not new codegen — the expected
shape of progress while the wall is `vocab-gap`. The third is a 10x jump from
one very small class: empty bodies are ~4.4% of blocked functions by count,
and accepting them also unblocks the many TUs that are *mostly* trivial
accessors and destructors.

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

Fixture-level status at the same commit: **17/31 `Port=Match`, 0 mismatch**;
`cargo test --workspace` green.

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

Blocking-feature histogram, top 10 (percentages are of *blocked* functions),
with what each bucket is now known to be:

| Functions | % | Feature | What the bytes are |
|---:|---:|---|---|
| 363,684 | 14.8 | `call-token-0xB9` | **member / indirect calls** — the callee is an *expression*, not `26 <tok>` |
| 167,205 | 6.8 | `expr-cast` | `40 <target-type>` |
| 144,276 | 5.9 | `call-token-0x33` | a literal where a CALL was expected |
| 119,800 | 4.9 | `expr-call-in-expr` | a call nested inside an expression |
| 107,253 | 4.4 | `body-0x3A` | an **empty function body** |
| 82,491 | 3.4 | `expr-load-type-864540` | **float** |
| 80,284 | 3.3 | `call-token-0x26` | `26 dest 26 callee BD …` (assign a call result) |
| 75,081 | 3.1 | `expr-load-type-888541` | **double** |
| 70,078 | 2.9 | `body-0x53` | first statement is an `if`/compound |
| 47,640 | 1.9 | `expr-load-type-864383` | **void\*** |

…plus a long tail of further distinct features (1,217 more rows at the
mid-day measurement; the row count moved with the `call-anchor-*` retirement
below and is deliberately not re-quoted from the old scan).

**The `call-anchor-*` family is gone** — it was 12.4% of blocked functions
(235,886 + 43,269 + 24,600) and it was measuring the port's own hardcoded
anchor, not a real gap (§G2, and the standing caution in §G1). Do not compare
this histogram to the mid-day one without accounting for that retirement.

How to read the bucket names (`func::Block::feature`) — `<production>-0xNN`
means the parse was inside that grammar production and could not consume byte
`NN`:

- `call-token-*` — the byte where the `BD` CALL token was expected after the
  callee expression.
- `body-*` — the byte that opened the function body, where only a call ref
  (`26`), LOAD (`B9`) or literal (`33`) is modeled.
- `expr-*` — inside the operand stream; `expr-*-type-NNNNNN` reports the
  operand's whole inline type, because the type triple *is* the feature (int
  vs unsigned vs float vs pointer). Named buckets (`expr-cast`,
  `expr-call-in-expr`, the relationals) carry only **capture-verified** names
  — see the correction below.

**The census's relational opcode names were guessed, and were wrong.**
Compiling one probe per relation against the live toolchain (commit 45421f6,
`docs/CODEGEN_W6_COMPARE.md` §1.1) measured `0x1F` `==`, `0x20` `!=`,
`0x21` `<=`, `0x22` `<`, `0x23` `>=`, `0x24` `>`. The table had said `0x20`
`==`, `0x21` `!=`, `0x23` `<=`, `0x25` `>=`, and had **no** name for `==` at
all — so three buckets were mislabelled and every `==` landed in an unnamed
one. Diagnostic only (acceptance never consults the name) but the ranked
blocker lists in `docs/GAPS.md` are keyed on these strings. A hex bucket is a
result; a wrong name is a lie that survives into the roadmap.

Notes:

- **Replay soundness holds at full strength**: 871/871 byte-exact, 0 diverged
  on both the 2026-07-20 and the 2026-07-29 `--replay-every 1` passes. The
  oracle holds on the capturable real workload.
- The 7 `capture-fail`s are all `synth_xbox/soundtouch` files the real 360
  build excludes (x86-only `#error` guards) or builds with per-target flags —
  a workload-manifest refinement, not a port gap.
- **The TU wall is still `vocab-gap` for every TU that contains code**: those
  866 TUs die at `c2_il` function decode before codegen is consulted, because
  `functions()` is all-or-nothing per TU. That is unchanged and expected — a
  TU with 700 functions of which 699 are in class is still one `vocab-gap`.
  The 5 matches are exactly the TUs with *no* functions to decode. The census
  is what moves per widening step; the TU buckets move much later.

Remaining for G5: keep the JSONL baselines diffable scan-over-scan (coverage
must be monotone), and keep characterizing the head of the histogram —
`expr-cast` (`40 <target-type>`) is now the largest *schedulable* unknown,
`body-0x3A` the largest cheap one. The §G1 re-rank is no longer blocked on
`call-token-0xB9`: that bucket is characterized (member/indirect calls,
`IL_CALL_GRAMMAR.md` §3.2/§6.2) and is out of class by construction rather
than schedulable.

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
R1    empty-TU obj emission — first nonzero match bucket   [DONE 2026-07-29: 5/878]
W5    multi-scratch expressions                     [CHAINS DONE 2026-07-29; trees open]
W6    integer comparisons → bool                    [LEAVES DONE 2026-07-29; <,<=,>= vs k≠0 open]
W7    shifts + bitwise + strength reduction         (staged)
W8    control flow (if/else, ternary, loops)        (first CFG; IR restructure)
W9    division / modulo
W10   general frames + locals (+ .pdata labels W-UNW-1)
W11   calls generalized (+ W-UNW-1 for multi-fn TUs)
W12   memory / struct access
W13   float codegen (13a param-only, 13b constants→.rdata)
W14   data sections / globals
P-F0.2 FE characterization probes → FE_BUNDLE_MVP.md   (parallel track)
P-F1  c1-core MVP + Grade-1/2 gates                    (parallel track)
P-F2  FE widening in lockstep + perf-fe                (parallel track)
P3    c2rs compose — source→obj in-process, byte-exact + perf-fe scale
```

**The W-numbering above is the pre-census estimate and is not the running
order.** The current histogram (§G5), read against the now-characterized CALL
and body grammar, ranks the schedulable work as: **empty function bodies**
(`body-0x3A`, 4.4%, staged as `w10_empty_fn.cpp` — the cheapest large bucket,
and R1's function-level analogue), then **casts** (`expr-cast`, 6.8%), then
**non-int operand types** (float/double/`void*`, 8.4% — W13/W12), then
**generalized calls** (`call-token-0x33` + `expr-call-in-expr` +
`call-token-0x26`, 14.1% — W11), with control flow (`body-0x53`, 2.9% — W8)
behind them. The single largest bucket, `call-token-0xB9` at 14.8%, is **not
schedulable work at all**: it is member and indirect calls, which have no
relocatable callee name (indirect) or need a `this`/vtable model (member), and
which must keep failing closed until W11/W12 exist (§G1,
`IL_CALL_GRAMMAR.md` §6.2). The attribution of buckets to rungs remains
*inference*; the histogram itself is measurement.

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
6. **Empty function bodies** (`body-0x3A`, 4.4% of blocked functions) —
   fixture `w10_empty_fn.cpp` is already staged; no instruction selection
   needed, so it is the cheapest remaining large bucket. Note the
   `IL_CALL_GRAMMAR.md` §4.2 trailing-expression variant must stay rejected.
7. **W5 trees** — `(a+b)*(c+d)` and deeper, per `CODEGEN_W5_SCRATCH.md` §7
   (the cursor rule, the level-order emission rule, and the six-row shape
   gate G1–G6); `w5_tree_neg.cpp`'s eleven functions must stay
   `NotImplemented`.
8. **Port `codec.rs` to the variable-width token read** (`tok16` → the
   `read_token_var` rule), round-trip gate unchanged, so typed coverage can
   be measured on real bundles rather than fixtures (§G2).
9. **Characterize `expr-cast`** (`40 <target-type>`, 6.8%) — the largest
   schedulable unknown left in the histogram.
10. **P-F0.2**: argv/line-number/whitespace probes → `docs/FE_BUNDLE_MVP.md`.

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
