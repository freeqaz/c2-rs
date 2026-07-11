//! Minimal IL parse for the MVP function classes: a straight-line all-`int`
//! left-associative arithmetic leaf (`int add3(int,int,int)` and friends), a
//! bare terminal void tail call (`void f(){ g(); }`), and a framed non-leaf
//! `return g(a) + k`. This is deliberately NOT a general IL disassembler.
//!
//! **Acceptance is a positive whole-body parse (W4b2-v).** [`parse_segment`]
//! tokenizes the entire `.ex` operand stream of a function segment — from the
//! `4C 4F 11` ('LO') marker to the segment end — and accepts only if the whole
//! token sequence is exactly one of the three recognized [`BodyShape`]s; the
//! parse must *reach the end*, so trailing statements, a second call, a
//! non-trivial call-argument region, or any unmodeled byte fail the function
//! closed (`None` → the caller reports `NotImplemented`, never a mis-emit).
//! This replaced an earlier trio of gates that each matched on a *local* byte
//! neighborhood around the first CALL and so silently over-accepted (two
//! reviews caught the same two functions dropping trailing/in-argument work).
//!
//! Three `.gl`/`.ex` facts drive the emitter, per `ILPARSE` spec:
//!   * the mangled function name(s) (from `.gl`) — copied verbatim into the COFF
//!     symbol + string table (also the external callee name for call shapes);
//!   * the source path (from `.gl`) — provenance only, not embedded in the MVP
//!     obj;
//!   * the body shape (from `.ex`) — a LOAD/ADD op stream, a tail call, or a
//!     framed call, which codegen lowers to PPC.
//!
//! Reference decoder mirrored: `dc3-decomp/msvc-src/tools/il_parser.py`
//! (`ILGlobals`, `_detect_token_width`, `ILFunction._parse_body`);
//! grammar cross-checked against live-toolchain `.ex` captures of every fixture.

use crate::IlBundle;

/// A single straight-line IL operation in the integer-arithmetic class.
///
/// The binary ops are postfix (each pops two operands, pushes one result).
/// `Sub` is **non-commutative** — its operand order is load-bearing (see the
/// codegen for the `subf` operand mapping); `Add`/`Mul` are commutative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IlOp {
    /// Load a named variable (by IL token) onto the expression stack.
    Load(u16),
    /// Push an integer literal constant (IL opcode `0x33`, `<type> <varint>`).
    Lit(i32),
    /// Pop rhs then lhs, push `lhs + rhs` (IL opcode `0x02`, commutative).
    Add,
    /// Pop rhs then lhs, push `lhs - rhs` (IL opcode `0x03`, NON-commutative).
    Sub,
    /// Pop rhs then lhs, push `lhs * rhs` (IL opcode `0x04`, commutative).
    Mul,
}

/// A **framed non-leaf call** of the verified `return g(a) + k` class (W4b2):
/// the call result is consumed (so `f` allocates a stack frame and is non-leaf),
/// then a small integer literal `k` is added and returned. Codegen emits the
/// constant 0x24-byte frame (prologue, `bl <callee>`, `addi r3,r3,k`, epilogue)
/// plus the `.pdata` unwind record — see `c2_core::codegen`/`coff`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FramedCall {
    /// The single external callee's mangled name (from `.gl`), e.g. `?g@@YAHH@Z`.
    pub callee: String,
    /// The post-call `+ k` literal (`k` fits a signed 16-bit `addi` immediate;
    /// commutative, so no non-commutative opt-in gate is needed).
    pub add_k: i32,
}

/// A parsed MVP function: enough to drive the codegen + COFF emitter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IlFunction {
    /// Mangled name, e.g. `?add3@@YAHHHH@Z` (verbatim from `.gl`).
    pub mangled_name: String,
    /// Source path from `.gl`, e.g. `z:\...\mvp_add3.cpp` (provenance only).
    pub source_path: Option<String>,
    /// Formal-parameter IL tokens, in declaration order (a, b, c → r3, r4, r5).
    pub params: Vec<u16>,
    /// Straight-line body op stream (loads + adds). Empty for a tail/framed call.
    pub ops: Vec<IlOp>,
    /// If this function is a **tail call** to a single external, its mangled
    /// name (the callee). Codegen then emits a `b <callee>` with a REL24
    /// relocation instead of an arithmetic body. W4a: single external only.
    pub tail_call: Option<String>,
    /// If this function is a **framed non-leaf call** (`return g(a) + k`), the
    /// callee + post-op literal. Distinct from `tail_call` (which is a bare
    /// `b g`). W4b2: single-function TU, single external only.
    pub framed_call: Option<FramedCall>,
}

/// The int type encoding inline in the `.ex` body (`86 41 74`), per `IL_FORMAT`.
const INT_TYPE: [u8; 3] = [0x86, 0x41, 0x74];

/// Real token-width detector (ports `il_parser._detect_token_width`): find the
/// first `4F 02`, count the bytes to the next `4F`. That gap is the token
/// width. Defaults to 2 if the anchor is not found.
pub fn detect_token_width(ex: &[u8]) -> usize {
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == 0x4F && ex[i + 1] == 0x02 {
            let mut j = i + 2;
            while j < ex.len() && ex[j] != 0x4F {
                j += 1;
            }
            let gap = j - (i + 2);
            if gap == 2 || gap == 4 {
                return gap;
            }
        }
        i += 1;
    }
    2
}

/// Extract the mangled name from `.gl`: the first `?`-prefixed, NUL-terminated
/// ASCII run whose second byte is alphabetic and which contains `@@` (the
/// `__cdecl`/global marker). Mirrors `ILGlobals._parse`.
pub fn mangled_name(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    while i < gl.len() {
        if gl[i] == b'?' {
            // Read to NUL.
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.len() >= 3
                && bytes[1].is_ascii_alphabetic()
                && contains_subslice(bytes, b"@@")
                && bytes.iter().all(|b| b.is_ascii_graphic())
            {
                return Some(String::from_utf8_lossy(bytes).into_owned());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    None
}

/// Extract **all** mangled names from `.gl`, in file order — one per function
/// in the translation unit. Same acceptance test as [`mangled_name`]; used for
/// multi-function TUs where `.gl` carries a name per function.
pub fn mangled_names(gl: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < gl.len() {
        if gl[i] == b'?' {
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.len() >= 3
                && bytes[1].is_ascii_alphabetic()
                && contains_subslice(bytes, b"@@")
                && bytes.iter().all(|b| b.is_ascii_graphic())
            {
                out.push(String::from_utf8_lossy(bytes).into_owned());
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Extract the source path from `.gl`: a `<letter>:\…\<name>.cpp` NUL-terminated
/// ASCII run (case-insensitive drive + `.cpp` suffix). Provenance only.
pub fn source_path(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    while i + 2 < gl.len() {
        if gl[i].is_ascii_alphabetic() && gl[i + 1] == b':' && gl[i + 2] == b'\\' {
            let start = i;
            let mut end = i;
            while end < gl.len() && gl[end] != 0 {
                end += 1;
            }
            let bytes = &gl[start..end];
            if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                let s = String::from_utf8_lossy(bytes).into_owned();
                if s.to_ascii_lowercase().ends_with(".cpp") {
                    return Some(s);
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    None
}

fn contains_subslice(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || hay.len() < needle.len() {
        return needle.is_empty();
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

/// Read a big-endian IL token of `tw` bytes as a u16 (MVP tokens are 2 bytes,
/// e.g. `e3 09` → `0xE309`). Returns `None` if out of range or `tw != 2`.
fn read_token(ex: &[u8], p: usize, tw: usize) -> Option<u16> {
    if tw != 2 || p + 2 > ex.len() {
        return None;
    }
    Some(((ex[p] as u16) << 8) | ex[p + 1] as u16)
}

fn find_byte(hay: &[u8], b: u8) -> Option<usize> {
    hay.iter().position(|&x| x == b)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The fixed 6-byte callee-reference tail of a CALL token: `BD <3-byte return
/// type> 00 80 01 10 00 00`. Verified identical across int and void calls, so
/// it anchors the end of the 10-byte CALL token.
const CALL_CALLEE_ANCHOR: [u8; 6] = [0x00, 0x80, 0x01, 0x10, 0x00, 0x00];

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BodyShape {
    /// Straight-line all-`int` arithmetic leaf (`return a+b+c`, `return a+5`,
    /// `return 42`, …): a postfix LOAD/LIT/ADD/SUB/MUL stream returning `int`.
    StraightLine { params: Vec<u16>, ops: Vec<IlOp> },
    /// Bare terminal void tail call (`void f(){ g(); }`): exactly one CALL whose
    /// void result is discarded, with **nothing** after its `4C 4B` void
    /// call-end but the return plumbing → codegen emits a single `b <callee>`.
    VoidTailCall,
    /// Framed non-leaf `return g(a) + k`: exactly one int-returning CALL whose
    /// argument region is exactly the single passthrough LOAD, a `55 <int>`
    /// call-end, then exactly one literal `+ k` (ADD), returned.
    FramedCall { add_k: i32 },
}

/// Advance `*p` past `pat` iff the stream matches it there; return whether it
/// did. The single primitive the positive parser is built on — every grammar
/// token is consumed through an `eat` (fixed pattern) or a typed read, so an
/// unrecognized byte anywhere fails the whole parse closed.
fn eat(seg: &[u8], p: &mut usize, pat: &[u8]) -> bool {
    if seg.len() >= *p + pat.len() && &seg[*p..*p + pat.len()] == pat {
        *p += pat.len();
        true
    } else {
        false
    }
}

fn eat_byte(seg: &[u8], p: &mut usize, x: u8) -> bool {
    if seg.get(*p) == Some(&x) {
        *p += 1;
        true
    } else {
        false
    }
}

/// Consume an optional `4F 01 NN` statement/label marker (a per-statement
/// sequence index c1xx emits in multi-function TUs, absent in single-function
/// ones). Specific to `4F 01` — it never eats the `4F 12` separator or the
/// `4F 02` module marker.
fn eat_opt_stmt_marker(seg: &[u8], p: &mut usize) {
    if seg.get(*p) == Some(&0x4F) && seg.get(*p + 1) == Some(&0x01) && *p + 2 < seg.len() {
        *p += 3;
    }
}

/// Read an int literal varint `33 <int-type> <varint>` payload (the `33
/// <int-type>` prefix already consumed): a single byte if `< 0x80` (the value),
/// else `0x80` + a 4-byte LE i32. (Verified: 5→`05`, 42→`2a`, 200→`80 c8000000`,
/// 70000→`80 70110100`.) Any other lead byte → `None`.
fn read_varint(seg: &[u8], p: &mut usize) -> Option<i32> {
    let marker = *seg.get(*p)?;
    if marker < 0x80 {
        *p += 1;
        Some(marker as i32)
    } else if marker == 0x80 {
        let v = i32::from_le_bytes([
            *seg.get(*p + 1)?,
            *seg.get(*p + 2)?,
            *seg.get(*p + 3)?,
            *seg.get(*p + 4)?,
        ]);
        *p += 5;
        Some(v)
    } else {
        None
    }
}

/// Consume the shared statement/function-tail plumbing that follows the body
/// expression of *every* accepted shape, and require the parse to reach the end
/// of the segment (the fail-closed terminal — anything trailing rejects). With
/// `has_result_type`, a `41 <int-type>` result annotation is expected first
/// (present for an int return, absent for a void call). Layout (verified):
/// `[41 <int>]?` result-type · `3A <tok>` assign · `[4F 01 NN]?` · `54 02 29
/// <tok>` return · `4F 12` · `47 54 01 54 00` GT-terminate · then EITHER the
/// segment end (a non-last function, split before the next `4F 1F`) OR the
/// module end `4F 02 20 00 · 4F 01 NN · 4D` and trailing zero-fill (the last
/// function).
fn eat_return_plumbing(seg: &[u8], p: &mut usize, tw: usize, has_result_type: bool) -> Option<()> {
    if has_result_type && !eat(seg, p, &[0x41, INT_TYPE[0], INT_TYPE[1], INT_TYPE[2]]) {
        return None;
    }
    // ASSIGN: 3A <tok>
    if !eat_byte(seg, p, 0x3A) {
        return None;
    }
    read_token(seg, *p, tw)?;
    *p += tw;
    eat_opt_stmt_marker(seg, p);
    // RETURN: 54 02 29 <tok>
    if !eat(seg, p, &[0x54, 0x02, 0x29]) {
        return None;
    }
    read_token(seg, *p, tw)?;
    *p += tw;
    // Function-tail: 4F 12 · 47 54 01 54 00
    if !eat(seg, p, &[0x4F, 0x12]) || !eat(seg, p, &[0x47, 0x54, 0x01, 0x54, 0x00]) {
        return None;
    }
    // A non-last function's segment ends exactly here (the split cuts before the
    // next `4F 1F`). Otherwise the last function carries the module end.
    if *p == seg.len() {
        return Some(());
    }
    if !eat(seg, p, &[0x4F, 0x02, 0x20, 0x00]) || !eat(seg, p, &[0x4F, 0x01]) {
        return None;
    }
    *p += 1; // module label index NN
    if !eat_byte(seg, p, 0x4D) {
        return None;
    }
    // Trailing zero-fill to the end of `.ex`.
    while seg.get(*p) == Some(&0) {
        *p += 1;
    }
    if *p == seg.len() {
        Some(())
    } else {
        None
    }
}

/// Consume the postfix LOAD/LIT/ADD/SUB/MUL operand stream of a straight-line
/// arithmetic leaf, stopping (without consuming) at the `41` result-type marker
/// that begins the return plumbing. Fail-closed: any byte that is not a modeled
/// operand/opcode (a comparison `24`, shift `09`, bitwise `0B`, ternary `43 42`,
/// …) rejects the whole function. Requires at least one op.
fn parse_arith(seg: &[u8], p: &mut usize, tw: usize) -> Option<Vec<IlOp>> {
    let mut ops = Vec::new();
    loop {
        match *seg.get(*p)? {
            0xB9 => {
                // LOAD <token> <int-type>
                *p += 1;
                let tok = read_token(seg, *p, tw)?;
                *p += tw;
                if !eat(seg, p, &INT_TYPE) {
                    return None; // non-int operand → out of class
                }
                ops.push(IlOp::Load(tok));
            }
            0x33 => {
                // LITERAL: 33 <int-type> <varint>
                *p += 1;
                if !eat(seg, p, &INT_TYPE) {
                    return None; // non-int literal → out of class
                }
                ops.push(IlOp::Lit(read_varint(seg, p)?));
            }
            0x02 => {
                *p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                *p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                *p += 1;
                ops.push(IlOp::Mul);
            }
            // The `41` result-type annotation begins the return plumbing.
            0x41 => break,
            _ => return None,
        }
    }
    if ops.is_empty() {
        None
    } else {
        Some(ops)
    }
}

/// Parse the formal-parameter list of a straight-line leaf: after the `46` ('F')
/// marker (before the `LO` marker), a run of `2D <token>` entries emitted in
/// *reverse* of declaration order. An empty list is legitimate (a zero-param
/// `int konst(){return 42;}` still emits `46` immediately before `LO`).
fn parse_formals(seg: &[u8], lo: usize, tw: usize) -> Option<Vec<u16>> {
    let f = find_byte(&seg[..lo], 0x46)?;
    let mut p = f + 1;
    let mut rev = Vec::new();
    while seg.get(p) == Some(&0x2D) {
        p += 1;
        let tok = read_token(seg, p, tw)?;
        p += tw;
        rev.push(tok);
    }
    rev.reverse();
    Some(rev)
}

/// **The positive whole-body parser (W4b2-v).** Parse a single `.ex` function
/// segment as *exactly one* of the three recognized [`BodyShape`]s, tokenizing
/// the entire operand stream from the `4C 4F 11` ('LO') marker to the end of the
/// segment. Acceptance is by a complete positive match — every token is
/// consumed through a fixed-pattern `eat` or a typed read, and the parse must
/// reach the segment end — so a second CALL, any computation after a terminal
/// call, a non-trivial call-argument region, or any unmodeled byte fails the
/// whole function closed (`None` → the caller reports `NotImplemented`). This
/// replaces the earlier trio of neighborhood-scanning gates (`parse_body`,
/// `is_tail_call`, `parse_framed_call`) that each accepted on a *local* byte
/// pattern and so over-accepted trailing/second-call computation.
///
/// Grammar (verified against live-toolchain captures of every fixture + probe):
/// ```text
///   body   := 'LO'(4C 4F 11) 'SS'(53) stmt?  ( arith | vcall | fcall )
///   stmt   := 4F 01 NN                                    (multi-fn only)
///   arith  := (LOAD | LIT | 02|03|04)+  <return int>       LOAD:=B9 tok INT
///   vcall  := 26 tok  CALL  4C 4B  <return void>           LIT :=33 INT varint
///   fcall  := 26 tok  CALL  LOAD  55 INT 4C  33 INT k 02  <return int>
///   CALL   := BD <3-byte ret type> 00 80 01 10 00 00       (fixed 10 bytes)
/// ```
/// `<return …>` is the shared plumbing consumed by [`eat_return_plumbing`]
/// (result-type for int, then assign/return/tail/segment-or-module end). The
/// framed callee argument region is exactly the single passthrough LOAD; the
/// post-op is exactly one literal + ADD (commutative) whose `k` fits a signed
/// 16-bit `addi` — `* k`/`- k`/wide `k`/a second literal/a second call all
/// reject. The `callee` name is not in `.ex`; the caller pairs it from `.gl`.
fn parse_segment(seg: &[u8], tw: usize) -> Option<BodyShape> {
    let lo = find_subslice(seg, &[0x4C, 0x4F, 0x11])?;
    let mut p = lo + 3;
    // 'SS' statement-start, then an optional statement/label marker.
    if !eat_byte(seg, &mut p, 0x53) {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);

    match *seg.get(p)? {
        // Call shapes both open with a `26 <tok>` function/result-temp ref.
        0x26 => parse_call_shape(seg, &mut p, tw),
        // Straight-line arithmetic opens with a LOAD or a bare literal.
        0xB9 | 0x33 => {
            let ops = parse_arith(seg, &mut p, tw)?;
            eat_return_plumbing(seg, &mut p, tw, true)?;
            let params = parse_formals(seg, lo, tw)?;
            Some(BodyShape::StraightLine { params, ops })
        }
        _ => None,
    }
}

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call or the framed `return g(a) + k`. See
/// [`parse_segment`] for the grammar; fail-closed at every step.
fn parse_call_shape(seg: &[u8], p: &mut usize, tw: usize) -> Option<BodyShape> {
    // 26 <tok> function/result ref.
    if !eat_byte(seg, p, 0x26) {
        return None;
    }
    read_token(seg, *p, tw)?;
    *p += tw;
    // The fixed 10-byte CALL token: BD <3-byte return type> <anchor>.
    if !eat_byte(seg, p, 0xBD) {
        return None;
    }
    *p += 3; // 3-byte return type (void=82 07 03, int=86 41 74); anchor pins it
    if !eat(seg, p, &CALL_CALLEE_ANCHOR) {
        return None;
    }

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type). `g();g();` and `g();return a+1;` fail here — a
    // second `26` call or a `B9` statement stands where the return plumbing must.
    if eat(seg, p, &[0x4C, 0x4B]) {
        eat_return_plumbing(seg, p, tw, false)?;
        return Some(BodyShape::VoidTailCall);
    }

    // FRAMED int call: the argument region is EXACTLY one passthrough LOAD, then
    // the `55 <int>` call-end and its `4C` marker.
    if !eat_byte(seg, p, 0xB9) {
        return None; // no/complex arg region we do not model
    }
    read_token(seg, *p, tw)?;
    *p += tw;
    if !eat(seg, p, &INT_TYPE) {
        return None;
    }
    // Anything other than the `55 <int> 4C` call-end here (a second arg LOAD, an
    // in-argument literal like `g(a+1)`) means an argument region we do not
    // model → reject.
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) || !eat_byte(seg, p, 0x4C) {
        return None;
    }
    // Post-op: EXACTLY one literal `33 <int> k` immediately followed by ADD.
    // A second call (`g(a)+g(1)` → `26 …`), a second literal (`g(a)+1+2` → a
    // second `33 …`), or SUB/MUL (`03`/`04`) all fail one of these `eat`s.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    let k = read_varint(seg, p)?;
    if !eat_byte(seg, p, 0x02) {
        return None; // non-ADD post-op → non-commutative / strength-reduced
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return None;
    }
    eat_return_plumbing(seg, p, tw, true)?;
    Some(BodyShape::FramedCall { add_k: k })
}

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
const FN_START: [u8; 2] = [0x4F, 0x1F];

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
fn split_functions(ex: &[u8]) -> Vec<&[u8]> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == FN_START[0] && ex[i + 1] == FN_START[1] {
            starts.push(i);
            i += 2;
        } else {
            i += 1;
        }
    }
    let mut segs = Vec::with_capacity(starts.len());
    for k in 0..starts.len() {
        let end = if k + 1 < starts.len() { starts[k + 1] } else { ex.len() };
        segs.push(&ex[starts[k]..end]);
    }
    segs
}

impl IlBundle {
    /// Parse this bundle as a sequence of straight-line add-chain functions
    /// (the MVP class, generalized to a multi-function TU). Returns `None` if
    /// the required files are absent, or if the `.gl` name count does not match
    /// the `.ex` function count, or if ANY function body is outside the class
    /// (the caller — `PortC2` — then reports `NotImplemented` for the whole TU).
    ///
    /// Names come from `.gl` in file order; bodies from `.ex` split at each
    /// `4F 1F`. The two are paired positionally.
    pub fn functions(&self) -> Option<Vec<IlFunction>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let names = mangled_names(gl);
        let src = source_path(gl);
        let tw = detect_token_width(ex);
        let segs = split_functions(ex);
        if segs.is_empty() || names.len() < segs.len() {
            return None;
        }
        // `.gl` lists the defined functions first (one per `.ex` segment, paired
        // positionally), then any external callees.
        let n_defined = segs.len();
        let externals = &names[n_defined..];

        // Calls (externals present) are handled only for a **single-function
        // TU with a single external** — the positive parse ([`parse_segment`])
        // recognizes exactly two call shapes:
        //   * W4b2 framed non-leaf `return g(a) + k` → the 6-section framed obj;
        //   * W4a bare terminal tail call `void f(){ g(); }` → single `b g`.
        // The callee name is not in `.ex`; it is paired from the single `.gl`
        // external. Scope is single-function only: the `.pdata` label counters
        // ($M2545/$M2546/$T2547) are a fixed toolchain seed for the first
        // function but shift when preceding functions consume slots (W-UNW-1
        // probe, docs/CODEGEN_PPC_MVP.md), so a multi-function TU that contains
        // a call is rejected here rather than mis-numbered.
        if !externals.is_empty() {
            if n_defined == 1 && externals.len() == 1 {
                match parse_segment(segs[0], tw)? {
                    BodyShape::FramedCall { add_k } => {
                        return Some(vec![IlFunction {
                            mangled_name: names[0].clone(),
                            source_path: src,
                            params: Vec::new(),
                            ops: Vec::new(),
                            tail_call: None,
                            framed_call: Some(FramedCall {
                                callee: externals[0].clone(),
                                add_k,
                            }),
                        }]);
                    }
                    BodyShape::VoidTailCall => {
                        return Some(vec![IlFunction {
                            mangled_name: names[0].clone(),
                            source_path: src,
                            params: Vec::new(),
                            ops: Vec::new(),
                            tail_call: Some(externals[0].clone()),
                            framed_call: None,
                        }]);
                    }
                    // A `.gl` external but a straight-line body is a contradiction
                    // (no call to bind the external to) → reject.
                    BodyShape::StraightLine { .. } => return None,
                }
            }
            return None;
        }

        // No externals: every function must be a straight-line arithmetic leaf
        // (W1–W3). A call shape with no external to bind is rejected.
        let mut funcs = Vec::with_capacity(n_defined);
        for (name, seg) in names.iter().take(n_defined).zip(segs) {
            match parse_segment(seg, tw)? {
                BodyShape::StraightLine { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                    });
                }
                _ => return None,
            }
        }
        Some(funcs)
    }

    /// Parse this bundle as a SINGLE MVP function. Convenience wrapper over
    /// [`IlBundle::functions`]; returns `None` unless the TU has exactly one
    /// in-class function.
    pub fn mvp_function(&self) -> Option<IlFunction> {
        let mut fs = self.functions()?;
        if fs.len() == 1 {
            fs.pop()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mangled_name_from_gl_slice() {
        let gl = b"\x00\x00?add3@@YAHHHH@Z\x00trailing";
        assert_eq!(mangled_name(gl).as_deref(), Some("?add3@@YAHHHH@Z"));
    }

    #[test]
    fn mangled_name_rejects_stray_question_mark() {
        // `?` not followed by an alpha / no `@@`.
        assert_eq!(mangled_name(b"? not a name\x00"), None);
    }

    #[test]
    fn source_path_from_gl_slice() {
        let gl = b"\x12\x20\x00z:\\tmp\\ilcap\\mvp.cpp\x00\x10";
        assert_eq!(
            source_path(gl).as_deref(),
            Some("z:\\tmp\\ilcap\\mvp.cpp")
        );
    }

    #[test]
    fn token_width_two_from_4f02_gap() {
        // `4F 02 20 00 4F` → gap 2.
        let ex = [0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01];
        assert_eq!(detect_token_width(&ex), 2);
    }

    #[test]
    fn mangled_names_collects_all_in_order() {
        let gl = b"\x00?add2@@YAHHH@Z\x00pad\x00?add4@@YAHHHHH@Z\x00";
        assert_eq!(
            mangled_names(gl),
            vec!["?add2@@YAHHH@Z".to_string(), "?add4@@YAHHHHH@Z".to_string()]
        );
    }

    // ---- positive whole-body parser (W4b2-v) --------------------------------
    //
    // Every fixture below is a REAL `.ex` function segment captured from the
    // live 16.00.11886.00 toolchain (`/Bd /d2nop /Ox /GS- /c`), transcribed from
    // the `4F 1F` split point. Straight-line segments include the `46` formals
    // marker; call segments start at the `LO` marker (call shapes carry no
    // formal list). Each accepted segment is a *last* function, so it ends at
    // the module end `… 4F 02 20 00 4F 01 NN 4D` — the parser must reach it.

    #[test]
    fn parse_segment_accepts_straight_line_add3() {
        // `int add3(int a,int b,int c){ return a+b+c; }` (mvp_add3, single fn).
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD c
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type int
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // separator + GT terminate
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // module end
        ];
        assert_eq!(
            parse_segment(seg, 2),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409, 0xE509], // a, b, c
                ops: vec![
                    IlOp::Load(0xE309),
                    IlOp::Load(0xE409),
                    IlOp::Add,
                    IlOp::Load(0xE509),
                    IlOp::Add,
                ],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_literal_return_and_wide() {
        // `int konst(){ return 42; }` — empty formal list (`46` then `LO`), a
        // bare literal, and the multi-function statement markers `4F 01 NN`.
        let konst: &[u8] = &[
            0x46, // formals marker, empty list
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0E, // LO SS + stmt marker
            0x33, 0x86, 0x41, 0x74, 0x2A, // LIT 42
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0F, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xEA, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x10,
            0x4D,
        ];
        assert_eq!(
            parse_segment(konst, 2),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(42)],
            })
        );
        // `int kw(){ return 70000; }` — the wide (`0x80` + 4-byte LE) varint.
        let kw: &[u8] = &[
            0x46, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x0D, // formals/LO/stmt
            0x33, 0x86, 0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, // LIT 70000 (wide)
            0x41, 0x86, 0x41, 0x74, 0x3A, 0xEA, 0x09, 0x4F, 0x01, 0x0E, 0x54, 0x02, 0x29, 0xEA,
            0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01,
            0x0F, 0x4D,
        ];
        assert_eq!(
            parse_segment(kw, 2),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(70000)],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_nonlast_function_reaching_segment_end() {
        // `int add2(int a,int b){ return a+b; }` as the FIRST function of a
        // multi-fn TU: the segment is split before the next `4F 1F`, so it ends
        // right after `47 54 01 54 00` (no module end). The parse must accept by
        // reaching that segment end, not by finding a module marker.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals b,a
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x07, // LO SS + stmt marker
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE6, 0x09, 0x4F, 0x01, 0x08, // ASSIGN + stmt marker
            0x54, 0x02, 0x29, 0xE6, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // GT terminate = segment end
        ];
        assert_eq!(
            parse_segment(seg, 2),
            Some(BodyShape::StraightLine {
                params: vec![0xE309, 0xE409],
                ops: vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Add],
            })
        );
    }

    #[test]
    fn parse_segment_accepts_bare_void_tail_call() {
        // `void f(){ g(); }` (mvp_call): exactly one void call, `4C 4B`, then
        // only the return plumbing → a bare `b g` tail call.
        assert_eq!(parse_segment(MVP_CALL, 2), Some(BodyShape::VoidTailCall));
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(MVP_FRAMED, 2),
            Some(BodyShape::FramedCall { add_k: 1 })
        );
    }

    #[test]
    fn parse_segment_rejects_all_out_of_class_call_shapes() {
        // The four W4b2-i probes plus the five W4b2-v defects — each a real
        // captured segment the positive parse must reject at the parser level
        // (→ None → NotImplemented), never mis-emit. Named by their `.cpp`.
        let cases: &[(&str, &[u8])] = &[
            ("g(a) - 1 (submod)", GA_SUBMOD),
            ("g(a) * 5 (mulmod)", GA_MULMOD),
            ("g(a) + 70000 (widemod)", GA_WIDEMOD),
            ("g(a + 1) (argframed)", GA_ARGFRAMED),
            ("g(); g(); (two_calls)", TWO_CALLS),
            ("g(); return a+1; (call_then_stmt)", CALL_THEN_STMT),
            ("g(a + 1) + 1 (argframed_plusk)", ARGFRAMED_PLUSK),
            ("g(a) + g(a + 1) (two_framed_calls)", TWO_FRAMED_CALLS),
            ("g(a) + 1 + 2 (plus1plus2)", PLUS1PLUS2),
        ];
        for (label, seg) in cases {
            assert_eq!(parse_segment(seg, 2), None, "must reject: {label}");
        }
    }

    #[test]
    fn parse_segment_rejects_unmodeled_arithmetic_ops() {
        // add3.cpp seg with a comparison/ternary (`24` GT, `43 42` CB) — the
        // parser must fail closed on the first unmodeled opcode, not skip it.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, // formals
            0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10, // LO SS stmt
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, // LOAD
            0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74, // LOAD
            0x24, // GT — unmodeled → reject
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        assert_eq!(parse_segment(cmp, 2), None);
    }

    // ---- real captured segments (transcribed from live-toolchain `.ex`) -----

    /// `void f(){ g(); }` — accepted bare void tail call.
    const MVP_CALL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1; }` — accepted framed call (k=1).
    const MVP_FRAMED: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08,
        0x4D,
    ];
    /// `return g(a) - 1;` — non-commutative post-op (SUB) → reject.
    const GA_SUBMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x03, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x09,
        0x4D,
    ];
    /// `return g(a) * 5;` — strength-reduced post-op (MUL) → reject.
    const GA_MULMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x05, 0x04, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07,
        0x4D,
    ];
    /// `return g(a) + 70000;` — wide post-op immediate → reject.
    const GA_WIDEMOD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09,
        0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20,
        0x00, 0x4F, 0x01, 0x07, 0x4D,
    ];
    /// `return g(a + 1);` — in-argument arithmetic (LOAD+LIT+ADD before `55`)
    /// → the argument region is not the bare passthrough LOAD → reject.
    const GA_ARGFRAMED: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55,
        0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7,
        0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0A,
        0x4D,
    ];
    /// `void f(){ g(); g(); }` — a SECOND call stands where the void tail call's
    /// return plumbing must be → reject (defect #1).
    const TWO_CALLS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ g(); return a + 1; }` — a second statement follows the
    /// void call's `4C 4B` (a `B9` LOAD where the return plumbing must be) →
    /// reject (defect #2).
    const CALL_THEN_STMT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4C, 0x4B, 0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a + 1) + 1; }` — in-argument arithmetic AND a
    /// framed post-op: the arg region carries LOAD+LIT+ADD before `55` → reject
    /// (defect #3; a naive post-`55` search would mis-accept as framed g(a)+1).
    const ARGFRAMED_PLUSK: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55,
        0x86, 0x41, 0x74, 0x4C, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];
    /// `int f(int a){ return g(a) + g(a + 1); }` — a SECOND call follows the
    /// first call-end where the framed post-op literal must be → reject
    /// (defect #4).
    const TWO_FRAMED_CALLS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x26, 0xE4,
        0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86,
        0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x02, 0x41,
        0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 1 + 2; }` — a SECOND literal+ADD follows the
    /// framed post-op where the result-type must be → reject.
    const PLUS1PLUS2: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x01, 0x02, 0x33, 0x86, 0x41, 0x74, 0x02, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
}
