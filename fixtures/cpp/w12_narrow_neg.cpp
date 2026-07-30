// **Negative** (T3) — everything one byte away from the narrow-pointee load leaf
// in `w12_narrow_getters.cpp`. Every function here must keep refusing, and the TU
// as a whole must come out `NotImplemented`.
//
// The accepted class is: **one** load of a **naturally aligned** 1-, 2-, 4- or
// 8-byte *integer*, through at most one byte-offset add, with at most one
// conversion and that conversion either a cv-strip at the same width and
// signedness or an *exact* widening to `int`, and nothing after it but the return.
// Each function below breaks exactly one of those, and each is a captured case
// where the emitted code differs — none is a range limitation.
//
// ## The conversion is where the plausible wrong rules live
//
//     int nw_widen_short(short* p) { return *p; }
//         /O1        a8630000                    lha  r3,0(r3)
//         /Ox, /O2   a1630000 7d630734           lhz  r11 ; extsh r3,r11
//
// This is the one shape in the whole table whose *instruction count* depends on the
// optimization mode — one `lha` when c2 is minimizing size, a two-instruction
// zero-load-then-extend pair otherwise. `docs/IL_LOAD_TYPES.md` §3 records only the
// `/Ox` form and says "never lha"; that is true of every *unconverted* short load
// (`g_s_s` in the positive fixture is `lhz` at both modes) and false here. Both
// lowerings are measured, but this codegen path takes no mode parameter, so the
// shape is refused rather than emitted from a coin flip. It is the only refusal
// here that a mode-aware lowering would close as-is.
//
//     unsigned nw_uint_from_char(char* p)  { return *p; }  89630000 7d630774
//     unsigned nw_uint_from_uchar(unsigned char* p) { return *p; }  88630000
//     short    nw_sh_from_char(char* p)    { return *p; }  89630000 7d630774
//     int      nw_ll_to_int(long long* p)  { return (int)*p; }  e9630000 7d6307b4
//     long long nw_ll_from_int(int* p)     { return *p; }       81630000 7d6307b4
//
//     bool     nw_bool_from_uchar(unsigned char* p) { return *p; }
//         89630000 314bffff 7c6a5910   lbz r11 ; addic r10,r11,-1 ; subfe r3,r10,r11
//
// That one is the neighbour that would look identical under the cv-strip rule and
// is not. The parser accepts a conversion as free when the target's (width,
// signedness) equals the source's — and `unsigned char` and `bool` are the *same*
// accepted (tag, kind) pair `(82, 12)`, so it cannot tell them apart. If this
// arrived as a `2C` it would be admitted and emit one `lbz`, three instructions
// short of what c2 emits. It does not arrive as a `2C`: c2 does not *convert* to
// `bool`, it **normalizes** with a carry-bit `!= 0` idiom, and the IL carries a
// `33`-literal compare (`33 82 12 20 00 20`) where the accepted shape has its
// conversion — so the parse refuses on the `33`. Measured identical at `/Ox` and
// `/O1`. The accepted directions of the same aliasing (`unsigned char` from
// `bool`, `wchar_t` from `unsigned short`, …) are the `a_*` cases in the positive
// fixture, and they really are the bare load; this is the one that is not.
//
// The other five carry the same `2C … 00` token as an accepted widening and differ only
// in its *target* type. The first two emit exactly what the accepted `int` form
// emits — refusing them costs real cases — but the target family is admitted one
// captured (source × target) pair at a time, because the last three show what the
// same token does elsewhere: `char`→`short` still extends (so "the target is
// narrow, therefore free" is wrong), and both directions across the 4/8 boundary
// pay an `extsw` (`7d6307b4`), which no accepted shape emits at all. A blanket
// "a `2C` over a load is free" rule silently drops an instruction from three of
// these five.
//
// ## Alignment is carried by the tag, and it is not the width
//
//     #pragma pack(1);  struct P { char pad[3]; long long q; short h; };
//     long long nw_ds(P* s) { return s->q; }    39600003 7c63582a
//                                              li r11,3 ; ldx r3,r3,r11
//     short nw_packed_h(P* s) { return s->h; }  a063000b  lhz r3,11(r3)
//
// `ld` is DS-form: the low two bits of its 16-bit field belong to the form, so an
// offset of 3 is not representable and c2 does not try — it materializes the offset
// and uses the indexed `ldx` instead. In the IL, a packed member's TYPE tag drops
// to the *alignment* class while the kind keeps the width: `30 82 81 13` is an
// 8-byte signed load with an align-1 tag, against `30 88 81 13` for the aligned
// one. That is why the parser matches (tag, kind) **pairs** literally instead of
// deriving the width from the tag's low nibble — under that derivation this
// function's load reads as width 1 and emits `lbz` for a `long long`, which is a
// wrong-bytes emit rather than a refusal. `nw_packed_h` is the same tag family at
// width 2, where c2's `lhz r3,11(r3)` *is* what the port would emit: it is refused
// anyway, because the accepted set is "naturally aligned pairs" and admitting the
// align-1 family piecemeal is what would let the `ld` case in.
//
// ## A narrow value in an arithmetic position is a different rung
//
//     int nw_add1(char* p)          { return *p + 1; }
//         89630000 7d6b0774 386b0001   lbz r11 ; extsb r11,r11 ; addi r3,r11,1
//     int nw_widen_param(char a)    { return a; }        7c630774  extsb r3,r3
//     int nw_cmp(char a, char b)    { return a < b; }    7c6b0774 7c8a0774 …
//
// `nw_add1` extends **in place** (`extsb r11,r11`) where the accepted getter
// extends across registers (`extsb r3,r11`) — the same two opcodes, different
// register fields, so a lowering that reused the leaf's rule here would emit
// plausible wrong bytes. `nw_widen_param` is the ~8.8k widen-param rung (no load at
// all; its `B9` operand is a `char` value, not a pointer, so it cannot reach the
// leaf); `nw_cmp` extends *both* operands, while `IL_TYPE_TAGS.md` §3.2 has
// `short a + b` extending neither — extension placement is
// (operator × operand × result)-dependent, which is exactly why the accepted class
// stops at the getter.
//
// ## The remaining shapes
//
//     float  nw_f(float* p)  { return *p; }   c0230000  lfs f1,0(r3)
//     double nw_d(double* p) { return *p; }   c8230000  lfd f1,0(r3)
//     int    nw_store(char* p, char v) { *p = v; return 0; }
//                                            98830000 38600000  stb r4,0(r3) ; li r3,0
//     bool   nw_two(bool* p, bool* q) { return *p && *q; }   two loads and a branch
//
// FP pointees are the next rung (the encoders exist since W13a; the leaf needs the
// FP result register and the `_fltused` shell effect). A store is the other side of
// the same IL: the pointer is the destination and `32 <TYPE>` writes through it.

#pragma pack(push, 1)
struct P {
    char pad[3];
    long long q;
    short h;
};
#pragma pack(pop)

int nw_widen_short(short* p) { return *p; }

unsigned nw_uint_from_char(char* p) { return *p; }
unsigned nw_uint_from_uchar(unsigned char* p) { return *p; }
short nw_sh_from_char(char* p) { return *p; }
int nw_ll_to_int(long long* p) { return (int)*p; }
long long nw_ll_from_int(int* p) { return *p; }
bool nw_bool_from_uchar(unsigned char* p) { return *p; }

long long nw_ds(P* s) { return s->q; }
short nw_packed_h(P* s) { return s->h; }

int nw_add1(char* p) { return *p + 1; }
int nw_widen_param(char a) { return a; }
int nw_cmp(char a, char b) { return a < b; }

float nw_f(float* p) { return *p; }
double nw_d(double* p) { return *p; }
int nw_store(char* p, char v) {
    *p = v;
    return 0;
}
bool nw_two(bool* p, bool* q) { return *p && *q; }
