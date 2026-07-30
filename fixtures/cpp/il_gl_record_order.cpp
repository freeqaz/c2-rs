// **Negative**, and the case that disproves positional name pairing.
//
// `functions()` used to name the Nth body with the Nth `?…@@…` run in `.gl`. This
// TU's `.gl` lists, in order:
//
//   ??__Egs@@YAXXZ    the dynamic initializer c2 synthesizes for `gs`
//   ?w_add@@YAHH@Z
//   ?gs@@3US@@A       data
//   ??0S@@QAA@XZ      the constructor, external
//
// and `.ex` has two bodies. `mangled_names` requires the second byte to be
// alphabetic, so it never saw either `??`-prefixed name and returned
// `[?w_add@@YAHH@Z, ?gs@@3US@@A]` — two names for two bodies, which *looked*
// paired. It would have named the second function after a **variable** and
// emitted a `.text` symbol called `?gs@@3US@@A`.
//
// That never fired only because parsing the initializer thunk's body failed
// first, so the TU refused for an unrelated reason. A wrong symbol name is a
// relocation against the wrong symbol — wrong bytes, not a gap — and it was
// resting on an ordering `.gl` does not promise.
//
// The binding is now per record: each `.gl` function record carries a framed
// `80 <LE32>` body-start offset (`codec::gl_offset_framed`), those offsets are
// gated 1:1 and in order against the `.ex` `4F 1F` split points, and a record's
// name is the mangled run immediately preceding its offset field. Under that rule
// this TU binds `??__Egs` and `?w_add` to the two bodies — correctly — and then
// refuses, because `?gs@@3US@@A` is an unclaimed data definition
// (`il_gl_data_symbol.cpp`) and the thunk body is out of class anyway.
//
// Four probes were built to try to break the positional rule directly — `extern`
// data, a static member, a namespace, a template — and all four happened to list
// definitions first. This one does not, and no probe would have found it if the
// name scan had not been broadened to see `??` names at all.

struct S {
    S();
    int x;
};

S gs;

int w_add(int a) { return a + 1; }
