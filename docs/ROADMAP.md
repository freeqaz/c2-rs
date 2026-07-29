# ROADMAP — from the MVP port to a fully implemented stack

Status: living document (written 2026-07). Describes where the port stands,
what is missing, and the ordered plan to close it. The invariants at the
bottom are **load-bearing** — every phase must preserve them.

## 1. Where we are

The foundation is proven and fast; the port itself is deliberately narrow.

**Proven (differential, against real `c2.dll`/`c1xx.dll` 16.00.11886.00 under wibo):**

- **P0.1** — standalone-c2 IL-bundle replay is byte-exact on all 25 fixtures
  (`c2rs replay`) **and on all 871 capturable TUs of the real dc3-decomp
  workload** (`c2rs gap --replay-every 1`, 2026-07-20 baseline — after two
  real-workload fixes: a missing `lstrcpynA` shim in wibo that killed c2 on
  large TUs, and the `1033/clui.dll` resources symlink beside `c2host`,
  without which any TU that triggers a diagnostic dies with C1510). The
  reference side of the differential is real, on real code.
- **P-F0.1** — standalone-c1 (front-end) replay is byte-exact on all 25
  fixtures (`c2rs replay-c1`). The same porting path is open for `c1xx.dll`.
- **The port (`c2-core::PortC2`) is byte-exact on the MVP class**
  (`c2rs diff`): straight-line integer add/sub/mul chains with immediate
  folding and wide constants, multi-function TUs of those, bare void tail
  calls, integer tail calls `return g(<arg>)` (passthrough / `+0` fold /
  arg-setup), and the framed non-leaf `return g(a) + k` (6-section obj with
  `.pdata`). Everything else returns `NotImplemented` — fail closed, never a
  guess.
- **Harness + tooling**: oracle self-test, corpus generator, obj→IL retrieval
  baseline, IL-space search prototype, edit gate; `perf`/`perf-scale`
  (~200–290× per obj, ~897k obj/s at 32 threads vs ~3.1k for real c2).
- **P2 / P2b measurement** — `c2rs gap` (real-workload TU scan) plus the
  function-level census (`c2rs census <cpp>`, and the scan-wide histogram in
  the `gap` report). The port's real coverage is now a *measured* number:
  **7,114 / 2,462,571 functions in class (0.29%)** over the 871 capturable dc3
  TUs, with the blocking-feature histogram that ranks the remaining work
  (§G5).
- **IL codec (K1/K2a)** — round-trip-gated typed-islands-over-opaque-spans
  model of `.ex`/`.gl`; float-leaf (`Box::Volume` shape) token vocabulary is
  typed; K3a length-consistent edits verified *as edits* against real c2.

**Staged but not yet ported** (fixtures/probes already in-repo, pointing at
the next classes):

- `il_bool_materialization.cpp` — integer comparisons → boolean (`x != 0`,
  `x > 7`, signed/unsigned).
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
classes, in the *original* intended working order — the census has since
landed and provisionally re-ranks this list; see the note after the table
and §G5:

| # | Class | New mechanisms required | Staged? |
|---|-------|-------------------------|---------|
| W5 | **Multi-scratch expressions** (tree depth > 2, e.g. `(a+b)*(c+d)`) | COLOR scratch order beyond r11; operand-stack depth limit lifted | test-pinned reject |
| W6 | **Integer comparisons → bool** (`x!=0`, `x>7`, signed/unsigned) | `cmpwi`/`cmplwi`, cr bits, `subfe`/`addze` materialization idioms | `il_bool_materialization.cpp` |
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

**Provisional re-rank from the P2b census (INFERENCE — see §G5 for the
measured histogram it is derived from).** Grouping the top-20 blocking
features by the grammar production they died in:

- **Call-shaped** blocks (`call-token-*` 24.0% + `call-anchor-*` 12.4% +
  `expr-call-in-expr` 2.7%) = **~39% of blocked functions** from the top-20
  rows alone — a lower bound, since further `call-*` buckets sit in the
  1,217-row tail. That is the **W11** (calls generalized) rung.
- **Statement/body-shaped** blocks (`body-*`: `0x3A` a body opening with
  ASSIGN, `0x53` a further statement marker, plus `0x9B/0x4F/0xB3/0xAD/
  0x29/0x67/0x9A`) = **~12%** from the top-20 rows, again a lower bound.
  That is the **W10** (general frames + locals + statements) rung.
- **W6 (comparisons) and W7 (shifts) do not appear in the top 20 at all.**

So the measured demand says W11 and W10 dominate and the W5→W6→W7→W8 order
above is *not* demand-driven. The re-rank is deliberately labelled
provisional: the exact meaning of several top buckets is **not yet
characterized** — most importantly `call-token-0xB9` (14.8%, the single
largest bucket) means a LOAD stands where the fixed 10-byte CALL token was
expected, so `26 <tok>` is evidently **not** always a call prefix. Until that
grammar question is answered, the grouping above may be mis-attributing a
large share of the histogram. Characterize `26`/`B9` first, then commit to
the re-order.

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
COLOR knowledge lands when W5/W10 demand real register-order modeling.

### G5 — Coverage measurement (instrument LANDED; P2b census LANDED; baseline below)

The measuring tool is **`c2rs gap`** (real-workload gap scan): run the whole
pipeline — capture with the project's real flags → port → byte compare — over
real dc3 TUs, bucketing each into `capture-fail` / `vocab-gap` /
`codegen-gap` (keyed by the port's own `NotImplemented` reason) / `mismatch`
/ `match`, with JSONL records for longitudinal diffing and a
`--replay-every N` soundness lane extending P0.1 to real workloads.
`scripts/gen_dc3_workload.sh` generates the inputs from a dc3-decomp
checkout.

**P2b — function-level census: LANDED** (commits 63b1ad1, ec401a5). The
TU-level scan is all-or-nothing, so it could only ever report 871 ×
`vocab-gap` and could not rank W5–W14. There is now a per-function view:
`c2rs census <cpp> [--flags-file F --cwd D] [--keep-il DIR]` for a single TU,
and the `c2rs gap` report additionally prints scan-wide function coverage
plus the blocking-feature histogram. Each function segment is run through the
*same* positive parser and keeps its **first** blocking `(production, byte,
offset)` — so the histogram measures the unknown vocabulary honestly (hex
buckets, never guessed names) instead of asserting it.

**Baseline (2026-07-29, 878 dc3 TUs, real `/O1 /Oi /EHsc` flags, 37.3 s at
`--jobs 16`):**

| Class | TUs | % |
|---|---|---|
| match | 0 | 0.0 |
| mismatch | 0 | 0.0 |
| codegen-gap | 0 | 0.0 |
| **vocab-gap** | **871** | **99.2** |
| capture-fail | 7 | 0.8 |

**Function census: 7,114 / 2,462,571 functions in class (0.29%).** (Same
instrument before the variable-token-width fix of §G2: 4,154 / 2,462,571 =
0.17%.)

**Replay soundness re-proven at full strength on 2026-07-29 with the
post-fix code: 871/871 capturable TUs byte-exact, 0 diverged**
(`--replay-every 1`, 43.4 s at `--jobs 16`). The oracle holds on the entire
capturable real workload, so every IL-derived measurement above rests on a
reference side that was itself checked, not assumed.

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

Blocking-feature histogram, top 20 (percentages are of *blocked* functions):

| Functions | % | Feature |
|---:|---:|---|
| 363,684 | 14.8 | `call-token-0xB9` |
| 235,886 | 9.6 | `call-anchor-0x00` |
| 166,483 | 6.8 | `expr-op-0x40` |
| 144,276 | 5.9 | `call-token-0x33` |
| 107,253 | 4.4 | `body-0x3A` |
| 80,284 | 3.3 | `call-token-0x26` |
| 70,078 | 2.9 | `body-0x53` |
| 67,012 | 2.7 | `expr-call-in-expr` |
| 43,269 | 1.8 | `call-anchor-0x08` |
| 41,878 | 1.7 | `expr-load-type-864383` |
| 34,573 | 1.4 | `expr-load-type-864275` |
| 28,487 | 1.2 | `body-0x9B` |
| 26,666 | 1.1 | `body-0x4F` |
| 24,600 | 1.0 | `call-anchor-0x20` |
| 22,947 | 0.9 | `expr-lit-type-821230` |
| 15,782 | 0.6 | `body-0xB3` |
| 15,480 | 0.6 | `body-0xAD` |
| 14,594 | 0.6 | `body-0x29` |
| 13,248 | 0.5 | `body-0x67` |
| 12,405 | 0.5 | `body-0x9A` |

…and **1,217 more distinct features** in the tail.

How to read the bucket names (`func::Block::feature`) — `<production>-0xNN`
means the parse was inside that grammar production and could not consume byte
`NN`:

- `call-token-*` — the byte where the fixed 10-byte CALL token
  (`BD <3-byte ret type> 00 80 01 10 00 00`) was expected after a `26 <tok>`
  reference.
- `call-anchor-*` — the 6-byte anchor `00 80 01 10 00 00` did not match.
- `body-*` — the byte that opened the function body, where only a call ref
  (`26`), LOAD (`B9`) or literal (`33`) is modeled.
- `expr-*` — inside the operand stream; `expr-*-type-NNNNNN` reports the
  operand's whole 3-byte inline type, because the type triple *is* the
  feature (int vs unsigned vs float vs pointer).

Notes:

- **Replay soundness holds**: 871/871 byte-exact, 0 diverged on the 2026-07-20
  full `--replay-every 1` pass; the sampled lane on the 2026-07-29 runs has
  likewise never diverged (110 sampled, 0 diverged). The oracle holds on the
  capturable real workload.
- The 7 `capture-fail`s are all `synth_xbox/soundtouch` files the real 360
  build excludes (x86-only `#error` guards) or builds with per-target flags —
  a workload-manifest refinement, not a port gap.
- **The TU wall is still `vocab-gap`**: every real TU dies at `c2_il` function
  decode before codegen is consulted, because `functions()` is all-or-nothing
  per TU. That is unchanged and expected — a TU with 700 functions of which
  699 are in class is still one `vocab-gap`. The census is what moves per
  widening step; the TU buckets move much later.

Remaining for G5: promote match-% to the headline metric once nonzero, keep
the JSONL baselines diffable scan-over-scan (coverage must be monotone), and
characterize the top histogram buckets (notably `call-token-0xB9`) so the
provisional re-rank in §G1 can be committed to.

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
W5    multi-scratch expressions                     (unblocks expression shapes)
W6    integer comparisons → bool                    (staged; small; frequent)
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

**The W-order above is the pre-census estimate and is now known not to be
demand-driven.** The measured histogram (§G5) puts call-shaped blocks (~39%
of blocked functions from the top-20 rows alone) and statement/body-shaped
blocks (~12%) at the top, i.e. **W11 and W10**, while W6 (comparisons) and W7
(shifts) do not appear in the top 20 at all. The re-rank is *inference* from
the histogram, not a measured schedule, and it is blocked on characterizing
`call-token-0xB9` / the `26` prefix (§G1). Sequence to run instead: the
grammar-characterization step first, then re-rank, then the rung it picks.

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
   not TUs.~~ **DONE 2026-07-29** (63b1ad1, ec401a5) — see G5. Headline:
   **7,114 / 2,462,571 functions in class (0.29%)**, 1,237 distinct blocking
   features, denominator anchored on the `LO` body marker (**not** `.gl`
   names). Bundled finding: IL tokens are variable width (40f767d, §G2),
   which retired two fabricated bucket families and moved the numerator
   4,154 → 7,114.
3. **Characterize `call-token-0xB9` / the `26` prefix** — the single largest
   bucket (14.8%) says a LOAD stands where the CALL token was expected, so
   `26 <tok>` is not always a call. Until this is understood the top of the
   histogram cannot be attributed to a rung, and the §G1 re-rank stays
   provisional. This is now the highest-value next step.
4. **Port `codec.rs` to the variable-width token read** (`tok16` → the
   `read_token_var` rule), round-trip gate unchanged, so typed coverage can
   be measured on real bundles rather than fixtures (§G2).
5. **W5**: lift the depth-2 stack limit using the documented COLOR scratch
   order; positive fixture `(a+b)*(c+d)`, negative depth-5 tree.
6. **P-F0.2**: argv/line-number/whitespace probes → `docs/FE_BUNDLE_MVP.md`.
7. Commit the W-ladder re-rank once step 3 lands; update this doc.

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
