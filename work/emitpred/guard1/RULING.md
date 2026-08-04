# guard1 — Phase B ruling

Agent `guard1`, lane `w-emitpred`. Phase-A derivation frozen and committed at
**`75bf3bb`** (`work/emitpred/guard1/DERIVATION.md`, verbatim, unedited by the
lead) **before** any prediction, result, or observed set was read. Everything
below was written after that commit.

**Provenance of the leak.** The Phase-A brief written by the lead disclosed that
`a5c4`'s `cand`/`cand2` measure Selection = 2, which entails that both symbols
are emitted. That is **the lead's leak, not this guard's contamination** — the
lead has accepted the attribution. My response was to disclose it in
`DERIVATION.md` §0 before writing anything, quarantine `a5c4`, name **`a5c1` as
the load-bearing clean cell before looking at it**, and pre-commit the two
contested rulings (Selection-byte admissibility, V6 weighting) inside the frozen
file. As it turns out **the leak is not load-bearing for any conclusion below**:
the decisive cell is `a5c1`, which was clean, and the Selection byte is ruled
non-decisive in §4.

---

# 1. The finding that needs no experiment: R1 is internally inconsistent as text

Before any cell, any obj, any count. R1 reads:

> **Roots:** (1) every definition with external non-COMDAT linkage — plain
> extern, `extern "C"`, *any out-of-line definition* (member, static member,
> virtual), and anonymous-namespace functions not declared `static`; …

Take the intersection

> an out-of-line member function definition marked `inline`

It is simultaneously **"*any* out-of-line definition"** — a root, by the
emphasised list item — and **COMDAT linkage** — not a root, by the head. The
head excludes what the list includes. No reordering, and no choice between the
two readings the lane debated, repairs it: Reading A discards the list's
emphasis, Reading B discards the head's qualifier. **Both readings survive only
by deleting part of the sentence.**

This is provable by reading, in the absence of a compiler. It was written into
`DERIVATION.md` §1.5 during the quarantine, it is reached through the *member*
item and therefore does not depend on `a5c1`, `a5c4`, or any cell in the lane,
and it is immune to the lead's leak.

It also reframes the rest of §5 below. `a5c1`/`a5c4` are not a puzzle about
*which* of two readings is right. They are two more instances of an intersection
R1 cannot resolve under **either** reading — reached through the non-member
items (`extern "C"` ∧ inline, `extern` ∧ inline) instead of the member one.

**A defect that costs nothing to find outranks one that costs 59 objects.** §2's
R1 should be repaired on this ground alone, whatever the lane's violation count
turns out to be. The repair is in §6.

---

# 2. Per-cell table

`G` = my frozen Phase-A derivation. `A` = the first agent's registered
prediction. `T` = truth (obj CODE-characteristic COMDAT leader set). Rulings per
the prereg: **VIOLATION CONFIRMED** (G agrees with A, T contradicts both),
**AMBIGUOUS** (G disagrees with A — automatic), **MATCH** (T agrees with G).
**GAP** is my added sub-class, defined and justified in §3.

## 2a. axes1's A5 cells

| cell | G | A | T | ruling |
|---|---|---|---|---|
| `a5c1` | `{anchor}` | `{anchor}` | `{anchor}` | **MATCH** |
| `a5c4` | `{anchor}` | `{anchor}` | `{anchor, cand, cand2}` | **VIOLATION CONFIRMED** |

G and A are identical on both cells. On `a5c4` two independent derivations agree
and truth contradicts both: the prereg's AMBIGUOUS trigger — *derivation
disagreement* — **did not fire**. See §4 for why this overrides axes1's own
self-imposed AMBIGUOUS grade, and why the Selection byte is not the reason.

## 2b. axes1's other two graded violations

I was not asked to re-derive these and did not freeze predictions for them.
Reported with the basis for each, and not counted as guard-confirmed on a
strength I do not have.

| cell/obj | A | T | ruling | basis |
|---|---|---|---|---|
| `a6c5` tu2.obj | `{anchor2, ?v@C@@UAAHH@Z}` | `{anchor2}` | **VIOLATION CONFIRMED (by transfer)** | my frozen `a9_04` derivation applies §2's propagation clause to a virtual call and adds the callee — the identical clause application, derived independently under quarantine, agreeing with A |
| `a8c5` user.obj | `{anchoru, ?ea@@YAHH@Z}` | `{anchoru}` | **VIOLATION CONFIRMED (weakest of the three)** | `ea` is a non-inline external out-of-line definition: a root under Reading A *and* Reading B, so no ambiguity risk exists for a guard to find |

## 2c. axes2's ten candidate violations

| cell | G | A | T | ruling |
|---|---|---|---|---|
| `a2_08` | 7: anchor, f, g, ~V, `??0`, `??0`copy, `??_G` | 8: same + `??4` | 5: anchor, f, g, ~V, `??_G` | **VIOLATION CONFIRMED** (reasoning corrected — §5c) |
| `a3_03` | 13 | 13 (identical) | 13 predicted **all present** + `??_ED@@W3AAPAXI@Z` | **MATCH on §2's domain + GAP** — demoted |
| `a3_04` | 9 | 9 (identical) | 9 **all present** + `??_DD@@QAAXXZ` | **MATCH on §2's domain + GAP** — demoted |
| `a3_05` | 9 | 9 (identical) | 9 **all present** + `??_DD@@QAAXXZ` | **MATCH on §2's domain + GAP** — demoted |
| `a3_06` | 16 | 16 (identical) | 16 **all present** + `??_DB`, `??_DC`, `??_DD` | **MATCH on §2's domain + GAP** — demoted |
| `a4_09` | 4: anchor, seed, helper, `??__E` | 4 (identical) | 3: anchor, seed, helper | **MATCH on §2's domain + GAP** — demoted |
| `a9_04` | `{anchor, ?f@D@@}` | identical | `{anchor}` | **VIOLATION CONFIRMED** |
| `a9_05` | `{anchor, ??1D}` | identical | `{anchor, ??1D, ??_GD, ?f@D@@, ?g@D@@}` | **VIOLATION CONFIRMED** |
| `a9_06` | `{anchor, ??1D}` | identical | `{anchor}` | **VIOLATION CONFIRMED** |
| `a9_07` | `{anchor, ?f@D@@}` | identical | `{anchor, ??_9D@@$BA@AA}` | **VIOLATION CONFIRMED** on the missing `?f@D@@`; the `??_9` extra is a GAP, not a second violation |

**Independence check, and it is the strongest thing in this table:** on 9 of the
10 cells my frozen derivation is **symbol-for-symbol identical** to axes2's,
produced without sight of it. The single difference is `a2_08`, where axes2 also
listed the implicit copy-assignment operator and I did not — same reading, one
extra symbol. §2's text is therefore *reproducibly applicable* on these
structures: the AMBIGUOUS class does not fire anywhere in axes2's set. That is a
result in §2's favour and should be reported as such.

**Mechanical re-grade (the lead flagged that `observed.json` has no verdict
field, so I am the only machine check on axes2's prose):** I recomputed
predicted-vs-observed for all ten from `PREDICTIONS.md` and `observed.json`
independently. **All ten of axes2's set-level comparisons are arithmetically
correct** — every "all N predicted present plus X" and every "predicted, absent"
claim checks out against the raw leader sets. The disagreement below is about
**classification**, never about axes2's data.

---

# 3. The demotion, and the pre-registered ground for it

Five of axes2's ten — `a3_03`, `a3_04`, `a3_05`, `a3_06`, `a4_09` — share one
property: **every symbol §2's text can produce was predicted correctly and is
present in truth.** The entire discrepancy is *extra* symbols drawn from
`??_D`, `??_E…W3`, `??__E`.

Calling that a "violation of §2-as-stated" scores §2 against a domain it never
claims. §2's thesis sentence is *"emission is a least-fixpoint reachability from
roots, computed over **kept definitions**"* — `??_D` and `??_E…W3` are not
definitions in any source; they are back-end closure symbols. §2 mentions
exactly one synthesized name (`??_G`) and only as a rider on the vtable rule.

**The prereg itself already carved out this exact set.** V5:

> F1 recomputed **excluding synthesized-name families** (`??_G ??_E ??_D ??__E
> ??__F ??_9` prefixes) … attribution only … the misses are the #152 synthesis
> cap

Every demoted discrepancy above is in that list — `??_E`, `??_D`, `??__E`. The
lane pre-registered a *separate metric* to isolate the synthesis families,
precisely because they measure the synthesis cap and not the predicate's model.
Scoring them a second time inside V6 double-counts what V5 exists to separate.
This is not a rule I invented after seeing the numbers; it is the prereg's own
partition applied consistently.

So the class is real and important — R3 must emit these symbols or refuse, and
axes2's Family 1 write-up is correct and valuable — but it is a **coverage
gap**, not a case of §2 predicting something false. The two must not carry equal
weight. This is the deflationary check finding something to demote, which per
the brief is the outcome to be *more* trusted than "nothing to demote".

**What survives the demotion is undiminished.** The five confirmed axes2
violations are all in the non-synthesized domain: ordinary source functions
(`?f@D@@`, `?g@D@@`) and ordinary destructors (`??1D`) predicted-and-absent or
present-and-unpredicted. Those are §2 being *wrong*, not §2 being silent.

---

# 4. Question 1 — axes1's count is **3**, and the Selection byte is not why

**Ruling: `a5c4` is a VIOLATION CONFIRMED. axes1's confirmed count is 3
(`a5c4`, `a6c5`, `a8c5`), not 2.**

### The argument, which does not use the leaked byte

The prereg's AMBIGUOUS class exists for one reason: to protect §2 when its text
is too loose for two readers to apply it in common, so that convicting it on one
reader's arbitrary choice would be unfair. That defence needs **some** reading of
§2 under which truth is correct. Here there is none:

* Reading B predicts `a5c1` emits `cand`. **Truth(`a5c1`) = `{anchor}`.**
  Reading B is not merely textually disfavoured — it is **empirically dead**.
* Reading A predicts `a5c4` emits nothing but `anchor`. Truth emits three.
  Reading A is dead too.
* A and B are the only readings either agent generated, independently, in
  quarantine.

Once Reading B is refuted by `a5c1`, Reading A is the **only surviving reading of
§2**, and under it `a5c4` is a straightforward false negative. There is no live
reading left to shelter it. The AMBIGUOUS grade would be sheltering §2 behind a
reading the lane's own data has already killed.

**`a5c1` — the clean, unleaked cell — carries this entirely.** The leak is
irrelevant to it.

### Why axes1 graded it AMBIGUOUS, and why that was the honourable error

axes1 registered, at `3401ffb`, a *rival reading* for `a5c1`/`a5c4` and bound
itself to grade AMBIGUOUS if truth matched it. It then followed that rule. But
the lane prereg's guard-1 trigger is **inter-agent derivation disagreement**, and
axes1's registered *primary* prediction for `a5c4` was `{anchor}` — identical to
mine. axes1 self-imposed a stricter, self-deflating rule than the prereg
required, and it lowered its own count by one.

**That is the guard mechanism working from the biased agent's own side, and it
should be credited as such.** An adversarial agent that declines to promote its
own cell when its own pre-registered rule says not to is exactly the behaviour
the prereg was trying to buy. The correction runs *against* the lane's declared
deflationary bias, and it is being made by the guard rather than by the agent
that would benefit — which is the intended division of labour.

### Is the Selection byte admissible? — argued, per my frozen pre-commitment

Pre-committed in `DERIVATION.md` §3.1 before Phase B: **admissible as
corroboration, never as an interpretation of §2, and never sufficient to convert
AMBIGUOUS to VIOLATION.** The reason is that §2's authority over its own words is
not delegated to c2's output. If it were, §2 could never be violated at all —
every disagreement would be re-read as "§2 must have meant whatever c2 did" —
which would dissolve this lane's own refutations along with everything else.
AMBIGUOUS is a claim about two derivations disagreeing over English; a byte in an
object file cannot repair a disagreement about English.

**And on inspection axes1's use of the byte is wrong in a way worth recording.**
axes1 argued: Selection = 2 means c2 classifies `cand`/`cand2` as COMDAT-linkage,
therefore they are "outside R1", therefore their emission is "unexplained by any
root clause". That inference presumes R1's biconditional — that roots and COMDATs
are complementary. The byte does not confirm that premise; **it refutes it.**
`cand` and `cand2` are simultaneously COMDATs (Selection 2) *and* emitted with no
referrer, i.e. roots. The correct conclusion is not "these are outside R1" but
**"R1's head clause is false as a rootness criterion: c2 has roots that are
COMDATs."** That is a sharper defect than the one axes1 named, and §6's repair
addresses it.

So the byte is admitted, and it is *not* load-bearing: the ruling stands on
`a5c1` alone.

---

# 5. Question 2 — does R1 admit a consistent reading? Plus three probes

**No. R1 as written admits no consistent reading — confirmed exactly as my
frozen §1.6 decision table specified in advance.** Truth is the **split** case:
`a5c1` emits no `cand`, `a5c4` emits both. Reading A explains `a5c1` and fails
`a5c4`; Reading B explains `a5c4` and fails `a5c1`. Registered before truth,
observed after: no reallocation of the goalposts was possible.

But axes1 stopped one step short, and its proposed repair says so of itself
("a hypothesis from two cells; it is not fitted and should not ship unprobed").
**c2's actual behaviour here is perfectly consistent** — the inconsistency is
purely textual, and the rule is simple once R1's conflation is undone.

### The conflation

R1 lists **"plain extern, `extern "C"`"** as co-equal root spellings. They are
not the same kind of thing:

* `extern` is a **storage-class specifier** on the function.
* `extern "C"` is a **language-linkage specification** — it changes name
  decoration and nothing about emission.

Only the first confers rootness, and it does so **even when the function is also
`inline` and therefore emitted as a COMDAT.**

### Three post-hoc probes of my own design, `work/emitpred/guard1/probes/`

Labelled post-hoc, outside the graded set, compiled at the workload flags
(`/O1 /Oi /EHsc /GS- /c`, X360 `16.00.11886.00` under wibo), CODE-characteristic
COMDAT leaders:

| probe | source (all `cand` unreferenced) | leaders |
|---|---|---|
| **P3** control (restates `a5c1`) | `extern "C" inline int cand(int){…}` | `{?anchor@@YAHH@Z}` — **not emitted** |
| **P1** | `extern "C" { extern int cand(int); }` **+** `extern "C" inline int cand(int){…}` | `{?anchor@@YAHH@Z, cand}` — **emitted** |
| **P2** | `inline int cand(int){…}` **then** `extern int cand(int);` | `{?anchor@@YAHH@Z, ?cand@@YAHH@Z}` — **emitted** |

**P1 vs P3 is a minimal pair differing by exactly one declaration.** Both are
`extern "C"`, both `inline`, both unreferenced; P1 adds a storage-class `extern`
declaration of the same entity and the symbol flips from absent to present. So
`extern "C"` is *not* what makes a root — the storage-class `extern` is. P2 shows
the effect is **entity-level, not order-dependent**: the `extern` declaration
works after the inline definition.

The rule now explains **every A5 cell (9/9), all three probes, and `a4_07`**:

| evidence | spelling | emitted? | rule |
|---|---|---|---|
| a5c1, P3, a5c6 `hciU` | `extern "C" inline`, unref | no | inline, no storage-class extern ⇒ COMDAT, not a root |
| a5c4 `cand`, `cand2`; P1; P2 | storage-class `extern` ∧ `inline`, unref | **yes** | storage-class extern ⇒ root, despite COMDAT |
| a5c7, a4_07 | `extern "C"` **non-inline**, unref | yes | non-inline external ⇒ root (the `extern "C"` is irrelevant) |
| a5c3, a5c9, a5c6 `hsiU` | `static inline`, unref | no | internal linkage |
| a5c2, a5c6 `hiR/hsiR/hciR`, a5c8 | inline, referenced | yes | propagation, not rootness |

**Is that a "consistent reading" of R1, rescuing it?** No — and this is the point
to be careful about. Any rule that treats `extern` and `extern "C"` *differently*
**contradicts R1's list, which names them together as co-equal**. So it is a
**repair of §2, not a reading of it.** §2-as-written remains without a consistent
reading; what the probes buy is that the repair is now fitted rather than
guessed.

---

# 6. Minimal proposed repair to §2

Replace R1's clause (1). New text, with every change forced by a cell or probe
above:

> **Roots:** (1) every definition that is **emitted out-of-line independently of
> any reference** — namely: (a) any external-linkage definition that is **not
> `inline`** (plain namespace-scope definitions, out-of-line member/static-member/
> virtual definitions, and definitions inside an `extern "C"` block — the
> language-linkage specification affects name decoration only and is **not**
> itself a rootness criterion); (b) any definition, **`inline` or not**, for
> which some declaration of the same entity carries the **storage-class
> specifier `extern`** (`extern` declaration + `inline` definition in either
> order, and the `extern inline` spelling) — such a definition is a root **even
> though it is emitted as a COMDAT**; and (c) anonymous-namespace
> **namespace-scope** functions not declared `static`.
>
> *Note: rootness and COMDAT-ness are independent axes. The COMDAT Selection
> byte records the latter and must not be used to infer the former — case (b)
> emits Selection-2 (ANY) COMDATs that are nonetheless roots.*

Three deletions and their justification:

1. **"external non-COMDAT linkage" leaves the head.** It is refuted directly:
   `a5c4`/P1/P2 are roots *and* Selection-2 COMDATs. This also removes the
   head-vs-list contradiction of §1, because rootness is no longer defined by a
   property the list's members can contradict.
2. **`extern "C"` is demoted from a root spelling to a decoration note.** P1 vs
   P3 is the minimal pair. This is the single most consequential word in the
   repair.
3. **"*any* out-of-line definition" is qualified with "not `inline`"** and given
   the case-(b) exception. Without this, §1's member-function intersection stays
   unresolvable.

Case (c) picks up axes2's independently-derived A4 repair (`a4_05`/`a4_06`).

**Not repaired here, deliberately:** the vtable rule and the propagation rule.
Their defects (§7) are separate clauses, and the repairs axes2 proposes for them
— "constructor **or destructor**", and a virtual/non-virtual split in
propagation — are correct in direction but are the subject of live violations
rather than of this clause.

---

# 7. Confirmed counts, and V6

## Confirmed violations, separately

| agent | graded VIOLATION by the agent | **guard-confirmed** | demoted to GAP | notes |
|---|---:|---:|---:|---|
| **axes1** | 2 (+1 AMBIGUOUS) | **3** | 0 | `a5c4` **promoted** from AMBIGUOUS (§4); `a6c5`, `a8c5` upheld |
| **axes2** | 10 | **5** | 5 | `a2_08`, `a9_04`, `a9_05`, `a9_06`, `a9_07` upheld; `a3_03/04/05/06`, `a4_09` demoted (§3) |
| **total** | 12 | **8** | 5 | one promotion, five demotions |

## V6

Registered: *"axes (of 9) with ≥ 1 confirmed violation of §2-as-stated"*, point
**1**, interval **[0, 3]**.

| axis | agent | broken? | on what |
|---|---|---|---|
| A1 header depth | axes1 | no | 8/8 MATCH |
| **A5** linkage crossings | axes1 | **BROKEN** | `a5c4` (§4) |
| **A6** multi-TU shared header | axes1 | **BROKEN** | `a6c5` — virtual call ≠ ODR-use of callee |
| A7 pragma roots | axes1 | no | 10/10 MATCH |
| **A8** PCH | axes1 | **BROKEN** | `a8c5` — weakest of the three, see §2b |
| **A2** templates | axes2 | **BROKEN** | `a2_08` |
| A3 virtual/MI | axes2 | **not broken by the 4 cells I checked** | all four demoted to GAP; **status open**, see below |
| A4 anon namespaces | axes2 | **not broken by `a4_09`** | demoted to GAP; **status open**, see below |
| **A9** vtable w/o ctor | axes2 | **BROKEN** | `a9_04/05/06/07` |

**V6 = 5 of 9.** Registered interval [0, 3] — **V6 lands outside its registered
interval on the high side**, and it does so *after* the most deflationary
accounting I can defend (five demotions). The registered point estimate of 1 was
too optimistic by a factor of five.

**V6 = 5 is a lower bound, not an estimate.** Six cells are graded AMBIGUOUS by
axes2 on the *same* self-imposed rule I overrode for `a5c4` in §4 — the agent
registered a rival reading in advance and declined to promote when truth matched
it: `a2_04`, `a3_01`, `a3_02`, `a3_08` (the `??_G`-when-the-destructor-is-not-
virtual over-statement), `a4_05`, `a4_06`. **Consistency demands I flag that the
`a5c4` reasoning may promote several of them**, which would break A3 and A4 and
put V6 at **7 of 9**. I have not promoted them: guard 1 may not confirm a
violation without an independent frozen derivation, and I was not asked to
derive those six. **This is the single highest-value follow-up in the lane** and
it points the opposite way from my demotions, which is why it belongs in the
headline rather than a footnote.

## Question 3 — A6 and A9: two axes, and the double-counting worry dissolves

My frozen pre-commitment (`DERIVATION.md` §3.2) was: V6's registered unit is
*axes*, it feeds a **ship** gate, a broken axis is unsafe regardless of whether
its cause was seen elsewhere, and redefining a registered unit after seeing that
two axes share a cause is a goalpost-move that happens to lower the score. That
stands.

**But Phase B makes the argument unnecessary, because the premise is false.**
A9 is not merely a replication of A6. Its four confirmed violations contain
**two opposite mechanisms**:

* `a9_04`, `a9_06`, `a9_07` — propagation **over**-predicts: a virtual call, a
  virtual `delete`, and a pointer-to-virtual-member reference the vtable *slot*,
  not the definition. This is axes1's A6 mechanism, independently replicated.
* **`a9_05` — the vtable rule **under**-predicts**: an out-of-line virtual
  destructor definition forces the whole vtable closure (`f`, `g`, `??_G`) with
  **no constructor anywhere in the TU**. §2 says "a kept **constructor**".
  `a2_08` shows the same trigger defect from a second direction (an explicit
  instantiation forces `??_G` with no constructor emitted). **axes1 never found
  this** — its A6 cell had a kept constructor and it reported the vtable rule
  *confirmed*.

So A9 is independently broken by a mechanism A6 never touched, and A6/A9 count
as two axes **without any double-counting**. The correct summary is:

* **Two axes, three mechanisms** — over-inclusive propagation (A6 + A9),
  under-inclusive vtable trigger (A9 + A2), and R1's linkage conflation (A5).
* The over-inclusive mechanism was **found by axes1 and independently replicated
  by axes2 on different structures with no sight of axes1's work**. That
  replication *raises* confidence the mechanism is real while *lowering* the
  number of distinct repairs §2 needs. Both halves of that sentence must travel
  together; a bare "5 of 9 axes broken" headline conceals it.
* Mechanisms 1 and 2 are **mirror images and must be repaired together** — as
  axes2 correctly says, in `a9_05` the vtable is kept so everything comes along;
  in `a9_06` it is not kept so nothing does, including the destructor that was
  syntactically called.

---

# 8. Where the first agent was too eager — and where it was not eager enough

The brief warns that finding nothing to demote is a result to be suspicious of.
I found five demotions, one promotion, and two corrections of reasoning.

### Too eager (demoted)

1. **`a3_03`, `a3_04`, `a3_05`, `a3_06` — graded VIOLATION; every §2-reachable
   symbol was predicted exactly right.** 13/13, 9/9, 9/9, 16/16 present. The
   sole discrepancies are `??_E…W3` and `??_D`, both in the prereg's own V5
   exclusion list (§3). These four cells are among the **strongest confirmations
   of §2's vtable closure in the entire lane** — the wide "every virtual of C
   including inherited" reading, the per-base-subobject application, and the
   `??_G` rider all held across MI, virtual bases, and a diamond — and they are
   currently filed as refutations.
2. **`a4_09` — graded VIOLATION over a single `??__E` symbol.** The function set
   is exactly right, and the cell is a *strong positive* for §2: the dyninit
   thunk was folded away by the optimizer, yet `helper` (a `static` reachable
   only through that thunk) and `seed` are both emitted anyway. **The thunk's
   propagation survives the thunk's own elimination** — which is §2's
   pre-optimization clause working precisely as written, on the hardest case
   available. axes2 records this corroboration in its own Family 4 write-up and
   still files the cell as a violation. The clause is under-specified (§2 gives
   no rule for which initializers produce a thunk); that is a gap, not a false
   prediction.

### Reasoning corrected (verdict survives)

3. **`a2_08` — right verdict, partly wrong ground.** axes2's stated basis is
   that three implicit special members were predicted and are absent. But
   [temp.explicit] instantiates only members **defined at the point of
   instantiation**; implicitly-declared special members are not, unless
   odr-used. So R2's "including never-referenced members" never reached them,
   and **that over-prediction is a shared C++-semantics error — mine as much as
   axes2's** (my frozen derivation made it too, and recorded the exclusive
   sub-reading as a registered risk). Under the corrected reading §2 predicts
   `{anchor, f, g, ~V}` and truth adds `??_G` **with no constructor kept** — so
   the cell is still a VIOLATION, on the Family-2 trigger defect, which axes2
   also identified. Robust to the correction, but the write-up should lead with
   the surviving ground.
4. **`a9_07` — "two independent directions" is one violation and one gap.** The
   missing `?f@D@@UAAHH@Z` is a genuine refutation of P's "an address-take" for
   virtuals. The extra `??_9D@@$BA@AA` is V5-family coverage. Both are real;
   they are not both violations.
5. **`a8c5` (axes1) — upheld but ranked weakest.** It is a violation of §2 read
   as a *source-side* root enumeration; c2 itself never saw the definition, so
   c2's emit behaviour is not contradicted. axes1 says exactly this in its own
   "Scope honesty" paragraph, so this is transparency rather than over-eagerness
   — but a reader counting "violations of the emit predicate" should know one of
   the three is about definition ownership under `/Yu`, not about emission.

### Not eager enough (promoted, and one flagged)

6. **`a5c4` — promoted AMBIGUOUS → VIOLATION** (§4). The deflationary lane
   under-called its own strongest linkage result.
7. **Six axes2 AMBIGUOUS cells may be under-called on the same self-imposed
   rule** (§7). Unadjudicated for want of a guard derivation; flagged as the
   top follow-up because it would take V6 from 5 to 7.

**Net effect of the guard: −5 demotions, +1 promotion, and V6 moves from
(axes1 2 + axes2 4 = 6 axes) to 5 — still outside the registered [0, 3].** The
demotions do not rescue §2; they relocate half of axes2's findings from "§2 is
wrong" to "§2 is silent", which is a different repair with a different owner
(#152's synthesis cap, not #161's model).

---

# 9. What I did not check

* No independent derivation exists for `a6c5`, `a8c5`, or the six axes2
  AMBIGUOUS cells; §2b and §7 say so at each use.
* The A1, A7, and the 19 axes2 MATCH cells were not re-derived. A guard that
  only re-derives claimed violations cannot detect a **missed** violation — a
  cell graded MATCH in error would be invisible to this process. That asymmetry
  is inherent to the prereg's guard-1 design and is worth recording as a
  limitation of the guard, not of the agents.
* Truth was read only through `observed.json`, `results.json`, `grades.json` and
  my own three probe objs. No instruction byte was read anywhere.

**Artifacts:** `work/emitpred/guard1/DERIVATION.md` (frozen, `75bf3bb`), this
file, and `work/emitpred/guard1/probes/{p1,p2,p3}*.cpp` with objs under
`probes/out/` (build products — not to be committed). Nothing under `crates/`
or `docs/` was touched. No git command was run by me.
