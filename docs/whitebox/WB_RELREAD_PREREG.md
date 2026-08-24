# WB_RELREAD — PRE-REGISTRATION (`WB_RELATION_FINDINGS.md` §5's three ranked follow-up reads)

    Tag:       w-relread
    Date:      2026-08-24
    Kind:      CHARACTERIZATION lane (`../rungs/README.md` § "Lane kinds")
    Base:      67f276409
    Branch:    wt-w-relread
    Board:     #3517-#3520 reserved
    Fixtures:  none · Census: +0 · predicted reach: 0
    Image:     compilers/X360/16.00.11886.00/c2.dll, sha256
               c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
               — VERIFIED by this lane against `C2_MAP_METHOD.md` §0 before the
               first read, on both copies reachable from this box
               (`~/ghidra-projects/bin/c2dll`, and the `dc3-decomp` compat
               fallback). Both match.

**Frozen before any read of the image.** Committed as this lane's first commit.

---

## 0. The assignment, restated so it can be scored

`WB_RELATION_FINDINGS.md` §5 ranks three follow-ons off `w-c7`:

1. **Read `FUN_10c1a908`'s ten switch arms** (~½ day) — *"would retire `#423`'s
   grid entirely"*. Six emitter pairs named: `FUN_10c198d2`/`FUN_10c19bc0`
   (default), `FUN_10c19936`/`FUN_10c19c87` (cases 2 **and** 8),
   `FUN_10c199bc`/`FUN_10c19d50` (3), `FUN_10c19a07`/`FUN_10c19da9` (4),
   `FUN_10c19a7f`/`FUN_10c19e9a` (5), `FUN_10c19af9`/`FUN_10c19f69` (6) — *"the
   pair being selected by a flag this lane did not identify."*
2. **Reconcile `#2102` against §2** — board **`#3490`**: `#2102` reads
   `FUN_10c1ac5c` @ `0x10c1ac5c` as normalising every unsigned relation to
   **ULE**; `WB_RELATION_FINDINGS.md` §2 makes its terminal code 8 unsigned
   **LT**. *"Do not quote either as settled until someone reads the terminal
   arm."*
3. **The eight FP relations (codes 11–18)**, unnamed anywhere in this tree.

**Predicted reach 0.** Zero bytes under `crates/` and `fixtures/`. Nothing is
adopted, so no `DISCLOSURE.md` row is due; the first lane to bake any byte
below into a port table owes one per table.

---

## 1. WHAT ORIENTATION ALREADY FOUND — declared, so it is not scored as a blind prediction

**This must be stated up front or the score is a lie.** Before writing this
file I read `CLAUDE.md`, `WB_RELATION_FINDINGS.md`, `../rungs/2026-08-24-w-c7.md`,
`C2_MAP_METHOD.md` §0, and — because the brief says the citations are
unverified and `#2102` is named in the assignment — I grepped `docs/BOARD.md`
for `#2102`. That grep returned **board `#2207`** (lane `wb-selfit`), which I
had not been pointed at, and it says the question in follow-on 2 is **already
answered by a different read**:

> **THE RELATION-CODE ENUM IS `wb-select`'s, AND IT WAS DERIVED WITHOUT THE
> NAMES.** SETTLED from the enum's own name array (`0x10c38690` → the pool
> descending from `0x10b197f4`): `0 ILLEGAL, 1 EQ, 2 NE, 3 LT, 4 GT, 5 LE,
> 6 GE, 7 ULT, 8 UGT, 9 ULE, 10 UGE, 11 SO, 12 NSO, 13 NS`.

`WB_SELECT_RECONCILED.md` §8 carries the same decode and files a correction
against `#2102` for saying `ULE`. So there are **three** live readings of code
8, not two — `#2102`'s **ULE**, `w-c7`'s **unsigned LT**, and `#2207`'s
**UGT** — and `#3490` is stated against the wrong pair.

**I therefore do not get to claim the enum as a blind prediction.** What I
register below is (a) whether the **image** carries that array at those
addresses, (b) what breaks in `WB_RELATION_FINDINGS.md` §2 if it does, and
(c) the three reads I was actually sent to do. Where a prediction is
downstream of `#2207` rather than independent of it, the row says so and is
scored as **derived**, not as a hit.

---

## 2. Registered SEARCHES

Per `w-2e4`'s adopted rule — **do not predict the existence of an artefact you
have not looked for; register the SEARCH instead** — each block below names
what will be looked for and where, before the outcome is known.

### S1 — the relation-code name array

**Search:** decode a pointer array at `0x10c38690` (and, if it is not there,
scan `.data` for any pointer run into the pool below `0x10b197f4`) and read the
strings it points at, from **raw image bytes**, not from `data.tsv`.

| | registered | p |
|---|---|---|
| **S1a** | the array resolves at **exactly** `0x10c38690` | **0.65** |
| **S1b** | the decoded names are `0 ILLEGAL, 1 EQ, 2 NE, 3 LT, 4 GT, 5 LE, 6 GE, 7 ULT, 8 UGT, 9 ULE, 10 UGE, 11 SO, 12 NSO, 13 NS` — **derived from `#2207`, not independent** | 0.80 |
| **S1c** | the array is **longer than 14 entries** and names codes 14–18 as well (`0x10b189cc` has non-trivial entries at 15–18, so those codes exist) | **0.55** |
| **S1d** | at least one **other** independent pointer array in the image also names these codes (a second, differently-spelled table) | **0.30** |

### S2 — the IL-opcode → relation-code site (`w-c7`'s prereg **W2 MISS**)

`w-c7` registered a *location* prediction, recovered a *value*
(`code = IL opcode − 0x1E`), and scored it a **MISS** on its own initiative.
That value is only consistent with `#2207`'s enum if `Rel::from_opcode`'s order
matches it. It does not: `crates/c2-il/src/func/mod.rs:1411-1416` reads
`0x21 => Le, 0x22 => Lt, 0x23 => Ge, 0x24 => Gt` (**read before freezing this
file**; it is repo source, not the image).

| | registered | p |
|---|---|---|
| **S2a** | **`code = IL opcode − 0x1E` is FALSE** as `WB_RELATION_FINDINGS.md` §2 states it — the true relation is a *permutation*, not a subtraction. **Conditional on S1b** | **0.75** |
| **S2b** | the mapping site exists in the image as a **byte table** (rather than a switch or a chain of compares) | 0.55 |
| **S2c** | I find and name it with an absolute VA | **0.40** — this is the same prediction `w-c7` missed, registered at a *lower* credence than its 0.5 because I now know it is not where a subtraction would be |

### S3 — what `0x10b189b8` actually is

`WB_RELATION_FINDINGS.md` §1 calls it **"strictness flip — `<` ↔ `<=`"** and its
§2 constraint 4 uses that name to *derive* the code assignment. Under `#2207`'s
enum, `b8` pairs `(3 4) = LT↔GT` and `(5 6) = LE↔GE`, which is **operand
exchange (reflection)**, not strictness.

| | registered | p |
|---|---|---|
| **S3a** | `0x10b189b8` is **reflection / operand exchange**, not strictness | **0.75** (conditional on S1b) |
| **S3b** | if S3a holds, `WB_RELATION_FINDINGS.md` §2's *"over-determined four ways"* is **over-determined three ways at most** — constraint 4 assumes the answer | **0.70** |
| **S3c** | both readings of `b8` are involutions fixing `{1,2}`, so **the tables alone cannot distinguish them** — the naming needs the strings or a consumer read. I register this as the *reason* `w-c7` could go wrong while being careful | 0.85 |

### S4 — `FUN_10c1a908`'s ten switch arms (follow-on **1**)

**Search:** the decompiled body and the objdump at `0x10c1a908`; the six named
emitter pairs; the flag that selects within a pair.

| | registered | p |
|---|---|---|
| **S4a** | the six emitter-pair addresses in §5 all resolve to real function starts | **0.60** — the brief warns a cited table had 4 non-mints among 8 rows |
| **S4b** | the switch selector is the **normalized relation code** (after `a4`/`cc`), not the raw one | 0.80 |
| **S4c** | the within-pair flag is **operand width** (32- vs 64-bit) | **0.35** |
| **S4d** | — alternative: the flag is **value vs. branch** (produce a GPR 0/1 vs. set a condition/branch) | **0.30** |
| **S4e** | — alternative: the flag is **result polarity / an inverted-sense bit** (the `+0xb` byte `FUN_10bd507f` flips) | **0.20** |
| **S4f** | — none of the above / something I have not listed | 0.15 |
| **S4g** | the arm count is **exactly ten** as §5 says (six pairs = a default + five coded arms, with 2 and 8 sharing) | **0.55** |
| **S4h** | **the read RETIRES `#423`'s 36-cell grid** — i.e. after it, every cell of (relation × signedness × `k ∈ {0,1,2}`) is predicted by a rule stated in this file | **0.45**. `w-c7` asserted it *would*; I am registering it below even odds because a switch arm can be read correctly and still not close a grid whose cells include a cost tie-break |

### S5 — `FUN_10c1ac5c`'s terminal arm (follow-on **2** = `#3490`)

| | registered | p |
|---|---|---|
| **S5a** | the terminal/canonical code really is **8** | 0.75 |
| **S5b** | code 8 is **UGT** — so **both** `#2102` (ULE) **and** `w-c7` (unsigned LT) are wrong, and `#2207`/`WB_SELECT_RECONCILED.md` §8 is right. **Derived from `#2207`** | 0.80 |
| **S5c** | **`#3490` ends this lane SETTLED**, with a named terminal arm VA | **0.60** |
| **S5d** | the normalisation is a **swap of the operands plus a table lookup**, not a re-derivation | 0.65 |

### S6 — codes 11–18 (follow-on **3**)

| | registered | p |
|---|---|---|
| **S6a** | 11/12 are the **overflow** pair (`SO`/`NSO` per `#2207`), and are therefore **not** "FP ordered/unordered" as `WB_RELATION_FINDINGS.md` §2 calls the whole block 11–18 | **0.70** |
| **S6b** | `WB_RELATION_FINDINGS.md` §2's *"11–18 … left **fixed** by both `a4` and `b8`"* is **wrong for 11–14**: `a4[11..14] = 00` and `b8[11..14] = 00`, which is a map to **ILLEGAL**, not a fixed point. (Read off the table bytes **as quoted in `w-c7`'s own §1**, before opening the image — so this is a reading-comprehension claim about the findings file, not an image read) | **0.85** |
| **S6c** | 15–18 (fixed by `a4` and `b8`, negation-paired `(15 16)(17 18)`) are the **floating-point** relations | 0.55 |
| **S6d** | I can name 14–18 from a string in the image | **0.45** |

---

## 3. Registered FAILURE MODES — the part that has bitten four lanes

### M1 — instrument defect (`#3483`: a parameter test proves REPRODUCIBILITY, NEVER ATTRIBUTION)

Any count I quote (array length, xref count, arm count) will be re-run with a
parameter changed that it **must not** depend on, and the denominator printed.
Specifically:

* **array length** — decode with the walk bound set to 14, 20, 32 and 64 entries
  and report where it stops on its own terms (a non-pointer, an out-of-section
  target), not where my bound stopped it. **I predict at least one of my counts
  moves under this test** — p = **0.40**.
* **xref counts** — `w-c7`'s *"31 xrefs from 26 functions"* on `0x10b189cc` will
  be re-counted from `xrefs.tsv` **and** from `objdump_intel.asm` independently.
  p(the two agree exactly) = **0.5**.
* **attribution, not just stability**: for every count, name the denominator and
  the population it is over. A stable number about the wrong population is
  `#3483`.

### M2 — traversal invariance

Every objdump-derived listing is produced in **1, 3 and 7 chunks** and the
concatenations compared byte-for-byte (`w-2e4`'s control). A listing that is not
chunk-invariant is not quotable.

### M3 — the fence must be WATCHED REFUSING

The reader tool takes the image path and **verifies sha256 before reading a
byte**. It will be run against (a) a **truncated** copy and (b) a copy with **one
byte flipped**, and both refusals recorded in the findings. p(I write a fence
that passes a broken image on the first attempt) = **0.25** — `w-2e4` found
exactly this class of defect in its own scanner.

### M4 — the anti-tidy / over-tidy calibration

`w-r8idiom`'s misses ran **anti-tidy** (three of five predicted the mechanism
messier than it is). `w-2e4`'s misses were both about **what the image
contains**. My S4c–S4f block is deliberately a *distribution over named
alternatives* rather than one guess, so an "I predicted messier" or "I predicted
tidier" outcome is scoreable rather than retro-fittable.

### M5 — the failure mode specific to THIS lane

**I have read `#2207` before predicting.** The risk is that I ratify it because
it is written down and confidently phrased, exactly as `w-c7` ratified its own
constraint 4. Control: **S1 decodes the strings from raw image bytes with my own
tool**, not from `data.tsv`, not from `decomp_all.c`, and not from
`WB_SELECT_RECONCILED.md`. If the raw decode disagrees with `#2207`, `#2207` is
what gets corrected. p(the raw decode disagrees with `#2207` in at least one
entry) = **0.20**.

### M6 — publishing a name I have not earned

`w-r8idiom` refused to name `0x2e4`; `w-2e4` named it structurally while still
refusing the name. **Every claim in the findings carries `[R]` (read from the
image) or `[O]` (measured over objs), and no claim carries both.** An inference
between the two is marked `[I]` and is not a finding. I register in advance
that I expect to **refuse at least one name** this lane — p = **0.6**.

---

## 4. Cost and deliverables

| | registered |
|---|---|
| **X1** | this lane is **≤ 1 session**, interval [0.5, 1.5] |
| **X2** | **0 net `crates/` and `fixtures/` lines**, hard — verified by `git diff --stat 67f276409..HEAD -- crates fixtures c2host` being **empty** |
| **X3** | deliverables: this file, `WB_RELREAD_FINDINGS.md`, a rung, 1–4 board rows in `#3517`–`#3520`, and 1–2 tools under `docs/whitebox/scripts/` |
| **X4** | the gate is **non-regression only** — this lane cannot move it and will not quote it as support |
| **X5** | I take the three follow-ons **in §5's ranked order** and stop when the budget is spent rather than half-taking a later one. **Exception, declared here:** S1/S2/S3 are taken **first** because they are the frame every later read is stated in, they are cheap (≈1 hour), and follow-on 2 is *about* them. If that ordering is wrong, it is wrong on the record |

## 5. What would make this lane **FAILED**

Producing no address-cited finding; or publishing a name without a read behind
it; or quoting a count that moves under M1 without saying so; or leaving
`#3490` in the same state it is in now **without saying that is what happened**.
A `#3490` left open with the reason named is `instrument`, not `FAILED`.
