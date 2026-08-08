# WB CAMPAIGN 2 (2026-08-08) — the GENERATORS, not the outputs

> **PROVENANCE — DISASSEMBLY-DERIVED.** Same footing as
> [`CAMPAIGN_2026-08-08.md`](CAMPAIGN_2026-08-08.md): lanes read the static
> disassembly of Microsoft's `c2.dll` (image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 — verify the sha256 before quoting
> any VA). Everything produced is **navigation** until it earns a
> [`DISCLOSURE.md`](DISCLOSURE.md) row, and **the obj is the sole judge** of
> every reading (§7, the `.bss` retraction).
>
> **Standing rules 1–5 of CAMPAIGN_2026-08-08.md apply verbatim to every lane
> here.** They are not repeated; read them before starting.

## Why this campaign exists

Campaign 1 (wb-reader / wb-frame / wb-memcpy) went 3-for-3 on its success
floors and redirected the program: the frontier converts in the **emitter**
(ROADMAP §10.24). The redirect bought six TU conversions in one day after 24
flat rungs. But every one of those conversions is a **transcription** — a
bespoke recognizer plus a hand-derived word sequence for one function shape —
and `CEILING.md`'s arithmetic stands: converting the entire remaining frontier
lands at `A∧B∧C` ≈ 27, and the rest of the distance to 871 is function-byte
codegen with no leverage multiplier yet identified.

The user's directive (2026-08-08, verbatim intent): *keep going to the source
of truth; work on the factors themselves; extract the correct **shape of the
code** from the actual binary.* Transcription reads c2's **outputs** one at a
time. This campaign reads c2's **generators** — the algorithms that produce
those outputs — so that one confirmed reading converts a *class*, not a TU:

- **Register choice and instruction order** are the reason transcriptions
  don't generalize. If the policy is readable and obj-confirmed, a shipped
  class becomes a *derivation* and the reach-pool (124 TUs in `B∧C ∖ A∧B∧C`)
  becomes addressable in bulk.
- **The inliner** is priority 2 of §10.24 and the named blocker of the
  frontier's largest TU (`keygen_xbox`, #151, #1477-as-retracted).
- **EH (factor D)** is the highest-worth single row on the frontier
  (`Main.cpp` — factor D over 740 objs) and its reader chain is STUCK at 2,
  which black-box laddering cannot price further.
- **The one-witness-per-side choosers** (#1767) block `mmio` and `Biquad`
  *by the project's own evidence rule* — a rule that a **mechanism read from
  the binary and then obj-confirmed** satisfies where a 2-point fit cannot.

The whitebox method is unchanged and validated: disassembly names the
mechanism → black-box grid frozen **before** the first `cl.exe` confirms →
`DISCLOSURE.md` row on adoption. Navigation is free; adoption is not.

## Board ranges

#1800–#1819 belong to the running conversion lane w-json. This campaign owns
**#1820–#1899**, split per lane below. Rows not minted are declared unminted
in each lane's rung.

## Lane WB-D — `wb-regalloc`: register choice and instruction order

**Question.** What policy assigns registers and orders instructions in c2's
PPC emitter? This is "the shape of the code" — the single reading that would
turn transcription into derivation.

**Ground-truth anchors (all already measured, all free re-checks):**
- Every shipped transcription class matched c2's register choice and word
  order exactly once the words were copied — so the policy is deterministic
  and, on straight-line + simple-CFG bodies, apparently a stable walk.
- w-osfinfo: the scratch register is the key (r11 vs r10 broke a walk keyed on
  r11 alone); pairs close before the next opens (interleaving REFUTED).
- w-xlr: comparison signedness flows from the IL TYPE byte to
  `cmpwi`/`cmplwi` (#1788) — type reaches the selector; where is it consumed?
- The `/FAsc` listing seam: c2 narrates its own blocks and label counter —
  the listing writer's xrefs lead back into the emitter's own structures.

**Firm deliverables.**
1. Locate the emitter stages: instruction selection, register assignment,
   and final ordering (W-EMIT label cluster and the listing writer's xrefs
   are the entries), with VAs.
2. Read the **register-choice policy**: allocation order, scratch selection,
   argument-register assignment, CR-field choice, and what state makes r10
   appear where a shipped walk expected r11.
3. Read the **ordering policy**: is emitted word order a deterministic
   traversal of an IR the reader builds? Name the traversal.
4. **Obj-check against captured ground truth**: pick ≥3 functions OUTSIDE
   every shipped class (frontier or reach-pool), predict register assignment
   and instruction order from the reading alone — frozen before grading —
   and grade against the reference disassembly. Misses are retractions, not
   hedges.
5. The judgment deliverable: a written, specific answer to *"can a general
   lowering be derived for a class (loops, multi-way ifs) rather than a TU?"*
   — what it needs, its predicted reach over the reach-pool, and the first
   class to attempt. "No, because X" with X specific is also a result.

**Success floor**: either one policy reading (register OR order) survives a
frozen check on ≥1 function outside every shipped class, or a written finding
of why the policy is not readable (e.g. it is an artifact of IR construction
order spread across the reader).

**Seams**: `docs/whitebox/` (new `WB_REGALLOC_FINDINGS.md`) + `work/wb-regalloc/`
+ its rung + board rows **#1820–#1839**. Does not touch `crates/`.

## Lane WB-E — `wb-inline`: the inliner decision function

**Question.** When does c2 inline a callee? Ground truth anchor:
`?supershuffle@@YAXPAD@Z` (`keygen_xbox.cpp`) — c2 emits it with the callee
inlined; the port's honest answer is 21 words vs c2's 26+ (board #1477 as
retracted: the frame reading was wrong, the **inliner** is the defect).
§10.24 names the inliner priority 2.

**Firm deliverables.**
1. Locate the inline decision (the memcpy lane's dispatcher/option-word
   findings are prior art — `0x10c2e310` reads option-word bit 23 for
   favor-speed; the inliner plausibly consumes the same word), with VAs.
2. Read the **decision function**: size thresholds, callee properties
   (leaf? address-taken? varargs?), option-word bits, call-site properties.
3. An **obj-check grid frozen before the first `cl.exe`**: every rival
   predicate's per-cell prediction written down, with an asserted minimum of
   discriminating cells (w-clear's confound check in advance).
4. The specific answer for `?supershuffle`: which clause fires, and what the
   port would need (a real inliner pass vs. an inlined-body transcription
   license) — with the honest cost of each.
5. Pre-drafted DISCLOSURE rows for anything obj-confirmed.

**Success floor**: a decision-function reading that survives its grid, or a
retraction with the surviving rival named.

**Seams**: `docs/whitebox/WB_INLINE_FINDINGS.md` + `work/wb-inline/` + rung +
board rows **#1840–#1859**. No `crates/`.

## Lane WB-F — `wb-eh`: factor D's machinery

**Question.** How does c2 lay out EH — `.pdata`, unwind info, EH descriptors,
their symbols and relocations? Anchor: `Main.cpp`, the highest-worth frontier
row (EH, factor D over 740 objs), whose reader chain measured 2 rungs then
STUCK where black-box laddering cannot see.

**Firm deliverables.**
1. Locate the EH emission machinery (prior art to honor, not re-tread:
   `.rdata$r` is RTTI not EH — the w-eh5 retraction; the `/FAsc` listing
   seam shows EH layout by name per the listing-seam memory).
2. Read the **table formats and conventions**: `.pdata` record layout,
   unwind-code emission, funclet/descriptor symbols, label-counter
   interaction, relocation types — with VAs.
3. **Un-stick the Main.cpp chain**: name what its next reader rung actually
   is (the construct, its IL bytes, its meaning per c2's reader), so the
   chain becomes priceable again.
4. Black-box confirmation design per reading; run the cheap ones in-lane
   (a minimal try/catch fixture graded against real c2 is in scope).
5. Pre-drafted DISCLOSURE rows; a priced route to `Main.cpp` (or a priced
   decline — also a result).

**Success floor**: the Main.cpp chain's stuck rung is NAMED with a reading
that survived at least one obj check, or a specific finding of why the
construct is unreadable.

**Seams**: `docs/whitebox/WB_EH_FINDINGS.md` + `work/wb-eh/` + rung + board
rows **#1860–#1879**. No `crates/`.

## Lane WB-G — `wb-chooser`: the one-witness-per-side blockers

**Question.** `mmio` and `Biquad` each need the port to choose between two
lowerings, and each side of the choice has exactly ONE witness in the corpus
— which #1767's field/pin evidence rule refuses to fit (correctly: a 2-point
fit is a coin toss dressed as a rule). A **mechanism read from the binary and
obj-confirmed on cells the corpus does not contain** is the remedy the rule
itself anticipates.

**Firm deliverables.**
1. From the two lanes' decline records (board rows citing #1767/#1786),
   reconstruct the exact choice points: the two forms, the two witnesses,
   and the IL that distinguishes them. Re-derive at base; inherited prices
   have been wrong twice.
2. Find each **choice mechanism** in the binary; read its inputs, with VAs.
3. **Manufacture the missing witnesses**: design fixture cells that populate
   both sides of each choice several times over, frozen predictions first,
   graded against real c2. This converts one-witness-per-side into
   many-witness-with-mechanism.
4. If a mechanism survives: the chooser rule stated in port terms +
   pre-drafted DISCLOSURE rows, unblocking two frontier TUs for a follow-on
   code lane. If not: the refutation, and what the grid DID establish.
5. Explicitly out of scope: shipping the chooser into `crates/` (code lane's
   job, with the DISCLOSURE rows in the same commit).

**Success floor**: at least one of the two choice points has ≥3 witnesses per
side with a mechanism reading consistent with all of them — or a written
finding that the choice is not mechanism-driven (e.g. it hangs off IR
construction order), which would retire those two rows honestly.

**Seams**: `docs/whitebox/WB_CHOOSER_FINDINGS.md` + `work/wb-chooser/` + rung
+ board rows **#1880–#1899**. No `crates/`.

## What DONE looks like

Each lane: findings doc under `docs/whitebox/`, rung under `docs/rungs/`
(header format per the registry test — copy a 2026-08-08 `wb-*` rung as the
template; `Fixtures: none` rungs carry a literal `+0` in `Census:`), board
rows from its range, PREREG frozen before the first probe, everything
committed on the lane branch with absolute paths scrubbed to `<repo>`/`<home>`
**before** committing. The coordinator merges `--no-ff`, then edits
`ROADMAP.md` with the campaign outcome and dispatches code lanes to adopt
what survived — adoptions carry their DISCLOSURE rows in the same commit.

The campaign's exit question, answerable lane by lane: **which factor gets a
general rule, and what is the first class-conversion lane it licenses?**
