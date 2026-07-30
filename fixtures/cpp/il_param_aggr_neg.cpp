// **Negative** — a parameter's argument-*register* number is not its declaration
// *index*. A by-value aggregate wider than 8 bytes occupies more than one GPR and
// shifts every later parameter along, so the two facts diverge exactly there.
//
// This was a live wrong-bytes emit on mainline, found by an adversarial reviewer
// probing an unrelated change:
//
//   int gb(Big v, H* h) { return h->mi; }   // Big is 20 B -> r3/r4/r5, so h is r6
//   c2:    80 66 00 00   lwz r3,0(r6)
//   port:  80 64 00 00   lwz r3,0(r4)       Mismatch @ offset 537
//
// It is the FOURTH instance of the pattern in `docs/GAPS.md` §6 — two facts
// sharing one field, indistinguishable across the entire corpus because every
// fixture parameter was a scalar. Every gate was green while it was live: 99
// fixtures, four mode lanes at 0 mismatch, and the 2,885-case expression sweep.
//
// The admitted half of the ladder is `fixtures/cpp/il_param_aggr.cpp` — a separate
// translation unit on purpose, because a TU with any refused function emits no obj
// at all and its positive cases would then be graded by nothing. Read the two
// together; the boundary is the point:
//
//   in class (there)          refused (here)
//   ────────────────────      ──────────────────────────
//   4 B  struct   one GPR     12 B struct   TWO GPRs
//   8 B  struct   one GPR     16 B struct   TWO
//   union, float,             20 B struct   THREE — the original mis-emit
//   double, long long,        12 B struct ahead of a member load through `this`
//   reference, array
//
// The 4- and 8-byte cases are the discriminating neighbours: they are the same C
// construct as the 12-byte one and were ALWAYS correct, which is why nothing
// smaller than a 12-byte parameter could have exposed the bug. A corpus holding
// only the safe half of a pair cannot see the dangerous half — stated once more
// because this is the fourth time it has been the mechanism.
//
// `ceil(size/8)` is **contradicted**, not merely unproven — the distinction matters,
// because "fits every point I measured" invites a future implementer to implement
// it. It holds only for *POD* aggregates (16 B → r5, 24 B → r6, measured). Outside
// that, measured by disassembling the reference:
//
//   12 B polymorphic class        ONE GPR   — passed by hidden reference
//   16 B class with a copy ctor   ONE GPR   — likewise; `.sy` even records it as a
//                                             4-byte POINTER, kind 03
//   300 B struct                  ZERO GPRs — stack-homed: lwz r11,324(r1) ;
//                                             lwz r3,0(r11) ; blr
//   65540 B struct                ZERO      — lis/ori/ldx off r1
//
// So the register footprint depends on *how* a type is passed, which depends on its
// triviality and its size in ways this port has not characterized. Refusing is
// therefore not a placeholder for an easy formula; it is the honest answer until the
// passing convention itself is captured.
//
// A *third* fixture, `il_param_poly_neg.cpp`, carries the other refusal reason —
// widths this reader cannot read at all — and it has to be its own TU for the same
// grading reason: `.sy` binds a whole file or none of it, so one unreadable record
// there would relabel every function here from `param-multi-reg` to
// `param-width-undetermined` and this ladder would stop measuring the boundary it
// exists to measure.

struct A3 { int a, b, c; };
struct A4 { int a, b, c, d; };
struct A5 { int a, b, c, d, e; };
struct Vec { float x, y, z; };

struct H { int mi; };

// Two or more GPRs — every one of these MUST be NotImplemented.
int a3(A3 v, H* h) { return h->mi; }
int a4(A4 v, H* h) { return h->mi; }
int a5(A5 v, H* h) { return h->mi; }

struct C {
    int m;
    int g(Vec v, H* h) const;
};

// `this` in r3, `Vec` in r4/r5, so `h` is r6 while its index says r4.
int C::g(Vec v, H* h) const { return h->mi; }
