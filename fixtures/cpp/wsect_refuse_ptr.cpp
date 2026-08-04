// w-sect / board #174 — MUST REFUSE. An address-valued initializer stores zero
// bytes and carries its address entirely in a `.data` relocation; §8.6 records
// member-pointer, vftable and cross-section initializers as unexercised.
// The `.in` element tag is `02` (a symbol address) rather than `01`.
int gi;
int* gp = &gi;
