# RULING_ADDENDUM — corrections to `RULING.md` §7/§8

    Lane:    w-emitpred
    Amends:  work/emitpred/guard1/RULING.md, committed at 9e22d3a
    Author:  lane lead (w-emitpred), correcting the lead's own instructions
    Status:  the original RULING.md and DERIVATION.md are NOT edited.

**Why this is a separate file.** `RULING.md` and `DERIVATION.md` are frozen,
committed artifacts. A freeze that can be revised after the fact is not a
freeze, and this lane's entire Part-2 discipline rests on committed-before-look
ordering. So the correction lives here and the record shows both documents.
Anyone quoting `RULING.md` §7/§8 must read this file alongside it.

---

## 1. The "refile as confirmations" instruction was WRONG

**The error is the lead's, not guard 1's.** I instructed guard 1 to move
`a3_03`/`a3_04`/`a3_05`/`a3_06` and `a4_09` out of the violations section and
into the confirmations section. That instruction, followed literally, would have
caused **the mirror image of the error guard 1 had just caught** — deleting a
real finding in the direction that flatters §2, having just corrected a filing
that flattered the lane's thesis.

### The correct grade is two-part

> **MATCH on §2's domain **+** GAP.**

Both halves are real, **both must travel together**, and they have **different
owners**:

| half | what it says | owner |
|---|---|---|
| **the confirmation** | §2's vtable closure **held** — the wide "every virtual including inherited" reading, per-base-subobject application, and the `??_G` rider — across multiple inheritance, virtual bases, and a diamond. `a4_09` confirms the pre-optimization clause on the hardest available case: the dyninit thunk's propagation **survives the thunk's own elimination**. | **#161** |
| **the gap** | c2 emits code COMDATs that **§2's text has no clause capable of producing**: `??_DB@@QAAXXZ`, `??_DC@@QAAXXZ`, `??_DD@@QAAXXZ` (virtual-base destructors), `??_ED@@W3AAPAXI@Z` (the adjustor form of the vector-deleting destructor), and on `a4_09` a `??__E` thunk that c1xx **names in the `.gl`** and c2 then does **not** emit. | **#152** |

Dropping the gap half would delete a **coverage** finding. **R3 must emit those
symbols or refuse**, and that is outstanding work — it does not become
unnecessary because §2's vtable rule was vindicated on the same cells.

### Guard 1's §8 line, to be carried verbatim wherever these cells are quoted

> **The demotions do not rescue §2; they relocate half of axes2's findings from
> "§2 is wrong" to "§2 is silent."**

"Wrong" and "silent" are different defects with different repairs. Neither is
"fine".

### Counting correction, also the lead's

I wrote "**two** demoted cells", which reads as two cells. It is **two groups,
five cells**: `a3_03`, `a3_04`, `a3_05`, `a3_06` (one group) and `a4_09`.

---

## 2. The contamination reasoning was wrong in mechanism, right in conclusion

I disqualified guard 1 for the six unpromoted cells (`a2_04`, `a3_01`, `a3_02`,
`a3_08`, `a4_05`, `a4_06`) on the grounds that its mechanical cross-check
recomputed predicted-vs-observed across ten cells. **That recompute never
touched the six.** The reasoning was wrong.

**The actual disqualifier:** guard 1 **read `axes2/RESULTS.md` in full**. That
file's per-cell table carries predicted/observed **cardinalities** and verdict
notes for **all 35 cells**, and names specific absent symbols.

### The correct rule, now operative for every guard in this lane

> **Anyone who has read `axes1/RESULTS.md` or `axes2/RESULTS.md` is disqualified
> for _every_ cell in that agent's grid — not merely the cells they
> recomputed.**

The conclusion stands unchanged: guard 1 is disqualified for the six, and
`guard2` must derive them. But the rule is broader than the one I applied, and
the broader form is what protects future work.

### Applied to guard 2

`guard2`'s brief already forbade both `RESULTS.md` files, `observed.json`,
`il_names.json`, `PREDICTIONS.md`, `spec.json`, everything under `axes1/` and
`guard1/`, `MAGNITUDE.md`, and `docs/PHASE7_VALIDATION.md`.

**Provenance audited by the lead before guard 2 derived anything.** Every file
it had read: `CLAUDE.md`, the six `cell.cpp` sources, `docs/PHASE7_PLAN.md`, the
prereg. Its one directory-walk used `find -type f` and **listed** filenames
without reading them, so `spec.json` was never opened. **Clean.** The broader
rule was then sent to it explicitly, together with a request that it flag any
sentence of the lead's that entails a symbol's presence — since **a Selection
byte, a section assignment, a size, or a checksum all do**, and that is exactly
how the lead leaked into guard 1.

---

## 3. V6 must be reported BOTH ways — and the robustness is the finding

**This is the most valuable of the three corrections.** Guard 1 exposed the
single interpretive call that moved its own headline number **up**. That is
precisely what a guard is for, and it should be published, not resolved
silently.

| reading | V6 | basis |
|---|---:|---|
| **guard 1's** | **5** | the prereg's AMBIGUOUS clause triggers on **inter-agent** disagreement, and axes1's registered *primary* prediction was identical to guard 1's — so A5 is broken |
| **the hostile alternative** | **4** | a pre-registered **rival** reading counts as a derivation disagreement, so A5 is **unbroken**; A6, A8, A2, A9 remain |

### Lead with the robustness, not with either number

**Both exceed the registered interval [0, 3].**

So the headline — **the pre-registration missed on the high side** — is
**robust to the one interpretive call that inflates it**. That robustness is
worth more than the extra point. **A result that survives its own author's most
hostile reading is the strongest thing this lane can publish**, and it is a
stronger claim than V6 = 5 asserted flatly.

The registered point estimate was **1**. The interval was **[0, 3]**. Under
every reading this lane can defend, the true value is **at least 4**, and it
remains a **lower bound** for the two reasons in `RULING.md` §9 and
`PHASE7_VALIDATION.md` §6d: six cells are unpromoted pending guard 2, and the
guard's construction is **asymmetric** — it re-derived only *claimed*
violations, so **a cell graded MATCH in error is structurally invisible to it**.

---

## 4. What is unchanged

Everything else in `RULING.md` stands: the a5c1 argument and its immunity to the
lead's leak; the Selection-byte correction (**R1's head is false as a rootness
criterion — c2 has roots that are COMDATs**); the "c2 is consistent, the
inconsistency is purely textual" reframing and the `p1`/`p3` minimal pair that
isolates it; the storage-class-`extern` vs language-linkage-`extern "C"` repair,
with its caveat that this is a **repair, not a reading**, and §2-as-written
still has none; V6 counting **axes, not mechanisms**; and the REACH/GAP
distinction carried into scoring.
