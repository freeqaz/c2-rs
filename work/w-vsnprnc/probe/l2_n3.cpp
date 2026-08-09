// GRID-L cell l2_n3 — 3 formals, a literal inserted at slot 1, so
// 2 formal(s) move up one register. TWO moves — the next step out, and the boundary this grid is for.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l2_n3(int, int, int, int);
int grid_l_forward_l2_n3(int formal_number_0, int formal_number_1, int formal_number_2) { return grid_l_callee_l2_n3(formal_number_0, 0, formal_number_1, formal_number_2); }
