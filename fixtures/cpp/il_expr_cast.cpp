// **Characterization** — the cast *statement*, and which casts are free.
//
// `docs/IL_CAST_CONVERT.md` pins `2C <TYPE target> <varint>` and its emission table.
// What this fixture adds is the two facts a *statement-level* cast turns on, both
// captured here:
//
// 1. **A cast through a local is byte-identical to a cast in the return
//    expression.** c2 allocates the local away, so
//
//      short s = (short)a; return s;      ->  7c630734  extsh r3,r3 ; blr
//      return (short)a;                   ->  7c630734  extsh r3,r3 ; blr
//
//    The IL differs — the statement form carries `2C 84 21 11 00 32 84 21 11 4B`
//    and then re-loads `s` with `2C 86 41 74 00`, while the expression form is two
//    adjacent `2C`s — but the obj does not. So the *pair* (source type, target type)
//    that matters is the end-to-end one, and the intermediate store is noise. That
//    is the same collapse `docs/IL_CAST_CONVERT.md` §2.2(b) records for a chain of
//    casts in one expression, reached a second way.
//
// 2. **A round trip through a same-width integer local is free.** Captured, both a
//    bare `blr`:
//
//      unsigned u = (unsigned)a; return (int)u;   ->  4e800020
//      long     l = (long)a;     return (int)l;   ->  4e800020
//
//    These are the cheapest members of the `expr-convert` bucket and the only ones
//    this document can claim without a typed operand stack: the source type is
//    visible on the *preceding* LOAD in the same statement, so a narrow rule
//    "`2C` between two int-like types, directly over an int-like LOAD" is decidable
//    with what `parse_expr` already reads. It is deliberately **not implemented**
//    here, because the same widening applied one token later — over an arithmetic
//    result, where the operand stack no longer carries a type — is exactly the
//    mis-emit `docs/IL_CAST_CONVERT.md` §4.2 lists, and the two are one code path.
//
// The narrowing siblings are the negatives, and their instructions come straight off
// the reference obj:
//
//   short          2c 84 21 11 00   ->  7c630734  extsh  r3,r3
//   char           2c 82 11 70 00   ->  7c630774  extsb  r3,r3
//   unsigned char  2c 82 12 20 00   ->  5463063e  clrlwi r3,r3,24
//   (int)(unsigned char)(short)a    ->  5463063e  clrlwi r3,r3,24   (ONE instruction
//                                       for three `2C` tokens — per-token lowering
//                                       would emit `extsh ; clrlwi ; nothing`)
//   float          2c 86 45 40 00   ->  extsw ; std ; lfd ; fcfid ; frsp
//   (int)(float)   2c 86 41 74 00   ->  fctiwz ; stfd ; lwz r3,-12(r1)
//
// The float pair also shows why the target type alone cannot drive the emission: the
// *same* token `2c 86 41 74 00` is nothing here, an `extsb` after a `char` load, and
// a three-instruction `fctiwz` sequence after a `float` load.

int k_short(int a) {
    short s = (short)a;
    return s;
}

int k_short_ret(int a) { return (short)a; }

int k_char(int a) {
    char c = (char)a;
    return c;
}

int k_uchar(int a) {
    unsigned char c = (unsigned char)a;
    return c;
}

int k_unsigned(int a) {
    unsigned u = (unsigned)a;
    return (int)u;
}

int k_long(int a) {
    long l = (long)a;
    return (int)l;
}

int k_chain(int a) { return (int)(unsigned char)(short)a; }

float k_i2f(int a) { return (float)a; }
int k_f2i(float a) { return (int)a; }
