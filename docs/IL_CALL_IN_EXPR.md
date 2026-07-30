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
>
> **UPDATE (2026-07-30): D4 is LANDED and §16 supersedes §14.7's ranking.** It
> walks *past* each receiver form and names what blocks the body **next**, plus how
> many constructs the body needs in all (up to four). Acceptance is unchanged —
> census 280,020, per-TU in-class identical, four lanes identical, mismatch 0 — and
> the sub-buckets sum to §14.1's parents exactly, with 19 functions re-decomposed
> out of `op-0x99` and accounted for individually. **Read §16.1 first**: the three
> 0.0 %-complete forms do *not* share a second blocker, only 21.0 % of the bucket
> is reachable within four constructs, §14.7 (6) goes from last to **first** on a
> measurement (21,642 bodies), and §16.4 finds that D2's own form classification
> undercounts member-call chains by **4.4x** for the same statement-position reason
> §9.2 records.

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

## 16. D4, landed — the **second** blocker, per receiver form

`docs/IL_CALL_IN_EXPR.md` §14.6 stated the limit D2 knowingly left open:
*"`recv-object`'s 64,905 at 0.0 % complete is a grammar result, not a claim about
what those bodies contain … no attempt was made to characterize WHAT ELSE blocks
them. That is the next measurement anyone ranking D5 should take."* Three forms —
`recv-object` 64,905, `data-addr` 56,634, `recv-load` 51,086 — hold **172,625
functions at 0.0 % whole-body complete**, 64 % of the bucket, and a completeness
column that reads 0.0 for its three largest rows has no ordering information left
in it.

This rung answers it. **Decode without acceptance again**: the completeness
matcher now walks *past* the receiver form, names the construct that blocks the
body next, and then grants constructs greedily — up to four — to say **how many**
the body needs. Every path still returns a refusal.

Baseline `/home/free/code/milohax/c2-rs/work/dc3-workload/scan-1137.jsonl` (878
rows, `fn_total` 2,462,571, in class **280,020 = 11.37 %**, `expr-call-in-expr`
268,140, mismatch 0); the result is
`work/dc3-workload/scan-sb.jsonl` from the same list, flags and `--cwd`.

| | baseline | D4 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 280,020 (11.37 %) | 280,020 (11.37 %) | **0** |
| TUs whose per-TU in-class count moved | — | — | **0** |
| TUs whose class moved | — | — | **0** (still 6 `match`, 7 `capture-fail`) |
| mismatch | 0 | 0 | 0 |
| **non-`mcall`** census keys / their sum | 924 / 1,914,411 | 924 / 1,914,411 | **0 / 0** |
| `expr-call-in-expr` total | 268,140 | 268,140 | **0** |
| `expr-call-in-expr` keys | 28 | **1,355 − 924 = 431** | — |

Fixture lane: `bench` 110 pass / 0 fail / 0 error; `/Ox` 45 match, `/O1` 42,
`/O2` 42, `/Ox /Gy` 42, 0 mismatch in all four; `expr_sweep` checked=3259
mismatches=0; `cargo test --workspace` green.

**Exactly 19 functions were re-decomposed, and they are accounted for one by
one.** `walk` now tokenizes a `99 <T> 00` member bind met *inside* an open
call-argument region (at depth 0 a `99` is decisive and returns before that arm),
without which it cannot see a member call nested in an argument list and files the
whole production as `op-0x99`. That bucket goes to **0** and its 19 functions
land in `recv-intrinsic-this-adjust` (+16), `op-0x9B` (+1), `chained` (+1) and
`other` (+1). Every other parent is unchanged to the function:

| parent | baseline | D4 |
|---|---:|---:|
| `recv-object` · `data-addr` · `recv-load` · `recv-field` · `recv-field-off0` · `data-read` · `recv-deref` · `nested-call` · `op-0x5C` · `recv-intrinsic-*` · `recv-call` · `op-0x67` · `op-0x64` · `op-0x05` · `op-0x09` · `op-0x59` | — | **unchanged** |
| `op-0x99` | 19 | **0** |
| `recv-intrinsic-this-adjust` | 21,251 | 21,267 |
| `op-0x9B` | 39,360 | 39,361 |
| `chained` | 8,000 | 8,001 |
| `other` | 4 | 5 |

### 16.1 The answer, stated plainly: they do **not** share one second blocker

The rung existed to distinguish two worlds — *"if those bodies share **one**
further blocker, then that blocker plus the receiver form is the largest rung
left; if each carries three unrelated ones, they are far off"*. **MEASURED: it is
neither, and the reading differs per form.** The three giants have three
*different* dominant second blockers, and only **15,962 of their 172,625
functions (9.2 %) are one construct away** from whole:

| form | total | dominant second blocker | its share | **1 away** | within 4 |
|---|---:|---|---:|---:|---:|
| `recv-object` | 64,905 | `type-ptr` 27,992 / `branch-0x39` 23,632 | 43 % / 36 % | 2,418 (3.7 %) | 11,084 (17.1 %) |
| `data-addr` | 56,634 | **`plain-call` 55,925** | **98.8 %** | 1,063 (1.9 %) | **22,357 (39.5 %)** |
| `recv-load` | 51,086 | **`chain-bind` 29,001** | 56.8 % | **12,481 (24.4 %)** | 14,697 (28.8 %) |

So: `data-addr` is almost perfectly concentrated on one second blocker and almost
none of it completes with the pair; `recv-load` is half concentrated and a quarter
of the whole form completes with the pair; `recv-object` is split between a cheap
construct and **control flow**, which is not a rung at all. Reporting a single
"dominant second blocker" for the bucket would have been the wrong shape of
answer, which is why the deliverable is a distribution.

The grant-count distribution over the whole bucket — this is the number that
decides where effort goes:

| extra constructs the body needs | functions | share of 268,140 |
|---|---:|---:|
| **0** — the form alone finishes it (D2's `-whole`) | 1,429 | 0.5 % |
| **1** — the pair finishes it | **22,645** | 8.4 % |
| **2** | **25,588** | 9.5 % |
| 3 | 4,528 | 1.7 % |
| 4 | 2,152 | 0.8 % |
| **> 4, or the chain hit something unmodelable** | 120,514 | 44.9 % |
| the second blocker itself has no production (UNMEASURED pair) | 51,810 | 19.3 % |
| the **form** has no production (D2 residue, mostly `op-0x9B`) | 39,474 | 14.7 % |
| **reachable within four constructs** | **56,342** | **21.0 %** |

### 16.2 The construct vocabulary, and the witness for each

`GAPS.md` §6's mis-attribution failure is the hazard, and the guard is structural:
the key names a **construct** at the refusal, never the byte or the position. Six
constructs are new here; the rest reuse `Block::feature`'s capture-verified opcode
table (now shared, `body::expr_opcode_name`, so the `expr-*` and `mcall` families
cannot disagree about what a byte is called).

| construct | key | functions | witness | status |
|---|---|---:|---|---|
| conditional branch `<op> <label>` | `branch-0x38` / `branch-0x39` | 23,644 + 87 | probe `b_if` (`if (gO.Ok())`): `4C 38 00 0A … 29 00 0A` — **the label is defined later in the same segment**; two wild ones in `Ham.cpp` incl. `… 0b 39 2b 67` = `if (x & 1)` | **MEASURED** as a branch. `38` is taken when the condition is **false** (one probe). `39`'s polarity is **UNDETERMINED** — hence the byte in the key |
| chain link `99 <T> 00 <call>` | `chain-bind` | 35,520 | probe `c_ret` + wild `Mesh.cpp` | **MEASURED** |
| bare CALL token over an already-pushed callee | `plain-call` | 55,925 | probe `a_str` (`uc("hi")`) | **MEASURED** |
| byte-offset add outside a receiver designator | `off-add` | 11,211 | wild `Ham.cpp` destructor-with-`delete` | **MEASURED** |
| a pointer-typed operand in a **call-argument region** | `type-ptr` | 42,332 | wild `Mesh.cpp` `b9 <t> 86 43 83 20` inside a plain call's args | **MEASURED** |
| `9B <T> <tok>` by-value temporary bind | `temp-bind` | 1,404 | D2's `BYVAL_TEMP` | decoded, **no production** — UNMEASURED pair |

The full second-blocker ranking across all forms, with how much of each is
reachable:

| second blocker | functions | whole within 4 | note |
|---|---:|---:|---|
| `plain-call` | 55,925 | **21,648** | the data-symbol-address-into-a-call shape |
| `type-ptr` | 42,332 | 2,413 | a **measure-visible gate of the modeled class**: `eat_int_operands` takes `int` only, while the statement level already takes pointers |
| `chain-bind` | 35,520 | **18,460** | member-call chains |
| `branch-0x39`/`-0x38` | 23,731 | **0** | control flow. No production, by design |
| `call-nested-call` | 12,902 | 8,885 | a plain call reached through an argument region |
| `off-add` | 11,211 | **2** | almost worthless alone — see the `delete` witness |
| `call-op-0x9B` | 9,041 | 0 | a nested by-value-temporary call |
| `type-int1` | 5,438 | 2,215 | `bool`/`char` |
| `plumbing-0x3A` | 4,634 | 0 | the return tail is not the modeled one |
| `intrinsic-call` (`0x40`) | 4,004 | 0 | |
| `call-recv-load` · `call-recv-field` | 3,974 · 3,659 | 1 · 1,279 | |
| `type-real` | 2,553 | 2 | |
| `cmp-*` (`lt`/`eq`/`gt`/`ge`/`ne`/`le`) | 3,772 | 0 | a comparison, i.e. control flow's operand |
| `op-0x5C` | 890 | 0 | a destructor statement trailer whose flag is neither measured value |
| `op-0x80` (579) · `op-0x08` (234) | 813 | 0 | **uncharacterized, and labelled: possible desync** (§16.6) |

**The residue is not a payload being read as vocabulary.** §14.2's third caution
is that a long flat tail of `op-0xNN` buckets means the parser is wrong, not that
the vocabulary is large — that is how the `66`-descriptor LEB bug surfaced, at
17,757 functions over 197 buckets at 80–300 each. D4's second-blocker residue is
**24 distinct `op-0xNN` bytes over 11,245 functions (4.2 % of the bucket), and
9,041 of them are one byte** (`0x9B`, a known construct), with the next three at
899 / 579 / 234 and the remaining twenty at ≤ 155. That is a concentrated
distribution, not a flat one.

### 16.3 The three named pairs a rung could be built on

Ranked by **how many bodies are whole-body complete once every construct in the
row is handled** — the number §13.3 and §15.4 established as the one that tracks
census yield:

| rank | form × second [× third] | k | whole | what it is |
|---|---|---:|---:|---|
| **1** | `data-addr` × `plain-call` × **`type-ptr`** | 2 | **20,579** | a global's or string literal's address passed to an ordinary call that also takes pointers — **§17 measures this row as 87.5 % *two* addresses per call, which is a different lowering; see §17.1** |
| | `data-addr` × `plain-call` | 1 | 1,063 | the same with int-only arguments |
| | | | **21,642** | **row 1 total** |
| **2** | `recv-load` × `chain-bind` | 1 | **12,480** | `p->A()->B()` — a two-link chain on a pointer formal |
| | `recv-field-off0` × `chain-bind` | 1 | 2,666 | the same off a sub-object at offset 0 |
| | `recv-intrinsic-this-adjust` × `chain-bind` | 1 | 1,686 | the same off a base sub-object |
| | `recv-field` × `chain-bind` | 1 | 835 (+68 at k=2) | the same at a nonzero offset |
| | `data-addr` × `chain-bind` × `type-real-lit` | 3 | 714 | |
| | | | **18,449** | **row 2 total — chains, across five receiver forms** |
| **3** | `recv-object` × `call-nested-call` × `call` | 2 | 4,904 | a named-object receiver plus a plain call, both nested |
| | the same | 3 | 2,335 | |
| **4** | `recv-object` × `type-ptr` | 1 | 2,410 | a named-object member call with a pointer argument |
| **5** | `recv-intrinsic-this-adjust` × `call-recv-field` | 1 | 705 | a base-delegating destructor that also destroys a member |
| | `recv-field-off0` × `call-recv-field` | 1 | 574 | |
| **6** | `recv-load` × `type-int1` × `type-aggregate` | 3 | 1,472 | |
| **7** | `recv-field` × `call-nested-call` | 1 | 216 | |

**§14.7 (6) is wrong, and this is the measurement that corrects it.** It ranked
data-symbol addressing **last** — *"the largest share of the bucket and, at 0.0 %
whole-body complete, the last place to expect census movement"*. That was a true
statement about the form **alone** and a misleading one about the *row*: paired
with the plain call it opens and the pointer arguments that call takes, it is the
**largest reachable block in the bucket**, at 21,642. The 0.0 % reading was not
wrong, it was one-dimensional.

### 16.4 The finding nobody was looking for: `chained` undercounts chains by ~4.4×

The `chained` sub-bucket is 8,001 functions. **`chain-bind` appears as a second
blocker 35,520 times**, and the two are the *same source construct*. Controlled
witness, one probe pair, two functions differing only in the presence of an
assignment destination:

```cpp
int c_ret(O* p) { return p->Next()->Get(); }          // -> recv-load-then-chain-bind-whole
int c_asg(O* p) { int x; x = p->Next()->Get(); return x; }   // -> chained-whole
```

Both are `26 <Get> 26 <Next> B9 <p> 99 … 4C 99 … 4C`, byte for byte, two links.
But `mod.rs`'s statement dispatch treats a statement-head `26 <tok>` as an
**assignment destination**, so in the return-position form it eats the outer
method push and `parse_expr` starts one `26` later — where exactly one method is
stacked, which is `recv-load`. §9.2 recorded that "statement position, not
construct, decides which bucket a whole function lands in"; §14.5 guarded against
repeating it *inside* the bucket, and it turns out D2's **form classification
itself** still has it, one level up, for every chain in a value position.

So the chain population is `chained` 8,001 + 35,520 = **43,521 functions, 16.2 %
of the bucket** — the second largest production in it after `recv-object`, and
D2's table put it at 2.8 %. §14.4 already caught §2 over-estimating chains by 6×;
this is the correction in the other direction.

D4 also gives `chained` the production D2 left unwritten (§14.6 listed its
completeness as UNMEASURED), so the form is now measured. **MEASURED and worth
recording: `chained-whole` is 0 on the workload** — 7,895 of the 8,001 block next
on `type-ptr` and then on `off-add`. Real chains carry pointer arguments; the
probe's `int`-only chain is not representative, which is §14.2's fourth caution
again.

### 16.5 What the instrument is, and its one asymmetry

* **Furthest-refusal, not first-refusal.** The matcher speculates: at every value
  position it tries the form's production first, and that attempt walks *into* a
  call before finding the byte it cannot take. `Fail` keeps the deepest position
  reached, so a body whose second statement is `q->o.Get()` records the refusal at
  the sub-object address inside that call and files as `then-call-recv-field`, not
  as a useless `then-op-0x26`.
* **A `26` at the refusal is re-classified by `walk`**, D2's own backward
  classifier, so a second blocker that is another member call is filed by *its*
  receiver designator. A `26`-opened production whose *receiver* is wrong carries
  the `26`'s own offset (`FailKind::Receiver`), because the receiver's first byte
  is `2C` for a decayed string literal, `26` for a named object and `9B` for a
  by-value temporary — three constructs that would otherwise become three
  uninformative opcodes.
* **The sharding gate holds.** `Block::aux` widened to `u64` (6 bits form + 17
  payload + 1 whole + 6 blocker + 23 blocker payload + 3 grant count + 5 third-kind
  = 61 bits) rather than being squeezed, because truncating an intrinsic selector
  would silently **merge** two census buckets. Nothing per-TU is representable:
  the only payloads are an intrinsic selector, a type *class*, an opcode byte and
  a structural sub-kind. 431 keys over 878 TUs.
* **The honesty gate composes.** `form_is_measured` gates the form; `blocker_is_measured`
  gates the pair. A key with **neither** `-whole…` nor `-more` means the pair's joint
  completeness does not exist as a number — it is not a measured incompleteness.
  `branch`, `temp-bind`, `virtual`, `intrinsic-call`, `cmp-*`, `plumbing`, the
  structural kinds and every `op-0xNN` are in that class: 51,810 functions.
* **The one asymmetry, stated because it biases a number.** The first pass (D2's
  `-whole`) uses D2's argument grammar verbatim — int-like operands only — so every
  `-whole` count in §14.1 and §15.4 is reproduced exactly. The second and later
  passes additionally admit the granted constructs *inside* argument regions,
  because `gO.Set(p->Get())` blocks on a receiver form in an argument and a measure
  that refused arguments would report `-more` for a body that only ever needed the
  two. So `-whole<k>` is measured against a marginally looser frame than `-whole`,
  in the argument-nesting dimension only. Both remain grammar upper bounds with no
  codegen gate applied (§14.1's warning stands in full).
* **The greedy chain is bounded at four grants** and stops early on an unmodelable
  construct or on a construct that repeats (which would mean a production failed to
  consume what the classifier named — a bug, not a body).

### 16.6 What is NOT measured, labelled

* **`op-0x9B`, 39,361 functions — still UNMEASURED at both levels, and now the
  single largest unnamed thing in the census.** §14.6's characterization stands
  (`9B <aggregate-TYPE> <tok>` binds a by-value returned temporary; `0x44`/`0x64`
  sit between the cv strip and the bind and are undecoded), and D4 deliberately did
  **not** decode it: making `9B` a receiver *form* requires the walk to treat a
  depth-0 `32` store as non-decisive mid-statement, which would move functions
  between D2 sub-buckets on a hypothesis rather than a measurement. Its shape is
  now visible from the other side too — `call-op-0x9B` is a second blocker 9,041
  times, 8,343 of them in `recv-intrinsic-this-adjust` — so a rung that decodes it
  buys information in two places at once. **This is the largest thing on the
  worklist whose size is not yet known.**
* **`op-0x80` (579) and `op-0x08` (234) — possible desync, uncharacterized.**
  Neither byte appears as a first blocker anywhere else in the census, only as a
  D4 second blocker inside `recv-intrinsic-this-adjust`, and `0x80` is the varint
  escape byte. 813 functions, 0.3 % of the bucket. The residue *shape* argues
  against a systemic problem (§16.2) but this is a labelled unknown, not a clean
  bill of health.
* **`type-ptr`'s 42,332 is a gate of the *measure* as much as of the port.**
  `eat_int_operands` accepts int-like operands only, while `eat_admitted_type` at
  the statement level already accepts pointers, so a pointer argument refuses in
  one place and not the other. The narrower rule is the accepted class's
  (`IntTailCall` and friends are graded on `int`), so the count is honest — but a
  pointer argument is register-allocated exactly like an int, and this is the
  cheapest 42,332-function construct in the table.
* **`recv-object`'s two dominant rows are unreachable in different ways.**
  `branch-0x39` at 23,632 needs basic blocks, a register allocator across them and
  a `/Gy` layout — a phase, not a rung, and no production was written for it on
  purpose. `type-ptr → call → …` at 19,645 is `-more`: three constructs deep and
  still blocked.
* **The polarity of `branch-0x39`** is undetermined (one probe pins `0x38` as
  branch-if-false; `0x39`'s two wild witnesses establish that it is a branch, not
  which way). A rule with one or two witnesses is a guess, and this one is
  labelled.
* **`off-add`'s 11,211 with 2 reachable is the clearest "looks reachable and is
  not" in the table**, and the witness says why: `~X() { … delete mThing; }` needs
  an off-add, a `30` indirect load, a conditional branch and a pointer store, in
  that order, and the branch stops the chain.

### 16.7 The order of work, re-ranked

> **Superseded by §17.6.** Item 1 below was taken, and the row is not what this
> ranking says it is: 87.5 % of it passes **two** symbol addresses to one call, which
> c2 lowers through a `.rdata`-pool-relative selection and a scheduler, not through a
> second relocation pair. Read §17 before acting on item 1.

1. **`data-addr` + `plain-call` + pointer arguments** — **21,642** grammar-complete
   bodies (1,063 at k = 1, 20,579 at k = 2), the largest reachable block in the
   bucket. Needs: REFHI/REFLO on a data symbol, the `$SG…` `.rdata` COMDAT for a
   string literal, an ordinary call as a value, and pointer-typed argument
   operands. **No frames beyond what the port already emits and no control flow.**
   This is §14.7 (6) promoted from last to first on a measurement.
2. **Member-call chains** — **18,449** (17,667 of them at k = 1, across five
   receiver forms), plus the `chained` bucket's own 8,001 once `type-ptr` is
   handled. Needs a frame, `this` in a callee-saved register and one `bl` per link
   — the shape §15.1's `q1` witness already showed the reference emitting and the
   port refusing. Fix `mod.rs`'s statement dispatch at the same time or the census
   will keep filing chains as `recv-load` (§16.4).
3. **Decode `op-0x9B`** — 39,361 functions with no measurement at all, and 9,041
   more visible as `call-op-0x9B` second blockers. Characterization first, exactly
   as §14.7 (5) said; D4 raises its priority because it now blocks two rows.
4. `recv-object` + `call-nested-call` (4,904 at k = 2, 2,335 at k = 3) and
   `recv-object` × `type-ptr` (2,410 at k = 1).
5. `recv-intrinsic-this-adjust` / `recv-field-off0` × `call-recv-field` (705 + 574)
   — a destructor that destroys both a base and a member.
6. **Control flow.** 23,731 functions in this bucket name a branch as their second
   blocker, `cmp-*` another 3,772, and `body-0x29` (48,102) plus `body-0x67`
   outside it. Nothing in this bucket is a *rung* for it; it is the next phase.

### 16.8 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                     # 110 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                        # 45 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                        # 42 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                           # checked=3259 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-sb.jsonl
# the probes (gitignored scratch; every function named in §16.2's witness column):
#   work/sb/probes/s1.cpp   b_if (branch) · c_ret / c_asg (the chain pair) ·
#                           a_str (plain-call) · o_add · a_ptr · r_ptr
./target/release/c2rs census work/sb/probes/s1.cpp --keep-il work/sb/il/s1
# the wild witnesses, at the workload's own flags:
./target/release/c2rs census src/system/rndobj/Mesh.cpp   --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --keep-il work/sb/il/mesh
./target/release/c2rs census src/system/hamobj/Ham.cpp    --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --keep-il work/sb/il/ham
# work/sb/tools/segs.py splits a .ex on `4F 1F` and dumps whole segments matching a
# byte pattern — the 40-byte census window is too narrow for a second blocker.
```

Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several
reflinked worktrees with different contents, and reading one through a relative
path has already produced a published wrong number in this project.

---

## 17. D5, landed — what §16's row 1 actually is, and why no lowering shipped

§16.7 ranked `data-addr × plain-call × type-ptr` **first**: 21,642 grammar-complete
bodies, *"the largest reachable block in the bucket"*, needing *"REFHI/REFLO on a
data symbol, the `$SG…` `.rdata` COMDAT for a string literal, an ordinary call as a
value, and pointer-typed argument operands. **No frames beyond what the port already
emits and no control flow.**"*

That list is right about everything it names and **wrong about what the row is**.
This rung took it, characterized it by capture, and found a construct the list does
not contain: **87.5 % of the row passes TWO data-symbol addresses to one call**, and
c2 materializes only *one* of them through a relocation pair — the other is
`addi rD, rAnchor, <difference of their .rdata pool offsets>`. Instruction selection
for the dominant shape therefore depends on a **whole-TU `.rdata` layout decision**,
which is a different and much larger piece of work than "one more relocation".

So **no lowering shipped**, deliberately, under §16's own escape clause: the
relocation and section shapes *are* fully established below and the port could emit
them; the *instruction selection* and the *argument-setup order* are not, and shipping
a lowering that cannot be graded byte-exact is the one thing this project does not do.
What shipped instead is the measurement, in the census, so the row can never be
mis-ranked from its size again.

Baseline `work/dc3-workload/scan-da-base.jsonl` — a fresh scan taken **in this
worktree** rather than a reflinked copy, precisely because §16.8's warning applies
here: 878 rows, `fn_total` 2,462,571, in class **280,020 = 11.37 %**, 1,355 keys,
mismatch 0, reproducing §16's D4 scan to the function. It also agrees to the function
with `work/dc3-workload/scan-1137.jsonl` on rows, `fn_total` and in-class. The result
is `work/dc3-workload/scan-da.jsonl` from the same list, flags and `--cwd`.

| | baseline | D5 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 280,020 (11.37 %) | 280,020 (11.37 %) | **0** |
| TUs whose per-TU in-class count moved | — | — | **0** |
| TUs whose class moved | — | — | **0** (still 6 `match`, 7 `capture-fail`) |
| mismatch | 0 | 0 | 0 |
| keys | 1,355 | 1,358 | +3 |
| functions re-keyed | — | — | **21,646 out, 21,646 in, nothing else moved by one** |

Fixture lane: `bench` 110 pass / 0 fail / 0 error; `/Ox` 45 match, `/O1` 42, `/O2` 42,
`/Ox /Gy` 42, 0 mismatch in all four; `expr_sweep` checked=3329 mismatches=0 (3,259
before, +70 for this class); `cargo test --workspace` green.

### 17.1 The split, and it is the answer

`-1sym` / `-2sym` / `-3sym+` is the number of data symbols a finished body
materializes **into a call argument register** — the number that decides whether the
row needs one relocation pair or a pool-relative selection. Over the whole workload:

| key | functions | TUs |
|---|---:|---:|
| `data-addr-2sym-then-plain-call-and-type-ptr-whole2` | **18,925** | 813 |
| `data-addr-1sym-then-plain-call-and-type-ptr-whole2` | 1,654 | 344 |
| `data-addr-1sym-then-plain-call-whole` | 1,058 | 816 |
| `data-addr-2sym-then-plain-call-whole` | 4 | |
| `data-addr-3sym+-then-plain-call-whole` | 1 | |
| `data-read-2sym-then-plain-call-and-type-real-whole2` | 4 | |
| **one symbol** | **2,712 (12.5 %)** | |
| **two symbols** | **18,933 (87.5 %)** | |

The 21,642 §16 ranked is 21,646 here — the extra 4 are the `data-read` row, which
§16.3 did not list and which splits the same way.

**What the two-symbol shape is.** An `argshape` walk over 40 workload TUs found
2,730 symbol-carrying statement-head plain calls in exactly **four** source shapes,
every one of them two symbols plus an int literal:

```text
   1,211  f(T*, "…", <line>, "…")            void     the MILO_ASSERT family
   1,159  f(int, "…", <line>, "…", int)      returns
     180  f(int, int, "…", <line>, "…")      returns
     180  f(int, T*, "…", <line>, "…")       void
```

i.e. `("expression", __LINE__, __FILE__)`. Not "a global's address passed to a
call" — an assertion macro, 813 TUs deep.

**The 12.5 % that is one symbol is real and that walk missed it**, which is worth
recording as a method note: the walk required the body to *open* on `26 <callee> BD`
and so dropped every call in an assignment right-hand side (`x = uc("hi")`), where
the statement opens on the destination push. It reported 0 single-symbol calls; the
census reports 2,712. **The census key is the measurement; the ad-hoc sample was
biased and its zero was an artifact of the anchor.** §14.2's third caution, one level
over: a sample that anchors on a byte measures the anchor.

### 17.2 What was established by capture — the parts that ARE settled

All at the fixture profile (`/Ox /GS- /c`) unless stated; probes in `work/da/probes`.

1. **The relocation quad.** A data symbol's address is `lis rS,sym@ha` +
   `addi rD,rS,sym@l`, carrying `REFHI(0x10)+PAIR` at the `lis` and
   `REFLO(0x11)+PAIR` four bytes later, both PAIR records with **symbol index 0** —
   byte-for-byte the shape `coff.rs` already emits for a pooled FP constant, which is
   the question this rung was asked to answer. The addend is **never** folded into
   the relocation: `ui(&gA[2])` is `lis ; addi r11,r11,0 ; addi r3,r11,8`, a third
   instruction. 6 witnesses.
2. **`.rdata` is NOT a COMDAT at `/Ox`.** §16.7's *"the `$SG…` `.rdata` COMDAT"* is
   wrong twice. At the fixture profile every string literal in the TU goes into **one
   ordinary `.rdata` section** (characteristics `0x40300040` = INIT_DATA | ALIGN_4 |
   MEM_READ, `Selection` 0, aux `CheckSum` = the existing `coff::coff_checksum` of its
   raw bytes — verified on two sections), entries in IL first-reference order, each
   padded to 4 bytes with **no trailing pad** and **no dedup** (`"hi"` twice gets two
   entries). The section is placed **before** `.text`, so `.text` becomes section 6
   and the fixed symbol prefix grows from 13 slots to 13 + 2 + one per literal.
3. **The literals' symbol names are generated by c2, and the rule is measured.**
   `$SG<n>` where `n` is the string literal's operand token read **little-endian**
   (`read_token_var` reads the same two bytes big-endian). Verified at n = 2535, 2536,
   2538, 2541, 2549, 2552, 2556, 2563, 2570, 3035, 5535 — probes built to walk the
   token across a 0x100 boundary, because on the narrow ones a constant offset of 1280
   fits equally well and is wrong (§14.2's fourth caution, again). They are STATIC
   symbols (`sc=3`), `Value` = the byte offset in `.rdata`, emitted immediately after
   the `.rdata` section symbol in offset order.
4. **At the workload's own profile the names come from `.gl`.** `/O1` implies `/GF`
   and `/Gy`, so string literals become `??_C@_…` **COMDAT** `.rdata` sections — and
   the mangled name is already in `.gl`, one per literal (237 names against 237 `.in`
   string records in `src/system/rndobj/Mesh.cpp`). So the `??_C@` mangling never has
   to be reimplemented; it is read like a callee's name. **The two profiles need two
   different emitters**, and only the `/Ox` one is close to what the port has.
5. **No `.rdata` at all when the TU references only named objects.** `ui(gA)` with
   `gA` extern adds exactly one undefined-external DATA symbol (`typ=0x0000`, not
   `0x0020`) to the referencing function's symbol group and four relocation records.
   That is the *smallest* possible increment to the existing packed emitter.
6. **Undefined externals are emitted in IL-stream first-reference order** within each
   function's group — the callee first because its `26` push precedes the arguments,
   then each data symbol in push order. Not relocation order (the `b`'s REL24 is the
   last relocation and its symbol is the first).
7. **A defined or static global is out of class and must stay out.** It puts a
   `.data`/`.bss` section *in the middle* of the section table (before the second
   `.XBLD$W` for a defined one, after `.text` for a static one). The `.gl` record
   separates them: at `name_nul + 5` a linkage byte reads `02` undefined-extern
   (5 witnesses), `01` defined here (2), `04` static (1). Today `gl_defined_names`'s
   unclaimed-name rule already refuses every such TU, and any rung that makes a data
   reference "accounted for" must re-impose this or it will emit a 5-section obj
   against a 6-section reference.
8. **Function alignment.** Nothing new: `.text` pads functions to 8 bytes with zeros,
   which `PortC2::build` has done since the MVP.

### 17.3 What is NOT established, and it is what stopped the rung

**(a) Two symbols in one call are not two relocation pairs.** c2 emits exactly **one**
`lis`/`addi` pair per function and derives every other symbol address from it:

```text
   void h1() { d1("aa", "bb"); }
     lis  r11,0        REFHI($SG…)     <- one pair for the whole function
     addi r3,r11,0     REFLO($SG…)
     addi r4,r3,-4                     <- the OTHER string, by pool-offset difference
     b    ?d1
```

The `-4` is the difference of the two entries' `.rdata` offsets. So selection for
this shape needs the pool laid out *before* instructions are chosen, which the port's
per-function `select_text` cannot see. Three-symbol calls chain the same way
(`addi r5,r3,-8 ; addi r4,r3,-4`).

**(b) Which symbol is the anchor is offset-dependent, not shape-dependent, and no
rule was derived.** The load-bearing witness is **six byte-identical functions in one
TU**:

```text
   void h1() { d1("a1","b1"); }   …   void h6() { d6("a6","b6"); }

   h1:  lis r11 ; addi r4,r11,0 ; addi r3,r4,+4    <- anchors the RIGHT argument
   h2:  lis r11 ; addi r3,r11,0 ; addi r4,r3,-4    <- anchors the LEFT one
   h3…h6: identical to h2
```

Same source shape, same string lengths, different instruction sequences. The only
thing that separates `h1` is that one of its literals landed at `.rdata` offset **0**.
The same split appears in the 20-function `s5` sweep and in the assert-shape probes
(`g1(p,"expr",42,"file")` anchors its *last* symbol; `g2`, `g3`, `g4` anchor their
first). *"Anchor the symbol at pool offset 0 if this call has one, else the leftmost
source-order symbol"* fits all 14 witnesses — and it is a **HYPOTHESIS with no
mechanism behind it**, exactly the kind of rule this project labels rather than
implements.

**(c) The argument setup is scheduled, and no ordering rule survived the witnesses.**
The `lis` is hoisted to the top of the function and the dependent `addi` is separated
from it, but the rest of the setup — `li` for literal arguments, `or` for formals that
must move — is interleaved in an order that is neither ascending nor descending in
argument slot:

```text
   g1(p,"e",42,"f")     lis ; li r5,42 ; addi r6 ; addi r4,r6,8            slots 2,3,1
   g2(a,"e",43,"f",7)   lis ; li r7,7 ; addi r4 ; li r5,43 ; addi r6,r4,-4 slots 4,1,2,3
   g3(a,b,"e",44,"f")   lis ; li r6,44 ; addi r5 ; addi r7,r5,-4           slots 3,2,4
   u8("hh",5,p)         lis ; or r5,r3 ; li r4,5 ; addi r3                 slots 2,1,0
   w3("s6",11,22)       lis ; li r5,22 ; addi r3 ; li r4,11                slots 2,0,1
```

Two witnesses differing only in argument count and return type produce different
orders; a "descending slot" rule fits four of the five and a "gap of exactly one
instruction between `lis` and `addi`" rule fits eleven of twelve. Neither is a rule.

**(d) With two or more formals that must shift, c2 pre-saves into scratch.**
`v4("s3",a,b)` emits `or r11,r3,r3` **before** the `lis` (which then takes r10) and
resolves the shift through the save, where the obvious descending-order sequence
needs no save at all. One move (`u4("dd",p)`, `u8("hh",5,p)`) does not trigger it.
Two witnesses, no rule.

### 17.4 The estimate, quoted before the outcome, and how it did

Quoted before any implementation and before the scan, against the 21,642 bound:
**point estimate 11,000, range 7,000–16,000, biased as an OVER-estimate** — i.e. the
realized figure was expected to come in below it. §15.4's correction (*"quote a
generated-code estimate at the `-whole` bound, because a compiler-generated body has
no freedom in the gated fields"*) was explicitly **not** applied, on the stated ground
that these are hand-written bodies. The four deductions named were: multi-statement
bodies (the matcher admits 64 statements, the port has no multi-call frame); the
defined-vs-extern gate; multi-argument register allocation mixing a scratch-materialized
address with a formal permutation; and §16.5's argument-nesting asymmetry.

**The outcome is 0, and the bias direction was right for the wrong reason.** Every
deduction listed is real, but none of them is what stopped the rung: the blocker is a
construct that was not on the list at all (§17.3 (a)), and it takes 87.5 % of the row
with it. The generalizable correction is narrower than "hand-written bodies have more
freedom": **an estimate made from a grammar measure cannot see a codegen construct
that the grammar does not distinguish.** `data-addr` is one grammar symbol whether the
call materializes one address or three, and the difference between those is the whole
rung. The `-Nsym` split exists so the next estimate is made from a key that *can* see
it.

The bucket-drop-equals-census-gain check (§13.3, §15.4) is **0 = 0** here and carries
no information; the meaningful invariant this rung has instead is that the re-key is
an exact partition: 21,646 functions left three keys and 21,646 entered six, no other
key moved by a single function, and no TU's in-class count or class changed.

### 17.5 The measure, and its one honest limit

`Fail::syms` counts a data designator only when [`eat_data_designator`] succeeds
**inside an open call-argument region** and the `26 <tok>` is not immediately followed
by `BD`. Both conditions are load-bearing and each has a witness in the unit tests:

* the **callee push** reaches the same production (that is why §16.2 named the second
  blocker `plain-call` and not `op-0xBD`), so counting every designator reports
  `uc("hi")` as two;
* an assignment statement's **destination push** is swallowed by the greedy value
  sequence before `body_matches` reaches its assignment arm, so counting outside
  argument regions reports `x = uc("hi")` as two as well.

The limit that follows: a data symbol referenced **outside** a call argument — a store
destination, a chain receiver — counts 0 and the key carries no suffix at all
(`data-addr-then-chain-bind-and-type-real-lit-whole3`, 714 functions, is that case).
That is deliberate: the count answers "how many symbol addresses must be in registers
at the call", and a number that also counted store destinations would not.

The count is **per body, not per call**. For this row they are the same number
(single-statement bodies), and for a two-call body they are not; nothing measured the
difference.

Sharding gate: 2 bits (61…62) of the existing `u64` `Block::aux`, three
values, nothing per-TU representable — `the_symbol_count_is_the_addresses_the_call_materializes`
retags both literal tokens in a witness and asserts the key does not move. The
matcher speculates, so every cursor rewind now restores the count with it
(`Fail::mark`/`Fail::rewind`, one per `*p = save`); without that, a body needing two
grants counts its designators twice.

### 17.6 The order of work, re-ranked

1. **Member-call chains — 18,449** (17,667 at k = 1, across five receiver forms), plus
   the `chained` bucket's own 8,001 once `type-ptr` is handled. Unchanged from §16.7
   (2), and now the largest reachable block in the bucket by default. Needs a frame,
   `this` in a callee-saved register and one `bl` per link. Fix `mod.rs`'s statement
   dispatch at the same time or the census keeps filing chains as `recv-load` (§16.4).
2. **Decode `op-0x9B`** — 39,361 functions with no measurement at all, plus 9,041 more
   visible as `call-op-0x9B` second blockers. Still the largest unnamed thing.
3. **`data-addr-1sym` — 2,712**, and *only* this sub-row. It needs §17.2's relocation
   quad, the `/Ox` `.rdata` string pool, the extern-linkage gate, and nothing from
   §17.3 — provided the call's remaining arguments need **no setup instruction**
   (`uc("hi")`, `u3(p,"cc")`, `return u7("gg")`: `lis ; addi rD,r11,0 ; b`, adjacent
   and unambiguous). How much of the 2,712 that is has **not been measured** — a
   `-Nsym`-style split on "setup instructions besides the address" is the measurement,
   and it is cheap now that the walk is there. A rung that assumes all 2,712 will hit
   §17.3 (c) on the first `f("str", p)`.
4. `recv-object` + `call-nested-call` (4,904 at k = 2) and `recv-object` × `type-ptr`
   (2,410 at k = 1).
5. `recv-intrinsic-this-adjust` / `recv-field-off0` × `call-recv-field` (705 + 574).
6. **`data-addr-2sym` — 18,933 — is a PHASE, not a rung.** It needs a TU-wide `.rdata`
   pool layout visible to instruction selection, the anchor rule of §17.3 (b), and the
   argument scheduler of §17.3 (c). It is still the single largest grammar-complete
   block in the bucket and it should stay on the board — but ranked by size it will
   keep coming out on top and it is not takeable until the scheduler is a modeled
   thing, which is the same conclusion §16.7 (6) reached about control flow.

### 17.7 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                     # 110 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                        # 45 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                        # 42 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                           # checked=3329 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-da.jsonl
# the probes (gitignored scratch; every witness in §17.2/§17.3 comes from one):
#   work/da/probes/s1.cpp  six single-symbol pushes: literal, array, &gA[k], &gS.b
#   work/da/probes/s2.cpp  one symbol beside formals and literals, at every slot
#   work/da/probes/s3.cpp  formal permutations, two extern objects, no literal
#   work/da/probes/s4.cpp  the four assert shapes, verbatim from the workload
#   work/da/probes/s5.cpp  20-function sweep: slots x filler x symbol count
#   work/da/probes/s6.cpp  SIX BYTE-IDENTICAL two-string calls — the anchor witness
./target/release/c2rs compile work/da/probes/s6.cpp --keep-obj work/da/s6.obj
python3 work/da/tools/coff.py  work/da/s6.obj     # sections, symbols, aux, relocations
python3 work/expr/tools/objdis.py work/da/s6.obj
# the argument-shape walk over real TUs (biased — see §17.1 — but it is what found
# the two-symbol shape):
python3 work/da/tools/argshape.py 'work/da/il/smp*/*.ex'
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

## 18. D6, landed — the frame audit: which rows are still local, and which are not

§17.6 left three rungs ranked by size and a hypothesis nobody had tested: that
**the port is out of leaf-shaped work, and essentially everything remaining needs a
frame or a non-local decision**. The evidence for it was circumstantial and is
quoted here so the answer can be graded against it — chains put a call's result in
the next call's receiver; `op-0x9B` is a by-value temporary receiver that §14 guessed
needs "frame + 2 `bl`"; `recv-object`'s second-largest second blocker is
`branch-0x39`; `this-adjust` (141,800) and `base-member-addr` (122,949) are described
in §13 as mostly non-leaf.

This rung answers it by measurement. **MEASURED: the hypothesis is right in the
aggregate and wrong about two of its five named rows**, and the two it is wrong about
are the two largest things on the board.

Baseline `work/dc3-workload/scan-fa-base.jsonl` — taken **in this worktree** rather
than read through a relative path, per §16.8: 878 rows, `fn_total` 2,462,571, in class
**280,020 = 11.37 %**, 1,358 keys, mismatch 0, reproducing §17's D5 scan to the
function. The result is `work/dc3-workload/scan-fa.jsonl` from the same list, flags
and `--cwd`.

| | baseline | D6 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 280,020 (11.37 %) | 280,020 (11.37 %) | **0** |
| TUs whose per-TU in-class count moved | — | — | **0** |
| TUs whose class moved | — | — | **0** (still 6 `match`, 7 `capture-fail`) |
| mismatch | 0 | 0 | 0 |
| keys | 1,358 | 1,363 | +5 |
| functions re-keyed | — | — | **37,662 out, 37,662 in** (§18.3) |

Fixture lane: `bench` 110 pass / 0 fail / 0 error; `/Ox` 45 match, `/O1` 42, `/O2` 42,
`/Ox /Gy` 42, 0 mismatch in all four; `expr_sweep` checked=3329 mismatches=0;
`cargo test --workspace` green.

### 18.1 The instrument, and why it is not a grammar measure

`GAPS.md` §6's newest entry is that **a grammar measure cannot see a codegen construct
the grammar does not distinguish** — the finding that cost D5 its rung. The obvious
reaction is to widen the grammar; the useful one is to add a measure that does not
depend on it at all.

[`body::call_tokens`] counts the CALL tokens in a segment. It is not a parse: it walks
the raw bytes and stops nowhere, so it reports the same number for a body the census
can decode and one it cannot. From that count comes the **frame class**, and its
middle value is honest rather than convenient:

| class | what it settles | share of the workload |
|---|---|---:|
| `calls-0` | no call, so LR is never clobbered — **cannot** need a frame | 869,614 (35.3 %) |
| `calls-1` | a tail call stays a leaf; a call whose result is computed on does not. **The count cannot tell them apart and this class does not pretend to.** | 790,302 (32.1 %) |
| `calls-2plus` | the first `bl` clobbers LR and the return address is still live — **always** a frame | 802,655 (32.6 %) |

**The measure is graded in two independent ways, and the second one is the reason to
believe it.**

1. **Against the reference obj.** On every TU where `.gl` binds one name per segment,
   segment *k* pairs 1:1 with emitted function *k*, so the IL count can be compared
   with the obj's own `bl`/`b`-with-REL24 count. Over the 110 fixtures plus this
   rung's probes: **696 of 705 functions agree, 98.7 %**. Both failure directions are
   named and both are one-sided — an `0x40` intrinsic that lowers to a real branch is
   not a `BD` (`memcpy`, `memset`, `dynamic_cast`, an aggregate copy, 6 witnesses),
   and c2 sometimes inlines or folds a call the IL still spells (3 witnesses).
2. **Against the in-class functions, which are a standing control group of 280,020.**
   A shape the whole-body parser accepted as a leaf cannot contain two calls, so any
   in-class function reading `calls-2plus` is a false positive by construction. The
   census reports the control group every scan, and it currently reads:

   ```
   indirect-load-leaf  157,912   c0=157,912  c1=0       c2+=0
   empty-body           73,339   c0= 73,339  c1=0       c2+=0
   empty-dtor-delegation 27,501  c0=0        c1=27,501  c2+=0
   straight-line        10,960   c0= 10,960  c1=0       c2+=0
   empty-dtor-member     6,234   c0=0        c1= 6,234  c2+=0
   empty-dtor-member-adjusted 2,229 c0=0     c1= 2,229  c2+=0
   float-leaf            1,005   c0=  1,005  c1=0       c2+=0
   void-tail-call          840   c0=0        c1=   840  c2+=0
   ```

   Every one of the 280,020 lands exactly where its parsed shape says it must, and
   **none reads `calls-2plus`**.

That second grade is what forced the measure to be right. The first version counted a
`BD` whenever the following bytes were merely TYPE-shaped, and it read **10,088
in-class leaves as two-call bodies** — a 3.6 % false-positive rate that would have put
tens of thousands of spurious functions into the "needs a frame" column. The fix is
`GAPS.md` §6's own rule applied literally: **a field that never varied is
indistinguishable from a constant, so require it and fail closed.** The CALL token now
has to produce all of

* **calling convention `00`** — 15,095 of 16,100 candidate sites in
  `src/lazer/meta_ham/HamUI.cpp`, the rest spread over 200-odd distinct bytes, which
  is the signature of a payload byte rather than a field;
* **the `80` escape form** of the fn-type id — 15,090 of those 15,095;
* **id ≥ 0x1000**, the floor of the per-TU function-type id space. Measured range
  0x1001…0x1081 across the fixtures and 0x1001…0xFA89 in the wild TU; exactly one
  candidate site fell below it and it was a false positive.

A bare `67` virtual dispatch is **not** counted: a virtual call carries its own `BD`,
so counting the `67` as well double-counted it. Removing it and requiring the three
fields took the obj grade from 98.0 % to 98.7 % and the control group from 10,088 to 0.

The count is carried in the scan as a **second axis** (`fn_frames`, keyed
`"<class>|<census key>"`) rather than as a suffix on the ranked keys, because four
sessions of recorded tables name those keys and renaming all of them to carry an
orthogonal fact would break every recorded comparison for nothing.

### 18.2 The per-row verdict

Every row below is **MEASURED** on the frame axis; the `witness` column is a
controlled capture at the fixture profile whose emitted instructions were read, not
inferred. Probes are in `work/fa/probes` (gitignored).

| row | functions | `calls-0` | `calls-1` | `calls-2plus` | verdict | witness |
|---|---:|---:|---:|---:|---|---|
| `op-0x9B` (`expr-call-in-expr`) | 39,361 | 0 | 0 | **39,361 (100 %)** | **frame, no exceptions** | `p2.cpp` |
| `chained` (after §18.3) | 45,663 | 0 | 0 | **45,663 (100 %)** | **frame, no exceptions** | `p3.cpp` |
| `recv-object` | 64,904 | 0 | 2,568 | **62,336 (96.0 %)** | **frame** | `p3.cpp` |
| `recv-load` | 16,430 | 0 | 43 | 16,387 (99.7 %) | frame | — |
| `recv-intrinsic` | 19,447 | 0 | 2,784 | 16,663 (85.7 %) | frame | `p7.cpp` |
| `recv-field` / `-off0` | 20,724 | 0 | 2,019 | 18,705 (90.3 %) | frame | — |
| `data-addr` | 56,634 | 3 | 49,184 | **7,447 (13.1 %)** | **not a frame problem** | §17 |
| `data-read` · `nested-call` · `recv-deref` · `recv-call` | 4,864 | 2 | 5 | 4,857 | frame | — |
| `expr-intrinsic-this-adjust` (2113) | 141,800 | 15 | 57,894 | **83,891 (59.2 %)** | **split** | `p7.cpp` |
| `expr-intrinsic-base-member-addr` (2117) | 122,949 | **32,372** | 56,186 | 34,391 (28.0 %) | **mostly local** | `p7.cpp` |
| `body-0x9B` (statement head) | 27,073 | 2,035 | 7,624 | 17,414 (64.3 %) | split | `p1.cpp` |
| `expr-op-0x9B` | 9,437 | 0 | 808 | 8,629 (91.4 %) | frame | `p1.cpp` |
| `body-0x29` (control flow) | 48,102 | 1,622 | 4,674 | 41,806 (86.9 %) | frame **and** a branch | — |
| `expr-convert` | 26,796 | 8,246 | 1,736 | 16,814 (62.7 %) | split | — |

**§13's "mostly non-leaf" reading of the two class-layout intrinsics is wrong for
2117 and only 59 % right for 2113.** 32,372 of `base-member-addr`'s 122,949 issue no
call at all, and the capture says what they emit: `int D2::mem1() const { return b; }`
is `lwz r3,4(r3) ; blr`, `return b + d;` is `lwz ; lwz ; add ; blr`, and
`void D2::mem3(int v) { b = v; }` is `stw r4,4(r3) ; blr`. Two, four and two
instructions, no frame, **no `.pdata` entry at all**. That is the largest genuinely
leaf-shaped block left in the census.

### 18.3 The chain classification, fixed — and the corrected population

§16.4 measured that `chained` undercounts chains ~4.4× and named the cause exactly:
`mod.rs`'s body dispatch cannot tell a statement-head `26 <tok>` assignment
*destination* from a stacked *method* push, so a chain in a **value** position has its
outer method push eaten and files as whatever its inner receiver is.
`return p->Next()->Get();` and `x = p->Next()->Get();` are the same bytes plus one push
and landed in different buckets.

`mcall::reanchor_chain` fixes it on the error path of the assignment parser's
right-hand side, under three conditions, all required:

1. the refusal is this module's and sits **exactly where the destination push ended**;
2. walking from the statement head classifies as `Chained` where walking from the
   probe did not;
3. **the statement contains one depth-0 `99` bind per stacked method.**

Condition 3 is the load-bearing one and it is what a naive fix would have missed.
`x = p->Get()` has the *same* two-symbol head run as a two-link chain in a value
position — `26 <x> 26 <Get>` against `26 <Get> 26 <Next>` — and differs only in that
the statement contains **one** bind rather than two. Without the bind count the fix
would have traded a 4.4× undercount for an overcount of every single-link assignment
in the corpus. `PROBE_ONE_LINK_ASSIGN` is that control, in the unit tests, beside the
pair.

The count re-enters `walk_detail` rather than running a second tokenizer, so the count
and the classification cannot drift apart — `GAPS.md` §6's "one fact, one locator",
which matters more than usual here because the re-anchor's whole job is to disagree
with that walk about one leading token.

**The movement is an exact partition, and every function is accounted for:**

| form | baseline | D6 | delta |
|---|---:|---:|---:|
| `chained` | 8,001 | **45,663** | **+37,662** |
| `recv-load` | 51,086 | 16,430 | −34,656 |
| `recv-intrinsic` | 21,294 | 19,447 | −1,847 |
| `recv-deref` | 1,072 | 250 | −822 |
| `recv-field` | 14,297 | 13,963 | −334 |
| `recv-call` | 7 | 5 | −2 |
| `recv-object` | 64,905 | 64,904 | −1 |
| `data-addr` · `data-read` · `recv-field-off0` · `nested-call` · `op-0x*` · `other` | — | — | **0** |
| `expr-call-in-expr` total | 268,140 | 268,140 | **0** |
| non-`mcall` census keys | 1,914,411 | 1,914,411 | **0** |

Every form that lost functions is one that can be the *innermost* link of a chain;
every form that cannot be one is unchanged to the function. Acceptance is untouched
(the `Err` stays an `Err`), no TU's in-class count moved and no TU's class moved.

**The corrected population is 45,663 — 17.0 % of the bucket, a 5.71× undercount, not
4.4×.** §16.4's 43,521 was an estimate built by adding the `chained` bucket to the
`chain-bind` second-blocker count, and it was low for a structural reason worth
recording: a body whose *second* blocker is a chain link is a body the chain
production had not been tried on, so the two populations were neither disjoint nor
exhaustive. After the fix 6,563 rows still name `chain-bind` as a second blocker —
those are third links and chains nested inside other forms — so up to **52,226**
bodies contain a chain, but the 45,663 is the one that is a form and the one to rank.

### 18.4 What `op-0x9B` actually is — MEASURED by capture

§16.6 called it *"the largest thing on the worklist whose size is not yet known"* and
deliberately did not decode it. It does not need decoding to be sized: the frame axis
runs outside the grammar, so an undecoded form is measured like any other.

**It is a by-value returned aggregate temporary, and it is 100 % frame.** The source
that produces it is a **plain** (non-member) call returning a struct, whose result
receives a member call, in an assignment:

```cpp
struct V { int a; int Val() const; };
extern V gFV();
int a1() { int x; x = gFV().Val(); return x; }        // -> expr-call-in-expr-op-0x9B
```

`work/fa/probes/p1.cpp` establishes what it is *not*: with the producer a **member**
call (`gO.GetV().Val()`) the walk sees two stacked methods and files `chained`, and
with the temporary bound to a named local (`V v = gO.GetV(); return v.Val();`) the
statement opens on the `9B` itself and files `expr-op-0x9B`. All three are the same
construct at different statement positions, which is §9.2 for the fourth time.

The reference, fixture profile, and it is the general frame in miniature:

```text
?a1@@YAHXZ:
    mfspr r12,r8,r0        <- mflr
    stw   r12,-8(r1)
    stwu  r1,-96(r1)       <- a 96-byte frame for a 4-byte temporary
    bl    ?gFV@@YA?AUV@@XZ
    stw   r3,80(r1)        <- the returned aggregate, spilled to a frame slot
    addi  r3,r1,80         <- its address becomes the receiver
    bl    ?Val@V@@QBAHXZ
    addi  r1,r1,96
    lwz   r12,-8(r1)
    mtspr r12,r8,r0
    bclr
```

with a `.pdata` entry `00000000 40000B03` — function length 11 words, prolog length
3. Two temporaries in one body (`x = gFV().Val(); y = gFV().Val();`) grows the frame
to 112 and the prolog to 4, adds `std r31,-16(r1)`, and uses r31 to carry the first
result across the second call.

**Is classifying it safe?** §16.6's reservation was that making `9B` a receiver *form*
requires the walk to treat a depth-0 `32` store as non-decisive mid-statement, which
would move functions between D2 sub-buckets on a hypothesis. That reservation still
stands and this rung did not touch it — **and it no longer matters for ranking.** The
frame axis already prices the row without moving a single function: 39,361 at 100 %
`calls-2plus`, plus `body-0x9B`'s 27,073 at 64.3 % and `expr-op-0x9B`'s 9,437 at
91.4 %. Decoding `9B` is now a *decode* task with a known payoff, not an unknown.

### 18.5 The answer to the hypothesis, and the share that needs a frame

**Over the 2,182,551 blocked functions:**

| | functions | share of blocked |
|---|---:|---:|
| `calls-0` — provably no frame | 626,398 | 28.7 % |
| `calls-1` — a tail call or a small frame | 753,498 | 34.5 % |
| `calls-2plus` — **provably a frame** | **802,655** | **36.8 %** |

36.8 % is a **lower bound and it is exact**. To price the middle class, the same
question was asked of the code c2 actually emits: over **178,969 emitted functions**
across 871 workload objs, read straight off `.text` with no IL and no grammar
(`work/fa/tools/frames.py`, a function is framed iff it saves LR or moves r1):

| | emitted functions | leaf | framed |
|---|---:|---:|---:|
| `calls-0` | 81,217 | 80,624 | 593 (0.7 %) |
| `calls-1` | 37,682 | 21,572 | **16,110 (42.8 %)** |
| `calls-2plus` | 60,070 | 1,028 | 59,042 (98.3 %) |
| **all** | **178,969** | 103,224 | **75,745 (42.3 %)** |

The `calls-2plus` row is 98.3 % rather than 100 % because the obj-side counter counts
a `bctr` jump table as a call; the rule itself has no exception. Applying the 42.8 %
to the blocked middle class gives a point estimate of **≈ 1,129,500 blocked functions
needing a frame, 51.8 %** — LABELLED as an estimate, because the split is measured on
emitted code and applied to a population that is mostly *not* emitted (see §18.6).

**So: the hypothesis holds in the aggregate and fails on its two largest named rows.**
Every construct it named is confirmed — `op-0x9B` 100 %, chains 100 %, `recv-object`
96 %, `recv-load` 99.7 % — and half of everything left needs a frame. But
`this-adjust` is 59 % rather than "mostly", and `base-member-addr` is **28 %**, with
32,372 functions that issue no call at all. §13's reading of those two was inferred,
and this is the measurement that corrects it.

### 18.6 What is NOT established, labelled

* **The `calls-1` framed share is measured on emitted code and applied to IL bodies,
  and those are different populations.** `src/lazer/meta_ham/HamUI.cpp` has **9,551
  `4C 4F 11` function bodies in its `.ex` and emits 350 functions**; across the
  workload it is 2,462,571 IL bodies against 178,969 emitted, **7.3 %**. Every number
  in §18.2 is over IL bodies (the census denominator) and every number in the emitted
  table is over emitted functions, and the 42.8 % is the one place they are mixed.
  Nothing measured how the frame split of an *unemitted* inline body compares.
* **The emitted/IL gap is itself unexplained and unmeasured**, and it is not this
  rung's finding to close. What is established is that the port **fails closed** on
  it: `int f(int a){return a+1;} struct S{int m; int Unused() const {return m;}};`
  censuses **2/2 in class**, the reference emits **one** function, and the port
  returns `NotImplemented` — `bundle.rs`'s `bound.len() != segs.len()` gate refuses
  because `.gl` binds one name and the splitter found two segments. No wrong-bytes
  emit, but the gate is doing this work incidentally rather than by design.
* **`calls-1` is not decomposed per row.** Every row's middle column is priced with
  one corpus-wide constant. A row whose single call is always in tail position
  (`this-adjust`'s `one()`) and one whose single call never is (`store()`) get the
  same 42.8 %, and both shapes are in the capture. Splitting `calls-1` by tail
  position is the next cheap measurement and it was not taken.
* **`call_tokens` undercounts intrinsic calls by construction.** `memcpy`, `memset`,
  `dynamic_cast` and an aggregate copy lower to real branches and carry no `BD`, so
  `expr-intrinsic-memset`'s 42.5 % and the two `base-*cast` rows are floors, not
  values. 6 witnesses, all in the fixture grade.
* **The re-anchor is scoped to chains and to one call site.** `parse_expr` raises
  `CALL_IN_EXPR` from other places and only the assignment parser's right-hand side
  re-anchors. §5 records the *same* dispatch splitting `expr-intrinsic-this-adjust`
  from `expr-call-in-expr-recv-intrinsic-this-adjust` by one leading literal, and this
  rung did **not** fix that one: the discriminator there is not a bind count and
  deriving it would move 141,800 functions on an untested rule.
* **`calls-2plus` says a frame is needed, not that a frame is sufficient.** 23,632
  `recv-object` functions are `calls-2plus` *and* name `branch-0x39` as their second
  blocker; they need basic blocks as well. The frame axis is one dimension.

### 18.7 The order of work, re-ranked — and it is the general frame

§17.6 ranked chains first at 18,449, `op-0x9B` second, and `data-addr-1sym` third.
The frame axis says those are not three rungs; they are **one rung's first three
customers**.

1. **The general frame, plus per-COMDAT `.pdata`.** Not because it is the largest row
   — it is not a row — but because **every one of the top rows is 96–100 % framed and
   none of them can be taken without it**: `chained` 45,663 (100 %), `op-0x9B` 39,361
   (100 %), `recv-object` 62,336 framed, `recv-load` 16,387, `recv-field` 18,705,
   `recv-intrinsic` 16,663. That is **199,000 framed functions in the `expr-call-in-expr`
   bucket alone**, and 802,655 across the census. What it needs, all of it measured in
   this rung's captures: a **variable frame size** (96 for one temporary, 112 for two);
   **LR save/restore** (`mflr r12 ; stw r12,-8(r1)` … `lwz ; mtlr ; blr`); **callee-saved
   GPRs** (`std r31,-16(r1)`, `std r30,-24(r1)`, allocated in descending order and
   restored after the `mtlr`); a **frame-slot allocator** for by-value temporaries
   (`stw r3,80(r1) ; addi r3,r1,80`); and a **`.pdata` entry per framed function**
   carrying `0x40000000 | (function_words << 8) | prolog_words` with prolog lengths of
   **3, 4 and 5** in this rung's captures alone — where `coff::build_pdata` hardcodes
   3 — and **no entry at all** for a leaf. Per-COMDAT because the workload's `/O1`
   implies `/Gy`, and `PortC2::build` refuses a framed call there today by name.
2. **`base-member-addr`'s 32,372 `calls-0` functions** — the largest genuinely
   leaf-shaped block left, `lwz`/`stw` at a folded displacement, no frame and no
   `.pdata`. This is the one item the frame hypothesis got wrong and the one rung that
   can still be taken *before* the frame. `expr-intrinsic-base-member-addr` already has
   a decoder (`try_parse_base_member_load`); what refuses these is the surrounding
   expression, not the intrinsic.
3. **`data-addr-1sym` — 2,712, and 100 % `calls-1`.** §17.6 (3) survives the audit
   intact: the row needs no frame, and §17.3's two-symbol pool problem does not touch
   it. Still gated on §17.6's unmeasured question — how many have a call argument
   needing no setup instruction.
4. **`recv-object × type-ptr` — 2,410, and 100 % `calls-1`.** A named-object receiver
   with a pointer argument. Measured leaf: `gC.Val()` is
   `lis r11,gC@ha ; addi r3,r11,gC@l ; b ?Val`, and three byte-identical bodies in one
   TU emit three identical sequences — the §17 tell, run and passed. It needs §17.2's
   relocation quad and nothing else.
5. **Control flow.** Unchanged from §16.7 (6), and the frame axis adds that it is
   entangled: `body-0x29` is 48,102 functions at **86.9 % `calls-2plus`**, so the
   branch rung and the frame rung will meet.

Items 2, 3 and 4 total **37,494 functions** and are the entire remaining local
inventory above a thousand functions. Item 1 is 802,655. **The hypothesis's practical
conclusion is correct even though two of its rows are not: the next rung is the
general frame.**

### 18.8 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 110 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 45 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 42 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=3329 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16 \
  --jsonl work/dc3-workload/scan-fa.jsonl        # prints the frame-class block
# the probes (gitignored scratch; every witness in §18.2/§18.4 comes from one):
#   work/fa/probes/p1.cpp  op-0x9B at three statement positions
#   work/fa/probes/p2.cpp  the by-value temporary, four byte-identical bodies
#   work/fa/probes/p3.cpp  chains, named-object receivers, both class-layout intrinsics
#   work/fa/probes/p5.cpp  an in-class IL body c2 never emits (the §18.6 gate)
#   work/fa/probes/p6.cpp  the re-anchor pair plus four controls that must not move
#   work/fa/probes/p7.cpp  2113/2117 at BOTH ends of their frame split
./target/release/c2rs compile work/fa/probes/p7.cpp --keep-obj work/fa/p7.obj
python3 work/fa/tools/coff.py    work/fa/p7.obj    # sections, symbols, aux, relocations
python3 work/expr/tools/objdis.py work/fa/p7.obj
# the emitted-code frame split (§18.5): capture every workload obj, then read .text
sh      work/fa/tools/capture_all.sh                # -> work/fa/objs/*.obj, ~35 s at -P16
python3 work/fa/tools/frames.py work/fa/objs/*.obj
# the IL-vs-obj grade of `call_tokens` (§18.1): pairs 1:1 only where .gl binds
python3 work/fa/tools/ilcalls.py grade <bundle>.ex <obj>
```

Always difference the scans through **absolute** paths and print each one's row count
and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several reflinked
worktrees with different contents, and reading one through a relative path has already
produced a published wrong number in this project.

## 19. D7, landed — the address leaf: §18.7's item 2, and the half of it that had no census key

§18.7 ranked **`base-member-addr`'s 32,372 `calls-0` functions** second, behind the
general frame, as *"the largest genuinely leaf-shaped block left … the one item the
frame hypothesis got wrong and the one rung that can still be taken before the
frame."* It also guessed what they were: `lwz r3,4(r3) ; blr`, `lwz ; lwz ; add ; blr`,
`stw r4,4(r3) ; blr`.

This rung characterized them first and then took the largest **whole-body-complete**
one. **MEASURED: the guess is right about the constructs and wrong about their
weights** — the store is 1.2 % of the block, not a third of it — and the shape that
was actually takeable is one §18 did not name at all: the **address leaf**,
`return &s->m;`. Taking it through the 2117 designator alone would have been a 6,933
rung. Grepping for every site implementing the same rule (`GAPS.md` §6, and §17.4's
own correction) found the *plain* designator refusing the identical construct at a
second site, five times bigger and with no census key of its own.

Baseline `work/dc3-workload/scan-bma-base.jsonl` — taken **in this worktree**, per
§16.8: 878 rows, `fn_total` 2,462,571, in class **280,020 = 11.37 %**, 1,363 keys,
mismatch 0, 6 `match` / 7 `capture-fail`, reproducing §18's D6 scan to the function.
The result is `work/dc3-workload/scan-bma.jsonl` from the same list, flags and `--cwd`.

| | baseline | D7 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 280,020 (11.37 %) | **320,641 (13.02 %)** | **+40,621** |
| TUs whose per-TU in-class count moved | — | 826 up | **0 down** |
| TUs whose class moved | — | — | **0** (still 6 `match`, 7 `capture-fail`) |
| mismatch | 0 | 0 | 0 |
| keys | 1,363 | 1,363 | **0 new, 0 gained** |
| sum of every blocker key's delta | — | — | **−40,621** |

**The bucket drop equals the census gain exactly**, to the function: 257 keys moved,
every one of them **down**, and their total is the in-class gain. §13.3's invariant
holds for the fourth time and for the same reason — the production accepts an entire
segment or nothing.

Fixture lane: `bench` **112 pass / 0 fail / 0 error** (110 + the two new fixtures);
`/Ox` **46** match (was 45), `/O1` **43** (42), `/O2` **43** (42), `/Ox /Gy` **43**
(42), 0 mismatch in all four; `expr_sweep` **checked=3516 mismatches=0** (3,329
before, +187 for this class); `cargo test --workspace` green.

### 19.1 The sub-shape census, before any implementation

The instrument is `work/bma/tools/bma{,2,3,4}.py` (gitignored scratch): a partial
re-implementation of `body/` in Python that reproduces the
`expr-intrinsic-base-member-addr` census key and then classifies **what follows the
designator's `4C` apply**, as a *whole body* reaching the return plumbing and the
segment end.

**It is graded before it is used.** Over 19 stride-sampled workload TUs (64,398
segments) it splits the 2117 key's frame classes **26.4 % / 44.9 % / 28.7 %**, against
the census's own **26.3 % / 45.7 % / 28.0 %** over all 2,462,571 — so it is
reproducing the key, not a neighbouring one.

Of the **863** sampled bodies whose first blocker is 2117 and whose `call_tokens` is 0:

| sub-shape | n | share | what it is |
|---|---:|---:|---|
| two designators + a binary op | 455 | **52.7 %** | `a == b`, `(a-b)/20`, `(a-b)>>2` over two inherited members. §18's `lwz ; lwz ; add`, and it is the biggest — but it is **not** whole-body complete under any load production: it needs the compare / divide / shift lowerings, and the divisions are magic-multiply. |
| **address return** | **184** | **21.3 %** | `return &b1;` — one `addi`. **Taken.** |
| store + further statements | 116 | 13.4 % | a `stw` and then more body |
| designator refused | 60 | 7.0 % | the designator's own pointer TYPE is not `is_ptr_to_4` — a `short`/`char` member. Now accepted by the address path (§19.2). |
| load with extra offset adds | 33 | 3.8 % | `p->t[2]` on an inherited array: `lwz r3,16(r3)`. Whole-body complete, **not taken** (§19.6). |
| **whole-body store** | **10** | **1.2 %** | `void D::sb1(int v){ b1 = v; }` → `stw r4,12(r3) ; blr` |
| store, partial | 5 | 0.6 % | |

**§18.7's third example was the smallest thing in the block.** `stw r4,4(r3) ; blr` is
a real construct and it is 1.2 % of the `calls-0` 2117 functions (≈370 across the
workload); the stores in that bucket are overwhelmingly one statement of several. The
generalizable note is narrow and worth keeping: **a captured example proves a
construct exists and says nothing about its weight**, and §18's three examples came
from one hand-written probe (`p7.cpp`) rather than from a frequency count.

### 19.2 What the address leaf is — MEASURED by capture

`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`, `p4.cpp`; every word below read off the
reference obj at the fixture profile (`/Ox /GS- /c`) with
`work/bma/tools/objdis.py`.

```text
   int*  f(S* s)         { return &s->b; }     38630004  addi r3,r3,4      ; blr
   int*  f(int x, S* s)  { return &s->b; }     38640004  addi r3,r4,4      ; blr
   int*  D::pb1()        { return &b1; }       3863000c  addi r3,r3,12     ; blr   (2117, 8+4)
   int*  DR::pt2()       { return &t[2]; }     38630010  addi r3,r3,16     ; blr   (two `28`s)
   int*  f(S* s)         { return s->arr; }    38630028  addi r3,r3,40     ; blr   (the decay)
   int*  f(S* s)         { return &s->a; }               blr                       (K = 0)
```

and **no `.pdata` entry**: it is a leaf. The production is

```text
   <designator>                       B9 <tok> <PTR4>   |   the intrinsic-2117 form
   ( 33 <int-like> k  27 <PTR>        byte-offset adds, ANY number, summed
   | 33 <int-like> k  28 00 00 )*
   [ 2C <PTR> 00 ]                    array-to-pointer decay / cv strip, free
   41 <PTR>                           result type: a POINTER
   <return plumbing, to the segment end>
```

Four things it establishes that the existing leaf shapes did not:

1. **The member's own width never reaches the instruction.** `char`, `short`, `int`,
   `long long`, `float` and `double` inherited members all emit the identical `addi`
   (`p2.cpp`, `DW::ac`…`DW::ad`), where the *load* leaf one token away picks
   `lbz`/`lhz`/`lwz`/`ld` from exactly that field. So the address path needs a third
   pointer predicate, [`is_ptr_any`], beside `is_ptr_to_4` (pointee width 4, gates a
   `lwz`) and `is_ptr4_kind` (four exact tags, gates a pointer value in a register).
   Two predicates for two facts became three for three.
2. **The TYPE tag's width nibble is not a dependable statement of the pointee width
   in this position**, which is the second and better reason not to gate on it:
   `char*` carries `86 43` and `short*` carries `84 43`, while `long long*`, `float*`
   and `double*` all carry `86 43`. Raw witnesses in `p2.cpp`'s
   `41 86 43 f0 08` (char) against `41 84 43 91 08` (short).
3. **The offset-add run is unbounded here**, where the load leaf admits at most one.
   Every add folds into the same displacement, so `LIT(0) 28 · LIT(8) 28` costs one
   `addi`. A cap of one would refuse `&p->t[2]` on an inherited array.
4. **`K == 0` emits nothing, and only from the first argument register.** From r4 or
   r5 c2 emits a real `mr r3,r4` (`p3.cpp`, `z_r4`/`z_r5`/`i_z_r4`) — the same
   boundary `straight_line_is_out_of_class` draws for the bare-parameter identity.
   Refused here rather than assumed, by the **parser**, with `addr_leaf_text`'s own
   refusal as a second lock.

The `§17.3(b)` locality tell was run and passed: `work/bma/probes/p4.cpp` has **six
byte-identical `&s->b` functions, four byte-identical `&p->b1`, and three
byte-identical zero-offset ones** in one TU, and they emit six identical `addi
r3,r3,4`, four identical `addi r3,r3,12`, and three identical bare `bclr`. The
decision is local.

### 19.3 The two sites, and why the second one is the rung

`try_parse_ptr_identity_leaf`'s own header already recorded the plain form and
already knew its size — *"it occurs: 7 of the 40 pointer-shaped bodies in the three
TUs scanned are this, and admitting them as identities would emit a bare `blr` where
c2 emits `addi r3,r3,12`"* — and `fixtures/cpp/w12_ptr_leaf_neg.cpp` carried it as
`n_addr_of`, a **negative**. §18.7 ranked the 2117 half and did not connect the two,
because the plain half has **no census key of its own**: it files under
`expr-load-type-A643<id>`, whose third byte is a per-TU type id, so the population is
smeared over **256 sharded keys** and cannot be seen in a ranked histogram at all.

The measured split, in the 19-TU sample: **928 plain against 184 intrinsic, 5.0×**.
The realized workload split is the same shape:

| designator | key(s) | drop |
|---|---|---:|
| plain (`B9` + `27`/`28` adds) | `expr-load-type-*`, 256 sharded keys | **−33,688** |
| intrinsic 2117 | `expr-intrinsic-base-member-addr` | **−6,933** |
| | | **−40,621** |

**§17.4's correction — "grep for every site implementing a rule you change" — is what
paid here, and the amount is 4.9×.** It is also a second, sharper instance of
`GAPS.md` §6's "a grammar measure is blind to what the grammar does not distinguish":
this time the grammar *did* distinguish it, and the **key sharding** hid it. A key
that carries a per-TU id is not merely noisy — it is invisible to the ranking that
drives the roadmap.

### 19.4 The estimate, quoted before the outcome, and how it did

Quoted before implementation, from the 19-TU sample and before any scan:

> point estimate **+33,000**, range **24,000–40,000**, biased as an **OVER**-estimate
> — i.e. the realized figure was expected to come in below the point estimate.
> Deductions named, all one-sided downward: the `parse_params`/`this` binding, `.sy`'s
> one-register-each gate, the zero-offset non-first-parameter refusal, and the exact
> return plumbing. §15.4's "quote a generated-code estimate at the bound" was
> explicitly **not** applied, per its own scope — these are hand-written accessors.
> Predicted split: ≈6,500 from the 2117 key, ≈26,500 from `expr-load-type-*`.

**The outcome is +40,621 — above the point estimate and above the top of the range,
and the stated bias direction was WRONG.** Per key: 2117 predicted 6,500, realized
**6,933** (+6.7 %); plain predicted 26,500, realized **33,688** (+27 %).

The cause is specific and it is not "the deductions were too big" — every deduction
named is real. The raw sample scale, uncorrected, gave **35,489** for the plain half
and **7,036** for the intrinsic half, i.e. **42,525 total, within 4.7 % of the truth**.
The estimate was then *corrected downward* by 0.923, a normalizer computed from the
ratio of the sample's 2117-key rate to the workload's — and applied to **both** halves,
including the one that key does not measure.

> **The correction is the error.** A normalizer derived from one census key is only
> valid for that key's population. Applied to the 2117 half it was harmless (that
> half came in 6.7 % high, consistent with a normalizer that was slightly too strong);
> applied to the plain half, which the key does not measure at all, it moved a 2.6 %
> estimate to a 21 % one. This is §18.6's own recorded limitation one level over —
> *"the `calls-1` framed share is measured on emitted code and applied to IL bodies,
> and those are different populations"* — and the rule that follows is:
> **when a rung spans two keys, normalize each half against its own key; where a half
> has no key, quote the raw share and widen the range instead of correcting it.**

The other estimating rules held. §15.4's bound rule was correctly *not* applied
(these are hand-written), and §19.1's whole-body-completeness ranking converted 1:1 —
the bucket drop equals the census gain to the function, for the fourth rung running.

### 19.5 The control group

§18.1's standing check: a shape the whole-body parser accepted as a leaf cannot
contain two calls, so any in-class function reading `calls-2plus` is a false positive
by construction. The new key lands exactly where its parse says it must:

```
addr-leaf           40,621   c0=40,621  c1=0  c2+=0
indirect-load-leaf 157,912   c0=157,912 c1=0  c2+=0
```

All 40,621 read `calls-0`, none reads anything else, and the frame axis is an
instrument that knows nothing about this production. Every other in-class key is
unchanged to the function.

### 19.6 What is NOT established, labelled

* **The twelve unwitnessed tags in [`is_ptr_any`] are a HYPOTHESIS.** Four are
  captured (`84`, `86`, `A4`, `A6`); the other twelve are the cross product of two
  axes each witnessed separately — the cv bits `0x20`/`0x10` (from `is_ptr4_kind`'s
  own captures) and the width nibble 2/4/6/8. The cross itself has no witness. Tags
  with bit `0x40` set are refused, and `kind` must be exactly `0x43`: a function/code
  pointer (`0x44`) is refused here even though `is_ptr4_kind` admits it as a loaded
  value, because "the pointee width does not matter" has not been checked for code.
* **The zero-offset `mr r3,rN` is measured and not implemented.** `7c832378` is in
  the capture (`p3.cpp` `z_r4`). It is refused because the identity leaf beside it
  refuses the same thing for the same reason, and widening one without the other
  would put two rules where there is one. It is a small, known, cheap item.
* **The `2C` is still assumed free on an "always observed" basis.** Pointer→pointer
  emits nothing at every captured site (`docs/IL_LOAD_TYPES.md` §3/§4, and `r_d` /
  `a_arr0` here), and cross-class conversions refuse. The sweep crosses it against
  cv-qualification and array decay; it does not cross it against a *reinterpret*,
  because none was produced.
* **The `28` payload `00 00` remains undecoded**, exactly as in
  `try_parse_indirect_load_leaf`. It is required literally.
* **The 33/863 `load with extra offset adds` sub-shape is measured and NOT taken**
  (≈1,200 workload functions). `p->t[2]` on an inherited array is `lwz r3,16(r3)` —
  captured, `p3.cpp` `l_e2`/`l_n1` — and admitting it means letting a *run* of offset
  adds into the load path, whose "exactly one add" rule is a real gate for the plain
  designator (a chained subscript there needs `slwi`/`lwzx`). 1,200 functions is not
  worth perturbing a gate with a documented wrong-bytes history in the same rung that
  changed its designator decoder.
* **`fixtures/cpp/w12_ptr_leaf_neg.cpp`'s `n_deref_c` is in class and its header says
  it must refuse.** `char f(char* pc){ return *pc; }` is `lbz r3` and T3's
  `LoadIndSized` admits it. This predates this rung and was not touched; the file
  still produces `Port=NotImplemented` whole, so nothing is mis-emitted, but the
  comment is stale. `n_addr_of` *was* touched: it is now `a_off4` in
  `fixtures/cpp/w16_addr_leaf.cpp`, graded byte-exact rather than merely refused.
* **The sub-shape census is a 19-TU sample, not the census.** Only the two totals it
  predicted are graded against the whole workload; the per-sub-shape shares in §19.1
  are the sample's. A `-subshape` split in the census key would make them exact and
  was not taken — the rung it would rank is §19.7 (2), and that one needs a
  measurement of its operators before its size matters.

### 19.7 The order of work, re-ranked

> **SUPERSEDED by §20.** Every ranking in this section was taken from a histogram
> whose operand-type key carried a per-TU type id and so hid the largest construct
> in the census across 256 shards. The board below is not wrong about the items it
> names — it is missing a 983,707-function row that outranks all of them.

The board is §18.7's, minus item 2, and with what §19.1 measured inside it:

1. **The general frame, plus per-COMDAT `.pdata`.** Unchanged and still first:
   802,655 `calls-2plus` functions across the census, ~199,000 of them in the
   `expr-call-in-expr` bucket alone, and none of the top rows is takeable without it.
2. **The two-member binary op — 455/863 of the 2117 `calls-0` block, ≈17,000
   functions.** Now the largest leaf-shaped thing left, and §19.1 is the first
   measurement of it. It is `lwz ; lwz ; <op>` and it is **not one rung**: the
   captured operators are `==`/`!=` (which reach the existing branchless compare
   spine, but over two *memory* operands rather than a formal and a literal),
   `-` then `>>` (a shift by a literal), and `-` then `/` by a constant (a
   magic-multiply). Ranking these needs an operator histogram, which the census does
   not yet carry for this key and which is the cheap next measurement.
3. **`data-addr-1sym` — 2,712, and 100 % `calls-1`.** Unchanged from §18.7 (3).
4. **`recv-object × type-ptr` — 2,410, and 100 % `calls-1`.** Unchanged from §18.7 (4).
5. **The whole-body store through 2117 — ≈370**, and the load-with-extra-adds —
   ≈1,200. Both are measured, both are one `stw`/`lwz`, and both are small. They are
   listed so nobody re-derives them.
6. **Control flow.** Unchanged, and still entangled with the frame.

### 19.8 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
./target/release/c2rs census fixtures/cpp/w16_addr_leaf.cpp   # 29/29 in class
./target/release/c2rs diff   fixtures/cpp/w16_addr_leaf.cpp   # Port=Match
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 112 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 46 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 43 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=3516 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-bma.jsonl
# the sub-shape census (gitignored scratch tooling):
#   work/bma/tools/bma.py   segment split, `call_tokens`, a best-effort tokenizer
#   work/bma/tools/bma2.py  reproduces the 2117 census key, classifies the tail
#   work/bma/tools/bma3.py  the whole-body forms of §19.1
#   work/bma/tools/bma4.py  the address leaf at BOTH designators — the 5.0x finding
#   work/bma/tools/objdis.py  minimal COFF + PPC dump (sections, symbols, .text, .pdata)
python3 work/bma/tools/bma3.py work/bma/ils/*
python3 work/bma/tools/bma4.py work/bma/ils/*
# the probes (gitignored scratch; every witness in §19.2 comes from one):
#   work/bma/probes/p1.cpp  the plain designator: widths, argument positions, offsets
#   work/bma/probes/p2.cpp  the 2117 designator, OUT OF LINE so c2 actually emits it
#   work/bma/probes/p3.cpp  the boundaries: offset 0 from r4/r5, 32764 vs 32768
#   work/bma/probes/p4.cpp  the §17.3(b) tell: 13 byte-identical bodies in one TU
./target/release/c2rs compile work/bma/probes/p2.cpp --keep-obj work/bma/p2.obj
python3 work/bma/tools/objdis.py work/bma/p2.obj
```

Always difference the scans through **absolute** paths and print each one's row count
and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several reflinked
worktrees with different contents, and reading one through a relative path has already
produced a published wrong number in this project.

## 20. D8, landed — the de-sharded census key, and the head it had been hiding

D7 gained +40,621 functions and **82.9 % of that gain (−33,688) came out of
`expr-load-type-*` keys the ranked histogram could not show**. The rung had been
ranked entirely from the 17.1 % that had a named key
(`expr-intrinsic-base-member-addr`, −6,933); the plain designator refusing the same
construct five times as often was found by grepping for the rule, not by the
instrument. That is the `GAPS.md` §6 sharding failure firing a second time, after
having been recorded, hand-regrouped once for one analysis, and left in the key.

This rung fixes the key and re-ranks. **The construct that was hiding is the
largest single row in the census**, by a factor of 4.7 over the top row that was
visible.

### 20.1 What was sharded, and what each key became

A TYPE is `<tag> <kind> <LEB128 id>` (`docs/IL_TYPE_TAGS.md` §1). The first two
bytes are fixed vocabulary — the tag is the *slot's* width plus a qualifier (`86`
plain, `A6` const, `96` volatile, `82`/`84`/`88` the other widths), the kind's low
nibble is the type **class** (1 signed · 2 unsigned · 3 data pointer · 4 code
pointer · 5 real · 6 aggregate · 7 void · A real literal). The id is **an index into
the TU's own type table**: every distinct pointee and every typedef gets a fresh
one. `Block::feature` put all three bytes in the bucket name.

Two keys were affected, and both are now `<tag><kind>`:

| old | new | shards folded | functions |
|---|---|---:|---:|
| `expr-load-type-XXXXXX` | `expr-load-type-XXXX` | 578 → 16 | 1,188,492 |
| `expr-lit-type-XXXXXX` | `expr-lit-type-XXXX` | 270 → 13 | 69,226 |

The head rows and their shard counts: `expr-load-type-A643` was **128** names,
`-8643` **128**, `expr-lit-type-8643` **128**, `-8641` **129**. A *single* TU
(`src/lazer/game/Game.cpp`) carries all 128 `A643` shards by itself, which is the
per-TU allocation stated as a measurement rather than as an inference.
Scan-wide: **1,363 distinct features → 544**.

**The id is kept, not discarded.** `Block::aux` still packs the whole triple exactly
as `blk_type` wrote it and `FnCensus::hex` still carries the raw bytes of the site,
so an analysis that wants the type-table index has it. It is only out of the *name*,
which is the only place it did damage.

### 20.2 The exact-partition check

Baseline `work/dc3-workload/scan-deshard-base.jsonl`, taken **in this worktree** per
§18.8's rule: 878 rows, `fn_total` 2,462,571, in class **320,641 (13.02 %)**, 1,363
keys, mismatch 0, 6 `match` / 7 `capture-fail`. Result: `scan-deshard2.jsonl`, same
list, flags and `--cwd`.

| | baseline | D8 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 320,641 (13.02 %) | 320,641 (13.02 %) | **0** |
| mismatch / `match` / `capture-fail` | 0 / 6 / 7 | 0 / 6 / 7 | 0 |
| distinct keys | 1,363 | **544** | −819 |
| functions leaving a sharded key | — | **1,257,718** | across 848 names |
| functions entering the folded keys | — | **1,257,718** | across 29 names |
| residual disagreement, per TU **and** per frame class | — | **0** | — |

The check is stronger than a total: every one of the 878 rows was compared key by
key *and* frame-class by frame-class, folding the baseline's names through the
rename, with **zero** residual. Every key outside the two families is byte-identical
per TU. This is a *coarsening*, which is checkable at the source as well as in the
data — same old key ⇒ same new key, pinned by
`the_operand_type_rekey_is_an_exact_coarsening` over every shape `feature()` can
take — because the parse itself is untouched and the change is in the key formatter.

### 20.3 The corrected ranking

Top rows over the 2,141,930 blocked functions, with the frame classes §18's audit
made available:

| # | functions | % blocked | calls-0 | calls-1 | calls-2plus | key |
|---:|---:|---:|---:|---:|---:|---|
| 1 | **666,907** | **31.1 %** | **298,770** | 147,829 | 220,308 | `expr-load-type-A643` |
| 2 | **316,800** | **14.8 %** | 83,040 | 149,817 | 83,943 | `expr-load-type-8643` |
| 3 | 141,800 | 6.6 % | 15 | 57,894 | 83,891 | `expr-intrinsic-this-adjust` |
| 4 | 116,016 | 5.4 % | 25,439 | 56,186 | 34,391 | `expr-intrinsic-base-member-addr` |
| 5 | 92,724 | 4.3 % | 8,267 | 80,562 | 3,895 | `expr-load-type-8645` (float) |
| 6 | 79,158 | 3.7 % | 2 | 79,154 | 2 | `expr-load-type-8885` (double) |
| 7 | 48,102 | 2.2 % | 1,622 | 4,674 | 41,806 | `body-0x29` |
| 8 | 39,361 | 1.8 % | 0 | 0 | 39,361 | `expr-call-in-expr-op-0x9B` |
| 9 | 34,795 | 1.6 % | 7,318 | 12,682 | 14,795 | `expr-intrinsic-memset` |
| 10 | 32,381 | 1.5 % | 4 | 31,485 | 892 | `expr-bit-and` |
| 11 | 29,552 | 1.4 % | 28,720 | 832 | 0 | `fn-tail-0xB9` |
| 12 | 28,285 | 1.3 % | 6,954 | 11,191 | 10,140 | `expr-lit-type-8643` |

**Rows 1 and 2 are the same construct at two tags** — a 4-byte data-pointer operand,
const-qualified and plain — and together they are **983,707 functions, 45.9 % of
everything blocked and 39.9 % of the whole corpus**. Add the pointer *literal* row
and it is 1,011,992. The top row the sharded histogram could show was
`expr-intrinsic-this-adjust` at 141,800: **the real head is 4.7× it, and it was never
on any list.**

The **whole-body-complete** column §14.1 introduced does not exist for these rows:
the `-whole` suffix is `mcall`'s, and no equivalent bit is carried for an operand-type
refusal. §20.5 measures it by counterfactual instead. For every
`expr-call-in-expr-*` row the suffix is still in the key and unchanged by this rung.

### 20.4 Larger than it looked — and the claim it falsifies

§18.7 closed with: *"Items 2, 3 and 4 total **37,494 functions** and are the entire
remaining local inventory above a thousand functions."* **That is false, and it was
false when it was written** — it was computed from the sharded histogram, in which no
pointer row was large enough to appear.

**Blocked and call-free (`calls-0`), ranked, at HEAD: 585,777 functions.**

| functions | key |
|---:|---|
| 298,770 | `expr-load-type-A643` |
| 83,040 | `expr-load-type-8643` |
| 28,720 | `fn-tail-0xB9` |
| 25,439 | `expr-intrinsic-base-member-addr` |
| 23,220 | `expr-lit-type-8212` |

The pointer rows alone hold **381,810 call-free functions** — provably needing no
frame, by §18's own instrument — against the 37,494 the frame audit called the whole
of it. §18.7's *practical* conclusion (the general frame is the biggest single piece
of work) survives; its claim to have enumerated the local inventory does not.

The two items §19.7 (5) deferred are **unchanged** by this rung — they live inside
`expr-intrinsic-base-member-addr`, which never sharded — and they are now two orders
of magnitude below the head: the load-with-extra-offset-adds ≈1,200 and the
whole-body store ≈370. Neither is the best available rung any more.

### 20.5 The next rung, and its size — MEASURED by counterfactual

The question the ranking cannot answer on its own is how much of a 983,707-function
row is *only* the type gate. Measured directly: `eat_int_like` was widened in a
**scratch build** to admit a 4-byte data-pointer type wherever an int-like one is
admitted, and the census re-run over the same 878 TUs. Nothing was committed; the
build was reverted and the gate re-run against the committed tree.

**Admitting the pointer operand type releases 1,011,992 functions from the type keys,
and 14,038 of them (1.4 %) become whole-body complete.** The census numerator goes
**320,641 → 334,679 (13.02 % → 13.59 %)**, and the shapes that gain are ones the port
already emits:

| shape | before | after | delta | frame class |
|---|---:|---:|---:|---|
| `straight-line` | 10,960 | 17,016 | **+6,056** | all `calls-0` |
| `int-tail-call` | 0 | 5,268 | **+5,268** | `calls-1` |
| `multiarg-tail-call` | 0 | 2,692 | **+2,692** | `calls-1` |
| `indirect-load-leaf` | 157,912 | 157,934 | +22 | `calls-0` |
| **total** | 320,641 | **334,679** | **+14,038** | 6,078 `calls-0` / 7,960 `calls-1` / **0** `calls-2plus` |

`int-tail-call` and `multiarg-tail-call` are **at zero on the real workload today**:
the port models `return g(a)` and `return g(a,b)` and the workload's calls almost
always pass a pointer, so the shape has never once fired outside the fixtures. That
is the single most useful thing in this table.

The other 98.6 % moves one token deeper, and the counterfactual names where to:
`expr-op-0x27` **+504,223** (the `27` sub-object address in a general expression
position, outside a leaf designator), `expr-convert` **+198,545** (the `2C` cast),
`expr-op-0x99` **+134,431** (the by-value temporary bind). **The pointer type is a
gate in front of the expression layer over pointers, not the layer itself** — which
is the honest reading of why row 1 is 31 % and the rung behind it is 14,038.

**The hazard, and it is measured too.** `p + 1` on an `int*` is `addi r3,r3,4`, not
1, so admitting a pointer into the modeled add-chain is a wrong-bytes emit and not a
gap. A second counterfactual added exactly that guard — refuse if the accepted
expression contains `02`/`03`/`04` over a pointer operand — and the numerator was
**identical at 334,679**: not one of the 14,038 does arithmetic on a pointer, while
**964** bodies elsewhere do and are refused by the guard. The rung must carry the
guard; it costs nothing it would otherwise have gained.

So the next rung is: **admit the 4-byte data-pointer operand type (`<tag> 43`) at the
`B9` LOAD and `33` LIT positions of `parse_expr`, guarded against arithmetic.
Estimated +14,038 functions, +4.4 % on the numerator, 6,056 of them call-free
straight-line leaves and 7,960 tail calls that need no frame.**

Stated before implementation, per §13.1's rule, with the bias direction called: the
14,038 is a **grammar-completeness floor, not a byte-exactness claim** — the
counterfactual says those bodies *parse*, and only the compiler can say whether they
*emit*. It is a floor for three reasons and could be low: only `eat_int_like` was
widened, so the result-type, store-type and call-return-type positions still refuse
int-only; the `86 43`/`A6 43` pair may not exhaust the pointer tags; and D7's own
lesson (§19.3) is that the same rule usually has a second site. It could equally be
optimistic if any of the three shapes emits differently for a pointer than for an
int — `IL_TYPE_TAGS.md` §3 measures identity as a bare `blr` for every type
including pointers, which is evidence for the leaf half and says nothing about the
argument half.

### 20.6 What is NOT established, labelled

* **`A6` = const, `96` = volatile is quoted from `readers.rs`'s comment, not
  re-verified here.** It is why rows 1 and 2 are two rows rather than one. The 2:1
  ratio between them is consistent with `A6 43` being mostly `this` (a `T *const`),
  but **no capture in this rung tested that**, and if the two tags turn out to be one
  construct the head is one 983,707-function row.
* **The counterfactual is a census measurement, not a differential.** No TU became
  wholly in class under it, so the four mode lanes and the sweep graded nothing about
  the widened build. `mismatch` stayed 0 in the counterfactual scans, which is a
  statement about TU-level acceptance and **not** evidence that the widened bytes
  would be right.
* **The class-nibble decode (`3` = data pointer, …) comes from
  `docs/IL_LOAD_TYPES.md` §1 and `IL_TYPE_TAGS.md` §2**, both capture-backed, and is
  re-used unchanged. This rung added no new type captures.
* **Aggregates (`kind & 0x0F == 6`) still shard slightly**, on the *size* nibble the
  aggregate encoding packs into the tag/kind pair. That is a property of the type and
  not a per-TU index, and the whole class is 3,107 functions across three keys, so it
  is recorded rather than fixed.
* **The `expr-op-0x27`/`expr-convert`/`expr-op-0x99` figures are counterfactual
  positions**, i.e. where the parse *would* stop once the type gate opens. They are
  not measurements of the committed instrument and must not be quoted as census rows.

### 20.7 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 112 pass 0 fail 0 error
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 46 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 43 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=3516 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-deshard2.jsonl        # 320641/2462571, 544 keys
# the exact-partition check: fold the baseline's names through the rename and
# compare per TU AND per frame class; the residual must be 0 (§20.2).
# the counterfactual (scratch, reverted): widen `eat_int_like` to take a
# `<tag> 43` type via `read_type`, rebuild, re-scan -> 334679/2462571.
```

Always difference the scans through **absolute** paths and print each one's row count
and `fn_total` first — see §18.8's warning, which is why this rung's baseline was
re-taken in its own worktree rather than read from a neighbouring one.

## 21. D9, landed — the 4-byte pointer operand, and the 98.6 % it does not buy

§20 de-sharded the census key and the head that appeared was one construct at two
tags: `expr-load-type-A643` (666,907) and `expr-load-type-8643` (316,800), a
4-byte data pointer in a `parse_expr` operand position — **983,707 functions,
45.9 % of everything blocked**. This rung opens that gate.

It is worth stating the shape of the result first, because the row size invites
the wrong reading: **admitting the pointer operand type releases 1,015,144
functions from the type keys and puts 14,016 of them (1.4 %) in class.** The
pointer *type* is a gate standing in front of the pointer *expression* layer, not
the layer itself. The rung is worth doing anyway — 6,056 call-free straight-line
leaves, and the first workload functions the two tail-call shapes have ever had —
but it is a 14,016-function rung sitting on a 983,707-function row, and the
remaining 98.6 % moves exactly one token deeper.

### 21.1 What the head is — MEASURED, with witnesses

Three claims §20 carried as inference are now capture-backed. Every byte string
below is transcribed from a live `--keep-il` capture of a tracked fixture.

**(1) `A6` is `const` on the POINTER, not on the pointee — so the head really is
two rows, and the second one is bigger.** §20.6 flagged this as quoted from
`readers.rs` rather than verified. Verified now, and the discriminating case is
the one no probe had run:

```text
  int f(int* p)         B9 p 86 43 F4 08                     plain
  int f(const int* p)   B9 p 86 43 87 20                     const POINTEE — still 86
  int f(int* const p)   B9 p A6 43 8F 20  2C 86 43 F4 08 00  const POINTER  — A6
  int C::m() const      B9 t A6 43 8A 20  2C 86 43 8F 20 00  `this`         — A6
```

A const-qualified *pointee* does not move the tag; it moves only the type-table
id, which §20 took out of the key. So `A643` is `T *const` — overwhelmingly
`this` — and `8643` is everything else. They are **two types and one value
class**: a 4-byte word in a register, lowered identically, which is what lets one
predicate admit both. `A643` is also the harder half, because in the wild it
arrives with a `2C` cv-strip attached (see §21.5).

**(2) c1xx PRE-SCALES pointer arithmetic. The hazard as stated in §20.5 does not
exist in the IL.** §20.5 said `p + 1` is `addi r3,r3,4` and concluded that
admitting a pointer into the modeled add chain is a wrong-bytes emit. The first
half is true of the *machine code*; the second does not follow, because the
scaling has already happened by the time the backend sees it:

```text
  int*    f(int* p)          { return p + 1; }  B9 p 86 43 F4 08 · 33 86 41 12 04 · 02
  int*    f(int* p)          { return p + 3; }                     33 86 41 12 0C · 02
  int*    f(int* p)          { return p - 1; }                     33 86 41 12 04 · 03
  char*   f(char* p)         { return p + 1; }  B9 p 86 43 F0 08 · 33 86 41 12 01 · 02
  double* f(double* p)       { return p + 1; }  B9 p 86 43 C1 08 · 33 86 41 12 08 · 02
  int*    f(int* p, int k)   { return p + k; }  B9 p · B9 k 86 41 74 · 33 86 41 12 04 · 04 · 02
  int     f(int* p, int* q)  { return p - q; }  B9 p · B9 q · 03 · 33 86 41 74 02 · 0A
```

The literal is the byte offset, the variable index carries an explicit `04`
multiply by the pointee width, and the pointer *difference* divides with an
arithmetic shift `0A` that the operand vocabulary refuses on its own. The modeled
chain would in fact emit the right instruction for all but the last.

**The guard ships anyway.** That is a decision, not an oversight, and the reasons
are in `body/expr.rs`: it is a *second* rule (that the front end scales at every
arity, pointee width and cv-spelling this parser can reach) which would need its
own byte grading over its own sweep axis; it costs **0** of the gain, measured
twice; and with it the pointer-difference class fails closed twice rather than
once. `fixtures/cpp/w17_ptr_operand_neg.cpp` is the whole boundary, and
`scripts/expr_sweep.sh` sweeps it across eight pointee widths × nine expressions
× three positions. Deleting the guard is a rung, with a fixture cost — not a
tidy-up.

**(3) The rule has three sites, not one, and two of them are worth nothing
alone.** §20.5's counterfactual widened `eat_int_like` globally and could not say
which position paid. Measured here by building the narrow version first: widening
**only** `parse_expr`'s LOAD and LIT moved **1,013,468** functions between census
keys and moved the numerator by **exactly 0**. A real call site spells the pointer
twice —

```text
  int h1(int*); int f(int* p){ return h1(p); }
    … BD 86 41 74 00 <id> · B9 p 86 43 F4 08 · 55 86 43 F4 08 · 4C · 41 86 41 74
                             ^ the LOAD           ^ the FORMAL type
```

— and a body that *returns* a pointer spells it a third time at the `41` result.
This is D7's §19.3 lesson firing again, and it is the reason §13.1's "grep for
every site implementing the rule" is in the method: the estimate was formed on one
site and the defect sat on three.

### 21.2 The estimate, stated before the outcome

Per §13.1's rule. §20.5's counterfactual said **+14,038** and called itself a
floor. This rung's plan was narrower than that counterfactual — four named
positions rather than every `eat_int_like` call site, plus the arithmetic guard —
so:

> **Estimate: +13,000 ± 2,000, biased LOW relative to the counterfactual's
> 14,038.** Low because the counterfactual's global widening also reached the
> `32` store type and, through `eat_value_type(ValueClass::Int4)`, relaxed the
> class-*agreement* gate between a `30` load, its `2C` target and its `41` result
> — positions this rung deliberately leaves alone. High only if the four
> positions turn out to interact.

Outcome **+14,016**, inside the interval, 22 short of the counterfactual — and
the 22 are exactly its `indirect-load-leaf` row, i.e. exactly the agreement-gate
relaxation named as the reason for the low bias. The prediction and its stated
bias direction both held, which is the first time in this document that the
counterfactual, the estimate and the outcome have been reconciled item by item.

### 21.3 What shipped

One predicate and one guard.

* `readers::is_ptr4_kind` **moved** from `body/shapes.rs` to `readers.rs` and is
  now `pub(crate)`. It was already the project's answer to "is this TYPE a 4-byte
  pointer *value*" — the pointer-identity leaf and the pointer getter are gated on
  it and byte-graded — and the alternative was a second copy, which is the "one
  fact, two locators" mistake `find_byte` was deleted for.
* `readers::eat_int_like_or_ptr4` — consume an int-like triple **or** a 4-byte
  pointer, reporting *which*, and leaving the cursor untouched on a refusal. The
  caller is told which because the two are not interchangeable under arithmetic.
* Applied at **four** positions, all of them annotations on a value rather than
  selectors for an instruction: `parse_expr`'s `B9` LOAD and `33` LIT, the
  `55 <TYPE>` call-argument formal type, and the `41 <TYPE>` result type.
* **The guard** sits at the end of `parse_expr`, on the whole sub-expression
  rather than on the adjacent token: one `Vec<IlOp>` is one value, and the pointer
  may be loaded before or after the operator, so a single check when the stream
  ends covers every interleaving. Key `expr-ptr-arith:eof`, so its cost is a
  number rather than an argument.

Untouched, and deliberately: the `30` indirect load, where the pointee's width
*does* pick the instruction (`lbz`/`lhz`/`lwz`) and which is gated by
`is_ptr_to_4` / `value_class`; the `32` store type; and `mcall`'s speculative
second-blocker walk, which still uses the narrow `eat_int_like` and therefore now
*understates* whole-body completeness for pointer-carrying `26` chains.

### 21.4 The outcome — MEASURED

Baseline `work/dc3-workload/scan-ptrbase.jsonl`, taken **in this worktree** per
§18.8: 878 rows, `fn_total` 2,462,571, in class **320,641 (13.02 %)**, 544 keys,
mismatch 0, 6 `match` / 7 `capture-fail`. Result `scan-ptr21.jsonl`, same list,
flags and `--cwd`.

| | baseline | D9 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 320,641 (13.02 %) | **334,657 (13.59 %)** | **+14,016** |
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| distinct keys | 544 | 570 | +26 |
| TUs changing class | — | **0** | — |
| functions LOST, any TU | — | **0** | — |

By shape, with the frame class §18's audit makes available:

| shape | before | after | delta | frame class |
|---|---:|---:|---:|---|
| `straight-line` | 10,960 | 17,016 | **+6,056** | all `calls-0` |
| `int-tail-call` | 0 | 5,268 | **+5,268** | all `calls-1` |
| `multiarg-tail-call` | 0 | 2,692 | **+2,692** | all `calls-1` |
| **total** | 320,641 | **334,657** | **+14,016** | 6,056 `calls-0` / 7,960 `calls-1` / **0** `calls-2plus` |

`int-tail-call` and `multiarg-tail-call` were at **zero on the real workload**:
the port has modeled `return g(a)` and `return g(a,b)` since the MVP and the
workload's calls almost always pass a pointer, so until this rung the shapes had
never once fired outside the fixtures.

**The guard's cost, measured rather than inherited.** With the guard the
numerator is 334,657; with the guard compiled out and everything else identical it
is **334,657**. It catches **964** bodies — independently the same 964 §20.5
reported — and costs **0**. Not one of the 14,016 does arithmetic on a pointer.

**And it is byte-graded, which the counterfactual was not.** §20.6 recorded that
no TU flipped under the counterfactual, so the lanes and the sweep graded nothing
about the widened build and its `mismatch 0` said nothing about whether the bytes
were right. This rung:

* `fixtures/cpp/w17_ptr_operand.cpp` — 23 functions, **23/23 in class**, whole obj
  **byte-exact** against real `c2`. The pointee across all four target widths plus
  `void`, pointer-to-pointer and a code pointer; the four tag spellings, including
  the in-class `A6` witness `int* const`; the null-pointer LIT in an argument and
  as a whole body; the pointer in every argument slot at arities 1–3; the `41`
  result with no call; and a reference parameter, which spells no pointer in C++
  at all and is one throughout the IL.
* `fixtures/cpp/w17_ptr_operand_neg.cpp` — 25 functions, **0/25 in class**, and the
  file must never mismatch. The arithmetic boundary across pointee widths and both
  operators, as a body and inside an argument; the guard's collateral (arithmetic
  on the *int* beside an untouched pointer); the `A6` `this` behind its `2C`; the
  relational, the address-of-a-local, the 8-byte non-pointer, and the ninth
  argument.
* `scripts/expr_sweep.sh` grew **227 cases** (3,516 → 3,743, `mismatches=0`) over
  exactly those axes: 15 pointees × 5 positions, every argument slot at arities
  1–4 with the pointer moved through each, mixed-pointee argument lists, 8 pointee
  widths × 9 arithmetic expressions × 3 positions, and 13 refusing neighbours.
  **104 of the new cases grade `Match`** and 136 `NotImplemented`, so the sweep is
  measuring emitted bytes here and not only refusals.
* Four mode lanes, mismatch 0 in each: `/Ox` 46 → **47**, `/O1` 43 → **44**, `/O2`
  43 → **44**, `/Ox /Gy` 43 → **44**. The `+1` is `w17_ptr_operand.cpp` in every
  lane; no baseline dropped.

### 21.5 Where the other 98.6 % went

The exact accounting, over the whole 878-TU scan. Five keys lost population and
nothing else did:

| functions | key that emptied |
|---:|---|
| 666,907 | `expr-load-type-A643` |
| 316,800 | `expr-load-type-8643` |
| 28,285 | `expr-lit-type-8643` |
| 1,676 | `result-type-0x41` |
| 1,476 | `expr-load-type-8644` (a code pointer at a LOAD) |
| **1,015,144** | **total released** |

1,001,128 of them landed on a deeper key and 14,016 came in class — the two sum to
the release exactly. So the answer to "does the bucket drop equal the census
gain" is **no, and by three orders of magnitude**: this rung cleared a *first*
blocker for a million functions and finished the body for 1.4 % of them. Where
the rest stopped:

| functions | new key | what it is |
|---:|---|---|
| +504,245 | `expr-op-0x27` | the sub-object byte-offset add, in a general expression position rather than inside a leaf designator |
| +198,545 | `expr-convert` | the `2C` cast / cv-strip — where the `A6` `this` goes |
| +134,431 | `expr-op-0x99` | the by-value temporary bind |
| +44,262 | `expr-out-of-class:eof` | **parses**, and the port declines the lowering (a result not already in r3, and its neighbours) |
| +15,682 | `expr-intrinsic-dynamic-cast` | |
| +13,087 | `expr-op-0x30` | an indirect load in a general expression position |
| +964 | `expr-ptr-arith:eof` | the guard |

Two of those are a different kind of row from the rest. `expr-out-of-class:eof`
is the first sizeable population that is **grammar-complete and codegen-blocked**
— the parse reaches the end of the segment and `straight_line_is_out_of_class`
refuses it — so it is the first pointer-derived work that needs an emitter rather
than a decoder. And `expr-op-0x27` at 504,245 is now the largest single row in the
census by a factor of 2.5, which makes it the ranked next rung; it is the same
`27` the address leaf (§19) already lowers in a *designator* position, so the
question it poses is whether that lowering generalizes to an expression position
rather than whether the construct is understood.

### 21.6 What is NOT established, labelled

* **`is_ptr4_kind` admits code pointers (kind `44`) as well as data pointers
  (`43`), and the census head was data pointers.** The `44` population is small
  (1,476 functions at a LOAD) and is admitted on the existing predicate's existing
  claim — both load with the same `lwz` — plus one fixture (`t_fp`) and five sweep
  cases. It is a widening this rung did not measure separately.
* **The 22-function gap to §20.5's counterfactual is inferred, not measured.** It
  is attributed to the `ValueClass::Int4` agreement gate on the strength of the
  counterfactual's `indirect-load-leaf +22` row matching it exactly; no build was
  made to confirm the mechanism.
* **`mismatch 0` on the workload is still a TU-level statement.** 865 of the 878
  TUs are `vocab-gap` and never reach the port at all, so the workload scan grades
  the *decoder*. The byte grading of this rung is the two fixtures, the 3,743-case
  sweep and the four lanes — 104 pointer cases and 23 fixture functions with
  emitted bytes compared — and nothing wider than that.
* **`mcall`'s second-blocker walk was not widened.** Its `-whole` bits therefore
  now understate whole-body completeness wherever a `26` chain carries a pointer
  operand. That makes the `expr-call-in-expr-*-whole` counts conservative, not
  wrong, and it was left out of this rung deliberately.
* **The pre-scaling measurement (§21.1 (2)) is seven witnesses from one probe
  TU.** It is recorded as the reason the guard is a conservatism rather than a
  rescue; it is *not* sufficient to delete the guard, which is the whole point of
  writing it down as a rung rather than acting on it.
* **`expr-out-of-class:eof` is not decomposed.** 44,262 functions is stated as one
  row; which of `straight_line_is_out_of_class`'s clauses each hits is unmeasured.

### 21.7 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 114 pass 0 fail 0 error
./target/release/c2rs census fixtures/cpp/w17_ptr_operand.cpp     # 23/23 in class
./target/release/c2rs diff   fixtures/cpp/w17_ptr_operand.cpp     # Port=Match
./target/release/c2rs census fixtures/cpp/w17_ptr_operand_neg.cpp # 0/25 in class
./target/release/c2rs diff   fixtures/cpp/w17_ptr_operand_neg.cpp # Port=NotImplemented
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 47 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 44 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=3743 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-ptr21.jsonl        # 334657/2462571, 570 keys
# the type-tag witnesses of §21.1, from the tracked fixtures themselves:
./target/release/c2rs census fixtures/cpp/w17_ptr_operand.cpp     --keep-il /tmp/il-pos
./target/release/c2rs census fixtures/cpp/w17_ptr_operand_neg.cpp --keep-il /tmp/il-neg
# the guard's cost: compile out the `saw_ptr` check, rebuild, re-scan -> 334657.
```

Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first — §18.8.

## 22. D10, landed — the ranking that refuted its own head, and the register move

§21 closed with `expr-op-0x27` "the ranked next rung; it is the same `27` the
address leaf (§19) already lowers in a *designator* position, so the question it
poses is whether that lowering generalizes." This rung asked, measured, and the
answer is **no** — the row is 505,122 functions and 0.14 % of it is takeable.
The rung that shipped is the one beside it: 43,319 functions, one instruction,
and it converted 1:1.

It also found two live wrong-bytes emits on mainline, which outranked everything
and went first (§22.6).

### 22.1 The two candidates, decomposed — MEASURED

Baseline taken **in this worktree** per §18.8, and its row count and denominator
printed before any differencing: `work/dc3-workload/scan-base27.jsonl`, 878
rows, `fn_total` 2,462,571, in class **334,657 (13.59 %)**, mismatch 0, 6
`match` / 7 `capture-fail`. That agrees with §21.4's figure, which is the only
thing agreeing on `fn_total` proves.

| | functions | calls-0 | calls-1 | calls-2plus |
|---|---:|---:|---:|---:|
| `expr-op-0x27` | 505,122 | 314,350 | 74,659 | 116,113 |
| `expr-out-of-class:eof` | 46,200 | **46,200** | 0 | 0 |

**`expr-out-of-class` was one key covering five different lowerings** — a
register move, a `lis`/`ori`, a strength-reduced multiply, a `subfic`, a stack
frame — which is `GAPS.md` §6's conflation failure in miniature: a row that
cannot be decomposed cannot be ranked. Split (census-key-only change,
numerator unchanged):

| functions | clause | what it costs |
|---:|---|---|
| **43,319** | `expr-out-of-class-bare-nonfirst-formal` | one `mr r3,rN` |
| 2,881 | `expr-out-of-class-bare-nonformal` | nothing implied — a global, a `.sy` local |
| 0 | `expr-out-of-class-formals9` | a frame |
| 0 | `expr-out-of-class-wide-neg-lit` | `lis`/`ori` for a negative |
| 0 | `expr-out-of-class-mul-by-lit` | strength reduction |
| 0 | `expr-out-of-class-lit-minus-reg` | `subfic` |

Three of the six clauses are **zero on the real workload** — they exist only in
fixtures — which is a measurement the single key could not make. The head clause
is 93.8 % of the row, every one of it `calls-0`, and spread over **831 of the
878 TUs** (largest single TU 196), so it is not one file's idiom.

**And it is 100 % whole-body complete, by construction rather than by estimate.**
The refusal is raised in `parse_segment_shape` *after* `parse_expr`,
`eat_return_plumbing` **and** `parse_params` have all succeeded — the `:eof` in
the key is the parse reaching the end of the segment. There is no second blocker
to discover behind this row, which no previous rung in this document could say.

### 22.2 The `27` counterfactual — MEASURED, and it refutes §21.5's ranking

`parse_expr` was given a `0x27` arm in a **scratch build** (consume
`27 <TYPE>`, fold the preceding literal into an `AddrOf`), with a thread-local
flag sunk in `parse_segment_detail` so that a body which parses to the end is
counted and **never claimed in class**. Nothing was committed; the build was
reverted and every number below re-taken against the committed tree.

**Admitting the `27` releases all 505,122 functions and leaves 685
grammar-complete — 0.14 %.** Where the rest stop:

| functions | new key | what it is |
|---:|---|---|
| +218,166 | `expr-op-0x30` | an indirect load in a general expression position |
| +76,893 | `expr-op-0x99` | the by-value temporary bind |
| +55,828 | `expr-op-0x32` | a store |
| +27,095 | `expr-load-type-8645` | a float operand |
| +14,218 | `expr-call-in-expr-data-addr-then-off-add-more` | |
| +11,933 | `expr-load-type-8212` | |
| +10,643 | `expr-call-in-expr-data-addr-then-off-add-whole` | |

The largest destination is the one the port explicitly refuses on measured
grounds: `*p + 1` puts the load in r11 and `*p * 3` strength-reduces
(`codegen::indirect_load_text`), so `expr-op-0x30` is not a decode gap either.
**`expr-op-0x27` is a gate standing in front of the expression layer over
pointers, exactly as the pointer *type* was a gate standing in front of it** —
D9's finding one token deeper, and the second time this document has been handed
"the largest row is the obvious next rung" and measured it to 1.4 % and then
0.14 %.

The 685 are not zero-value, but they are not a rung: they carry an `AddrOf` into
a general expression, which no capture covers.

### 22.3 The ranking, and the estimate — stated before the outcome

| | `expr-op-0x27` | `expr-out-of-class-bare-nonfirst-formal` |
|---|---:|---:|
| row | 505,122 | 43,319 |
| whole-body complete | **685 (0.14 %)** | **43,319 (100 %)** |
| needs a frame | 190,772 | **0** |
| lowering | `AddrOf` in an expression — no capture | one `mr r3,rN` — captured |
| locality tell | not run (nothing to take) | **run, passes** |

Per §13.1's rule:

> **Estimate: +43,300 ± 700, biased LOW.** Biased low because the same rule has
> a second parser site — `try_parse_addr_leaf`'s `off == 0 && ix != 0` refusal,
> the zero-offset sub-object address from a non-first argument, which is the
> same `mr` — whose population was **not** measured and which lands in other
> keys. High only if a body in the row emits something other than the move.

Outcome **+44,003**: 43,319 from the named key and **684** from that second
site. The low bias was the right call and its named cause was the whole of it.

The 684 are, to within one function, the 685 the `27` counterfactual measured as
its entire grammar-complete residue — because a zero-offset designator spells
its offset as a literal `0` behind a `27`, so those bodies were filed under the
505,122-function row. **The whole of what candidate B would have bought arrived
as a side effect of candidate A's second site.**

### 22.4 The locality tell, run before committing — MEASURED

§6's `data-addr` rung was ranked #1 at 21,642 and yielded 0 because instruction
selection there depends on a whole-TU constant-pool layout; the cheap tell is
several byte-identical source functions in one TU emitting *different*
sequences. Run first, one TU, eight functions:

```text
  int  f2(int a,int b)               { return b; }   7c832378   mr r3,r4
  int  g2(int a,int b)               { return b; }   7c832378   identical
  int  f3(int a,int b,int c)         { return c; }   7ca32b78   mr r3,r5
  int  g3(int a,int b,int c)         { return c; }   7ca32b78   identical
  int  f4(int a,int b,int c,int d)   { return d; }   7cc33378   mr r3,r6
  S*   p2(int a, S* s)               { return s; }   7c832378   mr r3,r4
  S*   p3(S* r, S* s)                { return s; }   7c832378   mr r3,r4
  int  h8(…8 ints…)                  { return h; }   7d435378   mr r3,r10
```

The word is a function of the formal's argument slot and of nothing else — not
of position in the file, not of the pool, not of the other functions present.
`mr rD,rS` is `or rD,rS,rS` (opcode 31, XO 444). A second probe TU adds the
member and address forms:

```text
  int  C::mm(int x,int y) const      { return y; }   7ca32b78   mr r3,r5  (this=r3)
  S*   C::ps(S* q) const             { return q; }   7c832378   mr r3,r4
  int* a0(int k, S* s)               { return &s->a; } 7c832378 mr r3,r4
  int* a4(int k, S* s)               { return &s->b; } 38640004 addi r3,r4,4
  S*   cv(int k, const S* s)         { return (S*)s; } 7c832378 mr r3,r4
  unsigned u2(int a, unsigned b)     { return b; }   7c832378   mr r3,r4
  short    s2(int a, short b)        { return b; }   7c832378   mr r3,r4
  long long l2(int a, long long b)   { return b; }   7c832378   mr r3,r4
  float    f2(int a, float b)        { return b; }   4e800020   blr — NOT this class
```

`a0`/`a4` are the pair that separates the two lowerings, and the last line is
where the FP alarm came from (§22.6).

### 22.5 What shipped

The rule had **three** sites, and grepping for them before quoting a number is
§13.1's method:

* `chain::straight_line_out_of_class_ctx` — the clause is gone; a bare LOAD is
  refused only when its token is **not a formal at all**. The function now
  returns *which* clause fired rather than a bool, so every remaining refusal is
  its own census key and the predicate is `.is_some()` of it.
* `codegen::select_text`'s finalize — the `Base::Phys(other)` arm, which had
  carried the refusal since the class was written, records a `PlanOp::RegMove`.
  It is always the last plan entry, so its destination is r3 by the existing
  rule; the mode does not reach it, because there is no intermediate to
  allocate.
* `codegen::addr_leaf_text` — the zero-offset-from-a-non-first-argument refusal
  becomes the same `mr`. Its parser twin `try_parse_addr_leaf`'s
  `off == 0 && ix != 0` is gone with it.
* `shapes::try_parse_assign_body_detail` — **a fourth site, and it was a
  defect.** It produced a `BodyShape::StraightLine` without consulting the
  out-of-class predicate at all, so `int f(int a){ int x = a; return x * 3; }`
  censused **in class** while `PortC2` returned `NotImplemented`: exactly the
  census/gate disagreement the predicate was extracted from codegen to prevent,
  reintroduced by a second producer that never called it. It shares the
  predicate now.

`try_parse_ptr_identity_leaf` needed no change — it already called the shared
predicate rather than keeping a copy, which is why the pointer half of this rung
came for free.

### 22.6 The alarm this rung found first — TWO live wrong-bytes emits

`float f2(int a, float b) { return b; }` above compiles to a bare `blr`, and
probing why produced a `Mismatch` on mainline. Two of them, both in the FP leaf,
both the `GAPS.md` §6 pattern (*two facts that share one field until some
construct pulls them apart*), and neither reachable from the fixture corpus:

```text
  float mixfp(int a, float b, float c) { return b * c; }
    c2    ec2100b2   fmuls f1,f1,f2
    port             fmuls f1,f2,f3          WRONG

  float fp_pass2(float a, float b)     { return b; }
    c2    fmr f1,f2 ; blr
    port              blr                    WRONG
```

1. **`float_leaf_text` maps parameter `n` to `f(n+1)`.** The FP file is numbered
   over the FP parameters *alone*, so any non-FP parameter ahead of a float
   breaks the identity. `fixtures/cpp/w13_fabi.cpp` **states this rule in a
   comment and carries `fp_skip`, which is exactly the failing case** — it hid
   because that TU also holds an out-of-class function and the port is
   all-or-nothing per TU, so those bytes were never emitted. Alone in a TU it
   reproduces immediately. A characterization fixture that documents a rule the
   emitter does not implement is not a test of it.
2. **A bare `return <FP parameter>` that is not the first is an `fmr`.** The
   integer class has gated precisely this shape since it was written — it is the
   43,319-function clause this rung is *about* — and the FP class never got the
   same gate. `GAPS.md` §6 instance 2 verbatim: *a locator nobody consults is
   not shared.*

**Gated closed, not fixed:** `try_parse_float_leaf` now requires **every formal
to appear as an FP operand of the body**. Each such operand carries the FP type
in its own `B9` LOAD, so a formals list holding nothing else is what proves the
index is the register number. The remaining over-refusal is honest rather than
lazy: whether `float f(float a, float b){ return b*b; }` needs an `fmr` depends
on `a`'s type, the body never mentions `a`, and the two spellings have
**identical body bytes**. That is the three-valued *undetermined* `GAPS.md` §6
asks for. `.sy` records each formal's type kind and would decide it — the reader
already reads the kind and discards it — which is a rung with its own grading,
not a tidy-up. Codegen keeps a second lock: a single remaining FP stack value
that is not f1 refuses.

**Cost, measured: census 334,657 → 333,652, −1,005.** Every one of those 1,005
moves to `expr-load-type-8645` and every one of them was numerator that could
never have been emitted. A rung that *lowers* the headline by removing bodies
the port would have got wrong.

`fixtures/cpp/w13_fparam_neg.cpp` (19 functions, 0 in class, never mismatching)
and 100 sweep cases over lead-parameter type × position × arity × width pin it.
**28 of those cases mismatch against the previous build and 0 against this one**,
which is the check that the new axis is not vacuous.

### 22.7 The outcome — MEASURED

Baseline for the rung is the post-alarm tree: `scan-fpfix.jsonl`, 878 rows,
`fn_total` 2,462,571, in class **333,652 (13.55 %)**. Result `scan-w18.jsonl`,
same list, flags and `--cwd`.

| | baseline | D10 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 333,652 (13.55 %) | **377,655 (15.34 %)** | **+44,003** |
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| TUs changing class | — | **0** | — |
| functions LOST, any TU | — | **0** | — |
| TUs gaining | — | 831 | max 196 |

**The bucket drop equals the census gain exactly, with zero residue** — the
fourth rung in this document to convert 1:1, and the first whose row was
*known* to be whole-body complete before it was taken rather than inferred from
a `-whole` suffix:

| | released | landed |
|---|---:|---:|
| `expr-out-of-class-bare-nonfirst-formal` | −43,319 | `straight-line` +43,319 |
| `expr-op-0x27` | −684 | `addr-leaf` +684 |
| **total** | **−44,003** | **+44,003** |

No key gained population. All 44,003 are `calls-0`.

Byte-graded, and this rung's grading is not only census movement:

* `fixtures/cpp/w18_reg_move.cpp` — **38/38 in class, whole obj byte-exact**
  against real `c2`. Every argument slot r4..r10 at every arity; `int` and
  `unsigned`; six pointer spellings including `void*`, `int**` and a `const`
  cast; member functions at three arities (where `this` takes r3); the
  zero-offset/nonzero-offset address pair; an 8-byte by-value aggregate ahead of
  the moved formal; the tail-call argument setup; and two byte-identical bodies
  in the file for the locality tell.
* `fixtures/cpp/w18_reg_move_neg.cpp` — **15/15 refused**, and the file must
  never mismatch. A `Big` by-value aggregate (where the index stops being the
  register number — `GAPS.md` §6 instance 4), the ninth argument, a global and a
  file-scope `static`, the ternary, and the narrow/wide/FP widths.
* `scripts/expr_sweep.sh` grew **268 cases** (3,743 → 4,011, `mismatches=0`):
  100 on the FP-parameter axis and 168 on the move — every slot × arity, 14
  value classes × 3 positions, the address pair at 3 positions, members at 3
  arities, and the aggregates/ninth-argument/global neighbours that must refuse.
  **114 of the move cases grade `Match`**, so the sweep is comparing emitted
  bytes here and not only counting refusals.
* Four mode lanes, mismatch 0 in each: `/Ox` 47 → **48**, `/O1` 44 → **45**,
  `/O2` 44 → **45**, `/Ox /Gy` 44 → **45**. The `+1` is `w18_reg_move.cpp` in
  every lane; no baseline dropped.

A new shape came with it that no fixture could have predicted: **a tail call
whose argument is a non-first formal.** `int_tail_call_text` builds its argument
setup by running `select_text` and dropping the trailing `blr`, so
`return g(b);` is now `mr r3,r4 ; b g` — byte-exact, and unreachable before this
class existed.

### 22.8 The corrected ranking

Over the 2,084,916 still-blocked functions:

| # | functions | % blocked | calls-0 | calls-1 | calls-2plus | key |
|---:|---:|---:|---:|---:|---:|---|
| 1 | 504,438 | 24.2 % | 313,666 | 74,659 | 116,113 | `expr-op-0x27` |
| 2 | 225,341 | 10.8 % | 10,287 | 71,380 | 143,674 | `expr-convert` |
| 3 | 141,800 | 6.8 % | 15 | 57,894 | 83,891 | `expr-intrinsic-this-adjust` |
| 4 | 134,431 | 6.4 % | 0 | 90,274 | 44,157 | `expr-op-0x99` |
| 5 | 118,330 | 5.7 % | 27,138 | 56,186 | 35,006 | `expr-intrinsic-base-member-addr` |
| 6 | 98,813 | 4.7 % | 10,702 | 84,215 | 3,896 | `expr-load-type-8645` (float) |
| 7 | 82,810 | 4.0 % | 2 | 82,806 | 2 | `expr-load-type-8885` (double) |
| 8 | 48,102 | 2.3 % | 1,622 | 4,674 | 41,806 | `body-0x29` |

Row 1 is unchanged in size and **now carries a measurement**: 0.14 % of it is
whole-body complete, so it is not the next rung and should stop being listed as
one. The honest reading of rows 1–5 together is that ~1.1 M functions are
waiting on **one** thing — an expression layer over pointers (`27` designators,
`2C` conversions, `99` binds) — and that layer needs the general frame for
between a third and all of each row.

The next rungs this rung can name, with their measured populations:

1. **The FP `fmr`, and `.sy`'s formal type kinds.** `.sy` already reads each
   formal's kind and discards it; carrying it makes the FP-argument index
   derivable, un-refuses what §22.6 gated closed, and gives the FP leaf the
   register move the integer one now has. Population: the 1,005 this rung
   removed plus the `float`/`double` operand rows' leaf share. It is also the
   only rung this document can name that *starts* from a known mis-emit.
2. **`expr-out-of-class-bare-nonformal`, 2,881 functions**, all `calls-0`. Not
   one lowering but several — a global read is a pool reference and therefore
   has §6's non-local hazard, a `.sy` local is not. Needs decomposing before it
   is worth ranking; it is small either way.
3. **`fn-tail-0xB9`, 29,552 functions, 28,720 of them `calls-0`** — the largest
   call-free row that is not part of the pointer-expression layer.

### 22.9 What is NOT established, labelled

* **The `27` counterfactual is a grammar measurement, not a differential.** No
  TU flipped under it, so no lane and no sweep graded the widened build; its
  `mismatch 0` is a statement about TU-level acceptance. Its 685 is a grammar
  upper bound with no codegen gate applied, exactly as §21.5's 14,038 was.
* **The 684/685 correspondence is asserted from two independent counts, not
  traced function by function.** They are one apart and the mechanism (a
  zero-offset designator spells `33 <int> 0 27 <TYPE>`) is understood, but no
  per-function identity check was run.
* **The move was measured on `/Ox` probes and graded on all four lanes, but the
  878-TU census is `/O1`.** The instruction has no intermediate to allocate, so
  the one rule that differs between the modes cannot reach it; that is an
  argument, and the lanes are the evidence.
* **`mismatch 0` on the workload is still a TU-level statement.** 865 of the 878
  TUs are `vocab-gap` and never reach the port. This rung's byte grading is the
  two fixtures (53 functions), the 4,011-case sweep and the four lanes.
* **The FP gate's over-refusal is not sized.** How many workload functions are
  FP leaves with an unused formal is not measured — they are inside the
  `expr-load-type-8645`/`-8885` rows and were never separated.
* **`straight_line_out_of_class_ctx`'s three zero clauses are zero on *this*
  workload.** A corpus with hand-written arithmetic rather than game engine
  code would populate `mul-by-lit`; the finding is about this corpus.
* **The 8-byte by-value aggregate is admitted on `.sy`'s declared width alone.**
  `int f(Pair v, int b){ return b; }` is byte-graded as `mr r3,r4`, and the
  claim that *every* 8-byte aggregate takes exactly one GPR rests on
  `ONE_GPR_MAX` and two witnesses, not on the ABI document.

### 22.10 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 117 pass 0 fail 0 error
./target/release/c2rs census fixtures/cpp/w18_reg_move.cpp        # 38/38 in class
./target/release/c2rs diff   fixtures/cpp/w18_reg_move.cpp        # Port=Match
./target/release/c2rs census fixtures/cpp/w18_reg_move_neg.cpp    # 0/15 in class
./target/release/c2rs diff   fixtures/cpp/w18_reg_move_neg.cpp    # Port=NotImplemented
./target/release/c2rs census fixtures/cpp/w13_fparam_neg.cpp      # 0/19 in class
./target/release/c2rs diff   fixtures/cpp/w13_fparam_neg.cpp      # Port=NotImplemented
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 48 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 45 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=4011 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-w18.jsonl           # 377655/2462571
# the alarm, reproduced on a one-function TU (it does NOT reproduce inside
# w13_fabi.cpp, which is the whole point of §22.6):
printf 'float f(int a, float b, float c){ return b*c; }\n' > /tmp/a.cpp
./target/release/c2rs diff /tmp/a.cpp        # before: Mismatch. after: NotImplemented
printf 'float f(float a, float b){ return b; }\n' > /tmp/b.cpp
./target/release/c2rs diff /tmp/b.cpp        # before: Mismatch. after: NotImplemented
# the `27` counterfactual (scratch, reverted): add a `0x27` arm to `parse_expr`
# folding the preceding literal into an `AddrOf`, and sink any body that then
# parses to the end under its own key in `parse_segment_detail` so nothing is
# claimed in class. Re-scan -> 505,122 released, 685 whole-body complete.
```

The sweep prints its own generated case count before it runs; **compare that
number against the one recorded here before believing a green sweep** — a
generator that silently drops cases reports a pass over a smaller corpus
(`GAPS.md` §6). Always difference the scans through **absolute** paths and print
each one's row count and `fn_total` first — §18.8.

## 23. D11, landed — the constructor epilogue, and the FP gate's over-refusal sized

§22.8 named three next rungs and this one measured two of them against each
other before touching either. The FP `fmr` — *"the only rung this document can
name that starts from a known mis-emit"* — is **1,005 functions**, exactly and
only the population §22.6's gate removed. `fn-tail-0xB9` is **28,717**
takeable, one shape, and its lowering is **no instruction at all**. The ranking
inverted on the measurement, the 28.6× candidate shipped, and it converted with
a residue of 3 that is named.

### 23.1 The two candidates, decomposed — MEASURED

Baseline taken **in this worktree** per §18.8, its row count and denominator
printed before any differencing:
`work/dc3-workload/scan-fpbase.jsonl`, 878 rows, `fn_total` 2,462,571, in class
**377,655 (15.34 %)**, mismatch 0, 6 `match` / 7 `capture-fail`. That agrees
with §22.7, which is the only thing agreeing on `fn_total` proves.

**Candidate A — the FP register numbering.** §22.9 left this "not sized", and
it is one scratch build to size: `try_parse_float_leaf`'s
`params.len() != seen.len()` gate — the one §22.6 closed the two mis-emits with
— was made to sink its refusals under their own census key instead of returning
`None`. The gate is raised *after* `eat_return_plumbing` succeeds, so everything
it counts is whole-body complete by construction, the same free-completeness
`:eof` gives.

| functions | key | what it is |
|---:|---|---|
| 1,004 | `fp-leaf-extra-formals-bare` | a body that is one FP LOAD — the `fmr`, or nothing |
| 1 | `fp-leaf-extra-formals-arith` | an FP chain with a formal it does not read |
| **1,005** | | all `calls-0`, spread over 475 TUs, largest single TU 5 |

**The over-refusal is exactly the 1,005 the gate cost and not one function
more** (§22.6 measured census 334,657 → 333,652). That is worth stating as a
result rather than a footnote: the conservative gate did not spill into the
`expr-load-type-8645`/`-8885` rows the way the pessimistic reading assumed. The
98,813 + 82,810 those rows hold block on operand types, member loads and
conversions ahead of any question about which FP register a formal occupies.

The rung is also **buildable** — `.sy` carries what it needs. A formal's record
gives `<tag> <kind> … <size> … <tid>`, and the FP scalars are pinned by capture
(`float f` → `86 05 … 04 … 40`; `double d` → `88 05 … 08 … 41`), against `int`'s
`86 01 … 04 … 74`. `read_record` reads all four fields at `DEPTH_FORMALS` and
keeps only `size`. So the answer to §22.8's "if the IL does not carry it, say
so" is that **it does carry it**; the rung is not blocked, it is small.

**Candidate B — `fn-tail-0xB9`, 29,552 functions, 28,720 `calls-0`.** The
largest call-free row that had been named and never decomposed. What it is,
read off a live capture rather than inferred:

```text
  … 4C 4F 11 53   <body statements>   3A <label> 54 02 29 <label>
                                      B9 <this> <TYPE> 41 <TYPE>
                                      4F 12 47 54 01 54 00
```

A **value expression between the RETURN and the function tail**. Every other
shape in the parser puts its returned value *before* the `3A`, where
`eat_return_head`'s `has_result_type` annotation covers it; a constructor does
not, because its statements each end on a `4B` discard and the thing it returns
— `this`, which MSVC constructors hand back in r3 — is written after the `29`.

That makes the whole row **grammar-complete by construction in the same way
`:eof` is**, and for a sharper reason: the refusal is raised by `eat_fn_tail`,
which every accepted shape reaches *last*. A body filed under `fn-tail-0xNN`
has already parsed under a real shape. This is a second free-completeness
family and it should be read the way `:eof` is — `fn-tail-0x26` at 4,663 is the
other member and is unexamined.

Confirmed by counterfactual (scratch, reverted, nothing claimed in class):
admitting the epilogue takes 29,552 to **29,549 grammar-complete, 99.99 %**,
and — the number that decided the shape of the fix —

| functions | shape that had parsed | frame class |
|---:|---|---|
| **28,717** | `EmptyBody` | `calls-0` |
| 832 | a call shape | `calls-1` |
| 0 | anything else | — |
| 0 | epilogue naming a token that is **not** `this` | — |

### 23.2 The frame axis, and why it is the whole gate — MEASURED

§18's first check, and here it is not a formality. A leaf constructor leaves
`this` in r3 and the epilogue costs nothing; add one call and c2 has to spill
it. Both read off reference objs (`/Ox /GS- /c`):

```text
  struct T { int m; T(); };  T::T() {}
    4e800020                                            blr

  struct B { int b; B(); };  struct D : B { D(); };  D::D() {}
    7d8802a6 9181fff8 fbe1fff0 9421ffa0                 prologue
    7c7f1b78                                            mr r31,r3     <- this spilled
    4bffffed                                            bl B::B
    7fe3fb78                                            mr r3,r31     <- and restored
    38210060 8181fff8 7d8803a6 ebe1fff0 4e800020        epilogue
```

So the 832 are a frame **and** a second register move, and they stay refused.
The recognizer is therefore wired into exactly one arm — the empty-body one —
rather than into the shared return plumbing, which is what would have picked up
all 29,549.

### 23.3 The locality tell, run before committing — MEASURED

§6's cheap check, and the one the `data-addr` rung skipped at a cost of a whole
rung. Eight distinct constructors in one translation unit, varying arity, member
count, member type and file position:

```text
  struct A  { int m;          A(); };   A::A() {}            4e800020  blr
  struct C  { int m, n;       C(); };   C::C() {}            4e800020  blr
  struct D  {                 D(); };   D::D() {}            4e800020  blr
  struct E  { int m;          E(int); };E::E(int a) {}       4e800020  blr
  struct F  { double d;       F(); };   F::F() {}            4e800020  blr
  struct G  { int m;      G(int,int); };G::G(int,int) {}     4e800020  blr
```

One sequence, no exceptions, and the 4-byte inter-function padding the emitter
already produces. The decision is local.

### 23.4 The ranking, and the estimate — stated before the outcome

| | A: the FP `fmr` | B: the constructor epilogue |
|---|---:|---:|
| row | 1,005 | 29,552 |
| whole-body complete | **1,005 (100 %)** | **29,549 (99.99 %)** |
| needs a frame | 0 | 832 |
| **takeable** | **1,005** | **28,717** |
| lowering | one `fmr`, plus `.sy` type-kind plumbing | **no instruction** |
| locality tell | not needed (one register file, no pool) | **run, passes** |

B is 28.6× A and its lowering is *cheaper*, so B, and it is not close. Per
§13.1's rule:

> **Estimate: +28,717, biased LOW.** The counterfactual measured this exact
> population and the implementation gates on exactly it, so the point estimate
> is the measurement. Biased low because the shipped recognizer adds one literal
> requirement the counterfactual did not check — that the `B9` operand type and
> the `41` result type be byte-identical — whose refusing population was **not**
> measured. High only if a TU changes class or a body loses.

### 23.5 What shipped

`grep`ing for every site that implements the rule first, per §22.5:

* `expr::eat_return_head` — `eat_return_plumbing` split into head and
  `eat_fn_tail`, byte for byte, no behaviour change. The two halves already
  existed as named pieces for the generated destructor (§14), which wedges its
  own trailer in the same slot; this rung reuses the split rather than adding a
  second one.
* `shapes::eat_ctor_this_epilogue` — the recognizer. **Two fields required
  literally**, per §6's "a field that never varied is indistinguishable from a
  constant": the token must be the one `parse_this_token` positively bound (not
  "some token", and an `Absent` or undetermined binding refuses), and the loaded
  type must be byte-identical to the result type.
* `body::parse_segment_shape`'s `0x3A` arm — the **only** caller. The shape it
  returns is `BodyShape::EmptyBody` unchanged, because the emitted bytes are
  unchanged; there is no new plan op, no new codegen arm and no new gate in
  `c2-core`. This is the first rung in this document that is a decode widening
  with **zero** emitter change.

The 13 other `eat_return_plumbing` callers are untouched and keep refusing the
epilogue, which is what leaves the 832 framed bodies out.

### 23.6 The outcome — MEASURED

`scan-w19.jsonl` against `scan-fpbase.jsonl`, same list, flags and `--cwd`,
both differenced through absolute paths.

| | baseline | D11 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 377,655 (15.34 %) | **406,372 (16.50 %)** | **+28,717** |
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| TUs changing class | — | **0** | — |
| functions LOST, any TU | — | **0** | — |
| TUs gaining | — | 830 | max 120 |

**The estimate landed exactly**, and the stated low bias had a population of 0 —
no body on this workload spells the two types differently.

The bucket drop is 28,720 against a census gain of 28,717, and **the residue of
3 is named rather than absorbed**:

| | released | landed |
|---|---:|---:|
| `fn-tail-0xB9` | −28,720 | `empty-body` +28,717 |
| | | `module-end-0x4F` +3 |

Those 3 consume the epilogue, reach the function tail and then fail the
*module* end — the same 3 the counterfactual could not make `:eof`
(29,552 − 29,549). Two independent instruments agree on them. All 28,717 are
`calls-0`; no other key moved by one function.

Byte-graded, and the grading is not only census movement:

* `fixtures/cpp/w19_ctor_this.cpp` — **17/17 in class, whole obj byte-exact**
  against real `c2`. Seven member layouts (none, one `int`, two, a `double`, an
  array, two pointers); unused parameters at arity 1, 2 and 4; a pointer and a
  `float` parameter; an 8-byte by-value aggregate ahead of a scalar; a copy
  constructor; two byte-identical bodies for the locality tell; and the control
  group of empty bodies that have **no** epilogue (a free function, a `const`
  member, a `static` member).
* `fixtures/cpp/w19_ctor_this_neg.cpp` — **0/8 refused**, and the file must
  never mismatch: the base-class call, a call to a free function, a member
  sub-object, a store through `this` written three ways, a virtual class, and a
  returned object that is not `this`.
* `scripts/expr_sweep.sh` grew **121 cases** (4,011 → 4,132, `mismatches=0`):
  the 9 × 10 cross product of member layout against parameter list, the copy
  constructor and aggregate/reference/pointer parameters, every argument slot
  r4..r10 filled and unread, a nested class and a namespaced one, the
  destructor and empty-member controls, and the 11 refusing neighbours.
  **111 of the new cases grade `Match`**, so the sweep is comparing emitted
  bytes here and not only counting refusals.
* Four mode lanes, mismatch 0 in each: `/Ox` 48 → **49**, `/O1` 45 → **46**,
  `/O2` 45 → **46**, `/Ox /Gy` 45 → **46**. The `+1` is `w19_ctor_this.cpp` in
  every lane; no baseline dropped.

### 23.7 The corrected ranking

Over the 2,056,199 still-blocked functions:

| # | functions | % blocked | calls-0 | calls-1 | calls-2plus | key |
|---:|---:|---:|---:|---:|---:|---|
| 1 | 504,438 | 24.5 % | 313,666 | 74,659 | 116,113 | `expr-op-0x27` |
| 2 | 225,341 | 11.0 % | 10,287 | 71,380 | 143,674 | `expr-convert` |
| 3 | 141,800 | 6.9 % | 15 | 57,894 | 83,891 | `expr-intrinsic-this-adjust` |
| 4 | 134,431 | 6.5 % | 0 | 90,274 | 44,157 | `expr-op-0x99` |
| 5 | 118,330 | 5.8 % | 27,138 | 56,186 | 35,006 | `expr-intrinsic-base-member-addr` |
| 6 | 98,813 | 4.8 % | 10,702 | 84,215 | 3,896 | `expr-load-type-8645` (float) |
| 7 | 82,810 | 4.0 % | 2 | 82,806 | 2 | `expr-load-type-8885` (double) |
| 8 | 48,102 | 2.3 % | 1,622 | 4,674 | 41,806 | `body-0x29` |

Row 1 still carries §22.2's measurement — 0.14 % whole-body complete — and
should still not be scheduled against. The rungs this rung can name, with their
measured populations:

1. **`fn-tail-0x26`, 4,663 functions** — the *other* member of the
   free-completeness family this rung discovered. Undecomposed, and decomposing
   it costs one scratch build now that the pattern is known.
2. **The FP `fmr`, 1,005 functions**, all `calls-0`, all whole-body complete,
   `.sy` proven to carry the type kinds it needs (§23.1). Small, exactly sized,
   and it is still the only rung that starts from a closed mis-emit.
3. **`expr-out-of-class-bare-nonformal`, 2,881 functions**, all `calls-0` and
   all `:eof`. Still not one lowering — a global read is a pool reference and
   has §6's non-local hazard, a `.sy` local does not — and still needs
   decomposing before it is worth ranking.
4. **The 832 framed constructors.** They are the general-frame rung's smallest,
   cleanest test case: one call, one spill of `this`, one restore, and the
   whole body already parses.

### 23.8 What is NOT established, labelled

* **The counterfactual is a grammar measurement, not a differential.** No TU
  flipped under either scratch build, so no lane and no sweep graded them; their
  `mismatch 0` is a statement about TU-level acceptance only. Both were reverted
  and every number above re-taken against the committed tree.
* **`this` is proven to survive to the epilogue only for the shapes admitted.**
  The argument is that an `EmptyBody` emits nothing and therefore cannot move
  r3, and the evidence is the fixture and the 111 matching sweep cases. It is
  not a proof about every leaf shape — which is why the recognizer has one
  caller rather than living in the shared plumbing, and why widening it to
  another shape needs its own grading.
* **The 3-function residue is attributed from two agreeing counts, not traced
  function by function.** The counterfactual's 29,552 − 29,549 and the scan's
  28,720 − 28,717 are both 3, and both land on the module end, but no
  per-function identity check was run.
* **The `.sy` FP type kinds are pinned by one capture each.** `float` → tid
  `0x40`, `double` → tid `0x41`, against `int`'s `0x74`, from a five-function
  probe. `long double` and every vector type are unwitnessed, so a rung built on
  this must require the tuple literally and fail closed — §6's rule, stated here
  because the next rung is the one that will read it.
* **The census is `/O1` and the fixture grading is `/Ox` plus three more
  lanes.** The epilogue emits nothing, so the mode's one differing rule
  (accumulator allocation) has nothing to reach; that is an argument, and the
  four lanes are the evidence.
* **`mismatch 0` on the workload is still a TU-level statement.** 865 of the 878
  TUs are `vocab-gap` and never reach the port. This rung's byte grading is the
  two fixtures (25 functions), the 4,132-case sweep and the four lanes.
* **`fn-tail-0x26`'s 4,663 are asserted to be the same *family*, not the same
  *shape*.** They share the free-completeness property and nothing else was
  measured about them.

### 23.9 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp        # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                      # 119 pass 0 fail 0 error
./target/release/c2rs census fixtures/cpp/w19_ctor_this.cpp       # 17/17 in class
./target/release/c2rs diff   fixtures/cpp/w19_ctor_this.cpp       # Port=Match
./target/release/c2rs census fixtures/cpp/w19_ctor_this_neg.cpp   # 0/8 in class
./target/release/c2rs diff   fixtures/cpp/w19_ctor_this_neg.cpp   # Port=NotImplemented
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                         # 49 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                         # 46 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                            # checked=4132 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-w19.jsonl           # 406372/2462571
# the two counterfactuals (scratch, reverted):
#  A: in `try_parse_float_leaf`, replace the `params.len() != seen.len()` refusal
#     with its own census key.               -> 1,005, of which 1,004 are one LOAD
#  B: in `eat_return_plumbing`, consume `B9 <tok> <TYPE> 41 <TYPE>` before the
#     tail and sink the result under its own key so nothing is claimed in class,
#     tagged by the BodyShape that parsed.   -> 28,717 EmptyBody + 832 call + 0 other
# the lowering, read off the reference obj rather than inferred:
printf 'struct T { int m; T(); };\nT::T() {}\n' > /tmp/a.cpp
./target/release/c2rs diff /tmp/a.cpp        # Port=Match — the epilogue costs nothing
printf 'struct B { int b; B(); };\nstruct D : B { D(); };\nD::D() {}\n' > /tmp/b.cpp
./target/release/c2rs diff /tmp/b.cpp        # Port=NotImplemented — the call spills `this`
```

The sweep prints its own generated case count before it runs; **compare that
number against the one recorded here before believing a green sweep** (§6).
Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first — §18.8.

## 24. D12, landed — the second-largest row, decomposed to a gate, and the free conversion inside it

§23.7 named three next rungs. This one measured all three before touching any,
and the ranking it produced is not the one the row sizes suggested:

* **`fn-tail-0x26`, 4,663 — refuted at zero build cost.** Every one of the 4,663
  is `calls-2plus` (§24.1). §18's frame axis settles it outright: two calls
  always need a frame, so the takeable population is **0**. The free-completeness
  *family* was real; this member of it is not takeable until the general-frame
  phase.
* **`expr-convert`, 225,341 — the second-largest row on the board, never
  decomposed, and it is a gate.** Admitting the `2C` releases the whole row and
  leaves **5,562 grammar-complete, 2.47 %**. That is the third time this document
  has been handed a top-of-the-list row and measured it into single-digit
  percentages (§21's 1.4 %, §22's 0.14 %, now 2.47 %), and the pattern is now
  explicit enough to state as a rule: **a row that names a token near the leaves
  of the grammar is a gate; the rung is whatever is complete behind it.**
* **The FP `fmr`, 1,005** — unchanged from §23.1, still exactly sized, still the
  only rung starting from a closed mis-emit.

The 5,562 shipped: 5.5× the FP rung, and its lowering is **no instruction at
all**.

### 24.1 The three candidates, decomposed — MEASURED

Baseline taken **in this worktree** per §18.8, its row count and denominator
printed before any differencing: `work/dc3-workload/scan-base-convert.jsonl`,
878 rows, `fn_total` 2,462,571, in class **406,372 (16.50 %)**, mismatch 0, 6
`match` / 7 `capture-fail`, 570 keys. That agrees with §23.6, which is the only
thing agreeing on `fn_total` proves.

| candidate | functions | calls-0 | calls-1 | calls-2plus | takeable |
|---|---:|---:|---:|---:|---:|
| `expr-convert` | 225,341 | 10,287 | 71,380 | 143,674 | **5,562** |
| `fn-tail-0x26` | 4,663 | **0** | **0** | **4,663** | **0** |
| FP `fmr` (§23.1) | 1,005 | 1,005 | 0 | 0 | 1,005 |

**`fn-tail-0x26` cost one query.** §23.1 called it "the same *family*, not the
same *shape* — nothing else about them was measured", and the first thing
measured about them settles the whole candidate. `fn-tail-0xB9` split 28,720
`calls-0`/832 `calls-1`; `fn-tail-0x26` is 4,663 `calls-2plus` and nothing else.
A free-completeness family says a body has finished parsing; it says nothing
about whether the body needs a frame, and those are independent axes.

**The `expr-convert` counterfactual** (scratch, reverted, nothing claimed in
class): `parse_expr` was given a `2C <TYPE> <varint>` arm that records whether
the target was the operand's own value class, with a thread-local sunk in
`parse_segment_detail` so that a body which then parses to the end is counted and
**never** accepted. In-class stayed at 406,372 in the counterfactual build, which
is the check that no speculative parse set the flag spuriously.

| functions | key | frame class |
|---:|---|---|
| 2,480 | grammar-complete, every conversion **int4→int4** | 2,478 `calls-0`, 2 `calls-1` |
| 3,082 | grammar-complete, every conversion **ptr4→ptr4** | 3,082 `calls-1` |
| **0** | grammar-complete with any **cross-class** conversion | — |
| **5,562** | **total, 2.47 % of the row** | 2,478 / 3,084 / **0** |

Two things fall out of that table and neither was predictable from the row size.
**Not one completing body mixes the classes** — so the conservative rule and the
lax one have identical yield here, which is what makes the conservatism free
rather than merely defensible. And the ptr4 half is **entirely `calls-1`**: it is
the argument of a tail call, cv-stripped or widened to `void *` on the way in,
which is a shape the port has emitted since the MVP and the workload could not
reach.

Where the other 97.5 % stops, over the whole 878-TU scan:

| functions | key | what it is |
|---:|---|---|
| +145,851 | `expr-op-0x99` | the by-value temporary bind |
| +24,400 | `expr-op-0x64` | |
| +14,934 | `expr-load-type-8212` | a 2-byte operand |
| +11,594 | `call-multiarg-postop:eof` | |
| +6,407 | `expr-ptr-arith:eof` | §21's guard, reached through a conversion |
| +4,075 | `expr-cmp-eq` | |
| +3,401 | `expr-op-0x41` | |

### 24.2 The locality tell, run before committing — MEASURED

§6's cheap check, skipped once at the cost of a whole rung. Four byte-identical
bodies in one TU, at varied file positions and with different neighbours between
them, plus the pair that separates this class from the register move:

```text
  unsigned d1(int a,int b) { return (unsigned)(a+b); }  7c632214 4e800020
  int      pad1(int a)     { return a*3; }              546b083c 7c635a14 4e800020
  unsigned d2(int a,int b) { return (unsigned)(a+b); }  7c632214 4e800020   identical
  unsigned d3(int a,int b) { return (unsigned)(a+b); }  7c632214 4e800020   identical
  int      pad2(int a,int b){ return a-b; }             7c641850 4e800020
  unsigned d4(int a,int b) { return (unsigned)(a+b); }  7c632214 4e800020   identical
  unsigned s2(int a,int b) { return (unsigned)b; }      7c832378 4e800020   mr r3,r4
  unsigned r1(int a)       { return (unsigned)(unsigned)a; }       4e800020
  unsigned k1()            { return (unsigned)7; }      38600007 4e800020
```

`add r3,r3,r4` in all four, wherever they sit and whatever surrounds them, and
`(unsigned)(a+b)` is byte-for-byte `a+b`. The decision is local, and the
conversion is worth zero instructions.

The conversions that are *not* free were read off the same kind of probe and are
the negative fixture's whole content:

```text
  char      f(int a) { return (char)a; }      7c630774  extsb  r3,r3
  short     f(int a) { return (short)a; }     7c630734  extsh  r3,r3
  unsigned char  f(int a)                     5463063e  rlwinm r3,r3,0,24,31
  unsigned short f(int a)                     5463043e  rlwinm r3,r3,0,16,31
  long long f(int a) { return (long long)a; } 7c6307b4  extsw  r3,r3
  float     f(int a) { return (float)a; }     a five-instruction stack round trip
  int       f(S* p)  { return (int)p; }       4e800020  blr      <- FREE, refused
  S*        f(int a) { return (S*)a; }        4e800020  blr      <- FREE, refused
```

The last two are the honest half of the boundary: the cross-class reinterpret
**is** free in both directions and is refused anyway, because a reinterpret has
never been graded across the widths, cv-spellings and argument positions this
parser reaches. §24.6 gives what that conservatism costs as a number.

### 24.3 The ranking, and the estimate — stated before the outcome

| | `expr-convert` | `fn-tail-0x26` | FP `fmr` |
|---|---:|---:|---:|
| row | 225,341 | 4,663 | 1,005 |
| whole-body complete | 5,562 (2.47 %) | 4,663 (100 %) | 1,005 (100 %) |
| **needs a frame** | 143,674 | **4,663 — all of it** | 0 |
| **takeable** | **5,562** | **0** | **1,005** |
| lowering | **no instruction**, and the predicate already exists byte-graded | — | one `fmr` + `.sy` type-kind plumbing |
| locality tell | **run, passes** | not run (nothing to take) | not needed |

5.5× the FP rung at a fraction of its work — the FP rung needs a new emitter arm
and a new `.sy` field on the axis where §22.6's two live mis-emits lived, while
this one needs a decode arm and no emitter change at all. Per §13.1's rule:

> **Estimate: +5,562 exactly, biased LOW.** The counterfactual measured this
> population and the implementation gates on exactly it, so the point estimate is
> the measurement rather than an interval. Biased low by the §13.1/§22.5 hazard —
> a second parser site implementing the same rule — which is **sized rather than
> waved at** this time: the only other keys naming a `2C` outside `parse_expr`
> are `call-postop-0x2C` and `call-bound-store-0x2C`, **4 functions between them
> on the whole workload**, and this rung does not widen them. So the low bias has
> a ceiling of 4. High only if a TU changes class or a body loses.

Outcome **+5,562**, and the ceiling-of-4 low bias had a population of 0. This is
the second consecutive rung whose point estimate landed exactly, and the first
whose stated bias direction was given a *measured* bound instead of a name.

### 24.4 What shipped

`grep`ing for every site that implements the rule first, per §22.5.

* **`readers::ValueClass`, `value_class` and `eat_value_type` moved** from
  `body/shapes.rs` to `readers.rs` and are `pub(crate)`. They were already the
  project's answer to "are these two 4-byte types the same *kind* of value", they
  are what `finish_indirect_load` and `try_parse_ptr_identity_leaf` gate their
  own `2C` on, and the alternative was a second copy — the "one fact, two
  locators" mistake §21.3 moved `is_ptr4_kind` to avoid. Nothing about them
  changed.
* **`expr::parse_expr` grew a `0x2C` arm** and one variable. The arm consumes
  `2C <TYPE> 00` iff the target is the value's own class, pushes **no** `IlOp`,
  and refuses otherwise under a key that names the target's `<tag><kind>` — never
  its per-TU id (§20).
* **No codegen, no new `BodyShape`, no new gate in `c2-core`.** The second rung
  in this document that is a pure decode widening with zero emitter change, after
  §23's.

The variable is the one thing worth stating precisely, because "the class of the
last operand" is not obviously "the class of the value on top of the stack":

> Every accepted conversion preserves the class; every accepted operator over a
> *pointer* is refused outright by §21's guard; arithmetic over int4 values
> yields an int4 value. So an **accepted** sub-expression has exactly one class
> throughout, and the two readings coincide. Where they can differ —
> `(void *)(s + 1)`, whose last operand is the literal — the guard refuses the
> body anyway, so only the census key moves. `w20_convert_neg.cpp` carries that
> case with the attribution written down.

The trailing varint is required to be literally `0`, per §6's rule about a field
that never varied, with `expr-convert-tail` counting the exceptions. There are
none on this workload.

### 24.5 The outcome — MEASURED

`scan-w20.jsonl` against `scan-base-convert.jsonl`, same list, flags and `--cwd`,
both differenced through absolute paths.

| | baseline | D12 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 406,372 (16.50 %) | **411,934 (16.73 %)** | **+5,562** |

> Both columns are pre-D13 and therefore both over-claim by ~9,230 (the
> census/gate repair, `ROADMAP.md` §6c). The **delta** is unaffected — the gates
> D13 moved are orthogonal to this rung — but the absolute numerator is not;
> the current one is 402,704.
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| distinct keys | 570 | 581 | +11 |
| TUs changing class | — | **0** | — |
| functions LOST, any TU | — | **0** | — |
| TUs gaining | — | 826 | max 17 |

826 of 878 TUs gain and the largest single TU gains 17, so this is a property of
the corpus rather than of one file — the same spread test §22.1 applied to the
register move.

Byte-graded, and the grading is not only census movement:

* `fixtures/cpp/w20_convert.cpp` — **44/44 in class, whole obj byte-exact**
  against real `c2`. The 4×4 int spelling matrix (`int`/`unsigned`/`long`/
  `unsigned long` as source and target) and both implicit witnesses; the
  conversion at every position of a two- and three-operand chain over `+`, `-`
  and `*`; conversions over literals; nested conversions two and three deep; the
  converted formal at argument slots 2, 3 and 5 (where the D10 register move sits
  underneath); the conversion inside a call-argument region at arities 1 and 2;
  the pointer half as a tail-call argument, as an identity and through a `const`
  receiver; member functions where `this` is a const pointer in r3; and two
  byte-identical bodies for the locality tell.
* `fixtures/cpp/w20_convert_neg.cpp` — **0/19 in class**, and the file must never
  mismatch. Ten conversions that emit an instruction (`extsb`, `extsh`, two
  `rlwinm` masks, `extsw`, the float round trip) as a body, over a chain and over
  a call result; the cross-class reinterpret in both directions, in a body and in
  an argument; a conversion feeding pointer arithmetic and a pointer difference;
  a `const int` *parameter*, whose operand type blocks ahead of the conversion and
  which is a different key on purpose; and the float/double operand side.
* `scripts/expr_sweep.sh` grew **211 cases** (4,132 → 4,343, `mismatches=0`) over
  exactly those axes: the 16-cell spelling matrix, 45 two-operand chains (3
  operators × 5 operand pairs × 3 conversion positions), 45 three-operand chains
  (9 operator pairs × 5 conversion slots), the nesting ladder, every argument slot
  at arities 1–8, 9 call-argument forms, 6 pointee widths × 4 target pointer
  spellings, a pointer conversion at each slot of a 1–3-argument tail call, three
  member forms, and 48 refusing neighbours. **134 of the 211 new cases grade
  `Match`** and 77 `NotImplemented`, so the sweep is comparing emitted bytes here
  and not only counting refusals.
* Four mode lanes, mismatch 0 in each: `/Ox` 49 → **50**, `/O1` 46 → **47**,
  `/O2` 46 → **47**, `/Ox /Gy` 46 → **47**. The `+1` is `w20_convert.cpp` in every
  lane; no baseline dropped. `c2rs bench` 119 → **121 pass, 0 fail, 0 error**.

### 24.6 Where the un-gained population went — named, not absorbed

The bucket drop is **225,341** against a census gain of **5,562**, so the answer
to "does the drop equal the gain" is **no, by a factor of 40** — this rung
cleared a *first* blocker for a quarter of a million functions and finished the
body for 2.47 % of them. §24.1 has the deeper destinations. What is new here is
the residue this rung's own refusals created, which is the part a rung can hide:

| functions | key | calls-0 | what it is |
|---:|---|---:|---|
| 4,972 | `expr-convert-no-value-0x2C` | 3,419 | a `2C` at the **head** of a `parse_expr` region |
| 3,247 | `expr-convert-target-8642` | 0 | target is a width-4 **unsigned with a per-TU id** (an enum or typedef) |
| 1,625 | `expr-convert-target-A641` | 0 | target is a **`const int`** with a per-TU id |
| 812 | `expr-convert-target-A642` | 0 | target is a **`const unsigned`** with a per-TU id |
| 809 | `expr-convert-target-8422` | 809 | target is `unsigned short` — a real `rlwinm` |
| 14 | `expr-convert-target-*` (rest) | 11 | the cross-class reinterpret and stragglers |
| **11,479** | | | |

Two of those rows are results rather than bookkeeping.

**`expr-convert-no-value` is the data-symbol address, and it is 4,972 functions.**
The parse enters `parse_expr` on a `2C` with no operand behind it, which one
capture shows to be
`4C 4F 11 53 · 26 <sym> · 2C <ptr> 00 · 41 <ptr>` — a `26` symbol push,
cv-stripped and returned. The mechanism is structural rather than inferred: the
assignment-body parser consumes a leading `26 <tok>` as a destination and hands
the rest to `parse_expr`, so a `26` in a *value* position is the only way this
key can fire. It was inside `expr-convert` before and is now its own number.

**The int-spelling whitelist over-refuses by 5,684, all `calls-1`.**
`eat_value_type(Int4)` is `eat_int_like`, an exact four-triple whitelist, so a
conversion whose target is a width-4 integer carrying a *per-TU type id* — an
enum, a typedef, a `const int` — refuses even though `is_int4_type` would admit
it on the nibbles and the emitted instruction is the same nothing. That is
`8642 + A641 + A642 = 5,684 functions`, and it is **larger than this rung's own
gain**. It is not taken here because `eat_int_like` is shared with the
already-byte-graded getter and identity leaves and with the `41` result
annotation, so widening it is a change to a locator three shipped shapes depend
on and needs its own grading — which is precisely the ranked next rung, now with
a number instead of a suspicion.

### 24.7 A pre-existing census/gate disagreement, found and NOT fixed here

Probing the fixture turned up a defect that predates this rung and is orthogonal
to it, recorded because §22.5 found the same class of bug and because an
unrecorded one is worse than an open one:

```text
  int f(int a,int b,int c){ return a + b*c; }   census: 1/1 in class   Port=NotImplemented
  int f(int a,int b,int c){ return a - b*c; }   census: 1/1 in class   Port=NotImplemented
  int f(int a,int b,int c){ return a*b + c; }   census: 1/1 in class   Port=Match
```

A `*` after the first operator is accepted by `parse_segment` and refused by
`codegen`, so the census **over-claims**. Verified on the committed tree with the
D12 changes stashed, so it is not this rung's. It is the §22.5 defect with the
producers swapped — a gate in codegen that the census does not consult — and its
direction is the safe one (no bytes are emitted), but it means the headline
numerator is an upper bound by an unmeasured amount. Sizing it is one scratch
build and is the honest next instrument task.

### 24.8 The corrected ranking

Over the 2,050,637 still-blocked functions:

| # | functions | % blocked | calls-0 | calls-1 | calls-2plus | key |
|---:|---:|---:|---:|---:|---:|---|
| 1 | 504,445 | 24.6 % | 313,667 | 74,661 | 116,117 | `expr-op-0x27` |
| 2 | 280,282 | 13.7 % | **0** | 114,059 | 166,223 | `expr-op-0x99` |
| 3 | 141,800 | 6.9 % | 15 | 57,894 | 83,891 | `expr-intrinsic-this-adjust` |
| 4 | 118,331 | 5.8 % | 27,139 | 56,186 | 35,006 | `expr-intrinsic-base-member-addr` |
| 5 | 98,813 | 4.8 % | 10,702 | 84,215 | 3,896 | `expr-load-type-8645` (float) |
| 6 | 82,810 | 4.0 % | 2 | 82,806 | 2 | `expr-load-type-8885` (double) |
| 7 | 48,102 | 2.3 % | 1,622 | 4,674 | 41,806 | `body-0x29` |
| 8 | 39,366 | 1.9 % | 0 | 2 | 39,364 | `expr-call-in-expr-op-0x9B` |

`expr-convert` has left the table entirely and `expr-op-0x99` has taken its
population and its place. Row 2 has **zero `calls-0` functions**, which is the
same refutation `fn-tail-0x26` got in §24.1 and should be applied before anyone
ranks it: nothing in it is takeable without a frame.

The rungs this rung can name, with their measured populations:

1. **The int-spelling whitelist, 5,684 functions**, all `calls-1`, all of them
   conversions that emit nothing and refuse on a type-table id (§24.6). Larger
   than this rung's gain; it changes a locator three graded shapes share, so it
   needs their fixtures re-run rather than only its own.
2. **The FP `fmr`, 1,005 functions**, all `calls-0`, all whole-body complete,
   `.sy` proven to carry the type kinds it needs (§23.1). Unchanged by this rung
   and still the only one that starts from a closed mis-emit. The `.sy` FP kinds
   are one capture each and `long double` and the vectors are unwitnessed, so it
   must require the tuple literally and fail closed.
3. **The cross-class reinterpret**, measured at 14 functions on this workload
   (§24.6) — free in both directions, and not worth its own grading at that size.
   Recorded so the next reader does not re-derive it.
4. **`expr-convert-no-value`, 4,972 functions**, 3,419 `calls-0` — the data-symbol
   address in a value position, which is §6's `data-addr` family and carries that
   family's non-local constant-pool hazard. Decompose before ranking.
5. **The 832 framed constructors** (§23.7) and the general frame, which rows 1–4
   of the table above are now overwhelmingly waiting on.

### 24.9 What is NOT established, labelled

* **The counterfactual is a grammar measurement, not a differential.** No TU
  flipped under it, so no lane and no sweep graded that build; its `mismatch 0`
  is a statement about TU-level acceptance only. It was reverted and every number
  above re-taken against the committed tree.
* **`mismatch 0` on the workload is still a TU-level statement.** 865 of the 878
  TUs are `vocab-gap` and never reach the port. This rung's byte grading is the
  two fixtures (63 functions), the 4,343-case sweep and the four lanes.
* **"The conversion is free" rests on the same evidence the getter and identity
  leaves rest on, plus this rung's probes and fixture.** int4→int4 and ptr4→ptr4
  at width 4 emit nothing in every witness taken; no capture in this project has
  produced a counter-example. It is not a proof about every spelling the type
  table can hold — which is exactly why the int side keeps its four-triple
  whitelist and the 5,684 it refuses is written down as a number in §24.6 rather
  than quietly admitted.
* **`expr-convert-no-value`'s content is one capture plus a structural
  argument.** The witness is a `26` symbol push behind the conversion and the
  argument is that the assignment-body parser is the only path that can enter
  `parse_expr` at a `2C`; no per-function survey of the 4,972 was run.
* **The 5,684 int-spelling over-refusal is attributed from the key names**
  (`8642` / `A641` / `A642` are width-4 integer `<tag><kind>` pairs that
  `is_int4_type` admits and `eat_int_like` does not). No build was made that
  widens the whitelist, so 5,684 is a *ceiling* on that rung's decode gain, not a
  whole-body-completeness measurement.
* **The census is `/O1` and the fixture grading is `/Ox` plus three more lanes.**
  The conversion emits nothing, so the mode's differing rule (accumulator
  allocation) has nothing extra to reach; that is an argument, and the four lanes
  are the evidence.
* **§24.7's disagreement is characterized on three probes, not sized.** How many
  workload functions the census over-claims by is unmeasured.
  **SIZED 2026-07-30 (D13, `ROADMAP.md` §6c): 9,230 functions, 2.24 % of the
  numerator — and the shape §24.7 names contributed ZERO of it.** The workload
  has 62,813 straight-line functions and not one whose operand stack passes depth
  2. The whole 9,230 was two causes this section did not look for: 9,028
  generated destructors whose callee token has no `.gl` symbol, and 202 functions
  carrying an optimization word the port does not emit under. All four gates
  (including this one) have been moved into the parser, each with its own census
  key; the corrected census is **402,704 / 2,462,571 (16.35 %)** and the residual
  disagreement is 0 on the workload. Read this bullet as the standing warning it
  turned into: **a characterized defect is a witness, not a measurement**, and the
  ratio between the two here was not a rounding error but the entire quantity.

### 24.10 Reproduction

```sh
cargo build --release
./target/release/c2rs census fixtures/cpp/w5_chain.cpp          # 4/4 in class
cargo test --workspace
C2RS_JOBS=16 ./target/release/c2rs bench                        # 121 pass 0 fail 0 error
./target/release/c2rs census fixtures/cpp/w20_convert.cpp        # 44/44 in class
./target/release/c2rs diff   fixtures/cpp/w20_convert.cpp        # Port=Match
./target/release/c2rs census fixtures/cpp/w20_convert_neg.cpp    # 0/19 in class
./target/release/c2rs diff   fixtures/cpp/w20_convert_neg.cpp    # Port=NotImplemented
C2RS_JOBS=16 scripts/mode_lane.sh /Ox                           # 50 match, 0 mismatch
C2RS_JOBS=16 scripts/mode_lane.sh /O1                           # 47 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=16 scripts/expr_sweep.sh                              # checked=4343 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 16 \
  --jsonl work/dc3-workload/scan-w20.jsonl           # 411934/2462571, 581 keys
# (that numerator was corrected to 402704/2462571, 584 keys, by D13 — the
#  census/gate repair in ROADMAP.md 6c. Quote the newer one.)
# the `fn-tail-0x26` refutation — no build at all, just the frame axis:
python3 -c "import json,collections;f=collections.Counter();\
[f.update(json.loads(l)['fn_frames']) for l in open('work/dc3-workload/scan-base-convert.jsonl')];\
print({k:v for k,v in f.items() if k.endswith('|fn-tail-0x26')})"   # all calls-2plus
# the `expr-convert` counterfactual (scratch, reverted): give `parse_expr` a
# `2C <TYPE> <varint>` arm that tracks the operand's value class and records
# whether the target matched it, and sink any body that then parses to the end
# under its own key in `parse_segment_detail` so nothing is claimed in class.
# Re-scan -> 225,341 released, 5,562 complete (2,480 int4 + 3,082 ptr4, 0 mixed).
# the lowering, read off the reference obj rather than inferred:
printf 'unsigned f(int a,int b){ return (unsigned)(a+b); }\n' > /tmp/a.cpp
./target/release/c2rs compile /tmp/a.cpp --keep-obj /tmp/a.obj   # add r3,r3,r4 ; blr
printf 'char f(int a){ return (char)a; }\n' > /tmp/b.cpp
./target/release/c2rs compile /tmp/b.cpp --keep-obj /tmp/b.obj   # extsb r3,r3 ; blr
# §24.7, on the committed tree with this rung stashed:
printf 'int f(int a,int b,int c){ return a + b*c; }\n' > /tmp/c.cpp
./target/release/c2rs census /tmp/c.cpp   # 1/1 in class
./target/release/c2rs diff   /tmp/c.cpp   # Port=NotImplemented  <- the disagreement
```

The sweep prints its own generated case count before it runs; **compare that
number against the one recorded here before believing a green sweep** (§6).
Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first — §18.8.

## 25. W23, landed — the store leaf, and the row this document ranked last

Written up in full in **`docs/IL_STORE_LEAF.md`**; recorded here because it
settles three of this document's own open items and corrects one of its
measurements.

* **§19.1's "whole-body store — 10/863, 1.2 %" was right about the proportion and
  it is now taken.** The store leaf is **+23,645** functions (census
  418,628 → 442,273, 17.00 % → 17.96 %, mismatch 0, disagreement 0), of which
  **740** come from the 2117 key §19.7 (5) sized at "≈370". The rest is the same
  production through the *plain* designator, which §19.3's lesson predicted and
  which is **29x** the intrinsic half here against D7's 5.0x.
* **§24.8's rows 5 and 6 are refuted by measurement.** `expr-load-type-8645`
  (98,813) completes **1,004** bodies under a full type widening — and that
  population is §23.1's FP `fmr`, not a new rung. `expr-load-type-8885` (82,810)
  completes **0**: its `calls-1` mass is a `2C`-converted FP value in a
  *call-argument* region (`call-end-0x88` 44,050 + `expr-convert-target-8885`
  38,756 when the type is admitted), so it converges with the **FP
  argument-register** item, not with the frame.
* **§24.8's row 4 (`expr-op-0x27`, via `GAPS.md` §6) was measured too narrowly.**
  The 685-whole-bodies figure came from a counterfactual that admitted the
  *token* inside `parse_expr`; half the row is a *statement* that no widening of
  `parse_expr` can finish. This rung took 22,095 out of it. See `GAPS.md` §6's
  corrected box.
* **That rung was then taken in the same session (W24).** `expr-load-type-8212`
  + `expr-lit-type-8212` — 52,650 released, **+22,311** in class, no emitter
  change, mismatch 0, disagreement 0. Census after both rungs: **464,584 /
  2,462,571 = 18.87 %**, from 418,628. `IL_STORE_LEAF.md` §9.
