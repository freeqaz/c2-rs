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
use crate::codegen::encode::{encode_addi, encode_blr, encode_mr};
use crate::codegen::frame::FrameLayout;
use c2_il::LINK_FIRST_SLOT;
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
        // A synthetic operand-stream carrier, never a function: `select_text`
        // reads `params` and `ops` and nothing else, and the label counter never
        // sees this value.
        eh_bare: false,
        eh_unwind_callees: Vec::new(),
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

/// **WCL — a CHAIN LINK's argument setup**: the marshalling for a call whose
/// slot 0 is already filled, because its receiver is the previous call's result
/// and a `bl` has just left it in r3.
///
/// It disagrees with [`moves_descending`] — the rule every other call in this
/// file uses — on the one thing they both have an opinion about, and the
/// disagreement is **measured on both sides of the same probe TU**
/// (`work/WCL/probe/p2.cpp`, `p3.cpp`, `/O1 /GS- /c`):
///
/// ```text
///   a call whose list starts at slot 0 — DESCENDING destination
///     void f(int a,int b,int c){ v1(a); g2(c,b); }   mr r4,r31 ; mr r3,r30
///     void f(int a,int b,int c){ v1(a); g2(b,c); }   mr r4,r30 ; mr r3,r31
///     void f(int a,int b){ v1(a); g3(a,b,5); }       li r5,5 ; mr r4,r30 ; mr r3,r31
///     int  f(O* p,int j){ v1(j); return p->gia(j); } mr r4,r30 ; mr r3,r31
///
///   a chain link, whose list starts at slot 1 — ASCENDING destination
///     int f(O* p,int j,int k){ return p->Next()->gia2(j,k); }
///       … mr r31,r4 ; mr r30,r5 ; bl ?Next ; mr r4,r31 ; mr r5,r30 ; bl ?gia2
///     int f(O* p,int j,int k){ return p->Next()->gia2(k,j); }
///       …                        bl ?Next ; mr r4,r30 ; mr r5,r31
///     int f(O* p,int j,int k){ return p->Next()->gia3(j,5,k); }
///       …                        bl ?Next ; mr r4,r31 ; li r5,5 ; mr r6,r30
/// ```
///
/// The fourth free-function row is the one that matters, and it is why this is
/// not "member calls go the other way": it **is** a member call, its `this` is
/// saved, its slot 0 therefore needs an instruction, and it comes out
/// descending with everything else. The axis is whether the argument list
/// starts at slot 0, not whether there is a receiver.
///
/// Literals interleave in the same order rather than being grouped, in both
/// families — `li r5,5` sits between the two moves in the third row above and
/// between them again in the last. That is the reason this walks the slots once
/// instead of emitting the moves and then the constants.
///
/// No cycle machinery: the sources are the callee-saved file (`r31`/`r30`) and
/// the destinations are argument registers, two disjoint sets, so nothing can
/// clobber anything and a value wanted **twice** is simply written twice
/// (`p->Next()->gia2(j,j)` → `mr r4,r31 ; mr r5,r31`, captured).
fn link_setup_text(
    link: &[c2_il::SlotArg],
    saved_reg: impl Fn(usize) -> Option<u8>,
) -> Result<Vec<u8>, BackendError> {
    let mut w = Vec::with_capacity(4 * link.len());
    for (i, a) in link.iter().enumerate() {
        // Ascending slot order IS the emission order; the loop is the rule.
        let dst = *ARG_REGS.get(LINK_FIRST_SLOT + i).ok_or_else(|| {
            out_of_class("a chain link's argument past the eight register slots")
        })?;
        match a {
            c2_il::SlotArg::Formal(pi) => {
                let src = saved_reg(*pi).ok_or_else(|| {
                    out_of_class("a chain link reads a formal that is not callee-saved")
                })?;
                w.extend_from_slice(&encode_mr(dst, src));
            }
            // `li rD,k` is `addi rD,0,k` — the same encoder the leaf selector's
            // bare constant goes through, at a register it cannot name.
            c2_il::SlotArg::Lit(k) => {
                let k = i16::try_from(*k)
                    .map_err(|_| out_of_class("a chain link's literal wider than an addi immediate"))?;
                w.extend_from_slice(&encode_addi(dst, 0, k));
            }
        }
    }
    Ok(w)
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
        // **WCL** — a chain link is its own arm, and it is asked FIRST because
        // the two arms below both key on Class A/Class B, and a link's lowering
        // is the same in either: its arguments come out of the callee-saved file
        // when there are any and out of nowhere when there are not. The Class A
        // chain with a literal link argument (`p->Next()->gia(7)` → `li r4,7`)
        // is exactly the case that would take the wrong arm otherwise, and it
        // would take it silently, into `li r3,7`.
        let mut setup = if let Some(link) = &c.link_args {
            if i == 0 {
                // A link is never the first call: its receiver is a previous
                // call's result. The IL parser builds it that way; this is the
                // backstop, because the arm below would then emit no saves.
                return Err(out_of_class("a chain link in the first call position"));
            }
            link_setup_text(link, saved_reg)?
        } else if i == 0 || seq.saved.is_empty() {
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
                    // A framed sequence call's slots are all formals by
                    // construction — `c2_il`'s `seq_call_arg_sources` is what
                    // refuses a literal on the way in.
                    let (t, w) = permute_args_parts(&formal_slots(sources))?;
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
        // **WCO — `return p->a()->b()->m;`**: the last call left a pointer in r3
        // and the body reads through it. One `lwz r3,off(r3)`, in exactly the
        // position the `CallValue` post-op's `addi` occupies.
        //
        // **It does NOT fold at offset 0**, and that is the one rule this tail
        // does not share with its sibling: `CallValue { add_k: 0 }` above emits
        // nothing because `r3 + 0` is already the answer, while `*(r3 + 0)` is a
        // memory read that has to happen. MEASURED, two functions one line apart
        // in `work/WCO/probe/p1.cpp` at `/O1 /GS- /c` — `c_off0` is a 40-byte
        // body ending `lwz r3,0(r3)` and `c_addr0` is a 36-byte body with no
        // instruction between the `bl` and the epilogue.
        //
        // The width is 4 by construction (the IL parser admits only a 4-byte
        // integer or a 4-byte pointer here and names every other width); a
        // narrow or 8-byte member is `lbz`/`lhz`/`ld` and a `float` one is
        // `lfs` into f1, which is a different register file.
        c2_il::SeqTail::CallLoad { off } => {
            let d = i16::try_from(off).map_err(|_| {
                out_of_class("call-sequence tail load offset exceeds a 16-bit displacement")
            })?;
            crate::codegen::encode::encode_lwz(RET_REG, RET_REG, d).to_vec()
        }
        // **WFL — the same read-through whose member is floating point**: one
        // `lfs`/`lfd` into **f1**, the other register file. Delegated to
        // [`crate::codegen::leaf::float::chain_result_fp_load_text`] rather than
        // encoded here, beside the FP leaf's register model and for the same
        // reason `fp_tail_call_text` lives there: what the instruction decides is
        // that the destination is f1 and not r3. The base register is this
        // sequence's contribution — the last `bl` left the pointer in r3.
        //
        // The TU-level half of this tail is `_fltused`, produced by
        // `c2_il::IlFunction::touches_floating_point` and consumed by
        // `coff::Function::is_float`; a body that reached here without it emits
        // an obj one symbol short.
        c2_il::SeqTail::CallLoadFp { off, double } => {
            crate::codegen::leaf::float::chain_result_fp_load_text(RET_REG, off, double)?
        }
        // **WCB/WCR — `return a->m() <rel> b->n();`**, the register-register
        // comparison spines (`docs/CMP_PRODUCES_A_VALUE.md` reading 4). All three
        // — `==`, signed order, unsigned order — live in
        // [`crate::codegen::leaf::compare::cmp_of_two_call_results`] beside the
        // *leaf* comparison spines they share their temp-allocation rule with,
        // rather than here beside the call sequence they share a frame with.
        //
        // The operand roles are **not** the emission order: the first call's
        // result is in `result_reg` and the second's is still in r3, and which of
        // those is the source's left operand is `lhs_first`. c2 orders the calls
        // by the order c1xx NUMBERED their receivers (`this` last, whatever
        // register it is in) and keeps the spine's operands in source order, so
        // both facts are needed and neither implies the other.
        c2_il::SeqTail::Cmp { cmp, lhs_first } => {
            crate::codegen::leaf::compare::cmp_of_two_call_results(cmp, lhs_first, result_reg, mode)?
        }
        // **WEC — `return <a callee-saved formal>;`**: one `mr r3, rSaved`, in
        // exactly the position `CallValue`'s `addi` post-op occupies.
        //
        // The register is re-derived from `seq.saved` through the same
        // `saved_reg` closure every other consumer here uses, and a formal that
        // is not saved is an error rather than a guessed register: this tail
        // exists precisely because the value predates the last `bl`, so reading
        // it out of an argument register would be reading a clobbered one.
        c2_il::SeqTail::SavedFormal { param } => {
            let src = saved_reg(param).ok_or_else(|| {
                out_of_class("the tail returns a formal that is not callee-saved")
            })?;
            moves_descending(&[(RET_REG, src)])
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
pub fn permute_args_text(sources: &[c2_il::SlotArg]) -> Result<Vec<u8>, BackendError> {
    permute_args_parts(sources).map(|(text, _)| text)
}

/// The slot list of a call whose arguments are all formals, for the two callers
/// that cannot spell a literal: a **framed** sequence call's `arg_sources`, whose
/// interleaving with the callee-saved copies is measured only for formals.
fn formal_slots(sources: &[usize]) -> Vec<c2_il::SlotArg> {
    sources.iter().map(|&i| c2_il::SlotArg::Formal(i)).collect()
}

/// **WLA — the literal argument slots of a tail call**, `li r<3+i>,k` each.
///
/// Reached only when every non-literal slot is the formal already sitting in its
/// own argument register (`c2_il`'s `lit_arg_tail_call` is the primary gate;
/// the `in_place` check below is the backstop), so there is no move to schedule
/// against and the whole setup is the constants.
///
/// **The order is DESCENDING destination**, the same rule
/// [`moves_descending`] uses and the opposite of a chain link's. Captured,
/// `work/WLA/probe/p1.cpp` at `/O1 /GS- /c`:
///
/// ```text
///   void f(int a,int b) { g3(a, b, 7); }   38a00007  li 5,7
///   void f(int a)       { g3(a, 5, 6); }   38a00006  li 5,6
///                                          38800005  li 4,5     <- 6 before 5
/// ```
fn lit_slots_text(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    let in_place = slots.iter().enumerate().all(|(i, a)| match a {
        c2_il::SlotArg::Lit(_) => true,
        c2_il::SlotArg::Formal(pi) => *pi == i,
    });
    if !in_place {
        return Err(out_of_class(
            "a literal argument beside a formal that has to move: the moves and \
             the `li` interleave, and the same list over a real permutation cycle \
             — where the r11 break temp wants a slot in that order too — is not \
             characterized",
        ));
    }
    let mut w = Vec::new();
    let mut writes = Vec::new();
    // Descending destination: walk the slots from the top.
    for (i, a) in slots.iter().enumerate().rev() {
        let c2_il::SlotArg::Lit(k) = a else { continue };
        let dst = *ARG_REGS.get(i).ok_or_else(|| {
            out_of_class("a literal argument past the eight register slots")
        })?;
        let k = i16::try_from(*k)
            .map_err(|_| out_of_class("a literal argument wider than an addi immediate"))?;
        // `li rD,k` is `addi rD,0,k` — the same encoder the leaf selector's bare
        // constant goes through, at a register it cannot name.
        w.extend_from_slice(&encode_addi(dst, 0, k));
        writes.push(dst);
    }
    Ok((w, writes))
}

/// [`permute_args_text`] plus **the registers its moves write**, which Class B
/// needs in order to decide whether a callee-saved copy has to be hoisted in
/// front of the marshalling (`c2_il`'s `plan_saved_gprs`). It is one function
/// returning two views of one cycle decomposition rather than a second walk of
/// the same permutation: a write set derived independently would be the "two
/// implementations of one rule" shape `docs/GAPS.md` §6 #9 records, and this one
/// cannot drift from the bytes because it is computed beside them.
fn permute_args_parts(slots: &[c2_il::SlotArg]) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
    // **WLA** — a list carrying a literal is asked FIRST, because everything
    // below reads a slot as a formal index and a `Lit` has none. The two forms
    // do not mix in class (`c2_il`'s `lit_arg_tail_call` admits a literal only
    // beside formals that are already in place), so this is a dispatch and not a
    // precedence.
    if slots.iter().any(|a| matches!(a, c2_il::SlotArg::Lit(_))) {
        return lit_slots_text(slots);
    }
    let mut sources = Vec::with_capacity(slots.len());
    for a in slots {
        match a {
            c2_il::SlotArg::Formal(i) => sources.push(*i),
            // Unreachable by the dispatch above, stated as a refusal rather than
            // an `unreachable!` because the CLI must degrade cleanly.
            c2_il::SlotArg::Lit(_) => {
                return Err(out_of_class("a literal argument reached the permutation walk"))
            }
        }
    }
    let sources = &sources[..];
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

    /// **WCO — the chain-result designator, and the one place its two forms
    /// disagree.**
    ///
    /// `p->a()->b()->m` is `lwz r3,off(r3)` and `&p->a()->b()->m` is
    /// `addi r3,r3,off`. At `off == 0` the add folds to nothing and the load
    /// does NOT — measured, `c_off0` (40 B) against `c_addr0` (36 B) in
    /// `work/WCO/probe/p1.cpp` at `/O1 /GS- /c`. Both directions are pinned,
    /// because the tempting simplification is to give `CallLoad` the same
    /// `add_k: 0` fold its sibling has.
    #[test]
    fn the_chain_tail_load_does_not_fold_at_offset_zero_but_the_add_does() {
        let seq = |tail| c2_il::CallSeq {
            calls: vec![
                c2_il::SeqCall { callee: "?a@@YAPAUM@@XZ".into(), arg_ops: vec![IlOp::Load(9)], arg_sources: None, link_args: None },
                c2_il::SeqCall { callee: "?b@@YAPAUM@@XZ".into(), arg_ops: Vec::new(), arg_sources: None, link_args: Some(Vec::new()) },
            ],
            tail,
            saved: Vec::new(),
        };
        let tail_of = |t| {
            call_seq_parts(&[9], &seq(t), OptMode::O1).expect("in class").1
        };
        // `lwz r3,4(r3)` / `lwz r3,0(r3)` — the load is emitted at both offsets.
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: 4 }), vec![0x80, 0x63, 0x00, 0x04]);
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: 0 }), vec![0x80, 0x63, 0x00, 0x00]);
        // `addi r3,r3,4` — and NOTHING at 0.
        assert_eq!(tail_of(c2_il::SeqTail::CallValue { add_k: 4 }), vec![0x38, 0x63, 0x00, 0x04]);
        assert_eq!(tail_of(c2_il::SeqTail::CallValue { add_k: 0 }), Vec::<u8>::new());
        // A negative displacement is representable and is not a fold either.
        assert_eq!(tail_of(c2_il::SeqTail::CallLoad { off: -4 }), vec![0x80, 0x63, 0xFF, 0xFC]);
        // Past the signed-16-bit displacement it refuses rather than truncating.
        // The IL parser gates this; the second lock is here.
        assert!(call_seq_parts(&[9], &seq(c2_il::SeqTail::CallLoad { off: 0x8000 }), OptMode::O1).is_err());
    }

    /// **WFL — the same designator step in the other register file.**
    ///
    /// The words are read off the reference obj (`work/WFL/probe/p1.cpp`,
    /// `/O1 /GS- /c`) and the destination is `f1`, not r3 — which is the whole
    /// reason this is a variant and not a width flag on `CallLoad`. It does not
    /// fold at 0 either, for the same reason its integer sibling does not.
    #[test]
    fn the_chain_tail_fp_load_is_an_lfs_or_an_lfd_into_f1() {
        let seq = |tail| c2_il::CallSeq {
            calls: vec![
                c2_il::SeqCall { callee: "?a@@YAPAUM@@XZ".into(), arg_ops: vec![IlOp::Load(9)], arg_sources: None, link_args: None },
                c2_il::SeqCall { callee: "?b@@YAPAUM@@XZ".into(), arg_ops: Vec::new(), arg_sources: None, link_args: Some(Vec::new()) },
            ],
            tail,
            saved: Vec::new(),
        };
        let tail_of = |t| call_seq_parts(&[9], &seq(t), OptMode::O1).expect("in class").1;
        // `lfs f1,4(r3)` = c0230004 and `lfd f1,16(r3)` = c8230010 — the two
        // cells `c_f` and `c_d` compile to, one primary opcode apart.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 4, double: false }),
            vec![0xC0, 0x23, 0x00, 0x04]
        );
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 16, double: true }),
            vec![0xC8, 0x23, 0x00, 0x10]
        );
        // No fold at 0: `*(r3 + 0)` is a memory read that has to happen.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 0, double: false }),
            vec![0xC0, 0x23, 0x00, 0x00]
        );
        // `lfd` is D-form, NOT the DS-form the integer `ld` is, so an 8-byte
        // load at a displacement that is not a multiple of 4 encodes fine and
        // needs none of the alignment gate the load leaf carries.
        assert_eq!(
            tail_of(c2_il::SeqTail::CallLoadFp { off: 6, double: true }),
            vec![0xC8, 0x23, 0x00, 0x06]
        );
        // Past the signed-16-bit displacement it refuses rather than truncating.
        assert!(
            call_seq_parts(&[9], &seq(c2_il::SeqTail::CallLoadFp { off: 0x8000, double: false }), OptMode::O1)
                .is_err()
        );
    }

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
        assert!(permute_args_text(&formal_slots(&[0, 1])).unwrap().is_empty());
        assert!(permute_args_text(&formal_slots(&[0, 1, 2])).unwrap().is_empty());

        // g3(b,a,c) — swap r3/r4, r5 untouched.
        assert_eq!(
            permute_args_text(&formal_slots(&[1, 0, 2])).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(a,c,b) — swap r4/r5. The temp still takes the source of the cycle's
        // LOWEST destination (r4's), not r3's, which is not in the cycle at all.
        assert_eq!(
            permute_args_text(&formal_slots(&[0, 2, 1])).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7D, 0x64, 0x5B, 0x78, // mr r4,r11
            ]
        );
        // g3(c,b,a) — swap r3/r5, r4 untouched.
        assert_eq!(
            permute_args_text(&formal_slots(&[2, 1, 0])).unwrap(),
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
            permute_args_text(&formal_slots(&[1, 2, 0])).unwrap(),
            vec![
                0x7C, 0x8B, 0x23, 0x78, // mr r11,r4
                0x7C, 0xA4, 0x2B, 0x78, // mr r4,r5
                0x7C, 0x65, 0x1B, 0x78, // mr r5,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
        // g3(c,a,b): r3<-r5, r4<-r3, r5<-r4.
        assert_eq!(
            permute_args_text(&formal_slots(&[2, 0, 1])).unwrap(),
            vec![
                0x7C, 0xAB, 0x2B, 0x78, // mr r11,r5
                0x7C, 0x85, 0x23, 0x78, // mr r5,r4
                0x7C, 0x64, 0x1B, 0x78, // mr r4,r3
                0x7D, 0x63, 0x5B, 0x78, // mr r3,r11
            ]
        );
    }

    // ---- WCL: a chain link's marshalling runs the OTHER way ------------------

    /// The two orders, side by side, from the two halves of one probe TU.
    ///
    /// This is the whole rung in one assertion. Both bodies are Class B with the
    /// same two saved formals in the same two registers; they differ only in
    /// whether argument slot 0 belongs to the call or to the `bl` in front of it,
    /// and c2 emits the moves in opposite orders because of it. A single
    /// `moves_descending` for both — which is what "reuse the shipped rule, it is
    /// the same marshalling" would have produced — is byte-wrong on every chain
    /// link that carries two or more arguments.
    #[test]
    fn a_chain_links_moves_ascend_where_every_other_calls_descend() {
        let saved = |pi: usize| [Some(31u8), Some(30u8)].get(pi.wrapping_sub(1)).copied().flatten();
        // `int f(O* p,int j,int k){ return p->Next()->gia2(j,k); }`
        //   … bl ?Next ; mr r4,r31 ; mr r5,r30 ; bl ?gia2
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(1), c2_il::SlotArg::Formal(2)], saved).unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xC5, 0xF3, 0x78, // mr r5,r30
            ]
        );
        // …and the same slots wanting the other values: still ascending, so the
        // ORDER is not a function of which register the value is in.
        // `gia2(k,j)` → `mr r4,r30 ; mr r5,r31`.
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(2), c2_il::SlotArg::Formal(1)], saved).unwrap(),
            vec![
                0x7F, 0xC4, 0xF3, 0x78, // mr r4,r30
                0x7F, 0xE5, 0xFB, 0x78, // mr r5,r31
            ]
        );
        // The shipped rule for a list that starts at slot 0 — captured from
        // `void f(int a,int b,int c){ v1(a); g2(c,b); }` — goes the other way,
        // and it still does.
        assert_eq!(
            moves_descending(&[(RET_REG, 30), (4, 31)]),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xC3, 0xF3, 0x78, // mr r3,r30
            ]
        );
    }

    /// A literal link argument is `li r<slot>,k` **in slot order**, interleaved
    /// with the moves rather than grouped before or after them. Captured:
    /// `int f(O* p,int j,int k){ return p->Next()->gia3(j,5,k); }` is
    /// `mr r4,r31 ; li r5,5 ; mr r6,r30`.
    #[test]
    fn a_chain_links_literals_interleave_in_slot_order() {
        let saved = |pi: usize| [Some(31u8), Some(30u8)].get(pi.wrapping_sub(1)).copied().flatten();
        assert_eq!(
            link_setup_text(
                &[
                    c2_il::SlotArg::Formal(1),
                    c2_il::SlotArg::Lit(5),
                    c2_il::SlotArg::Formal(2)
                ],
                saved
            )
            .unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x38, 0xA0, 0x00, 0x05, // li r5,5
                0x7F, 0xC6, 0xF3, 0x78, // mr r6,r30
            ]
        );
        // A value wanted TWICE is two ordinary moves — the sources are the
        // callee-saved file and the destinations the argument one, so there is no
        // cycle to break and no dead `mr r11`.
        // `p->Next()->gia2(j,j)` → `mr r4,r31 ; mr r5,r31`.
        assert_eq!(
            link_setup_text(&[c2_il::SlotArg::Formal(1), c2_il::SlotArg::Formal(1)], saved).unwrap(),
            vec![
                0x7F, 0xE4, 0xFB, 0x78, // mr r4,r31
                0x7F, 0xE5, 0xFB, 0x78, // mr r5,r31
            ]
        );
        // Slot 0 is the receiver, so seven explicit arguments fill r4..r10 and an
        // eighth would be stack-homed. The IL parser draws the same bound
        // (`mcall-chain-link-arg-overflow`); this is the backstop.
        let seven = vec![c2_il::SlotArg::Lit(1); 7];
        assert_eq!(link_setup_text(&seven, saved).unwrap().len(), 28);
        let eight = vec![c2_il::SlotArg::Lit(1); 8];
        assert!(link_setup_text(&eight, saved).is_err());
    }

    /// **WLA — a literal argument is one `li` and no move, and the order of two
    /// of them is DESCENDING**, which is the opposite of a chain link's.
    ///
    /// Both halves are read off `work/WLA/probe/p1.cpp` at `/O1 /GS- /c`. The
    /// single-literal rows agree with either order, so the two-literal one is
    /// the whole assertion: emitting them ascending is byte-wrong on every call
    /// that carries more than one constant.
    #[test]
    fn a_literal_call_argument_is_one_li_and_two_of_them_descend() {
        use c2_il::SlotArg::{Formal, Lit};
        // `void f(int a,int b){ g3(a, b, 7); }` -> `li 5,7` (38a00007).
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Lit(7)]).unwrap(),
            vec![0x38, 0xA0, 0x00, 0x07]
        );
        // `void f(int a){ g2(a, 5); }` -> `li 4,5` (38800005).
        assert_eq!(
            permute_args_text(&[Formal(0), Lit(5)]).unwrap(),
            vec![0x38, 0x80, 0x00, 0x05]
        );
        // `void f(int a){ g3(a, 5, 6); }` -> `li 5,6` THEN `li 4,5`.
        assert_eq!(
            permute_args_text(&[Formal(0), Lit(5), Lit(6)]).unwrap(),
            vec![
                0x38, 0xA0, 0x00, 0x06, // li 5,6
                0x38, 0x80, 0x00, 0x05, // li 4,5
            ]
        );
        // Negative and zero immediates are the same encoder.
        assert_eq!(
            permute_args_text(&[Formal(0), Formal(1), Lit(-1)]).unwrap(),
            vec![0x38, 0xA0, 0xFF, 0xFF]
        );
        // A formal that has to MOVE beside the literal: refused here as well as
        // in the IL parser, because `g3(a,7,b)` is `mr r5,r4 ; li r4,7` and the
        // same list over a real cycle is not characterized at all.
        assert!(permute_args_text(&[Formal(0), Lit(7), Formal(1)]).is_err());
        assert!(permute_args_text(&[Formal(1), Formal(0), Lit(7)]).is_err());
        // Past the eight register slots an argument is stack-homed.
        let nine = [
            Formal(0), Formal(1), Formal(2), Formal(3), Formal(4), Formal(5),
            Formal(6), Formal(7), Lit(1),
        ];
        assert!(permute_args_text(&nine).is_err());
        // …and a literal wider than the `addi` immediate. The IL parser draws
        // the same bound (`call-arg-lit-wide`); this is the backstop.
        assert!(permute_args_text(&[Formal(0), Lit(70000)]).is_err());
    }

    #[test]
    fn argument_permutations_refuse_the_uncharacterized_shapes() {
        // Two disjoint cycles (`g4(d,c,b,a)`): c2 hoists both saves into r11 and
        // r10 and then picks one of several clobber-free orders. One capture does
        // not determine which.
        assert!(permute_args_text(&formal_slots(&[3, 2, 1, 0])).is_err());
        // A repeated argument (`g3(b,a,b)`) emits a dead `mr r11,r4` that no
        // live-value-driven solver would produce.
        assert!(permute_args_text(&formal_slots(&[1, 0, 1])).is_err());
        // More arguments than register slots.
        assert!(permute_args_text(&formal_slots(&[0, 1, 2, 3, 4, 5, 6, 7, 8])).is_err());
    }

}
