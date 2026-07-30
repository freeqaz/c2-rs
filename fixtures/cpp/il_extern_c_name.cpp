// **Negative** — an undecorated (`extern "C"`) function name must refuse, and
// this is the fixture for a regression that a fix *introduced*.
//
// `il_gl_record_order.cpp` replaced positional name pairing with a per-record
// binding: a `.gl` record's name is the mangled run preceding its framed
// body-start offset field. "Preceding" was implemented as *nearest preceding
// mangled run*, and a mangled run was one containing `@@`. An `extern "C"` name
// has no `@@`. So this TU's second record was skipped by the name scan and
// borrowed the **first** record's name: both bodies bound to
// `?w_mangled@@YAHH@Z`, and the obj carried two `.text` symbols with the same
// name. Wrong bytes at obj offset 804.
//
// Worse, it was order-dependent — putting the `extern "C"` function first made
// the *first* record nameless, the binding returned nothing, and the TU refused.
// So one order was a silent mis-emit and the mirror image was a clean refusal.
//
// The `.gl` record layout settles it. Both records are the same shape:
//
//   00 <name> 00  86 01 05 04 00 00 00  80 01 10 00 00 00 00  80 <LE32 offset>
//                 \___ TYPE ________/  \___ framing _______/
//
//   00 '?w_mangled@@YAHH@Z' 00 … 80 54 0a 00 00     offset 2644
//   00 'c1'                 00 … 80 b7 0a 00 00     offset 2743
//
// The name is not "the nearest mangled run", it is **the run immediately
// preceding this record's framing** — positionally determined, exactly like the
// offset field itself, and identifiable without looking at the name's contents
// at all. Fifteen bytes separate the name's terminating NUL from the offset field
// in both records here, and nineteen in a `void()` record (`il_gl_record_order`),
// the difference being the TYPE field's width.
//
// So the name is now read positionally and *then* judged: a record name that is
// not a mangled `?…@@…` function name refuses the TU. That makes `extern "C"`
// out of class **positively**, rather than by the name scan failing to see it —
// the same distinction `il_stmt_global.cpp` had to make about locals. Absence
// from a scan proves nothing; it only says the scan did not match.
//
// Refusing is right rather than conservative. c2 stores `c1` **inline in the
// 8-byte COFF symbol name field** (`[14] c1 INLINE val=8 sec=5`) instead of in
// the string table, which every mangled name uses — a different encoding path,
// characterized by exactly one capture, and nothing yet says what it does to
// storage class or to `/Gy` COMDAT naming.

int w_mangled(int a) { return a + 1; }
extern "C" int c1(int a) { return a + 2; }
