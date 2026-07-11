// Float three-factor product — the commutative MUL-reorder near-miss class
// (Box::Volume shape). Straight-line, no branches/relocs. The return's two
// MULs (`04`) are the reorder move's target; the inner MUL's leaves are two
// FloatLoads.
float fmul3(float a, float b, float c) { return a * b * c; }
