// GRID-L cell l3_n2 — 2 formals, a literal inserted at slot 2, so
// 0 formal(s) move up one register. the literal LAST: every formal already in place. The control.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l3_n2(int, int, int);
int grid_l_forward_l3_n2(int formal_number_0, int formal_number_1) { return grid_l_callee_l3_n2(formal_number_0, formal_number_1, 0); }
