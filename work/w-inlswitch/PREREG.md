# PREREG — lane `w-inlswitch`, c2's own inliner decision surface

**Registered 2026-08-28, BEFORE any byte of `c2.dll` or of any disassembly
listing was opened by this lane.** Base tree `4b79bf46a`, branch
`wt-w-inlswitch`.

Lane kind: **characterization** (`docs/rungs/README.md` § "Lane kinds", kind 3).
`Fixtures: none`. `Census: +0`. **Predicted reach: 0** — zero `crates/` bytes,
**no `DISCLOSURE.md` row** (a disclosure row accompanies an adoption and this
lane makes none), **no `scripts/gate.sh` row** (`#3691`: a 22nd count-bearing
row makes `scripts/gate_identity_diff.sh` exit 2 for every other live lane this
wave).

Charter: `docs/DECISIONS_2026-08-22.md` § Decision 22,
`docs/ADOPTION_BRIEF_2026-08-28.md` §L4. Board **#3768**–**#3773**.

---

## 0. What this lane already knows, and from where — declared before predicting

Honesty about the starting state, because two of the numbers below are already
decided by a **committed artifact of a previous lane** and it would be dishonest
to register them as predictions.

Read before writing this file, all committed, none of them the image:
`CLAUDE.md`; `docs/ADOPTION_BRIEF_2026-08-28.md`; `docs/DECISIONS_2026-08-22.md`
§22; `docs/rungs/README.md`; `docs/whitebox/ref/P_INLINE.md` §5, §6.6.1, §6.7;
`docs/rungs/2026-08-27-w-inlfit.md`; `docs/rungs/2026-08-28-w-lowerband.md` §7;
`work/w-inlfit/PREREG.md`; `work/w-inlfit/optmap.py`; **`work/w-inlfit/optmap.out`**;
`work/w-lowerband/dwordwrites.py`; `work/w-lowerband/bytescan.py`.

**Already established, from `work/w-inlfit/optmap.out` alone** (commands run in
this tree, output quoted in §1 of the findings page): the recovered descriptor
table names **24** `-inl`-prefixed switches, not 21, over **24 distinct**
value-word addresses that tile `0x10c45db4`–`0x10c45e10` **contiguously with no
gap** (`(0x10c45e10 − 0x10c45db4)/4 + 1 = 24`). **23** carry the numeric-option
suffix `#`; one (`-inlnlw`, `0x10c45db8`, kind `0x0101`) is a **boolean**.

So `#3718`'s and `docs/ADOPTION_BRIEF_2026-08-28.md` §L4's *"21 undocumented
`-inl*#` switches"* is **already contradicted by the artifact it cites**, and
this lane does not get credit for predicting it. What it must still do is
**re-derive it from the image** (re-run `optmap.py` here rather than quote the
`.out`), and decide whether `21` was a different, defensible count under some
stated screen — because "the brief is wrong" and "the brief screened
differently" are different findings.

---

## 1. The three targets, and the prediction for each

### P1 — the 24 switches: **defaults**

`0x10c45db4`–`0x10c45e10` is above the image's raw `.data` end
(`0x10c3cc00`, `P_INLINE` §5/§6.6.1 fact 2), so every one of the 24 value words
is **BSS, zero at load**. Therefore *"load-time default"* has two possible
meanings and the brief's request presumes the wrong one: the C-language default
is `0` for all 24 by construction, and any **operative** default must be planted
by an explicit initializer store in `.text`, exactly as the descriptor table
itself is.

**PREDICTED: at least 12 of the 24 carry an explicit initializing store** in
`.text` that is not the descriptor's own `value_ptr` plant.
**Refuted if fewer than 12 do** — which would mean c2's inline knobs are almost
all "0 means the feature is off / the default is elsewhere", a different and
more interesting shape.

### P2 — the 24 switches: **readers**

**PREDICTED: at most 11 of the 24 have any reader** (any instruction that loads
the word, as opposed to the descriptor plant and the initializer store). The
reason registered in advance: a 24-entry contiguous knob block with names as
specific as `-inlocsa1#`…`-inlocsa4#` reads as a **development-era tuning
harness**, and this project's repeated finding (`P_INLINE` §5's POGO model,
`C2_MAP_METHOD.md` §7 case 1) is that the most model-like code in the inliner is
not the code the workload takes.

**Refuted if 12 or more have a reader.**

**Sub-prediction, separately gradeable:** **PREDICTED at most 6 of the 24 have a
reader I can tie to a *named decision*** — a specific branch whose outcome the
word changes. This is the number the brief actually asks for and I expect it to
be the small one.

### P3 — `DAT_10c3de20`: **the "narration" claim FAILS**

`w-lowerband` §7 filed it as *"naming the switch that sets it to `2` would make
c2 narrate its own inline decisions"*, and the brief repeats it as *"the direct
measurement of the quantity this whole thread is about."*

**PREDICTED: there is no such switch, and the phrase does not survive.**
Two halves, gradeable separately:

* **(a) PREDICTED: no descriptor record's `value_ptr` is `0x10c3de20`, and no
  writer of it is reachable from a resolvable command-line switch name.** Half
  of this is already known — `optmap.out` has no such row (`w-lowerband` says
  so, and the artifact agrees) — so the load-bearing half is the **writer
  walk**: all ten writers resolve to internal mode decisions, not to option
  handlers.
* **(b) PREDICTED: `DAT_10c3de20` is a compilation-MODE selector, not a
  diagnostic.** Setting it to `2` would **change** c2's inline decisions, not
  report them, so even if a switch existed it would not "narrate". My advance
  guess at the mode axis, registered so it can be scored: **whole-program /
  LTCG-style operation** (`-ltcg` is in the table at `0x10c46308`), or the
  front-end-vs-back-end phase distinction.

**Refuted if** a writer traces to an option handler with a recoverable switch
name, **or** if the `== 2` arm gates a *reporting* path (a diagnostic emit, a
`/FAsc`-style listing, an ETW/trace call) rather than a codegen-policy path.
Either refutation is the more valuable outcome and both are named here so that
neither can be discovered and then reframed.

### P4 — `FUN_10b5da2f`

573 B, unread, reads `k` = `DAT_10c2ea98` twice (`0x10b5da64` as `(n+2) × k`,
and `0x10b5dacb`, `#3734`).

**PREDICTED: it is a capacity/limit computation and it is NOT on the inline
decision path this workload takes** — i.e. no call chain from the inline driver
`FUN_10b61ee1` / the candidacy function `FUN_10b5fb5f` reaches it, and its own
callers sit in a different pass. Registered because `k`'s *other* reader
(`FUN_10b5e4cc`) is the one that produces `DAT_10c46318`, and the brief's own
framing — *"a general inliner scaling knob"* — is `[I]`, not `[R]`.

**Refuted if** a caller chain from the inliner band reaches it, which would make
`k` a live run-time input at two places instead of one and raise the stakes on
P5.

### P5 — is `k`'s **run-time** value settled?

`#3734` correction 2: `k = 3` is the **load-time** value; that it is the
run-time value under `/O1` is *not* established, because `0x10c29800` plants
`k`'s **address** in the `-vol#` descriptor — a handle for a generic
numeric-option setter to store through. And since `DAT_10c46318` is BSS, C8's
ceiling is settled only when `k` is.

**PREDICTED: I can settle it, and the answer is `3`.** The route registered in
advance: read the **kind-`0x2401` handler** (the numeric-option setter shared by
`-Gs#`, `-Gt#`, `-vol#`, and 23 of the 24 `-inl` words) and show that it stores
only when the switch is **present on the argument vector**; then confirm the
project's own c2 command line carries no `-vol#`.

**Refuted if** the setter has an unconditional initialization sweep that writes
every descriptor's value word (which would make `3` *not* the operative value),
or if any second writer of `0x10c2ea98` exists, or if the handler's semantics
cannot be pinned — in which case the honest verdict is **"still not settled"**
and this lane says that in those words rather than asserting `3`.

**This lane does NOT adopt 128 under any outcome** (Decision 22 §3, `#3732`:
eight counterexamples in each direction). Settling `k` settles a *provenance*
question, not the port's constant, whose defect is a **unit** (`P_INLINE`
§6.6.1's second link) and is untouched here.

---

## 2. Controls — each watched failing before any verdict is quoted (`#3336`)

This lane's specific hazard is recorded four times in this repo: a *"no reader /
no writer / no cell exists"* claim that was really a claim about **an
instrument's index**. Three instruments over three different populations, and
the two that can miss are watched missing.

| # | instrument | population | what it can miss |
|---|---|---|---|
| **I1** | the objdump Intel listing, regex over `ds:0x…` operands | linear decode of `.text` | anything inside a **desynchronised run** — c2 has a ~150 KB data block at the head of `.text` |
| **I2** | Ghidra `xrefs.tsv` / `data.tsv` (control-flow-driven, independent database) | the whole export | anything Ghidra failed to disassemble, and it re-materialises loads inside idioms |
| **I3** | a **decode-independent byte scan** of all of `.text` for the direct-addressed encodings that touch a given absolute address | every byte of the section | nothing, by construction; it over-reports |

* **I3 GREEN control:** it must find the store at `0x10b5e4d7`
  (`mov DWORD PTR ds:0x10c46318,0x3e8`), which `P_INLINE` §6.6.1 establishes
  independently. If it does not, the scan is broken and no absence claim from
  this lane may be quoted.
* **I3 RED control:** it must return **zero** hits for a **planted** address
  that no instruction can reference (an address outside every section). A scan
  that reports hits for a nonexistent target is matching noise.
* **I1 RED control:** re-run the harvest with the address window shifted off the
  descriptor block and confirm the record count collapses — a harvest that
  succeeds wherever it is pointed has no discriminating power.
* **Cross-population rule:** every "N readers / N writers" figure in the
  findings page is reported **per instrument**, with its denominator beside it
  (`#3470`/`#1002`). Where I1 and I2 disagree the disagreement is printed, not
  reconciled silently.

---

## 3. What this lane will NOT do

* Write any byte under `crates/` — `git diff master..HEAD -- crates/` must be
  empty at the tip.
* Add a `DISCLOSURE.md` row or a `gate.sh` row.
* Touch `work/w-inlmetric/CLAUSES.tsv` (`w-clausefix`'s this wave),
  `docs/whitebox/ref/P_GLOBREGS.md`, `docs/whitebox/ref/P_REGALLOC.md`,
  `docs/STATUS.md`, `docs/rungs/INDEX.md`.
* Add, remove, renumber or restate any clause row in `P_INLINE` §6.1. The
  reachable denominator stays **21 of 24** and this lane does not restate 24 as
  a reachable count (`#3505`).
* Adopt `128`, or restate it as settled.
* Rewrite any existing `P_INLINE` section. Amendments are **beside**, in a new
  §6.8, with every claim tiered `[R]` / `[O]` / `[I]`.

## 4. Priority, if not all three land

Charter order: **target 3, then 1, then 2.** A target not reached is named as
not reached, in those words, with what stopped it.

## 5. What makes this lane `FAILED`

Producing neither (a) address-cited answers for at least one of the three
targets nor (b) a precise statement of what the read does not reach. "The
switches are undocumented" restated at greater length is not a deliverable.
A lane that names 24 switches and ties **zero** of them to a reader has produced
a table transcription, not a characterization, and says `FAILED` in that word.
