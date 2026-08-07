//! The **store leaf**: `s->m = v;`, integer and floating-point.
//! Third consumer of [`super::designator`]; see `docs/IL_STORE_LEAF.md`.

use crate::func::body::expr::{
    eat_fn_tail, eat_return_head, eat_return_plumbing, eat_scopes, parse_formals, BODY_SCOPE_DEPTH,
};
use crate::func::body::BodyShape;
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, eat_value_type, is_ptr4_kind, is_volatile_tag, read_token_var,
    read_type,
    read_varint, value_class,
};
use crate::func::sy::{fp_reg_of, ArgClass, SyView};
use crate::func::IlOp;

use super::ctor_dtor::eat_ctor_this_epilogue;
use super::designator::{
    eat_addr_offset_adds, eat_offset_adds, is_ptr_any, parse_base_member_designator,
    sized_ptr_width, store_fp_value_width, store_value_width,
};
use super::params::parse_params;
use crate::func::readers::is_ptr_to_4;

/// Try to parse a **store leaf**: a whole body that is one store into a
/// sub-object and nothing else — `void f(S* s, int v){ s->m = v; }`,
/// `void D::set(int v){ Base::m = v; }`, `void f(S* s, int v){ s->arr[2] = v; }`,
/// `void f(int* p, int v){ *p = v; }`, `void f(S* s){ s->m = 7; }`.
///
/// ```text
///   <designator>                       the object pointer, the same two spellings
///   ( 33 <int-like> k 27 <PTR>         byte-offset adds, any number, summed
///   | 33 <int-like> k 28 00 00 )*
///   [ 2C <PTR> 00 ]                    a cv strip / array-to-pointer decay
///   ( B9 <tok> <VT>                    THE VALUE: a formal,
///   | 33 <VT> <k>                      an integer literal,
///   | <designator> 30 <VT> [2C <VT'> 00] )   or an indirect LOAD — WSL,
///                                      `d->a = s->b;`, the body of every
///                                      hand-written copy assignment
///                                      ([`parse_load_value`])
///   32 <VT>                            the store; its TYPE restates the value's
///   4B                                 statement end — and the body ends here
///   <return plumbing, void, reaching the segment end>
/// ```
///
/// where `<designator>` is either a plain pointer LOAD `B9 <tok> <PTR4>` or the
/// intrinsic-2117 `base-member-addr` production ([`parse_base_member_designator`]),
/// whose two literals contribute their sum to the offset before the adds — the
/// same pair of spellings [`try_parse_addr_leaf`] and
/// [`try_parse_indirect_load_leaf`] accept, reached through the same decoder.
///
/// **This is one store instruction, and the width picks it.** MEASURED at the
/// fixture profile — every word below read off the reference obj
/// (`work/lf/probes/p1.cpp`):
///
/// ```text
///   void s_a (S* s, int v)       { s->a  = v; }   90830000  stw  r4,0(r3)
///   void s_b (S* s, int v)       { s->b  = v; }   90830004  stw  r4,4(r3)
///   void s_p (S* s, void* v)     { s->p  = v; }   90830008  stw  r4,8(r3)
///   void s_c (S* s, char v)      { s->c  = v; }   9883000c  stb  r4,12(r3)
///   void s_sh(S* s, short v)     { s->s  = v; }   b083000e  sth  r4,14(r3)
///   void s_q (S* s, long long v) { s->q  = v; }   f8830020  std  r4,32(r3)
///   void s_e2(S* s, int v)       { s->arr[2] = v; } 90830030  stw  r4,48(r3)
///   void s_k (S* s)              { s->a  = 7; }   39600007 91630000  li r11,7 ; stw r11,0(r3)
///   void s_arg2(int x,S* s,int v){ s->b  = v; }   90a40004  stw  r5,4(r4)  <- ANY two regs
///   void D::sb1(int v)           { b1 = v; }      90830004  stw  r4,4(r3)  <- 2117, 0+4
/// ```
///
/// and **no `.pdata` entry**: the body is a leaf, exactly like the load and
/// address leaves beside it.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **The value must be a GPR-class scalar** ([`store_value_width`]). A `float`
///   or `double` member is `stfs`/`stfd` from the FP file and the FP argument
///   number is not the parameter index.
/// * **No conversion on the value.** `void M::setb(bool v){ m0 = v; }` (an `int`
///   member, a `bool` parameter) carries a `2C 86 41 74 00` and emits
///   `548b063e ; 91630000` — `clrlwi r11,r4,24 ; stw r11,0(r3)` — a real mask
///   through the scratch register. The production admits a `2C` only on the
///   *address*, pointer→pointer, where it is free.
/// * **The stored TYPE must restate the value's `<tag><kind>`.** They are
///   byte-identical at every captured site, and requiring it is what makes a
///   misaligned read fail closed instead of picking a plausible width.
/// * **`K` must fit a signed 16-bit displacement**, and a `width == 8` store's
///   `K` must be a multiple of 4 (`std` is DS-form and cannot encode the low two
///   bits) — the same two bounds the load leaf draws.
/// * **Both the base and the value must be register arguments** (`params`
///   position < 8): past the eighth they are stack-homed, which needs a frame.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_store_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
    let params = parse_params(seg, lo).ok()?;
    let mut p = start;
    let st = parse_store_stmt(seg, &mut p, lo, sy, &params)?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;
    Some(BodyShape::StoreLeaf { params, ops: st.ops })
}

/// One store **statement**, from its designator through its `4B` — the unit
/// [`try_parse_store_leaf`] admits exactly one of and [`try_parse_store_run`]
/// admits a run of.
///
/// Extracted rather than copied: every gate the single store was byte-graded on
/// (the value's width class, the FP split, the `2C` rules, the displacement
/// bound, the DS-form alignment, the argument-register positions) is stated
/// **here, once**, so a run cannot admit a statement the leaf refuses.
/// `GAPS.md` §6's "one fact, one locator" in the form that costs coverage:
/// the store family already had three consumers of one designator, and a fourth
/// copy of the *statement* would have been the same mistake one layer out.
pub(crate) struct StoreStmt {
    /// The op group this statement contributes: `[Load(base), value,
    /// StoreInd]` for a GPR value, `[Load(base), StoreIndFp]` for an FP one.
    pub(crate) ops: Vec<IlOp>,
    /// The base object token and the byte range written, for the run's
    /// **overlap** gate. See [`try_parse_store_run`].
    pub(crate) base_tok: u32,
    pub(crate) off: i32,
    pub(crate) width: u8,
    /// True when the stored value is a literal. A run of more than one used to
    /// refuse this outright; since `w-wire` (board #642) it admits up to three
    /// distinct literals through one base symbol, and the emitted permutation
    /// is `c2_core::codegen::order`'s. See [`try_parse_store_run`].
    pub(crate) value_is_lit: bool,
    /// True when the value comes out of the FLOATING-POINT register file. A run
    /// that mixes the two files is scheduled rather than emitted in source
    /// order — see [`try_parse_store_run`].
    pub(crate) value_is_fp: bool,
    /// True when the value is an **indirect load** (`d->a = s->b;`) rather than a
    /// formal or a literal. A run may not mix the two kinds — MEASURED, see
    /// [`try_parse_store_run`].
    pub(crate) value_is_load: bool,
    /// **F2** — true when the value is a sub-object's **ADDRESS** (`s->p =
    /// &s->m;`), which materialises through the scratch register as one `addi`
    /// and is therefore a *producer* exactly as a literal is. See
    /// [`parse_addr_value`].
    pub(crate) value_is_addr: bool,
    /// The **source** object token of a loaded value, for the run's aliasing gate.
    /// See [`try_parse_store_run`].
    pub(crate) src_tok: Option<u32>,
}

/// The **VALUE as an indirect load** — `d->a = s->b;`, the body of every
/// hand-written copy constructor and copy assignment operator.
///
/// ```text
///   <designator>                   the source object pointer, the same two
///   ( 33 <int-like> k 27 <PTR>     spellings and the same shared offset-add run
///   | 33 <int-like> k 28 00 00 )*  the destination designator above uses
///   30 <TYPE>                      THE LOAD — and no `41`, which is the return
/// ```                              production's, not this one's
///
/// **Two instructions, one scratch register, no frame.** MEASURED — every word
/// read off the reference obj (`work/wsl/probe/p1.cpp`, `p2.cpp`, `p4.cpp` at
/// `/O1 /GS- /c`):
///
/// ```text
///   void c1 (S* d, Q* s) { d->a = s->qb; }   81640004 91630000  lwz r11,4(r4) ; stw r11,0(r3)
///   void c1s(S* d)       { d->a = d->b;  }   81630004 91630000  ONE base register
///   void c1d(int* d, int* s) { *d = *s;  }   81640000 91630000  the bare deref
///   void w_c(W* d, W* s) { d->c = s->c;  }   89640000 99630000  lbz ; stb
///   void w_h(W* d, W* s) { d->h = s->h;  }   a1640002 b1630002  lhz ; sth
///   void w_q(W* d, W* s) { d->q = s->q;  }   e9640008 f9630008  ld  ; std   (both DS-form)
///   void w_f(W* d, W* s) { d->f = s->f;  }   c0040010 d0030010  lfs f0 ; stfs f0
///   void w_g(W* d, W* s) { d->g = s->g;  }   c8040018 d8030018  lfd f0 ; stfd f0
///   void n1 (N* d, N* s) { d->m0 = s->in.y; } 81640008 91630000 the offset RUN folds
///   void bm1(D* d, D* s) { d->d0 = s->b1; }  8164000c 91630010  the 2117 designator
/// ```
///
/// The value goes through the **scratch** register in both files — `r11` and
/// `f0`, never `r3`/`f1` — which is the same r11 rule
/// [`super::leaf_load`] records for a load feeding an extension, and it is read
/// off the capture rather than assumed.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **No conversion on the loaded value.** `d->i = s->c` (a `char` member into
///   an `int` one) carries a `2C 86 41 74 00` and emits
///   `lbz r11,0(r4) ; extsb r11,r11 ; stw r11,4(r3)` — a real instruction in
///   between. The narrowing twin `d->c = (char)s->i` *is* free
///   (`lwz r11,4(r4) ; stb r11,0(r3)`), so the asymmetry is c2's own; both are
///   refused, because admitting the free one means deciding the direction from
///   two type triples and this production's whole point is that the two ends
///   agree.
/// * **The stored TYPE must restate the LOADED type**, byte for byte — the same
///   rule the formal-valued path makes, and here it is what pins the load's
///   opcode to the store's.
/// * **The `27`'s announced pointee width and the `30`'s must agree**, through
///   [`eat_offset_adds`] — [`super::leaf_load`]'s rule, reached through the same
///   shared walk rather than restated. Forking that walk is the exact defect W35
///   fixed.
/// * **The source base must be a register argument** (`< 8`) **and not
///   `volatile`**: a `volatile`-qualified pointer *formal* is a memory object
///   that c2 homes in the frame, so the body is not a leaf at all. A pointer *to*
///   volatile is a different bit position and is free — MEASURED, `v_src`/`v_dst`
///   in `work/wsl/probe/p2.cpp` are both the bare `lwz`/`stw` pair.
/// * **`ld` is DS-form**, so a width-8 load's offset must be a multiple of 4 —
///   the same bound the store side draws, now drawn twice because there are two
///   displacements.
///
/// Returns `None` — cursor untouched — for anything else, which is how a plain
/// pointer formal (`B9 <tok> <PTR> 32 <PTR>`) falls through to the ordinary value
/// position: it has no `30`.
fn parse_load_value(
    seg: &[u8],
    cursor: &mut usize,
    params: &[u32],
) -> Option<(StoreStmt, u8, u8)> {
    let mut p = *cursor;
    // The source designator — the same two spellings, tried in the same order,
    // as the destination one above.
    let (mut off, src_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    let (run, last_retype) = eat_offset_adds(seg, &mut p)?;
    off = off.checked_add(run)?;

    // The load.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    p += tw;
    // The width and the register file, through the SAME two locators the
    // formal-valued path asks — so "what is this value" has one answer in this
    // file and not two.
    let (width, is_fp) = match store_fp_value_width(tag, kind) {
        Some(w) => (w, true),
        None => (store_value_width(tag, kind)?, false),
    };
    // The `27` re-types the address and its tag carries the POINTEE width, so it
    // is a second, independent statement of what the `30` announces. Only the
    // LAST one is in a position to say — [`eat_offset_adds`] has the reason.
    if let Some((rt, rk)) = last_retype {
        let announced = if is_ptr_to_4(rt, rk) { 4 } else { sized_ptr_width(rt, rk)? };
        if announced != width {
            return None;
        }
    }
    // **A cv-qualification STRIP, which is the whole reason a copy assignment
    // parses at all.** A copy constructor and a copy assignment take
    // `const T&`, so the loaded member is `const int` and the member it is
    // stored into is plain `int`, and c1xx spells the difference as an explicit
    // `2C` between the two. MEASURED (`work/wsl/probe/p5.cpp`, the IL read off
    // the captured `.ex`):
    //
    // ```text
    //   d->a = s->a   const T* s     30 a6 41 86 20  2c 86 41 74 00  32 86 41 74
    //   d->a = s->a   volatile T* s  30 96 41 8a 20  2c 86 41 74 00  32 86 41 74
    //   d->c = s->c   const T* s     30 a2 11 8c 20  2c 82 11 70 00  32 82 11 70
    //   d->g = s->g   const T* s     30 a8 85 8e 20  2c 88 85 41 00  32 88 85 41
    // ```
    //
    // and it emits **nothing** — `f_const` is the same `lwz r11,0(r4) ;
    // stw r11,0(r3)` as its unqualified twin `f_plain`.
    //
    // **Class-preserving only**, and the gate is on the *kind byte* plus the
    // width, not on the tag: a `2C` here can also be a real widening —
    // `d->i = s->c` carries `2C 86 41 74 00` over a `30 82 11 70` and pays an
    // `extsb` between the two instructions — and the two are told apart by the
    // kind, which is the type's class-and-signedness and does not move under a
    // cv strip. Requiring it equal refuses every widening, including the
    // *free* unsigned ones, for the reason [`super::leaf_load`] gives about
    // deciding a direction from two type triples.
    let (mut vtag, mut vkind) = (tag, kind);
    if seg.get(p) == Some(&0x2C) {
        let mut probe = p + 1;
        let (t2, k2, _, tw2) = read_type(seg, probe)?;
        probe += tw2;
        if !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        let w2 = match store_fp_value_width(t2, k2) {
            Some(w) => (w, true),
            None => (store_value_width(t2, k2)?, false),
        };
        if w2 != (width, is_fp) || k2 != kind {
            return None;
        }
        vtag = t2;
        vkind = k2;
        p = probe;
    }
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `ld` is DS-form, exactly as `std` is on the store side.
    if width == 8 && !is_fp && off % 4 != 0 {
        return None;
    }
    let six = params.iter().position(|&t| t == src_tok)?;
    if six >= 8 {
        return None;
    }
    *cursor = p;
    Some((StoreStmt {
        // Only the value half; the caller prepends the destination's `Load` and
        // appends the store.
        ops: vec![
            IlOp::Load(src_tok),
            if is_fp {
                IlOp::LoadIndFp { off, double: width == 8 }
            } else if width == 4 {
                IlOp::LoadInd { off }
            } else {
                IlOp::LoadIndSized { off, width, sext: false }
            },
        ],
        base_tok: 0,
        off,
        width,
        value_is_lit: false,
        value_is_fp: is_fp,
        value_is_load: true,
        value_is_addr: false,
        src_tok: Some(src_tok),
    }, vtag, vkind))
}

/// The tail of a statement whose value is an [indirect load](parse_load_value):
/// the `32` store, its type restatement, the `4B`, and the destination's own two
/// bounds.
///
/// Split out for the reason [`super::leaf_load::finish_indirect_load`] is: the
/// value production and the store production are two decoders that must agree on
/// one width, and threading that agreement through a nest of `if let`s is where a
/// bound goes missing. Everything it checks is stated once, here, and restated in
/// `store_leaf_text` — the census/gate invariant.
fn finish_load_store_stmt(
    seg: &[u8],
    cursor: &mut usize,
    params: &[u32],
    base_tok: u32,
    off: i32,
    (mut st, ltag, lkind): (StoreStmt, u8, u8),
) -> Option<StoreStmt> {
    let mut p = *cursor;
    // The store, whose TYPE restates the LOADED value's — byte for byte, the same
    // requirement the formal path makes, and here it is what pins the load's
    // opcode to the store's.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (ltag, lkind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // The DESTINATION's displacement bound and DS-form alignment. Its source-side
    // twins were drawn in [`parse_load_value`]; both ends of a copy have their own
    // displacement and there is no reason for them to be equal.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    if st.width == 8 && !st.value_is_fp && off % 4 != 0 {
        return None;
    }
    let bix = params.iter().position(|&t| t == base_tok)?;
    if bix >= 8 {
        return None;
    }
    st.ops.insert(0, IlOp::Load(base_tok));
    st.ops.push(if st.value_is_fp {
        IlOp::StoreIndFp { off, double: st.width == 8, src: FP_SCRATCH }
    } else {
        IlOp::StoreInd { off, width: st.width }
    });
    st.base_tok = base_tok;
    st.off = off;
    *cursor = p;
    Some(st)
}

/// The FP scratch register a loaded floating-point value lands in — `f0`, never
/// `f1`. MEASURED: `lfs f0,16(r4) ; stfs f0,16(r3)`.
pub const FP_SCRATCH: u8 = 0;

fn parse_store_stmt(
    seg: &[u8],
    cursor: &mut usize,
    lo: usize,
    sy: SyView,
    params: &[u32],
) -> Option<StoreStmt> {
    let mut p = *cursor;
    // The designator. The intrinsic form is anchored on a `33` literal and the
    // plain form on a `B9`, so the two cannot be confused; the intrinsic is tried
    // first for the same reason the load and address leaves try it first.
    let (mut off, base_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            // A pointer *value* in a register: the `B9` operand position, where
            // the tag carries the pointer's own width.
            // …and NOT `volatile`. A volatile pointer formal is a memory
            // object: c2 homes it in the frame and reloads it, so this leaf is a
            // whole framed body. See `readers::is_volatile_tag` — the thirteenth
            // live wrong-bytes emit, and the position is load-bearing (the same
            // bit at the `27`/`30` designator positions is free).
            if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    off = off.checked_add(eat_addr_offset_adds(seg, &mut p)?)?;

    // A cv strip or an array-to-pointer decay applied to the ADDRESS, which emits
    // nothing (`void f(S* s, int v){ *(int*)s = v; }` is a bare `stw r4,0(r3)`).
    // Pointer→pointer only: a cross-class `2C` here is a reinterpret this port has
    // never probed.
    if seg.get(p)? == &0x2C {
        let mut probe = p + 1;
        let (tag, kind, _, tw) = read_type(seg, probe)?;
        if !is_ptr_any(tag, kind) {
            return None;
        }
        probe += tw;
        if !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    // THE VALUE AS AN INDIRECT LOAD — `d->a = s->b;`, tried FIRST because both
    // spellings of a source designator (`B9 <tok> <PTR>` and the 2117 `33 …`) are
    // prefixes of the formal/literal value's own two spellings, and only the `30`
    // separates them. On failure the cursor is untouched and the formal/literal
    // path below sees exactly what it always saw.
    {
        let mut probe = p;
        if let Some(st) = parse_load_value(seg, &mut probe, params)
            .and_then(|v| finish_load_store_stmt(seg, &mut probe, params, base_tok, off, v))
        {
            *cursor = probe;
            return Some(st);
        }
    }

    // **F2 — THE VALUE AS A SUB-OBJECT ADDRESS** (`s->p = &s->m;`), tried after
    // the indirect load because the two share their whole prefix and are told
    // apart by the `30` that only the load has. On failure the cursor is
    // untouched and the formal/literal path below sees exactly what it always
    // saw — a bare pointer formal (`B9 <tok> <PTR> 32 <PTR>`) reaches this
    // production, is admitted by it with a zero offset run, and would then emit
    // an `addi rD,rBase,0` that c2 does not, so the zero-length run is
    // **excluded here** and left to the formal path that was byte-graded on it.
    {
        let mut probe = p;
        if let Some(st) = parse_addr_value(seg, &mut probe, params)
            .filter(|st| st.off != 0)
            .and_then(|v| finish_addr_store_stmt(seg, &mut probe, params, base_tok, off, v))
        {
            *cursor = probe;
            return Some(st);
        }
    }

    // THE VALUE — a bare formal or an integer literal, and nothing computed. A
    // computed value lands in the scratch register first (`s->m = a + b` is
    // `add r11,r3,r4 ; stw r11`), which is a different instruction count and has
    // no capture behind it here.
    let (value_op, mut value_tag, mut value_kind) = match *seg.get(p)? {
        0xB9 => {
            let mut probe = p + 1;
            let (tok, w) = read_token_var(seg, probe)?;
            probe += w;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            p = probe;
            (IlOp::Load(tok), tag, kind)
        }
        0x33 => {
            let mut probe = p + 1;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            let k = read_varint(seg, &mut probe)?;
            p = probe;
            (IlOp::Lit(k), tag, kind)
        }
        _ => return None,
    };
    // **A `volatile` VALUE is a memory object, and reading it is a memory
    // access.** `void f(Q* s, volatile int v){ s->a = v; }` is
    // `stw r4,28(r1) ; lwz r11,28(r1) ; stw r11,0(r3)` — c2 homes the parameter
    // in the frame and reloads it, so the body is not a leaf at all — where this
    // production emitted the bare `stw r4,0(r3)`. **Live on mainline for as long
    // as the store leaf has existed** (`Port=Mismatch @ 8`, the whole obj a frame
    // short), and it is `readers::is_volatile_tag` at a THIRD position: `GAPS.md`
    // §6's thirteenth instance put the gate on the base LOAD, W35 measured that
    // the same bit at the `27`/`30` designator positions is free, and nobody
    // asked about the VALUE. Found by `scripts/sweep.d/82-store-run.py`'s
    // cv-qualification axis, which varies a qualifier that changes no operator
    // and no shape.
    if is_volatile_tag(value_tag) {
        return None;
    }
    // A **floating-point** stored value is `stfs`/`stfd` out of the FP argument
    // file, so it takes the whole rest of this production down a parallel path:
    // its register is not the formal's index, and the `2C` rules below are the
    // GPR classes'. MEASURED (`docs/CODEGEN_FP_ARGS.md` §3):
    //
    //     void s_f (S* s, float v)      { s->f = v; }        d0230004  stfs f1,4(r3)
    //     void s_d (S* s, double v)     { s->d = v; }        d8230008  stfd f1,8(r3)
    //     void s_two(S* s,float u,float v){ s->f = v; }      d0430004  stfs f2,4(r3)
    //
    // Sized before it was built, by counterfactual over the 878-TU workload:
    // **7,984 functions**, all `calls-0`.
    let fp_width = store_fp_value_width(value_tag, value_kind);
    if let Some(w) = fp_width {
        let st =
            finish_fp_store_stmt(seg, p, lo, base_tok, value_op, value_tag, value_kind, off, w, sy, params)?;
        *cursor = st.1;
        return Some(st.0);
    }
    let width = store_value_width(value_tag, value_kind)?;

    // A class-preserving conversion of the VALUE — `void f(S* s, S* v){ s->p = v; }`
    // converts `S*` to `void*` on the way in and emits nothing (`90830008`, the same
    // bare `stw` as the unconverted neighbour). Admitted only in the two 4-byte
    // classes [`eat_value_type`] was byte-graded on since the getter rungs, and
    // **only** there: over a narrow value a `2C` is a real instruction —
    // `void M::setb(bool v){ m0 = v; }` (an `int` member, a `bool` parameter) emits
    // `clrlwi r11,r4,24 ; stw r11,0(r3)` — so `width != 4` refuses rather than
    // silently dropping the mask.
    if seg.get(p) == Some(&0x2C) {
        let cls = value_class(value_tag, value_kind)?;
        let mut probe = p + 1;
        let (t2, k2, _, _) = read_type(seg, probe)?;
        if !eat_value_type(seg, &mut probe, cls) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        value_tag = t2;
        value_kind = k2;
        p = probe;
    }

    // The store, whose TYPE restates the value's.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    // The statement end. A store yields its value and `4B` discards it; a body
    // that goes on to use it is not this shape.
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `std` is DS-form: the displacement's low two bits are the form's, so an
    // offset that is not a multiple of 4 cannot be encoded at all. Natural
    // alignment makes one unreachable through a struct member, so this gate has
    // no witness — which is exactly why it refuses instead of masking.
    if width == 8 && off % 4 != 0 {
        return None;
    }
    let bix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if bix >= 8 {
        return None;
    }
    match value_op {
        IlOp::Load(vtok) => {
            let vix = params.iter().position(|&t| t == vtok)?;
            // Past the eighth argument the value is stack-homed, which needs a frame.
            if vix >= 8 {
                return None;
            }
        }
        // A wide **negative** constant. `emit_load_imm`'s `lis`+`ori` pair covers
        // non-negative values only, and the straight-line class already refuses
        // this in the PARSER (`expr-out-of-class-wide-neg-lit`,
        // `chain::straight_line_out_of_class_ctx`). Restating the bound here rather
        // than letting codegen refuse it is the census/gate invariant: the same
        // literal reached two shapes and only one of them gated it, so
        // `void f(S* s){ s->a = -70000; }` censused in class while `PortC2`
        // returned `NotImplemented` — the `GAPS.md` §6 "one fact, two locators"
        // failure, caught by probing the new production's own boundary.
        IlOp::Lit(k) if k < -0x8000 => return None,
        _ => {}
    }
    *cursor = p;
    Some(StoreStmt {
        ops: vec![IlOp::Load(base_tok), value_op, IlOp::StoreInd { off, width }],
        base_tok,
        off,
        width,
        value_is_lit: matches!(value_op, IlOp::Lit(_)),
        value_is_fp: false,
        value_is_load: false,
        value_is_addr: false,
        src_tok: None,
    })
}

/// **F2 — a sub-object's ADDRESS as the stored value**: `s->p = &s->m;`,
/// `listHead.mNext = &listHead;`, and — the reason this rung exists — the
/// reference bind `auto& lh = mListHead;`, which c1xx spells as a store of an
/// address into a local.
///
/// ```text
///   <designator>                    the VALUE's object pointer, the same two
///   ( 33 <int-like> k 27 <PTR>      spellings and the same shared offset-add run
///   | 33 <int-like> k 28 00 00 )*   every other consumer of `designator.rs` uses
///                                   — and NO `30`, which is what separates this
///                                   from [`parse_load_value`]
/// ```
///
/// **The `30` is the whole discriminator**, and this production is tried *after*
/// [`parse_load_value`] for exactly that reason: the two share their entire
/// prefix and differ only by whether the address is dereferenced. An address that
/// is loaded is a value in memory (`lwz r11 ; stw r11`); an address that is not
/// is one `addi r11,rBase,off` — different instructions, so admitting one as the
/// other would be a wrong-bytes emit rather than a gap, which is the same reason
/// [`BodyShape::AddrLeaf`] is kept apart from [`BodyShape::IndirectLoad`].
///
/// Read off `src/xdk/nuispeech/xboxheap.cpp`'s own `.ex` at the workload's
/// `/GR /O1 /Oi /EHsc` (`work/w-f23/il/xboxheap/`), source line 8
/// (`auto& listHead = mListHead;`):
///
/// ```text
///   26 11 0a                          the destination — a LOCAL reference variable
///   b9 0f 0a a6 43 81 20              `this`
///   33 86 41 74 08 27 a6 43 98 20     + 8                <- the address, no `30`
///   32 86 43 9b 20 4b                 stored, discarded
/// ```
///
/// and the emitted producer is `addi r11,r3,8`, whose value is then *stored*
/// twice (`stw r11,8(r3) ; stw r11,12(r3)`) — an interior address behaves as a
/// **producer**, exactly as a literal's `li` does, which is why
/// [`StoreStmt::value_is_addr`] is carried beside `value_is_lit` and not folded
/// into it.
///
/// What is REFUSED, each fail-closed rather than guessed:
///
/// * **A value base that is not one of this function's own formals**, or one past
///   the eighth. `xboxheap`'s own bind stores *into* a local and the two stores
///   that follow it read *from* one; a local in either position is board #839's
///   reference-bind obligation and is a **separate** refusal this production does
///   not pay (`docs/rungs/2026-08-08-w-f23.md` §4).
/// * **An offset outside a signed 16-bit displacement.** `&s->b` at 32768 is
///   `addis r3,r3,1 ; addi r3,r3,-32768` — two instructions — so the same bound
///   [`IlOp::AddrOf`] already carries is restated here rather than left to
///   codegen, which is this file's census/gate invariant.
/// * **A stored TYPE that is not a pointer**, and a `2C` conversion on the
///   address. The address production above admits a pointer→pointer `2C` on its
///   *own* base; a conversion between the address and the `32` has no capture.
///
/// Returns `None` — cursor untouched — for anything else, so a body that is not
/// this shape still reports its own blocking feature.
fn parse_addr_value(seg: &[u8], cursor: &mut usize, params: &[u32]) -> Option<StoreStmt> {
    let mut p = *cursor;
    // The value's designator — the same two spellings, in the same order, as
    // every other consumer of this module.
    let (mut off, vbase_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            // The same gate the destination designator makes, for the same
            // reason: a `volatile` pointer *formal* is a memory object c2 homes
            // in the frame, so the body is not a leaf at all.
            if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    off = off.checked_add(eat_addr_offset_adds(seg, &mut p)?)?;
    // **No `30`.** A dereference here is [`parse_load_value`]'s production and it
    // has already been tried; reaching this byte means the caller's cursor is
    // about to be handed to the `32`, so a `30` is a refusal and not a
    // fall-through — leaving it out would let a two-instruction load-and-store
    // parse as a one-instruction address-and-store.
    if seg.get(p) == Some(&0x30) {
        return None;
    }
    // `addi` cannot encode more than a signed 16-bit displacement in one word.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    let vix = params.iter().position(|&t| t == vbase_tok)?;
    // Past the eighth argument the base is stack-homed, which needs a frame.
    if vix >= 8 {
        return None;
    }
    *cursor = p;
    Some(StoreStmt {
        // Only the value half; the caller prepends the destination's `Load`
        // and appends the store — the same contract [`parse_load_value`] has.
        ops: vec![IlOp::Load(vbase_tok), IlOp::AddrOf { off }],
        base_tok: 0,
        off,
        // A pointer is 4 bytes on this target, and the `32`'s own type is
        // required to restate it in [`finish_addr_store_stmt`].
        width: 4,
        value_is_lit: false,
        value_is_fp: false,
        value_is_load: false,
        value_is_addr: true,
        src_tok: None,
    })
}

/// The tail of a statement whose value is an [address](parse_addr_value): the
/// `32` store, its pointer type, the `4B`, and the destination's own two bounds.
///
/// Split out for the reason [`finish_load_store_stmt`] is: the value production
/// and the store production are two decoders that must agree on one width, and
/// threading that agreement through a nest of `if let`s is where a bound goes
/// missing.
fn finish_addr_store_stmt(
    seg: &[u8],
    cursor: &mut usize,
    params: &[u32],
    base_tok: u32,
    off: i32,
    mut st: StoreStmt,
) -> Option<StoreStmt> {
    let mut p = *cursor;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    // **The stored type must be a POINTER**, positively — the value is an
    // address and `stw` is the only store this shape emits. A narrower or
    // wider `32` here would mean the address is being truncated or widened,
    // which is a real instruction and has no capture.
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if value_class(tag, kind) != Some(crate::func::readers::ValueClass::Ptr4) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // The DESTINATION's own displacement bound and its base's register position.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    let bix = params.iter().position(|&t| t == base_tok)?;
    if bix >= 8 {
        return None;
    }
    st.ops.insert(0, IlOp::Load(base_tok));
    st.ops.push(IlOp::StoreInd { off, width: 4 });
    st.base_tok = base_tok;
    st.off = off;
    st.width = 4;
    *cursor = p;
    Some(st)
}

/// The tail of [`parse_store_stmt`] for a **floating-point** stored value.
///
/// Split out rather than branched inline because almost every gate differs: the
/// value's register comes from the FP file, the conversion rules are the FP ones,
/// and a literal is a pooled `.rdata` COMDAT rather than an `li`.
///
/// What is REFUSED here, each because a capture shows it emits something else:
///
/// * **A conversion on the value.** `void s_narrow(S* s, double v){ s->f = v; }`
///   is `frsp f0,f1 ; stfs f0,4(r3)` — a real instruction through the FP scratch
///   register. Its free twin `void s_widen(S* s, float v){ s->d = v; }` is a bare
///   `stfd f1,8(r3)`, so the asymmetry is c2's own and not the C standard's; both
///   are refused, because admitting the free one means deciding the direction from
///   two type triples and only the narrowing one has been captured at more than
///   one offset. A rung, sized in `docs/CODEGEN_FP_ARGS.md` §5.
/// * **A literal value.** `void s_lit(S* s){ s->f = 1.5f; }` is
///   `lis r11 ; lfs f0,0(r11) ; stfs f0,4(r3)` with a REFHI/REFLO pair into an
///   `.rdata` COMDAT — the W13b constant machinery, which `codegen::function_gate`
///   refuses under `/Gy` anyway.
/// * **A value that is not a formal**, and a formal whose FP register the `.sy`
///   argument classes cannot determine.
#[allow(clippy::too_many_arguments)]
fn finish_fp_store_stmt(
    seg: &[u8],
    mut p: usize,
    lo: usize,
    base_tok: u32,
    value_op: IlOp,
    value_tag: u8,
    value_kind: u8,
    off: i32,
    width: u8,
    sy: SyView,
    params: &[u32],
) -> Option<(StoreStmt, usize)> {
    // No conversion, and no pooled constant.
    if seg.get(p) == Some(&0x2C) {
        return None;
    }
    let IlOp::Load(vtok) = value_op else {
        return None;
    };
    // The store, whose TYPE restates the value's — the same literal requirement
    // the GPR path makes, and for the same reason.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `stfs`/`stfd` are both plain D-form — unlike `std`, which is DS-form and
    // cannot encode a displacement that is not a multiple of 4. So there is no
    // alignment gate here, and the absence is a measured difference between the
    // two paths rather than an omission (`d8230008` is `stfd f1,8(r3)`, primary
    // 54, with all sixteen displacement bits its own).
    let bix = params.iter().position(|&t| t == base_tok)?;
    if bix >= 8 {
        return None;
    }
    // The value's FP register, resolved HERE — the one site that knows both the
    // formals order (`.ex`) and each formal's register file (`.sy`).
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    let fix = formals.iter().position(|&t| t == vtok)?;
    let src = fp_reg_of(&classes, fix)?;
    if src > 13 {
        // Past f13 the argument is stack-homed, which needs a frame.
        return None;
    }
    // The value's declared width and the stored width must be the same fact. They
    // are, at every capture, because a conversion is a visible `2C` that is
    // refused above — so a disagreement means a misread type, not a construct.
    if matches!(classes.get(fix), Some(ArgClass::Fp { double }) if *double != (width == 8)) {
        return None;
    }
    Some((
        StoreStmt {
            ops: vec![
                IlOp::Load(base_tok),
                IlOp::StoreIndFp { off, double: width == 8, src },
            ],
            base_tok,
            off,
            width,
            value_is_lit: false,
            value_is_fp: true,
            value_is_load: false,
            value_is_addr: false,
            src_tok: None,
        },
        p,
    ))
}

/// A returned value that is the **first formal** — `return s;` in
/// `S* g(S* s,int u){ s->a=u; return s; }`, and `return *this;` in a member,
/// where `this` is the first formal. Spelled `B9 <tok> <TYPE> 41 <same TYPE>`,
/// the ordinary value-return head.
///
/// **It costs nothing, and only at index 0.** MEASURED (`work/w37/probe/p6.cpp`,
/// four neighbours in one TU):
///
/// ```text
///   S*  r_first (S* s,int u,int v) { s->a=u; s->b=v; return s; }  90830000 90a30004         blr
///   int r_first_i(int u,S* s,int v){ s->a=v;         return u; }  90a40000                  blr
///   S*  r_second(int k,S* s,int u) { s->a=u;         return s; }  90a40000 7c832378  mr r3,r4
///   int r_formal(S* s,int u,int v) { s->a=u;s->b=v;  return u; }  mr r11,r3 ; … ; mr r3,r4
/// ```
///
/// The first two are a bare `blr` because the value is already in r3; the third
/// is one register move; and the fourth is worse than a move — the result
/// register displaces the base, so c2 saves it into r11 and **re-bases the later
/// stores**. So the gate is on the formal's *position*, which is the same fact
/// [`crate::func::body::chain::straight_line_out_of_class_ctx`] states for a bare
/// `return <formal>` in the integer class, and it refuses rather than emitting a
/// move this production has no capture for.
///
/// The LOAD, an optional class-preserving `2C`, and the `41` result must all
/// name the **same [`crate::func::readers::ValueClass`]** — reached through
/// [`eat_value_type`], the one locator the getter rungs byte-graded a free
/// conversion on. Byte-identity would have been tighter and is wrong here: `this`
/// is `T* const` and loads as `A6 43`, while `T* pset2(…){ …; return this; }`
/// annotates its result `86 43`, and that qualification conversion emits nothing
/// (`?ptr2@T@@QAAPAU1@HH@Z` is `90830000 90a30004 4e800020`, the same three words
/// as its `T&` twin). A CROSS-class one still refuses.
fn eat_first_formal_result(seg: &[u8], p: &mut usize, params: &[u32]) -> bool {
    let Some(&first) = params.first() else {
        return false;
    };
    let mut q = *p;
    if seg.get(q) != Some(&0xB9) {
        return false;
    }
    q += 1;
    let Some((tok, w)) = read_token_var(seg, q) else {
        return false;
    };
    if tok != first {
        return false;
    }
    q += w;
    let Some((tag, kind, _, tw)) = read_type(seg, q) else {
        return false;
    };
    let Some(cls) = value_class(tag, kind) else {
        return false;
    };
    q += tw;
    if seg.get(q) == Some(&0x2C) {
        let mut probe = q + 1;
        if !(eat_value_type(seg, &mut probe, cls) && eat_byte(seg, &mut probe, 0x00)) {
            return false;
        }
        q = probe;
    }
    if seg.get(q) != Some(&0x41) {
        return false;
    }
    q += 1;
    if !eat_value_type(seg, &mut q, cls) {
        return false;
    }
    *p = q;
    true
}

/// Try to parse a **store run**: a whole body that is a *sequence* of the store
/// statements [`parse_store_stmt`] admits, ending either on the ordinary void
/// return plumbing or on a `return *this` / `return this`.
///
/// ```text
///   ( <scopes / line markers> <store statement> )+
///   <return plumbing, void>            OR   <return head> <return this> <fn tail>
/// ```
///
/// **The store leaf's own limit was "and the body ends here", and it is exactly
/// the shape `GAPS.md` §6 calls the coverage-costing form of "one fact, one
/// locator".** The *assignment* statement parser beside it
/// ([`super::assign::try_parse_assign_body_detail`]) has had a statement list
/// since it was written; the store leaf, built later on the same shared
/// designator, never got one. Nothing was wrong — a recognizer that refuses more
/// than its sibling emits nothing, so no byte compare and no disagreement check
/// can see it. On the 878-TU dc3 workload the missing statement list and the
/// missing `return this` tail were worth **54,433 whole bodies, every one
/// `calls-0`**, measured by counterfactual before any of this was written.
///
/// **The lowering is one store per statement, in source order** — no scheduling,
/// no reordering, no coalescing. MEASURED, every word read off the reference obj
/// (`work/w37/probe/p1.cpp`, `p2.cpp`, `p4.cpp` at `/O1` and `/Ox`, which agree):
///
/// ```text
///   void s2 (S* s,int u,int v)      { s->a=u; s->b=v; }   90830000 90a30004
///   void s2r(S* s,int u,int v)      { s->b=v; s->a=u; }   90a30004 90830000  <- SOURCE order
///   void s2s(S* s,int v)            { s->a=v; s->b=v; }   90830000 90830004  <- one formal twice
///   void s2t(S* t,S* s,int u,int v) { t->a=u; s->b=v; }   90a30000 90c40004  <- two bases
///   void W3 (S* s,char h,short i,long long j)             9883001c b0a3001e f8c30020
///   void Fp2(S* s,float f,double d) { s->f=f; s->d=d; }   d0230010 d8430018  <- the FP file
///   T& set2(int u,int v) { a=u; b=v; return *this; }      90830000 90a30004  <- the epilogue is FREE
///   void F7(S*,int×7)  — seven stores, r4..r10, all one `stw` each
/// ```
///
/// Both bullets below are *captured* neighbours that emit something else, and
/// each is the reason the gate is not conservatism. **The first is no longer a
/// refusal** — `w-wire` (board #642) replaced it with a model:
///
/// * **A LITERAL value, in a run of more than one — MODELLED, not refused.**
///   Everything this bullet used to call unpredictable now has a holdout behind
///   it. c2 hoists the `li`s out of the store sequence, allocates them
///   r11/r10/r9 **descending** (`c2_core::codegen::alloc`, 250/250 holdout),
///   common-subexpresses equal literals, interleaves them with the stores
///   (`layout_slots`, 24,891/24,891) and — decisively — **reorders the stores
///   themselves** (`store_order`, 561/561): `{ s->a=1; s->b=u; }` is
///   `li r11,1 ; stw r4,4(r3) ; stw r11,0(r3)`, the two statements emitted in
///   the *opposite* order to the source. That is ORDER's `"0."` → `P0 S1 S0`,
///   and `c2_core::codegen::leaf::store::scheduled_gpr_run_text` emits it. The
///   gate in the body keeps the widening inside the region the models are
///   **exact** on: one base symbol, ≤ 3 distinct literals, a pool that holds
///   them, and no multi-word literal beside another producer.
/// * **Two statements writing overlapping bytes of the SAME base token.** c2
///   eliminates the dead one: `{ s->a=u; s->a=w; }` is a *single*
///   `stw r5,0(r3)`. Emitting both would be wrong bytes, and the gate is on the
///   byte RANGE rather than on the offset so that a packed/union overlap refuses
///   too. Two *different* base tokens may alias at run time and c2 keeps both
///   stores (`void s2t(S* t,S* s,…)` above), so the gate is deliberately keyed on
///   the token and not on "some pointer".
///
/// The tail that is **refused** is `return <formal>`: `int iset2(int u,int v)
/// { a=u; b=v; return u; }` is `mr r11,r3 ; stw r4,0(r3) ; mr r3,r4 ;
/// stw r5,4(r11)` — the result register displaces `this`, so the stores are
/// re-based and the body is no longer a plain sequence. `return *this` is free
/// for the opposite reason: `this` is already in r3 and a store writes no
/// register, so the epilogue is the same no-op
/// [`super::ctor_dtor::eat_ctor_this_epilogue`] measured for the empty
/// constructor. That recognizer had exactly **one** consumer before this rung —
/// the empty-body arm — which is the same "a locator nobody consults is not
/// shared" reading as the paragraph above, and it is worth 42,238 of the 54,433.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_store_run(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
    depth0: usize,
) -> Option<BodyShape> {
    let (stmts, params, p, depth) = collect_store_run(seg, start, lo, sy, depth0)?;
    run_tail(seg, p, lo, depth, stmts, params)
}

/// **The run and every gate on it, without the tail** — the statement loop and
/// the whole gate battery [`try_parse_store_run`]'s doc comment describes, split
/// out at exactly the point the *tail* begins.
///
/// Two productions end a run and there must be **one** statement of what a run
/// is: [`try_parse_store_run`] ends it on the void plumbing or on
/// `return *this`, and [`try_parse_store_run_call`] ends it on a call. A second
/// copy of the loop is where the overlap gate, the mixed-file gate, the pool
/// floor and the widened-literal clauses drift apart from each other — `GAPS.md`
/// §6's "one fact, one locator" in the form that costs coverage, which this
/// file's own header already records happening once to the *statement*.
///
/// Returns the statements, the formals, the cursor **after the last store** and
/// the lexical depth the walk reached — the four things a tail needs and nothing
/// else.
fn collect_store_run(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
    depth0: usize,
) -> Option<(Vec<StoreStmt>, Vec<u32>, usize, usize)> {
    let params = parse_params(seg, lo).ok()?;
    let mut p = start;
    let mut depth = depth0;
    let mut stmts: Vec<StoreStmt> = Vec::new();
    loop {
        let (sp, sd) = (p, depth);
        // Brace scopes and line markers open and close *between* statements, so
        // they are consumed at the boundary — the same rule
        // [`super::assign::try_parse_assign_body_detail`] applies, through the
        // same locator.
        if eat_scopes(seg, &mut p, &mut depth).is_err() {
            p = sp;
            depth = sd;
            break;
        }
        let mut q = p;
        match parse_store_stmt(seg, &mut q, lo, sy, &params) {
            Some(st) => {
                stmts.push(st);
                p = q;
            }
            // The cursor stays where the scopes left it: the tail is what follows
            // the last store, and `eat_return_head` requires the scope closes to
            // descend from the depth the statement walk actually reached.
            None => break,
        }
    }
    if stmts.is_empty() {
        return None;
    }
    if stmts.len() > 1 && stmts.iter().any(|s| s.value_is_lit) {
        // **The ONE-VALUE all-literal run, and nothing else.** See the doc
        // comment's literal bullet: with two or more *distinct* literal values c2
        // hoists the `li`s, allocates r11/r10/r9 by a rule these captures do not
        // determine, and reorders the stores. With exactly one distinct value the
        // whole question disappears — there is one materialization and one
        // register, so there is nothing to allocate and nothing to schedule, and
        // the stores come back in **source order** at every length probed.
        // MEASURED (`work/oneaway` grids 4–6, `/O1` and `/Ox` byte-identical):
        //
        // ```text
        //   { a=9; b=9; c=9; }              li r11,9 ; stw ; stw ; stw
        //   { a=9; …5 stores… }             li r11,9 ; stw x5
        //   { a=7; …6 stores… }             li r11,7 ; stw x6
        //   { a=0; b=0; }                   li r11,0 ; stw ; stw
        //   { c=1; h=1; i=1; }              li r11,1 ; stb ; sth ; stw   MIXED widths
        //   { c=0; h=0; i=0; q=0; }         li r11,0 ; stb ; sth ; stw ; std
        //   { s->a=3; t->b=3; s->c=3; }     li r11,3 ; stw 0(3) ; stw 4(4) ; stw 8(3)
        //   { a=100000; b=100000; }         lis+ori  ; stw ; stw          WIDE literal
        //   { t->r.a=4; t->r.b=4; t->z=4; } li r11,4 ; stw x3             nested offsets
        // ```
        //
        // and the neighbours that fix the boundary, every one of them a *reorder*
        // or a *permuted allocation* this shape would emit wrong:
        //
        // ```text
        //   { a=1; b=2; }                   li r11,1 ; li r10,2 ; stw ; stw
        //   { a=1; b=u; }                   li r11,1 ; stw r4,4 ; stw r11,0   REORDERED
        //   { a=1; b=2; c=3; d=1; }         1->r11 2->r10 3->r9   first-use order
        //   { a=1; b=2; c=3; d=2; e=1; }    1->r10 2->r11 3->r9   PERMUTED
        //   { a=1; b=2; c=1; d=2; }         1->r10 2->r11         PERMUTED
        //   { a=1; b=1; c=2; d=2; e=2; }    2->r11 1->r10, and the stores come
        //                                   back 2,0,1,3,4 — NOT source order
        // ```
        //
        // Four allocation rules were fitted to those and each is refuted by one
        // of the others (use count by `A1`, live-range length by `A2`, last-use
        // by `B6`, first-use by `B4`/`B7`). `GAPS.md` §6 instance #10 — measure
        // at the edge, do not fit the scheduler — so the multi-value run stays
        // refused and only the degenerate one is admitted.
        //
        // **All four are now derived consequences of ONE rule**, and the
        // neighbours above are recomputed from it in
        // `c2_core::codegen::alloc`'s tests: sort the distinct producers by
        // **use count descending**, tie to register-derived before constant,
        // tie within the register-derived by source order and within the
        // constants by *reverse* source order, then hand out r11, r10, r9 …
        // descending. See `docs/ALLOC.md` §1. Every one of the four rules above
        // is a projection of the use count onto a grid where every count
        // happened to be 1 or 2, which is why each fitted its own cells and
        // died on the next.
        //
        // # **The refusal MOVES — lane `w-wire`, board #642**
        //
        // The paragraph that stood here said *"the refusal does NOT move, and
        // that is deliberate. Knowing the register is not knowing the ORDER:
        // the store order when every store of the run is produced is board #544
        // and still open."* **#544 is closed.** `w-order2` settled the store
        // order (561/561 holdout), `w-frame2` settled the LAYOUT (24,891/24,891
        // holdout, board #602), and `w-alloc` settled the register (250/250).
        // `c2_core::codegen::leaf::store::scheduled_gpr_run_text` composes all
        // three and **emits** the permutation instead of refusing it.
        //
        // So a run may now carry **more than one distinct literal**, and may
        // **mix literals with formals** — the two shapes the doc comment's
        // literal bullet lists as reordered. The gate below is what keeps the
        // widening strictly inside the region every model is *exact* on, and
        // every clause of it is syntactic so that the census and the gate
        // cannot disagree (this crate cannot see `c2-core`'s models).
        //
        // **This is additive-ACCEPT and it is stated as such.** The guards in
        // the emitter are additive-*refusal*; this is not, and the two claims
        // are kept apart on purpose. Its safety comes from the gate landing
        // inside a measured-exact region plus five constructed counterexamples
        // (`docs/rungs/_2026-08-05-w-wire-prereg.md` §4), **one of which
        // fired** — see the multi-word clause below.
        //
        // A run whose values are neither literals nor formals stays refused:
        // the FP file and the indirect-load family are separate regimes with
        // their own gates further down.
        if stmts.iter().any(|s| s.value_is_fp || s.value_is_load) {
            return None;
        }
        // **F2's address value IS admitted here, and the four clauses below do
        // NOT come to cover it — they are never consulted.** A sub-object address
        // is a *producer* (one `addi` into the scratch pool), so a run mixing it
        // with a literal is the **mixed-kind** run `alloc::allocate` refuses
        // (board #836) and `w-seam` declined to lift (board #868: 12/12 at the
        // `addi`-interior spelling, **0/12** at `slwi`, and `ProducerKind` cannot
        // tell them apart). `w-heap` §4.1.1 refuted clause 1 on exactly this mix.
        //
        // Admitting it is nonetheless the fail-closed direction, and that is
        // MEASURED rather than argued: an `AddrOf` group is **four** ops where
        // every group `c2_core`'s emitter models is three, so
        //
        //   * `parse_simple_gpr_run`'s `[Load, v, StoreInd, ..]` pattern cannot
        //     match it at all — position 2 is the `AddrOf` — so
        //     `scheduled_gpr_run_text` returns `None` before `order::schedule`
        //     or `alloc::allocate` is asked anything. **The models the clauses
        //     below protect are never reached on such a run.**
        //   * `store_leaf_text`'s own walk has no arm for it either, and its
        //     terminal is `Some(Err(out_of_class("store run with an unmodeled
        //     residue")))` — an honest refusal, not a short emit.
        //
        // Graded on frozen cells at the workload's own flags: `f2_run_addr2`
        // (two addresses, no literal) and `f2_run_addr_lit` (an address beside a
        // literal — the mixed-kind cell) both go `vocab-gap → codegen-gap`, and
        // the 1,576 generated cases of `88-store-run-call.py` stay at 0
        // mismatch. The clauses below still apply to the LITERALS, so they only
        // ever refuse more.

        let lit = |s: &StoreStmt| match s.ops.get(1) {
            Some(IlOp::Lit(k)) => Some(*k),
            _ => None,
        };
        // Every literal statement must actually carry a `Lit` op where the
        // emitter reads one; a `value_is_lit` that does not is a misparse, not
        // a widening.
        if stmts.iter().any(|s| s.value_is_lit && lit(s).is_none()) {
            return None;
        }
        let mut distinct: Vec<i32> = Vec::new();
        for s in &stmts {
            if let Some(k) = lit(s) {
                if !distinct.contains(&k) {
                    distinct.push(k);
                }
            }
        }
        // The pre-existing class: **one** distinct literal and no formal beside
        // it. Admitted through any number of base symbols, exactly as before —
        // `{ s->a=3; t->b=3; s->c=3; }` is a measured cell. Left first and
        // whole so the widening cannot narrow it.
        let one_value_all_literal =
            distinct.len() == 1 && stmts.iter().all(|s| s.value_is_lit);
        if !one_value_all_literal {
            // **The widened class.** Four clauses, each naming the constant it
            // is drawn from:
            //
            // 1. **ONE base symbol.** With a single symbol every producer's
            //    `nsw` — the symbol-group transitions before its value is first
            //    consumed — is identically 0, so `MAX_SYMBOL_CROSSINGS` holds
            //    by construction rather than by check. Board #621 measured a
            //    rival layout clause that answers the multi-symbol case at
            //    99.44 % and refused to ship it; this lane does not resurrect
            //    it to widen coverage.
            // 2. **At most `MAX_MODELLED_PRODUCERS` = 3 distinct literals.**
            //    Past three, c2 REUSES a register freed by an already-emitted
            //    store rather than taking a fresh one, and the two modes
            //    disagree about which (board #541, and `w-wire` measured the
            //    mode split: `/O1` reuses r11, `/Ox` takes r8).
            // 3. **The pool must hold them.** `r12` is never allocated, so the
            //    pool is r11 downward, and it starts above the live-in formals.
            //    Once it runs out c2 descends into registers freed by emitted
            //    stores — including a formal's own — which is unmodelled.
            // 4. **No multi-word literal beside another producer.** THE
            //    COUNTEREXAMPLE THAT FIRED. `{ a=100000; b=1; }` is
            //    `lis r11 ; li r10 ; ori r11 ; stw r10,4 ; stw r11,0` — c2
            //    interleaves the halves of the wide load with the other
            //    producer AND emits the stores in the opposite order to the
            //    model's. Every ORDER/ALLOC grid used single-word `li` values.
            //    A run whose *only* producer is wide is untouched by this
            //    clause, and that cell is in the pre-existing class above.
            if stmts.windows(2).any(|w| w[0].base_tok != w[1].base_tok) {
                return None;
            }
            const MAX_MODELLED_PRODUCERS: usize = 3;
            if distinct.len() > MAX_MODELLED_PRODUCERS {
                return None;
            }
            // `params[0]` is r3, so the first free register is r(3 + len), and
            // the pool top is r11.
            const POOL_TOP: usize = 11;
            let pool_floor = 3 + params.len();
            if pool_floor > POOL_TOP || POOL_TOP - pool_floor + 1 < distinct.len() {
                return None;
            }
            if distinct.len() > 1
                && distinct.iter().any(|&k| !(-0x8000..=0x7FFF).contains(&k))
            {
                return None;
            }
        }
    }
    // **Every F2 statement must actually carry an `AddrOf` where the emitter
    // reads one.** The mirror of the `value_is_lit` / `lit(s).is_none()` check
    // above and for the same reason: a flag that disagrees with the op stream is
    // a MISPARSE, not a widening, and the whole safety argument for admitting an
    // address into a run is that `c2_core` sees a four-op group and declines. A
    // statement that claims to be an address and hands over three ops would be
    // emitted as an ordinary store — the wrong-bytes direction.
    if stmts
        .iter()
        .any(|s| s.value_is_addr != matches!(s.ops.get(2), Some(IlOp::AddrOf { .. })))
    {
        return None;
    }
    // **A run may not MIX loaded values with formal/literal ones.** A run whose
    // values are *all* indirect loads is emitted in source order at every length
    // and every width probed (below); a run whose values are all formals is the
    // same; a run that mixes them is SCHEDULED. MEASURED
    // (`work/wsl/probe/p1.cpp`, `p4.cpp`, read off the reference obj):
    //
    // ```text
    //   { d->a = s->a; d->b = u; }        lwz r11,0(r4) ; stw r5,4(r3) ; stw r11,0(r3)
    //   { d->a = s->a; d->b = 2;  }       lwz r11,0(r4) ; li r10,2 ; stw r10,4(r3) ; stw r11,0(r3)
    //   { d->a=s->a; d->b=u; d->c=s->c; d->d=v; }
    //                                     lwz ; stw r5 ; stw r11 ; lwz ; stw ; stw r6
    // ```
    //
    // — the load is hoisted, its store SINKS past the next statement, and a
    // literal in that company gets its own second scratch register (r10, where
    // a pure run uses only r11). The reverse order happens to come back in
    // source order (`{ d->a = u; d->b = s->b; }` is `stw r5 ; lwz r11 ; stw r11`),
    // and it is refused anyway: two orders of one mixed pair disagree, so there
    // is no rule here, only two data points. `GAPS.md` §6 instance #10 —
    // measure at the edge, do not fit the scheduler.
    if stmts.len() > 1 {
        let ld = stmts[0].value_is_load;
        if stmts.iter().any(|s| s.value_is_load != ld) {
            return None;
        }
    }
    // **No object may be both loaded FROM and stored TO in one run.** c2 forwards
    // through the pair and eliminates the dead half: MEASURED,
    // `void SW(S* d){ d->a = d->b; d->b = d->a; }` is a *single*
    // `lwz r11,4(r3) ; stw r11,0(r3)` — the second statement is gone entirely.
    // The gate is on the TOKEN and not on the byte range, because the elimination
    // is a dataflow fact about one object and not about one offset, and a run of
    // ONE is unaffected (`void c1s(S* d){ d->a = d->b; }` is the plain pair).
    if stmts.len() > 1 {
        for a in &stmts {
            let Some(src) = a.src_tok else { continue };
            if stmts.iter().any(|b| b.base_tok == src) {
                return None;
            }
        }
    }
    // **A run of THREE or more may not mix the two register files.** c2 stops
    // emitting source order and SCHEDULES: `{ s->i=u; s->j=v; s->a=w; }` (two
    // `stw`s then a `stfs`) comes back `stfs f1,0(r3) ; stw r4,32(r3) ;
    // stw r5,36(r3)`, and `{ s->a=u; s->x=v; s->y=w; }` (one `stw`, two FP)
    // comes back `stfs ; stw ; stfd` — the FIRST FP store moved and the second
    // did not, so it is not "floating point first" either and no rule here is
    // derived. MEASURED at the edge rather than fitted, which is `GAPS.md` §6
    // instance #10's lesson applied in advance: source order holds for every
    // pure-GPR run to length 7, for every pure-FP run to length 4, and for
    // **every** mixed run of exactly 2 (42 ordered type pairs in
    // `scripts/sweep.d/82-store-run.py`, both orders) — and breaks in 16 of the
    // 24 mixed triples. So the gate is drawn where the evidence stops.
    //
    // **It does not apply to a run of loaded values**, and that is measured, not
    // assumed: a loaded value is a self-contained `lfs f0 ; stfs f0` or
    // `lwz r11 ; stw r11` pair with no live range to schedule across, and every
    // mixed-file order probed comes back in source order —
    // `{ d->f=s->f; d->i=s->i; d->q=s->q; }`, `{ d->i; d->q; d->g; }`,
    // `{ d->f; d->g; d->i; }` and the four-statement
    // `{ d->c; d->f; d->h; d->g; }` (`work/wsl/probe/p4.cpp`, MF1–MF4), against
    // 16 of 24 wrong for the formal-valued triples. The two populations differ in
    // exactly the property the scheduler acts on, so they get different gates.
    if stmts.len() > 2 && !stmts[0].value_is_load {
        let fp = stmts[0].value_is_fp;
        if stmts.iter().any(|s| s.value_is_fp != fp) {
            return None;
        }
    }
    // **The `/Ox` scratch descent, bounded where it stops being a plain descent.**
    // A loaded value needs a register to sit in between its load and its store,
    // and at `/Ox` c2 gives each statement its OWN, descending — r11, r10, r9, …
    // in the GPR file and f0 then f13, f12, … in the FP one, the two files
    // counted independently. (`/O1` reuses r11/f0 for every statement, so it has
    // no bound at all; the gate is drawn for the stricter mode because this
    // parser has no mode and the census must not over-claim in either.)
    //
    // MEASURED, `work/wsl/probe/p6.cpp` (runs of 1..8 × 2..6 pointer parameters,
    // both modes) and `p7.cpp` for the edge. The descent is a plain one until it
    // reaches a register a PARAMETER holds, and then c2 starts skipping and
    // wrapping:
    //
    // ```text
    //   L7 (S* d, S* s)                r11 r10 r9 r8 r7 r6 r5            a plain descent
    //   L8 (S* d, S* s)                r11 … r5, r4        <- r4 is `s`, dead after its
    //                                                         own last load
    //   L9 (S* d, S* s)                r11 … r5, r11, r10  <- WRAPS instead
    //   P8 (int,int,S* d,S* s)         r11 … r7, r4, r3, r11  <- SKIPS r6/r5, uses the
    //                                                            two dead int params,
    //                                                            then wraps
    // ```
    //
    // Reconstructing that needs a liveness model of the parameter registers, and
    // fitting one from these four rows is `GAPS.md` §6 instance #10's mistake. So
    // the run is admitted only while the descent stays strictly ABOVE every
    // register a parameter could hold, which is exactly the region where every
    // witness is a plain descent. A parameter that is never read is dead from
    // entry and c2 will happily use its register (`P8`); counting it as live only
    // refuses more, and refusing more is the safe direction.
    let nload_gpr = stmts.iter().filter(|s| s.value_is_load && !s.value_is_fp).count();
    let nload_fp = stmts.iter().filter(|s| s.value_is_load && s.value_is_fp).count();
    if nload_gpr > 0 {
        // `params` past the eighth are stack-homed and hold no register, so the
        // highest register a parameter can occupy is r10.
        let max_arg_reg = 2 + params.len().min(8);
        if 11usize.checked_sub(nload_gpr - 1).is_none_or(|lowest| lowest <= max_arg_reg) {
            return None;
        }
    }
    if nload_fp > 1 {
        // The FP descent is f0 then f13, f12, … . Its floor is the highest
        // FP *argument* register, and 8 is the longest run witnessed (`FF8`).
        let formals = parse_formals(seg, lo).ok()?;
        let classes = sy.arg_classes(&formals).ok()?;
        let nfp = classes.iter().filter(|c| matches!(c, ArgClass::Fp { .. })).count();
        if nload_fp > 8 || 14usize.checked_sub(nload_fp - 1).is_none_or(|lowest| lowest <= nfp) {
            return None;
        }
    }
    for (i, a) in stmts.iter().enumerate() {
        for b in &stmts[i + 1..] {
            if a.base_tok == b.base_tok
                && a.off < b.off + i64::from(b.width) as i32
                && b.off < a.off + i64::from(a.width) as i32
            {
                return None;
            }
        }
    }
    Some((stmts, params, p, depth))
}

/// **F3 — a store run followed by a CALL**: the composition
/// `src/xdk/nuispeech/xboxheap.cpp`'s constructor is, and the cheapest shape on
/// the frontier that can move TU match.
///
/// ```text
///   ( <scopes / line markers> <store statement> )+      the run — one locator,
///                                                       [`collect_store_run`]
///   26 <method>                                         a statement-position
///   B9 <recv> <PTR> [ 2C <PTR> 00 ] 99 <PTR> 00         member call on `this`
///   BD <call token> <args> 4C
///   4B                                                  the result is discarded
///   <ctor `return this` epilogue> <fn tail>
/// ```
///
/// # The gate is `w-heap` §3.2's, and it is SYNTACTIC
///
/// Board **#870** states the boundary as *"a trailing call that takes an
/// ARGUMENT breaks the run"*. Measured over `w-heap`'s frozen GRID F3 that is
/// **one level too coarse**, and the refinement is what puts `xboxheap` on the
/// safe side (board **#1129**):
///
/// ```text
///   Alloc(initSize)   member on `this`, both actuals already in their slots
///       stw 5,16(3) ; mr 31,3 ; stw 3,0(3) ; stw 3,4(3) ; bl ; mr 3,31
///       ONE base (r3) throughout, no setup at all, `mr 31,3` additive    <- #866
///
///   Alloc(size)       the setup is `mr 4,5` — it writes r4
///       stw 5,16(3) ; mr 4,5 ; stw 3,0(3) ; mr 31,3 ; stw 3,4(3) ; bl ; mr 3,31
///       ONE base (r3) throughout. The run STILL TRANSFERS.
///
///   g1(initSize)      a FREE callee, so the setup is `mr 3,4` — it writes r3
///       mr 31,3 ; stw 5,16(3) ; mr 3,4 ; stw 31,0(31) ; … ; bl ; mr 3,31
///       the base SWITCHES r3 -> r31 mid-run and the setup INTERLEAVES  <- #870
/// ```
///
/// So the predicate a framed seam owes is **"the call's argument setup is empty,
/// or writes no register the run reads"**, and it is computable from the actual
/// list alone: every slot `i` must already hold the formal that occupies it, so
/// the call emits no move. That is `w-heap` §3.2's own wording — and this
/// production makes it **stricter** than that sentence, because the reader sees
/// an IL argument *spelling* rather than a register: a slot that is anything but
/// `Load(params[i])` refuses, including a member load and a literal, which
/// cannot be shown to leave `r3` alone without a capture each.
///
/// # What is REFUSED, and why each is a *captured* neighbour
///
/// * **Anything but a member call on a receiver that is already slot 0.** A
///   free function, or a member call on another object, both put a non-`this`
///   value in slot 0 and therefore force the setup to write `r3` — axes D and E
///   of `w-gen`'s fragment are *determined by* axis C for exactly this reason
///   (board #1142), so they are refused here rather than crossed.
/// * **A call whose result is used.** `4B` — discarded — only. A result that is
///   returned or that feeds a store is a different body and one of `w-gen`'s
///   48 over-accept guards (board #1141).
/// * **Anything but the constructor tail.** Only the ctor is framed: the `void`,
///   `return <call>` and discarded-`int` forms are frame words **0** and
///   tail-call *behind* the run (board #1131, #869). Three of the four cells
///   that *look* like this shape are branches, so the tail is required
///   positively and not inferred from the run.
///
/// # THIS SHAPE HAS NO EMITTER, AND THE REFUSAL IS IN [`crate::func::bundle`]
///
/// `IlFunction` has no carrier for *a store run followed by a call*: `ops` and
/// `call_seq` are alternatives that `c2_core::codegen::select` tries in a fixed
/// order, so a function carrying both emits **one** of them and silently drops
/// the other — a store run emitted with its `bl` missing is `#232`'s exact
/// shape. So `shape_to_function` returns `None` for this variant and the census
/// files it under its own `:eof` key
/// ([`crate::func::census::STORE_RUN_CALL_NO_CARRIER`]): the body is
/// grammar-complete and the only thing left wrong with it is that the **model**
/// cannot spell the composition. That is board **#844**, and this production
/// is the measurement that it and F3 are **one refusal seen from two ends**
/// rather than the two independent rows `w-heap` §5's table shows.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_store_run_call(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
    depth0: usize,
) -> Option<BodyShape> {
    let (stmts, params, mut p, mut depth) = collect_store_run(seg, start, lo, sy, depth0)?;
    // The call. Reached through the SAME four locators `try_parse_member_tail_call`
    // uses — `eat_callee_push`, `eat_receiver_this`, `eat_call_token`,
    // `eat_call_args` — rather than a second decoder for the same bytes. A
    // private copy of the receiver walk is the drift `GAPS.md` §6 instance #9
    // records, and the `volatile` gate `eat_receiver_this` inherits through
    // `eat_operand_type` is the thirteenth live wrong-bytes emit on this project.
    if seg.get(p) != Some(&0x26) {
        return None;
    }
    let callee_tok = super::calls::eat_callee_push(seg, &mut p).ok()?;
    // A second `26` here is a stacked method (`p->a()->b()`) or a named data
    // object standing where the receiver goes — both are other productions and
    // both put a non-`this` value in slot 0, so both are refused rather than
    // dispatched.
    if seg.get(p) == Some(&0x26) {
        return None;
    }
    let recv_tok = super::mcall_tail::eat_receiver_this(seg, &mut p).ok()?;
    let _ret = super::calls::eat_call_token(seg, &mut p).ok()?;
    let args = super::calls::eat_call_args(seg, &mut p).ok()?;
    // `this` is argument slot 0 and the list is in STREAM order — rightmost
    // source argument first — so slot `i` is `args[len - 1 - i]` and the receiver
    // goes on the END. Getting this backwards is invisible on a nullary call and
    // on any symmetric permutation, which is the shape of defect `GAPS.md` §6
    // keeps recording; `member_tail_call_puts_this_in_slot_zero` pins the same
    // fact for the tail form.
    let mut slots: Vec<&[IlOp]> = args.iter().map(|a| a.as_slice()).rev().collect();
    let recv = [IlOp::Load(recv_tok)];
    slots.insert(0, &recv);
    // **THE REGIME GATE.** Every slot must already hold the formal that occupies
    // it, so the call's argument setup is EMPTY and the run's base register is
    // never written. Slot 0 is the receiver and must be `params[0]` — which for a
    // member function is `this`, the base every store in the run is through.
    if slots.len() > params.len() {
        return None;
    }
    for (i, slot) in slots.iter().enumerate() {
        match slot {
            [IlOp::Load(t)] if params.get(i) == Some(t) => {}
            _ => return None,
        }
    }
    // The result is DISCARDED. A returned or consumed result is a different body
    // and one of `w-gen`'s over-accept guards.
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // A brace scope closes between the statement end and the return branch, the
    // same rule `try_parse_member_tail_call` applies at the same position.
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    // **Only the CONSTRUCTOR tail.** The other three forms are frame words 0 and
    // tail-call behind the run (#1131), so they must not reach a framed
    // composition. Required positively: `eat_return_head`, then the ctor's
    // `return this`, then the segment end.
    let mut q = p;
    eat_return_head(seg, &mut q, false, depth).ok()?;
    if !(eat_ctor_this_epilogue(seg, &mut q, lo) && eat_fn_tail(seg, &mut q).is_ok()) {
        return None;
    }
    Some(BodyShape::StoreRunCall {
        params,
        ops: stmts.into_iter().flat_map(|s| s.ops).collect(),
        callee_tok,
    })
}

/// The run's own **tail** — the three spellings [`try_parse_store_run`] admits,
/// applied to a run [`collect_store_run`] has already gated.
fn run_tail(
    seg: &[u8],
    p: usize,
    lo: usize,
    depth: usize,
    stmts: Vec<StoreStmt>,
    params: Vec<u32>,
) -> Option<BodyShape> {
    // The tail. Three spellings, and the two non-void ones are **the same fact
    // twice** — the value the function returns is already in r3, so the return
    // costs nothing. Which spelling appears is decided by the SOURCE construct
    // and not by this parser: an explicit `return *this;` / `return s;` is an
    // ordinary value return (`B9 <tok> <T> 41 <T>` ahead of the `3A`), while a
    // CONSTRUCTOR's implicit result sits *after* the `29`
    // ([`eat_ctor_this_epilogue`]). On the 878-TU workload the constructor form
    // is 42,238 of the 54,433 and the explicit one is ~1,000.
    let mut q = p;
    if eat_first_formal_result(seg, &mut q, &params)
        && eat_return_plumbing(seg, &mut q, false, depth).is_ok()
    {
        return Some(BodyShape::StoreRun {
            params,
            ops: stmts.into_iter().flat_map(|s| s.ops).collect(),
        });
    }
    let mut q = p;
    eat_return_head(seg, &mut q, false, depth).ok()?;
    let mut r = q;
    if eat_fn_tail(seg, &mut r).is_err() {
        // …or the constructor's `return this`, between the RETURN and the tail.
        r = q;
        if !(eat_ctor_this_epilogue(seg, &mut r, lo) && eat_fn_tail(seg, &mut r).is_ok()) {
            return None;
        }
    }
    Some(BodyShape::StoreRun {
        params,
        ops: stmts.into_iter().flat_map(|s| s.ops).collect(),
    })
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
    /// **F2 — `void f(S* s){ s->mFreeHead = &s->mListHead; }`**, the WHOLE
    /// captured segment from `work/w-f23/grid/f2_leaf_g8/` at the workload's own
    /// `/GR /O1 /Oi /EHsc` (board #1112 — the harness default `/Ox` is a
    /// different population and a refusal reads as paid at one while unpaid at
    /// the other). The cell is graded by both instruments in
    /// `work/w-f23/grid/f2_leaf_g8/{census,gap}.after.txt`: **in class**, and the
    /// port refuses it honestly (`codegen-gap`, *"sub-object address feeding
    /// arithmetic"*).
    const F2_ADDR_VALUE: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0B, 0x53, 0x53, 0x26, 0xF9, 0x09,
        0x46, 0x2D, 0xF8, 0x09, // formals: s
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0xF8, 0x09, 0x86, 0x43, 0x81, 0x20, // the DESTINATION: s
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, // + 0
        0xB9, 0xF8, 0x09, 0x86, 0x43, 0x81, 0x20, // the VALUE's base: s
        0x33, 0x86, 0x41, 0x74, 0x08, 0x27, 0x86, 0x43, 0x86, 0x20, // + 8, and NO `30`
        0x32, 0x86, 0x43, 0x83, 0x20, 0x4B, // the store, ptr, discarded
        0x3A, 0xFA, 0x09, 0x54, 0x02, 0x29, 0xFA, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **THE DISCRIMINATING NEGATIVE** — `void f(S* d, S* q){ d->mUsedHead =
    /// q->mFreeHead; }` from `work/w-f23/grid/f2_ctrl_load/`, whose IL differs
    /// from [`F2_ADDR_VALUE`] by exactly the **`30`**. It emits `lwz r11 ; stw
    /// r11` where the address value emits one `addi`, so admitting one as the
    /// other is a wrong-bytes emit rather than a gap — and it was `Port=Match`
    /// before this rung, so the assertion below is that it still decodes as a
    /// LOAD and not as an address.
    const F2_LOAD_VALUE_TWIN: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08, 0x53, 0x53, 0x26, 0xF7, 0x09,
        0x46, 0x2D, 0xF6, 0x09, 0x2D, 0xF5, 0x09, // formals: q, d  ->  params [d, q]
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0xF5, 0x09, 0x86, 0x43, 0x81, 0x20, //
        0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x86, 0x20, // d + 4
        0xB9, 0xF6, 0x09, 0x86, 0x43, 0x81, 0x20, //
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, // q + 0
        0x30, 0x86, 0x43, 0x83, 0x20, // THE `30` — this is a LOAD, not an address
        0x32, 0x86, 0x43, 0x83, 0x20, 0x4B, //
        0x3A, 0xF8, 0x09, 0x54, 0x02, 0x29, 0xF8, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn f2_admits_a_member_address_as_a_stored_value_and_the_30_still_separates_the_load() {
        // The address value produces `AddrOf`, so the op stream has FOUR ops
        // where a formal-valued store has three — which is what makes
        // `c2_core`'s `parse_simple_gpr_run` and every arm of `store_leaf_text`
        // decline it rather than emit a `stw` from a register that holds a
        // *base* pointer. The refusal is `straightline`'s
        // "sub-object address feeding arithmetic", measured on the graded cell.
        assert_eq!(
            parse_segment(F2_ADDR_VALUE, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0xF809],
                ops: vec![
                    IlOp::Load(0xF809),
                    IlOp::Load(0xF809),
                    IlOp::AddrOf { off: 8 },
                    IlOp::StoreInd { off: 0, width: 4 },
                ],
            })
        );
        // …and its twin, one byte away, must still be a LOAD. If F2 were tried
        // before `parse_load_value`, or if its `30` refusal were missing, this
        // would come back as `AddrOf` and emit one `addi` where c2 emits
        // `lwz`+`stw` — the wrong-bytes direction, not a gap.
        assert_eq!(
            parse_segment(F2_LOAD_VALUE_TWIN, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0xF509, 0xF609],
                ops: vec![
                    IlOp::Load(0xF509),
                    IlOp::Load(0xF609),
                    IlOp::LoadInd { off: 0 },
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
    }

    #[test]
    fn f2_leaves_the_zero_offset_address_refused_because_c2_emits_no_addi_there() {
        // `AddrOf { off: 0 }` emits **nothing**, so `s->p = &s->m;` with `m` at
        // offset 0 is the same bare `stw` the plain pointer value emits — and
        // that shape is already byte-graded through the formal path. Admitting
        // it here would put a second production over one fact, which is
        // `GAPS.md` §6's "one fact, two locators". Measured on the graded cell
        // `work/w-f23/grid/f2_leaf_g0/`, which stays `vocab-gap` after this rung.
        let mut seg = F2_ADDR_VALUE.to_vec();
        // The VALUE's offset literal — the second `33 86 41 74 <k>` in the body.
        let at = seg
            .windows(5)
            .rposition(|w| w == [0x33, 0x86, 0x41, 0x74, 0x08])
            .expect("the value's offset add");
        seg[at + 4] = 0x00;
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    /// W25: the store leaf, from whole captured segments — both designators, the
    /// widths that pick the opcode, the literal value, and the FP refusal.
    #[test]
    fn store_leaf_decodes_both_designators_and_refuses_a_float_value() {
        assert_eq!(
            parse_segment(STORE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0xF909, 0xFA09],
                ops: vec![
                    IlOp::Load(0xF909),
                    IlOp::Load(0xFA09),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // The width comes from the STORED type, not from the designator's pointer
        // tag — the two agree for an `int` member and this is where they part.
        assert_eq!(
            parse_segment(STORE_NARROW, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x010A, 0x020A],
                ops: vec![
                    IlOp::Load(0x010A),
                    IlOp::Load(0x020A),
                    IlOp::StoreInd { off: 12, width: 1 },
                ],
            })
        );
        assert_eq!(
            parse_segment(STORE_LIT, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x210A],
                ops: vec![
                    IlOp::Load(0x210A),
                    IlOp::Lit(7),
                    IlOp::StoreInd { off: 0, width: 4 },
                ],
            })
        );
        // The intrinsic-2117 designator reaches the same address by a different
        // route and must produce the byte-identical op stream.
        assert_eq!(
            parse_segment(STORE_BASE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x610A, 0x620A],
                ops: vec![
                    IlOp::Load(0x610A),
                    IlOp::Load(0x620A),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // …and the neighbour that emits `stfs f1` must refuse, in the parser.
        assert_eq!(parse_segment(STORE_FLOAT_NEG, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(STORE_FLOAT_NEG, NO_LOCALS)
                .unwrap_err()
                .feature(),
            "expr-op-0x27"
        );
    }

}
