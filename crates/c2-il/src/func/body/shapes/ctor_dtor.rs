//! Constructor epilogues and generated destructors.
//!
//! The `return this` a constructor emits after its RETURN (W19), and the
//! compiler-generated empty destructor in its base-delegation and member
//! sub-object forms (W14/W15). The member receiver is a consumer of
//! [`super::designator`].

use crate::func::body::expr::{eat_fn_tail, eat_scopes, BODY_SCOPE_DEPTH};
use crate::func::body::mcall;
use crate::func::body::{BodyShape, DtorSubObject};
use crate::func::readers::{
    eat, eat_byte, eat_opt_stmt_marker, eat_value_type, is_ptr4_kind, read_token_var, read_type,
    read_varint, ValueClass, INT_TYPE,
};

use super::params::parse_params;
use super::this_binding::{ThisBinding, parse_this_token};

/// The **constructor epilogue**: a value expression sitting between the RETURN
/// and the function tail, naming `this`.
///
/// ```text
///   … 3A <label> 54 02 29 <label>   B9 <this> <TYPE> 41 <TYPE>   4F 12 47 54 01 54 00
///                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// Every other shape in this module puts its returned value *before* the `3A`,
/// where [`eat_return_head`]'s `has_result_type` annotation covers it. A
/// constructor does not: its statements each end on a `4B` discard, and the value
/// it returns — `this`, which MSVC constructors hand back in r3 — is written after
/// the `29`. The parse used to stop dead on that `B9`, which is the census key
/// `fn-tail-0xB9`: **29,552 functions, the largest call-free row that was named
/// but never decomposed.**
///
/// **It costs no instruction at all**, and that is measured, not assumed. `this`
/// is already in r3 on entry and a leaf body cannot have moved it, so the epilogue
/// is a no-op. Captured from the live toolchain, eight empty constructors in one
/// translation unit — varying arity, member count, member type and position in the
/// file — every one of them exactly `4E 80 00 20`:
///
/// ```text
///   struct T { int m; T(); };  T::T() {}                 -> blr
///   struct E { int m; E(int); };  E::E(int a) {}         -> blr
///   struct G { int m; G(int,int); };  G::G(int,int) {}   -> blr
/// ```
///
/// That run is also the locality tell `docs/GAPS.md` §6 asks for before a row is
/// taken: byte-identical sources in one TU emitting one sequence means the
/// decision is local, which is what the `data-addr` rung lacked.
///
/// **The leaf restriction is load-bearing and is not conservatism.** Add a call
/// and c2 stops being able to leave `this` in r3:
///
/// ```text
///   struct B { int b; B(); };  struct D : B { D(); };  D::D() {}
///     mflr r12 ; stw r12,-8(r1) ; stw r31,-16(r1) ; stwu r1,-96(r1)
///     mr r31,r3 ; bl B::B ; mr r3,r31 ; …            <- this saved and restored
/// ```
///
/// so the 832 bodies whose epilogue follows a call need the general frame and stay
/// refused (they are `calls-1` to the frame measure, §18). Only the caller decides
/// that: this recognizer is used by exactly one arm, the empty-body one.
///
/// Both fields are required **literally**, per `docs/GAPS.md` §6's rule that a
/// field which never varied is indistinguishable from a constant. The token must
/// be the one [`parse_this_token`] bound — a positive identification, never
/// "some token we could not place" — and the loaded type must be byte-identical
/// to the `41` result type. Across the 29,549 sites the real workload has, the
/// token was `this` in **every** one; requiring it means a body that returns
/// anything else refuses instead of silently emitting a constructor's bytes.
pub(crate) fn eat_ctor_this_epilogue(seg: &[u8], p: &mut usize, lo: usize) -> bool {
    let this_tok = match parse_this_token(seg, lo) {
        Some(ThisBinding::Bound(t)) => t,
        // `Absent` and `None` alike: a free function has no `this` to return, and
        // an undetermined binding must never be read as one.
        _ => return false,
    };
    let mut q = *p;
    if seg.get(q) != Some(&0xB9) {
        return false;
    }
    q += 1;
    let (tok, w) = match read_token_var(seg, q) {
        Some(x) => x,
        None => return false,
    };
    if tok != this_tok {
        return false;
    }
    q += w;
    let load_ty = match read_type(seg, q) {
        Some((_, _, _, tw)) => &seg[q..q + tw],
        None => return false,
    };
    q += load_ty.len();
    if seg.get(q) != Some(&0x41) {
        return false;
    }
    q += 1;
    let res_ty = match read_type(seg, q) {
        Some((_, _, _, tw)) => &seg[q..q + tw],
        None => return false,
    };
    if res_ty != load_ty {
        return false;
    }
    *p = q + res_ty.len();
    true
}

/// Try to parse the **compiler-generated empty destructor** that does nothing but
/// destroy **one** sub-object — the largest coherent sub-shape of the
/// `expr-call-in-expr` bucket (`docs/IL_CALL_IN_EXPR.md` §5, §15).
///
/// There are **two** such destructors and they differ only in how the sub-object's
/// address is spelled. `docs/IL_CALL_IN_EXPR.md` §14.3 separated them; §5 had seen
/// only the first:
///
/// * a **base** sub-object, whose address comes from the `this`-adjust intrinsic
///   2113 and whose adjustment this shape requires to be 0
///   (`RECV-BASE` below, D1);
/// * a **member** sub-object, whose address is a plain `27` byte-offset add of a
///   literal `k` onto `this` — no intrinsic anywhere, and `k` may be zero (the
///   member is first in the layout, so the address arithmetic emits nothing) or
///   nonzero (one `addi r3,r3,k`).
///
/// ```text
///   33 <int> 0                     the leading literal (role UNKNOWN — see below)
///   26 <method-tok>                the SUB-OBJECT destructor, pushed first
///   <RECV-BASE | RECV-MEMBER>      the receiver — one of:
///
///   RECV-BASE:
///     33 <int> 2113  40 <PTR4>     intrinsic `this-adjust`, pointer result
///     66 02 <2 LEB128 type refs>   the class-pair descriptor
///     55 <int>                     selector argument terminator
///     33 <int> 0     55 <int>      the adjust OFFSET — required to be 0
///     B9 <this> <PTR4>  55 <PTR4>  the object pointer
///     4C                           -> the adjusted receiver
///
///   RECV-MEMBER:
///     B9 <this> <PTR4>             the object pointer
///     33 <int> k                   the member's byte offset within the object
///     27 <PTR4>                    byte-offset add -> the member's address
///
///   2C <PTR4> 00                   cv strip
///   99 <PTR4> 00                   member bind (a `99` bind is DIRECT dispatch)
///   BD <void> 00 <fn-type-id>      the CALL, void result, cdecl
///   4C                             ZERO explicit arguments (`this` is not one)
///   5C <int> 01                    opaque statement trailer
///   4B                             statement end
///   3A <lbl> 54 02 29 <lbl>        the return plumbing's branch/close/return
///   5E 01 21                       opaque sub-object trailer
///   4B
///   <function tail, reaching the segment end>
/// ```
///
/// **Why it needs at most one instruction.** A `99` bind is direct dispatch by
/// construction (virtual dispatch is opcode `67` with a `9A` bind —
/// `docs/IL_CALL_IN_EXPR.md` §3), so the call is a direct branch; the call has no
/// result; and nothing follows it. So the whole function is the sub-object's
/// address in r3 followed by a tail branch, and the address is `this` (already in
/// r3, zero instructions) plus a constant. MEASURED at the workload's own
/// `/O1 /Oi /EHsc` for the base form and at the fixture profile for the member
/// forms (`work/rf/probes/p3.cpp`, `q4`, `q7`, `q8`):
///
/// ```text
///   struct B1{~B1();int x;};  struct D1:B1{~D1();int y;};  D1::~D1(){}
///   ??1D1@@QAA@XZ:       48000000  b ??1B1@@QAA@XZ         base, adjust 0
///
///   struct MemA{~MemA();int a;};
///   struct HasMem { ~HasMem();  MemA m; };        HasMem::~HasMem() {}
///   ??1HasMem@@QAA@XZ:   4bfffff0  b ??1MemA@@QAA@XZ       member at 0
///
///   struct HasMem4{ ~HasMem4(); int pad; MemA m; };  HasMem4::~HasMem4() {}
///   ??1HasMem4@@QAA@XZ:  38630004  addi r3,r3,4            member at 4
///                        4bffffe4  b ??1MemA@@QAA@XZ
/// ```
///
/// The `addi` is not a new emitter: the adjust is handed to codegen as the
/// argument-setup operand stream `[Load(this), Lit(k), Add]`, which is what
/// `return g(a + k)` already lowers through (`int_tail_call_text`), so the one new
/// instruction in this shape is emitted by code that four mode lanes and the
/// expression sweep have been grading since the MVP.
///
/// **`k` must fit a signed 16-bit `addi`.** MEASURED: a member at offset 40,000
/// (`work/rf/probes/q3.cpp`, `struct Big{~Big(); char pad[40000]; MemA m;}`) emits
/// **two** instructions, `addis r3,r3,1 ; addi r3,r3,-25536`, which is a second
/// production with one witness. It is refused, and `whole_body_is_one_value` counts
/// that body as complete, so the `-whole` census figure is an upper bound over this
/// gate too.
///
/// **Why this lands in `expr-call-in-expr` at all**: the body opens on the `33`
/// literal, so the straight-line arm runs `parse_expr`, pushes `Lit(0)` and stops
/// on the `26`. The very same production reached through a plain base-method call
/// (`p->Bm()`, no leading literal) opens on the `26`, is dispatched to the
/// assignment parser and files under `expr-intrinsic-this-adjust` — one
/// production split across two census buckets by one leading byte.
///
/// **The two opaque trailers.** `5C <int> <f>` and `5E <n> <g>` are undecoded, and
/// two of those three payload fields **vary** — which is the only reason this
/// grammar is worth writing down rather than transcribing:
///
/// * **`<n>` counts destroyed sub-objects.** MEASURED,
///   `struct N1 : M1, M2 { ~N1(); };` (two bases, each with a destructor) emits
///   *two* member-call statements — the second with a nonzero adjust offset,
///   needing an `addi` — and closes with `5E 02 21` rather than `5E 01 21`.
///   Requiring `01` is therefore a real discriminator against the shape this
///   lowering would get wrong, and it says structurally what the grammar says.
///   MEASURED again for the member form, and it is the gate that matters most
///   there: `struct Two { ~Two(); MemA m; MemB n; };` (two destructible members,
///   at offsets 0 and 4) carries `5E 02 31` and **two** statements each with its
///   own leading `33 <int> 0` literal, and the reference does *not* emit two
///   branches — it emits a **frame**, `or r31,r3,r3`, and the two `bl`s in
///   **reverse declaration order** (`work/rf/probes/q1.cpp`):
///
///   ```text
///     ??1Two@@QAA@XZ: mflr/stw/stwu … ; or r31,r3,r3 ; addi r3,r3,4
///                     bl ??1MemB@@QAA@XZ ; or r3,r31,r31 ; bl ??1MemA@@QAA@XZ
///                     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; … ; blr
///   ```
///
///   `this` is live across the first call, so that shape needs a frame, a
///   callee-saved register and a call order this rung does not model. It is the
///   shape `docs/IL_CALL_IN_EXPR.md` §14.3 measured as 574 bodies "lost" to the
///   offset split, and the loss is real rather than an artifact: those bodies are
///   *grammar*-complete with both offsets admitted and *codegen*-complete under
///   neither.
///
///   **That capture is the FIXTURE profile's, and it understates the workload's
///   by a phase** (measured 2026-07-31, `docs/EH_RECORDS.md` §6). At the dc3
///   workload's `/O1 /Oi /EHsc`, *one* sub-object statement and nothing else is
///   the bare branch above, but a **second** sub-object — or one sub-object plus
///   any other body statement — mints a `__CxxFrameHandler` /
///   `__ehfuncinfo$` prefix, a second `.pdata`, a 64-byte `Selection = 5`
///   `.rdata` and an unwind funclet with the r12→r31 establisher convention.
///   `~Two(){}` is 120 B with EH at the workload profile and 0 B of EH at
///   `/Ox /GS-`. Do not size a widening of this production from the fixture
///   profile: `expr-call-in-expr-recv-{field-off0,field,intrinsic-this-adjust}-then-chain-bind-whole`
///   is 5,188 workload functions of exactly that shape and **none** of it is
///   reachable without the EH model.
/// * **`<f>` and `<g>` carry an exception-handling bit, and they co-vary.**
///   MEASURED by isolating one flag at a time over
///   `{/Od, /O1, /Ox} × {—, /Oi, /GS-, /GR, /EHsc, /EHa}`: **`/EH…` clears bit
///   `0x10` in both**, and nothing else in that matrix moves either byte. So the
///   fixture profile (`/Ox`, no `/EH`) gives `5C … 11` / `5E 01 31` and the dc3
///   workload profile (`/O1 /Oi /EHsc`) gives `5C … 01` / `5E 01 21`, and the
///   reference emits the **same four bytes** for both (checked at `/Ox`,
///   `/Ox /EHsc` and `/O1`). Both pairs are admitted, as a two-entry table of
///   measured values with the bit required to agree between them — not as a
///   skipped field. Had this been pinned to the one profile that was probed
///   first, the shape would have refused the entire workload or the entire
///   fixture lane depending on which one that was.
///
/// What the bit *means* is still UNKNOWN, and a third value refuses. The
/// separating probe for `<f>`'s low nibble would be a destructor of a class with
/// a virtual base, where MSVC's vbase-destruct flag should move it; not tested.
///
/// Each remaining gate, with the neighbour it separates (all MEASURED at
/// `/O1 /Oi /EHsc` against the live 16.00.11886.00 toolchain):
///
/// * **Selector exactly 2113 in its wide form** (base form only). 2113–2119 are
///   seven different operations and only this one is an unguarded adjust; a
///   *virtual* destructor goes through 2117/2116 and a whole different body
///   (`struct N3 : M4 { virtual ~N3(); };`, which blocks as
///   `expr-intrinsic-base-member-addr` and must keep doing so).
/// * **The base form's adjust offset must be 0.** A base at a nonzero offset is
///   reached only by a multi-base destructor, which has two calls and fails the
///   skeleton first, so there is no single-call witness for it. The *member* form's
///   offset is admitted nonzero because there is one — several, above — and the
///   two are separate literals in separate productions, so the base gate is not
///   loosened by the member one.
/// * **The descriptor must be exactly `66 02` + two refs** (base form only). Every
///   witness — a direct base, a two-level chain (`D4 : B4 : G4`), an empty base, a
///   multi-base class — carries `02`, because a destructor delegates exactly one
///   inheritance step. A field that never varied is required literally rather
///   than skipped structurally on the assumption that the shape keeps repeating.
/// * **The member form's offset must be non-negative and fit `addi`.** Zero is the
///   commonest case and emits nothing. A negative adjust has no witness at all —
///   only a virtual-base thunk would plausibly produce one, and that is a
///   different production — so it fails closed rather than being sign-extended on
///   the assumption that `addi` would do the right thing.
/// * **The member form admits `27` only, not `28 00 00`.** Both are byte-offset
///   adds and D2's classifier accepts either, but every captured generated
///   destructor carries the typed `27`; `28` is the subscript spelling
///   (`docs/IL_EXPR_LAYER.md` §4) and has no witness here.
/// * **The call must be `void`, cdecl, and carry ZERO explicit arguments.** The
///   receiver rides the operand stack into the `99`; a `55`-terminated argument
///   here would be a different callee.
/// * **The receiver must be the function's `this`, positively bound, and there
///   must be no other formal.** That is what puts the branch target's `this` in
///   r3 with no register move. `parse_params` refuses an undetermined `this`
///   (the line-70 rule).
/// * **The parse must reach the segment end.** A destructor with a real statement
///   (`N2::~N2() { h(); }`) has a second `26` where the plumbing must begin, and a
///   class with a destructible base *and* a destructible member (`N4 : M5 { M6 m; }`)
///   has a second member-call statement; both refuse, and both really do emit a
///   frame and two `bl`s.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_empty_dtor_delegation(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    /// The `void` result TYPE, required literally: this shape's whole licence to
    /// emit a bare branch is that there is no result to place.
    const VOID_TYPE: [u8; 3] = [0x82, 0x07, 0x03];
    /// The measured `(statement-trailer flag, sub-object-trailer flag)` pairs:
    /// `/EH…` clears bit `0x10` in both, and they always agree. Anything else
    /// refuses. See the doc comment.
    const TRAILER_FLAGS: [(u8, u8); 2] = [(0x11, 0x31), (0x01, 0x21)];

    let mut p = start;
    // The leading literal. Its role is UNKNOWN; it is required to be int-typed
    // and exactly zero, which is what every witness carries.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &INT_TYPE) {
        return None;
    }
    if read_varint(seg, &mut p)? != 0 {
        return None;
    }
    // The sub-object destructor's symbol, pushed before its receiver.
    if !eat_byte(seg, &mut p, 0x26) {
        return None;
    }
    let (callee_tok, w) = read_token_var(seg, p)?;
    p += w;
    // The receiver: the base form's intrinsic frame, or the member form's plain
    // byte-offset add. Tried base-first and each on its own cursor copy, because
    // the two open on different bytes (`33` vs `B9`) and neither may leave the
    // cursor moved for the other.
    let save = p;
    let (recv_tok, adjust, sub_object) = match eat_dtor_base_receiver(seg, &mut p) {
        Some(tok) => (tok, 0, DtorSubObject::Base),
        None => {
            p = save;
            let (tok, k) = eat_dtor_member_receiver(seg, &mut p)?;
            (tok, k, DtorSubObject::Member)
        }
    };
    // The cv strip on the receiver, then the member bind. A `2C`
    // pointer→pointer emits nothing (`docs/IL_LOAD_TYPES.md` §3), and a `99`
    // bind is direct dispatch.
    if !eat_byte(seg, &mut p, 0x2C)
        || !eat_value_type(seg, &mut p, ValueClass::Ptr4)
        || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x99)
        || !eat_value_type(seg, &mut p, ValueClass::Ptr4)
        || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    // The CALL: void result, cdecl, then the per-TU function-type id (decoded
    // only to find the token's end — it does not name the callee).
    if !eat_byte(seg, &mut p, 0xBD) || !eat(seg, &mut p, &VOID_TYPE) || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    read_varint(seg, &mut p)?;
    // Zero explicit arguments, then the two opaque trailers and the statement end.
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }
    if !eat(seg, &mut p, &[0x5C]) || !eat(seg, &mut p, &INT_TYPE) {
        return None;
    }
    let stmt_flag = *seg.get(p)?;
    let (_, want_subobject) = TRAILER_FLAGS.iter().copied().find(|&(f, _)| f == stmt_flag)?;
    p += 1;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // The return plumbing, with the sub-object trailer wedged between the `29`
    // return and the function tail — which is why this cannot call
    // `eat_return_plumbing` and shares only [`eat_fn_tail`] with it.
    let mut depth = depth;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    if !eat_byte(seg, &mut p, 0x3A) {
        return None;
    }
    let (label, w) = read_token_var(seg, p)?;
    p += w;
    for d in (BODY_SCOPE_DEPTH..=depth).rev() {
        eat_opt_stmt_marker(seg, &mut p);
        if !eat(seg, &mut p, &[0x54, d as u8]) {
            return None;
        }
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (back, w) = read_token_var(seg, p)?;
    p += w;
    // The branch and the return name the same label at every witness. Required,
    // for the same reason as everything else here: it is what the bytes say.
    if back != label {
        return None;
    }
    // `5E <n> <g>`: exactly one destroyed sub-object, and its EH bit must agree
    // with the statement trailer's.
    if !eat(seg, &mut p, &[0x5E, 0x01, want_subobject]) || !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_fn_tail(seg, &mut p).ok()?;

    // `this` in r3, no explicit formals, and the receiver IS that `this`.
    let params = parse_params(seg, lo).ok()?;
    if params.as_slice() != [recv_tok] {
        return None;
    }
    Some(BodyShape::EmptyDtorDelegation { callee_tok, this_tok: recv_tok, adjust, sub_object })
}

/// The **base** sub-object's receiver: the `this`-adjust intrinsic 2113 at
/// adjustment 0, whose result is the base's address. Returns the object-pointer
/// token. See [`try_parse_empty_dtor_delegation`]'s `RECV-BASE`.
fn eat_dtor_base_receiver(seg: &[u8], p: &mut usize) -> Option<u32> {
    /// `33 <int> 80 41 08 00 00` — selector 2113 `this-adjust`, wide form.
    const SELECTOR_2113: [u8; 5] = [0x80, 0x41, 0x08, 0x00, 0x00];

    // The `this`-adjust intrinsic, whose result is the receiver.
    if !eat_byte(seg, p, 0x33)
        || !eat(seg, p, &INT_TYPE)
        || !eat(seg, p, &SELECTOR_2113)
        || !eat_byte(seg, p, 0x40)
    {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, *p)?;
    if !is_ptr4_kind(tag, kind) {
        return None;
    }
    *p += tw;
    // The class-pair descriptor: exactly two type references, whose ids are
    // **LEB128** and not a fixed two bytes each ([`super::mcall::eat_class_descriptor`]).
    //
    // Stepping four bytes here is what made this shape refuse bodies that are its
    // own skeleton byte for byte, in every translation unit large enough to have
    // wide type ids — `src/App.cpp` carries one. It was found by a residue: the D2
    // split first spread 17,757 functions over 197 `op-0xNN` buckets, and flat over
    // the byte range is the signature of reading a payload as vocabulary.
    let n_refs = mcall::eat_class_descriptor(seg, p)?;
    if n_refs != 2 {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    // The adjust offset — zero, or this needs an `addi`. Unlike the member form's
    // offset this one stays pinned at zero: the only nonzero-adjust base is the
    // second base of a multi-base destructor, which has two calls and no
    // single-branch witness.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    if read_varint(seg, p)? != 0 {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    // The object pointer.
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (recv_tok, w) = read_token_var(seg, *p)?;
    *p += w;
    if !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    if !eat_byte(seg, p, 0x4C) {
        return None;
    }
    Some(recv_tok)
}

/// The **member** sub-object's receiver: `this` plus a literal byte offset through
/// a plain `27` add, with no intrinsic anywhere. Returns
/// `(object-pointer token, offset)`. See [`try_parse_empty_dtor_delegation`]'s
/// `RECV-MEMBER`.
///
/// The offset is required to be non-negative and to fit a signed 16-bit `addi`,
/// which is the whole codegen difference between this receiver and the base one —
/// and the boundary is measured, not assumed: a member at offset 40,000 emits
/// `addis r3,r3,1 ; addi r3,r3,-25536` (`work/rf/probes/q3.cpp`).
fn eat_dtor_member_receiver(seg: &[u8], p: &mut usize) -> Option<(u32, i32)> {
    // The object pointer. `this` is `A6`-tagged in a destructor and `86`-tagged
    // through a non-const path; `ValueClass::Ptr4` admits both and refuses the
    // width-8 and aggregate spellings.
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (recv_tok, w) = read_token_var(seg, *p)?;
    *p += w;
    if !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    // The member's byte offset within the object, as an int literal.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    let adjust = read_varint(seg, p)?;
    if adjust < 0 || i16::try_from(adjust).is_err() {
        return None;
    }
    // `27 <PTR4>` — the typed byte-offset add. Not `28 00 00`: that spelling has
    // no witness in this production.
    if !eat_byte(seg, p, 0x27) || !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    Some((recv_tok, adjust))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the globs keep that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::testutil::*;
    #[allow(unused_imports)]
    use crate::func::body::{parse_segment, parse_segment_detail};
    #[allow(unused_imports)]
    use crate::func::bundle::LO_MARKER;
    #[allow(unused_imports)]
    use crate::func::readers::find_subslice;
    #[allow(unused_imports)]
    use crate::func::sy::{Formals, SyView};
    #[allow(unused_imports)]
    use crate::func::test_fixtures::*;
    // ---- the generated empty destructor (D1) --------------------------------

    /// Splice a replacement for the first occurrence of `find` in `DTOR_DELEGATE`,
    /// leaving every other byte alone. Every negative below is one such edit, so
    /// each asserts about exactly one field.
    fn dtor_with(find: &[u8], repl: &[u8]) -> Vec<u8> {
        let at = DTOR_DELEGATE
            .windows(find.len())
            .position(|w| w == find)
            .expect("the field being edited");
        let mut v = DTOR_DELEGATE[..at].to_vec();
        v.extend_from_slice(repl);
        v.extend_from_slice(&DTOR_DELEGATE[at + find.len()..]);
        v
    }

    #[test]
    fn the_generated_empty_destructor_parses_under_both_trailer_flags() {
        // The same source captured twice, at the workload's flags and at the
        // fixtures'. The two differ only in the trailers' `0x10` bit and the
        // reference emits the same four bytes for both.
        for (seg, label) in [(DTOR_DELEGATE, "/O1 /Oi /EHsc"), (DTOR_DELEGATE_NOEH, "/Ox")] {
            assert_eq!(
                parse_segment(seg, NO_LOCALS),
                Some(BodyShape::EmptyDtorDelegation {
                    callee_tok: 0xE409,
                    this_tok: 0xFC09,
                    adjust: 0,
                    sub_object: DtorSubObject::Base
                }),
                "{label}"
            );
        }
    }

    // ---- the generated empty destructor, MEMBER form ------------------------

    /// Splice a replacement for the first occurrence of `find` in one of the member
    /// segments, leaving every other byte alone.
    fn mem_dtor_with(seg: &[u8], find: &[u8], repl: &[u8]) -> Vec<u8> {
        let at = seg
            .windows(find.len())
            .position(|w| w == find)
            .expect("the field being edited");
        let mut v = seg[..at].to_vec();
        v.extend_from_slice(repl);
        v.extend_from_slice(&seg[at + find.len()..]);
        v
    }

    #[test]
    fn the_member_destructor_parses_at_both_offsets() {
        // The two productions differ by exactly one literal, and that literal is
        // the whole codegen difference: nothing at 0, one `addi r3,r3,4` at 4.
        // MEASURED, `work/rf/probes/p3.cpp`:
        //   ??1HasMem@@QAA@XZ:   b ??1MemA@@QAA@XZ
        //   ??1HasMem4@@QAA@XZ:  addi r3,r3,4 ; b ??1MemA@@QAA@XZ
        assert_eq!(
            parse_segment(DTOR_MEMBER_OFF0, NO_LOCALS),
            Some(BodyShape::EmptyDtorDelegation {
                callee_tok: 0xE409,
                this_tok: 0x090A,
                adjust: 0,
                sub_object: DtorSubObject::Member
            }),
            "member at offset 0"
        );
        assert_eq!(
            parse_segment(DTOR_MEMBER_OFF4, NO_LOCALS),
            Some(BodyShape::EmptyDtorDelegation {
                callee_tok: 0xE409,
                this_tok: 0x0C0A,
                adjust: 4,
                sub_object: DtorSubObject::Member
            }),
            "member at offset 4"
        );
    }

    #[test]
    fn the_member_offset_must_fit_one_addi() {
        // MEASURED at the boundary (`work/rf/probes/k32764.cpp` / `k32768.cpp`,
        // `char pad[k]` before the member): 32,764 emits `addi r3,r3,32764` and
        // 32,768 emits **two** instructions, `addis r3,r3,1 ; addi r3,r3,-32768`.
        // The gate is therefore at the signed-16-bit edge and not at a round number,
        // and the escape spelling of the literal (`80` + 4 LE bytes) is what carries
        // a value that wide.
        let lit = |k: i32| {
            let b = k.to_le_bytes();
            vec![0x33, 0x86, 0x41, 0x74, 0x80, b[0], b[1], b[2], b[3], 0x27]
        };
        let find = [0x33, 0x86, 0x41, 0x74, 0x04, 0x27];
        for k in [8i32, 32_764, 32_767] {
            let seg = mem_dtor_with(DTOR_MEMBER_OFF4, &find, &lit(k));
            assert!(
                matches!(
                    parse_segment(&seg, NO_LOCALS),
                    Some(BodyShape::EmptyDtorDelegation { adjust, .. }) if adjust == k
                ),
                "offset {k} fits one addi"
            );
        }
        for k in [32_768i32, 65_536, -4] {
            let seg = mem_dtor_with(DTOR_MEMBER_OFF4, &find, &lit(k));
            assert_eq!(parse_segment(&seg, NO_LOCALS), None, "offset {k} does not");
        }
    }

    #[test]
    fn two_destroyed_members_in_one_body_refuse() {
        // The gate that matters most for the member form. MEASURED,
        // `work/rf/probes/q1.cpp` (`struct Two { ~Two(); MemA m; MemB n; };`): two
        // statements, each with its own leading `33 <int> 0` literal, `5E 02 31`,
        // and the reference emits a FRAME — `or r31,r3,r3`, two `bl`s in reverse
        // declaration order, `or r3,r31,r31` between them — because `this` is live
        // across the first call. Admitting it as one branch would be a wrong-bytes
        // emit, so both `5E 01` and reaching the segment end must refuse it.
        assert_eq!(
            parse_segment(
                &mem_dtor_with(DTOR_MEMBER_OFF0, &[0x5E, 0x01, 0x31], &[0x5E, 0x02, 0x31]),
                NO_LOCALS
            ),
            None,
            "two destroyed sub-objects"
        );
        assert_eq!(
            parse_segment(
                &mem_dtor_with(DTOR_MEMBER_OFF0, &[0x3A, 0x0A, 0x0A], &[0x26, 0xE4, 0x09]),
                NO_LOCALS
            ),
            None,
            "a second statement where the plumbing must begin"
        );
    }

    #[test]
    fn the_member_receiver_must_be_this_and_must_be_an_offset_add() {
        // The lowering puts the address in r3 with at most an `addi`, which is only
        // right because the base of the add is the incoming `this`.
        let mut seg = DTOR_MEMBER_OFF0.to_vec();
        let at = find_subslice(&seg, &LO_MARKER).unwrap();
        let recv = seg[at..]
            .windows(7)
            .position(|w| w == [0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x81, 0x20])
            .expect("the object pointer")
            + at;
        seg[recv + 1] = 0xF7; // a token no `2D` entry and no `this` group names
        assert_eq!(parse_segment(&seg, NO_LOCALS), None, "a receiver that is not this");
        // `28 00 00` is the other byte-offset add. D2's classifier accepts either,
        // but this production has no `28` witness, so it fails closed rather than
        // being admitted on the assumption that the two spell the same thing.
        assert_eq!(
            parse_segment(
                &mem_dtor_with(
                    DTOR_MEMBER_OFF0,
                    &[0x27, 0xA6, 0x43, 0x8A, 0x20],
                    &[0x28, 0x00, 0x00]
                ),
                NO_LOCALS
            ),
            None,
            "the untyped `28` offset add"
        );
    }

    #[test]
    fn the_member_form_keeps_the_base_forms_gates() {
        // The two receivers are alternatives in one shape, so a gate loosened for
        // one must not leak into the other. The base form's adjust offset stays
        // pinned at 0 (`a_nonzero_base_adjust_refuses`), and the member form's
        // leading literal stays pinned at 0 here — it is the byte that tells this
        // production apart from the 2117 `base-member-addr` designator, which opens
        // on the same `33` and carries the selector as its payload.
        let mut seg = DTOR_MEMBER_OFF0.to_vec();
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        seg[lo + 7] = 0x01; // the leading LIT's varint
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn the_trailer_flags_must_agree_and_a_third_value_refuses() {
        // The two flags co-vary across every witness. A mixed pair is not a
        // capture this port has ever seen, so it fails closed rather than being
        // read as "the bit does not matter".
        assert_eq!(
            parse_segment(&dtor_with(&[0x5E, 0x01, 0x21], &[0x5E, 0x01, 0x31]), NO_LOCALS),
            None,
            "EH bit clear in 5C, set in 5E"
        );
        // And an unmeasured flag value refuses outright.
        assert_eq!(
            parse_segment(&dtor_with(&[0x5C, 0x86, 0x41, 0x74, 0x01], &[0x5C, 0x86, 0x41, 0x74, 0x21]), NO_LOCALS),
            None,
            "an unmeasured statement-trailer flag"
        );
    }

    #[test]
    fn two_destroyed_subobjects_refuse() {
        // `5E <n> …` counts destroyed sub-objects, MEASURED: a two-base
        // destructor emits `5E 02 21` and two calls, the second at a nonzero
        // adjust. Requiring `01` is the gate that keeps this lowering — one bare
        // branch — away from that shape. This is the one payload field whose
        // variation is understood, so it is the one that must be pinned.
        assert_eq!(
            parse_segment(&dtor_with(&[0x5E, 0x01, 0x21], &[0x5E, 0x02, 0x21]), NO_LOCALS),
            None
        );
    }

    #[test]
    fn a_nonzero_base_adjust_refuses() {
        // A base at a nonzero offset costs a real `addi r3,r3,k` before the
        // branch. The adjust literal is the second `33 86 41 74 00` in the body;
        // the first is the leading literal, so edit through the `55` that follows.
        let seg = dtor_with(
            &[0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9],
            &[0x33, 0x86, 0x41, 0x74, 0x04, 0x55, 0x86, 0x41, 0x74, 0xB9],
        );
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_different_layout_intrinsic_refuses() {
        // 2113 is the UNguarded adjust. 2114 (`base-upcast`) is null-guarded and
        // lowers to five instructions with a control-flow split; the whole family
        // differs, so the selector is required exactly.
        let seg = dtor_with(
            &[0x80, 0x41, 0x08, 0x00, 0x00],
            &[0x80, 0x42, 0x08, 0x00, 0x00],
        );
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_receiver_that_is_not_this_refuses() {
        // The lowering is a bare branch precisely because `this` is already in r3.
        // Rebind the intrinsic's object-pointer argument to a token the pre-body
        // region does not bind, leaving the `this` group itself intact.
        let at = DTOR_DELEGATE
            .windows(12)
            .position(|w| w == [0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20])
            .expect("the object-pointer argument");
        let mut seg = DTOR_DELEGATE.to_vec();
        seg[at + 1] = 0xF7; // a token no `2D` entry and no `this` group names
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn the_leading_literal_must_be_zero_and_int_typed() {
        // Its role is UNKNOWN, so a different value is a body this grammar has no
        // witness for. (The 2117 `base-member-addr` designator is anchored on the
        // same `33` and is told apart by exactly this payload.)
        let mut seg = DTOR_DELEGATE.to_vec();
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        seg[lo + 7] = 0x01; // the leading LIT's varint
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_second_statement_and_a_short_segment_both_refuse() {
        // A destructor with a real statement, or with a destructible member, has a
        // second `26` where the return plumbing must begin — and really does emit
        // two branches and a frame.
        assert_eq!(
            parse_segment(&dtor_with(&[0x3A, 0xFD, 0x09], &[0x26, 0xE4, 0x09]), NO_LOCALS),
            None,
            "a second statement"
        );
        // And the parse must reach the segment end, which is the fail-closed
        // terminal every accepted shape shares.
        let cut = DTOR_DELEGATE.len() - 7; // drop the `47 54 01 54 00` fn tail
        assert_eq!(parse_segment(&DTOR_DELEGATE[..cut], NO_LOCALS), None);
    }

}
