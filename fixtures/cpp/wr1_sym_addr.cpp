// WR1 — the address of a NAMED, UNDEFINED-EXTERNAL data symbol passed as a call
// argument: `void f(S* s){ s->so(&gI); }`, `void f(){ gso(&gI); }`.
//
// Two instructions and a relocation quad, and nothing else. Every word below was
// read off a reference obj (`work/wr1/probes/p1.cpp`, `p2.cpp`, `p4.cpp`, at
// `/Ox /GS- /c`):
//
//   void a1(S* s)        { s->so(&gI); }   3d600000 lis r11,0 · 388b0000 addi r4,r11,0 · b
//   void a3()            { gso(&gI); }     3d600000 lis r11,0 · 386b0000 addi r3,r11,0 · b
//   void a7()            { gsp(&gI, 7); }  3d600000 · 38800007 li r4,7 · 386b0000 · b
//
// carrying REFHI+PAIR at the `lis` and REFLO+PAIR at the `addi`, both PAIR
// records against symbol index 0 — byte-for-byte the shape `coff.rs` already
// emits for a pooled FP constant, which is the question `IL_CALL_IN_EXPR.md`
// §17.2 asked and this fixture answers.
//
// **`a7` is the one that separates two readings of the schedule.** The `lis` is
// hoisted to the top of the function; the `addi` is NOT beside it — it takes its
// own argument slot's turn in the ordinary descending-destination walk, so a
// higher slot's `li` lands between the two halves and REFLO is at +8, not +4.
// Emitting the quad as an adjacent pair (the pooled-constant arrangement) was a
// live wrong-bytes emit on exactly this body.
//
// Every function here must be in class: `c2rs census` N/N and `c2rs diff`
// Match. The refusals — a string literal, a defined or static global, two
// symbols in one call, an offset off the symbol, a formal that has to move —
// are in `wr1_sym_addr_neg.cpp`, one per named cell.
//
// ---- the value axes this varies, and why -----------------------------------
//
//  * **the destination register**, slots 0 through 7 (`a3`, `a1`, `a5`, `a6`),
//    because the slot is the `addi`'s RD field and nothing else in the port
//    writes it;
//  * **the symbol NAME's length across the 8-byte boundary** — `?gI@@3HA` is
//    exactly 8 characters and goes in the COFF symbol record's inline name
//    field, `?gLongerName@@3HA` goes in the string table. Two different code
//    paths in `emit_external_symbol`, and no other fixture in this corpus
//    reaches the second one with a DATA symbol;
//  * **one symbol referenced by several functions** (`?gI@@3HA` below is
//    referenced eight times and must appear as exactly ONE symbol, which every
//    later reference relocates against — the same dedup rule a repeated callee
//    has, and the failure mode is a symbol table one entry too long);
//  * **the object's type**, since the `.gl` linkage gate reads a fixed offset
//    past a two-byte object type: an `int`, an array, a `double`, a `char`, a
//    pointer and a 40-byte class, which are the widths most likely to widen it;
//  * **a framed function in the same TU** (`fr` at the end), because the
//    compiler-label counter is advanced once per function in `.text` order and a
//    wrong stride is six wrong bytes. MEASURED (`work/wr1/probes/p3a.cpp` vs
//    `p3b.cpp`, identical `.gl` seed 2544): a data-address leaf consumes **1**
//    slot, exactly like any other leaf, and the framed function's labels come
//    out $M2554/$M2555/$T2556 either way.

struct S {
    void so(int*);
    void sq(int, int*);
    void sr(int, int, int*);
};

class Big {
public:
    char pad[40];
    virtual void v();
};

extern int gI;
extern int gArr[4];
extern double gD;
extern char gC;
extern int* gPI;
extern Big gBig;
extern int gLongerNameThanEightBytes;

void gso(int*);
void gsp(int*, int);
void gsq(int, int*);
void gs8(int, int, int, int, int, int, int, int*);
void gvo(void*);
void gdo(double*);
void gco(char*);
void gpo(int**);
int gf(int);

// ---- the destination register, across the eight argument slots --------------

void a1(S* s) { s->so(&gI); }                 // this in place, symbol -> r4
void a2(S* s, int k) { s->sq(k, &gI); }       // this + one formal in place -> r5
void a3() { gso(&gI); }                       // symbol -> r3
void a4(int k) { gsq(k, &gI); }               // formal in place, symbol -> r4
void a5(S* s, int j, int k) { s->sr(j, k, &gI); }
void a6(int a, int b, int c, int d, int e, int f, int g) { gs8(a, b, c, d, e, f, g, &gI); }

// ---- the symbol itself: array decay, and every object width -----------------

void b1() { gso(gArr); }
void b2() { gdo(&gD); }
void b3() { gco(&gC); }
void b4() { gpo(&gPI); }
void b5() { gvo(&gBig); }
void b6() { gso(&gLongerNameThanEightBytes); }

// ---- a literal beside the symbol: the `li` lands BETWEEN the two halves ------

void c1() { gsp(&gI, 7); }
void c2() { gsp(&gI, -1); }
void c3() { gsp(&gI, 32767); }

// ---- the same symbol from several functions: ONE undefined external ---------

void d1() { gso(&gI); }
void d2() { gso(&gI); }

// ---- and a FRAMED function after all of them, so the label stride is graded --

int fr(int a) { return gf(a) + 1; }
