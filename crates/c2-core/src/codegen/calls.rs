//! The frame/call complex: framed bodies, call sequences, tail calls, and
//! argument permutation. **One module on purpose.**
//!
//! This is the serial spine's home (Class B — values live across calls —
//! Class C helpers, multi-call accumulators, and the EH records after them).
//! Its parts are genuinely coupled to each other and to `coff.rs`'s symbol and
//! label order; pretending they are independent would invite two agents to
//! guess allocator order concurrently, which is the one thing the doctrine
//! forbids (`docs/ARCHITECTURE_SEAMS.md` §7, §9.8). One agent owns this file.

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{encode_addi, encode_blr, encode_mr, encode_subf};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{ARG_REGS, OptMode, RET_REG, SCRATCH_REG, out_of_class};
use crate::codegen::straightline::select_text;

/// Encode an unconditional relative branch `b` (primary opcode 18, AA=0, LK=0)
/// used for a **tail call**, to be paired with a REL24 relocation.
///
/// MSVC's PPC convention stores the displacement field as `−(the instruction's
/// own byte offset within .text)`, so the pre-link value references `.text`
/// offset 0; the linker then patches the 24-bit field from the target symbol.
/// Verified: a tail-call `b` at offset 0 → `0x48000000`; at offset 8 →
/// `0x4BFFFFF8` (displacement −8).
pub fn encode_tail_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC);
    word.to_be_bytes()
}

/// Encode a **linking** relative branch `bl` (primary opcode 18, AA=0, **LK=1**)
/// used for a non-leaf `bl <callee>` inside a framed function, paired with a
/// REL24 relocation. Same MSVC displacement convention as [`encode_tail_branch`]
/// (`disp = −(own .text offset)`) plus the link bit. Verified: `bl` at offset
/// 0xC → `0x4BFFFFF5` (disp −0xC, LK=1).
pub fn encode_call_branch(text_offset: u32) -> [u8; 4] {
    let disp = -(text_offset as i32);
    let word: u32 = 0x4800_0000 | (disp as u32 & 0x03FF_FFFC) | 1;
    word.to_be_bytes()
}

/// A framed non-leaf call's emitted body: the bytes, and the `.text` offsets the
/// caller needs — the REL24 site of the `bl` and the prologue length that
/// becomes the `$M(n)` label and the `.pdata` `PrologLen`.
pub struct FramedBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the `bl <callee>` (already includes
    /// `base_off`): the REL24 relocation site.
    pub bl_offset: u32,
    /// Prologue length in bytes, relative to the function start.
    pub prolog_len: u32,
}

/// Emit the `.text` for a **framed non-leaf call** `return g(<formal>) + k`
/// (W4b2).
///
/// ```text
/// 7d8802a6  mflr r12                prologue: save LR
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)         allocate the 96-byte frame
/// [7c832378  or  r3,rN,rN]          argument setup — ONLY when the argument is
///                                   not already the formal in r3
/// 4bfffff5  bl   <callee>           REL24 reloc site
/// 3863kkkk  addi r3,r3,k            the post-call op (+k)
/// 38210060  addi r1,r1,96           epilogue: free frame
/// 8181fff8  lwz  r12,-8(r1)         restore LR
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// **The argument-setup word was missing and that was a live wrong-bytes emit.**
/// This function used to emit one byte-constant 0x24-byte body, on the tacit
/// assumption that the call's argument is always the formal already in r3. The
/// parser only ever required the argument to be *a* formal, so
/// `int f(int a,int b){ return g(b) + 1; }` emitted 9 words where c2 emits 10 —
/// `or r3,r4,r4` at `.text+0xC` — with the `.pdata` `FuncLen` and both `$M`
/// labels wrong to match. 37 of 47 probes around the accepted class mismatched,
/// including every member function (`this` occupies r3, so a one-parameter
/// member's argument is in r4) and every free function with a leading `float`,
/// `double`, `long long`, pointer or 8-byte aggregate parameter. The sweep never
/// separated it because every generated framed case is `int F(int a){ return
/// g(a) + 1; }` — one parameter, in r3. `docs/GAPS.md` §6: a corpus holding only
/// the safe half of a pair cannot see the dangerous half.
///
/// The setup is computed by the caller through [`select_text`], the *same*
/// locator the integer tail call uses for the same job, so the formal → argument
/// register mapping has one implementation and not two.
///
/// `k` must fit the signed-16-bit `addi` immediate (the IL parser guarantees
/// this before constructing the [`c2_il::FramedCall`]).
///
/// `base_off` is the function's start within the `.text` section it lands in —
/// 0 under `/Gy` (its own COMDAT) and its packed offset otherwise. It exists
/// because the `bl` displacement follows MSVC's `disp = −(own .text offset)`
/// convention, so a framed function that is **not first** in a packed `.text`
/// needs a different branch word: `?f` at 0x08 with the `bl` at 0x14 gets
/// `4BFFFFED`, not the `4BFFFFF5` this function emitted unconditionally. That
/// was unreachable while a framed TU was gated to one function and became a
/// live wrong-bytes emit the moment the gate came off.
pub fn framed_call_text(
    setup: &[u8],
    add_k: i32,
    base_off: u32,
    frame: FrameLayout,
) -> Result<FramedBody, BackendError> {
    let k = add_k as i16; // range-checked upstream (c2_il::func::parse_segment)
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let bl_offset = base_off + prolog_len + setup.len() as u32;
    let mut text = Vec::with_capacity(prologue.len() + setup.len() + 8 + epilogue.len());
    text.extend_from_slice(&prologue);
    text.extend_from_slice(setup);
    // Call (LK=1); the REL24 reloc at `bl_offset` patches the target.
    text.extend_from_slice(&encode_call_branch(bl_offset));
    // Post-call op.
    text.extend_from_slice(&encode_addi(RET_REG, RET_REG, k)); // addi r3,r3,k
    text.extend_from_slice(&epilogue);
    Ok(FramedBody { text, bl_offset, prolog_len })
}

/// A **Class A many-call body**'s emitted `.text`, with one REL24 site per call.
pub struct SeqBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the `bl <callee>` words (already including
    /// `base_off`), in call order — one relocation site each.
    pub bl_offsets: Vec<u32>,
    pub prolog_len: u32,
}

/// Emit the `.text` for a **Class A many-call body** (`docs/GAPS.md` #35 step 2,
/// rung 1): a framed function whose body is a sequence of calls with nothing live
/// across any of them.
///
/// ```text
/// 7d8802a6  mflr r12                prologue — the shipped Class A three words
/// 9181fff8  stw  r12,-8(r1)
/// 9421ffa0  stwu r1,-96(r1)
/// <setup 0>                         per call: the argument marshalling …
/// 4bfffff5  bl   <callee 0>         … then the LINKING branch. REL24 site.
/// <setup 1>
/// 4bfffff1  bl   <callee 1>
/// <tail>                            nothing, `addi r3,r3,k`, or `li r3,k`
/// 38210060  addi r1,r1,96           epilogue
/// 8181fff8  lwz  r12,-8(r1)
/// 7d8803a6  mtlr r12
/// 4e800020  blr
/// ```
///
/// Byte evidence, one probe per row, all at `/O1 /GS- /c` (and the `.text` is
/// identical at `/Ox` and `/O2` — `docs/CODEGEN_FRAMED_CALLS.md`'s mode note):
///
/// ```text
///   void f(){ g1(); g2(); }             36 B  bl bl
///   void f(){ g1(); g2(); g3(); g4(); } 44 B  bl bl bl bl
///   void f(int a){ g1(a); g2(); }       36 B  a is already in r3
///   void f(){ g1(1); g2(2); }           44 B  li r3,1 · bl · li r3,2 · bl
///   void f(int a,int b){ g2(a,b); h(); }36 B  identity permutation, no moves
///   void f(int a,int b){ g2(b,a); h(); }48 B  mr r11,r4 · mr r4,r3 · mr r3,r11
///   void f(int a){ g1(a+1); g2(); }     40 B  addi r3,r3,1
///   int  f(int a){ g1(a); return 5; }   36 B  li r3,5 after the last bl
///   int  f(){ g1(); return g2(); }      36 B  no post-op at all
///   int  f(){ g1(); return g2()+1; }    40 B  addi r3,r3,1
/// ```
///
/// **The last call is a `bl`, never a `b`.** Every row above ends
/// `bl <callee> … addi r1,r1,96 … blr`: c2's tail-call transform is off once the
/// function is framed. The one shape that *is* tail-called is a lone statement
/// call with nothing after it (`void f(int a){ g(a); }` → a bare `b ?g`, five
/// sections, no `.pdata`), and the IL parser routes that to the tail-call
/// production so it can never reach here.
///
/// `setups[i]` is call `i`'s argument marshalling, already computed by the caller
/// through the *same* [`select_text`] / [`permute_args_text`] locators every other
/// call shape uses; `tail` is the post-call word(s), empty for a void body.
/// `base_off` is the function's start within its `.text` section — 0 under `/Gy`,
/// its packed offset otherwise — because the `bl` displacement follows MSVC's
/// `disp = −(own .text offset)` convention.
pub fn call_seq_text(
    setups: &[Vec<u8>],
    tail: &[u8],
    base_off: u32,
    frame: FrameLayout,
) -> Result<SeqBody, BackendError> {
    if setups.is_empty() {
        return Err(out_of_class("a call sequence with no calls"));
    }
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;
    let mut text = prologue;
    let mut bl_offsets = Vec::with_capacity(setups.len());
    for setup in setups {
        text.extend_from_slice(setup);
        let off = base_off + text.len() as u32;
        text.extend_from_slice(&encode_call_branch(off));
        bl_offsets.push(off);
    }
    text.extend_from_slice(tail);
    text.extend_from_slice(&epilogue);
    Ok(SeqBody { text, bl_offsets, prolog_len })
}

/// The parameter index a single-`Load` argument stream names, for a Class B call
/// that has to read it out of a callee-saved register instead of an argument one.
fn seq_load_param_index(params: &[u32], ops: &[IlOp]) -> Result<usize, BackendError> {
    match ops {
        [IlOp::Load(t)] => params
            .iter()
            .position(|q| q == t)
            .ok_or_else(|| out_of_class("a call argument that is not a formal")),
        _ => Err(out_of_class("expected a single passthrough argument")),
    }
}

/// Lower one call's argument setup, or a `Lit`-only tail, through
/// [`select_text`] — the locator the integer tail call and the single framed call
/// already use — and drop its trailing `blr`.
///
/// A synthetic [`IlFunction`] is the input because `select_text` reads exactly two
/// of its fields (`params` and `ops`). Copying the selection logic instead is how
/// the argument-register move went missing from `framed_call_text`
/// (`docs/ROADMAP.md` §6g item 1): the one-implementation rule is the whole point.
fn ops_setup_text(
    params: &[u32],
    ops: &[IlOp],
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    if ops.is_empty() {
        return Ok(Vec::new());
    }
    let synth = IlFunction {
        mangled_name: String::new(),
        source_path: None,
        params: params.to_vec(),
        ops: ops.to_vec(),
        tail_call: None,
        framed_call: None,
        call_seq: None,
        compare: None,
        float_leaf: None,
        fp_tail: None,
        fp_arg_sources: None,
        arg_sources: None,
        empty_body: false,
    };
    let mut t = select_text(&synth, mode)?;
    let blr = encode_blr();
    debug_assert!(t.ends_with(&blr), "select_text always terminates in blr");
    t.truncate(t.len() - blr.len());
    Ok(t)
}

/// The callee-saved GPR file, in allocation order: `r31` first, then `r30`.
/// Only two, because at three saved GPRs c2 switches to `__savegprlr_29` and the
/// whole prologue/epilogue/label shape changes ([`FrameLayout::needs_gpr_helper`],
/// `docs/CODEGEN_FRAMED_CALLS.md` §2.3) — the IL parser refuses that class, and
/// this array is the second lock on it.
const SAVED_GPRS: [u8; 2] = [31, 30];

/// Emit a set of **non-conflicting** register-to-register moves, highest
/// destination first.
///
/// This is §3.2's rule for a marshalling with no permutation to break, and it is
/// what a Class B call after the first always needs: its sources are callee-saved
/// registers (`r31`/`r30`) and its destinations are argument registers
/// (`r3`…`r10`), two disjoint sets, so no move can clobber another's source and
/// [`permute_args_text`]'s cycle machinery has nothing to do. Captured:
///
/// ```text
///   void f(int a,int b,int c){ v1(a); g2(c,b); }
///     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r4,r31 ; mr r3,r30 ; bl ?g2
/// ```
///
/// — slot 0 wants `c` (in r30) and slot 1 wants `b` (in r31), and the r4 move is
/// emitted before the r3 one, which is the same descending order the leaf
/// selector and the argument permutation already use.
fn moves_descending(moves: &[(u8, u8)]) -> Vec<u8> {
    let mut m: Vec<(u8, u8)> = moves.iter().copied().filter(|(d, s)| d != s).collect();
    m.sort_by(|a, b| b.0.cmp(&a.0));
    let mut w = Vec::with_capacity(4 * m.len());
    for (dst, src) in m {
        w.extend_from_slice(&encode_mr(dst, src));
    }
    w
}

/// The per-call argument setups and the post-call tail bytes of a framed
/// many-call body, in one place so the packed and `/Gy` emitters share them.
///
/// **Class B** (`seq.saved` non-empty) differs from Class A in two places and
/// nowhere else: the first call's setup gains one `mr rSaved, rArg` per saved
/// formal, and every later call marshals **out of** the saved registers instead of
/// the argument ones. The frame, the prologue and the epilogue are
/// [`FrameLayout`]'s job and the caller sets `saved_gprs` from the same list.
///
/// Byte evidence for the whole shape, `/O1 /GS- /c`:
///
/// ```text
///   void f(int a,int b){ v1(a); v2(b); }                52 B, F = 96
///     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
///     mr r31,r4 ; bl ?v1 ; mr r3,r31 ; bl ?v2
///     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
///
///   void f(int a,int b,int c){ v1(a); v2(b); v3(c); }   72 B, F = 112
///     … std r30,-24(r1) ; std r31,-16(r1) ; stwu r1,-112(r1)
///     mr r31,r4 ; mr r30,r5 ; bl ?v1 ; mr r3,r31 ; bl ?v2 ; mr r3,r30 ; bl ?v3
///     … ld r30,-24(r1) ; ld r31,-16(r1) ; blr
/// ```
///
/// The save moves are emitted **after** the first call's own setup, which the IL
/// parser keeps empty for this class ([`c2_il`]'s `plan_saved_gprs` records the
/// measured exception and why it is refused rather than modeled).
pub fn call_seq_parts(
    params: &[u32],
    seq: &c2_il::CallSeq,
    mode: OptMode,
) -> Result<(Vec<Vec<u8>>, Vec<u8>), BackendError> {
    // The TOTAL, not `seq.saved.len()`: a tail that keeps an earlier call's
    // result alive takes a register from the same file, so a body with two saved
    // formals and such a tail is already the helper class.
    if seq.saved_gprs() > SAVED_GPRS.len() {
        return Err(out_of_class(
            "three or more callee-saved GPRs: that is the `__savegprlr_N` helper \
             class, with a second REL24 site, a tail-branch epilogue and a \
             different /Gy label stride",
        ));
    }
    // Where each saved formal lives once it is saved: parameter index -> register.
    let saved_reg = |pi: usize| -> Option<u8> {
        seq.saved.iter().position(|&s| s == pi).map(|k| SAVED_GPRS[k])
    };
    // **The first call's RESULT**, when the tail consumes it across a later `bl`.
    // `docs/CODEGEN_FRAMED_CALLS.md` §3.1: "call results take the next descending
    // register after the parameters" — so it is `SAVED_GPRS[seq.saved.len()]`,
    // never a fixed r30. Captured at both widths: `bool f(const U* p){ return
    // p->m() == p->n(); }` saves one formal and takes r30 for the result, and the
    // hypothetical zero-formal form would take r31.
    let result_reg: Option<u8> = seq
        .tail
        .saves_a_call_result()
        .then(|| SAVED_GPRS.get(seq.saved.len()).copied())
        .flatten();

    let mut setups = Vec::with_capacity(seq.calls.len());
    for (i, c) in seq.calls.iter().enumerate() {
        let mut setup = if i == 0 || seq.saved.is_empty() {
            // The first call still reads its arguments out of the argument
            // registers — nothing has been clobbered yet — so it goes through the
            // same locators every other call shape uses. Class A takes this arm
            // for every call.
            //
            // **The save moves interleave with it, and where is measured.** A save
            // whose source register this marshalling OVERWRITES is hoisted in
            // front of the whole marshalling; one whose source it leaves alone is
            // emitted after it. Both halves in one capture,
            // `void f(int a,int b,int c,int d){ g2(a,d); v1(b); v2(c); }`:
            //
            // ```text
            //   mr r31,r4    b — r4 is about to be overwritten, so HOISTED
            //   mr r4,r6     the marshalling (slot 1 <- d)
            //   mr r30,r5    c — r5 is untouched, so it TRAILS
            //   bl ?g2
            // ```
            //
            // The hoist clears the **whole** marshalling, not just the instruction
            // that would clobber: `void f(int a,int b,int c,int d,int e){
            // g3(a,d,e); v1(b); }` is `mr r31,r4 ; mr r5,r7 ; mr r4,r6`, which
            // refutes the "save as late as possible" reading that fits the row
            // above. Within each group the moves go highest destination first,
            // which for the saves is the same as parameter order.
            let (text, writes) = match (&c.arg_sources, c.arg_ops.as_slice()) {
                // A non-identity permutation beside a save is refused by the IL
                // parser: c2 breaks the cycle with the **callee-saved register**
                // rather than r11 there, which is a different algorithm and not
                // this interleaving at all. This is the backstop.
                (Some(sources), _) => {
                    let (t, w) = permute_args_parts(sources)?;
                    if !seq.saved.is_empty() && !t.is_empty() {
                        return Err(out_of_class(
                            "a permuted first call beside a callee-saved copy: c2                              breaks the cycle through the callee-saved register                              instead of r11, which is not characterized",
                        ));
                    }
                    (t, w)
                }
                (None, []) => (Vec::new(), Vec::new()),
                // A single passthrough or literal argument selects to `mr r3,rN`
                // or `li r3,k` — one word writing r3, or nothing at all when the
                // value is already there.
                (None, [IlOp::Load(_)]) | (None, [IlOp::Lit(_)]) => {
                    let t = ops_setup_text(params, &c.arg_ops, mode)?;
                    let w = if t.is_empty() { Vec::new() } else { vec![RET_REG] };
                    (t, w)
                }
                // Anything computed. Under `/Ox` a chain intermediate goes to a
                // fresh **descending** register, which is the same file the saves
                // live in, so the write set is not `{r3}` and the interleaving is
                // not the measured one. The IL parser refuses this while anything
                // is saved; this is the backstop.
                (None, _) if !seq.saved.is_empty() => {
                    return Err(out_of_class(
                        "a computed first-call argument beside a callee-saved copy: \
                         the marshalling's write set reaches the callee-saved file \
                         and the interleaving is not characterized",
                    ))
                }
                (None, ops) => (ops_setup_text(params, ops, mode)?, Vec::new()),
            };
            let mut hoisted = Vec::new();
            let mut trailing = Vec::new();
            for (k, &pi) in seq.saved.iter().enumerate() {
                let src = *ARG_REGS.get(pi).ok_or_else(|| {
                    out_of_class("a stack-homed formal cannot be copied to a callee-saved GPR")
                })?;
                let mv = (SAVED_GPRS[k], src);
                if writes.contains(&src) {
                    hoisted.push(mv);
                } else {
                    trailing.push(mv);
                }
            }
            let mut w = moves_descending(&hoisted);
            w.extend_from_slice(&text);
            w.extend_from_slice(&moves_descending(&trailing));
            w
        } else {
            // A later call in Class B: every formal it reads is in a callee-saved
            // register by construction (that is what put it there), and a literal
            // argument is the same `li r3,k` the leaf selector emits.
            match (&c.arg_sources, c.arg_ops.as_slice()) {
                (Some(sources), _) => {
                    let mut moves = Vec::with_capacity(sources.len());
                    for (slot, &pi) in sources.iter().enumerate() {
                        let (Some(src), Some(&dst)) = (saved_reg(pi), ARG_REGS.get(slot)) else {
                            return Err(out_of_class(
                                "a call after the first reads a value that is not \
                                 in a callee-saved register",
                            ));
                        };
                        moves.push((dst, src));
                    }
                    moves_descending(&moves)
                }
                (None, [c2_il::IlOp::Load(_)]) => {
                    // `params` and the operand agree by construction; the parser
                    // resolved the token to a formal index before accepting.
                    let pi = seq_load_param_index(params, &c.arg_ops)?;
                    let src = saved_reg(pi).ok_or_else(|| {
                        out_of_class("a call after the first reads an unsaved formal")
                    })?;
                    moves_descending(&[(RET_REG, src)])
                }
                (None, ops) => ops_setup_text(params, ops, mode)?,
            }
        };
        // **Save the first call's result** before the second call's setup can
        // clobber r3. Measured, every capture of the shape:
        //
        // ```text
        //   mr r30,r3      the first result -> its callee-saved register
        //   mr r3,r31      the second call's receiver -> r3
        //   bl ?n
        // ```
        //
        // Emitted as a prefix rather than folded into `moves_descending`'s set:
        // the two are not one non-conflicting group — `mr r3,r31` reads r31 and
        // `mr r30,r3` reads r3, so their order is a data dependence, not a
        // sort key. (Descending order happens to agree here, and relying on that
        // coincidence is how a later widening with a lower-numbered save register
        // would silently invert them.)
        if i == 1 {
            if let Some(rr) = result_reg {
                let mut w = encode_mr(rr, RET_REG).to_vec();
                w.extend_from_slice(&setup);
                setup = w;
            }
        }
        setups.push(std::mem::take(&mut setup));
    }
    let tail = match seq.tail {
        c2_il::SeqTail::Void => Vec::new(),
        // The result is already in r3; `+0` folds away exactly as it does for the
        // single framed call.
        c2_il::SeqTail::CallValue { add_k: 0 } => Vec::new(),
        c2_il::SeqTail::CallValue { add_k } => {
            let k = i16::try_from(add_k)
                .map_err(|_| out_of_class("call-sequence post-op wider than an addi immediate"))?;
            encode_addi(RET_REG, RET_REG, k).to_vec()
        }
        // `return <literal>;` — the same `li r3,k` a bare-literal leaf emits, so it
        // goes through the same selector rather than a second encoder.
        c2_il::SeqTail::Lit(k) => ops_setup_text(params, &[IlOp::Lit(k)], mode)?,
        // **WCB — `return a->m() == b->n();`**, the register-register difference
        // spine (`docs/CMP_PRODUCES_A_VALUE.md` reading 4, `docs/rungs/2026-07-31-cmp-two-calls.md`):
        //
        // ```text
        //   subf r11,<lhs>,<rhs>          rB - rA, i.e. rhs - lhs
        //   cntlzw r10,r11 ; rlwinm r3,r10,27,31,31        (r11,r11 under /O1)
        // ```
        //
        // The operand roles are **not** the emission order: the first call's
        // result is in `result_reg` and the second's is still in r3, and which of
        // those is the source's left operand is `lhs_first`. c2 orders the calls by the order c1xx
        // NUMBERED their receivers (`this` last, whatever register it is in) and
        // keeps the spine's operands in source order, so both facts are needed
        // and neither implies the other.
        c2_il::SeqTail::CmpEq { lhs_first } => {
            let first = result_reg.ok_or_else(|| {
                out_of_class("a comparison of two call results needs a callee-saved register for the first")
            })?;
            let (lhs, rhs) = if lhs_first { (first, RET_REG) } else { (RET_REG, first) };
            let mut w = encode_subf(11, lhs, rhs).to_vec();
            w.extend_from_slice(&crate::codegen::leaf::compare::eq_zero_of_difference_in_r11(mode));
            w
        }
    };
    Ok((setups, tail))
}

/// Emit the `.text` for an **integer tail call** `return g(<arg>)` (and the
/// identity-fold `g(a) + 0`): the single call argument computed into r3 by the
/// leaf arithmetic selector, then a `b <callee>` tail branch (paired with a
/// REL24 relocation the caller registers at the returned offset). Returns the
/// `.text` bytes and the branch's byte offset within `.text` (= `base_off` +
/// the arg-setup length; the reloc site).
///
/// The argument setup reuses [`select_text`] (params → r3, the exact same
/// instruction class as a leaf `return <arg>`) and drops its trailing `blr`:
/// the argument value stays live in r3 across the tail branch. A passthrough
/// `g(a)` selects to just `blr` → an empty prefix → a bare `b g` at `base_off`,
/// byte-identical to the void tail call; `g(a + 1)` selects `addi r3,r3,1 ; blr`
/// → the prefix `addi r3,r3,1` and the branch at `base_off + 4`. Non-commutative
/// arg-setup (`k - a`, shifts) is rejected inside `select_text` (fail closed),
/// honoring the CLAUDE.md correctness boundary; `a + k`/`a - k` fold to `addi`.
pub fn int_tail_call_text(
    func: &IlFunction,
    base_off: u32,
    mode: OptMode,
) -> Result<(Vec<u8>, u32), BackendError> {
    let mut text = select_text(func, mode)?; // arg-setup + trailing blr
    let blr = encode_blr();
    debug_assert!(
        text.ends_with(&blr),
        "select_text always terminates in blr; the arg-setup is everything before it"
    );
    text.truncate(text.len() - blr.len()); // drop blr; the value stays live in r3
    let branch_off = base_off + text.len() as u32;
    text.extend_from_slice(&encode_tail_branch(branch_off));
    Ok((text, branch_off))
}

/// Lower an **argument permutation** for a multi-argument tail call: emit the
/// register moves that put each argument register's wanted value in place, with
/// `r11` as the single break temp.
///
/// `sources[i]` is the parameter index whose value argument slot `i` wants, so
/// slot `i` must end up holding what `ARG_REGS[sources[i]]` holds on entry.
/// `sources` being the identity is the passthrough case and emits nothing.
///
/// The rule, from the captures in `fixtures/cpp/il_call_multi.cpp`: decompose the
/// permutation into cycles; for each cycle save the source of its **lowest**
/// destination into the temp, then assign along the cycle in the order forced by
/// clobbering, filling that lowest destination from the temp last.
///
/// Only a **single** non-trivial cycle is accepted. Two disjoint cycles do both
/// saves up front (r11 then r10) and then have several clobber-free orders to
/// choose between, and the one capture available does not determine which — see
/// `rev4` in that fixture. A repeated argument is also refused: `dup3` emits a
/// *dead* `mr r11,r4`, which no live-value-driven solver would produce.
pub fn permute_args_text(sources: &[usize]) -> Result<Vec<u8>, BackendError> {
    permute_args_parts(sources).map(|(text, _)| text)
}

/// [`permute_args_text`] plus **the registers its moves write**, which Class B
/// needs in order to decide whether a callee-saved copy has to be hoisted in
/// front of the marshalling (`c2_il`'s `plan_saved_gprs`). It is one function
/// returning two views of one cycle decomposition rather than a second walk of
/// the same permutation: a write set derived independently would be the "two
/// implementations of one rule" shape `docs/GAPS.md` §6 #9 records, and this one
/// cannot drift from the bytes because it is computed beside them.
fn permute_args_parts(sources: &[usize]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    if sources.len() > ARG_REGS.len() {
        return Err(out_of_class(
            "more arguments than the eight register slots: the rest are \
             stack-homed; out of class",
        ));
    }
    // A value wanted by two slots is a duplicate, which emits a dead move.
    for (i, s) in sources.iter().enumerate() {
        if sources[..i].contains(s) {
            return Err(out_of_class(
                "an argument value is passed twice: c2 emits a dead `mr` through \
                 the temp for this shape; out of class",
            ));
        }
        if *s >= ARG_REGS.len() {
            return Err(out_of_class("argument sources a stack-homed parameter"));
        }
    }

    // Cycle-decompose. `sources[i] == i` is a fixed point and needs no move.
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        // Walk destination -> its source, collecting the cycle.
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            at = sources[at];
            // A source outside `sources`' own index range cannot close a cycle;
            // that is a permutation of a larger set than we were given.
            if at >= n {
                return Err(out_of_class(
                    "argument permutation references a slot outside the argument \
                     list; out of class",
                ));
            }
        }
        cycles.push(cycle);
    }

    if cycles.is_empty() {
        return Ok((Vec::new(), Vec::new())); // passthrough
    }
    if cycles.len() > 1 {
        return Err(out_of_class(
            "argument permutation has two or more disjoint cycles: c2 hoists both \
             saves (r11 then r10) and then has several clobber-free orders to pick \
             from, which one capture does not determine; out of class",
        ));
    }

    // **Only up to a three-element cycle.** Past three, c2 abandons the minimal
    // single-temp walk below: it hoists a *second* save into r10 and writes the
    // destinations in a different order. `return a4(c,d,b,a)` is
    //
    //   mr r11,r5 ; mr r10,r6 ; mr r6,r3 ; mr r5,r4 ; mr r4,r10 ; mr r3,r11
    //
    // — six moves and two temps against this function's five and one. Measured
    // over complete grids, not sampled: all 24 four-argument permutations and all
    // 84 single cycles of length 2–5 in a five-argument call give 0 mismatches at
    // lengths 2 and 3, 10 of 30 at length 4 and 16 of 24 at length 5. It was a
    // **live wrong-bytes emit on mainline** and no fixture reached it, because
    // `il_call_perm.cpp` and `il_call_multi.cpp` between them hold no cycle longer
    // than three. The order c2 picks past three is not characterized, so the
    // boundary is the measured edge rather than a fit.
    //
    // The primary gate is `c2_il`'s (`call-arg-long-cycle`), so the census and the
    // emitter agree; this is the backstop.
    if cycles[0].len() > 3 {
        return Err(out_of_class(
            "argument permutation has a cycle longer than three: past three c2 \
             hoists a second save into r10 and reorders the writes, and which \
             order it picks is not characterized; out of class",
        ));
    }
    // One cycle. Its lowest destination is filled from the temp, last.
    let cycle = &cycles[0];
    let lowest = *cycle.iter().min().expect("non-empty cycle");
    let reg = |slot: usize| ARG_REGS[slot];
    let mut t = Vec::new();
    let mut writes = vec![SCRATCH_REG];
    t.extend_from_slice(&encode_mr(SCRATCH_REG, reg(sources[lowest])));
    // Walk backwards from `lowest`: each step writes a destination whose old
    // value has already been consumed. This is the unique clobber-free order,
    // and it runs in whichever direction the cycle happens to go — which is why
    // `rot3` emits r4-then-r5 and `rot3b` emits r5-then-r4.
    let mut dst = sources[lowest];
    while dst != lowest {
        t.extend_from_slice(&encode_mr(reg(dst), reg(sources[dst])));
        writes.push(reg(dst));
        dst = sources[dst];
    }
    t.extend_from_slice(&encode_mr(reg(lowest), SCRATCH_REG));
    writes.push(reg(lowest));
    Ok((t, writes))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    #[test]
    fn encode_tail_branch_stores_negative_self_offset() {
        // Tail-call `b` displacement = −(own .text offset): offset 0 → 0x48000000,
        // offset 8 → 0x4BFFFFF8 (the REL24 reloc patches the target).
        assert_eq!(encode_tail_branch(0), [0x48, 0x00, 0x00, 0x00]);
        assert_eq!(encode_tail_branch(8), [0x4B, 0xFF, 0xFF, 0xF8]);
    }

    #[test]
    fn encode_call_branch_sets_link_bit() {
        // `bl` at offset 0xC → disp −0xC, LK=1 → 0x4BFFFFF5 (reference `bl g`).
        assert_eq!(encode_call_branch(0x0C), [0x4B, 0xFF, 0xFF, 0xF5]);
    }

    #[test]
    fn framed_call_text_matches_reference_body() {
        let plain = FrameLayout::default();
        // `int f(int a){ return g(a) + 1; }` — the verified 0x24-byte body. `a` is
        // already in r3, so the argument setup is empty.
        let b = framed_call_text(&[], 1, 0, plain).unwrap();
        assert_eq!(
            b.text,
            vec![
                0x7D, 0x88, 0x02, 0xA6, // mflr r12
                0x91, 0x81, 0xFF, 0xF8, // stw  r12,-8(r1)
                0x94, 0x21, 0xFF, 0xA0, // stwu r1,-96(r1)
                0x4B, 0xFF, 0xFF, 0xF5, // bl   g (REL24 @ 0xC)
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x38, 0x21, 0x00, 0x60, // addi r1,r1,96
                0x81, 0x81, 0xFF, 0xF8, // lwz  r12,-8(r1)
                0x7D, 0x88, 0x03, 0xA6, // mtlr r12
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
        assert_eq!((b.bl_offset, b.prolog_len), (0x0C, 0x0C));
        // `+ 2` differs only in the addi immediate.
        assert_eq!(framed_call_text(&[], 2, 0, plain).unwrap().text[19], 0x02);
        // Placed at 0x08 in a packed `.text` (a leaf ahead of it), the `bl` is at
        // 0x14 and its displacement follows: `4BFFFFED`, not `4BFFFFF5`. Every
        // other byte of the body is unchanged.
        let at8 = framed_call_text(&[], 1, 0x08, plain).unwrap();
        assert_eq!(&at8.text[12..16], &[0x4B, 0xFF, 0xFF, 0xED]);
        assert_eq!(&at8.text[..12], &b.text[..12]);
        assert_eq!(&at8.text[16..], &b.text[16..]);
        assert_eq!(at8.bl_offset, 0x14);
    }

    /// The argument-setup word, against the reference obj it was missing from.
    ///
    /// `int f(int a,int b){ return g(b) + 1; }` at `/Ox /GS- /c`, `.text` = 40
    /// bytes (10 words), `.pdata` `40000a03` (FuncLen 10, PrologLen 3), REL24 at
    /// 0x10. The port emitted the 9-word body with no `or` and every downstream
    /// field to match — a live wrong-bytes emit on mainline, and the reason
    /// [`Selected::Framed`] carries a setup at all.
    #[test]
    fn framed_call_moves_a_non_first_formal_into_r3() {
        let setup = encode_mr(RET_REG, 4);
        let b = framed_call_text(&setup, 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(b.text.len(), 0x28);
        assert_eq!(&b.text[12..16], &[0x7C, 0x83, 0x23, 0x78]); // or r3,r4,r4
        assert_eq!(&b.text[16..20], &[0x4B, 0xFF, 0xFF, 0xF1]); // bl, disp −0x10
        assert_eq!((b.bl_offset, b.prolog_len), (0x10, 0x0C));
        // …and from r5, the three-formal case (`or r3,r5,r5` = 7ca32b78).
        let b5 = framed_call_text(&encode_mr(RET_REG, 5), 1, 0, FrameLayout::default()).unwrap();
        assert_eq!(&b5.text[12..16], &[0x7C, 0xA3, 0x2B, 0x78]);
    }

    #[test]
    fn int_tail_call_passthrough_is_bare_branch() {
        // `return g(a)` — a is already in r3, so no arg setup: a bare `b g` at
        // offset 0 (`48000000`), reloc site 0 — byte-identical to the void
        // tail call. Verified against the live obj (.text=48000000, REL24 @0x0).
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309)]);
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
        assert_eq!(text, vec![0x48, 0x00, 0x00, 0x00]);
        assert_eq!(reloc, 0);
    }

    #[test]
    fn int_tail_call_arg_setup_prepends_addi() {
        // `return g(a + 1)` — the argument a+1 computed into r3 (`addi r3,r3,1`),
        // then `b g` at offset 4 (`4bfffffc`, disp −4), reloc site 0x4. Verified
        // against the live obj (.text=386300014bfffffc, REL24 @0x4).
        let f = func_with(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add],
        );
        let (text, reloc) = int_tail_call_text(&f, 0, OptMode::Ox).unwrap();
        assert_eq!(
            text,
            vec![
                0x38, 0x63, 0x00, 0x01, // addi r3,r3,1
                0x4B, 0xFF, 0xFF, 0xFC, // b g (disp −4)
            ]
        );
        assert_eq!(reloc, 4);
    }

    // ---- multi-argument tail-call permutations ------------------------------

    #[test]
    fn argument_permutations_match_the_reference() {
        // Captured from `int g3(int,int,int)` call sites; `sources[i]` is the
        // parameter index that argument slot i wants.
        // Passthrough: the parameters are already placed, so no moves at all.
        assert!(permute_args_text(&[0, 1]).unwrap().is_empty());
        assert!(permute_args_text(&[0, 1, 2]).unwrap().is_empty());

        // g3(b,a,c) — swap r3/r4, r5 untouched.
        assert_eq!(
            permute_args_text(&[1, 0, 2]).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(a,c,b) — swap r4/r5. The temp still takes the source of the cycle's
        // LOWEST destination (r4's), not r3's, which is not in the cycle at all.
        assert_eq!(
            permute_args_text(&[0, 2, 1]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7D, 0x64, 0x5B, 0x78, // mr r4,r11
            ]
        );
        // g3(c,b,a) — swap r3/r5, r4 untouched.
        assert_eq!(
            permute_args_text(&[2, 1, 0]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // The two 3-cycles run in OPPOSITE directions, so their middle moves come
        // out in opposite orders. Fixing either order as "the rule" mis-emits the
        // other, which is why both are pinned.
        // g3(b,c,a): r3<-r4, r4<-r5, r5<-r3.
        assert_eq!(
            permute_args_text(&[1, 2, 0]).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0xA4, 0x2B, 0x78, // mr r4,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(c,a,b): r3<-r5, r4<-r3, r5<-r4.
        assert_eq!(
            permute_args_text(&[2, 0, 1]).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
    }

    #[test]
    fn argument_permutations_refuse_the_uncharacterized_shapes() {
        // Two disjoint cycles (`g4(d,c,b,a)`): c2 hoists both saves into r11 and
        // r10 and then picks one of several clobber-free orders. One capture does
        // not determine which.
        assert!(permute_args_text(&[3, 2, 1, 0]).is_err());
        // A repeated argument (`g3(b,a,b)`) emits a dead `mr r11,r4` that no
        // live-value-driven solver would produce.
        assert!(permute_args_text(&[1, 0, 1]).is_err());
        // More arguments than register slots.
        assert!(permute_args_text(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).is_err());
    }

}
