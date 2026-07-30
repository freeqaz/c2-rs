// **Negative** — `__declspec(dllexport)` makes c2 splice `/EXPORT:<name>` into
// `.drectve`, which this port emits as a **constant**.
//
// That was a live wrong-bytes emit on a one-line getter, found by adversarial
// review:
//
//   __declspec(dllexport) int de(H* h) { return h->mi; }
//   -> census 1/1 functions in class,  Port=Mismatch @ offset 8
//
// Offset 8 is `PointerToSymbolTable`. The body was byte-perfect; the `.drectve`
// section grew by the directive, every later section's file offset shifted, and the
// obj diverged in the COFF header long before any instruction mattered. Exactly the
// failure shape `#pragma comment(lib, …)` already produces
// (`il_drectve_pragma.cpp`), reached by a different route — which is why the fix
// belongs beside `drectve_is_boilerplate` rather than in a body gate.
//
// MEASURED, and the measurement is the interesting part. Diffed against a plain twin
// in a file of the SAME BASENAME — differing names pollute a `.gl` diff through both
// the embedded source path and its checksum, which is what made a first attempt at
// this look like eighteen differing bytes instead of one. `.ex` and `.sy` are
// byte-identical; the single differing byte is a linkage field:
//
//   ?de@@YAHPAUH@@@Z\0      86 01 09 04 …   dllexport
//   ?de@@YAHPAUH@@@Z\0      86 01 05 04 …   plain
//
// A defined function's record continues after its name's NUL with a **two-byte**
// `<tag> <kind>` return type, then this byte, then the return size. The two-byte
// width was checked against the fourteen return types most likely to break it — a
// 20-byte aggregate, a reference, an enum, a class, a function pointer, `void`,
// `double`, `long long`, `bool`, `short` — and held at every one, so the field is at
// a fixed offset rather than behind a variable-width type. An earlier reading, from
// one `int`-returning probe, had assumed the opposite and nearly produced a
// positionally fragile gate.
//
// The gate is a **known-bad bit test (0x08), not a known-good allowlist**, and the
// weakening is deliberate. Values `03` (internal) and `05` (external) are the ones
// seen on defined functions in probes, but across every `?`-mangled run of six real
// translation units that byte takes `{0, 3, 4, 5, 6}` — and those runs include
// externals, callees and vtable symbols rather than only the records this gate
// applies to. Requiring `{03, 05}` could therefore refuse a real defined function
// carrying a fourth value and regress a TU that matches today; refusing on bit
// `0x08` cannot. The cost, so it is not read as completeness: a linkage needing some
// *other* directive, without this bit, still mis-emits.
//
// `plain` must stay in class, and is what says so if this over-reaches. The positive
// twin lives in `il_expr_member.cpp` and the other getter fixtures, which would stop
// matching if this rule caught an ordinary function.
//
// A census/gate asymmetry to know about before reading this file's census output:
// **the census reports 2/2 functions in class here**, including the exported one. The
// gate lives in `gl_defined_names`, which the census does not use — it has no name
// bound to a segment on real input, the same reason `fn-varargs` never fires on the
// workload. So the gate is STRICTER than the census reports, which is the safe
// direction, and the TU-level `Port=NotImplemented` is the thing that grades this
// rule. Do not read `2/2` as evidence the export is handled.

struct H { int mi; };

__declspec(dllexport) int de(H* h) { return h->mi; }

int plain(H* h) { return h->mi; }
