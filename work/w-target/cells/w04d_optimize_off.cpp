// GRID-W cell w04d_optimize_off — w04 variant — the intermediate is compiled at a different optimize mode. `splice.rs` has S6-mode-mismatch for the body; this asks the same question of the TARGET
void ext();
#pragma optimize("", off)
void g() { ext(); }
#pragma optimize("", on)
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
