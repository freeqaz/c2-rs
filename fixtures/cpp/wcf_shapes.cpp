// wcf — the control-flow grammar, DECODE ONLY.
//
// EVERY function in this file must census OUT of class. The rung this fixture
// belongs to decodes control flow and lowers none of it, so a positive count
// here is a regression, not progress: it would mean a body needing basic blocks
// was admitted by a straight-line emitter.
//
// What the fixture is FOR is the other half — that each shape refuses at a
// *named* control-flow key rather than at an opaque hex byte, and that the
// statement-layer scanner reads each of them end to end. The two claims are
// graded together by `c2rs census`, which prints both axes.
//
// One construct per function, everything else held fixed (an `int` formal, an
// `int` return, `+`/`-` only), so the census key that moves is attributable to
// the construct that moved. Deliberately NO calls, no members, no pointers:
// those would each add their own blocker and the body would refuse for a reason
// that has nothing to do with control flow — which is exactly the
// mis-attribution this fixture exists to avoid.

// ---- the diamond ---------------------------------------------------------
// `38 <Lelse>` brFALSE, the then-clause, `29 <Lelse>`.
int cf_if(int a) { if (a) return 1; return 2; }

// …and with an else arm, which adds the join jump `3A <Ljoin>` and a second
// label. Same shape class (one conditional, forward only).
int cf_if_else(int a) { if (a) return 1; else return 2; }

// The relation feeds the branch directly — no `2C` bool->int convert, unlike a
// comparison LEAF, which converts because it returns the value.
int cf_if_rel(int a) { if (a > 0) return 1; return 2; }

// `!` is not an opcode in a condition: the front end emits the OPPOSITE branch
// sense (`39` instead of `38`). This is the witness for the polarity of the
// pair, and it is why the census may name them at all.
int cf_if_not(int a) { if (!a) return 1; return 2; }

// `&&` and `||` are lowered to branches by the front end too — `1A`/`1B`/`1C`
// never appear in a condition. Two conditionals each, so both land in the
// two-branch shape class rather than the one-branch class.
int cf_if_and(int a, int b) { if (a && b) return 1; return 2; }
int cf_if_or(int a, int b) { if (a || b) return 1; return 2; }

// ---- the back edge -------------------------------------------------------
// A loop is not "more branches": it is a branch whose target label is defined
// EARLIER in the byte stream. `3A` carries no direction, so nothing but the
// label position separates this from the diamonds above.
int cf_while(int a) { while (a) { a = a - 1; } return a; }

// `for` rotates the increment ABOVE the condition in the byte stream even
// though it runs after it — a decoder that assumes source order mis-attributes
// the two assignments.
int cf_for(int n) { int s = 0; for (int i = 0; i < n; i = i + 1) { s = s + i; } return s; }

// `do`/`while` puts its top label at body depth, BEFORE any `53` — the first
// byte after the body's own scope open is a label definition.
int cf_do_while(int a) { do { a = a - 1; } while (a); return a; }

// `break` and `continue` need no new opcode: both are `3A <label>`, the same
// token a `return` and a `goto` use.
int cf_break(int a) { while (a) { a = a - 1; if (a) break; } return a; }
int cf_continue(int a) { while (a) { a = a - 1; if (a) continue; } return a; }

// ---- the jump ------------------------------------------------------------
// A user label is an ordinary `29 <tok>` and `goto` is an ordinary `3A <tok>`.
int cf_goto(int a) { if (a) goto out; a = a + 1; out: return a; }

// ---- the switch ----------------------------------------------------------
// Three more opcodes (`3B` dispatch, `3C` table header, `3D` case entry) and a
// jump table, and the table is emitted AFTER the case bodies.
int cf_switch(int a) { switch (a) { case 1: return 10; case 2: return 20; default: return 30; } }

// ---- the conditional expression ------------------------------------------
// `43 42 <2 bytes>` — control flow wearing an expression's clothes. c2 lowers it
// with two exits and a `bclr`, not a select.
int cf_ternary(int a, int b) { return a ? b : 2; }
