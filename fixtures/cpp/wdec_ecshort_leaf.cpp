// w-decouple — the ACCEPTED cell, and the first obj this project has graded for
// a DEFINED function symbol whose name uses the 8-byte COFF INLINE name field.
//
// `inc` is three bytes, so `coff::symbol::emit_symbol` takes its `b.name8`
// branch rather than interning into the string table. That branch is not new —
// the port has emitted `memcpy` (six bytes) through it, byte-exact, on every TU
// of `w-ifn`'s class — but it has only ever been graded on an **undefined
// external**. The DEFINED side was refused by `gl_defined_names`'
// `INLINE_NAME_MAX` clause, which W-EXTDATA left in place because *"that is the
// half no capture has graded"*.
//
// This is that capture. Board **#2374** measured the refusal (`ecshort`
// vocab-gap / `eclong` match, on the same four bytes) and never the accept.
//
// Two live workload TUs are the reason it matters: `src/Main.cpp`, whose one
// `.gl` record is `main` — four bytes, binding to its single `.ex` segment at
// offset 2713 exactly — and `src/xdk/nuispeech/mmio.cpp`, which binds four of
// eleven records and stops at `mmioSeek`, eight bytes, exactly at the bound.
// Both were `gl-stop-name-not-mangled` and both bind at this lane's tip.

extern "C" int inc(int a) { return a + 1; }
