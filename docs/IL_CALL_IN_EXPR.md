# The `expr-call-in-expr` bucket, decomposed

> **UPDATE (2026-07-30, `1caf463`): D1 is LANDED, and §13 records what it cost
> and what it yielded against the estimate made below.** Everything from §1 to
> §12 is the characterization as written, unedited, so the estimate can be
> compared with the outcome.

**Status: characterization only (2026-07-30). No code was changed.** The bucket
is the #1 blocking feature on the real dc3 workload — **304,813 functions, 13.0%
of 2,352,205 blocked** (MEASURED, `c2rs gap` over 878 TUs at HEAD, re-run for
this document) — and its name had never been checked against its contents. This
document samples it, splits it into productions, pins each production's byte
grammar with controlled probes, and ranks the pieces by *expected in-class
yield*, not bucket size.

Evidence labels:

* **MEASURED [W]** — the workload sample (§1): 21,319 blocking sites from 40
  real TUs, wide byte windows extracted through the public census API.
* **MEASURED [pX]** — a controlled probe TU in `/tmp/callprobe/probes/`
  (regenerable from §10), captured and compiled with the workload's own
  optimization flags (`/nologo /c /GR /O1 /Oi /EHsc`) against the live
  16.00.11886.00 toolchain under wibo. Reference `.text` bytes come from
  `c2rs compile --keep-obj`; the instruction decodes below were checked by hand
  against the PPC ISA, not trusted to the scratch disassembler.
* **HYPOTHESIS** — stated as such, with the evidence that would settle it.

---

## 0. Summary — what the name got right and wrong

The bucket is mechanically "`0x26` met inside `parse_expr`" (§2). What those
`26`s begin, over the sample:

| share | production | is it a call? |
|---:|---|---|
| ~64% | a **member call** — method-symbol push, receiver designator, `99`-bind, `BD` | yes, but a *member* call |
| ~18% | a **data-symbol address pushed as a call argument** (string literal, array decay, `&global`) | no — a data push |
| ~2.5% | a **data-symbol read or store** (global/static object member) | no — a data push |
| ~0.2% | a plain nested call `26 <fn> BD` — the thing the name literally describes | yes |

So the prior warning that the name is a **partial misnomer** is confirmed in
letter but needs re-aiming: the bucket is ~80% call-shaped after all — but
**99.7% of those calls are member calls**, not the nested plain call the name
suggests, and the plain nested call is 0.2%. The ~20% data-push remainder is
real and matches the earlier warning.

Two structural findings matter more than the shares:

1. **One production, two buckets.** The member-call spine is
   `26 <method> <receiver> 99 … BD …`. When the *receiver load* is what the
   parser reaches first (`return p->Get();` — the body dispatch sends the
   leading `26` to the assignment parser as a destination), the function files
   under `expr-load-type-{86,A6,96}43xx` (a pointer-typed operand), **not**
   here. Those pointer-typed-load buckets sum to **1,127,384 functions (47.9%
   of all blocked)** (MEASURED, scan JSONL). `expr-call-in-expr` holds only the
   sites where a `26` comes first — so this bucket is the *smaller* shard of
   the member-call wall, and neither bucket's size measures the production.
2. **The single largest coherent sub-shape is the compiler-generated empty
   destructor** delegating to its base's destructor: 2,046 of 21,319 sampled
   sites (9.6%), one rigid byte skeleton, and its reference lowering is a bare
   4-byte `b ??1Base…` — the exact `.text` shape the port's tail-call emitters
   already produce. That is the decode-only production with nonzero yield (§8).

---

## 1. Sampling method

* `c2rs gap … --jsonl` over all 878 TUs re-measured the bucket at **304,813**
  and gave per-TU counts (842 TUs carry the bucket).
* 40 TUs were sampled: 12 drawn at random from the top decile by bucket count,
  28 uniformly from the rest. Together they hold **21,319 bucket sites = 7.0%
  of the bucket**.
* Each TU's bundle was captured with
  `c2rs census <tu> --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --keep-il`,
  and a scratch tool (a `/tmp` crate with a path dependency on `c2-il`, no repo
  edits) re-ran `IlBundle::function_census()` on the kept bundles and dumped a
  48-byte-back / 96-byte-forward window around every `expr-call-in-expr`
  blocking offset — every blocked function, not one witness per TU.
* Classification walks forward from the blocking `26` with a width-complete
  tokenizer (the `read_token_var` / `read_type` / signed-varint rules of
  `docs/IL_CALL_GRAMMAR.md` and `docs/IL_CAST_CONVERT.md`) to the first
  decisive token (`BD`, `40`, `30`, `55`, `32`).

**These shares are estimates from a cluster sample.** 40 TUs of 842, 7% of the
sites; the stratification overweights large TUs, and the per-TU spread is real
(§2's `sd` column). Treat every share as ±5 points and every extrapolated count
as an order-of-magnitude-correct estimate, not a census row.

---

## 2. The decomposition — MEASURED [W]

Where the bucket fires in the parser first: the census key `expr-call-in-expr`
is minted at `crates/c2-il/src/func/body/mod.rs:175` for a `Block` raised at
`crates/c2-il/src/func/body/expr.rs:336` — byte `0x26` inside `parse_expr` —
and `parse_expr` runs in exactly four positions: the straight-line return
expression (`mod.rs:320`), an assignment right-hand side (`shapes.rs:108`), the
post-assignment return expression (`shapes.rs:145`), and a call-argument region
(`shapes.rs:1015`). A `26` in *statement-head* position never lands here — the
dispatch at `mod.rs:289` consumes it as an assignment destination or callee.

Pooled shares of the 21,319 sites (per-TU mean ± sd across the 40 TUs in
parentheses):

| sites | share | production | §
|---:|---:|---|---|
| 5,141 | 24.1% (26.8 ± 6.3) | member call, **loaded receiver** — `B9` formal/local pointer, incl. designator-chain receivers (`s->sub.M()`) | §3 |
| 4,421 | 20.7% (20.5 ± 4.3) | member call, **named-object receiver** — `26 <sym> [2C]` then `99`/`BD` (global/static object, singletons) | §3.2 |
| 3,780 | 17.7% (13.9 ± 6.7) | **chained** member calls / call-result receivers — `26 26 …`, incl. `G().Val()` | §4 |
| 3,763 | 17.7% (18.0 ± 3.4) | **data-symbol address as call argument** — string literal, array decay, `&global` | §6 |
| 3,629 | 17.0% (18.6 ± 10.0) | member call through a **class-layout-intrinsic receiver** (2113/2116), **dominated by generated destructors** | §5 |
| 534 | 2.5% | **data-symbol read/store** — `26 <sym> <lit> 27/28 → 30` or `2C → 32` | §7 |
| 45 | 0.2% | **plain nested call** `26 <fn> BD` | §7.3 |
| 6 | 0.03% | residue: `9B` temporary receiver (5), virtual `67` (1) | — |

The destructor row's 10-point sd is one small TU where destructors are 70% of
the bucket; the others are stable across TUs.

---

## 3. The member-call production — MEASURED [pA], [pF]

```text
MEMBER-CALL := 26 <method-tok>            the METHOD symbol, pushed FIRST
               <receiver-designator>      any designator expression (below)
               99 <TYPE> 00               member bind; trailing byte 00 at every site
               BD <ret-TYPE> 00 <varint>  the ordinary CALL token (cdecl)
               ( <expr> 55 <TYPE> )* 4C   EXPLICIT args only — `this` is NOT here
```

Witness, `int t1(Obj* p){ return p->Get(); }` [pA], whole body from `LO`:

```
53  26 e4 09                       method `Get`
    b9 f4 09 86 43 81 20           LOAD p : Obj*
    99 86 43 87 20 00              bind, trail 00
    bd 86 41 74 00 80 07 10 00 00  CALL ret=int conv=00 fn-type 0x1007
    4c                             zero explicit args
    41 86 41 74 3a f6 09 …         return plumbing
```

Facts, each with the probe that separates it from the plausible wrong rule:

* **The method symbol is pushed before the receiver, and it is a symbol like
  any other.** The same token `e4 09` names `Get` at all four of its call
  sites in [pA] while the receiver token varies. The wrong rule — "the first
  `26` is the receiver" — dies on the chained probe [pB], where *several*
  method tokens precede one receiver (§4).
* **`this` never appears in the argument region.** `t2: p->Set(5)` carries
  exactly one `55`-terminated argument (the 5); the receiver rides the operand
  stack into the `99`. A decoder that expected `this` as arg 0 would misparse
  every site.
* **Pointer and reference receivers are byte-identical.** `t1(Obj* p)` and
  `t3(Obj& r)` produce the same types and opcodes (`86 43 81 20` both).
* **Virtual dispatch does not use this form.** `t4: p->VGet()` opens
  `67 00 <tok> b9 <p> … 30 … 30 … 9a … bd` — a different production (opcode
  `67`, double indirect load through the vtable, a `9A` bind), blocked today
  as `body-0x67` / `expr-op-0x67`. So a `99`-bind site is direct dispatch **by
  construction**, which is what licenses lowering it to a direct `b`/`bl`.
* **A static member call has no `99` at all** — `t5: Obj::SGet()` is a plain
  `26 <fn> BD … 4C` (it blocks today only on the zero-argument gate,
  `call-args-none`).
* **`99`'s trailing byte is `00` at all 9,976 bind sites in the sample and
  every probe.** INDISTINGUISHABLE from a constant here; `docs/IL_EXPR_LAYER.md`
  §7's guess (a `this`-adjustment offset) remains untested — the separating
  fixture is still a multiple/virtual-inheritance member call where an adjust
  is required *without* the 2113 intrinsic appearing.

### 3.1 Receiver designator forms seen in the sample

| receiver bytes | meaning | share of bucket |
|---|---|---:|
| `B9 <tok> <ptr-TYPE>` | pointer/reference formal or local | 13.9% |
| `B9 <tok> <ptr> 33 <off> 27 <TYPE> [2C]` | member sub-object (`s->field.M()`) | ~10% |
| `26 <sym> 2C <ptr-TYPE> 00` | named object, address decay | 17.9% |
| `26 <sym>` (no convert) | named object, no decay | 2.9% |
| a whole call (§4) or intrinsic (§5) | computed receiver | ~35% |

The no-convert variant is unpinned. HYPOTHESIS: the `2C` appears when the
object's cv-qualification differs from the method's `this` type — but [pF]
`gM.M()` (non-const object, non-const method) *still* carries a `2C`, so the
rule is not "const only"; what distinguishes the 2.9% is not established.

### 3.2 Named-object receiver, reference codegen — MEASURED [pA]

`int t6(){ return gO.Get(); }`, workload flags:

```
3d600000  lis  r11, gO@ha      IMAGE_REL_PPC_REFHI + PAIR
386b0000  addi r3, r11, gO@l   IMAGE_REL_PPC_REFLO + PAIR
4bfffff8  b    ?Get@Obj@@QBAHXZ   IMAGE_REL_PPC_REL24
```

Address materialization: two instructions, a REFHI/REFLO pair against the
object's own symbol, then the tail branch. The relocation shape is the same
one W13b already emits for `__real@` constants, but the symbol class (.data /
extern) and the `addi`-into-r3 pattern are new emitter work.

---

## 4. Chained member calls — MEASURED [pB]

`int c2(N* p){ return p->Next()->Next()->Next()->Val(); }`:

```
26 e5 09  26 e4 09  26 e4 09  26 e4 09     Val, Next, Next, Next  (outermost FIRST)
b9 f0 09 86 43 81 20                       LOAD p
99 … 00  bd <N*> 00 … 4c                   innermost Next()   — binds the LAST-pushed method
99 … 00  bd <N*> 00 … 4c                   ->Next()
99 … 00  bd <N*> 00 … 4c                   ->Next()
99 … 00  bd <int> 00 … 4c                  ->Val()
41 <int> …
```

Method symbols stack LIFO: each `99…BD…4C` consumes the deepest un-bound method
and the current top-of-stack receiver, and its result is the next receiver.
This explains the workload's `26 <tok>` runs with the *same token repeated* —
`p->Next()->Next()` pushes `Next` twice. `int c3(){ return G().Val(); }` is the
receiver-is-a-plain-call variant (`26 <Val> 26 <G> BD … 4C 99 … BD … 4C`), the
sample's 15.6% `SYM SYM CALLBD` signature; real sites often interpose a
`9B`-temporary and unknown opcode `0x64` when the call returns an object by
value (UNKNOWN, one App.cpp witness quoted in the scratch notes; not decoded).

Codegen: two or more real calls — a frame, saved LR, a `bl` chain. W11. Nothing
here is decode-only.

---

## 5. The class-layout-intrinsic receiver — and what it mostly is: destructors

17.0% of the sample opens `26 <method> 33 86 41 74 80 41 08 00 00 40 …` — a
member call whose receiver is intrinsic **2113 `this-adjust`**
(`docs/IL_CAST_CONVERT.md`). Nearly all of it (3,605/3,631) is selector 2113
with **offset 0**, and the enclosing body is one rigid skeleton. Controlled
witness [pF]: `struct Base { ~Base(); int b; }; struct Der : Base { ~Der(); int d; };
Der::~Der() {}` — the generated body, whole, from `LO`:

```
53  33 86 41 74 00                LIT 0                     (role UNKNOWN — see below)
    26 e4 09                      method  = ??1Base…
    33 86 41 74 80 41 08 00 00    LIT 2113
    40 86 43 8e 20                intrinsic call, ptr result
    66 02 80 20 82 20             class-pair descriptor
    55 86 41 74                   selector arg end
    33 86 41 74 00  55 86 41 74   adjust offset 0
    b9 fc 09 a6 43 81 20  55 …    `this`
    4c                            -> adjusted receiver
    2c a6 43 84 20 00             cv strip
    99 86 43 85 20 00             bind
    bd 82 07 03 00 80 05 10 00 00 CALL void
    4c                            zero args
    5c 86 41 74 01                UNKNOWN opcode 5C, payload <int-TYPE> 01
    4b                            statement end
    3a fd 09  54 02  29 fd 09     return plumbing
    5e 01 21                      UNKNOWN opcode 5E, payload 01 21
    4b  4f 12 47 54 01 54 00      tail
```

* **The leading `33 …00` literal is why these land in `expr-call-in-expr`**:
  the body opens on `33`, so the straight-line arm runs `parse_expr`, pushes
  `Lit(0)`, and stops on the `26`. An ordinary base-method call (`p->Bm()`,
  probe [pE]) has no leading literal, opens on the `26`, is dispatched to the
  assignment parser, and files under `expr-intrinsic-this-adjust` instead —
  the same production split across two buckets by one leading byte.
* **`5C <int-TYPE> 01` and `5E 01 21` are UNKNOWN and byte-constant** across
  all 2,046 workload matches and the probe. HYPOTHESIS: the destructor
  flags/epilogue markers (the `01` resembles MSVC's vbase-destruct flag). A
  fixture that would separate them: a destructor of a class with a virtual
  base, where the flag payload should move. Not tested.
* **Reference codegen** [pF], workload flags:

  ```
  ??1Der@@QAA@XZ:  48000000  b ??1Base@@QAA@XZ    (4 bytes, one REL24)
  ```

  The offset-0 adjust emits nothing; the whole function is the tail branch —
  **byte-identical in form to the port's existing `VoidTailCall` emission.**

**Workload count — MEASURED [W]:** an exact matcher for this skeleton (2113 in
wide form, adjust literal 0, receiver = the bound `this` token, `2C`-strip,
void `BD`, zero args, the `5C…01`/`5E 01 21` trailers, full return plumbing) over
all 172,355 functions of the 40 sampled TUs finds **2,046 matches — 2,027 with
receiver verified = `this` — every one currently blocked as
`expr-call-in-expr`** (9.6% of the sampled bucket), spread across every large
TU (40–112 per big TU: dtors are re-emitted per TU from headers). No match had
a nonzero adjust offset — a multi-base destructor has two calls and fails the
skeleton, which is the right refusal.

---

## 6. Data-symbol address as a call argument — MEASURED [pC]

```text
ARG-ADDR := 26 <sym> [ 2C <ptr-TYPE> 00 ]              decay/convert
            [ 33 <long> <k> 28 00 00 ]                 &sym[k]
            55 <ptr-TYPE>
```

`f("hello")` pushes the string-literal symbol and decays it; `u2(gA)` decays an
array; `u2(&gA[2])` adds a scaled subscript first. Reference codegen:

```
s1: lis r11,??_C@_05CJBACGMB@hello?$AA@ ; addi r3,r11,… ; b ?f…   REFHI/REFLO + REL24
u3: lis r11,gA ; addi r11,r11,gA@l ; addi r3,r11,8 ; b ?u2…
```

Needs address materialization **and**, for string literals, emitting the
`??_C@…` `.rdata` COMDAT itself. Not decode-only.

## 7. Data-symbol reads, stores, and the plain nested call

### 7.1 Reads (2.3%) — MEASURED [pD]

`x = gS.b;`-shaped sites: `26 <dst> 26 <gS> 33 <off> 27 <TYPE> 30 <TYPE> 32 …`
(the bare `return gS.b;` form blocks in `expr-op-0x27`/`0x28` instead — again
the bucket split by statement position). Reference codegen:

```
gS.b  (off 4):  lis r11,gS  ; addi r11,r11,gS@l ; lwz r3,4(r11)
gT.a  (off 0):  lis r11,gT  ; lwz r3,gT@l(r11)                    <- REFLO folded INTO the lwz
sArr[i]:        lis ; slwi r10,r3,2 ; addi ; lwzx r3,r10,r11
```

The offset-0 form folds the REFLO into the load displacement — the neighbouring
probe pair `d1`/`d2` separates "always `addi` then `lwz 0(r11)`" from the real
rule, which is offset-dependent. Needs new codegen (REFHI/REFLO on data
symbols).

### 7.2 Stores (0.2%): `26 <dst-sym> … 2C … 32` — pointer-decay into a global.

### 7.3 The plain nested call (0.2%) — `26 <fn> BD` inside an argument region or
an RHS: `f(g(x))`, `return a + g(b)`. Two calls or a live value across a call —
frames, W11. That the *name* production is 45 sites of 21,319 is the measure of
the misnomer.

---

## 8. Decode-only vs needs-codegen, and expected yield

The port's existing emitters produce: straight-line int arithmetic over
formals, `b <callee>` tail calls (void/int/multi-arg permutation over formals),
one framed `+k` call, compare/FP leaves, `lwz` indirect-load leaves, empty
bodies. "Decode-only" below means: after decoding, the op stream lowers through
those, byte-exactly, with zero new instructions.

| # | production | decode-only? | est. functions fully in class | basis |
|---|---|---|---:|---|
| D1 | **empty-dtor base-delegation skeleton** (§5) | **yes** — routes to the existing `b <callee>` emitter; adjust 0 emits nothing | **~15k–29k** workload-wide | 2,046/21,319 sampled sites = 9.6% of a 304,813 bucket; exact whole-body matcher over 172,355 real functions; codegen witness [pF]. Range reflects the big-TU sampling bias. |
| D2 | member-call **decode** (§3, all receiver forms), still refusing emission | yes (decode) | ~0 directly | census re-attribution only — the same honest move as the intrinsic decode; splits this bucket and begins splitting the 1.13M pointer-load wall |
| D3 | **delegation acceptance**: tail member call, receiver = `this`/first formal, args = the remaining formals in declaration order | yes — identity permutation ⇒ bare `b` [pH],[pI]; needs D2 + accepting a non-int `41 <ret>` in the plumbing | **~2k–6k**, but drawn from `expr-load-type-xx43xx`, **not** this bucket | whole-body matcher: 422/172,355 sampled functions (0.245%); every variant probed emits exactly `b <callee>`; the 40 TUs all contain some |
| D4 | data reads / data args (§6, §7) | no — `lis/addi/lwz` + REFHI/REFLO on data symbols; string COMDATs | small until general | codegen witnesses [pC],[pD]; every site needs the new reloc path |
| D5 | named-object-receiver member call (§3.2) | no — D4's address materialization + the call | — | [pA] t6 |
| D6 | chained calls, computed receivers (§4) | no — frames, multiple `bl` | — | [pB] |

On the yield numbers: the project has measured repeatedly that a bucket is an
upper bound (intrinsic 2117: 149,200-function bucket, 32 in class). D1 is the
opposite case — a *shape-exact* count, not a bucket size: the matcher demands
the entire body byte-for-byte down to the return plumbing, so each match is a
function whose only barrier is the decode plus one `b` emission through
machinery that exists. The residual risks are integration, not shape: `.gl`
must bind the method token to `??1Base@@QAA@XZ` (the index accepts `?`-leading
names, `crates/c2-il/src/func/gl.rs:268`; verify on first implementation), and
the `5C`/`5E` trailers must be consumed as opaque *within this skeleton only* —
admitting them anywhere else would be the "skipped field" trap §6 of GAPS.md
warns about.

**Answer to "what to implement next": D1.** It is the smallest grammar in this
document — one rigid skeleton, every field either pinned or byte-constant
across 2,046 real witnesses + a controlled probe — it is decode-only into an
existing emitter, and it alone would retire ~10% of the #1 bucket. D2 is the
right vehicle to land it in (decode the member-call spine, accept only the D1
skeleton, refuse everything else by the usual fail-closed default).

---

## 9. Corrections to the record

1. **`expr-call-in-expr` is ~80% member calls, ~20% data pushes, 0.2% the
   nested plain call its name describes.** The "partial misnomer" warning was
   right that non-calls are filed here, but the data-push share is a fifth, not
   the bulk — and `docs/IL_EXPR_LAYER.md` §9's reading of this bucket as "a
   call consumed as a value … nearly free" describes the 0.2%, not the bucket.
   Its cheap rung (`int z = g1(a); return z;`) was already landed; what remains
   in the bucket is not that.
2. **Statement position, not construct, decides the bucket** for the whole
   member-call/designator family: the same source shape files under
   `expr-call-in-expr`, `expr-load-type-xx43xx`, `expr-intrinsic-this-adjust`,
   or `expr-op-0x27/0x28` depending on whether a `26`, a `B9`, a `33`, or an
   offset-add is what `parse_expr` reaches first. Ranking any one of these
   rows in isolation mis-ranks the production. The pointer-typed-load rows
   alone are 1,127,384 functions (47.9% of blocked).
3. **A census/gate disagreement exists today** (MEASURED, probe
   `/tmp/callprobe/probes/pG.cpp`): `int u1(){ return g(gi); }` with `gi` a
   global scalar censuses as **`int-tail-call`, in class**, while
   `c2rs diff` reports `Port=NotImplemented` (fail-closed downstream — the
   reference emits `lis; lwz r3,gi@l(r11); b g` with relocations the port
   cannot produce). Cause: the single-argument path of `parse_call_shape`
   (`crates/c2-il/src/func/body/shapes.rs:1129`) never checks that the
   argument's `Load` token is a formal, unlike the multi-arg path's
   `call-arg-nonformal` (`shapes.rs:1159`). The in-class numerator is slightly
   inflated by such functions; no wrong bytes are emitted.

---

## 10. Parser proximity (per production)

| production | nearest code | distance |
|---|---|---|
| member-call spine | `parse_call_shape` (`shapes.rs:927`) decodes `BD` fully; the `26`-vs-`BD` dispatch (`mod.rs:289`) and `rhs_is_call` (`shapes.rs:95`) already recognize callee pushes; `parse_this_token`/`read_this_group` (`shapes.rs:443,476`) already parse the *pre-body* `B9 <this> <T> 99 <T> 00` group — the same bind the body form uses | close: the bind-then-call sequencing and the method-symbol stack are new; every field reader exists |
| receiver designators | `try_parse_indirect_load_leaf` (`shapes.rs:551`) owns `B9`/`27`/`28`/`30`; `finish_indirect_load` (`shapes.rs:620`) the tail | close for loaded receivers; `26 <sym>` as a *value* designator exists nowhere |
| D1 dtor skeleton | `try_parse_base_member_load` (`shapes.rs:742`) already decodes a 2113-family sibling (selector-2117) end to end — selector, `66 02` descriptor, three `55`-terminated args | very close: same intrinsic frame, different selector + the `99…BD` tail + two opaque trailers |
| chained calls | nothing — one call per body everywhere (`parse_call_shape` returns single shapes) | far |
| data reads/args | `parse_expr` (`expr.rs:271`) has no `26` arm at all; `eat_int_like` (`readers.rs:294`) gates every load to int | far on the decode side; the reloc emitter (REFHI/REFLO pairs) exists only for `__real@` constants |
| plain nested call | `parse_call_shape` again, but only as a whole-body shape | structurally far (needs call-as-operand) |

---

## 11. Order of work (ranked by expected in-class yield, basis stated)

1. **D1 — the destructor skeleton** (§5, §8). Decode-only, ~15k–29k functions,
   basis: exact-shape count over 172,355 real functions + byte-level codegen
   witness. Prerequisite integration checks: `.gl` binding of `??1…` tokens;
   `5C`/`5E` consumed only inside the skeleton.
2. **D2 — member-call decode without acceptance** (§3). Yield ~0, but it is the
   measurement fix this bucket needs (the intrinsic-decode precedent: split the
   bucket by receiver form and enclosing statement), and D1/D3 land inside it.
3. **D3 — delegation acceptance** (§8). ~2k–6k functions, basis: 422/172,355
   shape-exact matches, all probed variants emitting one `b`. Draws from the
   pointer-load buckets, so its census effect will appear *there* — record that
   expectation now so the delta is attributable.
4. **D4 — data-symbol addressing** (§6, §7): the first genuinely new codegen
   (REFHI/REFLO on data symbols, offset folding rule per [pD], string COMDATs).
   Unlocks the 18% argument class and the 2.5% read class *jointly with* the
   enclosing-call work, and is a hard prerequisite of D5.
5. **D5/D6 — named-object receivers, chained calls, computed receivers**: W11
   proper (frames, multiple calls). The remaining ~50% of the bucket advances
   to new blockers as D1–D4 land; expect it to merge with the general-call
   ladder rather than clear.

---

## 12. Reproduction

```sh
cargo build --release -p c2-harness
# the scan + per-TU counts
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp \
  --jsonl /tmp/x.jsonl --jobs 16
# one sampled TU's bundle
./target/release/c2rs census src/App.cpp --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --keep-il /tmp/callprobe/il/src_App.cpp
# probes (sources in /tmp/callprobe/probes, flags "/nologo /c /GR /O1 /Oi /EHsc"):
#   pA member calls: t1 p->Get() · t2 p->Set(5) · t3 ref recv · t4 virtual ·
#      t5 static · t6 global obj · t7 field read · t8 int z=p->Get();return z
#   pB chains: p->Next()->Val(), triple chain, G().Val()
#   pC args: f("hello") · g(gi) · u2(&gA[2]) · u2(gA)
#   pD data reads: gS.b · gT.a · gi2 · sArr[2] · sArr[i]
#   pE two-base method calls (2113 offset 4 / 0)   pF Der::~Der(){} + gM.M()
#   pG the census/gate disagreement   pH/pI delegation + swapped-arg negative
./target/release/c2rs census probes/pA.cpp --flags-file /tmp/callprobe/flags.txt \
  --cwd /tmp/callprobe --keep-il /tmp/callprobe/pil/pA
./target/release/c2rs compile probes/pA.cpp --flags-file /tmp/callprobe/flags.txt \
  --cwd /tmp/callprobe --keep-obj /tmp/callprobe/pA.obj
python3 work/expr/tools/objdis.py /tmp/callprobe/pA.obj
```

The wide-window extractor and classifiers are scratch tooling in
`/tmp/ilstat` (a two-file crate: path-dependency on `c2-il`,
`IlBundle::load_from_dir` + `function_census`, prints TSV) and
`/tmp/callprobe/*.py`; both are regenerable from the descriptions in §1 and
the byte rules cited there.

---

## 13. D1, landed — the estimate, the outcome, and the two fields that varied

`1caf463`, 2026-07-30. Baseline for every number here is
`work/dc3-workload/scan-final2.jsonl` (878 rows, `fn_total` 2,462,571, in class
228,298 = 9.27 %, `expr-call-in-expr` 304,104); the result is
`work/dc3-workload/scan-nonleaf.jsonl` from the same list, flags and `--cwd`.

### 13.1 The estimate, stated before the outcome was measured

A **stratified 40-TU sample** (12 drawn at random from the top decile of the 841
TUs carrying the bucket, 28 uniformly from the rest, seed 20260730) was scanned
with the implementation and differenced against the baseline per TU:

| | |
|---|---:|
| sample denominator | 154,239 functions = **6.26 %** of the corpus |
| sample bucket | 18,906 sites = 6.22 % of 304,104 |
| in class, baseline → new | 14,808 → 15,751 = **+943**, 0 lost |
| extrapolation by `fn_total` | **+15,056** |
| cross-check, by bucket share (943/18,906 × 304,104) | +15,168 |

**Bias direction stated at the time: an over-estimate.** Every sampled TU carries
the bucket (37 of 878 do not and were outside the sampling frame), and 12 of 40
came from the top decile by bucket count, so the sample should have been
destructor-rich relative to the corpus.

### 13.2 The outcome, and the estimate was wrong in the direction I did not predict

| | baseline | new | delta |
|---|---:|---:|---:|
| rows | 878 | 878 | — |
| `fn_total` | 2,462,571 | 2,462,571 | 0 (no TU's denominator moved) |
| in class | 228,298 (9.27 %) | **246,162 (10.00 %)** | **+17,864** |
| `expr-call-in-expr` | 304,104 | **286,240** | **−17,864** |
| mismatch | 0 | **0** | — |
| TUs gaining / losing | — | 828 / **0** | — |
| TUs changing class | — | **0** | still 6 `match` |

**+17,864 against an estimated +15,056 — the estimate was 15.7 % LOW, and I had
predicted it would be high.** The mechanism: the sample's destructor density
*within* its bucket was 4.99 % against the corpus's 5.87 %, so stratifying on
bucket *size* picked TUs whose `expr-call-in-expr` is disproportionately the
member-call productions of §3–§6 rather than the generated destructor of §5. The
generalizable correction: **stratifying on the bucket does not stratify on the
sub-shape**, and for a whole-body estimate the right frame is the sub-shape's own
density, which is exactly what a bucket count cannot tell you. The direction of a
sampling bias is a claim like any other and this one was not measured, only
argued.

The §8 table's own estimate for D1 was "~15k–29k workload-wide", from
2,046/21,319 sampled sites; the outcome is inside that range and near its low
end.

### 13.3 The one number worth more than the yield

`expr-call-in-expr` fell by **exactly** the census gain, and **no other blocker
bucket moved by a single function** — none grew, and only this one shrank.

Every previous rung did the opposite. The `.sy` rung dropped a 554,056 bucket to
6,974 and put +17,286 in class, because 547 k functions merely cleared their
first blocker and hit the next one (`expr-call-in-expr` grew by 55 k in that very
scan). Decoding intrinsic 2117 moved 32 functions of its 149,200. Here first-blocker
attribution and in-class yield coincide, because the shape is *whole-body
complete*: the grammar accepts the entire segment from `LO` to the function tail
or nothing at all, so a function whose first blocker is this skeleton has no
second blocker to hit. That property — not the bucket size — is what made the
rung's yield predictable to within 16 % instead of within a factor of 4,600.

### 13.4 Corrections to §5 and §8 — two fields that were called constant and are not

§5 asserted that `5C <int-TYPE> 01` and `5E 01 21` are "UNKNOWN and byte-constant
across all 2,046 workload matches and the probe". **Two of those three payload
fields vary**, and both were found by probing rather than by transcribing:

1. **`5E`'s first payload counts destroyed sub-objects.** `struct N1 : M1, M2 {
   ~N1(); };` — two bases, each with a destructor — emits two member-call
   statements, the second at adjust offset `04`, and closes with **`5E 02 21`**.
   §5's guess ("the destructor flags/epilogue markers") was in the right family;
   the count is the part that is now measured, and requiring `01` is the gate that
   keeps the one-branch lowering away from the two-branch shape.
2. **Both trailers carry an exception-handling bit.** Isolating one flag at a time
   over `{/Od, /O1, /Ox} × {—, /Oi, /GS-, /GR, /EHsc, /EHa}`: **`/EH…` clears bit
   `0x10` in both**, and nothing else in that matrix moves either byte. §5's
   capture was taken at the workload's `/O1 /Oi /EHsc` and reads `5C … 01` /
   `5E 01 21`; the *fixture* profile (`/Ox /GS- /c`, no `/EH`) reads
   **`5C … 11` / `5E 01 31`** for the same source. The reference emits the same
   four bytes for both (checked at `/Ox`, `/Ox /EHsc`, `/O1`).

   Had the shape been pinned to whichever profile was probed first, it would have
   refused **either the entire workload or the entire fixture lane** — and the
   fixture lane is the only thing that grades bytes, so the second failure mode is
   a silently vacuous green. Both measured pairs are admitted, with the bit
   required to agree between them; a third value fails closed. §5's claim about
   `5C`'s low nibble stands as UNMEASURED.

`66 <n>`, the class-pair descriptor, is required to be exactly `n = 2`. Nine
D1-shaped witnesses spanning base/derived layout, an empty base, a two-level
chain, a multi-base intermediate base, a namespace-scoped base, a class-template
base and a nested-class base all carry `02`, because a destructor delegates
exactly one inheritance step. `66 03` **does** occur in the workload's
`expr-call-in-expr` sites (witness: `src/system/synth_xbox/MeterEffect.cpp`), but
at *chained* member calls, which this grammar refuses on several other counts.
Whether a base-delegating destructor can carry `66 03` is UNMEASURED; the
requirement fails closed.

### 13.5 Two gates that had to move, and one hazard that did not

* **The `0x0100` bit of the per-function optimization word is "constructor or
  destructor".** Measured one function kind at a time: a local needing cleanup, a
  `try`/`catch`, a virtual member, `throw()` and an ordinary member function all
  leave it clear; every constructor and destructor sets it, in both `/Ox`
  (`00a00105`) and `/O1` (`00200105`) forms. `PortC2` compared whole words, so
  **every constructor and destructor in the corpus was a `codegen-gap` however
  ordinary its body** — `A::~A() {}` decodes as `EmptyBody` and the reference
  emits a bare `blr` for it, identical to `void f() {}`. The bit is now masked;
  every other bit still has to match a verified word. See `docs/OPT_MODE.md`.
* **A void tail call had no arm in the `/Gy` COMDAT emitter.** It fell through to
  `int_tail_call_text`, which needs an operand stream, and refused with
  "expression did not reduce to a single value". `mvp_call.cpp` had therefore been
  a `codegen-gap` in the `/O1`, `/O2` and `/Ox /Gy` lanes for as long as those
  lanes have existed — and the workload compiles `/O1`, which implies `/Gy`.
  Fixed; those three lanes gain `mvp_call.cpp` and `w14_dtor_delegate.cpp`.
* **The hazard that is NOT new: COMDAT-by-inlineness.** A generated destructor in
  a real workload TU comes from a header, so it is *inline*, and c2 gives an
  inline function its own COMDAT `.text` **even at `/Ox`** while the TU's ordinary
  functions share a packed one. The port emits all-packed or all-COMDAT, never
  mixed, so such a TU would mis-emit if it ever came fully in class. Measured to
  be **symmetric with a pre-existing class**: an inline non-destructor whose body
  is straight-line (`struct S { int m(int a) const { return a+1; } };` under
  `__declspec(dllexport)`) produces the identical mixed layout, so this rung does
  not create the hazard. Every probe of it refuses today for other reasons; it is
  recorded here because nothing checks it.
* **Unrelated finding, not acted on.** A **template instantiation's** `.ex`
  segment ends `47 54 01 54 00 4D` — a bare module-end `4D` with no
  `4F 02 20 00 · 4F 01 <line>` before it — which `eat_fn_tail` (and therefore
  `eat_return_plumbing`, and therefore *every* shape) refuses as `module-end`.
  So `template struct D<int>;` of an otherwise-accepted destructor is out of class
  on the module framing alone. Not this rung's business; recorded as a witness.

### 13.6 What is next in this bucket

Unchanged from §11, with D1 struck: **D2** (member-call decode without
acceptance, to split the remaining 286,240 by receiver form), then **D3**
(delegation acceptance, whose census effect will appear in the
`expr-load-type-xx43xx` rows and not here), then **D4** (data-symbol addressing —
the first genuinely new codegen). D5/D6 are W11 proper.

### 13.7 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w14_dtor_delegate.cpp   # 9/9 in class
./target/release/c2rs diff   fixtures/cpp/w14_dtor_delegate.cpp   # Port=Match
C2RS_JOBS=16 scripts/mode_lane.sh /Ox     # and /O1, /O2, "/Ox /Gy": 0 mismatch
C2RS_JOBS=16 scripts/expr_sweep.sh        # checked=3009 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-nonleaf.jsonl
# the /EH bit, isolated one flag at a time:
#   for fl in /O1 "/O1 /EHsc" /Ox "/Ox /EHsc" /Od "/O1 /Oi" "/O1 /GS-" "/O1 /EHa"
#   do capture `struct B{~B();int b;}; struct D:B{~D();int d;}; D::~D(){}`
#      with --keep-il and read the `5C 86 41 74 <f>` / `5E 01 <g>` bytes
```
