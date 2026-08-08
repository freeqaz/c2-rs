# w-phase7 — PRE-REGISTRATION

    Lane:    w-phase7, 2026-08-08, worktree `wt-w-phase7` off master `b027eaad`
    Brief:   implement `rungs/_2026-08-04-w-emitp-findings.md` §6, five steps.
    This file is committed BEFORE the first probe. Scored in the rung's §7.

---

## 0. WHAT I READ BEFORE WRITING THIS, AND WHY IT CHANGES THE LANE

Disclosure first, because it is the thing that would otherwise look like a
post-hoc reframing.

Before writing this prereg I read `CLAUDE.md`, `docs/STATUS.md`, the spec
(`docs/rungs/_2026-08-04-w-emitp-findings.md`), and then **the tree**, to find
where §6's five steps would go. The tree says something the brief does not:

* `crates/c2-il/src/func/glalias.rs` **exists**, 585 lines, and is exactly §6
  step 1 + step 2 — the tag-0x10 decode, the RT+BIND gate, the shifted null, the
  `GlAliasTable`, and a `GlAliasStats` carrying every invariant §6 step 2 asks
  to be asserted.
* `docs/whitebox/DISCLOSURE.md` carries rows **W-ALIAS-1** and **W-ALIAS-2**,
  naming `0x10b9c01e`, `0x10b9c024`, `0x10b9c030`, `0x10b99621`, `0x10b99635` —
  exactly §6 step 5's five addresses. Landed at `d2bdadc`.
* `crates/c2-il/tests/gl_alias_corpus.rs` exists and dumps the Rust table per TU
  for comparison against `work/w-emitp/alias.py`.
* The module's own docs say, in as many words: **"There is no consumer in
  `crates/` today."**

So **steps 1, 2 and 5 are SHIPPED before this lane starts**, by lane `w-alias`,
and the brief's headline sentence — *"the spec … is written, complete, and
unimplemented"* — is **stale on 3 of its 5 steps**. What is genuinely
unimplemented is **step 3** (apply the resolution once, at the `in` `02`-node
resolution site) and **step 4** (never emit a name in `dom(alias)`), because
both need a *consumer* and there is none.

I also read the auto-memory note
`emit-predicate-is-the-binding-constraint.md`, which records that lane `w-quar`
spent the one-shot 21-TU gate on `JFP_ALIAS` (**passed, +0 TU reach**) and lane
`w-root` shipped the root rule out of sample (**TU reach 26 → 31 of 31,
saturated**) — both entirely in `work/*.py`, **zero `crates/` diff** (checked:
`git show --stat f57fe61e` touches only `work/w-root/` and docs).

**This lane's registered scope, given that:**

1. Re-derive steps 1/2/5's status **in the tree**, and report them as inherited
   rather than as this lane's work.
2. Find and measure the **`in` `02`-node resolution site that already exists in
   `crates/`** — `IlBundle::data_tu`, which turns a tag-02 `InSymbolRef.target`
   token into a **`.data` relocation's symbol name**. If that site can ever see
   an alias, the port emits a relocation naming `??_E<X>` where c2 names
   `??_G<X>`: **a wrong obj out of an accepted TU, board #232's exact shape.**
   This is step 3 at the only site that exists, and it is obj-affecting.
3. Ship steps 3 and 4 at that site, fenced, with the invariants asserted rather
   than assumed.
4. Publish the invariants and the live-population counts as `gap-metric` keys
   measured **in Rust over the 878-TU workload against the real objs** — the
   Python measured 850 TUs and never joined `dom(alias)` against `E` inside the
   harness.

**What I am NOT registering as in scope, stated now so absence cannot read as
success:** a full Rust port of `JFP_ALIAS` + the root rule (`work/w-root/
rootmodel.py`), which is what a *per-TU exact* figure measured by me would
require. See §4 and the decline clause D3.

---

## 1. WHAT I EXPECT — the questions, with points and intervals

Registered **before** running anything. Where a figure is inherited from
w-emitp's Python over 850 TUs, I register the Rust figure over the workload's
878 (871 graded, 7 capture-fail) as a *separate* measurement, because the
populations differ.

### Q1 — the inherited steps

| # | registered **point** | interval | why |
|---|---|---|---|
| **S1** | steps 1, 2, 5 already shipped: **3 of 5** | — | read from the tree, §0. Not a prediction; a disclosure. |
| **S2** | `git diff b027eaad -- crates/` at this lane's tip is **NOT** 0 bytes | — | steps 3/4 need a consumer, and I intend to ship one |

### Q2 — THE LIVE-HAZARD QUESTION (the one that could be a mismatch)

`IlBundle::data_tu` resolves every tag-02 `InSymbolRef.target` to a COFF name
and emits a `.data` relocation naming it. It runs **only when the TU defines no
functions** (`crates/c2-core/src/lib.rs:388`).

| # | registered **point** | interval | direction of error |
|---|---|---:|---|
| **H1** | TUs on the 878-TU workload where `data_tu` returns `Some` **and** any emitted relocation's target is in `dom(alias)`: **0** | [0, 3] | I expect **0**: an alias is a `??_E` deleting destructor, named only from a **vftable** initializer, and a vftable is a `.rdata` COMDAT that a functionless-TU `.data` writer should never carry. If it is **> 0** this lane has found a live wrong emit. |
| **H2** | `.in` tag-02 symbol references anywhere on the workload whose target token binds to a name in `dom(alias)` — **regardless** of whether any writer consumes them: **60 000** | [1 000, 400 000] | this is the *reachable* population, i.e. how much is one `functions()` widening away. w-emitp counted 96 220 alias records over 850 TUs and says a vftable's initializer names the alias, so this should be large. **A 0 here would refute the channel's own mechanism** and I would report that as the headline. |
| **H3** | TUs with ≥ 1 such reference: **700** of 878 | [100, 878] | |

**H1 and H2 are deliberately opposed.** H2 large + H1 zero is *"the hazard is
real and the port is fenced out of it by an unrelated refusal"*, which is the
honest thing to publish and is **not** a reason to skip the fix — the fence is
`funcs.is_empty()`, which no rule ties to aliases.

### Q3 — the decode's invariants, re-measured in Rust over the workload

§6 step 2 names five invariants and says two are not 1.0. Registered against
w-emitp's 850-TU Python figures; the interval allows for the different
population.

| # | invariant | Python (850 TUs) | registered **point** (Rust, 878) | interval |
|---|---|---:|---:|---:|
| **I1** | tag-0x10 records | 96 220 | **99 000** | [80 000, 120 000] |
| **I2** | bound / tag10 | 0.99584 | **0.9958** | [0.980, 1.000] |
| **I3** | `head_fail` | 352 | **360** | [0, 2 000] |
| **I4** | `rt_fail` | 0 | **0** | [0, 50] |
| **I5** | unbound target / tag10 (**"target binds 0.99950"**) | 0.00050 | **0.0005** | [0.0000, 0.0100] |
| **I6** | self-alias | 0 | **0** | [0, 0] — **a decline clause, D1** |
| **I7** | duplicate | 0 | **0** | [0, 20] |
| **I8** | `dom_with_body` (⊇ `dom(alias) ∩ U`) | 0 | **0** | [0, 0] — **a decline clause, D2** |
| **I9** | shape `??_E`→`??_G` / bound (**"target in U 0.99998"** is its sibling) | 0.99998 | **0.9999** | [0.990, 1.000] |
| **I10** | `dom(alias) ∩ E` — aliases that ARE emitted, over the real objs | 0 of 174 417 | **0** | [0, 0] — **a decline clause, D2** |
| **I11** | alias targets that are emitted | 3 945 | **4 000** | [1 000, 20 000] |

**The two non-unit invariants and what I will do about them, decided now:**

* **target does not bind (0.00050 of records).** The reader **drops the
  record**: no entry in the table, counted in `unbound_target`. A dropped alias
  makes the consumer resolve a name to itself, which is the *incumbent*
  behaviour and therefore fails toward the status quo, never toward a new wrong
  name. This is what `glalias.rs` already does and I am not changing it.
* **target not in `U` (0.00002).** The table **keeps** it. A consumer of step 3
  resolves the name and then the *writer* decides whether it can place the
  result; a reader that dropped it would silently restore the alias's own name,
  which is the one name §6 step 4 forbids. So: **resolve always, place never
  blindly.**

### Q4 — per-TU exact

| # | registered | |
|---|---|---|
| **P1** | I will **not** produce a Rust per-TU-exact figure over the corpus unless Q2/Q3 land early and cheaply. | see D3 |
| **P2** | If I do, the point is **472 of 850**-equivalent for the ceiling and **308** for `JFP_ALIAS`; but the honest expectation for a *first* Rust reimplementation is **below** both, because `refs.scan`'s terminus gate and `marks.parse_records`'s sequential `.in` parse are two more decoders to get exactly right and `ininit.rs`'s anchored scan is silent about 43.7 % of the `.in` stream (board #961). Registered point if attempted: **250 of 850**, interval [0, 480]. | |

### Q5 — the metrics, base vs tip

| # | registered **point** |
|---|---|
| **M1** | TU **match 11 → 11**. A model of the emit set is not a TU conversion; §5 of the spec moved match by zero and said so in its first line. |
| **M2** | **mismatch 0 → 0.** If it moves the lane is wrong and reverts. |
| **M3** | `codegen-gap 0 → 0`, `capture-fail 7 → 7` |
| **M4** | `vocab-gap 860 → 860` — no reader widening that changes what `functions()` accepts is in scope |
| **M5** | `gap-metric` diff: **new keys only, zero existing keys change value.** Any existing key that moves is an alarm and is reported as one. |
| **M6** | test count **1216 → 1216 + (10..30)** |
| **M7** | `scripts/gate.sh --require-graded` **18/18 PASS, 0 mismatch** |

---

## 2. THE CONTROLS

1. **The null is already shipped and I will use it, not describe it.**
   `gl_alias_table_shifted(gl, ±1)` — every corpus figure in Q3 is reported with
   its `p−1` / `p+1` counterpart. A field position quoted without its null is a
   field position that was searched for.
2. **A known-answer arm against the Python.** `crates/c2-il/tests/
   gl_alias_corpus.rs` already dumps the Rust table per TU; w-alias compared it
   name-for-name against `work/w-emitp/alias.py` on 850. I re-run the Rust side
   and assert the aggregate reproduces w-alias's committed dump **to the digit**
   if that dump is in the tree; if it is not, I say so rather than inventing an
   agreement.
3. **Counts, never statuses** (`docs/STATUS.md` trap 5). Every new `gap-metric`
   key is a count and prints its zero.
4. **A denominator on both sides of the change** (trap 0 / board #1002). H2's
   population is printed **before** step 3 lands and again after, and they must
   be equal — step 3 changes which *name* comes out, never how many refs there
   are.
5. **The `dom_with_body` guard is a precondition, not a comment.** Step 4
   suppresses an emit; if `dom_with_body > 0` on any TU, suppressing would
   delete a symbol that has a body. The consumer asserts it per TU and refuses
   the TU rather than suppressing.

---

## 3. DECLINE CLAUSES

* **D1 — self-alias > 0 anywhere.** Then "apply once, not transitive" is not
  established on this corpus and step 3 does not ship. Report the count.
* **D2 — `dom_with_body > 0`, or `dom(alias) ∩ E > 0`, on any TU.** Then step 4
  would suppress a symbol c2 emits. Step 4 does not ship as an unconditional
  rule; it ships as a per-TU guarded rule or not at all. Report the TUs by name.
* **D3 — the Rust emit-set model.** If Q2 + Q3 + the fix + the gate have not
  landed with time to spare, I do **not** start a Rust port of `rootmodel.py`.
  A half-ported fixpoint that produces a per-TU-exact number nobody can
  reproduce is worse than a named absence. I will report it as **not computed**
  and name what would compute it.
* **D4 — `mismatch` moves off 0.** Revert, and the revert plus its reasoning is
  the lane's result.
* **D5 — no extrapolation of reach.** `|{TU : model exact} ∩ B∧C|` is not
  computable from anything this lane owns and **will not be estimated**, per the
  spec's §5 and the brief's standing instruction. Lane `w-bcgap` owns it.

---

## 4. WHAT I ALREADY KNOW I WILL NOT MEASURE

Named here so that absence never reads as success:

1. `|{TU : model exact} ∩ B∧C|` — D5, w-bcgap owns it.
2. The instruction that turns `+0x20 & 0x2000` into the COFF Mark bit
   (`0x10b28ca3`) — named by w-emitp, not decoded there, not decoded here.
3. `0x10b8ac60`, the second reader of the alias bit.
4. The 510 outside-`U` emitted names on 162 TUs (spec §3.2) — the *next*
   channel, and not this one.
5. The 798 `$`-class residual names.
6. Order. A right set in the wrong order is still a mismatch.
7. Whether any of this holds off the workload's `/O1 /EHsc /GR`.
8. The `.gl` **reference list** decode (`work/w-refs/refs.py`) — needed for a
   Rust emit-set model, and it carries its own un-disclosed disassembly
   (`0x10b9bf99`, `0x10b276e4`, `0x10b9be44`). Adopting it would need new
   `DISCLOSURE.md` rows. Not adopted here.

---

## 5. THE ESTIMATE-STREAK NOTE (board #770)

The streak is ~10 optimistic / 2 pessimistic / 1 hit, and optimistic misses are
the pattern. The two places I am most likely to be optimistic here:

* **H2 / H3** — I have registered a large reachable population on the strength
  of one sentence in a findings doc ("a vftable's initializer names the alias").
  The `.in` reader in `crates/` is the **anchored** scan, which is silent about
  43.7 % of the stream, so the Rust number can be far below the Python's for a
  reason that has nothing to do with the channel. Registered interval bottoms
  at 1 000 for exactly that reason, and if it comes in low I will report it as
  a *reader* result, not a *channel* result.
* **The scope itself** — D3 exists because "and then port the model to Rust" is
  the optimistic clause of this brief, not of my estimate.

Where I expect to be **pessimistic**: H1. I have registered 0 and would not be
astonished by a nonzero, because every previous "the port is fenced out of that
class" sentence on `STATUS.md` has been retracted by someone widening an
instrument.
