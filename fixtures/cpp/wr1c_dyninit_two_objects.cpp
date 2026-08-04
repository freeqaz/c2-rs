// **W-R1c negative cell — two objects, which must REFUSE.**
//
// This is the fixture lane w-bss's finding demands. `OBJ_DYNINIT_SHAPE.md` §4.1
// says "`.bss` and `.CRT$XCU` are always exactly one each, **always last**, in
// that order", and `OBJ_DATA_BSS_SHAPE.md` §2.2 refutes it: that is true only of
// the dyninit-only TU class it was measured on.
//
// Two things go wrong here at once, and either alone is a different obj:
//
//  1. **The section order moves.** A plain `char b1;` beside a dynamic-initializer
//     object makes the two share one `.bss`, and the shared section is placed by
//     its *earliest* contributor — so it moves out from behind `.text$yc` and
//     **between the two `.XBLD$W` watermarks** (§2.2 rows 11 vs 12). The port
//     emits the dyninit-only layout and would be wrong at the section table.
//  2. **The `.bss` addresses permute.** With N ≥ 2 objects the offsets are not
//     source order; §5.2 resolves the rule (`.gl` record order for eager objects,
//     its exact reverse for deferred ones, never interleaved) but this port does
//     not implement it.
//
// `IlBundle::dyninit_tu` refuses on the *first* fact, structurally: it requires
// exactly one uninitialized object in the whole TU. `NotImplemented` at every
// lane is the correct verdict here, and a `match` would mean the count gate
// stopped working.

struct L { L(const char* s, int r); int a, b, c; };
static L sL("abc", 0);
static char b1;
