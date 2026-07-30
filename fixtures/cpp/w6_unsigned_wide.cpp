// **Negative** — the `==`/`!=` immediate predicate is not the carry spines', and
// sharing one was a live wrong-bytes emit in *both* optimization modes.
//
// The carry spines (`<`, `<=`, `>`, `>=`) gate on raw SIMM16 encodability, so
// `a > 4294967291u` is a perfectly good `subfic r11,r3,-5` and stays in class. The
// difference spines gate on the literal's **unsigned value** lying in `[0, 32767]`:
// against a large unsigned, c2 materializes the constant and subtracts, which is
// one instruction more. The port used the carry rule for both and emitted
// `addi r11,r3,1` for `a == 4294967295u` — four bytes short of the reference,
// diverging at obj offset 8.
//
// Four of the 108 cells of the comparison matrix, at `/Ox` and `/O1` alike. None of
// them was reachable from `w6_rel_k.cpp` or `w6_k_boundary.cpp`, and
// `scripts/expr_sweep.sh` never found them either, because every unsigned literal
// in the sweep is small — it writes `a %s %su` over `0 1 -1 5 -5 2 32767 -32768`
// and drops the negative ones for the unsigned lane, so the whole
// `[0x80000000, 0xFFFFFFFF]` half of the unsigned literal space was untested.
// They surfaced only when the full matrix was enumerated to characterize the `/O1`
// spines (`docs/CODEGEN_W6_O1.md`) — a table built for one purpose finding a bug in
// another.
//
// The near neighbours are all still in class, and that is the point of keeping them
// here: the discriminator is *signedness plus relation*, not the literal, so a
// gate written against the literal alone would refuse four working shapes to fix
// four broken ones.
//
//   n_eq_max / n_ne_max   unsigned == / != 0xFFFFFFFF   refuse
//   n_eq_m5  / n_ne_m5    unsigned == / != 0xFFFFFFFB   refuse
//   w_gt_m5               unsigned >  0xFFFFFFFB        in class (subfic)
//   w_eq_signed           signed   == -1                in class (difference)
//   w_eq_small            unsigned == 32767             in class (difference)

int n_eq_max(unsigned a) { return a == 4294967295u; }
int n_ne_max(unsigned a) { return a != 4294967295u; }
int n_eq_m5(unsigned a) { return a == 4294967291u; }
int n_ne_m5(unsigned a) { return a != 4294967291u; }

int w_gt_m5(unsigned a) { return a > 4294967291u; }
int w_eq_signed(int a) { return a == -1; }
int w_eq_small(unsigned a) { return a == 32767u; }
