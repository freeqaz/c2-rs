// w-decouple — the BOUNDARY cell: a defined name of EXACTLY eight bytes.
//
// `INLINE_NAME_MAX` is 8 and the clause was `len() <= INLINE_NAME_MAX`, so
// eight is the last length the incumbent walk refuses and the first this one
// has to get right. It is not a hypothetical boundary: `mmioSeek` is exactly
// eight bytes, and it is the record `src/xdk/nuispeech/mmio.cpp`'s walk stops
// on (board #2624) — four records bound, seven never reached.
//
// `seekhere` is eight; `n` is one, which is the other end of the same field.

extern "C" int seekhere(int a) { return a + 1; }
extern "C" int n(int a) { return a + 2; }
