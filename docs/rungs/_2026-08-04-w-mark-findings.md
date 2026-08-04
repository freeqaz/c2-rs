# w-mark — the missing ROOT channel is the DATA INITIALIZER, and the sole judge confirms it 15/15. The unfiltered reading of it is not a model.

    Lane:      w-mark, 2026-08-04, worktree `wt-w-mark` off master `451f1bd`
    Prereg:    work/w-mark/PREREG.md (= rungs/_2026-08-04-w-mark-prereg.md),
               committed at `1f2ec78` BEFORE any measurement against truth.
               Scored in §6.
    Ships:     NOTHING under `crates/`. No fixture, no codegen, no widening,
               no DISCLOSURE.md row (nothing is adopted).
    Status:    FINDINGS. TU match is 8 at both ends.

**One-line answer:** ***The channel is FOUND and it is causal — but decline
clauses 2 and 3 both fired, so the model half of this page is published as an
UPPER BOUND, not as a model.*** Three of the six external `Mark` call sites
(`10b98be8`, `10b98c08`, `10b98c7f`) are one channel: **c2 walks every data
symbol's initializer node list and marks every function whose address appears in
it**, before the compile loop, from the **`in` sub-stream no lane has ever
captured**. Retargeting a single `02` node's token and replaying through the real
`c2.dll` makes exactly the new function's COMDAT appear — **15/15 sufficiency,
12/15 necessity**, against a registered ≥3/5 each. Adding that channel's names as
roots moves w-roots' root-floor coverage from **0.18796 to 0.86926** and recall
from **0.74307 to 0.95991**, covering **84.4 %** of the incumbent's residual.
**But it takes precision from 1.00000 to 0.27289 and F1 down 42.763 pp**, because
the reading is *unfiltered* and `10b98e26` has three skips I pre-registered as
unmodelled. **w-refs' headline is confirmed and sharpened: the residual is a root
problem, and the roots are in a stream nobody has read.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-mark`, based on master **`451f1bd`** |
| c2-rs HEAD at the prereg | **`1f2ec78`**, clean — **no `crates/` change exists in this lane** |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER** | **`940d07dcb096…`** — **it did not move** (`work/w-mark/prov_{before,after}.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`**, checked at lane start, **not stale**. `.claude/worktrees/wibo` is a **symlink** to the sibling checkout — verified, because a directory-existence probe there silently picks it |
| c2.dll read | `compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c0…a66258`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| `gl` / `ex` / truth | **reused from w-emit unchanged**, 876 IL / 850 truth, same dc3 rev |
| **`in`** | **captured by this lane**, 876 of 878 TUs, same `cl /Bd /d2nop` recipe (c2 aborts `C1007 … in 'p2'`, so **no c2 output is produced** — quarantine-safe). The two misses are `FxSendPitchShift360.cpp` and `FxSendSynapse360.cpp`, w-emit's two by name |
| the join | the re-capture reproduces w-emit's cached `gl` **byte-identically** (`cmp`, run before the prereg), so the token spellings agree across the two caches |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-mark/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine is intact and w-emitpred's one-shot Part-1 gate is
UNSPENT** — §7. No held-out TU was read, captured or mutated; the three mutation
TUs were checked against `heldout.txt` by name.

---

## 1. The deliverable: what marks a symbol at each `Mark` call site

### 1a. There are SIX external call sites, not seven — and this corrects w-refs

`work/w-mark/xrefs.py 0x10b276e4` scans **every** `E8`/`E9 rel32` in `.text` and
**every** little-endian occurrence of the absolute address in the whole image. It
finds **seven `call` sites and zero absolute references**, so `Mark` is reached
by direct call only and the enumeration is complete for this image.

**One of the seven is `Mark`'s own recursion.** `10b27731` sits inside
`10b276e4 .. 10b2773e`, and it is the `Mark(tgt, edi)` line of w-refs' own
pseudocode. `_2026-08-04-w-refs-findings.md` §1b lists it among "seven other
`Mark` call sites"; **it is not other, and the external count is 6, in 4
enclosing functions.**

### 1b. Two gates w-refs' pseudocode omits, one of which is a mode switch

    Mark(ecx = sym, edx = force):                       ; 10b276e4
      if (sym[0x4c] & 0x20) return;                     ; 10b276ea
      if (ds:0x10c462c4 && !force) return;              ; 10b276ee
      sym[0x4c] |= 0x20;                                ; 10b276fb   THE SEED BIT
      if (!ds:0x10c3cf68) return;                       ; 10b27701   <-- OMITTED
      ecx = sym[0x80]; if (!ecx) return;                ; 10b2770a
      for (node = ecx[0xc]; node; node = node[0]):      ; 10b27715
          tgt = node[4][4]
          if (tgt[0x37] & 0x400) continue;              ; 10b27720
          if (tgt[0x4c] & 0x20) continue;               ; 10b27729
          Mark(tgt, force)                              ; 10b27731

* **`ds:0x10c3cf68` is the head of the global list of reference-list objects**
  (`10b27dbd: mov [esi+0x80],eax ; mov ecx,ds:0x10c3cf68 ; mov [eax],ecx ; mov
  ds:0x10c3cf68,eax`), so it is non-null as soon as **one** symbol has a list.
  **Benign on this workload** — w-refs measured the list bit on 1 506 591 of
  1 506 595 records — and recorded anyway, because an omitted gate is a decode
  defect whether or not it fires.
* **`ds:0x10c462c4` is a MODE and it is 0 for an ordinary compile.** The driver
  reads `ex`/`sy` only when it is zero (`10b7f026`) and `in` only when it is zero
  (`10b7f2e0`), so an ordinary p2 run has it clear. **Consequence: on this
  workload the `force` argument is inert and every site is live**, and `force` is
  the flag that keeps a site alive in the *other* mode.

### 1c. The phase order, from the p2 driver `10b7f022`

    10b7f09d  call 10b97f98
    10b7f0a2  if (ds:0x10c3cf68) call 10b27f3c   -> 10b2773f, the prune/fixpoint
              else               call 10b9937a
    10b7f0d2  call 10b34113      -> 10b98e26, THE INITIALIZER WALK  (S3/S4/S5)
    10b7f15f  the COMPILE LOOP:
                for (p = &ds:0x10c4630c; (s = *p); )
                    if ((s[0x4c] & 0x20) && !(s[0x4c] & 2)) {
                        s[0x4c] |= 2; unlink s; compile s;   ; 10b7f199 .. 10b7f1c5
                        goto 10b7f15f; }                     ; 10b7f1e5  RESTART
                    else p = &s[0x78];
    10b7f186  call 10c1bdc0 ; call 10b8070c      -> obj emission               (S1)

> ### **The compile loop restarts at the list head after every compiled function.**
> `10b7f1e5: je 0x10b7f15f` re-reads `ds:0x10c4630c`. The emit set is **not**
> computed before codegen and then consumed — it is a **worklist run to a
> fixpoint during codegen**, so compiling one function can mark another and that
> one is then compiled. This is what makes S2 a live root channel rather than a
> late no-op, and it is the architectural fact a Phase-7 emit model has to
> reproduce.

### 1d. THE TABLE — every external `Mark` call site, with its trigger

| # | site | enclosing fn / TU | reached from | trigger — the field and the tag tested | marks what reference-following cannot? |
|---|---|---|---|---|---|
| **S1** | `10b28ca3` | `10b28a9b` `coff.c`, the COFF symbol writer | itself (`10b28cb9`) and `10b29050`; both are entries in a **function-pointer table** at `10be810a`/`10be8182` walked during obj emission, i.e. **after** the compile loop | kind-1 symbol (`[+0x30]==1`) whose COMDAT-selection field `(+0x37 >> 5) & 0xF` **== 2** takes **`[sym+0x3f]`** as a token, resolves it (`10b9860d`), sets `[tgt+0x32] \|= 4`, and marks it when `[tgt+0x37] & 0x200000` (the tag-`0x0E` function marker set at `10b9bf50`). Then recurses `10b28a9b(tgt)` to emit the associated symbol | **In the symbol table YES, in the emit set NO** — it runs after `10b7f186`, so a mark here cannot cause codegen. It is the COMDAT-**associative** channel |
| **S2** | `10b3389b` | `10b33647` `dag.c` | `10b3421b` ← `10b7e032` ← `10b7e6af` = **compile-one-function** | walks the tuple/DAG operand chain (`[insn+0x2c]` then `[insn+0x28]`); operand kind `[op+8]`: 2/3 → `edi = *[[op+0x18]+8]`; 4 → `[op+0x18]` gated on `[edi+0x30]==3 && [op+4]==0x2a7`; 5/6 → `*[[op+0x24]+8]`. Marks when `[edi+0x30]==4`, `!(+0x37 & 0x400)`, `+0x37 & 0x200000`, `!(+0x4c & 0x20)` | **NO — this IS reference-following**, at codegen time, over the same operands w-emit's `.ex` 26-token proxy reads. It is the *mechanism* behind the propagation half w-emit failed to refute |
| **S3** | `10b98be8` | `10b98b00` `p2symtab.c` | `10b98e26` ← `10b34113` ← the driver, **before the compile loop** | walks the owner's **initializer node list** (`[sym+0x33]`, built by `10b9893b` from `[sym+0x28]`); a node with **`[n]==2 \|\| [n]==0x14`** resolves `[n+8]` as a token; recurses through data symbols; marks when the target is a function | **YES — the address-take in a DATA INITIALIZER** |
| **S4** | `10b98c08` | `10b98b00` | same | the early-out arm of the same walk: target is a function, `!(+0x4c & 2)`, and **`[[owner+0xc]+0x4d] == 0x1d`** | **YES**, same channel |
| **S5** | `10b98c7f` | `10b98c0f` `p2symtab.c` | `10b98e26` (`10b98ee7`), and `10b9ac38` | the recursive form; when the symbol reached **is itself** a function (`[+0x30]==4`, `+0x37 & 0x200000`, `!(+0x37 & 0x400)`) it is marked directly | **YES**, same channel |
| **S6** | `10b9aa26` | `10b9a897` `p2symtab.c`, intern-symbol-**by-name** | `10b9ae7e`/`10b9ae89`, called from **all of codegen** — `lower.c`, `code.c`, `cgintrin.c`, `mod.c`, `misc.c`, `globregs.c`, `ltcg.c` | hash the name (`10b8a01b`) into the 128-bucket table at `0x10c67db8`, create kind-4 with `+0x37 \|= 0x87` if absent, then when `(+0x37 & 0x1e0) != 0x80` and `!(+0x4c & 2)`: **`Mark(sym, force=1)`** and `[+0x20] \|= 0x2000`. In the LTCG mode the same arm first emits diagnostic `0x10c` (`10c1ef6f`) or ICEs at **`p2symtab.c:5447`** | **YES in principle** — a name c2 mints during lowering is not in the IL at all — **but** a minted name normally has no body and `10b7ef55` returns 0, so the compile loop skips it. Effect on the emit set **untested** (§9) |

**S3/S4/S5 are one channel.** It is the only one of the four that is *both*
unreachable by reference-following *and* upstream of the compile loop. **That is
the channel this lane measures**, and §2 says it is the right one.

### 1e. Why nobody could have seen it

`10b98e26` fills its list from `ds:0x10c67db4`, loaded from the **`in`
sub-stream** (`10b7f311: mov edx,0x10b13380 ("in") ; call 10b7e276`). **w-emit's
capture kept only `gl` and `ex` and deleted `in`**, so every number in w-emit,
w-roots and w-refs is blind to this channel *by construction* — w-refs §9.2 names
it as uncapturable from the cached corpus, and it was right. Capturing it costs
one `cl` run per TU and 34 MB.

### 1f. The `in` grammar, and a gate that closed at 876/876

    record := 0x07 <byte> <varU owner> <i32c 0> node*
            | 0x00        <varU owner> <i32c 0> node*     (the leading __C1_<build>)
    node   := 0x01 <i32c type> <i32c width> <value>       scalar
            | 0x02 <varU token> <i32c addend> <i32c width>   SYMBOL REFERENCE
            | 0x03 <i16c len> <len bytes>                 blob
            | 0x08 <i32c len>                             zero fill
    value  := i16c | i32c | i64c  by width, when type != 5
            | <width raw bytes>   when type == 5  (float / double)
    EOS    := a lone trailing 0x07

**The gate is exact consumption**, per file, fail-closed. Two runs, both recorded:

| run | clean | note |
|---|---|---|
| first, without the `type` clause | **823 / 876 = 0.93950** | all 53 desyncs on one node: a `float`/`double` stored **raw** instead of through `i32c`'s one-byte form |
| with the `type == 5` clause | **876 / 876 = 1.00000**, **1 885 700 `02` nodes** over the 850 graded TUs | |

**The fix was made before any truth was read and the gate consults no c2
output**, so prereg clause 4 is honoured; it is committed at `916318f` with both
numbers rather than silently improved.

**The grammar is confirmed independently by the C++ ABI structures it has to
reproduce.** On `src/system/utl/PoolAlloc.cpp`:

* `??_R0?AVrange_error@stlpmtx_std@@@8` → `ptr(??_7type_info@@6B@)`, one int 0,
  one 30-byte blob — the three fields of `_TypeDescriptor`, the blob being
  `.?AVrange_error@stlpmtx_std@@` + NUL;
* `_CT??_R0?AVrange_error@…??0range_error@…@Z268` → int 0, `ptr(??_R0?AV…)`,
  int 0, int −1, zero-fill 4, **int 268**, `ptr(??0range_error@stlpmtx_std@@QAA@ABV01@@Z)`
  — the seven fields of `_s__CatchableType`, with the **`268` in the symbol's own
  decorated name landing in `sizeOrOffset`** and the last field being the copy
  constructor, i.e. **a function reference that exists nowhere in `.gl`**;
* `_CTA4?AVrange_error@stlpmtx_std@@` → int **4**, then four `_CT` pointers.

A wrong width would have produced garbage names. It did not. **KA-C PASS.**

---

## 2. The result — 850 TUs, 174 417 emitted names, one variable changed

| | `RGL` — the reference list **(incumbent, w-refs)** | **`RGL + INIT` — roots gain every `in`-named function** |
|---|---:|---:|
| `\|P\|` | 129 604 | **613 532** |
| **precision** | **1.00000** | **0.27289** |
| **recall** | **0.74307** | **0.95991** |
| **micro-F1** | **0.85260** | **0.42496** |
| per-TU exact `P == E` | **132 / 850** | 34 / 850 |
| root-floor coverage (`Rfloor` = 36 141) | 6 793 = **0.18796** | 31 416 = **0.86926** |

> ### **Root coverage 0.18796 → 0.86926, and F1 0.85260 → 0.42496.**
> The channel supplies **4.6×** more of the root floor than bit `0x20` alone, and
> the unfiltered reading of it over-predicts by **4.7×**. Both are true and
> neither is the headline on its own.

* **84.4 % of the incumbent's residual is now covered**: 37 821 of the 44 813
  emitted names `P_RGL` missed. **31 502** of them are named by an `in` node
  directly; the other **6 319** arrive through the reference-list closure from
  those new roots.
* The channel explains at least one otherwise-unexplained emitted function on
  **661 of 850 TUs**.
* **KA-POS: 483 928 discriminating names.** The run graded the swap.

### 2.1 The coincidence calibration decline clause 2 demands

`I ∖ P_RGL` is 242 118 names, of which **31 502 (0.13011)** are emitted. Under
uniform token coincidence over the part of `U` the incumbent does not predict the
expectation is `(174 417 − 129 604) / (1 506 586 − 129 604)` = **0.03254**.

| | measured | expected under coincidence | ratio |
|---|---:|---:|---:|
| **w-mark, `in` `02` nodes** | **0.13011** | 0.03254 | **4.00×** |
| w-emit's loose non-`26` token scan (P-e) | 0.0277 | 0.0260 | 1.07× |

**So the extraction is NOT dominated by coincidence** — it is four times enriched
where w-emit's disqualified scan was 1.07× — **and it is still an
over-approximation of c2's own rule**, because 87 % of what it names is not
emitted. Per clause 2 the recall and F1 rows above are published as a **bound on
what the channel can reach**, not as a model, and that is said in the first line.

### 2.2 The residual, class by class — and one class it cannot touch

`E ∩ U` names no closure reaches, by w-roots' unchanged classifier:

| class | `∖ P_RGL` (w-refs) | **`∖ P_INIT`** | covered |
|---|---:|---:|---:|
| free / file-scope function | 15 128 | **60** | **99.60 %** |
| **VIRTUAL member** (vtable slot) | 11 987 | **449** | **96.25 %** |
| `$` in the qualified name | 5 659 | 837 | 85.21 % |
| **`??_G`/`??_E` deleting dtor** (`#152`) | 4 521 | **3 948** | **12.67 %** |
| other `$` | 2 926 | **0** | 100 % |
| non-virtual member | 2 007 | 806 | 59.84 % |
| static member | 1 097 | 367 | 66.55 % |
| undecorated (`extern "C"` / CRT) | 866 | 1 | 99.88 % |
| adjustor thunk (access code) | 88 | **0** | 100 % |
| **total** | **44 303** | **6 482** | **85.37 %** |

> **Two readings, and both are load-bearing.**
> **(1)** The two classes w-roots and w-refs said the gap was made of — **free
> functions of the `gEaseFuncs[]` address-taken kind (99.6 %) and ordinary
> virtual members (96.3 %)** — are essentially *closed* by this channel. That is
> the confirmation.
> **(2)** The one class it barely touches is **`??_G`/`??_E` deleting
> destructors, 12.67 %**, which then become **60.9 %** of what is left. That is
> exactly `#152`: those symbols are **synthesized by c2** and are not in the IL,
> so no `02` node can ever name them. **A perfect initializer model still leaves
> `#152` standing**, and after this lane `#152` is the largest single remaining
> class of the emit-set residual rather than a 10 % footnote.

---

## 3. Why the precision collapsed — stated as decode, not as an excuse

`I` is the **unfiltered** reading, and the prereg said so before the number
existed (§7.3, and M2's "the single outcome I most expect to be wrong about").
`10b98e26` and `10b98b00` carry three skips that `I` does not model:

| where | test | what `I` does |
|---|---|---|
| `10b98e9f` | `([owner + 0x20] & 0x60) == 0x20` → **skip the owner entirely** | ignores it |
| `10b98ba8` / `10b98ecd` | `[[owner + 0xc] + 0x4d] == 0x1d` | ignores it |
| `10b98ed9` | kind-1 owner with `[owner + 0x20] & 0x4000` | ignores it |

Each is a property of the **owner** — the data symbol the initializer belongs
to. Read together with S1 (a COMDAT-associative *data* symbol dragging in another
symbol) and with F3/F4 recursing *through* data symbols, they say something about
the shape of the answer that is disassembly-derived and not fitted:

> ### **c2's emit set is a joint fixpoint over DATA and CODE symbols, not a root set over functions.**
> A vftable's slots are marked because the vftable is reached; the vftable is
> reached because its class is used. Modelling the initializer channel as an
> unconditional root set — which is what this lane registered and measured —
> necessarily over-predicts, and the measured factor is **4.7×**.

**I did not model any of the three.** Reaching for them now, after the number
came in, is precisely the move w-roots' clause 4 forbade and honoured. **It is
named as the single highest-value next measurement and left undone** (§9).

---

## 4. KA-D — the mutation through the SOLE JUDGE, in the sufficiency direction

The mutation **retargets** one `02` node instead of deleting it, which tests both
directions in one replay and is byte-length preserving by construction: a `varU`
token is 2 bytes iff `b1 & 0x80 == 0`, so a 2-byte-for-2-byte swap moves nothing
else in the stream.

    pick F_old : emitted, named by exactly ONE `02` node, NOT in closure(Seed)
    pick F_new : in U, NOT emitted, NOT in closure(Seed), named by NO node,
                 token of the same varU width
    replay through real c2.dll under wibo 1.0.1-23-g4a9dd6f

Two independent walks of the `in` stream (`instream.parse` and
`mutate_init.find_nodes`) must agree token for token before anything is written;
they did, on all three TUs. The baseline replay reproduces the pipeline obj's
COMDAT-leader set on all three (`PoolAlloc` 77, matching w-refs' own baseline).

| TU | baseline leaders | `F_old` candidates | **sufficiency (M11)** | **necessity (M12)** |
|---|---:|---:|---:|---:|
| `src/system/utl/MemStream.cpp` | 128 | 11 | **5/5** | 2/5 |
| `src/system/net/HttpReq.cpp` | 63 | 10 | **5/5** | **5/5** |
| `src/system/rndobj/EventTrigger.cpp` | 977 | 94 | **5/5** | **5/5** |
| | | | **15/15** | **12/15** |

> ### **15/15 sufficiency. Pointing one initializer node at a function c2 was not going to emit makes exactly that function's COMDAT appear.**
> This is the direction w-roots called the sharper one and it is perfect here, on
> workload TUs, at the byte offset a decoder reports. Some of the retargets pull
> a whole subtree with them — `?SetSSLCertName@HttpReq@@UAAXPBD@Z →
> ?_M_allocate_block@?$_String_base@…` gains **60** COMDATs — which is correct
> fixpoint behaviour and exactly what §1c's restarting worklist predicts.

**Necessity is 12/15 and the three survivors are one family.** All three are on
`MemStream.cpp` and all three are exception-class **copy constructors**
(`??0__Named_exception@stlpmtx_std@@QAA@ABV01@@Z`,
`??0exception@std@@QAA@ABV01@@Z`, `??0logic_error@stlpmtx_std@@QAA@ABV01@@Z`)
reached from `_CT` catchable-type records. Removing one `_CT` reference does not
unmark them, so **a second channel reaches the EH copy-constructor family** —
plausibly the `db` stream or S6, neither of which this lane decoded. Scored as a
**miss on 3 of 15** and reported as one. Note that a `??_E` *adjustor thunk*
(`??_EEventTrigger@@$4PPPPPPPM@A@AAPAXI@Z`) **is** in the `in` stream and **is**
necessary — so `#152`'s class is not uniformly outside the channel, only mostly.

---

## 5. Known-answer controls

| # | control | registered pass | measured | |
|---|---|---|---|---|
| **KA-A** | reproduce the incumbent **exactly** | all eight | `\|U\|` **1 506 586**, `\|E\|` **174 417**, `\|E∩U\|` **173 907**, `\|Seed\|` **14 662**, `\|P_RGL\|` **129 604**, precision **1.00000**, recall **0.74307**, F1 **0.85260**, per-TU exact **132** — and `Rfloor` **36 141** with `Seed` coverage **0.18796**, both w-refs' post-hoc numbers to the digit | **PASS** |
| **KA-B** | terminus gate ≥ 0.95, nodes > 0 | ≥ 0.95 | **876/876 = 1.00000**, **1 885 700** `02` nodes | **PASS** |
| **KA-C** | the published witness (`_TypeDescriptor`, `_CatchableType`, `_CTA4`) | exact, zero extra | exact — §1f | **PASS** |
| **KA-D** | **mutation against the SOLE JUDGE** | ≥3/5 each | **sufficiency 15/15**, **necessity 12/15** | **PASS / PASS**, with §4's three survivors |
| **KA-E** | incumbent gate on the unmodified tree | every incumbent | §8 | **PASS** |
| **KA-F** | dc3 HEAD before/after; wibo version | no mid-run move | `940d07dcb096` → `940d07dcb096`; wibo `1.0.1-23-g4a9dd6f` | **PASS** |
| **KA-POS** | **positive check** — `P_INIT` and `P_RGL` must DISAGREE, printed as a count | > 0 | **483 928** names | **PASS** |

**Could KA-D have gone red? Yes, and it half did.** The necessity direction
failed on 3 of 15 and the failures are a named family, not noise. The sufficiency
direction is the one that could have failed silently — if the `02` node were a
*consequence* of emission rather than a cause, retargeting it would have changed
nothing — and it did not fail.

---

## 6. Scoring the pre-registration — 4 hits, 6 misses, 2 passes, and both decline clauses fired

| # | registered **point** | interval | measured | |
|---|---|---|---|---|
| **M1** | **recall 0.88** | [0.78, 0.96] | **0.95991** | **HIT** — inside, **above** the point; +21.68 pp over the incumbent |
| **M2** | precision 0.97 | [0.90, 1.00] | **0.27289** | **MISS, far below** — the outcome I registered as the one I most expected to be wrong about, and I was wrong by more than I expected |
| **M3** | **F1 0.925** | [0.85, 0.97] | **0.42496** | **MISS, far below**; **−42.763 pp** against the incumbent → **decline clause 3 FIRED** |
| **M4** | per-TU exact 0.30 | [0.10, 0.55] | **0.04000** (34/850) | **MISS below** |
| **M5** | `\|I\|` 45 000 | [15 000, 120 000] | **245 148** | **MISS above** |
| **M6** | `\|I ∖ P_RGL\|` 25 000 | [5 000, 70 000] | **242 118** | **MISS above** |
| **M7** | soundness 0.97 | [0.85, 1.00] | **0.14086** | **MISS, far below** → **decline clause 2 FIRED** |
| **M8** | **root coverage 0.75** | [0.40, 0.98] | **0.86926** | **HIT** — inside, **above** the point. w-roots measured 0.18796 |
| **M9** | terminus 0.99 | [0.90, 1.00] | **1.00000** | **HIT**, at the ceiling |
| **M10** | vtable-slot share of the residual 0.10 | [0.00, 0.30] | **0.67834** | **MISS above** — and §2.2 explains it: the *count* of vtable slots fell 16 508 → 4 397, but `??_G`/`??_E` fell far less than the rest, so the share rose |
| **M11** | **sufficiency 4/5**, pass ≥3/5 | — | **15/15** | **PASS**, above the point |
| **M12** | necessity 4/5, pass ≥3/5 | — | **12/15 = 0.80** | **PASS** pooled; on a per-TU reading **2 of 3 TUs pass** and `MemStream.cpp` misses at 2/5 — reported both ways |

**The declared bias was that the initializer walk is the missing channel, and I
registered M1, M7 and M8 high so being wrong would cost me. M8 hit above its
point by a wide margin and M7 missed by a factor of seven** — the channel is
right and my model of it was far too permissive. **Six of twelve registered
points are misses and four of the six are in the direction of the model working
better than it does.**

### 6.1 The decline clauses — two fired, both honoured literally

* **Clause 3 (F1 gain < +2.0 pp) FIRED.** Honoured: the first line says the model
  half is an upper bound, **and I did not go looking for a further channel.** S1's
  second entry, S6's reach, the `db`/`sy` streams and `10b98e26`'s three skips are
  **named in §9 and not decoded.** Not one was pursued after the number arrived.
* **Clause 2 (M7 < 0.50) FIRED.** Honoured: §2.1 publishes the coincidence
  calibration in w-emit P-e's exact shape (4.00× against 1.07×), M1/M3 are
  labelled a bound rather than a model, and the first line says so.
* **Clause 4 (no instrument tuning after truth) HONOURED.** The `in` grammar's
  one repair — the `type == 5` raw-FP clause — was made against the terminus gate,
  which reads **no c2 output**, **before** any truth was read, and is committed at
  `916318f` with both gate numbers. `Seed`, `U`, `RGL`, the truth reader and the
  closure operator are w-roots'/w-refs' as landed; KA-A proves it to the digit.
  After M3 came in at 0.425 I changed **nothing**.
* **Clause 5 (nothing ships) HONOURED.** No `crates/` change; `PortC2` still
  returns `NotImplemented` outside its class; no `DISCLOSURE.md` row is owed.
* **Clause 7 (a refuted §1d is reported first) NOT triggered:** KA-D confirms
  S3/S4/S5 in the direction that could have failed silently.

### 6.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends.
* **`census/gate disagreement` stays 0.** It did.
* **A high recall is not a shippable predicate.** Precision 0.273 means a
  fail-closed `Emit/Skip/Unknown` built on this would be wrong about three names
  in four it claims; and order is untouched.
* **`Rfloor` is a floor, not a target.** M8 is reported because w-roots was graded
  on it and the comparison was owed. The goal is `E`, and against `E` this lane's
  model is worse than the incumbent by 42.8 pp of F1.

---

## 7. The one-shot Part-1 gate — NOT spent, as pre-registered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**. The prereg registered this before any number existed, for a
reason about the object: **this lane's model has zero fitted parameters** — every
field, width and trigger is transcribed from a named instruction, and the one
run-time choice (the raw-FP clause) was fixed against a gate that reads no c2
output, with both its numbers published.

**The registered reversal condition did not trigger, and I checked it honestly.**
Nothing was chosen by looking at truth: the grammar came from the bytes and the
ABI structures, the `02`-node rule from `10b98b00`/`10b98c0f`, the root
definition from `10b98e26`, and after M3 came in at 0.425 I changed nothing and
implemented none of §3's three skips. **The gate stays owed by whoever first
ships a root model that *does* have parameters — and modelling §3's skips is
exactly that lane.**

---

## 8. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent (as recorded) | **this tree** |
|---|---|---|
| `cargo test --workspace --release` | 687 passed, **0 failed**, 25 targets | **689 passed, 0 failed, 25 targets** — see below |
| `cargo build --release` | 0 warnings | **0 warnings**, 0 in the test build too |
| `c2rs selftest` | 219 PASS | **219 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 628 verdicts, 0 mismatch | **12/12 PASS, 2 628 verdicts, 0 mismatch, 0 SKIP, 0 NO-RESULT** |
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 8 / 0 / 0 / 863 / 7 | **8 / 0 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 8 / 107 | **25 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |
| capture cache | 871 hit, 7 miss, **0 POISONED** | **871 hit, 7 miss, 0 POISONED** |

*Compared on the **FAILED** count and the **target** count, never the passed
count.*

**The test count moved and this lane did not move it.** `687` was measured at
`73e5831`/`c7f7529`; this branch's base is **`451f1bd`**, two merges later, and
**this tree contains no Rust change at all** (`git diff 451f1bd -- crates/` is
empty), so **689 is `451f1bd`'s number** and the `687` in the
w-emit/w-roots/w-refs gate tables is stale rather than wrong. Failed is 0 and
targets is 25 in both.

**Master moved under this lane while it ran** — it is at **`ed99bdf`**
(the merge of `wt-w-frame`) as this is written, and `451f1bd` is an ancestor of
it. Every gate number on this page is against `451f1bd`; the branch needs a
rebase before it lands, and the coordinator re-gates the merged tree.

---

## 9. What this lane did NOT measure — named, so absence never reads as success

1. **`10b98e26`'s three owner skips** — `([owner+0x20] & 0x60) == 0x20`,
   `[[owner+0xc]+0x4d] == 0x1d`, kind-1 with `[owner+0x20] & 0x4000`. **The
   single highest-value next measurement.** Decoded to the instruction, modelled
   **nowhere**, deliberately, under clause 3.
2. **S1's second entry** (`10b28b02`: kind-4 with `+0x37 & 0x400000` takes
   `[sym+0x4c]` as a token while `+0x4c` is the emit flag word on tag-`0x0E`
   records). Either the field is a union or the bit selects another record
   layout. **Not decided, and not decided by assertion.**
3. **S6's reach into the emit set.** Which workload symbols the by-name intern
   marks, and whether any of them has a body, is untested.
4. **The `0x14` node kind.** `[n]==2 || [n]==0x14` in memory; only the `0x02`
   byte kind is decoded. The terminus gate would have failed on an undecoded
   byte kind and did not, which bounds but does not close this.
5. **`db` and `sy`.** Still uncaptured — and §4's three necessity survivors say a
   second channel reaches the EH copy-constructor family.
6. **Whether the `in` owner is itself emitted.** `I` is keyed on the *target*
   only; the owner's own fate is never consulted. §3 argues from the disassembly
   that it must be.
7. **`-optref`** (`FUN_10b27b7f`), the only path that clears `0x20`. Absent from
   the workload.
8. **Order.** A right set in the wrong order is still a mismatch.
9. **The 21 quarantined TUs.** Untouched (§7).

---

## 10. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots, w-emit and w-refs: **no number minted, no `#N`
pinned in code, `BOARD.md` / `ROADMAP.md` untouched** (w-book2 owns the board).
Assign at merge.

| proposed | item | claim | where |
|---|---|---|---|
| **R-a** | **The missing emit ROOT channel is the DATA INITIALIZER, in the `in` sub-stream** — `10b98be8`/`10b98c08`/`10b98c7f`, driven by `10b98e26` before the compile loop, mark every function whose address appears in any data symbol's initializer node list | it takes root-floor coverage **0.18796 → 0.86926** and recall **0.74307 → 0.95991**, covering **84.4 %** of the incumbent's residual, on 850 TUs | this file §1d, §2 |
| **R-b** | **CONFIRMED against the sole judge, 15/15 in the SUFFICIENCY direction** — retargeting one `02` node's token to an unmarked, unemitted function makes exactly that COMDAT appear in real `c2.dll`'s obj; necessity is **12/15** and all three survivors are EH copy constructors | the direction that could have failed silently; byte-length preserving, two independent stream walks cross-checked before any write | §4 |
| **R-c** | **CORRECTS w-refs §1b: there are SIX external `Mark` call sites, not seven** — `10b27731` is `Mark`'s own recursion. Full `E8`/`E9` plus absolute-address scan finds no other reference, so the enumeration is complete for this image | a scan, not a reading; the table of all six with their triggers is the lane's primary deliverable | §1a, §1d |
| **R-d** | **The compile loop is a WORKLIST RUN TO A FIXPOINT DURING CODEGEN** — `10b7f1e5` restarts the scan at `ds:0x10c4630c` after every compiled function, so codegen of one function can mark another and that one is then compiled | the architectural fact a Phase-7 emit model has to reproduce; it is what makes `dag.c`'s `10b3389b` a live root channel | §1c |
| **R-e** | **c2's emit set is a joint fixpoint over DATA and CODE symbols, not a root set over functions** — `10b98e26` skips an owner on `([owner+0x20] & 0x60) == 0x20`, F3/F4 recurse *through* data symbols, and S1 drags COMDAT-associative data symbols in | disassembly-derived, not fitted; it is why the unfiltered reading over-predicts by **4.7×** and what the next lane must model | §3 |
| **R-f** | **`#152` is now the LARGEST remaining class of the emit-set residual** — the channel covers free functions **99.60 %** and ordinary virtuals **96.25 %** but `??_G`/`??_E` deleting dtors only **12.67 %**, so they go from 10.2 % to **60.9 %** of what is left | those symbols are synthesized by c2 and are not in the IL, so no `02` node can name them — a perfect initializer model still leaves `#152` standing | §2.2 |
| **R-g** | **CORRECTS w-refs' `Mark` pseudocode: the list recursion is gated on `ds:0x10c3cf68`, and `ds:0x10c462c4` is a MODE that is 0 for an ordinary compile** — so on this workload the `force` argument is inert and all six sites are live | `10b27701` and `10b27dc3`; the driver reads `ex`/`sy`/`in` only when `0x10c462c4` is clear | §1b |
| **R-h** | **The `in` grammar, with a gate that closed at 876/876 and 1 885 700 reference nodes** — and the scalar node carries a **type** code whose value 5 is raw floating point (823/876 before that clause) | fail-closed exact consumption, plus independent confirmation from `_TypeDescriptor`, `_s__CatchableType` and `_CTA4` reproducing field for field | §1f |

---

## 11. Reproducing every number here

```sh
# 0. the binary reads (no corpus needed)
work/w-mark/xrefs.py     0x10b276e4          # every caller of Mark: 7 calls, 0 absolute
work/w-mark/dis.sh       0x10b276e4 100      # Mark itself, both gates
work/w-mark/dis.sh       0x10b7f022 400      # the p2 driver and the restarting worklist
work/w-mark/dis.sh       0x10b98e26 120      # the initializer walk and its three skips
work/w-mark/callgraph.py callers 0x10b9a897  # S6's reach: all of codegen

# 1. capture the `in` sub-stream (RUNS cl; c2 aborts C1007, quarantine-safe)
python3 work/w-mark/capture_in.py $PWD/work/w-mark/in \
        work/emitpred/magnitude/tus.txt 20

# 2. the terminus gate + the headline scan (reads w-emit's cached gl/ex/truth)
python3 work/w-mark/instream.py work/w-mark/il/test          # one TU, the witness
python3 work/w-mark/scan.py  <main-repo>/work/w-emit/il work/w-mark/in \
        <main-repo>/work/w-emit/truth work/emitpred/magnitude/truthlist.txt \
        work/w-mark/scan.jsonl 20
python3 work/w-mark/score.py work/w-mark/scan.jsonl          # -> score.txt

# 3. KA-D — RUNS real c2.dll under wibo, on non-quarantined TUs
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-mark/mutate_init.py src/system/utl/MemStream.cpp   5
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-mark/mutate_init.py src/system/net/HttpReq.cpp     5
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-mark/mutate_init.py src/system/rndobj/EventTrigger.cpp 5
```

All scripts are **stdlib-only** and read-only against the corpus
(`mutate_init.py` writes only inside its own `work/w-mark/mut/` scratch and
restores the `in` between runs). `work/` is gitignored; the scripts and the text
outputs are force-added as records, and no IL, obj or `_CL_*` artifact is
committed.
