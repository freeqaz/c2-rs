// **Negative** — narrow integer arithmetic, which must keep refusing, and the
// mixed-signedness convert.
//
// Every function here is one step outside `il_type_intlike.cpp`, and each one
// defeats a different plausible "just treat narrow types as int" rule. Captures:
//
//   short  a+b   add r11,r3,r4 ; extsh r3,r11
//   char   a+b   add r11,r3,r4 ; extsb r3,r11
//   ushort a+b   rlwinm r10,r3,0,16,31 ; rlwinm r11,r4,0,16,31 ;
//                add r11,r10,r11 ; rlwinm r3,r11,0,16,31
//   uchar  a+b   rlwinm r10,r3,0,24,31 ; rlwinm r11,r4,0,24,31 ;
//                add r11,r10,r11 ; rlwinm r3,r11,0,24,31
//   bool   !a    rlwinm r11,r3,0,24,31 ; cntlzw r10,r11 ; rlwinm r3,r10,27,31,31
//
// Three separate inconsistencies, any one of which sinks a single rule:
//
//   * signed narrow leaves its inputs alone (the ABI already sign-extended them)
//     and extends only the result — two instructions. Unsigned narrow masks BOTH
//     inputs anyway, despite the ABI having zero-extended them, and takes four.
//   * the RESULT type drives input extension, not the operand type. `a_short` and
//     `a_sh2i` are the same expression over the same operands and differ only in
//     what they return, yet one extends the output and the other extends both
//     inputs and neither the output.
//   * the operator matters too: `a_mul` extends both inputs AND the output, where
//     `a_short` extends no input at all.
//
// So extension placement is a function of (operator, operand type, result type)
// with at least three behaviours visible in five captures — not enough to
// implement, and wrong guesses here are silent wrong bytes rather than refusals.
//
// `a_long`/`a_ulong` are the control: they are 32-bit, emit a bare `add` exactly
// like `int`, and DO pass. They sit here so the boundary is visible in one file —
// if a future change makes them refuse, the widening regressed.
//
// `a_mix` is a different mechanism again: `int + unsigned` inserts a `2C` convert
// in the IL, so it is a cast case, not a width case (`docs/IL_CAST_CONVERT.md`).

long a_long(long a, long b) { return a + b; }
unsigned long a_ulong(unsigned long a, unsigned long b) { return a + b; }

short a_short(short a, short b) { return a + b; }
unsigned short a_ushort(unsigned short a, unsigned short b) { return a + b; }
char a_char(char a, char b) { return a + b; }
unsigned char a_uchar(unsigned char a, unsigned char b) { return a + b; }
bool a_bool(bool a) { return !a; }
int a_sh2i(short a, short b) { return a + b; }
short a_mul(short a, short b) { return a * b; }

int a_mix(int a, unsigned b) { return a + b; }
