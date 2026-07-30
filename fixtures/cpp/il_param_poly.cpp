// **Positive** — a class with a vtable, passed by value, occupies exactly ONE
// argument register, because MSVC passes it by hidden reference.
//
// This file was `il_param_poly_neg.cpp` and every function in it was
// `NotImplemented`. It was not measuring a construct the port cannot lower; it was
// pinning a hole in the `.sy` reader, and its own header said so: "decode the
// `C6 81` form and it goes away". It has now been decoded, so the fixture asserts
// the opposite thing and is worth more for it — the parameter's width is a *fact*
// the reader now reads, and c2 grades whether it read it right.
//
// Two corrections were needed together, and neither is visible without the other
// (see `crates/c2-il/src/func/sy.rs`):
//
//   1. `C6 81 03` is a **wide type prefix** — the tag's bit 6 inserts one byte
//      before the kind. Kind `03` is a data POINTER, not an aggregate: that is the
//      hidden reference, and it is why the width is 4.
//   2. the size after the `04` lead is a **varint** followed by a separate unnamed
//      byte, not a little-endian `u16`. This record is where that byte is non-zero
//      (`08`), so the `u16` reading turned a 4-byte pointer into a 2052-byte object
//      and reported `param-multi-reg` — a decode error dressed up as a real
//      construct, which is worse than a refusal because it lands in the bucket that
//      ranks missing FEATURES.
//
// `plain` stays, and its role is unchanged in shape but inverted in sign: it used to
// show that one unreadable record refuses its innocent neighbours, and it now shows
// that they are no longer refused. That collateral cost was the whole reason this
// gap ranked first — measured over 878 translation units,
// `param-width-undetermined` was the single largest census blocker at 567,549
// functions against 1 for `param-multi-reg`.
//
// The multi-register ladder that IS a genuine construct refusal lives in
// `fixtures/cpp/il_param_aggr_neg.cpp` and must stay there: the two keys measure a
// reader gap and a missing feature and are still deliberately not summed.
//
// Every function here must be byte-exact.
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
