# w-inlfence — the inline fence is REAL, it is now a CLASS invariant instead of a whole-TU accident, and its whole reach on 878 TUs is **ONE FUNCTION** — because the port can enumerate a TU's own defined names on **25 of 871**

    Tag:       w-inlfence
    Slug:      w-inlfence
    Date:      2026-08-09
    Fixtures:  winlfence_opaque_callee.cpp winlfence_local_callee_neg.cpp
    Census:    712,238 → 712,237 (28.91 % → 28.91 %), **−1** function;
               emitted 39,644 → 39,643, **−1**. TU match **18 → 18**,
               mismatch **0 → 0**.
    Record:    this file; PREREG `work/w-inlfence/PREREG.md` committed at
               `2557e38a` **before the first `crates/` change** and before the
               first fixture line.
    Lane:      w-inlfence, worktree branch `worktree-agent-aa27ac4f82249ec4b`.
               Every measurement below is base master **`751351b6`** (the
               w-fltret merge) against this lane's tip.
    Ships:     one shared predicate in `crates/c2-il/src/func/bind.rs`
               (`callee_defined_here`, `callee_defined_here_unmodelled`,
               `defined_name_set`), asked by `IlBundle::functions` (unchanged
               behaviour), by the census as a new post-parse gate
               (`callee-defined-in-tu`) and by `diag.rs`'s re-ask; plus
               `tu_modelled_callees` in `census.rs`. Two new fixtures, one new
               integration test target, three unit tests. **One peer lane's test
               assertion inverted** (`dead_temp_elision.rs` m02). Board rows
               **#2220**–**#2227**; **#2228**–**#2239** left explicitly unminted.
               ROADMAP §10.29.
    Adopts:    **nothing.** No `DISCLOSURE.md` row, no `docs/whitebox/`
               constant, no threshold, no bracket, no flag bit. §7.

---

## 1. The result

> ### **THE EXPOSURE WAS REAL AND IT IS NOW A CLASS INVARIANT.** w-fltret admitted 444 emitted functions whose callees c2 inlines and `fnbyte-exact` moved by zero (#2082). Nothing was wrong on disk, because `IlBundle::functions` has refused *"a callee that is also DEFINED here"* wholesale since the MVP — but that refusal was **one `any()` at the bottom of a whole-TU gate**, and `WB_INLINE_FINDINGS.md` §7 explicitly proposes narrowing it. It is now **one named predicate** with three callers, a `_neg` fixture, an integration target of its own and three unit tests. **Board #2220.**

> ### **AND ITS ENTIRE REACH ON THE 878-TU WORKLOAD IS ONE FUNCTION.** Census **−1**, emitted **−1**. The reason is measured and it is not subtle: the port can enumerate a TU's own defined function names on **25 of 871** captured TUs. **845 have an EMPTY defined-name set** and 6 more are partial; **76 names in total** are readable across all 871 TUs put together; **212,114 of the 212,125 in-class rows that carry a callee (99.995 %) are FAIL-OPEN** on the inline question, and the port can positively establish that a callee is opaque for exactly **10** functions. **Board #2221.**

> ### **THE ONE FUNCTION IS `?supershuffle@@YAXPAD@Z`, AND IT IS THE FRONTIER'S ONLY MEASURED WRONG BODY.** `src/keygen_xbox.cpp`, the frontier's head TU — port **21 words**, reference **26**, seven words c2 has that the port does not (`work/w-inlfence/fnd_base_keygen.jsonl`). It is `wb-inline` §6's anchor and `wb-frame` §1's 26 words, reached from a completely different direction. `gap-metric frontier-codegen-wrong` goes **1 → 0** and `frontier-codegen-measured` **1 → 0**: across nine frontier TUs and 51 emitted functions this was the *only* positively-measured codegen error, and the fence converts it into a refusal. **Board #2222.**

> ### **THE FENCE'S OVER-BROADNESS TEST IS THE ORACLE'S OWN, AND IT PASSES AT 100 %.** `fnbyte-exact` **36,228 → 36,228**; the single row taken back is `fnbyte-differs` at base. 5,172 `tail` and 1,238 `seq` emitted rows are byte-exact against real `c2` — every one of them a call this fence must not take — and it takes none. **Board #2223.**

> ### **A NAIVE FENCE IS OVER-BROAD, AND SIX STANDING TESTS PROVED IT — THE PORT ALREADY HAS TWO GRADED INLINE MODELS AND THEY LIVE ONE CRATE UP.** Mechanism **E** (`c2_core::elide`, **1,877 of 1,877 byte-exact**) and mechanism **I** (`c2_core::splice`, **723 of 723**) both require a callee this TU defines, and their populations are opposite: E's callees are rows the parser REFUSED, I's are rows it ACCEPTED. Three drafts of the exemption were refuted in order — none, depth-1, fixpoint-only — by `dead_temp_elision.rs` (4 cells), `call_targets.rs` (1) and `empty_elision.rs` (1). The shipped fence refuses a callee the port has **no** model of. **Board #2225.**

> ### **AND ONE OF THOSE STANDING TESTS PINNED A WRONG EMIT AS THE EXPECTED OUTCOME.** `dead_temp_elision.rs` m02 asserted `FnByte::Differs` for a wrapper whose local callee keeps bytes — *"the honest outcome is a differ"* — since lane `w-inl0`. It is not the honest outcome; `CLAUDE.md`'s standing rule is that a refusal is strictly better than a wrong emit. The cell now asserts `Refused` and everything it was written to guard is unchanged. **Board #2224.**

> ### **THE FENCE ARMS ITSELF EXACTLY WHEN THE BIGGEST ITEM ON THE BOARD IS PAID.** The 88,228 fail-open call-carrying rows are fail-open *because* their TUs do not bind, and `vocab-gap`'s 853 TUs are what every widening lane is trying to close. `Bindings::per_record` makes the defined-name list total **as a precondition of binding at all**, so the day a TU binds, this fence starts firing on it. That is the property to check when the binding closes — not a promise this lane can grade today. **Board #2226.**

| | base `751351b6` | tip | Δ |
|---|--:|--:|--:|
| TU match | 18 | **18** | 0 |
| **mismatch** | **0** | **0** | **0** |
| per-function census | 712,238 | **712,237** | **−1** |
| **emitted census** | **39,644** | **39,643** | **−1** |
| **`fnbyte-exact`** | **36,228** | **36,228** | **0** |
| `fnbyte-differs` | 2,555 | **2,554** | **−1** |
| `fnbyte-reloc-differs` | 861 | 861 | 0 |
| `fnbyte-refused` | 130,116 | 130,117 | +1 |
| `fnbyte-elided` / `-exact` | 1,877 / 1,877 | **1,877 / 1,877** | **0** |
| `fnbyte-spliced` / `-exact` | 723 / 723 | **723 / 723** | **0** |
| `frontier-codegen-wrong` | **1** | **0** | **−1** |
| `FBM` | 0.20243 | **0.20243** | 0 |
| PROGRESS MASS | 0.20893 | 0.20893 | 0 |
| distinct body-blocker keys | 635 | 636 | **1 appeared, 0 vanished** |
| distinct emitted-blocker keys | 614 | 615 | **1 appeared, 0 vanished** |
| `gap-metric` keys | 251 | 251 | 0 appeared, 0 vanished, **17 changed** |
| workspace tests | 1,349 / 0 / 36 targets | **1,355 / 0 / 37** | **+6, +1 target** |
| `#[test]` count (`git grep -c`) | 1,357 | **1,363** | **+6** |

---

## 2. The fence, clause by clause, with each clause's cell

The fence is **one predicate with one exemption**, and saying so is the point:
seven differently-shaped refusals would be seven names for the same test.

### 2.1 The clause

> **C1 — a callee this TU also DEFINES, that the port has no model of, refuses
> the function.** `bind::callee_defined_here_unmodelled`.

`c2 cannot inline a body it does not have` is the whole of the reasoning, and it
needs no constant. Where the callee is a true undefined external the port keeps
its own call and is byte-exact — which is what
`fixtures/cpp/winlfence_opaque_callee.cpp` grades, whole-TU, against real `c2`.

| where | what |
|---|---|
| `c2-il/src/func/bind.rs` | `callee_defined_here` — the shared membership test, over `IlFunction::callees()` so **every** call carrier is covered, not just `tail_call` |
| `c2-il/src/func/bind.rs` | `callee_defined_here_unmodelled` — the same, minus the exemption; the census's form |
| `c2-il/src/func/bind.rs` | `defined_name_set` — the `.gl` defined names as a set, for the census, whose `Bindings::positional` names are **all** mangled names and would refuse every call in the workload |
| `c2-il/src/func/bundle.rs` | `IlBundle::functions`' whole-TU refusal now asks the shared predicate. **Behaviour unchanged on all 878 TUs** — the binding there is `per_record`, total and 1:1 with the `.ex` segments by construction |
| `c2-il/src/func/census.rs` | post-parse gate **(c)**, `callee-defined-in-tu`, applied LAST for the reason `OPT_MODE` is |
| `c2-il/src/func/census.rs` | `tu_modelled_callees` — the exemption, §2.2 |
| `c2-il/src/func/diag.rs` | `cause::LOCAL_CALLEE`'s re-ask, on the shared predicate, so the diagnostic cannot drift from the gate it re-states |

### 2.2 The exemption, and why a fence needs one

> **X — a callee the port ALREADY has a graded model of is not fenced.**

| mechanism | owner | what it models | graded |
|---|---|---|--:|
| **E** | `c2_core::elide` | a callee that reduces to **nothing** costs no branch | **1,877 / 1,877 exact** |
| **I** | `c2_core::splice` | a callee the port can **lower** is replaced by its own body | **723 / 723 exact** |

Their populations are opposite — E's callees are rows the parser **refused** and
I's are rows it **accepted** — so no single exemption covers both, and the
shipped set is the union: *reduces to nothing* **or** *the port can lower it*.

`tu_modelled_callees` is a **second implementation** of E's reduction, which is a
real cost and is stated in its own doc comment rather than hidden: `c2-core`
depends on `c2-il` and not the other way round, so the census cannot call
`TuEmptyCallees`. What keeps them in agreement is six standing integration cells,
§5.

### 2.3 The cells

`fixtures/cpp/winlfence_local_callee_neg.cpp` — **6 N cells** (all
`callee-defined-in-tu:eof`) and **3 X cells** (all in class), `Port=NotImplemented`,
verified per function.

| cell | shape | callee | at base |
|---|---|---|---|
| **N1** | void tail call | a local counted loop | `void-tail-call` |
| **N2** | `CallSeq` statement form (w-mcall) | two local loop members | `call-sequence` |
| **N3** | `SeqTail::CallValueFp` — **the 444's own class** | a local loop and a local float expression | `call-sequence-value-fp` |
| **N4** | int tail call, callee **over the `(300,308]` ceiling** | a 20-term multiply chain | `int-tail-call` |
| **N5** | int tail call, callee **loop-bodied over `(56,80]`** | the loop | `int-tail-call` |
| **N6** | `?supershuffle`'s shape — a `CallSeq` of local loop bodies | the loop, three times | `call-sequence` |
| **X1** | mechanism **E**'s exemption | an empty local callee | in class, **stays** |
| **X2** | mechanism **I**'s exemption | a lowerable local callee | in class, **stays** |
| **X3** | **direct recursion** | itself, lowerable | in class, **stays** |

**Each N cell's base verdict is a counterfactual, not an inference**: the base
binary (`work/w-inlfence/c2rs_base`, built from `751351b6`) censuses this file
and every N row reads `ok <shape>`. No cell is confounded, and the key is not a
fall-through — the fence is the *last* post-parse gate, so
`callee-defined-in-tu:eof` means the body parsed whole, the symbol resolved and
the mode was one the port emits under.

`fixtures/cpp/winlfence_opaque_callee.cpp` — **6 cells, whole-TU `match`** at the
fixture profile and at `/O1` and `/Ox`: the void tail call, the sequence's
statement form, its float value tail, the value tail with an argument, a defined
function nothing calls, and **F5, the near-miss** — this TU defines
`wif_local_leaf` and calls the external `wif_local_leaf_x`, of which the defined
name is a strict prefix. A `starts_with` regression fails a fixture instead of
going quiet.

`crates/c2-harness/tests/inline_fence.rs` — **four fenced shapes each paired with
its OPAQUE twin**, plus the two exemptions and their control. The pairing is what
a `_neg` fixture cannot give: board **#2085** is a `_neg` file whose whole-TU
refusal survived one of its own cells silently becoming a positive.

---

## 2.4 CAN THE PORT EMIT A WRONG BODY IN THIS CLASS? — **No**, and here is why in full

**No obj the port emits can contain a call to a callee the same TU defines.**
The emit path is `IlBundle::functions`, and three facts compose:

1. **The fence on the emit path is TOTAL, not fail-open.** The census's
   fail-openness (§4) comes from `gl_defined_names` yielding an empty pair when
   its walk stops. `functions()` cannot reach that state: it binds through
   `Bindings::per_record`, which **returns `None` unless the bound records are
   1:1 with the `.ex` `4F 1F` segments, in order**. So on the emit path, either
   the defined-name list is complete or there is no `IlFunction` at all.
2. **The predicate reads every call carrier.** It is written over
   `IlFunction::callees()`, which is the same iterator the symbol-accounting
   gate uses — tail calls, the framed call, every call in a `CallSeq`, both arms
   of a `CondTailPair`, both of an `IfCallJoin`, and the four transcription
   classes' helpers.
3. **A callee with no `.gl` symbol never gets that far**: `shape_to_function`
   refuses a token that does not resolve, per function, before this is asked.

**And the gate does NOT take the census's exemption.** §2.2's mechanism-E and
mechanism-I yields are census-only; `functions()` refuses a locally-defined
callee unconditionally, because `elide` and `splice` run inside
`c2_core::comdat_function_body` and no obj has ever been emitted for a TU that
defines one of its own callees.

**What pins it against a future narrowing** — which is the whole reason this
lane exists, since `WB_INLINE_FINDINGS.md` §7 proposes one — is
`crates/c2-harness/tests/inline_fence.rs`: four cells assert
`IlBundle::functions(...).is_none()` for a TU with an unmodelled local callee,
and four more assert `is_some()` for the opaque twin. A widening that admits the
first fails a test instead of shipping an obj.

**What is NOT claimed.** That the port emits *correctly* for every call it does
emit — `fnbyte-reloc-differs` is 861 and `fnbyte-differs` 2,554, and neither is
this fence's business. And that the census is honest about this class: it is not,
on 845 of 871 TUs (§4), and that is a driver being wrong, not an obj.

---

## 3. The census delta, signed and script-counted BY NAME

```text
census  712238 -> 712237  (-1)
emitted 39644 -> 39643  (-1)
== fn: 635 keys base, 636 keys tip, 1 appeared, 0 vanished
         +1         0 -> 1        callee-defined-in-tu:eof
== em: 614 keys base, 615 keys tip, 1 appeared, 0 vanished
         +1         0 -> 1        callee-defined-in-tu:eof
```

`work/w-inlfence/keys.py`. The **one** row, by name, off the judge's own
per-function byte test at base (`work/w-inlfence/fnd_base_keygen.jsonl`):

```text
sym  ?supershuffle@@YAXPAD@Z   shape seq   port_words 21   ref_words 26
                               del 7  (words c2 has and the port does not)
                               ins 2  (words the port has and c2 does not)
```

**Of w-fltret's 444, this fence takes back ZERO** — PREREG **P7 MISS**, **P8
MISS**, and they are the predictions this rung was most uncertain about and most
wanted to be wrong about. `call-sequence-value-fp` is **714 bodies / 444 emitted
before and after**. The 434 `Timer` TUs have an **empty** defined-name set, so
the census cannot see that `Split()` and `Ms()` are defined in them. §4 is why,
with the number.

**What the −1 is not**: it is not a retraction of anything, and it is not the
whole exposure. It is the fence's reach, and the reach is the finding.

---

## 4. WHY THE REACH IS ONE: the port cannot enumerate a TU's own names

`work/w-inlfence/probe.rs.txt`, applied as a scratch, measured through the
878-TU scan's own `bind_checks` map, reverted (`work/w-inlfence/scratch.patch`).

| | count |
|---|--:|
| captured TUs | 871 |
| …whose defined-name binding is **TOTAL** (records 1:1 with the `4F 1F` segments, in order) | **25** |
| …**partial** | 6 |
| …with an **EMPTY** defined-name set | **845** |
| defined names readable across the **whole workload**, SUMMED over all 871 TUs | **76** |
| in-class rows carrying ≥ 1 callee | **212,125** |
| …**FAIL-OPEN** (the TU's names cannot be enumerated) | **212,114 — 99.995 %** |
| …provably CLEAN (TU total, callee not defined here) | **10** |
| …**FENCED** | **1** |

The same four rows over the subset that carries an obj symbol name under
`EmitBinding` — a **superset** of the emitted census's population, and named as
such rather than reported as "emitted": **88,239 / 88,228 / 10 / 1**.

**The residue is not the type-index window.** `diag.rs`'s AB-g counterfactual —
the same walk under `bind::emit_offset_framed`, which has no window — yields
**76 names, the same 76**. Whatever stops the walk on 845 TUs, it is not the
framing this project already knows how to widen.

**And the 25 are exactly the ones that matter.** They are the 18 `match` TUs plus
**seven of the nine FRONTIER TUs** (`keygen_xbox`, `wordwrap`, `Biquad`,
`IPP_basicmath_xbox`, `EncryptXTEA`, `Pool`, `vsnprnc`); `Main.cpp` and `mmio.cpp`
are the two that do not bind. So the fence is live over precisely the region
where the port is closest to emitting — which is the region where a wrong emit
would happen first.

---

## 5. Three drafts of the exemption, each refuted by a standing test

Written in order, each one shipped-and-run before the next was thought of.

| draft | refuted by | what it got wrong |
|---|---|---|
| **none** — refuse every locally-defined callee | `call_targets.rs::the_ports_call_list_comes_from_the_emitter_and_not_from_a_copy`, and `dead_temp_elision.rs` × 4 | mechanism **E**: the port emits **no branch at all** for a call to an empty local callee, graded 1,877/1,877 |
| **depth 1** — exempt a callee whose own body emits nothing | `dead_temp_elision.rs::the_chain_closes_one_link_deeper` and two others | E's reduction is a **fixpoint** (board #924); a chain closes one and two links deeper |
| **fixpoint only** | `empty_elision.rs::a_returning_callee_is_mechanism_i_and_never_e` | mechanism **I**: `int g(int a){return a;} int f(int a){return g(a);}` is spliced, not elided, and the callee is IN class — the opposite population |

**All six cells are peer lanes' and none of them is this lane's own.** The
lane's own `_neg` fixture was *also* wrong twice in exactly the same way, and
that is PREREG **P17** hitting harder than registered: the first draft of N1–N3
gave them empty callees and graded mechanism E's exemption, the second gave them
in-class callees and graded mechanism I's. Both drafts produced a file that
`c2rs diff` grades `Port=NotImplemented` — **the graded property held while the
cells measured the wrong thing**, which is board #2085's shape occurring twice in
one file in one afternoon.

---

## 6. The declines, each named and sized

| # | declined | size, measured |
|---|---|---|
| **D1** | **the accept side of the inline predicate** — any rule of the form *"c2 will not inline this, so the port may keep the call"* | `WB_INLINE_FINDINGS.md` §7: *"The accept side is not offered."* Nothing here predicts c2's decision |
| **D2** | **direct recursion as an accept rule** (F5, 6 cells) | it converts **0 by construction**: a recursive callee is the caller, the caller is lowerable, and the exemption already yields. Cell **X3**, and the rung says explicitly that the two rules *agreeing* is not an adoption |
| **D3** | **every size ceiling** — `(300,308]`, `(212,252]`, `(100,116]`, `(156,164]`, the loop class's `(56,80]` | they are brackets, not numbers, and the port cannot ask *"how big is this callee"* without lowering it first. Cells **N4** and **N5** are the two ceilings, refused rather than exploited |
| **D4** | **varargs callee ⇒ never inlined** (F5) | it converts **0 by construction**: `Bindings::is_varargs` already refuses the whole TU on a *defined* variadic name, so no such callee ever reaches this fence |
| **D5** | `/Ob0` ⇒ nothing inlines (F3, 34 cells) | an accept side by another name. The `/Od` gate lanes are 18 fixtures at base and tip, unmoved |
| **D6** | **the budget** and the POGO cost model | `WB_INLINE_FINDINGS.md` §4.1/§4.2 record them READ, NOT CONFIRMED and unreachable; no row proposes them |
| **D7** | **restating SPLICE-0's own refusals** (`splice.rs` S1–S6) | the exemption is *"the callee's body is one the port lowers"*, which is **broader** than *"the splice will take it"*. A callee the splice declines is exempted and the port keeps its `bl` — that is the pre-existing behaviour, unchanged, and it is the fence's known hole. Its size is the gap between `fnbyte-spliced` (**723**) and the lowerable-callee population, which this lane did not instrument |
| **D8** | **any widening of `IlBundle::functions`** | held: the gate does **not** take the exemption. `elide` runs inside `comdat_function_body`, and no obj has ever been emitted for a TU that defines one of its own callees |
| **D9** | **a `__forceinline` cell** | it would grade nothing: such a callee is refused by the same clause as any other. Named in the fixture header so its absence is a decision |
| **D10** | **closing the fail-open residue** — 212,114 rows | it is not this lane's to close: it is `vocab-gap`, 853 TUs, the largest item on the board |
| **D11** | rewriting w-fltret's rung, w-inl0's rung or any board row | held. `dead_temp_elision.rs` m02's *assertion* is inverted with a dated comment beside it; its rung is untouched |

---

## 7. Adoption: NONE, and the black-box derivation is the reason

`WB_INLINE_FINDINGS.md`'s two pre-drafted rows (`W-INLINE-1`, `W-INLINE-2`) are
**route** rows needing no constant, and this lane carries neither, because it
copies nothing they describe. The fence's whole content is *"c2 cannot inline a
body it does not have"* — a statement with no ceiling, no favour-speed bit, no
instruction count and no bracket in it. `docs/DISCLOSURE.md` is unchanged and
`README.md`'s clean-room wording needs no edit.

Every number this rung quotes from that document — `(300,308]`, `(56,80]`, F5's
varargs and recursion, F3's `/Ob0` — appears in a **decline** row or a fixture
comment saying why it was *not* used. The one place a ceiling could have been
exploited is D3, and the cells that would have exploited it (N4, N5) refuse
instead.

---

## 8. Neutrality, at three levels

### 8.1 The 878 TUs, by name

```text
== per-TU verdict set: 0 changed, 0 only-in-base, 0 only-in-tip
```

match **18 → 18** (the same eighteen files), mismatch **0 → 0**, codegen-gap
**0**, vocab-gap **853**, capture-fail **7**, port-error **0**. **Every verdict
that moved, moved toward refusal — and none moved.** The single per-function
movement is `keygen_xbox.cpp`'s in-class count 2 → 1, on a TU that was
`vocab-gap` before and after.

### 8.2 Every blocker key, as a map

| column | base | tip | appeared | vanished | moved |
|---|--:|--:|--:|--:|--:|
| body blockers | 635 | 636 | **1** | **0** | 1 |
| emitted blockers | 614 | 615 | **1** | **0** | 1 |
| `prod` tags | 914 | 914 | 0 | 0 | 1 |

The one that appeared is this lane's own `callee-defined-in-tu:eof`, at **1** on
both columns. **No refusal this rung does not convert was re-keyed** — the
direction is refusal-only by construction, because the new gate is the last one
and every earlier key is reached first.

### 8.3 The `gap-metric` block, as a map

`work/w-inlfence/metricdiff.py`: **251 keys base, 251 tip; vanished 0, appeared
0, changed 17, unchanged 234.** All seventeen are downstream of the two census
counters and of the one refused body — the `fnbyte-*` family, `progress-*`, the
`cflow-residue-*` pair, and `frontier-codegen-wrong`/`-measured`/`-reader`.

### 8.4 The 314 (now **316**) fixtures, at `/O1` **AND** `/Ox`

Base **binary** (`751351b6`, rebuilt and kept as `work/w-inlfence/c2rs_base`)
against tip, over the same 316-entry list, compared per TU by name. The list was
regenerated **after the last fixture edit** and checked with `wc -l` against
`ls fixtures/cpp/*.cpp | wc -l` — w-fltret §9.2's third unnamed refusal was a
313-entry list that omitted its own `_neg` file, and that check is pre-armed in
this lane's PREREG §4 item 3.

| mode | base | tip | changed by name |
|---|---|---|---|
| `/O1` | match 158, mismatch **0** | match **158**, mismatch **0** | **0** |
| `/Ox` | match 143, mismatch **0** | match **143**, mismatch **0** | **0** |

`0 only-in-base, 0 only-in-tip` at both. Both binaries see both new fixtures, so
the counts include `winlfence_opaque_callee.cpp` as a `match` at both modes on
both sides — which is the point: **the accept side is byte-exact before and after
the fence.**

### 8.5 The FRONTIER

Byte-identical base to tip except one number: `src/keygen_xbox.cpp` reads
`18 | 20` blocked-emitted at base and **`19 | 20`** at the tip. The other eight
rows, the byte-fraction table, the control and the ledger are unchanged. The
frontier moved **in one direction, by one row, on its head TU** — and
`frontier-codegen-wrong` went **1 → 0** with it.

---

## 9. Gate

| lane | result |
|---|---|
| `cargo test --workspace --release` | **1,355 passed, 0 failed, 37 targets** (base 1,349 / 36 — **+6, +1 target**) |
| `#[test]` DELTA, by name at both revs | **+6** (1,357 → 1,363) |
| `scripts/gate.sh --require-graded --jobs 8` | §9.1 |
| `scripts/board_audit.sh` | §9.1 |
| `cargo test -p c2-harness --release --test rung_registry` | §9.1 |
| 878-TU workload scan | match **18** · mismatch **0** · census 712,237 · emitted 39,643 |
| fixtures, `c2rs census` | positive **6/6**, negative **6 fenced + 3 exempt of 17** |
| fixtures, `c2rs diff` | positive **Port=Match**, negative **Port=NotImplemented** |

### 9.1 The gate, at the shipping tree

```text
LANE                 VERDICT     graded/total  match  mismatch  flags
-------------------- ---------- ------------- ------ --------- --------------------
O1                   PASS          316/316       158         0  /O1
O1-EHsc              PASS          316/316       158         0  /O1 /EHsc
O1-Oi                PASS          316/316       158         0  /O1 /Oi
O1-Oi-EHsc           PASS          316/316       158         0  /O1 /Oi /EHsc
Ox                   PASS          316/316       143         0  /Ox
Ox-EHsc              PASS          316/316       143         0  /Ox /EHsc
Ox-Gy                PASS          316/316       141         0  /Ox /Gy
Ox-Gy-EHsc           PASS          316/316       141         0  /Ox /Gy /EHsc
O2                   PASS          316/316       147         0  /O2
O2-EHsc              PASS          316/316       147         0  /O2 /EHsc
Od                   PASS          316/316        18         0  /Od
Od-EHsc              PASS          316/316        18         0  /Od /EHsc
O1-Oi-GR             PASS          316/316       158         0  /O1 /Oi /GR
O1-Oi-EHsc-GR        PASS          316/316       158         0  /O1 /Oi /EHsc /GR
Ox-GR                PASS          316/316       143         0  /Ox /GR
Ox-EHsc-GR           PASS          316/316       143         0  /Ox /EHsc /GR
Od-GR                PASS          316/316        18         0  /Od /GR
Od-EHsc-GR           PASS          316/316        18         0  /Od /EHsc /GR
expr-sweep           PASS        19556/19556   19460         0  generated cases (of 19556)
mode-cross           PASS        90812/90812   90424         0  case-lane cells (of 90812)
hatch-red            PASS           14/14         11       n/a  arms (3 green controls)
ladder-red           PASS            5/5           3       n/a  arms (2 green controls)

lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 5688 fixture-verdicts across all lanes
sweep:  PASS — 19556 of 19556 selected cases reached, 19460 GRADED by the
        oracle (96 ungraded: no reference obj), 0 mismatch (corpus 19556)
cross:  PASS — 90424 of 90812 selected cells graded, 0 mismatch (product 90812)
```

```text
GATE: PASS — 18/18 lanes ran and every one of them graded a corpus,
  the sweep graded 19460 of 19556 generated cases and the cross graded
  90424 of 90812 case-lane cells, with 0 mismatches anywhere
  (96 sweep cases carried ungraded — the reference rejects the source).
```

**`hatch-red` and `ladder-red` passed on the FIRST run and no needle was
re-taken.** This lane's `crates/` edits are in `bind.rs`, `census.rs`,
`bundle.rs`, `body/mod.rs` and `diag.rs`, and neither `hatch.py`'s nor
`ladder.py`'s needles live in any of them — there is no `HATCH-DRIFT`. The
needle that *was* re-taken is a peer lane's test assertion, not a hatch edit:
`dead_temp_elision.rs` m02, §5 and board #2224.

| audit | result |
|---|---|
| `scripts/board_audit.sh` | **0** cited-but-rowless · **0** unresolved anchors · **0** raw line-number anchors · **0** rows-behind-the-prose · **0** duplicate row numbers (`work/w-inlfence/board_audit.out`, re-taken at the rebased tree) |
| `cargo test -p c2-harness --release --test rung_registry` | **2 passed**, after `scripts/gen_rung_index.sh` (the index is GENERATED and was regenerated, never hand-edited) |

### 9.2 Rebased onto master `0faa855a`, and why the gate above still stands

Master advanced by **3 commits** while this lane ran (`w-fltret2`, an
independent session that built §10.28's rung and declined to ship it).
**`git diff --stat 751351b6 0faa855a -- crates/ fixtures/ scripts/` is EMPTY** —
every one of them is `docs/` and `work/` — so the tree the gate ran on is
byte-identical in code to the rebased tip. Re-checked at the rebased tip anyway:
`cargo test --workspace --release` **1,355 / 0**, `rung_registry` **2 passed**,
`board_audit.sh` five zeros. Only `docs/BOARD.md` and `docs/ROADMAP.md`
conflicted, both pure appends, both resolved by keeping **both** sides; `w-fltret2`
re-landed into `#2088`–`#2096` and claimed no row in `#2220`–`#2239`.


---

## 10. PREREG scored

| # | prediction | p | outcome |
|---|---|--:|---|
| **P1** | the fence takes back ≥ 1 emitted function | 0.85 | **HIT** — exactly 1 |
| **P2** | the emitted delta is in `[−4000, −100]` | 0.55 | **MISS**, optimistic by two orders of magnitude |
| **P3** | the emitted delta is more negative than −1000 | 0.45 | **MISS**, optimistic |
| **P4** | the census delta is in `[−250000, −10000]` | 0.60 | **MISS**, optimistic |
| **P5** | **`fnbyte-exact` does not fall** | 0.72 | **HIT** — 36,228 → 36,228 |
| **P6** | ≥ 70 % of the rows taken back are NOT byte-exact at base | 0.70 | **HIT** — **100 %**, 1 of 1, `fnbyte-differs` |
| **P7** | ≥ 400 of w-fltret's 444 are taken back, BY NAME | 0.45 | **MISS** — **0**. §3 |
| **P8** | ≥ 1 of the 444 is taken back | 0.70 | **MISS** — **0**, and this is the lane's sharpest wrong prediction |
| **P9** | `?SplitMs@Timer@@QAAMXZ` is among the taken-back names | 0.55 | **MISS** |
| **P10** | mismatch stays 0 at every level | 0.95 | **HIT** — 878 TUs, 316 fixtures × 2 modes, 18 gate lanes, sweep, cross |
| **P11** | TU verdicts move toward refusal only, and in fact move zero | 0.85 | **HIT** — 0 changed |
| **P12** | `IlBundle::functions` is unchanged on all 878 TUs | 0.88 | **HIT** — by construction and by the scan |
| **P13** | the fail-open residue is **larger** than the population taken back | 0.50 | **HIT**, and by 212,114 to 1 |
| **P14** | ≥ 1 wb-inline decline clause converts 0 by construction and I can name which | 0.80 | **HIT** — **two**: varargs (D4) and direct recursion (D2) |
| **P15** | `#[test]` DELTA in `[+3, +12]` | 0.65 | **HIT** — +6 |
| **P16** | no census key vanishes; the new key is the only one that appears | 0.70 | **HIT** |
| **P17** | ≥ 1 of this lane's `_neg` cells is confounded on first writing | 0.55 | **HIT**, twice over — §5 |
| **P18** | ≥ 1 unnamed refusal fires at a pre-armed place | 0.65 | **HIT** — §10.2 |

**11 hits, 0 halves, 7 misses**, and the misses are one error: **the PREREG
predicted the size of the over-claim and not the reach of the instrument that
would remove it.** P2/P3/P4/P7/P8/P9 are six restatements of *"the census can see
this class"*, and it cannot — on 845 of 871 TUs it cannot see a TU's own name
list at all. That fact was measurable at base with the scratch this lane ended up
writing anyway, and writing it **first** would have re-pointed the whole
prediction set. Registered direction was PESSIMISTIC on the take-back; it was
**not pessimistic enough**, which is a first for this board's #770 tally.

### 10.1 The one prediction that carried the lane

**P5** is the only prediction that graded the fence rather than the census, and
it is the one that says the work is right: 36,228 byte-exact bodies before,
36,228 after, and the row removed is one the oracle grades wrong. The lesson from
#2081 — *"seventeen predictions about a census column and none about the byte
judge"* — was taken, and the byte judge is the reason this rung can claim
anything at all.

### 10.2 The unnamed-refusal budget — ONE budgeted, ONE spent, and it is the pre-armed one

**Pre-armed place 1, FENCE ORDER / CLAUSE REACHABILITY: FIRED, and it is the
budgeted refusal.** The fence's first three drafts each produced a clause that
was *reachable* but *wrong*, and the instrument the PREREG proposed — "the new
key must appear with a non-zero count and its rows must be subtracted from a
named prior key" — **would have passed all three**: every draft minted
`callee-defined-in-tu` with a non-zero count subtracted from a named key. What
caught them was six peer-lane integration cells and a rewritten `_neg` fixture.
The general form, for the next lane: **a new refusal key being non-zero says the
clause runs, never that it should have.**

**Board #1380 was pre-armed and did not fire.** Every scratch this lane applied
was preceded by a commit of all real work (`dc7c7a7a` before the probe,
`c6812799` before the base-binary build), and `git checkout -- crates/` ate
nothing. The pattern the PREREG registered — *commit first, then apply* — is the
one thing on this board that has now been tried after being written down.

**w-park's streak: 12/16.**

---

## 11. What this lane deliberately did NOT do

* **It did not widen `IlBundle::functions`.** The gate does not take the
  exemption and no obj changed anywhere.
* **It did not adopt one number from `WB_INLINE_FINDINGS.md`**, and the two
  ceilings it could have exploited are `_neg` cells instead (D3).
* **It did not close the fail-open residue** — 212,114 rows, `vocab-gap`'s
  territory, sized and handed on (D10).
* **It did not restate SPLICE-0's refusals** (D7), so the fence has a known hole
  on the callees the splice declines, and that hole is the pre-existing
  behaviour rather than something this rung introduced.
* **It did not rewrite `w-inl0`'s rung** when it inverted m02's assertion. The
  cell carries a dated comment naming this lane and the board row; the rung it
  came from is a dated record and stays as written.

---

## 12. Reproduction

```sh
<mainrepo>/scripts/configure_existing_worktree.sh .
cargo build --release -p c2-harness

# base and tip, and the three-level neutrality
sh work/w-inlfence/scan.sh scan_base            # at 751351b6
sh work/w-inlfence/scan.sh scan_tip3
python3 work/w-inlfence/keys.py       work/w-inlfence/scan_base.jsonl \
        --diff work/w-inlfence/scan_tip3.jsonl
python3 work/w-inlfence/metricdiff.py work/w-inlfence/scan_base.out \
        work/w-inlfence/scan_tip3.out
python3 work/w-inlfence/splice.py     work/w-inlfence/scan_base.jsonl \
        work/w-inlfence/scan_tip3.jsonl        # E and I, untouched

# §4 — the fence's REACH, with the scratch that measures it
git apply work/w-inlfence/scratch.patch        # then `git checkout -- crates/`
sh work/w-inlfence/scan.sh scan_probe2

# §3 — the one row, by name, off the judge's own byte test at BASE
work/w-inlfence/c2rs_base gap --list work/w-inlfence/one.txt \
    --flags-file work/dc3-workload/flags.txt --cwd <sib>/dc3-decomp \
    --fnbyte-diff-jsonl work/w-inlfence/fnd_base_keygen.jsonl

# §2.3 — the cells, and the base-binary counterfactual under them
./target/release/c2rs census fixtures/cpp/winlfence_local_callee_neg.cpp
work/w-inlfence/c2rs_base census fixtures/cpp/winlfence_local_callee_neg.cpp
./target/release/c2rs diff  fixtures/cpp/winlfence_opaque_callee.cpp

# §8.4 — the fixtures at both modes, both binaries
ls fixtures/cpp/*.cpp > work/w-inlfence/fixall.txt   # AFTER the last fixture
sh work/w-inlfence/fixscan.sh work/w-inlfence/c2rs_base work/w-inlfence/o1.txt fix_base_o1
sh work/w-inlfence/fixscan.sh ./target/release/c2rs   work/w-inlfence/o1.txt fix_tip_o1
python3 work/w-inlfence/fixdiff.py work/w-inlfence/fix_base_o1.jsonl \
        work/w-inlfence/fix_tip_o1.jsonl
```

**Committed**: `PREREG.md`, every `.py` and `.sh`, `scratch.patch`,
`probe.rs.txt`, and the path-scrubbed `.txt` analyses. **NOT committed**: the
`--jsonl` scans, `c2rs_base`, and the scratch as a `crates/` change
(`git diff master -- crates/` shows only this rung's real edits).
