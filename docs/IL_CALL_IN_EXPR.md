# The `expr-call-in-expr` bucket, decomposed

> **UPDATE (2026-07-30, `1caf463`): D1 is LANDED, and §13 records what it cost
> and what it yielded against the estimate made below.** Everything from §1 to
> §12 is the characterization as written, unedited, so the estimate can be
> compared with the outcome.
>
> **UPDATE (2026-07-30, `4e57207`): D2 is LANDED, and §14 is the bucket's actual
> decomposition — 23 named sub-buckets over the full 878-TU corpus, each with the
> fraction of it that is whole-body complete.** §14 supersedes §2's estimated
> shares (§14.4 tabulates the corrections; two are wrong by 6× and 400×) and §11's
> ranking (§14.7 replaces it). **Read §14.2 first if you are picking the next
> rung**: the split found that the `66 <n>` class-pair descriptor's refs are LEB
> ids and not the fixed pairs `shapes.rs` steps, which means D1 is refusing
> textbook base-delegating destructors in every large TU for want of a five-line
> fix.
>
> **UPDATE (2026-07-30, `a62633c`): §14.7's items (2) and (3) are LANDED together,
> and §15 is the result — the member-sub-object destructor at both a zero and a
> nonzero offset, +8,463 functions, census 11.03 % → 11.37 %, mismatch 0.** They
> were one production differing in one literal, and the bucket drop equalled the
> census gain exactly. §15.1 also **corrects §14.3's "574 recoverable"**: those
> bodies carry two destruct statements and the reference emits a *frame* with two
> `bl`s in reverse declaration order, so they are grammar-complete with both
> offsets and codegen-complete under neither. The remaining ranked work is
> §14.7 (4)–(7).

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

> **D2 is LANDED (`4e57207`, 2026-07-30) and §14 is its result.** The ranking in
> §11 survives in outline and is wrong in its order: the two destructor
> sub-shapes, not the receiver forms, are where the yield is, and §11 did not
> know they existed as separate productions.

---

## 14. D2, landed — the bucket, decomposed and weighed

`4e57207`. **Decode without acceptance**: `parse_expr` now walks the production a
`0x26` opens and names the construct, and every path still returns a refusal.
Verified against `work/dc3-workload/scan-10pct.jsonl` (878 rows, `fn_total`
2,462,571, in class 246,162 = 10.00 %, `expr-call-in-expr` 286,240) with
`work/dc3-workload/scan-d2.jsonl` from the same list, flags and `--cwd`:

| | baseline | D2 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 246,162 (10.00 %) | 246,162 (10.00 %) | **0** |
| mismatch / `match` TUs | 0 / 6 | 0 / 6 | 0 |
| TUs whose per-TU in-class count moved | — | — | **0** |
| TUs whose class moved | — | — | **0** |
| `expr-call-in-expr` keys | 1 | **23** (+6 `-whole`) | — |
| every **other** blocker bucket | — | — | **0 moved** |

Fixture lane: `bench` 109 pass / 0 fail / 0 error; `/Ox` 44 match, `/O1` 41,
`/O2` 41, `/Ox /Gy` 41, 0 mismatch in all four; `expr_sweep` checked=3009
mismatches=0; `cargo test --workspace` green.

### 14.1 The split, with the whole-body-complete fraction that ranks it

`-whole` counts the functions whose **entire segment** would parse if that one
form were admitted and nothing else changed (`mcall::whole_body_is_one_value`).
It exists because §13.3 measured that census yield tracks whole-body completeness
and not production coverage, and because a first-blocker count therefore cannot
rank these rows: D1 turned 17,864 first blockers into 17,864 in-class functions,
while the `.sy` rung turned 547,082 into 17,286. Read `-whole` as an **upper
bound**: it is a grammar measure and applies none of the codegen-class gates (no
`straight_line_is_out_of_class`, no `.sy` membership for a store destination, no
register assignment for the receiver, no `/Gy` layout, and a pointer-typed result
is admitted where the emitter has only been graded on `int`).

| sub-bucket | functions | % of bucket | whole | % whole | what it is | reference codegen for the whole-body case |
|---|---:|---:|---:|---:|---|---|
| `recv-object` | 64,905 | 22.68 % | 8 | 0.0 % | member call on a **named data symbol** (`26 <sym> [2C]`) | `lis r11,gO@ha ; addi r3,r11,gO@l ; b ?Get` **[p1 `r_named`]** |
| `data-addr` | 56,634 | 19.79 % | 1 | 0.0 % | a data symbol's **address** as a value (string literal, array decay, `&gA[k]`) | `lis ; addi r3,r11,… ; b` + the `$SG…` `.rdata` COMDAT for a literal **[p1 `a_str`/`a_arr`/`a_addr`]** |
| `recv-load` | 51,086 | 17.85 % | 1 | 0.0 % | member call on a `B9` **pointer formal/local** | **`b ?Get`** — 4 bytes, the existing tail-call emitter **[p1 `r_load`/`r_ref`]** |
| `op-0x9B` | 39,360 | 13.75 % | — | UNMEASURED | member call on a **by-value returned temporary** (`9B` binds it, opcode `0x44`/`0x64` undecoded) | frame + two `bl` **[p3 `t_byval`]** |
| `recv-intrinsic-this-adjust` | 30,888 | 10.79 % | **10,469** | **33.9 %** | member call through intrinsic **2113** — dominated by the **generated destructor delegating to a base** | `b ??1Base@@QAA@XZ` at adjust 0 (§5) |
| `recv-field` | 16,526 | 5.77 % | **2,816** | 17.0 % | receiver is a **sub-object address at a nonzero offset** (`33 <k≠0> 27 <T>`, no load) | `addi r3,r3,4 ; b ?Get` **[p3 `f_off4`, `??1HasMem4`]** |
| `recv-field-off0` | 12,995 | 4.54 % | **6,234** | **48.0 %** | the same **at offset 0** — the address arithmetic emits nothing | **`b ?Get`** / **`b ??1MemA@@QAA@XZ`** **[p3 `f_off0`, `??1HasMem`]** |
| `chained` | 8,000 | 2.79 % | 0 | 0.0 % | **two or more stacked method symbols** (`p->Next()->Val()`) | frame, one `bl` per link — W11 |
| `data-read` | 3,896 | 1.36 % | 0 | 0.0 % | a data symbol **read** (`… 30 <T>`) | `lis ; lwz r3,gO@l(r11) ; blr` **[p1 `d_read`]** |
| `recv-deref` | 1,072 | 0.37 % | 0 | 0.0 % | receiver **read from memory** (`… 27 <T> 30 <T>`) | `lwz r3,0(r3) ; b ?Get` **[p1 `r_thru`]** |
| `nested-call` | 713 | 0.25 % | 0 | 0.0 % | a plain call as a value (`26 <fn> BD`) — **the production the bucket is named after** | frame + two `bl` **[p1 `n_call`]** |
| `op-0x5C` · `op-0x99` · `op-0x67` · `op-0x64` · `op-0x05` · `op-0x09` · `op-0x59` | 127 | 0.04 % | — | UNMEASURED | honest residue: the walk met a byte it cannot tokenize | — |
| `recv-intrinsic-vbase-upcast` (15) · `-base-member-addr` (6) · `-base-downcast` (6) | 27 | 0.01 % | 0 | 0.0 % | member call through the other class-layout intrinsics | — |
| `recv-call` | 7 | 0.00 % | 0 | 0.0 % | receiver is a plain call's result (`G()->Val()`) | frame + two `bl` |
| `other` | 4 | 0.00 % | — | UNMEASURED | a decisive token reached over a value the walk did not name | — |
| **total** | **286,240** | 100 % | **19,529** | 6.8 % | | |

**The single most useful line in that table is not a count, it is the pair of
columns.** The three largest sub-buckets — 172,625 functions, 60 % of the bucket
— are **0.0 % whole-body complete**. Admitting `recv-load` alone, whose whole-body
case needs *zero new instructions*, would move on the order of **one** function.
Meanwhile `recv-field-off0` is a fifth the size and **48 %** complete. §8's
ranking by "expected in-class yield" put D5 (named-object receivers) above the
destructor shapes on the strength of the share; the completeness column reverses
that, and it is the same reversal §13.3 predicted would keep happening until it
was measured.

### 14.2 The measurement that matters more than the split: `66 <n>` is LEB, and D1 is losing functions to it

Every 2113–2119 intrinsic call carries a class-pair descriptor `66 <n> <ref>×n`.
`shapes.rs` steps it as **`1 + 2n` fixed bytes**, in `try_parse_base_member_load`
and in D1's own `try_parse_empty_dtor_delegation`. That is what every small probe
shows (`66 02 92 20 93 20`) and it is **wrong**: the refs are plain **LEB128 ids**,
and in any TU with enough types they are three bytes.

MEASURED, and found the way `GAPS.md` §6 says these are found — by a residue that
made no sense. D2's first workload scan spread **17,757 functions over 197
`op-0xNN` buckets** at 80–300 each, a flat distribution over almost the whole byte
range, which is the fingerprint of reading payload as vocabulary. Every witness was
a generated destructor from a large TU whose descriptor read
`66 02 fb 8a 01 e0 91 01` — two three-byte refs. With LEB refs the same scan leaves
**127 functions in 7 buckets**.

Three readings, and what separates them:

* **fixed 2 bytes** — agrees with `92 20`, `ad 20`, `a8 20` (every probe), and lands
  two bytes short of the following `55` on `fb 8a 01`, `e0 91 01`, `ff ff 01`,
  `d3 80 02`, `cd a5 02` (`src/App.cpp`, `src/lazer/game/Game.cpp`).
* **a `read_token_var` token** — takes `fb 8a 01 …` as *four* bytes, because byte 1
  has bit 7 set. Oversteps by one. Indistinguishable from LEB on every narrow
  witness, which is why only the wide ones settle it.
* **LEB128** — 2 bytes for `92 20`, 3 for `fb 8a 01`, and lands **exactly** on the
  `55` argument push at every witness in both TU sizes. The marker is what pins the
  width, the same way `41`/`55`/`4C 4B` pin `read_type`'s.

**The consequence, and it is the top item on the worklist.** The wild witness
`WILD_DTOR_WIDE_DESCRIPTOR` (`crates/c2-il/src/func/body/mcall.rs`,
`src/App.cpp` at the workload's flags) is D1's skeleton **byte for byte** —
selector 2113 in wide form, adjust offset 0, the `2C` strip, a void `BD`, zero
explicit arguments, `5C 86 41 74 01`, `5E 01 21`, the plumbing reaching the segment
end — and `try_parse_empty_dtor_delegation` refuses it solely because it steps the
descriptor four bytes and lands mid-ref. D1's +17,864 was therefore measured
**with this hole open**, and `recv-intrinsic-this-adjust-whole = 10,469` is its
size, as an upper bound (D2's completeness matcher does not re-apply D1's own
gates — the adjust offset being 0, the receiver being the bound `this`, the void
result, the zero argument count — so the realized figure is below 10,469).

**D2 deliberately does not fix `shapes.rs`.** Doing so changes *acceptance*, which
is the one thing this rung is not allowed to do, and the fix wants its own scan and
its own fixture. It is a ~5-line change (`eat_leb` over `n` refs, in the two
places) and it is the next rung.

### 14.3 The two destructor sub-shapes §5 did not know were separate

§5 characterized the generated destructor as delegating to its **base** through
intrinsic 2113. D2's `recv-field` rows are a **second, distinct** generated
destructor: a class with **no destructible base and one destructible member**,
whose receiver is `this + k` through a *plain* `27` byte-offset add with no
intrinsic anywhere. Controlled witnesses, both `-whole`
(`work/d2/probes/p3.cpp`, fixture profile, so the trailers read `5C … 11` /
`5E 01 31`):

```cpp
struct MemA  { ~MemA(); int a; };
struct HasMem  { ~HasMem();  MemA m; };            // member at offset 0
struct HasMem4 { ~HasMem4(); int pad; MemA m; };   // member at offset 4
HasMem::~HasMem() {}
HasMem4::~HasMem4() {}
```

```text
??1HasMem@@QAA@XZ:   4bfffff0  b ??1MemA@@QAA@XZ                       (4 bytes, one REL24)
??1HasMem4@@QAA@XZ:  38630004  addi r3,r3,4
                     4bffffe4  b ??1MemA@@QAA@XZ
```

The offset-0 form is **byte-identical in shape to what D1 already emits**, and its
IL differs from D1's only in the receiver designator: `B9 <this> <ptr> 33 <int> 0
27 <ptr> 2C <ptr> 00` where D1 has the whole 2113 intrinsic frame. Same leading
`33 <int> 0` literal, same `5C`/`5E` trailers, same plumbing.

That is why the offset is in the bucket *name*. §5's D1 required the adjust offset
to be 0 "because a base at a nonzero offset costs a real `addi r3,r3,k`", and a
`recv-field` bucket that merged the two could not say what fraction of it was
decode-only. Split: **`recv-field-off0` 12,995 / 6,234 whole (48.0 %)** and
**`recv-field` 16,526 / 2,816 whole (17.0 %)**.

One thing the split cost, recorded because it is a real fact and not an artifact:
merging the offsets, the completeness matcher counted **9,624**; split, it counts
6,234 + 2,816 = **9,050**. The **574** difference is bodies containing member
calls at *both* a zero and a nonzero offset — a destructor with two destructible
members — which need both forms and are complete under neither alone.

### 14.4 Corrections to §2's estimated shares

§2's shares came from a 40-TU cluster sample classified by byte signature, and
warned "treat every share as ±5 points". Against the full 878-TU census:

| §2 row | §2 estimate | MEASURED | verdict |
|---|---:|---:|---|
| named-object receiver | 20.7 % (+2.9 % no-convert) | **22.68 %** | confirmed |
| data-symbol address as a call argument | 17.7 % | **19.79 %** | confirmed |
| loaded receiver, incl. designator chains | 24.1 % | recv-load 17.85 % + recv-deref 0.37 % + recv-field(both) 10.31 % = **28.5 %** | confirmed in total; the *split* is new |
| class-layout-intrinsic receiver | 17.0 % | **10.81 %** | **over-estimated by 6 points** — the sample's destructor density was already known to be unrepresentative (§13.2) |
| **chained** member calls / call-result receivers | **17.7 %** | **chained 2.79 % + recv-call 0.00 %** | **over-estimated by a factor of 6.** §2's `26 26 …` byte signature counted a *named-object* receiver (`26 <method> 26 <sym>`) as a chain. D2 counts stacked **methods**, excluding a `26` followed by `BD` (a callee push, not a method) and excluding the trailing `26 <sym>` when the receiver is itself the symbol. `G().Val()` therefore has one method, not two. |
| plain nested call | 0.2 % | **0.25 %** | confirmed — the misnomer is measured |
| residue: `9B` temporary (5 sites), virtual `67` (1) | 0.03 % | **`op-0x9B` 13.75 %** | **under-estimated by 400×.** The sample saw five `9B` sites; the corpus has 39,360. A member call on a by-value returned temporary is the *fourth largest* production in the bucket. |
| data-symbol read/store | 2.5 % | **data-read 1.36 %** | halved; §7.2's store form does not reach this bucket at all (see below) |

Two structural corrections:

1. **§7.2's data store is not in this bucket.** A statement-head `26 <dst-sym>` is
   consumed by the body dispatch as an assignment destination and never reaches
   `parse_expr`, so `gS.b = a;` files under `expr-convert` / `expr-op-0x27`. D2
   has no `data-store` sub-bucket for that reason, and the four `other` functions
   are the nested case (`f(gS.b = a)`), which the walk cannot separate without a
   model of nested assignment it does not have. UNMEASURED, 4 functions.
2. **Virtual dispatch is not in this bucket either.** `x = p->VGet();` opens on
   `67` in the right-hand side and files as `expr-op-0x67` (probe p2 `v_virt`);
   the 14 `expr-call-in-expr-op-0x67` functions are a `67` met *inside* another
   expression.

### 14.5 What the keys are, and how they avoid the two instrument failures

`GAPS.md` §6 records **sharded keys** (a per-TU id in a key name splits one class
into hundreds of buckets) and **mis-attribution** (a function filed by the position
the parse stopped at, not by the construct). Both are guarded structurally, not by
intention:

* **Sharding.** `Block::ctx` is a `&'static str` and every detail lives in the
  `u32` `Block::aux`, laid out as 6 bits of form discriminant, 17 bits of payload,
  and 1 completeness bit. Nothing per-TU is *representable* in that layout. The
  walk reads operand tokens, inline TYPE ids, function-type ids and the
  class-pair descriptor's type refs — all per-TU — and none of them reaches a key.
  The only payloads that do are an intrinsic **selector** (a fixed c1xx-internal
  enum, shared across TUs, named by `intrinsic_name`) and a raw **opcode byte** in
  the residue. So the bucket count is bounded by the grammar: 23 keys over 878 TUs.
  `per_tu_identifiers_do_not_shard_the_bucket` retags a function-type id and an
  inline TYPE id in a witness and asserts the key does not move.
* **Mis-attribution.** The key is not the byte the walk ended on; it is the form of
  the **value the decisive token consumed**. A member call is filed by its receiver
  designator wherever in the statement it sits: probe `r_load` (`x = p->Get();`, an
  assignment right-hand side) and probe `r_arg` (`x = g1(p->Get());`, a
  call-argument region) are the same construct in the same bucket, and only their
  completeness bits differ. `statement_position_does_not_change_the_bucket` pins
  that. It matters because §9.2 is the same failure one level up — statement
  position, not construct, decides which bucket a whole *function* lands in — and
  repeating it inside the bucket would have measured the parser instead of the
  corpus.

The classification is a **forward width-complete walk with a backward decision**,
and that is the design's one non-obvious choice. The method symbols stack LIFO, so
`26 <A> 26 <B> 2C … 99` has `B` as the receiver while `26 <A> 26 <B> B9 … 99` has
`B` as a second method: the head run of `26` pushes cannot be split into methods
and a receiver by looking forward. The walk does not try. It remembers only the
**last value-producing token**, which is by definition the operand-stack top the
`99` binds, so the ambiguity never has to be resolved. A `2C` convert deliberately
does *not* update it — a cv-strip or pointer decay leaves the same value, and the
receiver's form is the form of what it converted.

Fields required literally, each because it never varied across the witnesses and a
field that never varied is indistinguishable from a constant: the `28`
byte-offset add's `00 00` trailer; the two `(5C, 5E)` destructor trailer flag pairs
(copied from D1, not re-derived); cdecl (`00`) as the calling convention.

### 14.6 The residue, and what is UNMEASURED

* **`op-0x9B`, 39,360 functions — the largest thing in this bucket that has no
  name.** Controlled witness (probe p3 `t_byval`, `x = GetV().Val();`): the method
  is pushed, then `9B <aggregate-TYPE> <tok>` binds a temporary, the call stores
  its by-value result into it, a second `9B` re-binds it, and opcode **`0x44`**
  sits between the cv strip and the `99`. Real sites also carry **`0x64`**. Neither
  `9B`'s role nor `44`/`64` is decoded, so the key stays hex — a hex bucket is a
  result, a guessed name is not. Its whole-body completeness is UNMEASURED: no
  grammar was written for it.
* **`op-0x5C` (81), `op-0x99` (19), `op-0x67` (14), `op-0x64` (9), `op-0x05` (2),
  `op-0x09` (1), `op-0x59` (1) — 127 functions total.** Honest residue.
* **Completeness is UNMEASURED, not zero, for** `chained`, `recv-other`,
  `intrinsic-*` (an intrinsic result consumed with no bind), `other`, `op-0x**`
  and `eof`. `mcall::form_is_measured` gates that explicitly so a missing grammar
  cannot be read as a measured 0 %.
* **Unverified claim, labelled.** `recv-intrinsic-this-adjust-whole = 10,469` is
  an upper bound on the D1-descriptor fix's yield, argued from one hand-checked
  wild witness plus the fact that D2's matcher is strictly looser than D1's
  grammar. The realized figure has not been measured and will only be known from
  the scan after the fix.
* **`recv-object`'s 64,905 at 0.0 % complete is a grammar result, not a claim
  about what those bodies contain.** They are multi-statement (the `src/system/world/Dir.cpp`
  witness has three member-call statements, an aggregate `30` load, and opcodes
  `5D`/`44`), and no attempt was made to characterize *what else* blocks them.
  That is the next measurement anyone ranking D5 should take.

### 14.7 The order of work, re-ranked by yield per unit of work

1. **`66 <n>` as LEB in `shapes.rs`** — ~5 lines, no new grammar, no new codegen,
   and it unblocks D1's existing accepted shape in every large TU. Up to **10,469**
   functions (§14.2, upper bound). Nothing else on this list has that ratio.
2. ~~**`recv-field-off0` for the generated destructor**~~ — **LANDED** with (3) as
   one rung, `a62633c`; §15. Realized **6,234 of 6,234**, i.e. the whole `-whole`
   count: every one of them was the generated destructor.
3. ~~**`recv-field` at a nonzero offset**~~ — **LANDED**, same rung. Realized
   **2,229 of 2,816**; §15.4 characterizes the 587 residual. The "574 bodies need
   it *together* with (2)" claim is **wrong** and §15.1 corrects it: those bodies
   have two destruct statements and the reference emits a frame and two `bl`s in
   reverse declaration order, so doing both offsets together does not recover
   them. Nothing short of the framed multi-destruct shape will.
4. **`recv-load` as a tail call** (D3 in §11) — its whole-body case emits a bare
   `b` with no new instructions, and its **whole-body-complete count in this bucket
   is 1**. Its yield is real but it lives in the `expr-load-type-xx43xx` rows, not
   here; §11 already said to expect the delta there, and §14.1's `-whole` column is
   the evidence that expecting it *here* would be wrong.
5. **`op-0x9B`, the by-value temporary receiver** — 39,360 functions and no
   grammar at all. The cheapest thing available is *characterization*, not
   implementation: decode `9B`/`44`/`64` far enough to split it the way this rung
   split the parent bucket, and measure its completeness before costing it.
6. **D4, data-symbol addressing** — `recv-object` 64,905 + `data-addr` 56,634 +
   `data-read` 3,896 = 125,435 functions, all needing REFHI/REFLO on data symbols
   and (for string literals) the `$SG…` `.rdata` COMDAT. It is the largest share of
   the bucket and, at 0.0 % whole-body complete, the *last* place to expect census
   movement from: the addressing is a prerequisite for those bodies, not a
   sufficient condition.
7. **`chained` (8,000) and `nested-call` (713)** — W11 proper.

### 14.8 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                     # 109 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                        # 44 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                        # 41 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                           # checked=3009 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-d2.jsonl
# the probes (gitignored scratch; sources listed in §14.1's witness column):
#   p1  receiver forms in an assignment RHS + the data-symbol pushes
#   p2  chains, call-result and intrinsic receivers, virtual, static, const object
#   p3  the offset-0 / offset-4 field receiver and both generated-destructor twins
./target/release/c2rs census work/d2/probes/p3.cpp --keep-il work/d2/il/p3
./target/release/c2rs compile work/d2/probes/p3.cpp --keep-obj work/d2/p3.obj
python3 work/expr/tools/objdis.py work/d2/p3.obj
```

Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several
reflinked worktrees with different contents, and reading one through a relative
path has already produced a published wrong number in this project.

---

## 15. D3m, landed — the member sub-object destructor, at both offsets

The rung §14.7 ranked (2) and (3), taken together because they are **one
production** differing in one literal. Baseline for every number here is
`/home/free/code/milohax/c2-rs/work/dc3-workload/scan-leb.jsonl` (878 rows,
`fn_total` 2,462,571, in class **271,557 = 11.03 %**, `expr-call-in-expr`
266,711, mismatch 0); the result is `work/dc3-workload/scan-rf.jsonl` from the
same list, flags and `--cwd`.

### 15.1 The characterization, with witnesses

§14.3 established that `recv-field` / `recv-field-off0` are a *second* generated
destructor: no destructible base, exactly one destructible **member**, receiver
`this + k` through a plain `27` byte-offset add with no class-layout intrinsic
anywhere. This rung re-captured the whole neighbourhood at the fixture profile
(`work/rf/probes/*.cpp`; every `.text` below is from `c2rs compile --keep-obj` and
disassembled, and every IL byte string from `census --keep-il`).

**MEASURED — the shape, and the offset is the only thing that varies.** Segments
`p3`, `q4`, `q7`, `q8` are byte-identical from the `2C` cv-strip onward to
`DTOR_DELEGATE`'s; the receiver is the whole difference:

```text
   33 86 41 74 00                LIT int 0        the leading literal (as D1)
   26 <??1MemA>                  the MEMBER's destructor, pushed first
   b9 <this> a6 43 81 20         the object pointer -- NO intrinsic frame
   33 86 41 74 <k>               LIT int k        the member's byte OFFSET
   27 a6 43 8a 20                byte-offset add -> the member's address
   2c a6 43 8b 20 00 · 99 … 00 · bd 82 07 03 00 <id> · 4c
   5c 86 41 74 11 · 4b · 3a <l> 54 02 29 <l> · 5e 01 31 · 4b · <fn tail>
```

| witness | source | `.text` |
|---|---|---|
| `p3` h0 | `struct HasMem { ~HasMem(); MemA m; };` | `b ??1MemA@@QAA@XZ` |
| `p3` h4 | `struct HasMem4 { ~HasMem4(); int pad; MemA m; };` | `addi r3,r3,4 ; b ??1MemA` |
| `q7` | member's member, offset 8 | `addi r3,r3,8 ; b ??1Inner` |
| `q8` | a 124-byte member, offset 8 | `addi r3,r3,8 ; b ??1BigMem` |
| `q4` | **non-destructible base** + member at 4 | `addi r3,r3,4 ; b ??1MemA` |
| `q6` | a **`const`** member at 0 and at 4 | `b ??1MemA` / `addi r3,r3,4 ; b ??1MemA` |
| `q5` | a member whose destructor is **virtual** | `b ??1MemV@@UAA@XZ` |

`q5` is the load-bearing one and it settles a claim the shape rests on: destroying
a member sub-object of *known* type is a **direct** call even when the member's
destructor is virtual. The bind is `99`, not `67`/`9A`, and c2 emits a plain
REL24 branch to `??1MemV@@UAA@XZ`. The licence to branch comes from the bind, not
from the callee — so the gate stays on the bind and does **not** need to inspect
the callee's virtualness.

**MEASURED — the offset gate, at the boundary.** `char pad[k]` before the member,
one TU per `k`:

```text
   k = 4, 8, 12, 32760, 32764   ->  addi r3,r3,k ; b            ACCEPTED
   k = 32768                    ->  addis r3,r3,1 ; addi r3,r3,-32768 ; b
   k = 32772                    ->  addis r3,r3,1 ; addi r3,r3,-32764 ; b
   k = 65532                    ->  addis r3,r3,1 ; addi r3,r3,-4    ; b
   k = 65536                    ->  addis r3,r3,1 ; b                     (!)
```

So the switch is at exactly the signed-16-bit edge, and past it there are **two**
further productions (`addis`+`addi`, and a bare `addis` when the low half is
zero), each with one witness. All three are refused; `k` is required to be
`0 ≤ k ≤ 32767`. The wide literal arrives in the escape spelling
(`33 86 41 74 80 <4 LE bytes>`), which `read_varint` already handles — a fixed
one-byte read would have desynced rather than refused.

**MEASURED — and this is the correction to §14.3's "574 recoverable".** A class
with **two** destructible members carries `5E 02` and **two** statements, each
with its own leading `33 <int> 0` literal, and the reference does *not* emit two
branches (`work/rf/probes/q1.cpp`, `struct Two { ~Two(); MemA m; MemB n; };`):

```text
   ??1Two@@QAA@XZ:  mfspr r12,r8 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
                    or r31,r3,r3           <- `this` saved: it is LIVE across the call
                    addi r3,r3,4 ; bl ??1MemB@@QAA@XZ      <- the SECOND member FIRST
                    or r3,r31,r31 ; bl ??1MemA@@QAA@XZ
                    addi r1,r1,96 ; lwz r12,-8(r1) ; mtspr … ; ld r31,-16(r1) ; blr
```

A frame, a callee-saved register, and the calls in **reverse declaration order**.
§14.3's 574 bodies are grammar-complete once both offsets are admitted and
codegen-complete under neither, so **this rung deliberately does not chase them**
and the reachable target is 9,050, not 9,624. They are refused twice over (the
`5E 01` count, and reaching the segment end) and swept as neighbours.

**MEASURED — the other refusing neighbours.** An **array** member is a destruct
*loop* plus a `??_I@…` helper function and blocks on `op-0x5C`, a different bucket
(`q2`). A member with no destructor leaves the body empty (already in class as
`empty-body`). A member pointer or reference is not a sub-object and is not
destroyed.

### 15.2 The estimate, stated before the outcome was measured

Per §13.2's lesson, the estimate is of **the fix**, not the finding. Every site
that implements the rule being changed, from a grep of the acceptance path:

1. `shapes::try_parse_empty_dtor_delegation` — the **only** acceptance site, and
   the only one that gates. Both receiver productions are alternatives inside it.
2. `body::parse_segment_shape`'s `0xB9 | 0x33` arm — the only dispatch that can
   reach it. Both forms open on the same leading `33 <int> 0`, so there is exactly
   one entry point and no statement-position twin of the §5 kind.
3. `mcall::eat_receiver` (D2's completeness matcher) and `census.rs`'s labels —
   diagnostic only, no gate.
4. `bundle.rs`'s lowering.

The candidate population is therefore bounded by the two buckets, because a body
whose parse stopped before the `26` is filed elsewhere and this change cannot
reach it. Upper bound = the two `-whole` counts:

| | |
|---|---:|
| `recv-field-off0-whole` (the zero-offset member form) | 6,234 |
| `recv-field-whole` (the nonzero-offset member form) | 2,816 |
| **upper bound** | **9,050** |

**Bias direction stated before the outcome: an over-estimate, and this one is
structural rather than argued.** The accepted grammar is a *strict subset* of the
matcher's for these two forms — the matcher applies none of: the leading `33 <int>
0` literal, the `5C`/`5E` trailer flag table, `5E`'s count being exactly 1, the
void result, zero explicit arguments, the receiver being positively the bound
`this` with no other formal, `27` rather than `28`, and `0 ≤ k ≤ 32767`. Every one
of those subtracts. Nothing in the change can admit a body outside those buckets.
**Point estimate 6,500 ± 1,500 in class.**

Second prediction, and the one worth more (§13.3): **the bucket drop should equal
the census gain exactly, and no other bucket should move.** The grammar accepts a
whole segment or nothing, and the shape is non-committal — a declining body falls
through to `parse_expr` and reports the identical blocker — so a function that
leaves `recv-field*` can only have gone in class, and one that stays cannot have
moved anywhere else.

### 15.3 What was implemented

`try_parse_empty_dtor_delegation` gained a second receiver production and lost
nothing: the 2113 frame moved out to `eat_dtor_base_receiver` verbatim, the new
`eat_dtor_member_receiver` sits beside it, and everything from the `2C` strip to
the function tail is shared. The two are tried in order on a cursor copy, so
neither can leave the other mid-token.

`BodyShape::EmptyDtorDelegation` now carries `this_tok`, `adjust` and
`sub_object: Base | Member`. `sub_object` is recorded rather than inferred from
`adjust`, because a member at offset 0 and a base at adjust 0 emit the **identical
four bytes** — the emitter cannot tell them apart and only the census wants to.
`census.rs` therefore reports three labels (`empty-dtor-delegation`,
`empty-dtor-member`, `empty-dtor-member-adjusted`) so the in-class gain can be
checked against the individual bucket drops instead of their sum.

**There is no new emitter.** At `adjust == 0` `bundle.rs` produces the same empty
`params`/`ops` D1 produced, so that path is byte-identical to before. At a nonzero
adjust it hands codegen `params = [this]`, `ops = [Load(this), Lit(k), Add]` —
literally `return g(this + k)`, which `int_tail_call_text` has lowered to
`addi r3,r3,k` + `b` since the MVP and which four mode lanes and the expression
sweep have been grading ever since. The parser's `0 ≤ k ≤ 32767` bound is exactly
the range that selector folds into one `addi`.

Fixture `fixtures/cpp/w15_dtor_member.cpp`, 10 functions, all in class and
`Port=Match`: offsets 0/4/8, a `const` member, a virtual-destructor member, a
member's member, a 124-byte member, a non-destructible base, braces on their own
lines (the line-70 formals-anchor hazard), and offset 32,764 — the top of the
accepted range.

`scripts/expr_sweep.sh` gained the cross product **member size × leading padding ×
cv-qualification** (5 × 14 × 3), the same offsets reached through a
non-destructible base, a member's member, a virtual-destructor member, the source
line 64–76, and 19 refusing neighbours: two/three destructible members at several
offset pairs, arrays of 2 and 3, a member with no destructor, a member pointer and
reference, an inlinable member destructor, a virtual enclosing destructor, a
virtual base, a template instantiation, and a constructor of the same class.
**checked=3259 mismatches=0** (3009 before).

### 15.4 The outcome, against the estimate

| | baseline (`scan-leb`) | result (`scan-rf`) | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 271,557 (11.03 %) | **280,020 (11.37 %)** | **+8,463** |
| `expr-call-in-expr-recv-field-off0-whole` | 6,234 | **0** | **−6,234** |
| `expr-call-in-expr-recv-field-whole` | 2,816 | **587** | **−2,229** |
| every **other** bucket, including both bare `recv-field*` | — | — | **0 moved** |
| mismatch | 0 | **0** | — |
| TUs gaining / losing | — | 806 / **0** | — |
| TUs changing class | — | **0** | still 6 `match`, 7 `capture-fail` |

**+8,463 against a point estimate of 6,500 ± 1,500 and an upper bound of 9,050.**
The bound held — 93.5 % of it was realized — and the point estimate was **30 %
low**. That is the second rung in a row where the whole-body estimate came in
under: D1 was 15.7 % low (§13.2). The generalizable correction is that the
deductions I listed in §15.2 sound like they should each cost something and
measurably cost almost nothing, because they are all gates on fields that a
*compiler-generated* body has no freedom in. The next whole-body estimate should
be quoted **at the `-whole` bound, minus only the deductions with a measured
population**, not at a hedged fraction of it.

**The bucket drop equals the census gain exactly: 6,234 + 2,229 = 8,463.** So does
the split by receiver production, which is why `census.rs` reports three labels:
the zero-offset member form and the adjusted one land in their own in-class
buckets and each matches its own bucket's drop. Nothing else moved by a single
function — the two bare buckets are unchanged at 6,761 and 13,710, every other
`expr-call-in-expr` sub-bucket is unchanged, and no blocker outside the bucket
moved. This is the D1 property again (§13.3) and for the same reason: the grammar
accepts a whole segment or nothing, and it is non-committal, so a declining body
falls through to `parse_expr` and reports the identical blocker.

**`recv-field-off0-whole` went to exactly zero.** All 6,234 were the generated
destructor. That is a stronger result than the nonzero form's 79 %, and the
asymmetry is worth recording because it is not explained by anything in the
grammar: the offset-0 and nonzero paths share every gate but the offset itself.

**The 587 residual — what it is not, and what it is.**

* **NOT the `addi` range.** Tested and refuted: every offset literal followed by a
  `27` in the residual TUs' `.ex` is ≤ 17,016 (`src/system/char/Character.cpp`,
  628 escape-form and 5,111 short-form literals scanned), well inside the accepted
  0…32,767. The `addis`+`addi` production this rung refused does not occur in the
  workload at all.
* **Witnesses for what it is** (hand-read from `Character.cpp`, whose residual is 5
  of 12). Two of its three destructor-shaped nonzero-offset statements are
  followed, where the return plumbing must begin, by:
  `4c 5c 86 41 74 01 4b · 26 <tok> b9 <tok> …` — a **second destruct statement**,
  and `4c 5c 86 41 74 01 4b · 4f 01 2f 53 26 <tok> 33 86 41 74 00 32 86 41 74` — a
  destructor with a **real store statement** in its body. Both are refused for the
  reasons the grammar was built to refuse them, and both really do emit more than
  one branch.
* **A third source, MEASURED to exist but not counted.** A plain
  non-destructor member call on a sub-object is also `-whole` under D2's matcher:
  probe `r1`, `struct C4 { int pad; M m; int f(int) const; };
  int C4::f(int a) const { return a + m.Get(); }`, files as
  `recv-field-whole` and needs a frame. Its offset-0 twin files as
  `recv-field-off0-whole`, so the same shape exists on both sides of the split and
  cannot by itself explain the asymmetry above.
* **The split between those three is UNMEASURED**, and so is the asymmetry. The
  plausible reading — that a class's *first* member is usually a scalar while the
  sub-object you call methods on comes later, so non-destructor `-whole` bodies
  concentrate at nonzero offsets — is a HYPOTHESIS with no measurement behind it.
  A per-body classifier over the residual TUs would settle it; nothing here did.

### 15.5 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
./target/release/c2rs census fixtures/cpp/w15_dtor_member.cpp # 10/10 in class
./target/release/c2rs diff   fixtures/cpp/w15_dtor_member.cpp # Port=Match
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                     # 110 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                        # 45 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                        # 42 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                           # checked=3259 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-rf.jsonl
# the probes (gitignored scratch; sources in §15.1's witness table):
#   p3  the offset-0 / offset-4 twins            q5  a virtual-destructor member
#   q1  TWO destructible members (the frame)     q6  a const member at both offsets
#   q2  an array member (a destruct loop)        q7  a member's member
#   q3  a member at offset 40,000                q8  a 124-byte member
#   k<N> one TU per offset N, across the addi boundary
./target/release/c2rs census work/rf/probes/q1.cpp --keep-il work/rf/il/q1
./target/release/c2rs compile work/rf/probes/q1.cpp --keep-obj work/rf/q1.obj
python3 work/expr/tools/objdis.py work/rf/q1.obj
```

Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several
reflinked worktrees with different contents, and reading one through a relative
path has already produced a published wrong number in this project.

---

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
