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
| P1 | **JamCRC** — the string-COMDAT name hash: poly `0xEDB88320`, init `0xFFFFFFFF`, no final XOR, over the literal *including* its NUL, rendered base-16 with digits `A`..`P`, MSB first, leading zeros suppressed. Plus the aux-record `CheckSum` variant with init `0`. | **`hash.c`** (`0x10b5a1fc .. 0x10b5b1a0`), alternate **`coffemit.c`** (`0x10b290dc .. 0x10b2b0dd`) | **PREREG** | *pending* |
| P2 | **The flag/argv parser** — the table of flags c2 accepts; an unrecognized flag yields `C1007 unrecognized flag '%s' in 'p2'`. | **`getflags.c`** (`0x10c1f415 .. 0x10c1f572`) | **PREREG** | *pending* |
| P3 | **The `/FAsc` listing writer** — emits the complete `.cod` MASM listing under `-FAasc -Fa <file>`. | **`list.c`** (`0x10b709b8 .. 0x10b71933`) for the target-independent half; a separate PPC instruction printer late in the image for the machine-dependent half | **IN-FLIGHT** | *pending* |
| P4 | **The COFF writer** — file header, 40-byte section headers, 18-byte auxiliary section-definition records. Output format fully known from 878 reference `.obj` files. | **`coffemit.c`** (`0x10b290dc .. 0x10b2b0dd`), with a model/reader layer in **`coff.c`** | **IN-FLIGHT** | *pending* |

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

*(graded once every child has reported; hits and misses tallied separately)*
<!-- OUTCOMES-END -->
