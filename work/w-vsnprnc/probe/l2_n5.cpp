// GRID-L cell l2_n5 — 5 formals, a literal inserted at slot 3, so
// 2 formal(s) move up one register. TWO moves — the next step out, and the boundary this grid is for.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l2_n5(int, int, int, int, int, int);
int grid_l_forward_l2_n5(int formal_number_0, int formal_number_1, int formal_number_2, int formal_number_3, int formal_number_4) { return grid_l_callee_l2_n5(formal_number_0, formal_number_1, formal_number_2, 0, formal_number_3, formal_number_4); }
