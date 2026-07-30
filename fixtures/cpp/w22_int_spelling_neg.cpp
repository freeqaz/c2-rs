// W22 — the negatives of the int-spelling widening. Every function here must
// stay OUT of class: the predicate is `is_int4_type`, which requires the tag to
// say 4-byte alignment AND the kind's high nibble to say 4-byte size, so the
// narrow types, `long long`, and a 4-byte int at a smaller alignment all refuse.
//
// The size check is not decoration: a TYPE's tag carries **alignment** and its
// kind carries **size**, equal for every naturally-aligned type — until
// `#pragma pack(4)` put an 8-byte `long long` behind a 4-byte tag and one `lwz`
// landed at the wrong offset (`docs/GAPS.md` §6, instance 3). This file keeps
// both halves of that pair in the corpus.

typedef short MyShort;
typedef char  MyChar;
typedef long long MyLL;
typedef unsigned char MyUChar;

#pragma pack(1)
struct P1 { char c; int i; };      // a 4-byte int at 1-byte alignment
#pragma pack(4)
struct P4 { char c; long long q; }; // an 8-byte value behind a 4-byte tag
#pragma pack()

int     packed_i (P1* p) { return p->i; }
long long packed_q(P4* p) { return p->q; }
MyShort id_sh(MyShort a) { return a; }
MyChar  id_ch(MyChar a) { return a; }
MyUChar id_uc(MyUChar a) { return a; }
MyLL    id_ll(MyLL a) { return a; }
MyShort sum_sh(MyShort a, MyShort b) { return a + b; }
