// **W70 negatives — the neighbours of `w70_bitwise.cpp` that must REFUSE.**
// `lane w-build`.
//
// `fixtures/README.md` asks for "the neighbour that would look the same under a
// plausible wrong rule". Two of the rows below are not merely plausible: they
// are cells this lane's own change emitted **wrong bytes** for before the guard
// that refuses them was written, and one of them refutes a rule the tree has
// held since `il_accum4.cpp`.
//
// Every function here must return `NotImplemented`, and the whole TU is graded
// for that. A wrong emit on any of them is the failure this file exists to
// catch.

// ---- 1. THE MIXED CHAIN, and the rule it refutes ----------------------------
//
// `select_text` allocates chain intermediates by `chain_has_add`: at `/Ox`, a
// chain containing any addition puts every intermediate in r11, and otherwise
// they descend r11, r10, r9. Enumerated over 11,664 four-leaf ARITHMETIC chains
// and right about all of them. Measured at `/Ox` with a bitwise operator in the
// chain (`work/w-build/probe/bits3-Ox.cod`):
//
//     (a & b) + c + d     and r11,r3,r4 ; add r11,r11,r5 ; add r3,r11,r6
//     (a + b) & c & d     add r11,r3,r4 ; and r10,r11,r5 ; and r3,r10,r6
//
// **Both contain an addition. The second descends anyway.** So `chain_has_add`
// is not the rule; the ten probed cells fit "an intermediate goes to r11 when
// its CONSUMER is an `add`" — which also reproduces every arithmetic cell
// `il_accum4.cpp` records, but is a hypothesis over ten witnesses against a rule
// enumerated over eleven thousand. Rather than swap them, the mixed chain
// refuses (`expr-out-of-class-bitwise-mixed-arith`) and this file holds the
// witness.
//
// `n_add_then_and` is the cell that mis-emitted. The other three are the
// neighbours that make the guard's boundary legible: they are all measured
// CORRECT under the shipped rule and are refused anyway, because the guard is
// stated over the mix and not over the direction of it.
int n_add_then_and(int a, int b, int c, int d) { return (a + b) & c & d; }
int n_and_then_add(int a, int b, int c, int d) { return (a & b) + c + d; }
int n_sub_then_and(int a, int b, int c, int d) { return (a - b) & c & d; }
int n_shr_then_add(int a, int b, int c, int d) { return (a >> b) + c + d; }

// ---- 2. MIXED SIGNEDNESS on `>>` ---------------------------------------------
//
// `sraw` and `srw` come from the same IL byte `0A`; only the operand TYPE
// separates them, and only the LEFT operand decides (probed both ways). The
// parser tracks a per-expression signedness rather than a per-operand one, so it
// refuses when both are live (`expr-shr-mixed-sign`) instead of guessing which
// operand a flag came from. c2 emits `sraw` for the first and `srw` for the
// second; both are refused.
int      n_shr_mixed_su(int a, unsigned b)      { return a >> b; }
unsigned n_shr_mixed_us(unsigned a, int b)      { return a >> b; }

// The same trap reached through a CONVERSION rather than through a formal, and
// this is the one that makes the guard necessary rather than tidy: an
// `int`->`unsigned` `2C` is a conversion `parse_expr` **already accepts as a
// no-op**, because `is_int4_type` admits both spellings and no instruction is
// emitted for the cast. It nonetheless moves `>>` from `sraw` to `srw`. Without
// the `2C` arm feeding the signedness flags, this body would have parsed with
// every operand looking signed and emitted `sraw`.
unsigned n_shr_cast(int a, int b) { return (unsigned)a >> b; }

// ---- 3. IMMEDIATES — three instruction families and two scratch registers ----
//
// The whole immediate axis refuses in `combine`. Rows chosen one per measured
// family, so the file records the shape of the axis and not just its existence:
//
//     a & 1        clrlwi r3,r3,31          contiguous mask -> rlwinm
//     a & 5        andi.  r3,r3,5           not contiguous, fits 16 bits;
//                                           RECORD-FORM, it writes CR0
//     a & 0x12345  lis r12,1 ; ori r12,r12,0x2345 ; and r3,r3,r12
//                                           neither: materialized, in **r12**
//     a | 0x12345  oris r3,r3,1 ; ori r3,r3,0x2345      two instructions
//     256 >> a     li r11,256 ; sraw r3,r11,r3          literal on the LEFT
int n_and_mask   (int a) { return a & 1; }
int n_and_andi   (int a) { return a & 5; }
int n_and_wide   (int a) { return a & 0x12345; }
int n_or_wide    (int a) { return a | 0x12345; }
int n_xor_wide   (int a) { return a ^ 0x12345; }
int n_shr_imm    (int a) { return a >> 1; }
int n_shl_imm    (int a) { return a << 1; }
int n_lit_shr_reg(int a) { return 256 >> a; }

// ---- 4. THE DEPTH-2 TREE ------------------------------------------------------
//
// `is_depth2_tree` is NOT widened to these operators, so a bitwise tree reaches
// operand-stack depth 3 in `chain_form` and refuses there. The measurement
// exists and points the other way — the `+`-root register swap depends on the
// ROOT alone and generalizes, `(a&b)|(c&d)` is `and r11 ; and r10 ; or r3,r11,r10`
// (no swap) while `(a&b)+(c&d)` is `and r10 ; and r11 ; add r3,r10,r11` (swap) —
// but that is 4 cells of a 216-cell root x op1 x op2 grid, against the accepted
// arithmetic three's 27 cells which were gridded whole. Both of the shape's known
// rewrites (N1 product flattening, N2 additive canonicalization) were found BY
// gridding it, so four witnesses do not license a sixfold widening.
int n_tree_and_or_and(int a, int b, int c, int d) { return (a & b) | (c & d); }
int n_tree_or_and_or (int a, int b, int c, int d) { return (a | b) & (c | d); }
int n_tree_and_add   (int a, int b, int c, int d) { return (a & b) + (c & d); }

// ---- 5. OPERAND CLASSES — and a CORRECTION to this file's own claim ----------
//
// `parse_expr` carries `expr-ptr-bitwise` and `expr-int1u-bitwise`, each its own
// census key rather than a widening of the existing `expr-ptr-arith` /
// `expr-int1u-arith` buckets: the fact is different (`+` over a pointer is
// *scaled*, `&` over one has no capture at all) and merging would move functions
// into buckets four rungs have compared across trees.
//
// **Neither of the two rows below reaches either guard, and this note is the
// correction rather than the removal.** Measured — `c2rs census` on this file
// reports both as **`expr-convert-target-8641`**: C++ has no `&` over pointers
// at all, so writing one requires a cast, and the cast refuses first; and `bool
// & bool` promotes both operands to `int`, so the IL spells the promotion as a
// `2C` that refuses first too. The two guards are therefore **unwitnessed** —
// they are fail-closed defence at a position the source language may not be able
// to reach, and they are shipped as that rather than as measured rows. What
// these two cells DO assert is that the refusal happens at all, and at a key
// that names a real construct.
int  n_ptr_and (int *p, int b) { return (int)((long)p & b); }
bool n_bool_and(bool a, bool b) { return a & b; }

// ---- 6. NON-ASCENDING LEAVES, and what the refusal costs ----------------------
//
// `canonicalize_chain` rewrites a non-ascending ADDITIVE chain into c2's
// canonical register order; it returns `None` for these operators, so
// `leaves_ascending` refuses instead. That is conservative rather than correct-
// by-necessity, and the number is measured: c2 emits **byte-identical** code for
// `c & b & a` and `a & b & c` (`and r11,r3,r4 ; and r3,r11,r5` for both), so it
// canonicalizes a bitwise chain exactly as it canonicalizes an additive one.
// Widening `canonicalize_chain` is a rung of its own; this is the row that
// prices it.
int n_desc_and(int a, int b, int c) { return c & b & a; }
int n_desc_or (int a, int b, int c) { return c | b | a; }

// ---- 7. A REPEATED LEAF -------------------------------------------------------
//
// `has_repeated_leaf` refuses any chain using a formal twice, because a repeated
// leaf licenses c2's algebraic rewriter (`a + a` is `slwi`, not `add r3,r3,r3`).
// The rewrite set is uncharacterized for these operators too — `a & a` is `a` —
// so the existing gate covers them and this row asserts that it does.
int n_repeat_and(int a, int b) { return a & b & a; }
