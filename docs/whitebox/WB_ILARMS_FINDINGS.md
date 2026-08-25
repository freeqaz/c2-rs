# WB_ILARMS — FINDINGS, and the prereg graded

**Lane `w-ilarms`, wave 11, 2026-08-25. Characterization lane, docs-only.**
Prereg: [`WB_ILARMS_PREREG.md`](WB_ILARMS_PREREG.md), committed first
(`7dac34e5e`), before the first image byte was decoded for this lane and
before any port site was catalogued. Deliverable:
[`WB_ILARMS_MAP.md`](WB_ILARMS_MAP.md). Board **#3567**–**#3572**.

**Outcome: `instrument`.** Two re-runnable instruments and one map; no
fixture, no census move, zero `crates/` bytes, reach 0 as predicted.

---

## 0. The one-paragraph answer

`P_ILRECORD.md`'s correction **stands at a third independent decode**: the
dispatch at `0x10bc2e08` is a two-level switch over **189 opcodes** with
**62 arm targets**, of which **61 are real** and one refuses **94** opcodes
into C1001. The check the earlier reads did not run is the one this lane
registered as its own falsifier — **are the 62 targets distinct?** They are,
62 of 62, which is what makes `62` an arm count rather than an entry count
and is exactly where R2's encoder read went `111 → 79`. Against that
denominator, the port has a decode site for **41 of 61 arms** and **68 of 95
handled opcodes**, has **no** site for 20 arms and 27 opcodes, reads exactly
**one** of the 94 refused opcodes (`0x2d`, and in a different position in the
stream), and mints **zero** of the IR nodes the arms mint. The most
consequential row is that **`NARROW(gate)` came out 0 of 95**: wherever the
port names one of these bytes, it names it in an **ungated** reader —
`control_flow::{step,operand}`, reached from `census.rs:448` on every body —
so the decode/admission unfusing decision 13 funded has a working precedent
inside the same crate, and the map names it.

---

## 1. The prereg, graded — 16 registered, 13 HIT, 2 MISS, 1 UNRESOLVED

### 1.1 Table verification — 6 of 6 HIT

| # | p | prediction | result |
|---|--:|---|---|
| **T1** | 0.95 | the six dispatch instructions decode operand-for-operand as published | **HIT.** `8b55cc 8d42ff 3dbc000000 0f872a130000 0fb6804a42bc10 ff24855241bc10` |
| **T2** | 0.90 | 189 index entries, values spanning `0..61`, `max+1 == 62` | **HIT** |
| **T3** | 0.70 | the 62 DWORD targets are 62 **distinct** arms | **HIT.** 62 of 62, zero duplicates |
| **T4** | 0.85 | exactly 94 in-range opcodes reach the `ja` target | **HIT.** 94 refuse / 95 handled / 61 real arms; arm 61 is the sole index equal to the `ja` destination |
| **T5** | 0.65 | the opcode→arm assignment agrees with `P_ILRECORD.md` §3 on all 61 real arms | **HIT, all 62 rows** — including arm 1's ten (not twelve: `0x07` refuses) and arm 4's ten (not eleven: `0x14` refuses) |
| **T6** | 0.85 | the byte table's extent confirmed two independent ways | **HIT, and taken one step past what was registered.** `0x10bc4307` opens `55 8b ec` = `push ebp; mov ebp,esp`, read from **raw bytes** rather than from `c2_strings.tsv` as `P_ILRECORD.md` did |

**The over-determination check the prereg registered in advance paid.** The
prereg asked how many *alternatives* "189" is also consistent with and named
three; T6 was written to separate them. The 16 bytes past the byte table are
`55 8b ec 83 ec 30 53 8b d9 8b 03 56 8b 75 08 81`, of which only **3 of 16**
are legal arm indices — a longer table would need **all** of them to be. So
the alternative is excluded rather than merely not assumed.

**And the independent-implementation control is green on all nine
comparisons.** `dump_ilarms.py --cross` re-runs `dump_ilrecord.py`'s reader:
both table VAs, both lengths, both bounds, the `ja` target, the **189 index
bytes element for element** and the **62 target words element for element**.
`ALL AGREE`. The prereg's methodology control **M-shape** — that a prior
artifact would disagree with the raw decode, as `#3547` found for arm 7's
prose — **did not fire on the tables**. That is a result, not an absence: the
arm-7 defect was in prose about a callee, and the table half survives
re-derivation intact.

### 1.2 The port's decode vocabulary — 3 HIT, 2 MISS, 1 UNRESOLVED

| # | p | prediction | result |
|---|--:|---|---|
| **V1** | 0.60 | the port reads between **25 and 45** of the 95 handled opcodes | **MISS — 68.** See §2, this is the lane's sharpest self-inflicted error |
| **V2** | 0.85 | ≥ 12 of the 61 real arms have no port reader | **HIT — 20** |
| **V3** | 0.60 | ≥ 8 of the port's readers are width-only | **HIT — 24 of 95** are width- or name-only (no operand field survives) |
| **V4** | 0.35 | ≥ 1 opcode the port reads is in the 94-refuse set | **HIT — exactly one, `0x2d`.** See §3 |
| **V5** | 0.80 | no single `crates/` file covers a majority of the mapped arms, and the decode is spread over ≥ 6 files | **MISS on the first limb, HIT on the second.** `control_flow.rs` reaches **40 of 61** real arms (65.6 %) and **40 of 41** mapped arms (97.6 %); 14 files carry a site |
| **V6** | 0.70 | C1/C2/C3's five opcodes all have a port site, and all are **narrower** than their arm | **HIT on the first clause (5 of 5 have a site); UNRESOLVED on the second, and the prereg is at fault.** See §4 |

### 1.3 The consumer sweep — 4 of 4 HIT

| # | p | prediction | result |
|---|--:|---|---|
| **S1** | 0.55 | the arm-count `189` survives in ≥ 5 `docs/` files beyond the five `P_ILRECORD.md` names | **HIT, exactly 5**: `ref/P_BLOCKORDER.md`, `ref/P_ENCODE.md`, `DECISIONS_2026-08-22.md`, `ROADMAP.md`, `rungs/2026-08-22-w-read-r2.md` |
| **S2** | 0.70 | ≥ 1 surviving occurrence is on a shelf the banner's own consumer list omits | **HIT — two, and both in `docs/whitebox/ref/`**, which is precisely the shelf `#3546` had just finished saying a topic grep does not reach |
| **S3** | 0.60 | **zero** occurrences in `crates/`, `scripts/` or `README.md` | **HIT.** `crates/` 0, `README.md` 0. `scripts/` has one near-miss worth naming rather than suppressing |
| **S4** | 0.50 | ≥ 1 document **DEPENDS** on 189-as-arms rather than merely mentioning it | **HIT — six do**, and one of them depends on it twice |

### 1.4 Disclosure — 2 of 2 HIT

| # | p | prediction | result |
|---|--:|---|---|
| **D1** | 0.97 | this lane owes **0** `DISCLOSURE.md` rows | **HIT, and checked rather than asserted**: `git diff --numstat master..HEAD -- crates/ scripts/` returns **0 files** across the whole lane |
| **D2** | 0.60 | a future adopter would owe ≥ 20 rows, stated as a number | **HIT** — `WB_ILARMS_MAP.md` §9 derives **≥ 23** for the DECODE subset and **≥ 67** for the whole dispatch, itemised, and says both are floors |

---

## 2. V1 MISSED BY 23 OPCODES, AND THE REASON IS A NAMED FAILURE MODE IN THIS TREE

Registered `[25, 45]`; the answer is **68 of 95**.

The prereg's own orientation section records why, which is the only reason
this is gradeable at all. The orientation grep returned **per-file arm
counts** and no opcode list, and the file I had actually read at that point
was `expr.rs`, whose `chain_skip_form` names about thirty opcodes. I
anchored the bracket on that table. **`chain_skip_form` is the port's
NARROWEST reader** — the chain sink's width table, poisoned and
environment-gated, off on every gate lane and every default scan. The port's
*widest* reader is `control_flow::{step,operand}`, which I had not opened,
and which runs on every body in the workload through `census.rs:448`.

**This is "ranking instruments measure themselves" (`#3505`) turned on a
prediction**, and it is the second time in two lanes on this subject:
`w-read-r5`'s own P1.2 missed a distinct-arm bracket by carrying R2's
`111 → 79` *ratio* across to a differently-shaped table. The generalisation
both instances support: **a bracket anchored on the artifact you happen to
have read measures that artifact, not the population.** The cheap
counter-move — enumerate the *entry points* before bracketing anything about
their reach — costs one grep and was not made.

**The miss is in the favourable direction and must not be over-read.** 68 of
95 is a statement about **width**, not about I1. §5 of the map states, and
this section repeats because it is the most likely misreading of the whole
lane: **no port site mints an IR node in the `≥ 0x2af` space**, and a grep of
all five crates for the node opcodes this dispatch mints returns zero
non-comment hits.

---

## 3. V4 HIT ONCE — `0x2d`, AND ITS CONTROL IS THE FINDING

The port reads exactly one byte this dispatch refuses:
`codec.rs:1139 try_prefix_token` and `codec.rs:1344 try_ex_token`, as
`ExToken::Formal(tok)`.

`P_ILRECORD.md` §1.3 asserts that the 94 refusing opcodes *"are legal `.ex`
tokens … `0x10bc2d7a` is one consumer of the `.ex` stream, not the
consumer"*. That sentence is unfalsifiable-sounding until something exhibits
a byte read in one stream position and refused in the other. **`0x2d` is that
byte**, and it comes with its own control: **`0x46` is read by the same two
port functions and is HANDLED by this dispatch, at arm 28.** Two bytes,
identical treatment on the port side, opposite sides of a boundary the binary
draws.

Two consequences, neither of them a re-pricing:

* the port's decode surface **already spans at least two c2 consumers of
  `.ex`**, so a slice scoped as "the port's IL readers" would be scoping
  across that boundary without saying so;
* the "other walk" `P_ILRECORD.md` §8 item 4 lists as unread is not
  hypothetical — the port has been reading its tokens for as long as
  `codec.rs` has existed.

The confirmation probe the prereg registered for this case (a `c2rs census`
key at non-zero count for `0x2d`) was **not run**, because the port site is
in the *container codec* rather than the body walk and the census key would
not distinguish the two positions. Recorded as not-run rather than quietly
dropped.

---

## 4. V6 IS UNRESOLVED BECAUSE THE PREREG UNDER-SPECIFIED IT — a defect in this lane's own instrument

V6 predicted the three keyable residue constructs' five opcodes would all
have a port site **and all be narrower than their arm**. The first clause
HITS: `0x27` (C1), `0x40` (C2), `0x99`/`0x9a`/`0x9b` (C3) all have sites.

The second clause cannot be decided under the definition the prereg fixed.
`NARROW` was defined with two limbs — a precondition the arm does not impose,
**or** fewer operand fields than the arm's class implies. All five clear limb
1 (they have ungated sites), and **limb 2 needs c2's per-class operand
grammar, which lives in the 29 class arms at `0x10b3d954` and was not read by
this lane.** So the honest verdict is `MATCHED*` on all five, where the
asterisk *is* the unchecked limb.

**That is a prereg defect, not a discovery**: a verdict whose second limb
requires a read the lane did not budget is a verdict that cannot be reached,
and the prereg should have said so or scoped limb 2 out. It is recorded here
rather than resolved by relaxing the definition after the fact.

**What could be salvaged, and was.** The class byte is this lane's own raw
read, and it works as an **internal consistency check on the port** without
any knowledge of what a class *means*: opcodes sharing a class share a
payload grammar, so a class the port reads two ways is a place the port's
widths were pinned per opcode from witnesses. That yields two results the
map carries (§6.1, §7.2 there):

* **`0x28` is `NARROW(fields)`**, provable inside the lane: class `02`, but
  the port hard-codes `28 00 00` where all six of its class-`02` siblings
  take a variable-width token. It **refuses** rather than desyncs, which is
  the correct direction.
* **Arms 17 (`0x37`) and 26 (`0x42`) are not really ABSENT.** The port reads
  those bytes as sub-opcodes of a `0x43` *escape*, and class `00` + class
  `02` + class `00` reproduce its `+4` and `+2` **exactly**. Reported beside
  the ABSENT count, **not folded into it** — a fifth category invented after
  the numbers are in is how a count stops being gradeable.

`65 of the 68` `MATCHED*` rows have limb 2 genuinely unchecked, and that is
published as a number rather than rounded away. Reading `0x10b3d954`'s 29
class arms closes all 65 at once and is the cheapest follow-up the map
exposes.

---

## 5. The consumer sweep, in full

**Method.** `grep -rniE "189[ -]?(arm|entry|entries|way|case|dispatch)|arm[s]? *[:=]? *189"` over `docs/`, `crates/`, `scripts/`, `README.md`, then every hit read in context and classified **DEPENDS** (the disputed number is load-bearing for a price or a claim) vs **MENTIONS** (an aside; the surrounding claim survives the correction). The brief required that distinction *"because asking whether a claim MENTIONS a disputed premise rather than whether it DEPENDS on one has cost a ratification here already"*.

Amendment style is `GOAL_DECISION_2026-08-21.md:18`'s, per `DOC_CONVENTIONS.md` §2 mitigation 1 — **the strike names its own superseding section inline, at the struck line**, and the original text stays visible under `~~ ~~`. Nothing was rewritten and nothing was deleted.

| file:line | classification | why | amended |
|---|---|---|---|
| `docs/STEP5_PRICING_2026-08-21.md:139` | **DEPENDS** | the I1 row's read *is* this dispatch and this is the size quoted for it | ✔ |
| `docs/WHITEBOX_LEVERAGE_2026-08-21.md:89` | **DEPENDS** | the 15–25 d in the adjacent cell is priced against the struck count | ✔ |
| `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md:968` | **DEPENDS** | same, and *"zero arms read today"* on the next line is **also stale** — all 62 were read at `#3415` | ✔ |
| `docs/whitebox/READ_PLAN_2026-08-21.md:99` | **DEPENDS** | the unread-targets table; *"ZERO read"* **also stale** | ✔ |
| `docs/whitebox/READ_PLAN_2026-08-21.md:174` | **DEPENDS** | the R5 row the 15–25 d estimate lives in | ✔ |
| `docs/whitebox/C2_MAP.md:1012` | **DEPENDS TWICE** | on the count, **and** on *"mechanical, recipe is exact"* — which `#3419` separately refuted and which **nobody had struck** | ✔ (both) |
| `docs/whitebox/ref/P_BLOCKORDER.md:202` | MENTIONS | a cross-reference row | ✔ |
| `docs/whitebox/ref/P_ENCODE.md:624` | MENTIONS | and *"unstarted"* is **also stale** | ✔ |
| `docs/DECISIONS_2026-08-22.md:163` (decision 5) | MENTIONS | corrected by decision 13 of the **same file**, which did not name itself at the struck line | ✔ |
| `docs/DECISIONS_2026-08-22.md:197` (decision 6) | MENTIONS | same | ✔ |
| `docs/DECISIONS_2026-08-22.md:230` (decision 6) | MENTIONS | same | ✔ |
| `docs/ROADMAP.md:12035`, `:12155` | MENTIONS | **NOT amended** — dated session history, and this tree's standing rule is that dated records stay as written | ✘ by decision |
| `docs/rungs/2026-08-22-w-read-r2.md:163` | MENTIONS | **NOT amended** — a dated rung record | ✘ by decision |

**Twelve amendments across eight files.** `P_ILRECORD.md`'s banner names
**five** consumers; the sweep found **eight live files**, and the two it
misses are both under `docs/whitebox/ref/` — the shelf `#3546` had just
finished establishing is invisible to a topic grep. **The banner's own
consumer list was incomplete, and the rule that catches that is the one
`#3546` wrote.**

**`scripts/`, the near-miss, named rather than suppressed.**
`scripts/sweep.d/46-source-line-collisions.py:68` reads `0xBD,  # 189 CALL`
— `189` as the **decimal value of `0xBD`**, not an arm count, so it is not a
consumer of the disputed premise. It is worth naming because it explains why
the error was sticky: **`189` is simultaneously the decimal spelling of the
last opcode and the size of the opcode domain `0x01..0xBD`**, and both
readings are true. A number that is right twice for the wrong reason survives
five documents.

---

## 6. The self-grade items, and the result on each

The prereg registered five ways this lane fails even if every number is right.

| # | item | result |
|---|---|---|
| 1 | **ordered by mass anywhere** | **clear.** The map's §3 is ordered by arm number, §5's refusal set by opcode, §7.1's absent arms by arm number. No table anywhere sorts by body count, opcode frequency or residue mass. §0 of the map says so in its own words and says the page does not answer "what first" |
| 2 | **a row silently dropped** | **clear.** All 62 arms appear, including the 20 with no site and the refusal |
| 3 | **a count without its denominator** | **clear.** §4 of the map is nothing but denominators; every count in this document carries `of N` |
| 4 | **the 94 not stated as out of scope with their count** | **clear.** Map §5, with the full set re-derived and the reason (they are legal tokens for a different walk, not impossible bytes) |
| 5 | **re-inheriting 189 as an arm count** | **clear.** Every use of `189` in the map and in this page is as the **opcode** count, and the sweep in §5 struck it as an arm count in twelve places |

---

## 7. The instruments' own defects, both self-inflicted and both fixed

Recorded because a scanner nobody grades is `#3336`'s failure mode with the
labels swapped.

1. **`scan_port_opcodes.py` attributed all 63 opcode arms in `expr.rs` to one
   function 1,200 lines above them.** It tracked the enclosing `fn` by brace
   depth, and this tree's doc comments carry fenced code blocks full of
   unbalanced braces. Replaced with nearest-preceding-`fn`.
2. **It reported the six relational opcodes `1F..24` as having NO reader in
   `control_flow::operand`.** They are on the **continuation line** of the arm
   above, and the matcher was single-line. Patterns are joined first;
   `control_flow.rs`'s arm reach went **39 → 40 of 61**.

Both were found by reading the instrument's output back against the source
rather than by anything the instrument could detect about itself. The
exclusion lists are **in the source with their reasons** — six files that
decode a different stream, ten functions whose literals are TYPE tags / PPC
masks / VI32 markers, two lines matching an opcode value in sub-opcode
position — so a later reader can disagree with a specific exclusion instead
of with a number.

**`dump_ilarms.py` shares no code with `dump_ilrecord.py` on purpose.** That
script hard-codes both table addresses, both lengths and both opcode bounds,
so re-running it can only test that the bytes at those addresses are
unchanged; it cannot test the constants. This one hard-codes the dispatch
head and derives the rest.

---

## 8. What this lane did NOT do

1. **No re-pricing of I1, in either direction.** `#1767`'s rule and `#3421`'s
   refusal govern. The two price-relevant numbers still push opposite ways
   and are still not combined.
2. **No read of the 29 operand-class arms at `0x10b3d954`**, which is why 65
   of 68 `MATCHED*` rows have limb 2 unchecked (§4).
3. **No read of any of the 76 callees.** Unchanged from `P_ILRECORD.md` §8.1:
   **0 of 76** bodies read, by that lane or this one.
4. **No obj-grid confirmation of anything structural**, because there is
   none available: this seam's output is an in-memory tree in a private
   opcode range that never reaches an obj (`P_ILRECORD.md` §8 item 5). The
   only thing the byte judge can say about this lane is that it changed no
   port behaviour, and it says it — §9.
5. **No claim that any port site is correct.** Every verdict is about
   existence, gating and width.

---

## 9. The gate

`sh scripts/gate.sh --jobs 4 --require-graded`, read at its **verdict line**
rather than its exit code — `GATE: REFUSED (DIRTY crates/)` exits 0.

`git diff --numstat master..HEAD -- crates/ scripts/` is **0 files** across
the entire lane, so the docs-only fence is checked rather than asserted. The
gate's own result is recorded in the rung,
[`../rungs/2026-08-25-w-ilarms.md`](../rungs/2026-08-25-w-ilarms.md) §1.
