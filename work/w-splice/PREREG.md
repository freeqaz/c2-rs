# w-splice — PREREG

    Lane:   w-splice
    Base:   master `cda124c` (the w-seq merge)
    Date:   2026-08-08
    Ships:  a MECHANISM in `crates/c2-core/` — SPLICE-0-PORT — the way
            `elide.rs` shipped mechanism E. Emitted bytes MOVE. No
            `crates/c2-il/` change: `IlBundle::functions()` stays exactly as it
            is, so `mismatch` cannot move.

Committed **before any probe of this lane exists** and before one line of
`crates/` is edited. Nothing below was measured first; every number quoted as an
input is transcribed from `docs/rungs/2026-08-08-w-seq.md`, which is landed.

---

## 0. The question, and what is already answered

`w-seq` graded, against real c2 on the whole 878-TU workload:

| hypothesis | graded | exact |
|---|---:|---:|
| **SPLICE-0** — c2's body for the caller **is** c2's body for the callee | 2,470 | **1,967** |
| — `seq` | 816 | **816** |
| — `tail` | 1,531 | 1,151 |
| — `framed` | 123 | **0** |
| SPLICE-P — the port's setup ++ the callee's body | 2,470 | 578 — **578/578** at `port_words == 1`, **0/953** at `port_words > 1` |
| SPLICE-N — two or more callees, concatenated | 548 | **0** |

and priced the shippable subset at **726** (`seq` 634 · `tail` 92): the cells
where SPLICE-0 is exact **and** the callee's disposition is `body:exact`, i.e.
the port already lowers the callee byte-exactly.

**726 is an enumeration. This lane must ship a PREDICATE**, and the whole risk
is that a predicate is a *superset* of an enumeration: it will fire on functions
w-seq never graded, including functions that are `fnbyte-exact` today.

---

## 1. THE PREDICATE, registered before it is measured

> ### **SPLICE-0-PORT.** The port's `/Gy` COMDAT body for `F` **is** the port's `/Gy` COMDAT body for `G` — text, relocations and data references — when every clause below holds.

Consulted only from `comdat_body_from_selected`, the one composition
(`crates/c2-core/src/comdat.rs`), asked **after** mechanism E and before the
ordinary `Tail`/`Seq` arms.

| # | clause | why it is here |
|---|---|---|
| **S1** | `select_function(F)` is `Selected::Tail` or `Selected::Seq` | `framed` is **0 of 123**; `cond-pair` is a conditional site (§2 of `INLINE_PREDICATE.md`) and was never graded; `plain`/`float` name no callee |
| **S2** | `F` names **exactly one call site**, with callee `G` | SPLICE-N is **0 of 548** |
| **S3** | **the port emits NOTHING around the call**: `Tail(setup)` with `setup.is_empty()`; or `Seq` with one call, an empty argument setup, no `guard`, no `early`, and a tail that is the ABI identity on r3 (`CallValue { add_k: 0 }`, or `SavedFormal { param }` whose formal is `params[0]`) | SPLICE-P is **578/578** at an empty setup and **0/953** with one. w-seq §4.2: every one of the 503 SPLICE-0 failures is a **field** of the callee's body rewritten — a source-register rename, a destination rename, a displacement fold. A non-identity setup is exactly the thing that renames |
| **S4** | `G != F` | self-recursion is `k11`'s shape; c2 does not inline it (`INLINE_PREDICATE.md` §4, `recurse` 336/336 refused) |
| **S5** | `G` is defined in **this bundle**, and every definition of that name agrees | `elide.rs`'s condition 1, for the same reason: without it the rule fires on an external |
| **S6** | the port composes a **complete** `/Gy` body for `G`, that body has **no frame**, and `G` is not itself spliced (ONE LEVEL — the fixpoint is §4's registered question, not an assumption) | a framed callee carries `.pdata` and a prologue; splicing it into `F` is a cell nobody has graded |
| **S7** | `len(body(G)) <= 64` bytes, and `G` is not varargs | **this is the inline decision, taken on the safe side of its own boundary.** `INLINE_PREDICATE.md` §2: `index(G) <= s`, and `N_max` is **UNBOUNDED** for `index <= 64` in *both* linkage classes (EXTERNAL §6.17.4; STATIC `i = index/4 <= 16`, §6.18.9) — so at `s <= 64` the decision is independent of linkage, of `inline`, of `nparams`, of `leaf` and of the site count. Varargs is the one categorical `N_max = 0` (§6.18.5) |
| **S8** | `F` materializes no data symbol of its own (`F.data_sym.is_none()`) | `F`'s whole body is discarded; an `F`-side data reference would be discarded with it, and no cell grades that |
| **S9** | mechanism **E** does not fire for `F` | one function, one answer. E is asked first and keeps its bytes |

**Everything S1–S9 reads is in the IL bundle or in the port's own emitter. No
clause reads the reference obj.** That is the difference between a predicate and
w-seq's list, and it is the clause set that has to be falsified, not the 726.

### 1.1 What the mechanism then emits

`F`'s `ComdatBody` := `G`'s `ComdatBody`: the same `text`, the same `calls`
(REL24 sites at the same in-section offsets, naming **`G`'s** callees), the same
`data_refs`, `frame: None`. **`F` acquires no relocation against `G`** — that is
the whole content of the mechanism, and it is why §5's relocation check is an
acceptance condition and not a nicety.

### 1.2 Where it may NOT fire — `PortC2::build`

`IlBundle::functions()` refuses any TU that defines one of its own callees, so
no bundle that can splice ever reaches the whole-obj emitter. That refusal is
**not** narrowed by this lane. Independently, both of `build`'s paths refuse a
bundle in which the predicate fires, with the reason named: a spliced `Seq`
loses its frame, and with it its `.pdata` record and its `$M`/`$M`/`$T` label
slots, and `frame_label_counter` is computed from the IL before any body exists.
That is `elide.rs`'s packed-path refusal, applied to both paths because the
label counter is a TU-level fact and the elision's `blr` was not.

---

## 2. GRID-T — the boundary, frozen before it is compiled

`sha256`-stamped by `gen_cells.py` and committed **before the first `cl.exe`**,
the discipline `w-seq` P7 and `w-fix` used. Every cell carries
`void anchor(){ ext_anchor(); }`, whose callee this TU does not define, so
"the port and c2 agree" cannot be confused with "the reader found nothing".
A cell the port refuses is printed **`UNGRADED`**, never dropped.

| cell | what it is | registered prediction |
|---|---|---|
| `t01` | empty setup, `tail`, lowerable leaf callee | **FIRES**, and the emitted body is byte-exact |
| `t02` | the same with a **larger** leaf callee | **FIRES**, byte-exact |
| `t03` | **the `Seq` shape** — one call, identity tail | **FIRES**, byte-exact |
| `t04` | **setup is a register move** | **REFUSES** (S3). c2's body is the callee's with a register field rewritten, so a rule that fired would emit wrong bytes |
| `t05` | **setup is arithmetic** | **REFUSES** (S3) |
| `t06` | **setup is a pointer offset** | **REFUSES** (S3) — the displacement fold |
| `t07` | **the caller is FRAMED** | **REFUSES** (S1) |
| `t08` | **two calls** | **REFUSES** (S2) |
| `t09` | **the callee is not in the port's class** | **REFUSES** (S6): no body to splice |
| `t10` | **the callee calls an external** — s12's shape | **FIRES**, and `F`'s single REL24 must name **`ext`**, not `g`. The relocation, not the byte, is the verdict |
| `t11` | **THE FIXPOINT** — `h` lowerable, `g` splices `h`, `f` splices `g` | **c2's answer is measured, not assumed.** Registered expectation: c2 closes it (all three bodies identical). The port takes **one level** either way in this rung; if c2 closes it and the port does not, that is a printed shortfall |
| `t12` | **direct self-recursion** | **REFUSES** (S4) |
| `t13` | a callee at the **size boundary** — a lowerable body above 64 bytes if the port can build one | **REFUSES** (S7) if it exists; if the port lowers nothing that large, that is printed as *"S7 never binds on today's port"* rather than claimed as a pass |
| `t14` | **CONTROL** — a caller with an empty setup whose callee this TU does **not** define | **REFUSES** (S5), and keeps its REL24 |

### 2.1 The prediction that would falsify the rule

**Any cell in `t04`–`t09`, `t12`, `t14` where the mechanism fires is a
falsification**, not a surprise, and the rule does not ship.

---

## 3. THE DECLINE FLOOR — stated before the scan, in the direction that costs

The rule **does not ship** if any of these is true at the tip:

1. **Any function moves `fnbyte-exact -> fnbyte-differs`**, checked per
   `(TU, FnCensus::emit_name)` (**#918** — never `IlFunction::mangled_name`)
   against the base scan. Not "net exact does not fall" — **per symbol, zero.**
2. `fnbyte-differs` **grows**, `fnbyte-match-tu-differs` becomes nonzero,
   `fnbyte-partition-broken` becomes nonzero, or scan `mismatch` becomes
   nonzero.
3. `scripts/gate.sh --jobs 6` is not 18/18 PASS with 0 mismatch, or
   `cargo test --workspace --release` is not 0 failed over 30+ targets.
4. **Any spliced function's relocation set disagrees with the reference obj's
   own COMDAT relocations** — per symbol, by name, type and offset (§5). A
   byte-exact body with a wrong relocation is board **#882** and this lane must
   not add to its 4,664.
5. A GRID-T refusal cell fires (§2.1).

**If the predicate ships fewer than 726, the shortfall is stated with its cause
and the predicate is NOT widened to chase the number.** w-seq §6.1 lists five
things the spec may not assume; widening past what a cell proves is how #232
shipped a live wrong emit for 255 commits.

## 3.1 Registered numeric expectations

| # | registered | why this number |
|---|---|---|
| **P1** | `fnbyte-differs` **3,195 -> 2,469 ± 60**, `fnbyte-exact` **35,982 -> 36,708 ∓ 60**, and **zero** symbols move the other way | the 726, minus whatever S3's identity-tail clause costs on the `seq` side |
| **P2** | **the claim most likely to lose** — the `seq` contribution is **634** and every one of the 816 SPLICE-0-exact `seq` differs has an **empty argument setup**. If `seq` bodies with a non-empty setup are also SPLICE-0 exact, S3 is stricter than c2 and the shortfall is on this row | S3 is generalized from `tail`'s 578/578 vs 0/953 split; **no measurement of the `seq` setups exists**, and this is where the rule can be wrong in the *under*-firing direction |
| **P3** | **S7 never binds**: zero functions are refused for `len(body(G)) > 64` on the workload | the port's lowered class is small leaves; if S7 does bind, its count is printed and the INLINE-P boundary is doing real work |
| **P4** | **CONTROL** — the number of functions the predicate fires on that were **`fnbyte-exact` before** is **0** | this is the one that decides whether S7's inline argument is sound. A nonzero count here with a byte change is a falsification of the safety argument, whatever the net |
| **P5** | **CONTROL** — `fnbyte-elided` / `-elided-exact` stay **1,516 / 1,516**, and the E population is untouched | S9. Two mechanisms, two answers, never one function with both |
| **P6** | **CONTROL** — `git diff master..HEAD -- crates/c2-il/` is **empty**; TU `match` stays **10** or a moved TU is **named**; `mismatch` stays **0** | #878's loaded gun |
| **P7** | the **family spread** of the moved population is printed — distinct symbols, distinct templates, top idioms — and a single-idiom result is **said so** | #925 / #952: 143 conversions that were all one template happened two rungs ago |
| **P8** | every spliced function's relocations are verified per symbol against the reference obj and the count is printed with its denominator | #882, and the brief's standing instruction that FBM does not check this today |

---

## 4. THE FIXPOINT QUESTION — registered as a question, not an answer

```cpp
int h(int a) { return a + 1; }
int g(int a) { return h(a); }
int f(int a) { return g(a); }
```

Mechanism E **is** a fixpoint (#920, #946). Whether SPLICE-0 closes the same way
— whether c2's body for `f` is `h`'s body — is **measured by `t11` and not
assumed**. The port takes **one level** in this rung regardless: the callee
context is built with the splice disabled, so `body(G)` is `G`'s own lowering.
If `t11` shows c2 closing the chain, the second level is a *named, priced,
unbuilt* rung and appears in "found and not taken", not in this commit.

---

## 5. THE RELOCATION CHECK — an acceptance condition, run per symbol

FBM compares a `.text` COMDAT's raw bytes and **not** its relocations
(`fnbyte-exact-relocated` = 4,664). A spliced body inherits its callee's
relocations, whose targets were resolved in the **callee's** context, so this is
precisely the mechanism where a byte-exact body can name the wrong symbol —
w-seq's `s12` is the compiled reproducer.

For **every** function the mechanism moves, this lane prints, from an obj it
compiled: the port's relocation set (symbol name, type, in-section offset)
against the reference obj's own COMDAT relocations for the same symbol. The
instrument lives in `work/w-splice/` and **not** in
`crates/c2-harness/src/gap/fnbytes.rs`, which a concurrent lane owns.

A disagreement on any symbol is decline-floor item 4.

---

## 6. What this lane will NOT do

* **Not widen `IlBundle::functions()`.** If a cell forces it, that is a finding
  to surface, not to bury (#878).
* **Not touch `crates/c2-harness/src/gap/fnbytes.rs` beyond the parameter type
  the composition requires.** No new `fnbyte-` key, no changed key.
* **Not model the register rename, the destination rename or the displacement
  fold.** They are 501 more functions (w-seq §10.2) and w-seq's own note says
  the measurement says *what* changed and not *what decides it*.
* **Not model `framed`.** 0 of 123.
* **Not use the inline map's 0.9716 decision rule as a gate.** S7 uses the
  *bound* under which that rule is categorical in both linkage classes, which is
  a different object from its residual.
