// **Negative** — a TU that DEFINES a data symbol must refuse.
//
// This was a live wrong-bytes emit, found by probing `.gl` record ordering rather
// than by any fixture. `int gv;` puts `?gv@@3HA` in `.gl` and gives c2's obj an
// extra section for it; the port emitted its fixed four-section shell and
// mismatched at **file offset 2** — `NumberOfSections`. The function itself was
// byte-exact. Nothing in the port ever looked at a `.gl` symbol that was not a
// function name, so a data definition was simply invisible.
//
// A defined static data member (`struct S { static int sm; }; int S::sm = 4;`)
// did the same thing, so this is a class and not one spelling.
//
// The gate is the accounting rule in `IlBundle::functions`: every mangled run in
// `.gl` must be claimed by a function record (via its framed body-start offset)
// or be a callee that some body resolved. `?gv@@3HA` is neither, so the TU is out
// of class. Note that `extern int gv;` and `int gv;` mangle identically, so this
// refuses both — which costs nothing, because reading a global is already out of
// class and c2 does not list an extern that is never referenced.
//
// `w_add` is deliberately the plainest in-class body there is. The point is that
// a perfectly modeled function is not enough when the TU around it is not.

int gv;

int w_add(int a) { return a + 1; }
