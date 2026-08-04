# w-refs — the recall gap is NOT an edge problem. c2's real reference relation and w-emit's 26-token proxy induce the same closure.

    Lane:      w-refs, 2026-08-04, worktree `wt-w-refs` off master `73e5831`
    Prereg:    work/w-refs/PREREG.md, committed at `b29810f`
               BEFORE any measurement against truth. Scored in §6.
    Ships:     NOTHING under `crates/`. No fixture, no codegen, no widening.
    Status:    FINDINGS. TU match is 8 at both ends.

**One-line answer:** ***NO — and the null is sharp enough to redirect the whole
Phase-7 root question.* Replacing w-emit's 26-token `.ex` operand proxy with the
per-symbol reference list c2 itself walks moves recall from **0.74200** to
**0.74307** (+0.107 pp) and F1 from **0.85186** to **0.85260** (+0.074 pp) — a
**WASH** by my registered ±2.0 pp band, so decline clause 3 fires. The two
relations agree on **1 621 332 of 1 621 624** edges (**0.99982**) and their
closures differ on **200** of 174 417 names. Precision does move, to the
ceiling: **0.99991 → 1.00000**, because the swap deletes the proxy's *entire*
12-name false-positive population and adds 187 names of which **187 are
emitted**. And a post-hoc recomputation **corrects the operative half of
w-roots' P-e**: w-emit's root floor over c2's own relation is **36 141**, not
36 228 — it moves by **0.24 % of itself**, so the 20.4 % floor is a fact about
the ROOTS, not an artifact of the proxy.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-refs`, based on master **`73e5831`** |
| c2-rs HEAD at measurement | **`b29810f`** (the prereg), clean — **no `crates/` change exists in this lane** |
| **dc3-decomp HEAD BEFORE the run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER the run** | **`940d07dcb096…`** — **it did not move** (`work/w-refs/prov_{before,after}.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`** — checked at lane start, **not stale** (w-roots found the sibling build at `1.0.1-7-dirty` and rebuilt it; that rebuild is what I am running) |
| c2.dll read | `compilers/X360/16.00.11886.00/c2.dll`, image base `0x10b00000`, `.text` VA `0x1000` @ file `0x400` |
| IL + truth | **reused from w-emit unchanged** (`work/w-emit/{il,truth}`), 876 IL / 850 truth, same dc3 rev |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-refs/` (gitignored); scripts and text outputs force-added as records, no IL or obj committed |

### 0a. Denominators, stated once

* **850** TUs graded, from `work/emitpred/magnitude/truthlist.txt`; 7 `MISSING`.
* **174 417** emitted names, **1 506 586** gate-clean records, **14 662** seeds —
  all three reproduce w-roots exactly (KA-A, §5).
* **The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` is still in force
  and was honoured.** No held-out TU was read, captured or mutated.
  **w-emitpred's one-shot Part-1 gate is UNSPENT** — §7.

---

## 1. The instrument is a decode of both halves of c2's fixpoint

Re-read with `objdump` before the prereg was written; `work/w-refs/dis.sh`
reproduces each address.

### 1a. The READER — `10b9bf99` … `10b9c007`

| VA | bytes | meaning |
|---|---|---|
| `10b9bf46` | `83 7d 88 0e  0f 85 ce 00 00 00` | `cmp [ebp-0x78],0xe; jne` — **the list is tag-`0x0E` only** |
| `10b9bf99` | `f7 46 4c 00 10 00 00  74 67` | `test [esi+0x4c],0x1000; je` — gated on the **same flag word** whose `0x20` bit is w-roots' seed |
| `10b9bfa9` | `83 3d 70 d0 c6 10 00` | `cmp ds:0x10c6d070,0` — count is `i32c` when set, `i16c`+`movzx` when clear |
| `10b9bfce` | `e8 48 39 08 00` | `call varU` — the token |
| `10b9bfd6` | `e8 cb 39 08 00  0f b7 d8` | `call i16c; movzx ebx,ax` — the use count |
| **`10b9bfde`** | **`66 85 db  74 20`** | **`test bx,bx; je` — a zero-use entry is parsed and then DROPPED. It is not an edge.** |

### 1b. The WALKER — `10b276e4`

    Mark(sym, edx):
      if (sym[0x4c] & 0x20) return;            // already marked -> stop
      if (ds:0x10c462c4 && edx == 0) return;
      sym[0x4c] |= 0x20;                       // MARK — the seed bit itself
      ecx = sym[0x80]; if (!ecx) return;       // the reference list
      for (node = ecx[0xc]; node; node = node[0]):
          tgt = node[4][4]
          if (tgt[0x37] & 0x400) continue;     // storage-class nibble 0xa: SKIP
          if (tgt[0x4c] & 0x20) continue;
          Mark(tgt, edi)

**c2's emit set is the least fixpoint of `flags4c |= 0x20` over exactly this
list**, and `p2/main.c`'s loop at `10b7f16b` then compiles whatever kept `0x20`.
That is not an analogy to w-roots' model — it *is* w-roots' model, with the
proxy replaced by the real relation. Which is why this lane is a clean
single-variable experiment and why its null is informative.

**Seven other `Mark` call sites exist** (`10b27731`, `10b28ca3`, `10b3389b`,
`10b98be8`, `10b98c08`, `10b98c7f`, `10b9aa26`). They are **additional ROOTS,
not additional edges** — `10b28ca3` in particular marks a function whenever a
*data* node resolves to it (`test [edi+0x37],0x200000`, the tag-`0x0E` marker set
at `10b9bf50`). This lane does not model any of them (§9).

### 1c. Two known-answer gates, both of which could have gone red

**The TERMINUS gate.** A decoded list must end **exactly** at the next record's
`<tag> <varU token> <0x00|0x26> <name>` header. Over 850 TUs:
**1 505 815 / 1 506 591 = 0.99948**, against a registered ≥ 0.98. **26 records
across the corpus discriminate** the two count widths (their count field is the
`0x80` escape) and the wide `i32c` reading wins every one, which is how
`ds:0x10c6d070` was fixed — before any truth was read, and recorded in the
prereg with its numbers.

**The PUBLISHED WITNESS.** `PHASE7_VALIDATION.md` §7 prints the reference list of
`?HolmesXboxPath@@YA?AVString@@PBD0@Z` — nine names, found by `glgraph.py`'s
*over-approximating payload scan*, at dc3 rev `fbf097a5`. This decode, from the
disassembly, at dc3 rev `940d07dc`, returns **exactly those nine, in that order,
with use counts**, plus one token the symbol index does not resolve. **Zero
missing, zero extra.** A wrong pair layout or a wrong count width would have
produced garbage names; it did not.

---

## 2. The result — 850 TUs, 174 417 emitted names, one variable changed

| | `R26` — the 26-token PROXY (incumbent) | **`RGL` — the REAL reference list** | `R26 ∪ RGL` |
|---|---:|---:|---:|
| edges | 1 621 624 | **1 622 745** | 1 622 917 |
| `\|P\|` | 129 430 | **129 604** | 129 604 |
| precision | 0.99991 | **1.00000** | 0.99991 |
| recall | 0.74200 | **0.74307** | 0.74307 |
| **micro-F1** | **0.85186** | **0.85260** | 0.85257 |
| per-TU exact `P == E` | 132 / 850 | **132 / 850** | 132 / 850 |
| delta over emit-everything (F1 0.20752) | +64.4348 pp | **+64.5084 pp** | +64.5057 pp |

> ### **Edge agreement 0.99982 — 1 621 332 of the proxy's 1 621 624 edges are also reference-list edges.**
> The instrument w-emit built out of `.ex` operand bytes reproduces c2's own
> per-symbol reference relation, restricted to bodies, to within two parts in ten
> thousand. That was not knowable before this lane; it is now measured.

### 2.1 What the swap actually does — 200 names, and they are all interesting

KA-G's discriminating count is **200 of 174 417**, and the whole content of the
change is legible in it:

| | n | of which emitted |
|---|---:|---:|
| `P_RGL` only — added by the real list | **187** | **187** |
| `P_26` only — added by the proxy | 13 | **1** |

**Every one of the 187 names the real list adds is emitted, and 12 of the 13 the
proxy adds are not.** Those 12 are the proxy's *entire* false-positive
population, which is why precision goes to **1.00000** — an exact zero, on
129 604 predictions. **The reference list is strictly better; it is just better
by 200 names.**

### 2.2 The list is sparse, and it does carry data references — they just have no body

| per the whole corpus | |
|---|---:|
| records with the list bit (`flags4c & 0x1000`) | 1 506 591 of 1 506 595 |
| (token, use-count) pairs decoded | **2 930 866** |
| pairs with use count **0** — parsed, then dropped (`10b9bfde`) | **54 846** (1.871 %) |
| records of storage class `0xa` — skipped by the walker | **1** |

On `src/App.cpp`, of 11 998 pairs: 6 989 resolve to a name **in `U`**, **3 572
resolve to a name that is not in `U`** (data symbols, externals — the list does
carry them), 1 206 resolve to nothing the symbol index will bind, and 231 have
use count 0. So the data references are real; they are removed by the
`∩ U` restriction because a vftable has no `.ex` body, and there is no second hop
out of them (§3).

---

## 3. Why the swap cannot close the gap — the structure, verified two ways

Registered in the prereg §1c **before any truth was read**, and it survived:

**The reference list is a tag-`0x0E` field.** `cmp [ebp-0x78],0xe / jne` at
`10b9bf46` puts the `+0x54` anchor, the `+0x4c` flag word and the list behind one
tag test. Data symbols — vftables, function-pointer tables — reach `10b9c01e`
instead and carry no list. **So there is no vftable → slot edge and no
`gEaseFuncs[]` → function edge in `.gl`, of any tag.**

Checked against the bytes as well as the disassembly, on `src/App.cpp`:

| name | token | occurrences in the whole 1.5 MB `.gl` |
|---|---|---:|
| `?EaseBackInOut@@YAMMMM@Z` — the canonical address-taken case | `e5 cf 01 00` | **1** — its own header |
| `??_7Message@@6B@` — the vftable | `d6 f9 01 00` | 8 — every `Message` ctor and `~Message` reference it |
| `??_GMessage@@UAAPAXI@Z` — the slot | `d8 f9 01 00` | 2 |

The vftable is a **target** of the list and a **dead end** in it. Generalised to
the corpus (N9): among the residual names with a 4-byte token, **17 972 of
32 886 — 54.6 %** occur **exactly once** in their TU's entire `.gl`. *Over half
of what the reference list misses is referenced by nothing in `.gl` at all.*

**This corrects the data-symbol half of `PHASE7_VALIDATION.md` §7**, which
carries from `glgraph.py`'s docstring the claim that the `.gl` records the
reference list *"for data symbols too, so a static table of function pointers
links to everything whose address it takes, and a vftable links to its slots"*.
§7 itself labels that *"a hypothesis from a docstring, not a measured fact"*, and
the measurement is: **the function half is exactly right** (§1c's witness
reproduces perfectly) **and the data half is wrong.**

### 3.1 The residual is the same shape it was, on both relations

| `E ∩ U` ∖ `P` | over `RGL` (n = 44 303) | over `R26` (n = 44 489) |
|---|---:|---:|
| free / file-scope function | 15 128 (**34.1 %**) | 15 128 (34.0 %) |
| **VIRTUAL member** (reachable only through a vtable slot) | 11 987 (**27.1 %**) | 11 992 (27.0 %) |
| `$` in the qualified name (template **or** adjustor thunk) | 5 659 (12.8 %) | 5 806 (13.1 %) |
| **`??_G`/`??_E` deleting dtor** (vtable slot, synthesized — `#152`) | 4 521 (**10.2 %**) | 4 521 (10.2 %) |
| other `$` | 2 926 (6.6 %) | 2 926 (6.6 %) |
| non-virtual member | 2 007 (4.5 %) | 2 032 (4.6 %) |
| static member | 1 097 (2.5 %) | 1 104 (2.5 %) |
| undecorated (`extern "C"` / CRT) | 866 (2.0 %) | 868 (2.0 %) |
| adjustor thunk (access code) | 88 (0.2 %) | 88 (0.2 %) |

**The two columns are the same histogram.** Vtable slots are **37.3 %** of the
residual under the real relation and 37.2 % under the proxy; free functions are
34.1 % under both. w-roots reported 38.2 % / 44.6 % over its 150-TU
characterisation sample; the difference between 44.6 and 34.1 is **population,
not relation** — the proxy's own number over all 850 is 34.0 %.

---

## 4. POST-HOC, NOT PRE-REGISTERED — the root floor, recomputed over c2's own relation

**This section was not registered.** It is reported separately, in its own
section, and never mixed into §6's scored table. It exists because this lane is
the first to hold the instrument that can grade **w-roots' landed claim P-e**:

> *"w-emit's 'roots must supply 20.4 %' is a fact about the `26`-PROXY, not about
> the roots — `Rfloor` counts every vtable slot and every address-taken free
> function, so it is an upper bound of unknown tightness."*

`Rfloor(t) = { f ∈ E(t) : no edge from any f' ∈ E(t) reaches f }`, recomputed with
the relation swapped and nothing else changed (`work/w-refs/rfloor.py`):

| relation | `\|Rfloor\|` | % of `\|E\|` | covered by `Seed` |
|---|---:|---:|---:|
| 26-token **proxy** (w-emit / w-roots) | 36 228 | 20.771 % | 6 797 = 0.18762 |
| **REAL `.gl` reference list** | **36 141** | **20.721 %** | 6 793 = **0.18796** |
| union of both | 36 140 | 20.720 % | 6 792 = 0.18794 |

> ### **The floor moves by 87 names — 0.24 % of itself — when you give it c2's own reference relation.**

**P-e's first clause is confirmed and its operative clause is REFUTED.** `Rfloor`
*does* contain every vtable slot and every address-taken free function, exactly
as P-e says. But the inference everyone will draw from "upper bound of unknown
tightness" — that a better edge relation would shrink it — is **measured false**.
The tightness is now known: **0.24 %**.

Two consequences, and they point in opposite directions from where w-roots left
the question:

1. **w-emit's "the roots must supply 20.4 % of every emitted name" stands.** It
   is a fact about the roots. It was never a fact about the proxy.
2. **w-roots' S3 miss is therefore about `0x20`, not about the instrument.**
   w-roots registered root coverage at a point of **0.85** with interval
   **[0.55, 1.00]** and measured **0.18762**; under c2's own relation it is
   **0.18796**. The miss is not rescued by a percentage point of it. `0x20` is a
   **perfectly sound and badly incomplete** root oracle, and that is now true
   against the real relation and not only against a proxy.

---

## 5. Known-answer controls

| # | control | registered pass | measured | |
|---|---|---|---|---|
| **KA-A** | reproduce the incumbent **exactly** | exact on `\|U\|`, `\|E\|`, `\|Seed\|`, `\|P_26\|`, precision, recall, F1, per-TU exact | **1 506 586 / 174 417 / 14 662 / 129 430 / 0.99991 / 0.74200 / 0.85186 / 132** — all eight identical | **PASS** |
| **KA-B** | terminus gate ≥ 0.98, discriminating count > 0 | **0.99948** (1 505 815 / 1 506 591); **26** discriminating records | **PASS** |
| **KA-C** | `PHASE7_VALIDATION.md` §7's published nine-name witness | **9/9, zero extra**, with use counts the original reader could not produce | **PASS** |
| **KA-D** | **mutation against the SOLE JUDGE**: zero one edge's use-count byte, replay through real `c2.dll` — **≥ 3/5 lose exactly that COMDAT** | **7/15 = 0.467** exact; **9/15 = 0.600** lose the target at all | **MISS** — §5.1 |
| **KA-E** | incumbent gate on the unmodified tree | every incumbent reproduced, §8 | **PASS** |
| **KA-F** | dc3 HEAD before/after, wibo version | `940d07dcb096` → `940d07dcb096`; wibo `1.0.1-23-g4a9dd6f`, checked not stale | **PASS** |
| **KA-G** | **positive check** — `P_RGL` and `P_26` must DISAGREE somewhere, printed as a count | **200** names (187 + 13). The run graded the swap | **PASS** |

### 5.1 KA-D missed, and the miss says the same thing the headline says

The mutation is byte-length preserving by construction: a small positive use
count is one `i16c` byte, so writing `0x00` over it removes the edge
(`10b9bfde`) and moves nothing else in the stream. The baseline replay reproduces
the pipeline obj's COMDAT-leader set on all three TUs, and the two independent
reference-list walks (`refs.reflist` and `mutate_ref.walk_pairs`) agree on
**0 of 1 294** records disagreeing.

| TU | baseline leaders | single-in-edge candidates | exact | target lost |
|---|---:|---:|---:|---:|
| `src/system/utl/PoolAlloc.cpp` | 77 | 33 | **0/5** | 0/5 |
| `src/system/utl/BeatMap.cpp` | 51 | 29 | **3/5** | **5/5** |
| `src/system/utl/Symbol.cpp` | 228 | 136 | **4/5** | 4/5 |
| | | | **7/15** | **9/15** |
| `src/system/math/SHA1.cpp` | 11 | **0** | — | — |

Scored against what I registered — *"lose exactly that COMDAT"* — this is
**7/15 = 0.467 against 0.60, a MISS**, and I report it as one. Two things are
worth reading next to it and neither rescues it:

* **Two of the eight non-exact cases lost MORE than the target**, not less
  (`lost=2` and `lost=3` on `BeatMap.cpp`, each including the target and its own
  downstream-only callees). That is correct fixpoint behaviour and my
  `lost == {f}` criterion counts it as a failure. Under the weaker *"the target
  is lost at all"* reading the score is **9/15 = 0.600**, exactly at the mark —
  reported beside the registered number, never in place of it.
* **All six genuine survivors are `??$MakeString@…` instantiations**, and the
  prereg required me to say which of the two explanations applies. It is the
  second: their in-degree is **1 in both relations** — the proxy and the real
  list agree that exactly one edge reaches them — so no instrument missed an
  edge. **Something else marks them**, i.e. one of §1b's seven other `Mark` call
  sites. `??$MakeString@…` is also the **top of the residual histogram** (484 /
  435 / 428 TU-instances, §2 tail), so the control failed on precisely the
  population the headline says is a root problem.

**KA-D is the only control here that consults the sole judge, and it is the one
that missed.** What it bounds is the claim *"the list is c2's edge relation"* —
that claim is supported for 9 of 15 edges and refuted for 6, all of one family.
It does **not** bound §2's numbers, which are decode-level and gated by KA-A/B/C.

---

## 6. Scoring the pre-registration — 10 hits, and the headline hit its interval while missing its point

| # | registered point | interval | measured | |
|---|---|---|---|---|
| **N1** | **RECALL 0.76** | [0.70, 0.85] | **0.74307** | **HIT** — inside the interval, **below the point**; +0.107 pp over the incumbent |
| **N2** | precision 0.9995 | [0.9950, 1.0000] | **1.00000** | **HIT**, at the ceiling |
| **N3** | **F1 0.862** | [0.820, 0.910] | **0.85260** | **HIT** — inside; **decision band vs the incumbent: WASH** (+0.074 pp, needed ≥ +2.0) |
| **N4** | union recall 0.79 | [0.72, 0.90] | **0.74307** | **HIT**, at the low edge — the union adds **nothing** over `RGL` |
| **N5** | edge agreement 0.90 | [0.70, 1.00] | **0.99982** | **HIT**, at the top |
| **N6** | per-TU exact 0.17 | [0.05, 0.45] | **0.15529** (132/850) | **HIT** — identical to the incumbent, to the TU |
| **N7** | vtable-slot share 0.37 | [0.25, 0.50] | **0.37262** | **HIT** — §1c's structural read survives |
| **N8** | free-function share 0.44 | [0.32, 0.56] | **0.34147** | **HIT**, low; the proxy's own number over 850 is 0.34019, so this is population, not relation |
| **N9** | unreferenced-in-`.gl` 0.70 | [0.40, 0.95] | **0.54649** (17 972 / 32 886) | **HIT** |
| **N10** | `\|P_RGL\|` 135 000 | [110 000, 175 000] | **129 604** | **HIT** |

**All ten intervals hit, and that is not the flattering reading.** Four of the
ten point estimates (N1, N3, N4, N8) are **above** their measured value, in the
direction of the swap working better than it did, and the two that carry the
lane's question — N1 and N3 — are both of them. **I registered the swap as a
modest improvement and it is a wash.** The intervals were wide because I did not
know; the points are where the belief was, and the belief was wrong in a
consistent direction.

*(Noted for the record, since this page quotes it: w-roots' S3 was registered at
a **point of 0.85** with interval [0.55, 1.00]; 0.55 is the interval floor and
the decline threshold, not the registration. Quoting 0.55 as the registration
understates that miss by more than half.)*

### 6.1 The decline clauses — clause 3 fired, and it is honoured literally

* **Clause 3 (`0.74200 < N1 < 0.90`) FIRED: "IMPROVED BUT NOT CLOSED".** Honoured
  literally. **I did not go looking for a second edge channel, a different record
  kind, a data-initializer stream, an extra `Mark` call site, or any root source
  beyond `Seed`.** §1b names the seven other `Mark` sites and §9 names the
  uncaptured stream *so that absence never reads as success* — naming them is
  where this lane stops. Not one was decoded.
* **Clause 1 (decode trust) NOT triggered:** KA-B 0.99948 ≥ 0.95.
* **Clause 4 (no instrument tuning) HONOURED.** `ds:0x10c6d070` was fixed to the
  wide `i32c` reading by the terminus gate **before any truth was read**, and the
  prereg records the three-TU validation with its numbers. `Seed`, `U`, `R26`,
  the truth reader and the closure operator are w-emit's/w-roots' as landed —
  KA-A proves it, to the digit. Nothing was changed after seeing a number.
* **Clause 5 (nothing ships) HONOURED.** No `crates/` change. `PortC2` still
  returns `NotImplemented` outside its class.
* **Clause 6 (a refuted §1c is reported first) NOT triggered:** N7 and N8 both
  landed inside their intervals, so §1c stands.
* The **post-hoc** §4 is labelled post-hoc in its own heading, in the commit
  message, and in `rfloor.py`'s docstring. It moves no registered number.

### 6.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends.
* **`census/gate disagreement` stays 0.** It did.
* **The single outcome I said I most expected to be wrong about** was N1 landing
  *below* the incumbent. It landed 0.107 pp above. The proxy did not beat the
  real relation; it tied it.

---

## 7. The one-shot Part-1 gate — NOT spent, as pre-registered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**. The prereg registered this before any number existed, for a
reason about the object: **a decode has no free parameters** — the one run-time
boolean was fixed against a gate that reads no c2 output, with its discriminating
count published — **and this lane is comparative by construction**, `RGL` against
`R26` on the *same* 850 TUs, which 21 held-out TUs cannot improve on.

**The registered reversal condition did not trigger, and I checked it honestly.**
Nothing was chosen by looking at truth: the layout came from named instructions,
the count width from the terminus gate, the zero-use drop from `10b9bfde`, the
class-`0xa` skip from `10b276e4`, and after N1 came in at 0.743 I changed
nothing. **The gate is still owed by whoever first ships a root model that
has parameters, and this lane deliberately did not become that lane.**

---

## 8. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent | this tree |
|---|---|---|
| `cargo test --workspace --release` | 687 passed, **0 failed**, 25 targets | **687 passed, 0 failed, 25 targets** |
| `cargo build --release` | 0 warnings | **0 warnings** |
| `c2rs selftest` | 219 PASS | **219 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 628 verdicts, 0 mismatch | **12/12 PASS, 2 628 verdicts, 0 mismatch** |
| TU match / mismatch / vocab-gap / capture-fail | 8 / 0 / 863 / 7 | **8 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 8 / 107 | **25 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |

*Compared on the FAILED count and the target count, never the passed count.*

---

## 9. What this lane did NOT measure — named, so absence never reads as success

1. **The seven other `Mark` call sites** (§1b). These are where the missing 25.8 %
   comes from, and `10b28ca3` — *a data node resolved to a function record, mark
   it* — is the shape the address-taken class needs. **Not decoded. Deliberately,
   under decline clause 3.**
2. **The data-initializer stream itself.** w-emit's capture kept only `gl` and
   `ex` of the `_CL_*` quintet; `in`, `db` and `sy` were discarded, so a channel
   living there is not merely unmeasured but **uncapturable from the cached
   corpus**. Re-capturing 850 TUs is a separate lane with its own prereg.
3. **Tag-`0x02`/`0x04`/`0x10` records.** They carry no reference list (§3).
   Whether they carry references some *other* way is untested.
4. **The 1 206-per-large-TU unresolved tokens.** They count against recall, the
   conservative direction, and are uncharacterised.
5. **Order.** A right set in the wrong order is still a mismatch.
6. **`-optref`** (`FUN_10b27b7f`), the only path that clears `0x20`. Absent from
   the workload.
7. **The 21 quarantined TUs.** Untouched (§7).
8. **Why `??$MakeString@…` survives edge removal** (§5.1). Characterised as a
   root, not localised to one.

---

## 10. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots: **no number minted, no `#N` pinned in code,
`BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md` untouched.** Assign at merge.

| proposed | item | claim | where |
|---|---|---|---|
| **Q-a** | **The recall gap is a ROOT problem, not an EDGE problem** — c2's own per-symbol reference relation and w-emit's 26-token proxy agree on **1 621 332 of 1 621 624** edges (0.99982) and their closures differ on **200** of 174 417 names | the swap the brief asked for, run as a single-variable experiment on 850 TUs with the incumbent reproduced to the digit | this file §2 |
| **Q-b** | **The real reference list takes precision to exactly 1.00000** — it adds 187 names of which 187 are emitted and removes 13 of which 12 were the proxy's entire false-positive population | the one thing the swap does buy, quantified | §2.1 |
| **Q-c** | **CORRECTS w-roots' P-e: the 20.4 % root floor is NOT an artifact of the proxy** — recomputed over c2's own relation it is 36 141 against 36 228, a move of **0.24 % of itself**. P-e's first clause stands, its operative clause does not | POST-HOC and labelled; the first measurement of `Rfloor`'s tightness rather than an assertion about it | §4 |
| **Q-d** | **CORRECTS the data-symbol half of `PHASE7_VALIDATION.md` §7** — the reference list is a tag-`0x0E` field (`cmp [ebp-0x78],0xe` at `10b9bf46`), so a vftable is a *target* of the list and a dead end in it, and a function-pointer table has no list at all. §7's function-half witness reproduces **9/9** | §7 labels the data half a docstring hypothesis; this measures it, both from the disassembly and from the bytes | §1c, §3 |
| **Q-e** | **Over half of what the reference list misses is referenced by NOTHING in `.gl`** — 17 972 of 32 886 residual names with a 4-byte token occur exactly once in their TU's entire `.gl` | the quantitative form of Q-d, and the reason no `.gl`-only model can close the gap | §3 |
| **Q-f** | **A `.gl` reference-list ENTRY is load-bearing for emit, on 9 of 15 tested edges** — zeroing one use-count byte and replaying through the real `c2.dll` removes the COMDAT; **all six survivors are `??$MakeString@…`**, with in-degree 1 in *both* relations, so a second `Mark` root reaches them | the registered mark was ≥3/5 exact and the measured 7/15 **MISSES** it; reported as a miss with its characterisation | §5.1 |
| **Q-g** | **The zero-use entry is real and common** — `test bx,bx / je` at `10b9bfde` drops **54 846** of 2 930 866 pairs (1.871 %); a reader that treats every pair as an edge is wrong on one entry in fifty-three | transcribed, not fitted; part of why the terminus gate closes at 0.99948 | §1a, §2.2 |

---

## 11. Reproducing every number here

```sh
# 0. the binary reads (no corpus needed)
work/w-refs/dis.sh 0x10b9bf99 120       # the reference-list reader
work/w-refs/dis.sh 0x10b276e4 100       # the fixpoint walker
work/w-refs/dis.sh 0x10b9bf46 20        # cmp [ebp-0x78],0xe — the tag-0x0E gate

# 1. the headline scan — reads w-emit's cached IL + truth, runs NO c2
python3 work/w-refs/scan.py  <repo>/work/w-emit/il <repo>/work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-refs/scan.jsonl 24
python3 work/w-refs/score.py work/w-refs/scan.jsonl        # -> score.txt

# 2. POST-HOC: the root floor over both relations
python3 work/w-refs/rfloor.py <repo>/work/w-emit/il <repo>/work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-refs/rfloor.jsonl 20

# 3. KA-D — RUNS real c2.dll under wibo on non-quarantined TUs
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-refs/mutate_ref.py src/system/utl/BeatMap.cpp  5
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-refs/mutate_ref.py src/system/utl/Symbol.cpp   5
C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo> \
  python3 work/w-refs/mutate_ref.py src/system/utl/PoolAlloc.cpp 5
```

All scripts are **stdlib-only** and read-only against the corpus
(`mutate_ref.py` writes only inside its own `work/w-refs/mut/` scratch and
restores the `.gl` between runs). `work/` is gitignored; the scripts and the text
outputs are force-added as records, and no IL, obj or `_CL_*` artifact is
committed.
