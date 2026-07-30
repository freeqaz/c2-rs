// A non-static member function with an ordinary **straight-line** body. `this`
// occupies r3, so the first explicit formal is r4 — and every shape that maps a
// token to an argument register has to know that, not just the one where the fact
// was first discovered.
//
// `int S8::m(int x) const { return x + 1; }` emitted `addi r3,r3,1` where the
// reference has `addi r3,r4,1`: `Port=Mismatch @ offset 537`, on a shape as plain
// as a shape gets, inside an accepted class, on master.
//
// This is the *second* time this exact fact bit, and the repeat is the lesson.
// `il_this_line70.cpp` records the first: the `this` token was located by a bare
// byte search and a member function on source line 70 lost it. That fix went into
// the one place the bug had been found — the indirect-load leaf — and
// `parse_this_token` stayed a function that exactly one shape called. Straight-line
// bodies, tail calls, comparisons and float leaves all went on mapping formals from
// r3. GAPS §6 already said "one fact, one locator"; the fix had obeyed the letter of
// it and not the substance, because a locator nobody consults is not shared.
// `parse_params` is now the single answer to "which register holds this token", and
// every shape uses it.
//
// Found by an adversarial reviewer probing a different change entirely. No fixture
// could have found it: the corpus had member functions with *load* bodies
// (`il_expr_member.cpp`) and straight-line bodies in *free* functions
// (`w5_chain.cpp`), and never the cross.
//
// The cases below are that cross, at every arity — with the free-function twin
// beside each one, since the two differ only in a register and the port emitted
// identical bytes for both.
//
//   m1/f1   one formal    `this` r3, x r4
//   m2/f2   two formals   x r4, y r5
//   m3      three formals and a mixed chain
//   s_*     `static` member — NO `this`, so it must NOT shift (the discriminator:
//           a rule that shifted on "is a member" rather than "binds a `this`"
//           would break exactly here)
//   c_*     const member — same shift; `this` is `C * const` rather than `C *`
//   mem_only reads a member through `this` — the load shape, which already knew
//           about `this`; here as the control that says the two paths agree
//
// `mem_add` (`return a + x;` — a member AND a formal in one body) is deliberately
// NOT here. It refuses on `expr-load-type-A64382`, the const-`this` pointer load,
// which is a different rung; keeping it would make this whole TU
// `Port=NotImplemented` and the fixture would grade nothing. It is the natural case
// to add once pointer-typed loads are admitted.
//
// Out of class deliberately: a member function whose body is anything but this
// class, and any body where the `this` binding cannot be established — which now
// refuses (`this-undetermined`) rather than silently assuming there is none.

struct S8 {
    int a;

    int m1(int x) const;
    int m2(int x, int y) const;
    int m3(int x, int y, int z);
    int mem_only() const;
    static int s1(int x);
    static int s2(int x, int y);
};

int S8::m1(int x) const { return x + 1; }
int S8::m2(int x, int y) const { return x + y; }
int S8::m3(int x, int y, int z) { return x + y + z; }

int S8::mem_only() const { return a; }

int S8::s1(int x) { return x + 1; }
int S8::s2(int x, int y) { return x + y; }

int f1(int x) { return x + 1; }
int f2(int x, int y) { return x + y; }
int f3(int x, int y, int z) { return x + y + z; }

struct Big {
    int p, q;
    int mul(int x, int y) const;
    int sub(int x, int y) const;
};

int Big::mul(int x, int y) const { return x * y; }
int Big::sub(int x, int y) const { return x - y; }
