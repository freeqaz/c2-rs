# Pre-registration — the `.data` aux `CheckSum` float exclusion (2026-08-04)

Lane `w-bss`, addendum to
[`_2026-08-02-w-bss-prereg.md`](_2026-08-02-w-bss-prereg.md). **Written and
committed before the 11 cells below were compiled.** Scored verbatim in
`docs/OBJ_DATA_BSS_SHAPE.md` §4.2; wrong predictions stay on the page.

## What this is testing, and why it needs a designed grid

`OBJ_DATA_BSS_SHAPE.md` §4.2 establishes that the `.data` section symbol's aux
`CheckSum` is **CRC-32, polynomial `0xEDB88320`, init `0`, no final XOR** over the
section's raw data. Two independent derivations agree: this lane's 9 probe cells,
and a blind re-derivation over the 871-obj workload census, where `zlib.crc32`
(init `0xFFFFFFFF`, final XOR) matched **0 of 9,139** sections and the init-0 form
matched 9,087. That agreement with the project's prior independent characterization
of the same polynomial is the known-answer control that licenses everything below.

**52 of 9,139 workload sections do not match**, and every one of them contains
floating-point initializers. The census's reading is that *the bytes written by
c2's floating-point initializer path are excluded from the running CRC*. The
evidence for it is good — one section's CheckSum is exactly the CRC of only its two
`int` words, another's is the CRC of a single `0x01` byte — but that is a
**mechanism inference fitted to 52 found cases**, and found cases cannot separate:

* whether the exclusion is byte-granular or word-granular;
* whether inter-object padding is excluded along with the FP object;
* whether it is "FP" at all, rather than "8-byte-aligned", "unrelocated", or
  something about the order the initializers were emitted.

Accumulating more found cases cannot settle that. A grid that varies **only** the
count, size and placement of float vs int initializers within one section can.

## The registered distinction between specification and hypothesis

Registered now, so it cannot be blurred later:

* **Specification (safe, and what a writer implements):** the `CheckSum` is a
  CRC-32/`0xEDB88320`/init-0/no-final-XOR over a **subset** of the section's raw
  bytes. On 9,087 of 9,139 workload sections that subset is all of them.
* **Hypothesis (fenced):** the subset omits the bytes written by the FP
  initializer path.

If the grid refutes the hypothesis, the specification survives unchanged and the
doc must say the predicate is unknown — **not** quietly widen the hypothesis to fit.

## The grid

11 cells, each a TU of namespace-scope external-linkage initialized objects that
share one non-COMDAT `.data`. Flags: the workload's,
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`.

| cell | source |
|---|---|
| f0 | `int a=1; int b=2;` — control, no FP |
| f1 | `float f=1.0f;` |
| f2 | `double d=1.0;` |
| f3 | `int a=1; float f=1.0f;` |
| f4 | `float f=1.0f; int a=1;` |
| f5 | `int a=1; float f=1.0f; int b=2;` |
| f6 | `int a=1; double d=1.0; int b=2;` |
| f7 | `float f=1.0f; float g=2.0f;` |
| f8 | `char c=1; float f=1.0f;` |
| f9 | `char c=1; char e=2; float f=1.0f; char g=3;` |
| f10 | `float p[2]={1.0f,2.0f}; int a=1;` |

## Registered point predictions

Predicted **layout** comes from `OBJ_DATA_BSS_SHAPE.md` §5.3 (`.data` walks
declaration order) and §5.4 (size-promoted alignment, padding becomes a reusable
hole). Predicted **CheckSum** is then the CRC over the predicted bytes under each
variant. **Both halves are being graded**: the layout is an out-of-sample test of
§5.4, and f6 is the cell that tests hole reuse (`b` is predicted to land at offset
4, *inside* the hole `d`'s 8-alignment opens, not after `d`).

| cell | predicted layout | predicted raw | ALL (no exclusion) | **VAR-A** drop FP bytes, keep padding | VAR-B drop FP bytes *and* padding | VAR-W drop FP *words* |
|---|---|---|---|---|---|---|
| f0 | `a@0 b@4`, size 8 | `00000001 00000002` | `0xD36E489C` | `0xD36E489C` | = | = |
| f1 | `f@0`, size 4 | `3F800000` | `0x497DF377` | `0x00000000` | = | = |
| f2 | `d@0`, size 8 | `3FF00000 00000000` | `0x38937B08` | `0x00000000` | = | = |
| f3 | `a@0 f@4`, size 8 | `00000001 3F800000` | `0x741DDAC7` | `0x77073096` | = | = |
| f4 | `f@0 a@4`, size 8 | `3F800000 00000001` | `0xA3FC9BB5` | `0x77073096` | = | = |
| f5 | `a@0 f@4 b@8`, size 0xc | `00000001 3F800000 00000002` | `0x2D8EDE4C` | `0xD36E489C` | = | = |
| f6 | **`a@0 b@4 d@8`**, size 0x10 | `00000001 00000002 3FF00000 00000000` | `0xCBF38A0B` | `0xD36E489C` | = | = |
| f7 | `f@0 g@4`, size 8 | `3F800000 40000000` | `0x4FEFF31E` | `0x00000000` | = | = |
| f8 | `c@0 f@4`, size 8 | `01 000000 3F800000` | `0x85D7F3E9` | **`0xB8BC6765`** | **`0x77073096`** | `0xB8BC6765` |
| f9 | `c@0 e@1 g@2 f@4`, size 8 | `01 02 03 00 3F800000` | `0x94DC906E` | **`0x9015E0C8`** | **`0xAAFD590F`** | `0x9015E0C8` |
| f10 | `p@0 a@8`, size 0xc | `3F800000 40000000 00000001` | `0x4FC66E91` | `0x77073096` | = | = |

**Registered primary prediction: VAR-A.** The exclusion is byte-granular and drops
exactly the FP objects' own byte ranges, leaving alignment padding in the CRC.
Confidence: medium. **f8 and f9 are the only two cells that discriminate**
VAR-A from VAR-B, and f8/f9 are also the only cells where VAR-W could differ from
VAR-A (it does not here, because `c` and `f` are already word-separated — so a
VAR-A hit leaves byte-vs-word granularity **still open**, and I register that in
advance rather than claiming the grid settles it).

Registered secondary predictions:

* **P-A** f0 is unchanged by every variant and must equal `0xD36E489C`. If f0 misses,
  the CRC characterization itself is wrong and nothing else in the table means
  anything. This is the control.
* **P-B** f1, f2, f7 — sections containing *only* FP — have `CheckSum = 0x00000000`.
  High confidence: §4.2 already measured `double d8=8.0` → `0`.
* **P-C** f4 and f3 have the **same** CheckSum as each other. Placement of the FP
  object relative to the int object does not matter, only which bytes it occupies.
* **P-D** f10 (an FP *array*) behaves exactly as an FP scalar. Medium — a
  differently-sized initializer could plausibly take a different emit path.
* **P-E** the predicted **layouts** are all correct, including f6's hole reuse.
  This is §5.4 graded out of sample; f6, f8 and f9 are the cells that can refute it.

## Registered decline

If the measured CheckSums match **none** of ALL / VAR-A / VAR-B / VAR-W on the
mixed cells, the lane records the grid as data, states that the exclusion predicate
is **unknown**, and leaves the safe specification (a CRC over some subset) as the
deliverable. Declining here is a good outcome; silently refitting a fifth variant to
whatever comes back is not, and is pre-emptively ruled out.

If the layouts miss, that is a refutation of §5.4 and must be reported as one in
`OBJ_DATA_BSS_SHAPE.md` §5.5 — not absorbed into the CheckSum discussion.
