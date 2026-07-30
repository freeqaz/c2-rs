// **Characterization** — `0x9B`, the compiler temporary, and why it is not `0x99`.
//
// The census bucket `body-0x9B` (27,190 functions, 1.1%) had no reading. It is a
// **designator for a compiler-generated temporary**, and its production is
//
//   9B <TYPE> <token>
//
// where the trailing field is a whole `read_token_var`, not the one-byte varint the
// visually adjacent `99` member-bind uses. Getting that wrong desynchronizes within
// one token, and it already did once: the single real-TU scope-depth counterexample
// recorded in `docs/IL_STMT_GRAMMAR.md` §12.4 is a scanner reading
// `9b 86 46 80 20 11 54 …` as `9B <TYPE> <1-byte varint>` and then treating the
// `54` — which is the token's own second byte — as a scope close.
//
// `t_tmp` below is the shape. `S t = mk();` with `mk()` returning a struct is
//
//   26 <t>                          push the local
//   9b 86 86 80 20 <temp-tok>       the temporary designator
//   26 <mk> bd 86 86 80 20 … 4c     call mk() -> S
//   32 86 86 80 20                  store the result into the temporary
//   9b 86 86 80 20 <temp-tok>       the same temporary again
//   44                              (payload-free; see below)
//   30 a6 86 8d 20                  load the struct out of it
//   32 86 86 80 20                  store into t
//   4b
//
// **The decisive width test is [P] `work/expr/p9.cpp`** (untracked; 32000
// `extern int vNNNNN;` declarations, regenerable), which pushes the token counter
// past 0x8000 and forces the wide form:
//
//   9b 86 86 80 20 f2 86 01 00      <- FOUR trailing bytes
//
// `f2 86 01 00` decodes as `read_token_var` to 0x86F2 = 34546, which is exactly one
// past `t` (34545) and two past the epilogue label (34544) in that TU's sequential
// allocation. Under the varint reading the field would be `f2` and the parse would
// resume on `86 01 00 26 …`, which is not a token boundary at all.
//
// The **complementary** test is [P] `work/expr/p10.cpp` — the same 32000-symbol
// TU with a member function — which shows `99` does *not* widen:
//
//   b9 ee 86 01 00 a6 43 82 20  99 86 43 84 20 00  46 4c 4f 11
//                               ^^^^^^^^^^^^^^^^^ still one trailing byte
//
// It stays `00` with a 34000-token space, and the `46` formals marker follows
// immediately — which it must, since every non-member function has one. So `99` and
// `9B` are adjacent opcodes with *different* trailing-field encodings, and neither
// can be inferred from the other.
//
//   UNKNOWN: `0x44`. It is **payload-free** at both sites here (`44 30 …` and
//   `44 55 …`), which contradicts `docs/IL_CALL_GRAMMAR.md` §7's provisional
//   `44 <TYPE>`: the byte after it is `30`/`55`, whose bit 7 is clear, so it cannot
//   be a TYPE. It sits between a temporary designator and a use of it, so
//   "materialize / bind" is the obvious guess and nothing here tests it.
//
//   UNKNOWN: the `99` trailing byte's meaning. Zero in every observation, including
//   a member function of a class with a base. **A fixture that would separate it:**
//   a member call on a class with multiple or virtual inheritance, where a `this`
//   adjustment is needed — if the byte is an offset it should become non-zero.
//
// Everything here must keep refusing: struct temporaries copy through the FP unit
// (`t_tmp` is `lfd f11,0(r3) ; stfd f11,-16(r1) ; lwz r3,-16(r1)`), which the frame
// model does not have.

struct S { int a; int b; };
struct C { int a; int b; int get() const; };
extern S mk();
extern int usr(const S& s);

int C::get() const { return b + a; }

int t_tmp() {
    S t = mk();
    return t.a;
}

int t_ref() { return usr(mk()); }
