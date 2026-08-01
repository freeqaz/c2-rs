// WR1 negative — every cell the named-data-symbol address REFUSES, one per
// reason, with the measurement behind each.
//
// `c2rs census` must report **0/N in class** and `c2rs diff` must report
// `Port=NotImplemented`. A cell that quietly enters the class here is a
// wrong-bytes emit somewhere in `wr1_sym_addr.cpp`'s neighbourhood, which is the
// whole point of carrying the refutations as a graded file rather than as prose.
//
//  1. **A STRING LITERAL** (`s1`..`s3`). It is the largest population on the row
//     and it is refused for three independent reasons, any one of which is
//     sufficient: its `.gl` record carries the `25` separator `gl_symbol_index`
//     excludes, so the token resolves to no name at all; at `/Ox` it needs a
//     packed `.rdata` pool of `$SG<n>` STATIC symbols placed BEFORE `.text`
//     (the fixed symbol prefix grows and `.text` becomes section 6); and at the
//     workload's own `/O1` it needs one `??_C@…` COMDAT `.rdata` per literal
//     whose raw bytes live in `.in`, a file this port does not decode.
//     (`docs/IL_CALL_IN_EXPR.md` §17.2 items 2–4.)
//
//  2. **A DEFINED global** (`d1`) and a **defined static data member** (`d2`).
//     Indistinguishable from an extern by mangling — `extern int g;` and
//     `int g;` are both `?g@@3HA` — and the difference is a whole `.data`
//     section in the MIDDLE of the section table. The `.gl` linkage byte is what
//     separates them: `02` undefined-extern, `01` defined here (§17.2 item 7,
//     and `gl::gl_extern_data_names` carries the byte evidence).
//
//  3. **TWO symbols in one call** (`t1`, `t2`). c2 emits exactly ONE `lis`/`addi`
//     pair per function and derives every other address from it —
//     `addi r4,r3,-4`, the difference of the two entries' `.rdata` pool offsets
//     — so instruction selection depends on a whole-TU pool layout that the
//     port's per-function selector cannot see, and WHICH symbol anchors is a
//     hypothesis fitted to 14 witnesses with no mechanism (§17.3 (a)/(b)).
//     18,933 functions wide, and it is a phase, not a rung.
//
//  4. **An OFFSET off the symbol** (`o1`, `o2`). The addend is never folded into
//     the relocation: MEASURED, `g1(&gT.b)` is `lis r11 ; addi r11,r11,0 ;
//     addi r3,r11,4` — a THIRD instruction whose base is the scratch, not the
//     destination (§17.2 item 1).
//
//  5. **A formal that has to MOVE** (`m1`, `m2`). This one is refused at the
//     edge of a capture rather than for want of one: `g3(k, &gI)` with `k` in r4
//     really is `mr r3,r4 ; addi r4,r11,0` and follows the ordinary hoist rule —
//     but the cell beside it does not. `s->m4(&gI, j, k)`, two formals shifting,
//     emits `mr r11,r4 ; lis r10,0 ; mr r6,r5 ; addi r4,r10,0 ; mr r5,r11`:
//     c2 pre-saves into r11 and the `lis` MOVES TO r10, where the obvious
//     descending walk needs no save at all (§17.3 (d)). One moved formal and two
//     are two different schedules and one probe does not separate them, so the
//     gate is the positive one and `call-arg-sym-permuted` is its measured cost.
//
//  6. **A global READ rather than addressed** (`r1`). `gso2(gI)` is `lis r11 ;
//     lwz r3,0(r11)` — a load, not an `addi`, and a different production.
//
//  7. **A CALL that is not a tail call** (`f1`). `return gf(&gI) + 1;` is framed,
//     and every capture behind this class is a leaf.

struct S {
    void so(int*);
    void sq(int, int*);
    void s2(int*, int*);
    void m4(int*, int, int);
};
struct T { int a; int b; };

extern int gI;
extern int gJ;
extern T gT;
extern int gArr[4];

int gDefined = 3;
struct C { static int sm; };
int C::sm = 9;

void gsn(const char*);
void gsn2(const char*, int);
void gso(int*);
void gso2(int);
void gsq(int, int*);
void gs2(int*, int*);
int gf(int*);

// 1 — a string literal, at every position the positive fixture admits an object
void s1() { gsn("aa"); }
void s2(S* s) { s->so((int*)"bb"); }
void s3() { gsn2("cc", 7); }

// 2 — a defined global, and a defined static data member
void d1() { gso(&gDefined); }
void d2() { gso(&C::sm); }

// 3 — two symbols in one call
void t1() { gs2(&gI, &gJ); }
void t2(S* s) { s->s2(&gI, &gJ); }

// 4 — an offset off the symbol
void o1() { gso(&gT.b); }
void o2() { gso(&gArr[2]); }

// 5 — a formal that has to move
void m1(int j, int k) { gsq(k, &gI); }
void m2(S* s, int j, int k) { s->m4(&gI, k, j); }

// 6 — the global's VALUE, not its address
void r1() { gso2(gI); }

// 7 — a framed call rather than a tail call
int f1() { return gf(&gI) + 1; }
