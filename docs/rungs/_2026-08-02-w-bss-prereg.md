# Pre-registration — lane `w-bss`, the `.data`/`.bss` section shape (2026-08-02)

Written and committed **before the first capture of this lane**. Scored verbatim
in `docs/OBJ_DATA_BSS_SHAPE.md`; wrong predictions stay on the page.

## What I am measuring, and why

§10.19 factored Phase 7 into four predicates and found the tightest is **C**,
the section shape: the port's COFF writer emits 6 section names, the workload
uses 13, and only **84 of 871** TUs are in reach. The greedy ladder's single
largest step is **`.bss`, worth +402 TUs**.

This lane produces the byte-level specification a later writer rung needs for
`.data` and `.bss`, exactly as `OBJ_DYNINIT_SHAPE.md` did for the `??__E` obj.
**Measurement only — nothing under `crates/`.** Reference objs from the real
`c2.dll` under wibo, at the workload's `/O1 /Oi /EHsc /GS- /c`.

Five questions, from the brief:

1. `.data`/`.bss` **section headers** — Characteristics, alignment,
   `SizeOfRawData` vs `PointerToRawData`, position in section order, COMDAT.
2. The **symbol records** for defined data — storage class, `Value`,
   `SectionNumber`, aux, and how `static` / `extern` / `const` /
   `__declspec(selectany)` move them.
3. **Address assignment** within `.bss` and `.data` — ordering, offsets,
   inter-object padding.
4. **The known hole**: with ≥3 namespace-scope objects the `.bss` address
   assignment is a name-keyed permutation (§10.16 measured `N=6` →
   `s6 s4 s3 s5 s1 s2` and declined it). **Crack it or bound it.**
5. **`.rdata` beyond the string case** — FP constants, `const` data, jump
   tables — deferring to `OBJ_DYNINIT_SHAPE.md` §5 / `OBJ_GY_SHAPES.md` where
   they already answer.

## My bias, in writing

**I want the ≥3-object permutation to yield**, because "cracked the hash the
previous lane declined" is the outcome that makes this lane look like more than
bookkeeping. The failure mode that points at is **declaring a hash fitted on
`s1..s6`-shaped names** — six two-character names differing in one digit — and
shipping a rule that is perfect in-sample and wrong on `?g_pFooBarBaz@@3PAVQux@@A`.
This is the exact shape of the four rules the held-out protocol has already
caught (§10.16, §10.19).

Guards, registered in advance:

* Any ordering/hashing rule gets its **out-of-sample predictions committed as a
  git object before the held-out cells are compiled**, per the project protocol.
* Name axes must include **long names, decorated names, names differing in
  length, and names differing only in a middle character** — not just a
  trailing-digit family.
* Declining is a good outcome. "It is a hash I could not crack, here is the
  permutation table across N and the hypotheses I killed" is registered here as
  a **success**, and the second guard is that a *bounded* result must still
  state, in TUs, how much of the +402 it blocks.

Secondary bias: I expect `.data`/`.bss` to be boringly regular (one section
each, no COMDAT, characteristics constant), which makes me likely to under-vary.
Registered mitigation: `__declspec(selectany)`, `const`, `extern`, thread-local
and header-shared objects are all in the grid **because** I predict they change
nothing, so a change is discoverable rather than assumed away.

## Registered point predictions

Graded at `/O1 /Oi /EHsc /GS- /c` unless stated.

| # | prediction | confidence |
|---|---|---|
| P1 | `.bss` Characteristics = `0xC0<a>00080` (`CNT_UNINIT_DATA\|READ\|WRITE`), `.data` = `0xC0<a>00040` (`CNT_INIT_DATA\|READ\|WRITE`), `<a>` the alignment nibble | high on `.bss` (§10.16 measured it), medium on `.data` |
| P2 | `.data` `SizeOfRawData` = sum of the initialized objects' sizes with a **real** `PointerToRawData`; `.bss` keeps `PointerToRawData = 0` and its size in `SizeOfRawData` | high |
| P3 | Section order: at most **one** `.data` and **one** `.bss` per obj, both after every code group, `.data` **before** `.bss`, and `.bss` before `.CRT$XCU` | medium — the `.data`/`.bss` relative order is a guess |
| P4 | Neither `.data` nor `.bss` is ever COMDAT — **including** under `__declspec(selectany)`, which I predict is instead diverted to a *separate* COMDAT section | low-medium. Named alternative: `selectany` makes `.bss`/`.data` itself a COMDAT with Selection 2 (ANY), which would break "exactly one `.bss`" |
| P5 | The defined-data symbol: `Value` = its byte offset within the section, `SectionNumber` = the section's 1-based index, `Type = 0`, **no aux record**; `static` ⇒ STATIC + undecorated name, non-`static` ⇒ EXTERNAL + decorated | high (§10.16 measured the two-linkage half on `.bss`) |
| P6 | A **`const`** namespace-scope object with a constant initializer lands in **`.rdata`, non-COMDAT**, and if unreferenced is **dropped entirely** (the emit predicate's "internal *const* data is dropped when unreferenced", PHASE7_PLAN §2) | medium |
| P7 | An `extern`-declared-but-not-defined object contributes **no** section and one **undefined EXTERNAL** symbol (`SectionNumber = 0`, `Value = 0`) | high |
| P8 | Address assignment inside `.data` uses the **same** permutation as `.bss` for the same name set — i.e. one ordering rule, two sections | medium. Named alternative: `.data` is source order and only `.bss` permutes |
| P9 | Inter-object padding inside `.bss`/`.data` is the minimum needed to satisfy each object's natural alignment, applied **in layout order** (so total size depends on the permutation) | medium |
| P10 | Zero-initialized objects go to `.bss`, non-zero-initialized to `.data`, and an explicit `= 0` is indistinguishable from no initializer | high |

## The permutation — registered hypotheses, in the order I will test them

The §10.16 table is `2: s1 s2`, `3: s3 s1 s2`, `4: s4 s3 s1 s2`,
`5: s4 s3 s5 s1 s2`, `6: s6 s4 s3 s5 s1 s2`.

| # | hypothesis | why it is on the list |
|---|---|---|
| H-A | **Subset stability** — the order is the restriction of one total order over names, independent of which subset is present and independent of N | the §10.16 table is *consistent* with the single order `s6 s4 s3 s5 s1 s2`, but the lane reported it as "spliced by name" rather than testing stability. If H-A fails the rest is moot |
| H-B | **Hash-bucket walk with push-front** — order = (bucket index ascending, then reverse insertion within a bucket). Testable **without knowing the hash**: reversing the objects' source order flips the relative order of same-bucket names and leaves different-bucket pairs alone. That partition *is* the bucket assignment | it is the standard shape and the one §10.16 guessed |
| H-C | The key is the **source identifier**, not the decorated name (`sL` vs `?sL@@3UL@@A`) | distinguishable by `static` vs `extern` on the same identifier |
| H-D | The key includes something other than the name — declaration order, type, size, alignment, or the `.gl` record order | the null against H-A/H-B; killed if two objects with identical everything-but-name permute and two with identical names-but-different-type do not |
| H-E | The bucket count is **fixed** (no rehash), so a 40-object TU has the same relative order as any subset of it | if false, every rule must be N-dependent and the deliverable is a table, not a rule |

**Registered decline floor for question 4**, set now: if after (i) establishing
the bucket partition for ≥60 names and (ii) searching a named family of hash
functions (the MSVC/PDB `hashSz`, ELF `hashpjw`, FNV-1/1a, CRC-32 and JamCRC,
`h = h*K + c` for K in {31,33,37,131,65599}, and shift-xor forms
`h = (h<<k) ^ c`) crossed with bucket counts up to 1024 and both name forms
(source identifier / decorated), **no member reproduces the partition**, the
lane **declines** and delivers the measured table plus the refuted list.

Registered in advance as an acceptable *partial* win: the **bucket partition**
itself is a deliverable even without the hash, because it converts "unknown
permutation" into "known equivalence classes with a known within-class rule".

## Structural axes for the grid

"A generated axis is only as good as the axes it varies", and this has bitten
the project three times on *structural* axes. Registered axes, all structural:

* **object count** N ∈ {1,2,3,4,5,6,8,12,20,40}
* **declaration order** — same name set, permuted source order (this is the
  H-B discriminator, so it is the most important axis in the grid)
* **name length** — 1, 2, 3, 8, 16, 32, 64 characters
* **name lexical order** — sets that are already sorted, reverse-sorted, and
  shuffled; sets differing only in the **first** vs only in the **last**
  character
* **type size** 1, 2, 4, 8, 16, 64, 256 B, and **alignment** via
  `__declspec(align(n))`
* **`static` vs `extern`** — and mixed within one TU
* **initialized vs not** (`.data` vs `.bss`), and mixed within one TU
* **objects split across headers** vs all in the TU
* `const`, `volatile`, `__declspec(selectany)`, arrays, and a struct with a
  constructor (the §10.16 shape) as the value axis inside cells

## Controls

* **Verify the probe made the structure I think it did before reading a result
  off it** — every probe is checked for its expected section count, `.bss`/`.data`
  size and symbol count before any ordering is read from it. A probe whose
  object was optimized away silently would otherwise read as a permutation.
* Grade at **`/O1`**, the workload's mode. `/Ox` does not imply `/GF`.
* Bytes come from the **obj**, never the `/FAsc` listing (§10.16).
* Never hand-write an expected obj; the real `c2.dll` is the sole judge.

## What would make this lane wrong-and-green

Reporting a permutation rule that reproduces every cell in a grid whose names
are all `s<digit>`. The held-out set must contain names from the **real
workload**, taken from the cached reference objs, not names I invented.
