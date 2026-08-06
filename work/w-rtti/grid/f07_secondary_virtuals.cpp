// FRESH: a secondary base with its own virtuals, so ??_R4.offset is non-zero
// at a value the spec's cells did not produce (its fields are 4 and 8 wide).
struct S1 { S1(); virtual void a(); double x; int y; };
struct S2 { S2(); virtual void b(); int z; };
struct S3 : S1, S2 { S3(); virtual void a(); virtual void b(); int w; };
S3::S3(){}
