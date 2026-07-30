// **Negative** — the class-layout half of the `0x40` family (ids 2113…2119),
// which is **86 % of the whole intrinsic footprint** on the real dc3 workload
// (329,205 of 381,488 blocked functions). Every function here must keep refusing.
//
// `fixtures/cpp/il_intrinsic_call.cpp` already shows that the emission depends on
// the *offset literal* (`00` → nothing, `04` → a null-guarded `addi`). This
// fixture separates the three things that one does not.
//
// **(1) 2113 vs 2114 is the NULL GUARD, not the offset.** `l_up2` and
// `l_this2` produce the *same class pair descriptor* and the *same* offset
// literal `08`, and differ only in the selector id — and that is exactly the
// difference between five instructions and one:
//
//   l_up2   (A2 *)m       33 86 41 74 80 42 08 00 00  40 86 43 b1 20   id 2114
//                         66 02 92 20 93 20 55 86 41 74
//                         33 86 41 74 >08< 55 86 41 74
//                         b9 <m> 86 43 b0 20 55 86 43 b0 20  4c
//     -> 2b030000  cmplwi r3,0
//        38630008  addi   r3,r3,8
//        4c9a0020  bclr   4,26        (return if r3 was non-null)
//        38600000  li     r3,0
//        4e800020  blr
//
//   l_this2 m->mb()       33 86 41 74 80 41 08 00 00  40 a6 43 96 20   id 2113
//                         66 02 92 20 93 20 55 86 41 74
//                         33 86 41 74 >08< 55 86 41 74
//                         b9 <m> 86 43 b0 20 55 86 43 b0 20  4c  99 … bd …
//     -> 38630008  addi r3,r3,8
//        4bffffc4  b <A2::mb>
//
// So 2113 is the adjustment for a member call's `this` (which the language
// guarantees non-null) and 2114 is a pointer *conversion* (where null must stay
// null). Reading the id as "a base adjustment" and lowering it from the offset
// would silently drop a control-flow split on 137,511 dc3 functions.
//
// **(2) 2115's offset is NOT pre-negated.** `l_down2` is the reverse conversion
// and its offset literal is `08`, positive, byte-identical to `l_up2`'s; the
// **id** is what makes it `addi r3,r3,-8`. docs/IL_CAST_CONVERT.md §1.4 recorded
// this as "mirror of 2114 with a negated offset", which the bytes refute.
//
// **(3) `0x66`'s second byte is a COUNT, not the constant `02`.** The
// class-pair descriptor is `66 <n> <n × token>`: two tokens for a non-virtual
// base pair, **three** where a virtual base is involved (`l_upv`, `l_fldv`):
//
//   l_up2   66 >02< 92 20 93 20                (A1/A2 pair)
//   l_upv   66 >03< b2 20 b4 20 b5 20          (DD / D1 / V1 triple)
//
// docs/IL_CALL_GRAMMAR.md §7 ranked `0x66` as the #1 unidentified opcode and
// recorded "the `02` is fixed in every observation but its meaning is unknown".
// It is the descriptor's arity.
//
// The offset-literal count is fixed per id, which is the cheap consistency check
// a decoder gets for free (captured here): 2113/2114/2115 → 1, 2117 → 2,
// 2116 → 4, 2118 → 5, 2119 → 0 (its two arguments are `26 <sym>` RTTI pushes
// instead). `l_dyn` pins **2119 = `dynamic_cast`**, which
// docs/IL_CAST_CONVERT.md §1.4 left UNKNOWN.
//
// Emissions, all read off this file's reference obj:
//
//   l_up1  (offset 0)  -> 4e800020  blr                      (nothing at all)
//   l_up2              -> the 5-instruction guarded form above
//   l_down2            -> cmplwi ; addi r3,r3,-8 ; bclr ; li r3,0 ; blr
//   l_this2            -> addi r3,r3,8 ; b <A2::mb>
//   l_fld2   (2117)    -> 8063000c  lwz r3,12(r3)     (both offsets folded in)
//   l_upv    (2116)    -> cmplwi ; bc ; blr ; lwz r11,0(r3) ; lwz r11,4(r11) ; add r3,r11,r3 ; blr
//   l_fldv   (2118)    -> lwz r11,0(r3) ; lwz r11,4(r11) ; add r10,r11,r3 ; lwz r3,4(r10)
//   l_dyn    (2119)    -> b <__RTDynamicCast>, with the two `26 <sym>` pushes
//                         resolving to the RTTI descriptors `??_R0?AUA1@@@8` and
//                         `??_R0?AUA2@@@8` (named relocations in this obj)
//
// Note `l_upv`: for a *virtual* base c1xx writes the null test into the IL
// itself (`b9 p 33 00 1f … 43 42 00 00`, a compare plus the `43` ternary select),
// whereas for 2114 it does not and c2 synthesizes the guard. The same source-level
// null check therefore lives on different sides of the c1xx/c2 boundary depending
// on the base's virtualness, which is one more reason the family cannot be
// lowered from the id. (`l_upv` consequently blocks the parser one token *before*
// its selector, on the `DD *` operand type of that compare — its census bucket is
// `expr-load-type-8643B2`, not an intrinsic bucket. The 2116 selector is still
// there in the bytes; it is simply not the first thing that refuses.)

struct A1 {
    int a;
    void ma();
    virtual void va();
};
struct A2 {
    int b;
    void mb();
    virtual void vb();
};
struct M : A1, A2 {
    int d;
};
struct V1 {
    int v;
    virtual void vv();
};
struct D1 : virtual V1 {
    int w;
};
struct D2 : virtual V1 {
    int x;
};
struct DD : D1, D2 {
    int y;
};

A1 *l_up1(M *m) { return m; }             // 2114, offset 0
A2 *l_up2(M *m) { return m; }             // 2114, offset 8
M *l_down2(A2 *p) { return (M *)p; }      // 2115, offset 8 (positive!)
void l_this2(M *m) { m->mb(); }           // 2113, offset 8
int l_fld2(M *m) { return m->b; }         // 2117, offsets 4, 8
V1 *l_upv(DD *p) { return p; }            // 2116, 66 03, offsets 4,4,0,0
int l_fldv(DD *p) { return p->v; }        // 2118, 66 03, offsets 4,4,4,0,0
A2 *l_dyn(A1 *p) { return dynamic_cast<A2 *>(p); } // 2119, two 26 <sym> pushes
