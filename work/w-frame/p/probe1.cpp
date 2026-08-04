void g2(void *, unsigned long);
void h3(void *, unsigned long, void *);
// A: exact w8 shape, control (must be in class)
void a_ctl(void *v1, void *v2, unsigned long ul) {
    if (v1 == 0) { g2(v2, ul); return; }
    h3(v1, 0, v2);
}
// B: w8 shape, scrutinee is 3rd formal (pointer) — isolates the position
void b_pos(void *v1, unsigned long ul, void *v3) {
    if (v3 == 0) { g2(v1, ul); return; }
    h3(v1, 0, v1);
}
// C: as B but int scrutinee — isolates signedness
void c_int(void *v1, unsigned long ul, int a) {
    if (a == 0) { g2(v1, ul); return; }
    h3(v1, 0, v1);
}
// D: as C but else-arm reuses no formal twice
void d_norep(void *v1, unsigned long ul, int a) {
    if (a == 0) { g2(v1, ul); return; }
    h3(v1, 0, 0);
}
