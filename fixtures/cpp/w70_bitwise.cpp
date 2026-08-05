// **W70 — the bitwise and shift binary operators**, register-register, in the
// straight-line integer class. `lane w-build`.
//
// Six IL opcodes that had no production in the accepting parser at all:
//
//     09  <<     0A  >>     0B  &     0C  |     0D  ^
//
// (`0A` is two instructions — see below — which is why six ops come from five
// operators.) Every one is a bare one-byte token, exactly as `mcall`'s
// `BARE_BINARY_OPS` records from its own capture, and each lowers to a single
// X-form instruction whose **destination is the RA field** — *not* the RT field
// `add`/`mullw`/`subf` use. `encode_and`'s test
// `the_logical_destination_field_is_ra_and_not_rt` is that hazard written down:
// the wrong field order produces a valid `and` with the destination and the left
// operand exchanged, which disassembles cleanly and computes the wrong thing.
//
// ## The one operator the IL byte does not determine
//
// `>>` is `sraw` over a signed left operand and `srw` over an unsigned one, from
// the **identical** IL byte `0A`. The distinction is one nibble of the operand
// TYPE (`86 41` vs `86 42`) — and `ValueClass::Int4`, which is what the
// expression parser tracks, deliberately collapses the two because every other
// modeled operator emits the same instruction over both. So `parse_expr` reads
// the signedness separately for this operator only, and refuses outright when
// the expression carries both (`w70_bitwise_neg.cpp`'s `n_shr_mixed`).
//
// Probed both ways round, and **only the left operand decides**:
//
//     int      f(int a, unsigned b) { return a >> b; }   7c632630  sraw
//     unsigned f(unsigned a, int b) { return a >> b; }   7c632430  srw
//
// `<<` is one instruction for both (`slw`, `7c632030`, probed over
// `int<<int`, `unsigned<<unsigned` and `int<<unsigned`), which is why there is
// one `Shl` and two `Shr`s in `IlOp`.
//
// ## Depth — and why the four- and five-leaf chains are HERE and not implied
//
// `il_accum4.cpp` records that c2 decides accumulator-versus-descending **once
// for the whole chain** and that the two candidate rules coincide at one
// intermediate. Every three-leaf chain has one intermediate. So the four-leaf
// rows below are not more of the same shape, they are the first rows that can
// separate the rules — the same lesson that file was written for, one operator
// family over. Measured at `/Ox` (`work/w-build/probe/bits3-Ox.cod`):
//
//     a & b & c & d       and r11,r3,r4 ; and r10,r11,r5 ; and r3,r10,r6
//     a & b & c & d & e   and r11 ; and r10 ; and r9 ; and r3
//
// — the descending allocation, r11 then r10 then r9, which is exactly what
// `select_text` already does when `chain_has_add` is false. A pure bitwise chain
// contains no addition, so it takes that path unchanged. At `/O1` (the
// workload's own mode) there is no descending case at all and every intermediate
// is r11; both are graded, because `scripts/lanes.txt` runs both.
//
// **A chain that MIXES these with `+`/`-`/`*` is refused**, and that refusal is
// this rung's most important measurement rather than its conservatism —
// `straight_line_out_of_class_ctx`'s `expr-out-of-class-bitwise-mixed-arith`
// carries the two probed cells that refute `chain_has_add` outright. The
// neighbour that would look the same under the wrong rule is
// `w70_bitwise_neg.cpp`'s `n_add_then_and`.
//
// ## What is NOT here
//
// Every immediate form. `a & 1` is `clrlwi`, `a & 5` is `andi.` (record-form —
// it writes CR0, there is no plain `andi`), `a & 0x12345` is `lis`+`ori`+`and`
// through **r12**, `a | 0x12345` is `oris`+`ori`, and `256 >> a` materializes
// the literal into r11 with `li` first. Three instruction families and two
// scratch registers across one axis, selected by a predicate over the
// immediate's *value*. `w42_shift_mask.cpp` and `w43_cmp_shift_or.cpp` are the
// two immediate cells this tree does model, and both are folds recognized ahead
// of the general expression path rather than emitted from it.

// ---- two leaves, one instruction: every operator, both signednesses ----------
int      s_and (int a, int b)                   { return a & b; }
int      s_or  (int a, int b)                   { return a | b; }
int      s_xor (int a, int b)                   { return a ^ b; }
int      s_shl (int a, int b)                   { return a << b; }
int      s_shr (int a, int b)                   { return a >> b; }   // sraw
unsigned u_and (unsigned a, unsigned b)         { return a & b; }
unsigned u_or  (unsigned a, unsigned b)         { return a | b; }
unsigned u_xor (unsigned a, unsigned b)         { return a ^ b; }
unsigned u_shl (unsigned a, unsigned b)         { return a << b; }
unsigned u_shr (unsigned a, unsigned b)         { return a >> b; }   // srw

// The operator in a later argument slot, so the register fields are not all r3/r4.
int      s_and_hi(int a, int b, int c, int d)   { return c & d; }
int      s_shr_hi(int a, int b, int c, int d)   { return c >> d; }

// ---- three leaves: ONE intermediate, where every allocation rule coincides ---
int      s_and3(int a, int b, int c)            { return a & b & c; }
int      s_or3 (int a, int b, int c)            { return a | b | c; }
int      s_xor3(int a, int b, int c)            { return a ^ b ^ c; }
int      s_shl3(int a, int b, int c)            { return a << b << c; }
int      s_shr3(int a, int b, int c)            { return a >> b >> c; }
unsigned u_shr3(unsigned a, unsigned b, unsigned c) { return a >> b >> c; }
// mixed operators inside the family — still one family, so still pure
int      m_and_or (int a, int b, int c)         { return (a & b) | c; }
int      m_or_and (int a, int b, int c)         { return (a | b) & c; }
int      m_shr_and(int a, int b, int c)         { return (a >> b) & c; }
int      m_shl_xor(int a, int b, int c)         { return (a << b) ^ c; }

// ---- four leaves: TWO intermediates, the first depth that separates the rules
int      s_and4(int a, int b, int c, int d)     { return a & b & c & d; }
int      s_or4 (int a, int b, int c, int d)     { return a | b | c | d; }
int      s_xor4(int a, int b, int c, int d)     { return a ^ b ^ c ^ d; }
int      s_shl4(int a, int b, int c, int d)     { return a << b << c << d; }
int      s_shr4(int a, int b, int c, int d)     { return a >> b >> c >> d; }
unsigned u_shr4(unsigned a, unsigned b, unsigned c, unsigned d) { return a >> b >> c >> d; }
int      m_and_or4(int a, int b, int c, int d)  { return a & b | c | d; }
int      m_shr_or4(int a, int b, int c, int d)  { return (a >> b) | c | d; }

// ---- five leaves: r11, r10, r9 — the whole characterized descending range ----
int      s_and5(int a, int b, int c, int d, int e) { return a & b & c & d & e; }
int      s_or5 (int a, int b, int c, int d, int e) { return a | b | c | d | e; }
int      s_shr5(int a, int b, int c, int d, int e) { return a >> b >> c >> d >> e; }
