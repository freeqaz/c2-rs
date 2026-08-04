// **Board #232 — the `26` name separator marks a symbol c2 puts in its OWN
// `.text` COMDAT, even in packed mode.** The shape that turned a refusal into a
// wrong emit, and the axis `il_gl_sep26.cpp` holds fixed.
//
// Found by `scripts/expr_sweep.sh` at `checked=14484 mismatches=1`
// (`62-ctor-base-delegation-0032`), bisected to `d0d8a98` — *"c2-il: the gate
// reads the 26 name separator (W-ADOPT, #151)"* — whose own message named this
// exact risk as *"the one place the widening could have produced wrong bytes
// instead of a refusal"* and added `il_gl_sep26.cpp` to guard it.
//
// **The guard was real and it could not see this.** `il_gl_sep26.cpp`'s
// `26`-introduced symbol is `??_GR` — a *deleting* destructor beside a vftable
// — so its TU is out of class for four other reasons and the `NotImplemented`
// it asserts is right for the wrong cause. The axis it holds fixed is
// **implicit vs explicit**, and the defect is on that axis: here `??1M` is a
// destructor **nobody wrote**, implicitly generated because `Bd` has one, and
// everything else about the TU is squarely in the port's class.
//
// What the reference emits at `/Ox /GS- /c` — SEVEN sections, TWO `.text`:
//
//     5  .text   raw=4   chars=0x60401020  sel=2   ??1M@@QAA@XZ   <- own COMDAT
//     6  .text   raw=48  chars=0x60400020          ??0D@@QAA@XZ   <- packed
//     7  .pdata  raw=8
//
// and `.gl` introduces `??0D@@QAA@XZ` with `00` and `??1M@@QAA@XZ` with `26`.
// The correspondence is exact and it is the whole content of the gate's new
// clause: **a DEFINED record whose name is `26`-introduced refuses the TU.**
//
// The port emitted SIX sections with both symbols packed into one `.text`, in
// the opposite order — `Port=Mismatch @ offset 2`, `NumberOfSections`.
//
// **The verdict this fixture must produce is `NotImplemented`**, and that is
// the claim under test. Teaching the packed writer to mint a per-function
// COMDAT is an emit-set / section-layout model — `docs/STATUS.md` puts that in
// Phase 7 and says plainly it is not reachable by widening — and its ordering
// half is unmeasured: across the generated sweep twelve cases carry two `.text`
// sections in packed mode and the COMDAT is **not always first**
// (`51-dtor-member` 0250 and 0255 put it second). Eleven of those twelve
// already refused for unrelated reasons; this was the one route through.
//
// The executable half of the control is
// `gl::tests::a_26_introduced_record_is_SEEN_by_the_scanner_and_REFUSED_by_the_gate`,
// which asserts the split W-ADOPT's own test conflated — the widened *scanner*
// still sees the name, and the *gate* refuses the record — and flips the one
// separator byte to `00` to show the refusal keys on it and on nothing else.

struct Bd { Bd(); ~Bd(); int b0; };
struct M : Bd { M();  };
struct D : M { D();  };
D::D() {}
