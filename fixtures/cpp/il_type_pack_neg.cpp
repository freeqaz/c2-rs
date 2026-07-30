// **Negative** — a TYPE's tag carries the value's *alignment*, and the kind's high
// nibble carries its *size*. Those agree for every naturally-aligned type, so a
// predicate reading only the tag looked right for months. `#pragma pack` separates
// them, and separated them into a wrong-bytes emit.
//
// `#pragma pack(4)` on `struct P { int a; long long q; }` puts `q` at offset 4 with
// 4-byte alignment, so its load TYPE is `86 81 …`: tag `86` says align 4, kind `81`
// says size 8. `is_int4_type` checked the tag, admitted it as a 4-byte integer, and
// the indirect-load leaf lowered `(int)s->q` to one `lwz` at the member's offset.
// The reference loads the *low word* of a big-endian 64-bit value, which is four
// bytes further along. `Port=Mismatch @ offset 8` — the very first instruction.
//
// Only the size check was added. A 4-byte `int` at a *smaller* alignment
// (`pack(1)`/`pack(2)`) still refuses on the tag, exactly as before, because whether
// c2 even emits a plain `lwz` for an unaligned load is unprobed and admitting it on
// the strength of this decode would be widening on an assumption rather than a
// measurement.
//
// Every function here must stay `NotImplemented`, and each is one field away from an
// admitted shape:
//
//   pk_ll     pack(4) `long long`   tag align 4, kind size 8   — WAS the mis-emit
//   nat_ll    natural `long long`   tag align 8, kind size 8   — refused on the tag,
//                                                                which is why the
//                                                                bug needed packing
//                                                                to surface at all
//   pk1_int   pack(1) `int`         tag align 1, kind size 4   — refused on the tag
//   pk2_int   pack(2) `int`         tag align 2, kind size 4
//   pk2_short pack(2) `short`       narrow, a separate rung
//   pk_dbl    pack(4) `double`      the same trap in the FP family
//
// `nat_ll` is the discriminating neighbour: it is the same C construct as `pk_ll` and
// was ALWAYS refused, because at natural alignment the tag reads 8 and the old
// tag-only check caught it. So the corpus contained the safe half of the pair and
// none of the dangerous half — the failure was invisible from inside the shapes the
// fixtures covered.
//
// The aligned cases that MUST stay admitted are in `il_expr_member.cpp` and
// `il_this_straightline.cpp`; if this tightening had over-reached, those would have
// stopped matching.

#pragma pack(push, 4)
struct P {
    int a;
    long long q;
    double d;
};
#pragma pack(pop)

#pragma pack(push, 1)
struct Q1 {
    char c;
    int i;
};
#pragma pack(pop)

#pragma pack(push, 2)
struct Q2 {
    char c;
    int i;
    short s;
};
#pragma pack(pop)

struct N {
    int a;
    long long q;
};

int pk_ll(P* s) { return (int)s->q; }
int pk_dbl(P* s) { return (int)s->d; }
int nat_ll(N* s) { return (int)s->q; }
int pk1_int(Q1* s) { return s->i; }
int pk2_int(Q2* s) { return s->i; }
int pk2_short(Q2* s) { return (int)s->s; }
