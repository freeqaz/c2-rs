// **Characterization** — `0x43` is an *escape* opcode with a sub-opcode byte, not
// "ternary".
//
// `c2_il::func::Block::feature` maps operand-stream byte `0x43` to the name
// `ternary`, from `docs/IL_CALL_GRAMMAR.md` §7's single observation that it always
// appeared as `43 42 00 00`. Two captures in one TU refute the generalization:
//
//   t_cond   a ? b : 2      b9 <a> 86 41 74  b9 <b> 86 41 74  33 86 41 74 02
//                           43 42 00 00                       41 86 41 74
//
//   t_bits   b->g           b9 <b> 86 43 8e 20  33 86 41 74 00  27 86 43 f5 08
//                           33 86 41 74 18  33 86 41 74 05
//                           43 37                             30 86 42 75
//                           2c 86 41 74 00                    41 86 41 74
//
// `43 42` carries two trailing bytes; `43 37` carries **none** — the byte after it
// is `30`, the indirect load. So the payload width is a function of the sub-opcode,
// and a decoder that treats `0x43` as a fixed four-byte token desynchronizes on
// every bitfield read in the corpus.
//
// Sub-opcode `0x42` is the conditional expression and `0x37` builds a bitfield
// designator from (bit offset, width) literals — 24 and 5 for the second 5-bit
// field of `struct B { unsigned f:3; unsigned g:5; }`, i.e. the *shift* 32−3−5 and
// the width, not the offset within the byte.
//
//   UNKNOWN: the two trailing bytes of `43 42`. `00 00` in both `t_cond` and
//   `t_cond_rel`, whose conditions differ (a bare value vs a relational). **A
//   fixture that would separate them:** a conditional whose arms are lvalues, or one
//   nested inside another — untested.
//
//   UNKNOWN: the rest of the sub-opcode space. Only `0x42` and `0x37` are witnessed
//   here and neither is derivable from the other, so the port must reject the whole
//   `0x43` family and the census should report `expr-op-0x43-NN` rather than one
//   bucket named after one sub-opcode.
//
// The conditional's codegen is a reminder that this is not a select:
//
//   a ? b : 2      cmpwi cr6,r3,0 ; mr r3,r4 ; bclr 4,26 ; li r3,2 ; blr
//   a > 1 ? 5 : 6  cmpwi cr6,r3,1 ; li r3,5 ; bclr 1,25 ; li r3,6 ; blr
//
// — a *conditional return*, two exits, the compare fused into the branch condition
// register. Nothing in the port has a second exit, and the bit pattern of the
// `bclr` condition changes with the relation, so this is a control-flow feature
// wearing an expression's clothes. All of these must keep refusing.

struct B { unsigned f : 3; unsigned g : 5; };

int t_cond(int a, int b) { return a ? b : 2; }
int t_cond_rel(int a) { return a > 1 ? 5 : 6; }
int t_bits(B* b) { return b->g; }
int t_bits_f(B* b) { return b->f; }
