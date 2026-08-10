// **W-MAIN2 — the `/EHsc` scope-object TU, the class `src/Main.cpp` is.**
//
// One function, one destructible local, one call while it is live. The obj is
// **two code regions in one `.text` COMDAT** — the body and an `__unwind$N`
// funclet — **two `.pdata` COMDATs in reverse region order**, and a 64-byte EH
// `.rdata` carrying `__unwindtable$`, `__ehfuncinfo$` and the ip-to-state array.
// The function symbol's `Value` is **8**, behind an ADDR32 prefix naming
// `__CxxFrameHandler` and its own `__ehfuncinfo$`.
//
// **NO `c2rs-profile:` line, deliberately** — the same reason
// `wvsnprnc_guard_chain_arity_store.cpp` has none. Two of the class's gates are
// flag-shaped: it is `/O1`-only (the mode is read from the `.ex` opt word in
// `PortC2::build`, board #1638's rule) and it needs `/EHsc`, without which c2
// emits no funclet, no second `.pdata` and no EH `.rdata` at all. At the fixture
// path's default `/Ox /GS- /c` this file is therefore `NotImplemented`, which is
// correct. **The grading happens in `scripts/gate.sh`'s `/O1` mode lanes** —
// `/O1 /EHsc`, `/O1 /Oi /EHsc` and `/O1 /Oi /EHsc /GR` — which put every fixture
// through `c2rs gap --flags-file`, and at the workload's own profile on
// `src/Main.cpp` itself.
//
// **ONE function on purpose.** `IlBundle::eh_scope_tu` refuses a TU with more
// than one `.ex` segment, and that is not tidiness: the `__unwind$N` funclet
// label is the one number of the ten this class mints that is **not** at a fixed
// offset from `coff::plan_labels`' cursor. It reads `B−2` when the EH function
// is the TU's first and `B+0` when anything precedes it, and the six cells of
// `work/w-main2/LABELS.md` do not separate the two readings that fit them. At
// one function only the `B−2` branch can fire, and it is measured on three
// distinct `.gl` seeds.
//
// The class of the local matters only through its `sizeof`, which is read out of
// `.db`'s `LF_CLASS` and sets the frame: `align16(80 + 4 + 8 + 8) = 112`, so the
// prologue's `stwu` immediate is `−112` and the object's address is `r31+80`.

class App {
    int _pad;

public:
    App(int, char **);
    ~App();
    void Run();
};

int main(int argc, char **argv) {
    App app(argc, argv);
    app.Run();
}
