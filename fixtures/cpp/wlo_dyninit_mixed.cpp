// **Both body forms in one `.ex`** — board #158, the decode half (w-lo).
//
// The inline constructor is a source function and opens `4C 4F 11`; the
// `??__EsL@@YAXXZ` thunk the namespace-scope object forces opens the bare `4C`
// (ROADMAP §10.12's table, rows 5 and 6). One capture, one `.ex`, both forms.
//
// **This is the fixture that grades the "strictly additive" claim.** The
// splitter's second pass is offered only those `4F 1F` regions that hold no
// `4C 4F 11` at all, so the constructor's segment must come out byte-identical
// to what it was before the pass existed while the thunk's segment appears
// beside it. A rule that anchored on the bare `4C` globally would re-split the
// constructor's body — its `IntCallEnd` ends with a `4C` and a `VoidCallEnd`
// begins with one — and that is invisible to a byte compare of the obj, because
// a TU this far out of class never reaches the emitter.
//
// `Port=NotImplemented`: the obj carries `.bss` and `.CRT$XCU` beside `.text`.

struct L {
    int v;
    L(int a) : v(a) {}
};

static L sL(3);
