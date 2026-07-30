// **Positive** — the two fields of a `.sy` type's *extent* that the reader used to
// fold into one `u16`, each pinned by a case where the two readings disagree.
//
// The old reading was `<size16 LE> <flags16 LE>`. The real layout is
//
//   <size varint> <one unnamed byte> <flags16 LE>
//
// and the varint is the same 1-or-5-byte form (`80` + LE32 above 0x7F) that the same
// function already used for a `07` static record's size — with a comment warning
// about exactly this mistake. Below 128 bytes with a zero following byte the two
// readings are byte-identical, which is why no probe caught it and why every
// discriminating case here is either large or by-reference.
//
// WHY IT IS GRADED BYTE-EXACT AND NOT BY CENSUS. Every function here is in class, so
// c2 emits an obj and the compare is the judge. That is deliberate and it is the
// hard part of writing this fixture: the natural probe — declare a big array and use
// it — puts the *declaring* function out of class, and a translation unit with any
// refused function emits no obj at all, so its positive cases would be graded by
// nothing. Leaving the array unreferenced keeps the body a bare member load while
// `.sy` still records the local at its true size, which is the only fact under test.
//
//   f127  size 127  -> varint 1 byte    both readings agree; the control
//   f128  size 128  -> varint 80 80 00 00 00   FIRST value where they diverge
//   f300  size 300  -> varint 80 2c 01 00 00
//
// On the reader as it stood, `f127` bound and `f128` did not: the `u16` read of
// `80 80` yields 32,896, `read_tid` then lands on a `00`, the record ends four bytes
// early and — because `.sy` binds a translation unit 1:1 or not at all — EVERY
// function in the file loses its formal widths. One 128-byte local array anywhere in
// a translation unit cost every function in it. That is the mechanism behind the
// `param-width-undetermined` census key standing at 567,549 functions over 878
// translation units while its sibling `param-multi-reg` stood at 1.
//
// `byref` pins the second field. A class with a user copy constructor is passed by
// hidden reference, so `.sy` records the parameter as kind `03` — a POINTER — with
// size 4, and the unnamed byte after the size is `08`:
//
//   86 03 00 03 | 04 | 04 | 08 | 00 00 | 80 0c 10 00 00
//
// The `u16` read makes that 0x0804 = 2052, which trips the `size > 8` test and reports
// `param-multi-reg` — a decode error landing in the bucket that ranks missing
// FEATURES rather than reader gaps, which is worse than a refusal. The truth is a
// 4-byte pointer in one register, and c2 agrees: `byref` is byte-exact.
//
// `byref`'s tag is the NARROW `86`, which matters: `fixtures/cpp/il_param_poly.cpp`
// carries the same `08` byte behind a WIDE `C6 81` prefix, so the two corrections are
// observed separately here and there rather than as one coincidence.
//
// NOT implemented and deliberately so: any size-to-register-count rule. `ceil(size/8)`
// is not merely unproven, it is refuted — a 16-byte class with a copy constructor and
// a 12-byte class with a vtable each take ONE register by hidden reference, so the
// size does not determine the count. Widths above 8 bytes still refuse.

struct H { int mi; };
struct CC { int a, b, c, d; CC(const CC&); };

int f127(int a, H* h) { char buf[127]; return h->mi; }
int f128(int a, H* h) { char buf[128]; return h->mi; }
int f300(int a, H* h) { char buf[300]; return h->mi; }
int byref(CC v, H* h) { return h->mi; }
