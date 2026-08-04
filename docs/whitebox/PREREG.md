# PREREG — location predictions made *before* looking

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).

This project's culture is pre-registration with a named incumbent. The
translation-unit partition recovered in [`C2_MAP.md`](C2_MAP.md) gives the lane
something it did not have before: **the ability to predict where a routine lives
before going to look.** That makes the known-answer controls sharper than a
plain hit/miss — a map that predicts the location of routines whose behaviour is
already fully known, and lands them, is strong evidence its range attributions
are sound. It costs nothing but the discipline of writing the guess down first.

**Grading rule.** A prediction is a HIT only if the routine's entry point falls
in the named file's recovered range, or in the ambiguous gap immediately
following it. Naming the wrong file is a MISS even if the address is close.
Hits and misses are graded and reported **separately**; a miss is publishable and
is not smoothed.

## Registration status — read this before believing any row

Pre-registration is only worth something if the timestamp is real. Three tiers
are used here and they are **not** equivalent:

| tier | meaning |
|---|---|
| **PREREG** | committed to git *before* the answer existed anywhere in the lane. Full strength. |
| **IN-FLIGHT** | stated to the analysis child in a message *before* that child returned its result, and recoverable from the message log — but not committed first. Weaker: it is contemporaneous, not cryptographically ordered. |
| **POST-HOC** | written after the answer was known. **Carries no evidential weight** and is recorded only for completeness. |

## Predictions

| # | routine (behaviour already fully known from black-box work) | predicted file | tier | outcome |
|---|---|---|---|---|
| P1 | **JamCRC** — the string-COMDAT name hash: poly `0xEDB88320`, init `0xFFFFFFFF`, no final XOR, over the literal *including* its NUL, rendered base-16 with digits `A`..`P`, MSB first, leading zeros suppressed. Plus the aux-record `CheckSum` variant with init `0`. | **`hash.c`** (`0x10b5a1fc .. 0x10b5b1a0`), alternate **`coffemit.c`** (`0x10b290dc .. 0x10b2b0dd`) | **PREREG** | **MISS** |
| P2 | **The flag/argv parser** — the table of flags c2 accepts; an unrecognized flag yields `C1007 unrecognized flag '%s' in 'p2'`. | **`getflags.c`** (`0x10c1f415 .. 0x10c1f572`) | **PREREG** | **HIT** |
| P3 | **The `/FAsc` listing writer** — emits the complete `.cod` MASM listing under `-FAasc -Fa <file>`. | **`list.c`** (`0x10b709b8 .. 0x10b71933`) for the target-independent half; a separate PPC instruction printer late in the image for the machine-dependent half | **IN-FLIGHT** | **HIT** (graded half; PPC half ungraded) |
| P4 | **The COFF writer** — file header, 40-byte section headers, 18-byte auxiliary section-definition records. Output format fully known from 878 reference `.obj` files. | **`coffemit.c`** (`0x10b290dc .. 0x10b2b0dd`), with a model/reader layer in **`coff.c`** | **IN-FLIGHT** | **HIT** (both halves) |

### Notes on the two IN-FLIGHT rows

P3 and P4 were put to the analysis children in briefing messages before those
children reported, so they are contemporaneous predictions rather than
retrofitted ones — but the lane did **not** commit them to git first, and they
are therefore explicitly **not** claimed as full pre-registration. The
distinction is recorded rather than papered over, because a pre-registration
scheme that quietly relabels post-hoc reasoning is worse than none: it converts
an honest hit rate into an inflated one.

P1 and P2 are committed here while the children hunting them are **still
running** and have returned nothing. Those two are the real test.

<!-- OUTCOMES-START -->
## Outcomes

All four hunts reported. Graded mechanically against
[`c2_tus.tsv`](c2_tus.tsv) — the scoring script locates each address and
classifies it `in-range` / `in-gap-after <file>`, so no judgement enters the
grade. Full write-up in [`C2_MAP.md`](C2_MAP.md) §6.

**The tiers are reported separately and are not pooled.**

| tier | hits | misses | rate |
|---|---:|---:|---|
| **PREREG** (committed before the answer existed) | **1** | **1** | 1/2 |
| **IN-FLIGHT** (contemporaneous, not cryptographically ordered) | **2** | **0** | 2/2 |

| # | routine | predicted | landed | grade |
|---|---|---|---|---|
| P1 | JamCRC | `hash.c` / alt `coffemit.c` | **nowhere — absent from `c2.dll`** | **MISS** |
| P2 | flag/argv parser | `getflags.c` | applier `10c1f572` **in range**; matcher `10c1f746` in the gap immediately after | **HIT** |
| P3 | `/FAsc` listing writer | `list.c` (+ unnamed PPC printer) | `10b70e57`, `10b71324` **in range**; `10b71d8f` in the gap after | **HIT** on the graded half |
| P4 | COFF writer | `coffemit.c` + model layer in `coff.c` | 5/5 **in range**; `10b28586` **in range** in `coff.c` | **HIT**, both halves |

### The two results worth carrying away

**P1 is the most valuable of the four, and it is the miss.** The predicted
routine does not exist in this binary at any address: no `0xEDB88320` table at
any 4-aligned offset, the polynomial immediate absent in every byte order, and
the `A..P` renderer absent (the only `ABCDEFGHIJKLMNOP` run is the base64
alphabet). The table lives in `mspdbXX.dll`. The search method was itself
controlled — two constants the port hardcodes and a fresh obj demonstrably
carries are *also* absent as immediates — so the absence is informative rather
than a failed search. The hunting child's own summary: *"I would have shipped a
wrong address had I pattern-matched hash-looking code near an emit site."* **No
`crc` label was published.** A control that changes what you publish is a
control that was worth running.

`hash.c` was doubly wrong: that region is the CSE/value-number hash (`% 0x65`,
101 buckets), and c2's *actual* string hash `0x10b8a01b` lies in an **unanchored
gap** — a file with no ICE site, and therefore invisible to this method. That is
the partition's known blind spot, caught by its own control.

**P4 is the strongest hit.** Two predicted routines did not merely land in the
right file — **they are the anchor addresses themselves**: `FUN_10b2b0dd`, the
COFF/BIGOBJ file-header writer, *is* `coffemit.c`'s `anchor_end`; `FUN_10b28586`,
the obj opener, *is* `coff.c`'s anchor. The reader/model-versus-writer split was
predicted from the two file names alone, before any disassembly, and it holds.

### Honest deductions

- P2: two of the parser's four sub-components (`FUN_10c1f3c9` wildcard compare,
  `FUN_10c1f34c` wide `atol`) land **one file early**, in the gap after
  `get_err.c`. The named routine landed; the helpers straddle the boundary.
- P3's machine-dependent half named **no file** ("a separate PPC instruction
  printer late in the image"). It is therefore **ungraded** — it can be neither
  hit nor miss. The observed `.cod` printer cluster straddles `mdlist.c`'s
  anchor and `mdlist.c` is indeed late in the image, but that is recorded as
  corroboration, not scored. A prediction vague enough to be unfalsifiable earns
  nothing, and inflating the denominator with it would be the exact failure this
  scheme exists to prevent.
<!-- OUTCOMES-END -->
