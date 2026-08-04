extern void g(void);
extern void h(void);
extern int  gi(int);
extern void g2(void*, unsigned long);

// A: framed?  one bl, two blocks, no value live across
void a_if_call_then(int a) { if (a) g(); h(); }

// B: two bl on exclusive paths, converging on the epilogue
void b_if_else_calls(int a) { if (a) g(); else h(); }

// C: two bl on exclusive paths + a trailing call (forces a frame)
void c_if_else_then(int a) { if (a) g(); else h(); g(); }

// D: a value live across a call, in a different block
int d_live(int a, int b) { int r = 0; if (a) r = gi(b); return r; }

// E: the negate_test shape without the enum: nested if, two calls, one join
int e_nest(int a, int b) {
    int r = 0;
    if (a >= 1) {
        if (a != 1) {
            if (a >= 2) r = gi(b); else r = gi(b + 1);
        }
    }
    return r;
}

// F: framed, one bl, value returned from it
int f_one_call(int a) { return gi(a) + 1; }
