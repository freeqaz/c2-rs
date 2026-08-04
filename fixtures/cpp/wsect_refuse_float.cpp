// w-sect / board #174 — MUST REFUSE. A floating-point initializer's bytes are
// OMITTED from the section's aux CheckSum (§4.2.1), and the byte-granularity
// finding behind that rule is labelled "not pre-registered" in its own
// document. Emitting one would encode a rule from three exploratory cells.
// The `.in` element type byte is `05` and the value is raw little-endian rather
// than a varint, so it refuses in the reader before the CheckSum matters.
double fd = 1.0;
