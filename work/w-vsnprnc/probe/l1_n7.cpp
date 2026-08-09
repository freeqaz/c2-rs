// GRID-L cell l1_n7 — 7 formals, a literal inserted at slot 6, so
// 1 formal(s) move up one register. ONE move — vsprintf_s's own family.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l1_n7(int, int, int, int, int, int, int, int);
int grid_l_forward_l1_n7(int formal_number_0, int formal_number_1, int formal_number_2, int formal_number_3, int formal_number_4, int formal_number_5, int formal_number_6) { return grid_l_callee_l1_n7(formal_number_0, formal_number_1, formal_number_2, formal_number_3, formal_number_4, formal_number_5, 0, formal_number_6); }
