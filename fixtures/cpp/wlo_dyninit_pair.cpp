// **Two bare-`4C` bodies in one TU** — board #158, the decode half (w-lo).
//
// `il_dyninit_static.cpp` carries ONE `??__E` thunk. This one carries two
// bodies with the same shape, because a namespace-scope object with a
// non-trivial constructor **and** destructor makes c2 emit both halves:
//
//   ??__EsL@@YAXXZ   `dynamic initializer for 'sL''
//   ??__FsL@@YAXXZ   `dynamic atexit destructor for 'sL''
//
// Both open with the bare `4C` (ROADMAP §10.12's table, rows 3 and 4). The
// fixture exists so the splitter's second pass is exercised against **two**
// marker-less segments in one `.ex` rather than one: a rule that finds the
// first such body and stops would pass `il_dyninit_static.cpp` and fail here.
//
// `Port=NotImplemented` and expected to stay so until the obj shape lands —
// this TU's obj carries `.bss` and `.CRT$XCU`/`.CRT$XTX` beside `.text`, and
// the port emits a fixed four-section shell. The decode is the other half.

struct L {
    L();
    ~L();
};

static L sL;
