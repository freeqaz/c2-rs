// **Negative** — a `#pragma comment(lib, …)` changes `.drectve`, and the port
// emits a fixed one. Live wrong-bytes emit, at obj file offset 8.
//
// `.drectve` is the linker-directive section, and until now it was pure boilerplate
// — the same 132 bytes in every reference obj, so the port emits it as a constant
// and the fixed four-section shell was right by construction:
//
//   /include:__C1_11886 /DEFAULTLIB:"OLDNAMES" /DEFAULTLIB:"LIBCMT"
//   /DEFAULTLIB:"XAPILIB" /DEFAULTLIB:"XBOXKRNL" /include:__C2_11886
//
// One `#pragma comment(lib, "somelib")` splices `/DEFAULTLIB:"somelib"` in after
// the first directive, making it 154 bytes. Every later section's file offset
// shifts by 22, so the first divergence is at offset 8 — `PointerToSymbolTable`
// — and the function body, which is byte-exact, never gets a chance to matter.
//
// This is the same shape of bug as `il_gl_data_symbol.cpp`: the port models
// function bodies and treats the rest of the obj as a constant, so anything that
// perturbs the shell is invisible to it. Both were found by probing TU-level
// structure rather than expressions, which is the axis the fixture corpus had
// almost no coverage of — every fixture until now was a body in a default shell.
//
// The pragma is not in the IL bundle's `.ex` at all; it reaches c2 through `.gl`
// or the argv. Whether the port should refuse (this) or model the directive list
// depends on a capture matrix over the `comment` pragmas (`lib`, `linker`,
// `user`, …) that does not exist yet, so it refuses on the honest ground that it
// cannot see the input that would decide.

#pragma comment(lib, "somelib")

int w_add(int a) { return a + 1; }
