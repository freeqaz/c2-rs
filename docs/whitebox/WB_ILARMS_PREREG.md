# WB_ILARMS — PREREGISTRATION

**Lane `w-ilarms`, wave 11, 2026-08-25. Characterization lane.**
`Fixtures: none` · `Census: +0` · predicted reach **0** · **docs-only, zero
`crates/` bytes**.

Committed **before** the first byte of `c2.dll` was decoded for this lane and
before any port site was catalogued. Frozen: nothing below is edited after this
commit; the grade lands in `WB_ILARMS_FINDINGS.md` beside it.

Funded by [`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md)
**decision 13** (row 4a(i) / I1, *"okay lets fund general decode now"*). Board
rows reserved **#3567**–**#3572**.

---

## 0. Orientation — declared, so it cannot be scored as prediction

Read before this file was written, and **not** blind:

* `docs/whitebox/ref/P_ILRECORD.md` in full, including its ⛔ banner and the
  arm-7 correction banner (`#3547`).
* `docs/DECISIONS_2026-08-22.md` decision 13; `docs/BOARD.md` rows `#3410`,
  `#3415`–`#3421`, `#3546`, `#3547`, `#3442`.
* `docs/rungs/README.md` § "Lane kinds"; `docs/DOC_CONVENTIONS.md` §2;
  `docs/whitebox/ref/README.md` §2, §2.1.
* An **existence check** on `docs/whitebox/` for a prior arm → port-site map:
  **none exists.** The 20 `WB_*_FINDINGS.md` and the 15 `ref/P_*.md` pages were
  listed; no page reconciles c2's dispatch arms against `crates/` decode sites.
  `docs/IL_DECODE_REACH.md`, `IL_EXPR_LAYER.md` and `IL_STMT_GRAMMAR.md` are
  port-side grammar documents and cite **no** c2 address for a dispatch arm.
* A **shape-only** grep of `crates/c2-il/src` and `crates/c2-core/src` for
  opcode-literal match arms, which returned a per-file count and no opcode
  list: `expr.rs` 63, `control_flow.rs` 41, `mcall.rs` 39, `body/mod.rs` 28,
  `codec.rs` 26, `mcall_tail.rs` 15, and twelve files below 8. That count is
  orientation; every opcode→site attribution in the deliverable is derived
  after this commit.
* The pinned image is present and its digest verified:
  `compilers/X360/16.00.11886.00/c2.dll`, sha256
  `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

**`#3546`'s rule applied in advance**: before hunting anything, grep
`docs/whitebox/ref/P_*.md` for the address or opcode. Done for this lane's
subject; the hit is `P_ILRECORD.md` itself and it is the artifact I am
reconciling, not evidence I may quote as a raw read.

---

## 1. The rule this lane binds itself to before it starts

**Decode from raw image bytes, never from a prior lane's artifact.**
`P_ILRECORD.md` §3's 62-row table, `dump_ilrecord.py`'s constants, `ADDR.tsv`,
`data.tsv` and `WB_ILRECORD_FINDINGS.md` are all **premises under test**, not
sources. `w-relread` registered this rule and it fired
(`WB_RELATION_FINDINGS.md` §2 wrong on 8 of 10 codes from its own mis-naming);
`w-relsite` then found `P_ILRECORD.md`'s own arm-7 cell wrong in both of its
clauses (`#3547`). A third instance is expected, not assumed.

Concretely: the two tables are re-parsed by a decoder written for this lane
that takes **only** the image path, locates the dispatch head by decoding the
instruction bytes at `0x10bc2e08` rather than by trusting the published table
VAs, and derives the table addresses, strides and extents from the operands it
decodes. `dump_ilrecord.py` is then re-run as an **independent second
implementation**, and the two are compared. Agreement is a control; a
disagreement is the finding.

**`[R]` is not `[O]`.** Everything about the dispatch tables is `[R]`. The
port-site half is not a read of c2 at all — it is a read of this repository —
and is marked `[src]` throughout to keep the two apart.

## 2. What is NOT being claimed

* No re-pricing of I1 in either direction. `#1767`'s rule and `#3421`'s refusal
  govern: the arm count and the callee surface push opposite ways and this lane
  does not combine them.
* No ranking. **The map is ordered by arm number and by opcode, never by mass.**
  The standing finding — a lane dispatched off a blocked-key size ranking finds
  the ranking was an artifact — has bound **five times** (`#3505`). If the
  output can be read as "work these first" the lane has produced the wrong
  artifact, and that is a self-grade item in §7.
* No claim that a port site is *correct*. "The port has a reader here" is a
  statement about existence and width, not about agreement with c2.
* Nothing is adopted into `crates/`, so **no `DISCLOSURE.md` row is owed by
  this lane**. §6 predicts how many a future adopter would owe.

---

## 3. Predictions — table verification (the premise I must not re-inherit)

The coordinator has **not** verified 61/95/94; it is one lane's read.

| # | prediction | p | falsified by |
|---|---|--:|---|
| **T1** | The instruction bytes at `0x10bc2e08` decode to exactly the six instructions `P_ILRECORD.md` §1.1 prints, operand for operand (`lea eax,[edx-1]` / `cmp eax,0xbc` / `ja 0x10bc4143` / `movzx eax,byte[eax+0x10bc424a]` / `jmp dword[eax*4+0x10bc4152]`) | 0.95 | any operand differing — a different bound, a different table VA, or a one-level switch |
| **T2** | The byte table at `0x10bc424a` holds 189 entries whose values span `0..61` inclusive, i.e. `max(index)+1 == 62` | 0.90 | a value ≥ 62, or a max < 61 |
| **T3** | The 62 DWORD entries at `0x10bc4152` are **62 distinct** targets — no two arms coincide | 0.70 | any duplicate target. This is the *alternative* T2 does not exclude: a 62-entry table can still name < 62 arms |
| **T4** | Exactly **94** of the 189 opcodes map to the index whose target is `0x10bc4143`, and that target is also the `ja` destination | 0.85 | a different count, or the `ja` target not appearing in the DWORD table |
| **T5** | The opcode→arm assignment I derive agrees with `P_ILRECORD.md` §3's opcode column on **all 61 real arms** | 0.65 | ≥ 1 arm whose opcode set differs. `#3547` already found one *prose* cell wrong; the *table* half has never been re-derived by anyone |
| **T6** | The byte table's extent is confirmed two independent ways — 189 entries ends at `0x10bc4306`, and the next function begins at `0x10bc4307` | 0.85 | the entry count and the function boundary disagreeing |

**Over-determination check, registered in advance** (the brief's rule: count how
many constraints the ALTERNATIVE also satisfies). The claim "189" is
*consistent with* at least three tables I have not distinguished: (a) a byte
table of exactly 189, (b) a longer byte table whose tail is never reached
because the `cmp/ja` bounds it, (c) a byte table that is really data for the
next function and happens to start there. T6 separates (a) from (b) only if the
function boundary is read from something other than the same lane's guess —
so I will read it from the raw bytes (a `push ebp`/`mov ebp,esp` or equivalent
prologue at `0x10bc4307`) rather than from `c2_strings.tsv`, which is what
`P_ILRECORD.md` used. If the prologue is not there I report (b)/(c) as live.

## 4. Predictions — the port's decode vocabulary

Denominators fixed **now**, before counting: the arm denominator is **61 real
arms** (62 less the refusal); the opcode denominator is **95 handled** of 189
dispatched; the refusal denominator is **94**.

| # | prediction | p | falsified by |
|---|---|--:|---|
| **V1** | The port has *some* reader — of any depth, including a pure width skip — for between **25 and 45** of the 95 handled opcodes | 0.60 | a count outside `[25,45]` |
| **V2** | ≥ 12 of the 61 real arms have **no** port reader at any site | 0.85 | fewer than 12 |
| **V3** | ≥ 8 of the port's opcode readers are **width-only** — they advance the cursor and push no semantics | 0.60 | fewer than 8 |
| **V4** | At least one opcode the port reads is in the **94-refuse** set, i.e. the port carries a reader for a token this dispatch will not accept | 0.35 | none is. **Either result is a finding**: a hit says the port's grammar is not this walk's grammar; a miss is a real containment result |
| **V5** | No single `crates/` file covers a majority of the mapped arms — the port's decode is spread over ≥ 6 files | 0.80 | one file covering > 50 % of mapped arms |
| **V6** | The three keyable residue constructs (C1 `0x27`, C2 `0x40`, C3 `0x99`/`0x9a`/`0x9b`) all have a port site, and **all of them are narrower than the arm** | 0.70 | any of the five opcodes having no site, or any port site admitting at least what its arm does |

**"Narrow" is defined before it is measured**, so the verdict cannot drift: a
port site is **NARROW** iff it decodes the opcode under a precondition the arm
does not impose (an environment gate, an admitted-shape gate, a fixed operand
form) *or* it consumes fewer operand fields than the arm's class implies. It is
**MATCHED** iff neither holds. It is **ABSENT** iff no site names the opcode.
No fourth verdict; a row I cannot decide is reported as **UNRESOLVED** and
counted, never guessed.

## 5. Predictions — the consumer sweep of "189"

| # | prediction | p | falsified by |
|---|---|--:|---|
| **S1** | The literal `189` used as an **arm** count survives in ≥ 5 files under `docs/`, beyond the five `P_ILRECORD.md` already names | 0.55 | fewer than 5 |
| **S2** | ≥ 1 surviving occurrence is in a file that is *not* in `P_ILRECORD.md`'s named list — i.e. the banner's own consumer list is incomplete | 0.70 | the named list being exhaustive |
| **S3** | **Zero** occurrences in `crates/`, `scripts/` or `README.md` | 0.60 | any hit. `#3314`'s three non-`docs/` sites were reached by no `docs/` grep, which is why this is a separate prediction |
| **S4** | At least one document **depends** on 189-as-arms rather than merely mentioning it — a price, a ratio or a per-arm multiplication | 0.50 | every occurrence being an unload-bearing mention. **The distinction is the point**: the brief's rule is to ask whether a claim DEPENDS on the disputed premise, not whether it MENTIONS it, and this is where that costs something |

## 6. Prediction — disclosure

| # | prediction | p | falsified by |
|---|---|--:|---|
| **D1** | This lane owes **0** `DISCLOSURE.md` rows, because it adopts nothing into `crates/` | 0.97 | any `crates/` byte changing |
| **D2** | A future adopter implementing the map's DECODE arms would owe **≥ 20** rows — one per adopted address — and I will state the number rather than gesture at it | 0.60 | my published count being un-derivable from the map |

## 7. Self-grade items — how this lane fails even if every number is right

1. **Ordered by mass anywhere.** If any table in the deliverable sorts by body
   count, opcode frequency, or residue mass, the lane failed §2's rule.
2. **A row silently dropped.** Every one of the 61 real arms appears in the map,
   including the ones with no port site and the ones I cannot decide. A missing
   row is a defect; an `ABSENT`/`UNRESOLVED` row is the deliverable working
   (`w-keymap` declared 30.89 % unattributable and that was the point).
3. **A count without its denominator in the same breath.** Every number in the
   deliverable carries `of N`.
4. **The 94 refusing opcodes not stated as out of scope, with their count.** A
   later reader must not price them.
5. **Re-inheriting 189.** If the finished deliverable states 189 as an arm count
   anywhere outside a quoted correction, the lane reproduced the defect it was
   sent to sweep.

## 8. Decline floor

* **DECLINE (outcome `declined`)** if the independent re-derivation contradicts
  62/189/94 in a way I cannot resolve inside the lane. The contradiction is then
  the deliverable and the map is not published on a premise I could not verify.
* **FAILED** if the image cannot be read, or if the map cannot be built for a
  reason other than the above.
* **NOT a decline**: a low port-site attribution rate. Even if only a handful of
  the 61 arms have a port reader, the map is published — the unattributed rows
  are the price signal every I1 slice needs, and suppressing them would be the
  #3336 failure (an instrument nobody can consume) in the other direction.
* **NOT a decline**: `P_ILRECORD.md` turning out wrong on some arm rows. That is
  amended beside per `ref/README.md` §2.1 and reported, and the map is built on
  the re-derived assignment.

## 9. Confirmation probe

`P_ILRECORD.md` §8 item 5 states why this seam has no obj observable: its
output is an in-memory tree in a private opcode range that never reaches an
obj. So the **structural** claims are `[R]` and stay `[R]`.

What *is* cheap and will be run: `scripts/gate.sh --jobs 4 --require-graded`,
which drives real `c2.dll` under wibo across 18 lanes. It confirms nothing about
the arm table — it confirms this lane changed no port behaviour, which is the
docs-only fence's own claim and the one thing here that can be graded by the
byte judge. The **verdict line** is read, not the exit code.

A second, genuinely cheap probe **is** available for one clause and is
registered here so it cannot be added post hoc: if the port carries a reader for
an opcode in the 94-refuse set (**V4** hits), that opcode's presence in the
dc3 workload's captured `.ex` streams is already recorded by the port's own
census keys, and a `c2rs census` key for it existing at non-zero count would
mean the token is real in this workload — which does **not** contradict the
refusal (a token can be legal in the stream and illegal in this walk, §1.3) but
does say which walk a port slice would be modelling. Reported as a finding
either way; not treated as a refutation of anything.
