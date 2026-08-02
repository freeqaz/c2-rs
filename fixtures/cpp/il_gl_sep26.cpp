// **The `26` name separator, at the gate** (W-ADOPT, board #151).
//
// `.gl` introduces a record's name with `00` **or** `26`. `NAME_SEPARATORS` has
// said so since `gl_symbol_index` was written, and §9.20 taught the
// instrument-side scanner (`gl_symbol_runs_all_separators`) both. The *gate* —
// `gl_defined_names`, and therefore `Bindings::per_record` and
// `IlBundle::functions` — kept the NUL-only reader, deliberately: widening what
// a gate accepts moves the emitted class and is a differential decision, not a
// reader repair.
//
// This is the smallest source that produces the shape. Three `.ex` bodies
// (`??1R` the destructor, `??_GR` the deleting destructor c2 synthesizes for a
// virtual one, `?w_use`), three framed `.gl` records, and the middle record's
// name is `26`-introduced:
//
//   125  00 '??1R@@UAA@XZ' 00           name ends 138
//   139  82 07 05 00 20 20 03 00 00 03 00   TYPE
//   150  80 05 10 00 00 00 00           framing
//   157  80 54 0a 00 00                 body @ 2644      (distance 19)
//   162  00 15 c8 18 01 01 ec 09 01 0e ef 09   record tail
//   174  26                             <- the separator the gate could not read
//   175  '??_GR@@UAAPAXI@Z' 00          name ends 191
//   192  86 03 05 04 20 20 01 00 00     TYPE
//   201  80 0f 10 00 00 00 00           framing
//   208  80 d1 0a 00 00                 body @ 2769      (distance 17)
//
// A `26`-introduced name is not mis-framed by the NUL-only reader. It is
// **never seen** — the reader opens a run only after a `00`. So the second
// record's "nearest preceding run" is the *first* record's name, ending at 138,
// **70 bytes** from the offset field at 208. `MAX_NAME_TO_OFFSET` is 32, so the
// record is unnameable and the whole TU refuses.
//
// That refusal is the only reason this was not a wrong-bytes emit, and it came
// from the distance bound rather than from anything knowing a name was missing.
// On a TU where some unrelated run happened to land inside 32 bytes of the
// offset field, the same reader binds a body to another symbol's name — the
// `il_gl_record_order` / `il_extern_c_name` failure again, reached by a third
// route. Under the widened scanner every record binds to its own name at
// distance 17–19, exactly like the `00`-introduced ones.
//
// Deliberately **not** in the port's class: a deleting destructor and a vftable
// are far outside it, and the verdict this fixture must produce is
// `NotImplemented`. That is the claim under test. Widening a scan adds names to
// **both** sides of `IlBundle::functions`' accounting rule — more records it can
// name, and more unclaimed runs it must account for — so "the gate sees more"
// does not point in a single direction, and seeing three more names must not
// turn a refusal into an emit.
//
// The executable form of the control is
// `gl::tests::the_gate_binds_a_26_introduced_record_to_its_own_name`, which
// transcribes the two records above and asserts both halves: the NUL-only
// reader refuses this shape, the widened one binds `??_GR@@UAAPAXI@Z` to 2769.

struct R {
    virtual ~R() {}
};

int w_use(R* p) { return 1; }
