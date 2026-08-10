// **W-MAIN2 `_neg` — ONE cell, ONE clause, and accepting it is a WRONG EMIT.**
//
// This file is the positive fixture with an explicit `return 0;`. `main` is the
// one function C++ lets you leave without one, and the two spellings are
// **different compilations**: the `.ex` grows a value assignment and a `3A`
// where the positive class has `5E 01 21 4B` straight after the member call's
// `4C 4B`, and `func::ehscope::match_template` requires that literal.
//
// **The wrong emit it fences, measured rather than asserted**
// (`work/w-main2/probe/n1.cpp`, real `c2.dll` at the workload's flags):
//
// ```text
//   no `return`   seed 2551   __unwind$ seed+10   $M seed+15 +16   $M seed+19 +20
//   `return 0;`   seed 2552   __unwind$ seed+11   $M seed+16 +17   $M seed+20 +21
// ```
//
// **Every one of the ten labels moves by +1** — the explicit return takes one
// more slot of the compiler-label counter — while the 124 `.text` bytes, both
// `.pdata` records and the 64-byte EH `.rdata` are **byte-identical**. So an
// accept here is not a refusal that should have been a match: it is ten wrong
// symbol names in an obj that still links, which is the whole reason
// `docs/LABEL_COUNTER.md` exists.
//
// **The must-fail mutation deletes the WHOLE conjunction** (#2698/#2699), and
// it has to: the tail clauses `5E 01 21 4B` and `54 02 29 <exit>` are adjacent,
// so relaxing only the first leaves the second standing on the return value's
// own bytes and the cell would go on refusing for a reason the fixture is not
// about. The mutation is *"after the member call's `4C 4B`, skip to the first
// `54 02 29`"* — one edit, both literals gone — and it is run in
// `work/w-main2/MUTATION.txt`.
//
// The three sibling clauses — a reordered statement list, a `maxState` other
// than 1, and a second scope object (`5E 02 21`) — are graded by
// `func::ehscope::tests`, each on a MUTATION of the workload TU's own captured
// segment, because each needs a `.ex` shape this one cell cannot also be. They
// are named here rather than counted: **this fixture grades the tail clause and
// nothing else.**
//
// Like its positive sibling this file carries no `c2rs-profile:` line and is
// `NotImplemented` at the fixture path's default `/Ox /GS- /c`; the `/O1 /EHsc`
// mode lanes of `scripts/gate.sh` are what grade it.

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
    return 0;
}
