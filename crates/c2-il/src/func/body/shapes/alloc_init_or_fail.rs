//! **W-UNDNAME — a guarded allocation whose initialization is guarded again,
//! linked into a list through the receiver, with a shared error store both
//! failure paths reach through a `goto`.**
//!
//! ```cpp
//!   void C::append(N *node) {
//!       if (node != 0) {
//!           P *p = (P *) gObj.alloc(K_SIZE, K_FLAG);   // a MEMBER call on a
//!           if (p != 0) {                              // named global object
//!               p->OFF_A = node;
//!               p->OFF_B = K_NEG;
//!               p->OFF_C = gVtable;                    // a second data symbol
//!               p->OFF_E = this->OFF_D;
//!           }
//!           this->OFF_D = p;
//!           if (p == 0) goto error;
//!           goto done;
//!       }
//!   error:
//!       this->OFF_F = K_STATUS;                        // a BYTE store
//!   done:
//!       ;
//!   }
//! ```
//!
//! This is `src/xdk/LIBCMT/undname.cpp`'s
//! `?append@DName@@QAAXPAVDNameNode@@@Z`, a FRONTIER TU with exactly one
//! emitted function — so the TU converts on this class or on none.
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-n` lowering
//!
//! The same argument [`super::guard_chain_shared_tail`] and
//! [`super::if_call_join`] make, and it is `docs/ARCHITECTURE_SEAMS.md` §7's: a
//! general control-flow lowering forces a block IR plus a value merge at the
//! join, sequenced with the frame/liveness spine, and that restructure has never
//! been sized. What ships here is **twenty-four words of one named function
//! class, `/O1` only**, `NotImplemented` outside.
//!
//! **Accepting this shape is not a claim about `cflow-if-n` as a class.** It
//! takes ONE more of the frontier's `cflow-if-n` functions.
//!
//! ## What the reference emits, and the three things about it a lowering gets wrong
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-extdata/ref/undname/dis.txt`, committed by the previous lane) and
//! decoded token by token in `work/w-undname/UNDNAME_BODY.md`:
//!
//! ```text
//!   mflr/stw/std r30/std r31/stwu -112   FrameLayout{saved_gprs:2} — byte for byte
//!   mr    r31,r3         (1) PARK the receiver: read after the call
//!   mr    r30,r4         (2) PARK the formal: read after the call
//!   cmplwi cr6,r4,0      (3) ┐ the `node != 0` guard, on cr6 …
//!   bt    26,->Lerr          ┘
//!   lis   r11,0          (4) REFHI  <object>
//!   li    r5,K_FLAG
//!   addi  r3,r11,0       (5) REFLO  <object>, into an ARG_REG
//!   li    r4,K_SIZE
//!   bl    <alloc>            REL24
//!   cmplwi cr0,r3,0      (6) … the `p != 0` guard on **cr0** …
//!   bt    2,->Llink
//!   lis   r11,0          (7) REFHI  <vtable>
//!   stw   r30,OFF_A(r3)
//!   li    r10,K_NEG
//!   addi  r11,r11,0      (8) REFLO  <vtable>, into the SCRATCH ITSELF
//!   stw   r10,OFF_B(r3)
//!   stw   r11,OFF_C(r3)
//!   lwz   r11,OFF_D(r31)
//!   stw   r11,OFF_E(r3)
//!  Llink:
//!   stw   r3,OFF_D(r31)
//!   cmplwi cr6,r3,0      (9) … and the `p == 0` guard on cr6 again
//!   bf    26,->epilogue
//!  Lerr:
//!   li    r11,K_STATUS
//!   stb   r11,OFF_F(r31) (10) a BYTE store, not a word
//!   addi/lwz/mtlr/ld r30/ld r31/blr
//! ```
//!
//! 1. **The externals interleave: `data · callee · data`.** The reference symbol
//!    table from index 15 is `<vtable>` (first referenced at `+0x40`), `<alloc>`
//!    (`+0x34`), `<object>` (`+0x24`) — strictly descending index against
//!    ascending offset, kind ignored. This is the body that pays board **#1720**:
//!    a writer emitting callees and then data symbols gets the middle one wrong,
//!    and **every relocation still resolves**.
//! 2. **Three tests, two condition registers, in the order cr6 · cr0 · cr6.**
//!    Nothing in the source distinguishes them; a class using one CR throughout
//!    emits the right program with two wrong `bc` operands.
//! 3. **The error block is reached three ways and the `goto` is real.** The IL
//!    has a synthesized label whose only statement is a jump to the error label,
//!    plus the `node == 0` arm's jump and the `p == 0` arm's. Three transfers,
//!    two labels, one block — and the block is SUNK below the link store, so IL
//!    order is not block order and nothing may be derived from it.
//!
//! ## The fence
//!
//! Every clause below is required literally, and each names the measurement
//! behind it rather than a preference.
//!
//! * **`/O1` only, asked FIRST, in the PARSER.** Board **#1638** has fired
//!   twice — a mode clause that lives only in the emitter makes the census count
//!   bodies in class that `PortC2` refuses. `census_gate.rs` is the cross-check.
//! * **A non-static member function with exactly ONE explicit formal.** `this`
//!   is r3 and the formal r4, and both are parked; the two `mr`s are a function
//!   of that arity. Asked through [`parse_params`] and [`parse_formals`]
//!   together, so the `this` binding is established positively rather than
//!   inferred from a count.
//! * **The allocation is a MEMBER call on a named global object**, whose address
//!   is the call's `this`. That is what makes argument setup
//!   `lis · li · addi · li` — the `addi` takes r3's slot and the two literals
//!   take r4's and r5's — and a free function with the same three arguments
//!   writes r3 from a literal instead (board **#870**'s regime, which this body
//!   avoids entirely).
//! * **Exactly two explicit call arguments, both literals inside `simm16`.** A
//!   computed argument is an operand stream this class has no words for.
//! * **Both data symbols are DESIGNATORS, and the second is a stored VALUE.**
//!   `<object>` reaches the call as its receiver, `<vtable>` reaches a store as
//!   its right-hand side; a body naming one symbol twice, or naming the same
//!   name for both, is one symbol in c2's table and a different obj.
//! * **The store widths are pinned by TYPE**: the four initializer stores and the
//!   link store are width-4, the status store is **not** — it is the one `stb`,
//!   and a width-4 status store is a `stw` and one wrong word.
//! * **Every label distinct.** Two aliasing labels are one block, and every
//!   displacement after the alias would be right for a program this is not.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::calls::eat_call_token;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_int4_type, is_ptr_to_4, read_token_var, read_type,
    read_varint,
};
use crate::func::AllocInitOrFail;

/// Consume any TYPE and discard it.
fn eat_any_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8, u32), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) => {
            *p += w;
            Ok((tag, kind, id))
        }
        None => Err(blk(seg, *p, what)),
    }
}

/// Consume a TYPE naming a width-4 integer (either sign).
fn eat_int4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_int4_type(tag, kind) => {
            *p += w;
            Ok(())
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// `26 <tok>` — a symbol push. Returns the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `B9 <tok> <TYPE>` — a value read. Returns the token and the type's two
/// discriminating bytes, because for this class the type decides an instruction.
fn eat_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, u8, u8), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    Ok((tok, tag, kind))
}

/// `29 <tok>` — a label definition.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `<op> <tok>` for a transfer opcode. Returns the target label.
fn eat_transfer(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `54 <k>`, requiring the exact depth `k`.
///
/// The depths are pinned rather than merely decoded, for
/// [`super::if_call_join`]'s reason: they are the only place the *bracing* of
/// the source shows up in this stream, and a differently braced body is a
/// different block plan.
fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x54) || !eat_byte(seg, p, k) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// `33 <TYPE> <varint>` — a literal that has to fit `simm16`, because every one
/// of them lands in a `li`/`addi`/`stw` immediate field.
fn eat_lit(seg: &[u8], p: &mut usize, what: &'static str) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "aiof-literal-wider-than-simm16"));
    }
    Ok(k)
}

/// True for a TYPE whose zero-compare c2 emits as **`cmplwi`** — an unsigned
/// width-4 integer or a width-4 pointer.
///
/// **This is the clause that keeps all three guards `cmplwi` and not `cmpwi`.**
/// The emitter has one `cmplwi` per guard and no way to vary it, so a SIGNED
/// operand tested against 0 would be the right program with three wrong words —
/// board #1706's rule (anything the emitter cannot vary must be refused by the
/// reader).
fn is_unsigned_or_ptr4(tag: u8, kind: u8) -> bool {
    is_ptr_to_4(tag, kind) || (is_int4_type(tag, kind) && (kind & 0x0F) == 0x2)
}

/// One zero test: `B9 <tok> <T> · 33 <T> 00 · <rel> · 38 <label>`.
///
/// Returns the label the `brfalse` names. The relation byte is the caller's,
/// because the three tests are `!=`, `!=` and `==` in that order and a body
/// spelling them differently branches the other way.
fn eat_zero_test(
    seg: &[u8],
    p: &mut usize,
    want_tok: u32,
    rel: u8,
    what: &'static str,
) -> Result<u32, Block> {
    let (tok, tag, kind) = eat_load(seg, p, what)?;
    if tok != want_tok {
        return Err(blk(seg, *p, "aiof-test-names-the-wrong-value"));
    }
    if !is_unsigned_or_ptr4(tag, kind) {
        return Err(blk(seg, *p, "aiof-test-is-signed-so-c2-emits-cmpwi"));
    }
    if eat_lit(seg, p, what)? != 0 {
        return Err(blk(seg, *p, "aiof-test-literal-not-zero"));
    }
    if !eat_byte(seg, p, rel) {
        return Err(blk(seg, *p, "aiof-test-relation"));
    }
    eat_transfer(seg, p, 0x38, what)
}

/// `B9 <base> <T> · 33 <int> <k> · 27 <T>` — a member reference `base->k`,
/// returning the byte offset. `27` is the member-offset operator and it emits no
/// instruction of its own: the offset lands in the following load or store's
/// displacement field.
fn eat_member(
    seg: &[u8],
    p: &mut usize,
    want_base: u32,
    what: &'static str,
) -> Result<i32, Block> {
    let (tok, _, _) = eat_load(seg, p, what)?;
    if tok != want_base {
        return Err(blk(seg, *p, "aiof-member-base-is-the-wrong-value"));
    }
    let k = eat_lit(seg, p, what)?;
    if !(0..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "aiof-member-offset-negative"));
    }
    if !eat_byte(seg, p, 0x27) {
        return Err(blk(seg, *p, "aiof-member-not-an-offset-op"));
    }
    eat_any_type(seg, p, what)?;
    Ok(k)
}

/// **The recognizer.** `start` is the first byte after the body's own `53`, the
/// leading line markers and the outer `if`'s own `53` — all eaten by
/// `eat_scopes`, so the cursor arrives on the first guard's `B9`. `lo` is the
/// `4C 4F 11` marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports its dispatch arm's blocker and no census key
/// moves.
pub(crate) fn try_parse_alloc_init_or_fail(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    // Board **#1638**, which has fired twice (w-cfgclass §5.3, w-data §6.5). A
    // gate that lived only in `codegen` would make the census count this body in
    // class while `PortC2` refused it — an error term on the published coverage
    // numerator, and exactly what `census_gate.rs` fails on. Asked FIRST, before
    // any body byte is read, so the refusal cannot depend on how far the walk
    // got.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "aiof-not-o1"));
    }
    // A non-static member function with ONE explicit formal: `this` in r3, the
    // formal in r4. `parse_params` prepends the `this` token when the pre-body
    // region binds one and REFUSES when the binding is undetermined, so this is
    // an established fact and not a count.
    let params = parse_params(seg, lo)?;
    let formals = parse_formals(seg, lo)?;
    if params.len() != 2 || formals.len() != 1 || params[1] != formals[0] {
        return Err(blk(seg, start, "aiof-not-one-formal-member-fn"));
    }
    let this_tok = params[0];
    let node_tok = params[1];

    let mut p = start;

    // ---- `if (node != 0)` --------------------------------------------------
    let l_err_entry = eat_zero_test(seg, &mut p, node_tok, 0x20, "aiof-node-test")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-node-scopes"));
    }

    // ---- `p = (P*) <object>.<alloc>(K_SIZE, K_FLAG);` ----------------------
    eat_opt_stmt_marker(seg, &mut p);
    let dest = eat_designator(seg, &mut p, "aiof-dest-designator")?;
    if dest == this_tok || dest == node_tok {
        return Err(blk(seg, p, "aiof-dest-is-a-parameter"));
    }
    let alloc = eat_designator(seg, &mut p, "aiof-alloc-designator")?;
    let object = eat_designator(seg, &mut p, "aiof-object-designator")?;
    // The object decays to its address (`2C`) and is pushed as the call's `this`
    // (`99`). Both are required: the `2C` is what makes this a *named global's*
    // address rather than a local's, and the `99` is what puts it in r3 ahead of
    // the explicit arguments. Neither emits an instruction of its own — together
    // they are the `lis`/`addi` pair.
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "aiof-object-no-decay"));
    }
    eat_any_type(seg, &mut p, "aiof-object-decay-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-object-decay-varint"))?;
    if !eat_byte(seg, &mut p, 0x99) {
        return Err(blk(seg, p, "aiof-object-not-a-this-push"));
    }
    eat_any_type(seg, &mut p, "aiof-object-this-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-object-this-varint"))?;
    eat_call_token(seg, &mut p)?;
    // `.ex` lists a call's arguments in REVERSE source order, so `k_flag` (the
    // second) arrives first. Required literally: this is the order the two `li`s
    // are emitted in and the registers they land in.
    let k_flag = eat_lit(seg, &mut p, "aiof-arg-flag")?;
    if !eat_byte(seg, &mut p, 0x55) {
        return Err(blk(seg, p, "aiof-arg-flag-sep"));
    }
    eat_any_type(seg, &mut p, "aiof-arg-flag-septype")?;
    let k_size = eat_lit(seg, &mut p, "aiof-arg-size")?;
    if !eat_byte(seg, &mut p, 0x55) {
        return Err(blk(seg, p, "aiof-arg-size-sep"));
    }
    eat_any_type(seg, &mut p, "aiof-arg-size-septype")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "aiof-call-arglist-close"));
    }
    // The cast of the returned pointer, then the store into `p`. The cast emits
    // nothing; requiring it is what says the callee's return type and `p`'s type
    // differ, which is `?getMemory`'s `PAX` against `pairNode *`.
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "aiof-call-result-no-cast"));
    }
    eat_any_type(seg, &mut p, "aiof-call-result-cast-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-call-result-cast-varint"))?;
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-call-store"));
    }
    eat_any_type(seg, &mut p, "aiof-call-store-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-call-store-end"));
    }

    // ---- `if (p != 0) { four stores }` -------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-init-scope"));
    }
    let l_link = eat_zero_test(seg, &mut p, dest, 0x20, "aiof-init-test")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-init-scopes"));
    }

    // store 1: `p->OFF_A = node;`
    eat_opt_stmt_marker(seg, &mut p);
    let off_a = eat_member(seg, &mut p, dest, "aiof-store-a")?;
    let (tok, _, _) = eat_load(seg, &mut p, "aiof-store-a-value")?;
    if tok != node_tok {
        return Err(blk(seg, p, "aiof-store-a-value-not-the-formal"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-store-a-op"));
    }
    eat_any_type(seg, &mut p, "aiof-store-a-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-store-a-end"));
    }

    // store 2: `p->OFF_B = K_NEG;` — a width-4 literal store.
    eat_opt_stmt_marker(seg, &mut p);
    let off_b = eat_member(seg, &mut p, dest, "aiof-store-b")?;
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "aiof-store-b-lit"));
    }
    eat_int4(seg, &mut p, "aiof-store-b-littype")?;
    let k_neg = read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-store-b-varint"))?;
    if !(-0x8000..=0x7FFF).contains(&k_neg) {
        return Err(blk(seg, p, "aiof-store-b-wider-than-simm16"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-store-b-op"));
    }
    eat_int4(seg, &mut p, "aiof-store-b-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-store-b-end"));
    }

    // store 3: `p->OFF_C = <vtable>;` — the SECOND data symbol, and the one
    // whose REFLO writes the scratch register itself.
    eat_opt_stmt_marker(seg, &mut p);
    let off_c = eat_member(seg, &mut p, dest, "aiof-store-c")?;
    let vtable = eat_designator(seg, &mut p, "aiof-vtable-designator")?;
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-store-c-op"));
    }
    eat_any_type(seg, &mut p, "aiof-store-c-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-store-c-end"));
    }

    // store 4: `p->OFF_E = this->OFF_D;` — the one store whose value is a LOAD,
    // and the load whose displacement `OFF_D` the link store below reuses.
    eat_opt_stmt_marker(seg, &mut p);
    let off_e = eat_member(seg, &mut p, dest, "aiof-store-e")?;
    let off_d = eat_member(seg, &mut p, this_tok, "aiof-store-e-value")?;
    if !eat_byte(seg, &mut p, 0x30) {
        return Err(blk(seg, p, "aiof-store-e-value-not-an-indirect-read"));
    }
    eat_any_type(seg, &mut p, "aiof-store-e-read-type")?;
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "aiof-store-e-value-no-cast"));
    }
    eat_any_type(seg, &mut p, "aiof-store-e-cast-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-store-e-cast-varint"))?;
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-store-e-op"));
    }
    eat_any_type(seg, &mut p, "aiof-store-e-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-store-e-end"));
    }
    // **Exactly FOUR initializer stores, and the count is pinned rather than
    // looped.** The emitted schedule interleaves the second `lis`/`addi` into
    // this run — the `lis` before the first store and the `addi` before the
    // third — and with one witness there is no way to tell "after the first"
    // from "three before the last". A fifth store is a different schedule, so
    // the walk consumes four and then requires the block to close.
    eat_close(seg, &mut p, 0x08, "aiof-init-close-8")?;
    eat_close(seg, &mut p, 0x07, "aiof-init-close-7")?;
    if eat_label(seg, &mut p, "aiof-link-label")? != l_link {
        return Err(blk(seg, p, "aiof-link-label"));
    }
    eat_close(seg, &mut p, 0x06, "aiof-link-close-6")?;

    // ---- `this->OFF_D = p;` — the link store, SHARED by both paths ---------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_member(seg, &mut p, this_tok, "aiof-link-store")? != off_d {
        // The load inside the `if` and this store name the SAME member. If they
        // could differ, the emitted `lwz`/`stw` pair would need two
        // displacements and the class would have a field it cannot vary.
        return Err(blk(seg, p, "aiof-link-store-is-a-different-member"));
    }
    let (tok, _, _) = eat_load(seg, &mut p, "aiof-link-value")?;
    if tok != dest {
        return Err(blk(seg, p, "aiof-link-value-is-not-the-allocation"));
    }
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "aiof-link-value-no-cast"));
    }
    eat_any_type(seg, &mut p, "aiof-link-cast-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-link-cast-varint"))?;
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-link-op"));
    }
    eat_any_type(seg, &mut p, "aiof-link-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-link-end"));
    }

    // ---- `if (p == 0) goto error;` — three transfers, two labels -----------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-fail-scope"));
    }
    let l_ok = eat_zero_test(seg, &mut p, dest, 0x1F, "aiof-fail-test")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-fail-scopes"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    // The `goto` itself: a jump to a synthesized label, immediately followed by
    // the jump to the error block that the source's `goto` compiled to and that
    // this arm can never reach. Both are required literally — a body with only
    // the second is a `goto` c2 spelled differently, and its block plan is a
    // different one this emitter has not been graded on.
    let l_goto = eat_transfer(seg, &mut p, 0x3A, "aiof-goto")?;
    let l_err = eat_transfer(seg, &mut p, 0x3A, "aiof-goto-dead")?;
    eat_close(seg, &mut p, 0x08, "aiof-fail-close-8")?;
    eat_close(seg, &mut p, 0x07, "aiof-fail-close-7")?;
    if eat_label(seg, &mut p, "aiof-ok-label")? != l_ok {
        return Err(blk(seg, p, "aiof-ok-label"));
    }
    eat_close(seg, &mut p, 0x06, "aiof-ok-close-6")?;
    eat_close(seg, &mut p, 0x05, "aiof-ok-close-5")?;
    eat_close(seg, &mut p, 0x04, "aiof-ok-close-4")?;
    let l_done = eat_transfer(seg, &mut p, 0x3A, "aiof-skip-error")?;

    // ---- the `node == 0` arm, which falls into the same error block --------
    if eat_label(seg, &mut p, "aiof-entry-err-label")? != l_err_entry {
        return Err(blk(seg, p, "aiof-entry-err-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "aiof-entry-err-scopes"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_transfer(seg, &mut p, 0x3A, "aiof-entry-err-jump")? != l_err {
        return Err(blk(seg, p, "aiof-entry-err-jump-elsewhere"));
    }
    // The synthesized label the `goto` above named, whose only statement is the
    // jump to the error block. This is the third transfer.
    if eat_label(seg, &mut p, "aiof-goto-label")? != l_goto {
        return Err(blk(seg, p, "aiof-goto-label"));
    }
    if eat_transfer(seg, &mut p, 0x3A, "aiof-goto-jump")? != l_err {
        return Err(blk(seg, p, "aiof-goto-jump-elsewhere"));
    }

    // ---- the error block: `this->OFF_F = K_STATUS;` — a BYTE store ---------
    if eat_label(seg, &mut p, "aiof-err-label")? != l_err {
        return Err(blk(seg, p, "aiof-err-label"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let off_f = eat_member(seg, &mut p, this_tok, "aiof-status-store")?;
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "aiof-status-lit"));
    }
    let (stag, skind, sid) = eat_any_type(seg, &mut p, "aiof-status-littype")?;
    if is_int4_type(stag, skind) || is_ptr_to_4(stag, skind) {
        // A width-4 status store is a `stw` and one different word. The one
        // instruction in this body whose OPCODE a type decides.
        return Err(blk(seg, p, "aiof-status-store-is-a-word-not-a-byte"));
    }
    let k_status = read_varint(seg, &mut p).ok_or(blk(seg, p, "aiof-status-varint"))?;
    if !(0..=0x7FFF).contains(&k_status) {
        return Err(blk(seg, p, "aiof-status-not-a-small-nonnegative"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "aiof-status-op"));
    }
    let (ttag, tkind, tid) = eat_any_type(seg, &mut p, "aiof-status-type")?;
    if (ttag, tkind, tid) != (stag, skind, sid) {
        return Err(blk(seg, p, "aiof-status-type-differs"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "aiof-status-end"));
    }

    // ---- the wind-down -----------------------------------------------------
    eat_close(seg, &mut p, 0x05, "aiof-wind-5")?;
    eat_close(seg, &mut p, 0x04, "aiof-wind-4")?;
    if eat_label(seg, &mut p, "aiof-done-label")? != l_done {
        return Err(blk(seg, p, "aiof-done-label"));
    }
    eat_close(seg, &mut p, 0x03, "aiof-done-close-3")?;
    eat_opt_stmt_marker(seg, &mut p);
    let l_epi = eat_transfer(seg, &mut p, 0x3A, "aiof-epilogue-jump")?;
    eat_close(seg, &mut p, 0x02, "aiof-wind-2")?;
    if eat_label(seg, &mut p, "aiof-epilogue-label")? != l_epi {
        return Err(blk(seg, p, "aiof-epilogue-label"));
    }
    // The function tail. Landing exactly on it is the whole acceptance claim: a
    // walk that ends anywhere else consumed a byte it did not understand.
    // PROV[O] the seven-byte `.ex` function tail `4F 12 47 54 01 54 00`, read off captures. Its own comment carries the acceptance claim: a walk that ends anywhere else consumed a byte it did not understand.
    const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];
    if seg.get(p..p + FN_TAIL.len()) != Some(&FN_TAIL[..]) {
        return Err(blk(seg, p, "aiof-not-the-function-tail"));
    }

    // Every label distinct: two of them aliasing would make two different
    // successors one block, and every displacement after the alias would be
    // right for a program this is not.
    let labels = [l_err_entry, l_link, l_ok, l_goto, l_err, l_done, l_epi];
    for i in 0..labels.len() {
        for j in i + 1..labels.len() {
            if labels[i] == labels[j] {
                return Err(blk(seg, p, "aiof-labels-alias"));
            }
        }
    }
    // Three names must be three names. A body naming the same symbol twice is
    // ONE undefined external in c2's table, so the symbol table is a record
    // shorter and every index after it moves.
    let names = [alloc, object, vtable];
    for i in 0..names.len() {
        for j in i + 1..names.len() {
            if names[i] == names[j] {
                return Err(blk(seg, p, "aiof-externals-alias"));
            }
        }
    }

    Ok(BodyShape::AllocInitOrFail(AllocInitOrFail {
        params,
        alloc_tok: alloc,
        object_tok: object,
        vtable_tok: vtable,
        k_size,
        k_flag,
        k_neg,
        k_status,
        off_a,
        off_b,
        off_c,
        off_d,
        off_e,
        off_f,
    }))
}
