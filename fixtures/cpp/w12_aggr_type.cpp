// Aggregate TYPEs that straddle the inline-size boundary — the `read_type`
// aggregate rule, captured end to end.
//
// **This fixture is a pure negative: every function here must be
// `NotImplemented`.** A struct copy needs `memcpy`-class codegen the port does
// not have, and `is_int4_type`/`is_ptr_to_4` are false for kind class 6, so
// aggregates are 100% refused. Nothing here is a widening step, and if any of
// these ever lands in class that is the alarm, not the win.
//
// READ THIS FIRST — what this fixture does NOT witness
// ----------------------------------------------------------------------------
// It does not discriminate the fixed `read_type` from the broken one. MEASURED:
// `c2rs census` prints byte-identical output for all four functions before and
// after the fix, because the parse refuses at the **base pointer LOAD** of the
// first statement and never reaches the aggregate TYPE two bytes later. The same
// is true of every other shape probed (a by-value struct return stops at the
// `9B` sret bind; a call returning a big struct stops there too), and a full
// 878-TU workload scan enters the aggregate branch **zero** times.
//
// So: the unit tests in `crates/c2-il/src/func/readers.rs` are what test the
// rule; this file's job is narrower and worth stating exactly, because a fixture
// that is assumed to prove more than it does is how this project lost weeks
// before (`docs/GAPS.md` §6, "a truncated fixture cannot witness the region it
// omits"):
//
//   1. it pins the four TYPE byte-forms as **real `cl.exe` output** rather than
//      as a table transcribed from a probe that is no longer on disk;
//   2. it holds the refusal — if a future rung widens the leaf and starts
//      admitting these, `c2rs bench` says so here first;
//   3. it holds `ReferenceReplay=ByteExact`, so the capture the byte-forms come
//      from is itself trustworthy.
//
// The captured `.ex` (`/Ox`, one line per function, `>` = the blocking byte):
//
//   cp31  >b9< 03 0a 86 43 81 20  b9 …  30 83 f6 80 20     32 83 f6 80 20
//   cp32  >b9< 07 0a 86 43 88 20  b9 …  30 82 06 20 87 20  32 82 06 20 …
//   cp33  >b9< 0b 0a 86 43 8f 20  b9 …  30 82 06 21 8e 20  32 82 06 21 …
//   cp40  >b9< 0f 0a 86 43 96 20  b9 …  30 86 06 28 95 20  32 86 06 28 …
//
// Every one of those matches `docs/IL_LOAD_TYPES.md` §1a's ladder — `cp32` and
// `cp33` byte-for-byte including the ids, from an independent capture — which is
// a second witness for the rule, not a restatement of the first.
//
// Why these four sizes
// ----------------------------------------------------------------------------
// An aggregate's size is a 5-bit field spread across tag bit 0 (as bit 4) and the
// kind's high nibble; when it does not fit, the field reads 0 and a statement
// varint carrying the real size is inserted **between the kind and the LEB id**.
//
//   * `A31` (31 B, align 1) is the **top of the inline field** — tag bit 0 set
//     plus kind high nibble F, `83 F6`. It is the neighbour a wrong "the size is
//     only the kind's high nibble" rule reads as 15, and a wrong "class 6 always
//     carries a varint" rule tries to read a size out of its id byte. 4 bytes.
//   * `A32` (32 B, align 1) is the **first size that cannot fit**, so the first
//     to carry the varint. 5 bytes. The 31/32 pair is load-bearing: it is the
//     only place where one byte of source flips the *encoding form*, so a reader
//     wrong about the boundary is wrong on exactly one of them.
//   * `A33` (33 B, align 1) is the discriminator against the plausible wrong
//     reading "the bytes after the kind are a fixed class token" — under that
//     rule one byte of struct growth could not move them, and here the third
//     byte moves `20`→`21` while tag and kind hold still. Note the ids move
//     *independently* (`87 20` → `8e 20`), which is what separates a size field
//     from a wider id.
//   * `T40` (40 B, align 4) carries the **other tag family**. `A31`…`A33` are
//     align 1 (tag `82`/`83`); the wild witness the rule was decoded against —
//     `src/system/meta/Sorting.cpp` `.ex` 0xc7e3,
//     `4c 30 86 06 80 14 10 00 00 a5 29 4b`, a 4,116-byte object — is align 4
//     (tag `86`). The alignment nibble and the size bit are separate fields, so a
//     fixture with only align-1 structs cannot witness the case the real workload
//     contains.
//
// Include-free by design (fixtures/README.md); `char`/`int` arrays fix the sizes
// and the alignments without a `#pragma pack`, which would add a second variable.

struct A31 { char b[31]; };
struct A32 { char b[32]; };
struct A33 { char b[33]; };
struct T40 { int  b[10]; };

void cp31(A31* d, const A31* s) { *d = *s; }
void cp32(A32* d, const A32* s) { *d = *s; }
void cp33(A33* d, const A33* s) { *d = *s; }
void cp40(T40* d, const T40* s) { *d = *s; }
