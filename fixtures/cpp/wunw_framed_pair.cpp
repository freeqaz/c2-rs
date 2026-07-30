// W-UNW-1: TWO framed functions in one TU, sharing one callee. The first
// multi-framed obj in the corpus, and the reason the `.pdata` emitter had to
// stop being a single-function special case.
//
// Packed (`/Ox`): ONE `.pdata` section holding two 8-byte records in `.text`
// order, two ADDR32 relocations (at 0x0 and 0x8, each against its own function
// symbol), and the section's aux CheckSum over all 16 bytes. `$T` values are
// the records' offsets: 0 and 8.
//
// `/Gy` (so also `/O1`, `/O2`): each framed function gets its OWN `.pdata`
// COMDAT, emitted immediately after its `.text` COMDAT with
// IMAGE_COMDAT_SELECT_ASSOCIATIVE and the aux `Number` field naming that
// `.text`'s section number.
//
// It also pins the framed label stride: 4 counter slots packed, 5 under `/Gy`,
// so `f2`'s `$M` numbers are the only thing that separates a right answer from
// a plausible one (`?f1` 2548/2549/2550 then `?f2` 2552/2553/2554 packed).
int g(int);
int f1(int a) { return g(a) + 1; }
int f2(int a) { return g(a) + 2; }
