// **Negative** — the *other* reason a formal's register cannot be established: a
// width this reader cannot read at all, as opposed to one it reads and refuses.
//
// A **polymorphic** class parameter's `.sy` record opens `C6 81 03 …` — a
// three-byte type prefix built on the `0x40` tag bit that `readers.rs` records as
// occurring and undetermined. One witness pair (`V` and a class derived from it) is
// not enough to decode a new prefix form, so `read_record` refuses it, and because
// `.sy` binds a translation unit 1:1 or not at all, the whole file's formal widths
// go *undetermined*. Every function here then refuses — including `plain`, whose
// own parameters are perfectly ordinary.
//
// That collateral refusal is the honest behaviour and the reason this is a separate
// fixture from `il_param_aggr_neg.cpp`: mixing the two would relabel that file's
// whole multi-register ladder as `param-width-undetermined` and it would stop
// measuring the 8-byte boundary it exists to measure.
//
// The census reports the two reasons under **different** keys on purpose.
// `param-width-undetermined` is a gap in this reader — decode the `C6 81` form and
// it goes away. `param-multi-reg` is a construct the port genuinely does not lower.
// Summing them into one bucket would rank a reader bug and a missing feature as one
// number, which is the measurement failure `docs/GAPS.md` §6 records in its own
// right.
//
// Every function here must be `NotImplemented`, and `plain` is the one that says
// why it matters: the cost of an unreadable record is paid by its neighbours.

struct V {
    virtual void f();
    int a;
};

struct Der : V {
    int b;
};

struct H { int mi; };

int poly(V v, H* h) { return h->mi; }
int poly_der(Der d, H* h) { return h->mi; }
int plain(int a, H* h) { return h->mi; }
