// **Ported (#35 step 2, rung 1 — Class A many-calls).** Two statement-position
// calls with nothing live across them: the smallest framed multi-call body.
//
//   7d8802a6  mflr r12          the shipped Class A three-word prologue
//   9181fff8  stw  r12,-8(r1)
//   9421ffa0  stwu r1,-96(r1)
//   4bfffff5  bl ?g             REL24 @ 0x0c
//   4bfffff1  bl ?g             REL24 @ 0x10 — the SAME symbol, a second site
//   38210060  addi r1,r1,96
//   8181fff8  lwz  r12,-8(r1)
//   7d8803a6  mtlr r12
//   4e800020  blr
//
// It was a *negative* fixture for a long time, and the note it carried is still
// the reason the shape is delicate: an earlier neighborhood gate checked only
// around the first call and emitted a single `b g`, silently dropping the second
// one. The whole-body parse recognizes the sequence now rather than guessing.
//
// Two facts this one fixture pins that no single-call fixture could:
//   * the last call is a `bl`, not a `b` — c2's tail-call transform is off once
//     the function is framed (a *lone* statement call IS tail-called; see
//     `mvp_argtail.cpp`);
//   * two call sites, one external symbol. `coff::Function` carries a *list* of
//     REL24 sites and emits one undefined external per distinct callee.
extern void g();
void f() { g(); g(); }
