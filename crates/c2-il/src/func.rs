//! Minimal IL parse for the MVP function class: a straight-line, all-`int`
//! left-associative add chain with one return (`int add3(int,int,int)` and
//! friends). This is deliberately NOT a general IL disassembler — it decodes
//! exactly the operand/opcode shape the MVP obj needs and reports `None` for
//! anything outside that class, leaving the general codec (the `A2` workstream)
//! for later.
//!
//! Three facts are extracted, per `ILPARSE` spec:
//!   * the mangled function name (from `.gl`) — copied verbatim into the COFF
//!     symbol + string table;
//!   * the source path (from `.gl`) — provenance only, not embedded in the MVP
//!     obj;
//!   * the body operand/opcode stream (from `.ex`) — `LOAD a, LOAD b, ADD,
//!     LOAD c, ADD, …` which codegen lowers to PPC `add`s + `blr`.
//!
//! Reference decoder mirrored: `dc3-decomp/msvc-src/tools/il_parser.py`
//! (`ILGlobals`, `_detect_token_width`, `ILFunction._parse_body`).

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

/// Parse the straight-line body: the formal list after `46` ('F') and the
/// LOAD/ADD operand stream after `4C 4F 11` ('LO'). Returns
/// `(params_in_decl_order, ops)`. `None` for anything outside the MVP class.
fn parse_body(ex: &[u8], tw: usize) -> Option<(Vec<u16>, Vec<IlOp>)> {
    // --- formals: after 0x46, a run of `2D <token>` entries (reverse order) ---
    let f = find_byte(ex, 0x46)?;
    let mut p = f + 1;
    let mut formals_rev = Vec::new();
    while p < ex.len() && ex[p] == 0x2D {
        p += 1;
        let tok = read_token(ex, p, tw)?;
        p += tw;
        formals_rev.push(tok);
    }
    // An empty formal list is legitimate — a zero-parameter function like
    // `int konst(){return 42;}` still emits the `46` marker, immediately
    // followed by `LO`. (Do not require ≥1 formal.)
    // The `F` list is emitted in reverse of declaration order.
    let mut params: Vec<u16> = formals_rev;
    params.reverse();

    // --- operand stream: after `4C 4F 11`, optional `53`, then B9/02 … ---
    let lo = find_subslice(ex, &[0x4C, 0x4F, 0x11])?;
    let mut p = lo + 3;
    if p < ex.len() && ex[p] == 0x53 {
        p += 1;
    }
    let mut ops = Vec::new();
    loop {
        if p >= ex.len() {
            return None;
        }
        match ex[p] {
            0xB9 => {
                // LOAD <token> <int-type>
                p += 1;
                let tok = read_token(ex, p, tw)?;
                p += tw;
                if p + INT_TYPE.len() > ex.len() || ex[p..p + INT_TYPE.len()] != INT_TYPE {
                    return None; // non-int operand → out of MVP class
                }
                p += INT_TYPE.len();
                ops.push(IlOp::Load(tok));
            }
            0x02 => {
                p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                p += 1;
                ops.push(IlOp::Mul);
            }
            // LITERAL: `33 <int-type> <varint>`. The varint is a single byte
            // when < 0x80 (the value directly); `0x80` introduces a 4-byte LE
            // i32. (Verified: 5→`05`, 42→`2a`, 200→`80 c8 00 00 00`,
            // 70000→`80 70 11 01 00`.) Other markers are unknown → out of class.
            0x33 => {
                p += 1;
                if p + INT_TYPE.len() > ex.len() || ex[p..p + INT_TYPE.len()] != INT_TYPE {
                    return None; // non-int literal → out of MVP class
                }
                p += INT_TYPE.len();
                if p >= ex.len() {
                    return None;
                }
                let marker = ex[p];
                let val: i32 = if marker < 0x80 {
                    p += 1;
                    marker as i32
                } else if marker == 0x80 {
                    if p + 5 > ex.len() {
                        return None;
                    }
                    let v = i32::from_le_bytes([ex[p + 1], ex[p + 2], ex[p + 3], ex[p + 4]]);
                    p += 5;
                    v
                } else {
                    return None; // unknown literal-width marker
                };
                ops.push(IlOp::Lit(val));
            }
            // `4F 01 NN` statement/label markers appear in the operand stream of
            // multi-function TUs (a per-statement sequence index c1xx emits);
            // they carry no codegen meaning here — skip the 3-byte marker.
            0x4F if p + 2 < ex.len() && ex[p + 1] == 0x01 => {
                p += 3;
            }
            // `53` ('S') statement-start marker — skip.
            0x53 => {
                p += 1;
            }
            // Result-type annotation (`41 <type>`) or RETURN (`54 …`) ends the
            // straight-line expression.
            0x41 | 0x54 => break,
            _ => return None,
        }
    }
    if ops.is_empty() {
        return None;
    }
    Some((params, ops))
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

/// The IL CALL opcode (`0xBD`) that introduces a function call in the `.ex`
/// operand stream (per `IL_FORMAT.md`).
const CALL_OP: u8 = 0xBD;

/// The `55 <int-type>` **call-end marker** that terminates an int-returning
/// call's descriptor+argument region. The consumed return value's post-op
/// (framed `+ k`) is emitted after it (see [`parse_framed_call`]).
const CALL_END: u8 = 0x55;

/// The fixed 6-byte callee-reference tail of a CALL token: `BD <3-byte return
/// type> 00 80 01 10 00 00`. Verified identical across int and void calls, so
/// it anchors the end of the 10-byte CALL token.
const CALL_CALLEE_ANCHOR: [u8; 6] = [0x00, 0x80, 0x01, 0x10, 0x00, 0x00];

/// The `4C 4B` **void call-end marker** that follows the CALL token of a
/// terminal `void f(){ g(); }` — no argument setup, no consumed result.
const VOID_CALL_END: [u8; 2] = [0x4C, 0x4B];

/// **Terminal** tail-call detector: after the `LO` marker, the body is a single
/// call whose value is neither consumed nor preceded by argument setup — i.e.
/// the bare `void f(){ g(); }` shape that codegen lowers to one `b <callee>`.
///
/// The CALL is a fixed 10-byte token (`BD <3-byte return type>` +
/// [`CALL_CALLEE_ANCHOR`]); a terminal void call is followed immediately by the
/// [`VOID_CALL_END`] marker and then only return plumbing. This is deliberately
/// tight (W4b2-i): the old gate accepted a CALL *anywhere* after `LO`, so
/// `g(a)-1` / `g(a)*5` / `g(a)+70000` / `g(a+1)` — all correctly refused by
/// [`parse_framed_call`] — were mis-classified as bare tail calls and emitted a
/// bare `b g`, dropping their (unmodeled) surrounding computation. Anything but
/// the exact terminal shape now returns `false` → the caller reports
/// `NotImplemented` instead of mis-emitting.
fn is_tail_call(seg: &[u8]) -> bool {
    let Some(lo) = find_subslice(seg, &[0x4C, 0x4F, 0x11]) else {
        return false;
    };
    let after = &seg[lo..];
    let Some(call) = after.iter().position(|&b| b == CALL_OP) else {
        return false;
    };
    // Require the fixed CALL token: BD <3-byte type> <CALL_CALLEE_ANCHOR>.
    let tok_end = call + 4 + CALL_CALLEE_ANCHOR.len();
    if after.get(call + 4..tok_end) != Some(&CALL_CALLEE_ANCHOR[..]) {
        return false;
    }
    // Terminal iff the void call-end marker follows immediately (no arg LOAD,
    // no `55` int call-value marker → no surrounding computation).
    after.get(tok_end..tok_end + VOID_CALL_END.len()) == Some(&VOID_CALL_END[..])
}

/// Detect the **framed non-leaf `return g(...) + k`** shape (W4b2) in a single
/// function segment and, if it matches, return `k`.
///
/// The `.ex` body for `int f(int a){ return g(a) + 1; }` (after the `LO`
/// marker) is: a `26 <tok>` result-temp, a CALL (`0xBD`) with its int return
/// type + descriptor and loaded argument(s), a `55 <int-type>` **call-end
/// marker**, then the post-op — a single integer literal `33 86 41 74 <varint>`
/// **immediately followed by an ADD** (`0x02`), then the return.
///
/// **Grammar fact (verified against real captures):** the post-call operation
/// is emitted *after* the `55 86 41 74` call-end marker; anything before it
/// belongs to the **argument** region. `int f(int a){ return g(a + 1); }` puts
/// its `+1` (`33 86 41 74 01 02`) *inside the args, before the `55` marker* —
/// so the post-op search MUST be anchored past the call-end marker or `g(a+1)`
/// is silently mis-accepted as framed `g(a)+1` (it is really a tail call with
/// arg setup, unmodeled → must land in `NotImplemented`). Captured evidence:
///   `g(a)+1`:  `… 55 86 41 74 | 4c 33 86 41 74 01 02 …`  (literal AFTER 55)
///   `g(a+1)`:  `… 33 86 41 74 01 02 | 55 86 41 74 4c 41 …` (literal BEFORE 55)
///
/// Honest, narrow acceptance (anything else → `None` → the caller reports
/// `NotImplemented`, never mis-emit):
///   * there must be a CALL after `LO`, followed by the `55 <int-type>`
///     call-end marker (an int-returning call whose value is consumed);
///   * there must be exactly one int literal `33 86 41 74 <varint>` **after the
///     call-end marker** (two literals / none → reject);
///   * the opcode **immediately after** that literal must be `0x02` (ADD). This
///     is the commutativity gate: `* k` (`0x04`) and `- k` (`0x03`) fall here
///     and are rejected — they strength-reduce / are non-commutative and change
///     the verified 0x24-byte frame (`a*5` is a 0x28-byte body);
///   * `k` must fit the signed 16-bit `addi` immediate (a wide `k` would need
///     an extra `addis`, again off the 0x24 frame).
fn parse_framed_call(seg: &[u8]) -> Option<i32> {
    let lo = find_subslice(seg, &[0x4C, 0x4F, 0x11])?;
    let after = &seg[lo..];
    // Must contain a CALL.
    let call = after.iter().position(|&b| b == CALL_OP)?;
    // Anchor the post-op search PAST the `55 <int-type>` call-end marker: any
    // literal/op before it is part of the argument region (e.g. `g(a+1)`), not
    // a framed post-op. This is the fix for the silent `g(a+1)` mis-accept.
    let call_end = [CALL_END, INT_TYPE[0], INT_TYPE[1], INT_TYPE[2]];
    let end_rel = find_subslice(&after[call..], &call_end)?;
    let post = &after[call + end_rel + call_end.len()..];

    // The int-literal marker `33 86 41 74` is specific enough to anchor on.
    let litpat = [0x33, INT_TYPE[0], INT_TYPE[1], INT_TYPE[2]];
    let lit = find_subslice(post, &litpat)?;
    // Exactly one literal — reject `g(a) + 1 + 2`-style multi-literal shapes.
    if find_subslice(&post[lit + litpat.len()..], &litpat).is_some() {
        return None;
    }
    let mut p = lit + litpat.len();
    let marker = *post.get(p)?;
    let k: i32 = if marker < 0x80 {
        p += 1;
        marker as i32
    } else if marker == 0x80 {
        let v = i32::from_le_bytes([
            *post.get(p + 1)?,
            *post.get(p + 2)?,
            *post.get(p + 3)?,
            *post.get(p + 4)?,
        ]);
        p += 5;
        v
    } else {
        return None; // unknown literal-width marker
    };
    // Commutativity + frame-shape gate: the op consuming the call result and the
    // literal must be ADD. (`*`/`-` → different instruction / non-commutative.)
    if *post.get(p)? != 0x02 {
        return None;
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return None;
    }
    Some(k)
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
        // TU with a single external** — two shapes:
        //   * W4b2 framed non-leaf `return g(a) + k` (checked first: it also
        //     contains a CALL, so it must win over the tail-call test); and
        //   * W4a bare tail call `void f(){ g(); }` → single `b g`.
        // Scope is single-function only: the `.pdata` label counters
        // ($M2545/$M2546/$T2547) are a fixed toolchain seed for the first
        // function but shift when preceding functions consume slots (W-UNW-1
        // probe, docs/CODEGEN_PPC_MVP.md), so a multi-function TU that contains
        // a framed call is rejected here rather than mis-numbered.
        if !externals.is_empty() {
            if n_defined == 1 && externals.len() == 1 {
                if let Some(add_k) = parse_framed_call(segs[0]) {
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
                if is_tail_call(segs[0]) {
                    return Some(vec![IlFunction {
                        mangled_name: names[0].clone(),
                        source_path: src,
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: Some(externals[0].clone()),
                        framed_call: None,
                    }]);
                }
            }
            return None;
        }

        // No externals: the straight-line arithmetic path (W1–W3).
        let mut funcs = Vec::with_capacity(n_defined);
        for (name, seg) in names.iter().take(n_defined).zip(segs) {
            let (params, ops) = parse_body(seg, tw)?;
            funcs.push(IlFunction {
                mangled_name: name.clone(),
                source_path: src.clone(),
                params,
                ops,
                tail_call: None,
                framed_call: None,
            });
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
    fn parse_body_decodes_literals() {
        // `a + 5`: LOAD a, LIT(int,5), ADD. And a 4-byte literal `80 c8000000`
        // = 200 to exercise the wide varint form.
        let small: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, // formals a
            0x4C, 0x4F, 0x11, 0x53, // LO S
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0x33, 0x86, 0x41, 0x74, 0x05, // LIT 5
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, 0x54, // result-type ends
        ];
        let (params, ops) = parse_body(small, 2).unwrap();
        assert_eq!(params, vec![0xE309]);
        assert_eq!(ops, vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Add]);

        let wide: &[u8] = &[
            0x46, 0x2D, 0xE3, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74,
            0x33, 0x86, 0x41, 0x74, 0x80, 0xC8, 0x00, 0x00, 0x00, // LIT 200 (wide)
            0x02, 0x41, 0x86, 0x41, 0x74, 0x54,
        ];
        let (_p, ops) = parse_body(wide, 2).unwrap();
        assert_eq!(ops, vec![IlOp::Load(0xE309), IlOp::Lit(200), IlOp::Add]);
    }

    #[test]
    fn mangled_names_collects_all_in_order() {
        let gl = b"\x00?add2@@YAHHH@Z\x00pad\x00?add4@@YAHHHHH@Z\x00";
        assert_eq!(
            mangled_names(gl),
            vec!["?add2@@YAHHH@Z".to_string(), "?add4@@YAHHHHH@Z".to_string()]
        );
    }

    #[test]
    fn parse_body_skips_multifunction_statement_markers() {
        // fn0 of the two-function bundle: note the `4F 01 02` marker between the
        // `4C 4F 11 53` LO and the first `B9` LOAD — absent in the single-fn
        // case, present in multi-fn TUs. Must be skipped, not rejected.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals b,a
            0x4C, 0x4F, 0x11, 0x53, // LO S
            0x4F, 0x01, 0x02, // statement/label marker (skip)
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result type int
            0x3A, 0xE6, 0x09, 0x54, 0x02, 0x29, 0xE6, 0x09,
        ];
        let (params, ops) = parse_body(seg, 2).unwrap();
        assert_eq!(params, vec![0xE309, 0xE409]); // a, b
        assert_eq!(ops, vec![IlOp::Load(0xE309), IlOp::Load(0xE409), IlOp::Add]);
    }

    #[test]
    fn parse_framed_call_extracts_k_from_real_body() {
        // The real `.ex` body tail of `int f(int a){ return g(a) + 1; }`, from
        // the `LO` marker: CALL (0xBD), loaded arg, call-end (0x55), then the
        // post-op literal `33 86 41 74 01` immediately followed by ADD (0x02).
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, // LO S
            0x26, 0xE4, 0x09, // result temp
            0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // CALL int
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a (arg)
            0x55, 0x86, 0x41, 0x74, // call-end marker
            0x4C, 0x33, 0x86, 0x41, 0x74, 0x01, // LIT 1
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result type int
            0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // ASSIGN + RETURN
        ];
        assert_eq!(parse_framed_call(body), Some(1));
    }

    #[test]
    fn parse_framed_call_rejects_literal_before_call_end() {
        // `int f(int a){ return g(a + 1); }` — the `+1` is INSIDE the argument
        // (literal `33 86 41 74 01` + ADD `02`) and lands BEFORE the `55` call-
        // end marker. It must NOT be read as a framed post-op (that would be
        // `g(a)+1`); it is a tail call with arg setup (unmodeled) → reject.
        // Real captured body tail of `return g(a+1)`.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, // LO S, result temp
            0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // CALL int
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a (arg)
            0x33, 0x86, 0x41, 0x74, 0x01, // LIT 1  (in-arg, BEFORE call-end)
            0x02, // ADD  (in-arg)
            0x55, 0x86, 0x41, 0x74, // call-end marker
            0x4C, 0x41, 0x86, 0x41, 0x74, // result type int
            0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // ASSIGN + RETURN
        ];
        assert_eq!(parse_framed_call(body), None);
    }

    #[test]
    fn parse_framed_call_rejects_sub_postop() {
        // `return g(a) - 1;` — identical to the accepted `g(a)+1` body except
        // the post-op is SUB (`0x03`) not ADD (`0x02`): the one-byte difference
        // c2 emits (it does NOT canonicalize `-1` to `+(-1)`). Non-commutative
        // → reject.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, //
            0x4C, 0x33, 0x86, 0x41, 0x74, 0x01, // LIT 1 (after call-end)
            0x03, // SUB — reject
            0x41, 0x86, 0x41, 0x74,
        ];
        assert_eq!(parse_framed_call(body), None);
    }

    #[test]
    fn is_tail_call_accepts_terminal_void_call() {
        // `void f(){ g(); }`: CALL token then the void call-end marker `4C 4B`
        // and only return plumbing → a bare `b g` terminal tail call.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE3, 0x09, // LO S, result temp
            0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // CALL void
            0x4C, 0x4B, // void call-end marker
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, // ASSIGN + RETURN
        ];
        assert!(is_tail_call(body));
    }

    #[test]
    fn is_tail_call_rejects_nonterminal_calls() {
        // Any call with argument setup or a consumed result is NOT a bare tail
        // call. `g(a) - 1`, `g(a) * 5`, and `g(a+1)` all have a `B9` LOAD (arg)
        // right after the CALL token instead of the `4C 4B` void marker → must
        // be rejected (→ NotImplemented), not mis-emitted as `b g`.
        let ga_minus1: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C,
            0x33, 0x86, 0x41, 0x74, 0x01, 0x03, 0x41, 0x86, 0x41, 0x74,
        ];
        let ga_times5: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C,
            0x33, 0x86, 0x41, 0x74, 0x05, 0x04, 0x41, 0x86, 0x41, 0x74,
        ];
        let ga_plus1_arg: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
            0x02, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74,
        ];
        assert!(!is_tail_call(ga_minus1));
        assert!(!is_tail_call(ga_times5));
        assert!(!is_tail_call(ga_plus1_arg));
    }

    #[test]
    fn parse_framed_call_rejects_bare_tail_call() {
        // `void f(){ g(); }`: a CALL but NO post-op literal → not a framed call
        // (must stay on the tail-call path, single `b g`). Real tc body tail.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, // LO S
            0x26, 0xE3, 0x09, // result temp
            0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // CALL void
            0x4C, 0x4B, // markers
            0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09, // ASSIGN + RETURN
        ];
        assert_eq!(parse_framed_call(body), None);
    }

    #[test]
    fn parse_framed_call_rejects_nonadd_postop() {
        // `return g(a) * 5;`: literal followed by MUL (0x04), not ADD — the op
        // strength-reduces to a 0x28-byte body, off the verified frame class.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, //
            0x4C, 0x33, 0x86, 0x41, 0x74, 0x05, // LIT 5
            0x04, // MUL — reject
            0x41, 0x86, 0x41, 0x74,
        ];
        assert_eq!(parse_framed_call(body), None);
    }

    #[test]
    fn parse_framed_call_rejects_wide_k() {
        // A wide literal (0x80 + 4-byte LE = 70000) does not fit the `addi`
        // immediate → an extra `addis` off the 0x24 frame → reject.
        let body: &[u8] = &[
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, //
            0x4C, 0x33, 0x86, 0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00, // LIT 70000 (wide)
            0x02, // ADD
        ];
        assert_eq!(parse_framed_call(body), None);
    }

    #[test]
    fn parse_add3_body_from_real_slice() {
        // The exact add3 `.ex` body tail (from _CL_*ex @0xA80..), starting at
        // the `F` formals marker.
        let body: &[u8] = &[
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO S
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD c
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result type int
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
        ];
        let (params, ops) = parse_body(body, 2).unwrap();
        assert_eq!(params, vec![0xE309, 0xE409, 0xE509]); // a, b, c
        assert_eq!(
            ops,
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
            ]
        );
    }
}
