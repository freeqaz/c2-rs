// w-sect / board #174 — ONE `.data` object. `.data` comes AFTER the second
// `.XBLD$W` watermark, in 754 of the 754 workload objs that have one.
// Its aux CheckSum is a REAL CRC-32 even though the section is not a COMDAT,
// which refutes OBJ_DYNINIT_SHAPE.md §2.3 (Rule D1).
int d1 = 7;
