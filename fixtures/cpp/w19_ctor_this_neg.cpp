// W19 negative — the boundary of the constructor epilogue.
//
// Every function here must be **out of class** (`census` 0/N) and the file must
// never mismatch. Each one is a constructor, so each one carries the same
// `return this` epilogue `w19_ctor_this.cpp` admits; what disqualifies them is
// everything *else* in the body.
//
// The first group is the one that matters, and it is the reason the recognizer
// is wired into the empty-body arm alone rather than into the shared return
// plumbing. **A call takes `this` out of r3**, and c2 has to put it back:
//
//   struct B { int b; B(); };  struct D : B { D(); };  D::D() {}
//     mflr r12 ; stw r12,-8(r1) ; stw r31,-16(r1) ; stwu r1,-96(r1)
//     mr r31,r3            <- `this` spilled to a nonvolatile
//     bl B::B
//     mr r3,r31            <- and restored, because it is the return value
//     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; lwz r31,-16(r1) ; blr
//
// That is a framed body and a second register move, neither of which this rung
// lowers. On the real workload it is **832 functions** (`calls-1` to the frame
// measure, `docs/IL_CALL_IN_EXPR.md` §18) against the 28,717 leaf ones — the
// split was measured by counterfactual before the rung was taken, not guessed
// after it.

// ---- a call in the body: the frame axis ----------------------------------
struct Base { int b; Base(); };
struct Derived : Base { Derived(); };
Derived::Derived() {}

void side_effect();
struct CallsFree { int m; CallsFree(); };
CallsFree::CallsFree() { side_effect(); }

struct Member { int m; Member(); };
struct HasMember { Member sub; HasMember(); };
HasMember::HasMember() {}

// ---- a store through `this`: the `27` designator ---------------------------
//
// `m = a` is `B9 <this> <ptr> 33 <int> 0 27 <TYPE> …  32 <TYPE> 4B`, whose
// leading `27` is `expr-op-0x27` — 504,438 functions and measured to **0.14 %**
// whole-body complete in §22.2. It is a different rung and a much worse one.
struct StoresBody { int m; StoresBody(int); };
StoresBody::StoresBody(int a) { m = a; }

struct StoresInit { int m; StoresInit(int); };
StoresInit::StoresInit(int a) : m(a) {}

struct StoresTwo { int m, n; StoresTwo(int, int); };
StoresTwo::StoresTwo(int a, int b) : m(a), n(b) {}

// ---- a vtable to install --------------------------------------------------
struct Virt { int m; virtual void f(); Virt(); };
Virt::Virt() {}

// ---- a returned object that is NOT `this` ---------------------------------
//
// The recognizer identifies the returned token positively against
// `parse_this_token`, so a body whose epilogue names anything else refuses
// rather than being read as a constructor. This is the shape that would have
// exercised the two-valued-answer bug `docs/GAPS.md` §6 keeps warning about.
struct Val { int m; Val(); };
Val make_val();
Val make_val() { Val v; return v; }
