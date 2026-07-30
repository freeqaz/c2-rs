//! Minimal IL parse for the MVP function classes: a straight-line all-`int`
//! left-associative arithmetic leaf (`int add3(int,int,int)` and friends), a
//! bare terminal void tail call (`void f(){ g(); }`), the integer tail-call
//! family `return g(<arg>)` (passthrough `g(a)`, the `g(a)+0` identity fold, and
//! arg-setup `g(a+1)`), and a framed non-leaf `return g(a) + k` (k ≠ 0). This is
//! deliberately NOT a general IL disassembler.
//!
//! **Acceptance is a positive whole-body parse (W4b2-v).** [`parse_segment`]
//! tokenizes the entire `.ex` operand stream of a function segment — from the
//! `4C 4F 11` ('LO') marker to the segment end — and accepts only if the whole
//! token sequence is exactly one of the recognized [`BodyShape`]s; the
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
    Load(u32),
    /// **Indirect** load: pop a pointer, push the 4-byte integer it designates,
    /// `off` bytes in (IL `30 <TYPE>`, optionally preceded by one byte-offset add
    /// — `27 <TYPE>` for a member or `28 00 00` for a subscript).
    ///
    /// Produced ONLY by [`try_parse_indirect_load_leaf`], and only as the second
    /// and last op of a two-op stream `[Load(base), LoadInd { off }]`. Nothing
    /// lowers it in combination with arithmetic, because c2 does not lower it
    /// that way: `*p + 1` is `lwz r11,0(r3) ; addi r3,r11,1` — the load lands in
    /// the *scratch* register, not the destination — and `*p * 3` is
    /// strength-reduced to `lwz r11 ; slwi r10,r11,1 ; add r3,r11,r10`. See
    /// `docs/IL_EXPR_LAYER.md` §6 and `fixtures/cpp/il_expr_load_neg.cpp`.
    LoadInd { off: i32 },
    /// Push an integer literal constant (IL opcode `0x33`, `<type> <varint>`).
    Lit(i32),
    /// Push a **floating-point literal** (W13b). The payload is always an
    /// IEEE-754 **binary64** bit pattern regardless of width — a `float` literal
    /// is stored as a double whose value is already rounded to float — with the
    /// width carried separately. Held as raw bits so no rounding happens here.
    FpLit { bits: u64, double: bool },
    /// Pop rhs then lhs, push `lhs + rhs` (IL opcode `0x02`, commutative).
    Add,
    /// Pop rhs then lhs, push `lhs - rhs` (IL opcode `0x03`, NON-commutative).
    Sub,
    /// Pop rhs then lhs, push `lhs * rhs` (IL opcode `0x04`, commutative).
    Mul,
    /// Pop rhs then lhs, push `lhs / rhs` (IL opcode `0x05`, NON-commutative).
    /// Only reached on the FP path — integer division is not modeled.
    Div,
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

/// A relational operator, as encoded by a single `.ex` operand-stream opcode.
///
/// The opcode is **sign-agnostic** — signed and unsigned probes emit the same
/// byte and differ only in the operand type. Verified per relation against live
/// captures; see `docs/CODEGEN_W6_COMPARE.md` §1.1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rel {
    /// `==`, opcode `0x1F`.
    Eq,
    /// `!=`, opcode `0x20`.
    Ne,
    /// `<=`, opcode `0x21`.
    Le,
    /// `<`, opcode `0x22`.
    Lt,
    /// `>=`, opcode `0x23`.
    Ge,
    /// `>`, opcode `0x24`.
    Gt,
}

impl IlOp {
    /// True for the binary operators a depth-2 tree node may carry.
    pub fn is_tree_binop(self) -> bool {
        matches!(self, IlOp::Add | IlOp::Sub | IlOp::Mul | IlOp::Div)
    }
}

impl Rel {
    fn from_opcode(b: u8) -> Option<Rel> {
        Some(match b {
            0x1F => Rel::Eq,
            0x20 => Rel::Ne,
            0x21 => Rel::Le,
            0x22 => Rel::Lt,
            0x23 => Rel::Ge,
            0x24 => Rel::Gt,
            _ => return None,
        })
    }
}

/// A **comparison leaf** (W6): `return <formal> <rel> <literal>;` materialized
/// to a boolean.
///
/// c2 lowers these *branchlessly* — no `cmpw`/`cmplw` at all — via carry-bit and
/// bit-extraction idioms whose exact instruction sequence depends on the
/// relation, the signedness, and (critically) on whether the literal is zero:
/// `k == 0` is folded to a shorter, different sequence rather than being a
/// special case of the general spine. See `docs/CODEGEN_W6_COMPARE.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompareLeaf {
    /// The compared formal's IL token (it occupies r3, the first argument).
    pub param: u32,
    /// The relation, with the formal on the left (`<formal> <rel> <k>`).
    pub rel: Rel,
    /// Whether the *operand* type is signed (`int`) or not (`unsigned int`).
    /// The opcode does not carry this; the operand type does.
    pub signed: bool,
    /// The literal right-hand side.
    pub k: i32,
}

/// A parsed MVP function: enough to drive the codegen + COFF emitter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IlFunction {
    /// Mangled name, e.g. `?add3@@YAHHHH@Z` (verbatim from `.gl`).
    pub mangled_name: String,
    /// Source path from `.gl`, e.g. `z:\...\mvp_add3.cpp` (provenance only).
    pub source_path: Option<String>,
    /// Formal-parameter IL tokens, in declaration order (a, b, c → r3, r4, r5).
    pub params: Vec<u32>,
    /// Straight-line body op stream (loads + adds) for an arithmetic leaf. For
    /// an **integer tail call** (`tail_call` set, int) this instead holds the
    /// single call argument's sub-expression, computed into r3 before the
    /// branch (`[Load]` passthrough, `[Load,Lit,Add]` for `g(a+1)`). Empty for a
    /// void tail call and for a framed call.
    pub ops: Vec<IlOp>,
    /// If this function is a **tail call** to a single external, its mangled
    /// name (the callee). Codegen emits a `b <callee>` with a REL24 relocation
    /// instead of an arithmetic body: a bare branch for the void tail call
    /// (`ops` empty) or `void f(){g();}`, or an argument-setup prefix + branch
    /// for an integer tail call (`ops` = the argument sub-expression). W4a:
    /// single external only.
    pub tail_call: Option<String>,
    /// If this function is a **framed non-leaf call** (`return g(a) + k`), the
    /// callee + post-op literal. Distinct from `tail_call` (which is a bare
    /// `b g`). W4b2: single-function TU, single external only.
    pub framed_call: Option<FramedCall>,
    /// If this function is a **comparison leaf** (`return a <rel> k;`, W6), the
    /// decoded comparison. Mutually exclusive with the other body kinds.
    pub compare: Option<CompareLeaf>,
    /// If this function is a **W13a floating-point leaf**, whether it is double
    /// precision. Mutually exclusive with the other body kinds.
    pub float_leaf: Option<bool>,
    /// A **multi-argument** tail call's argument permutation. `Some(sources)`
    /// means this is `return g(a1, …, an)` with `n >= 2` and every argument a bare
    /// parameter: `sources[i]` is the index into [`Self::params`] of the value that
    /// argument slot `i` (register `r(3+i)`) wants. Set together with
    /// [`Self::tail_call`], and then [`Self::ops`] is empty — the permutation, not
    /// an operand stream, is the whole argument setup.
    ///
    /// The one-argument case keeps using `ops` instead, because it can carry a
    /// computed argument (`g(a + 1)`) that the permutation form cannot express.
    pub arg_sources: Option<Vec<usize>>,
    /// True iff this function's body is **empty** (`void f() {}`): no expression at
    /// all, so codegen emits a bare `blr`. Mutually exclusive with the other body
    /// kinds.
    ///
    /// (These discriminators want to be one enum. [`BodyShape`] already *is* that
    /// enum — the parser produces it and `functions()` immediately flattens it into
    /// the parallel options above, which `PortC2::build` then re-derives through two
    /// separate priority chains. The remaining reason to defer is the CFG step's
    /// real body IR (docs/ROADMAP.md §G4), but carrying `BodyShape` here does not
    /// need that design and would remove the second decision tree. This doc block
    /// was itself misattached to `float_leaf` for a while, which is the kind of
    /// damage the sum type prevents.)
    pub empty_body: bool,
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
    // Same walk as the old byte loop: jump to each `?`, read its NUL-terminated
    // run, and resume after the run whether or not it was accepted.
    while i < gl.len() {
        let Some(k) = memchr_byte(b'?', &gl[i..]) else {
            break;
        };
        let start = i + k;
        let end = start
            + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
        let bytes = &gl[start..end];
        if bytes.len() >= 3
            && bytes[1].is_ascii_alphabetic()
            && contains_subslice(bytes, b"@@")
            && bytes.iter().all(|b| b.is_ascii_graphic())
        {
            return Some(ascii_string(bytes));
        }
        i = end + 1;
    }
    None
}

/// Extract **all** mangled names from `.gl`, in file order — one per function
/// in the translation unit. Same acceptance test as [`mangled_name`]; used for
/// multi-function TUs where `.gl` carries a name per function.
pub fn mangled_names(gl: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    // Same walk as `mangled_name`, collecting every accepted run.
    while i < gl.len() {
        let Some(k) = memchr_byte(b'?', &gl[i..]) else {
            break;
        };
        let start = i + k;
        let end = start
            + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
        let bytes = &gl[start..end];
        if bytes.len() >= 3
            && bytes[1].is_ascii_alphabetic()
            && contains_subslice(bytes, b"@@")
            && bytes.iter().all(|b| b.is_ascii_graphic())
        {
            out.push(ascii_string(bytes));
        }
        i = end + 1;
    }
    out
}

/// Every mangled-symbol run in `.gl`, as `(start, end, name)` in file order.
///
/// Deliberately **broader** than [`mangled_names`], which requires the second
/// byte to be alphabetic and therefore silently drops every `??`-prefixed name:
/// constructors (`??0S@@QAA@XZ`) and the `??__E` dynamic-initializer thunks that
/// a namespace-scope object with a constructor makes c2 emit. Those are real
/// symbols in the obj, and dropping them is what made a positional pairing look
/// safe — `.gl` for `struct S{S();}; S gs; int f(int);` lists
/// `??__Egs@@YAXXZ`, `?f@@YAHH@Z`, `?gs@@3US@@A`, `??0S@@QAA@XZ`, of which
/// `mangled_names` sees only the second and *fourth*, so pairing two names to
/// two bodies named the second function after a **variable**.
///
/// A run is accepted only if it is NUL-delimited, wholly printable, starts like
/// an identifier, and contains `@@`. The `@@` is what keeps the source path and
/// other incidental strings out: they are printable NUL-delimited runs too.
fn gl_symbol_runs(gl: &[u8]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < gl.len() {
        if gl[i] != 0 {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end] != 0 {
            end += 1;
        }
        if end >= gl.len() || end == start {
            i += 1;
            continue;
        }
        let bytes = &gl[start..end];
        let plausible = bytes.len() >= 3
            && bytes.iter().all(|b| b.is_ascii_graphic())
            && (bytes[0] == b'?' || bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
            && contains_subslice(bytes, b"@@");
        if plausible {
            out.push((start, end, ascii_string(bytes)));
        }
        i = end;
    }
    out
}

/// Bind each **defined** function's `.gl` name to the `.ex` offset of its body,
/// positively. Returns the `(body offset, name)` pairs in `.gl` record order,
/// plus every mangled run that no record claimed.
///
/// Each `.gl` function record carries a `80 <LE32>` body-start offset field,
/// located by its record framing ([`codec::gl_offset_framed`]) rather than by
/// what its value happens to be, and the record's name is the mangled run
/// immediately preceding that field. So the binding is per record.
///
/// This replaces "the Nth name belongs to the Nth body", an invariant `.gl` does
/// not promise. It happens to hold across the fixtures, and the four probes that
/// looked most likely to break it (`extern` data, static members, namespaces,
/// templates) all list definitions first — but nothing enforces it, `.gl`
/// interleaves data symbols and compiler-generated thunks into the same list,
/// and a shifted name is a `.text` symbol emitted under some other symbol's
/// name. That is a wrong-bytes emit, not a refusal, so it is not something to
/// leave resting on an unchecked ordering.
///
/// The unclaimed runs matter just as much: an unclaimed name is a symbol the
/// real obj carries and the port does not model. `int gv; int f(int a){…}` leaves
/// `?gv@@3HA` unclaimed and c2's obj has an extra section for it — the port used
/// to emit its fixed four-section shell and mismatch at file offset 2, the
/// section count. The caller must account for every unclaimed run or refuse.
fn gl_defined_names(gl: &[u8]) -> (Vec<(u32, String)>, Vec<String>) {
    let runs = gl_symbol_runs(gl);
    let mut claimed = vec![false; runs.len()];
    let mut bound: Vec<(u32, String)> = Vec::new();
    let mut p = 0usize;
    while p + 5 <= gl.len() {
        if crate::codec::gl_offset_framed(gl, p) {
            let off = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]);
            // The record's own name: the last mangled run to END at or before
            // this field. Searched backwards so an unnamed record cannot borrow
            // the name of a following one.
            match runs.iter().rposition(|&(_, end, _)| end <= p) {
                Some(k) => {
                    claimed[k] = true;
                    bound.push((off, runs[k].2.clone()));
                }
                // A framed offset with no name ahead of it is a record shape we
                // do not understand; refuse the whole TU rather than emit a
                // nameless function.
                None => return (Vec::new(), Vec::new()),
            }
            p += 5;
            continue;
        }
        p += 1;
    }
    let unclaimed = runs
        .iter()
        .zip(&claimed)
        .filter(|(_, &c)| !c)
        .map(|((_, _, n), _)| n.clone())
        .collect();
    (bound, unclaimed)
}

/// Build the `.gl` **symbol index**: operand token → mangled name.
///
/// `.gl` records have the shape
/// `<kind byte> <operand token> 00 <NUL-terminated name> 00 <TYPE> …`, so a
/// record is located by its `00 <name> 00` core and the token read backwards
/// from it with the same variable-width rule the operand stream uses.
///
/// This is what binds a call to its callee. The CALL token does *not* name the
/// callee — three different callees sharing one signature produce byte-identical
/// CALL tokens — so the name comes from the `26 <tok>` symbol push that precedes
/// it, resolved through this index. Verified on a real TU: 2323 of 2323 direct
/// call sites resolve, and the complementary controlled fixtures show tokens are
/// assigned in *declaration* order but used in *call* order, and that a repeated
/// callee repeats its token.
///
/// `.sy` is deliberately not consulted: it holds local and parameter names, and
/// real callees (`?MemPushTemp@@YAXXZ`) are absent from it and present here.
///
/// Names are accepted only if they look like whole mangled identifiers, so a
/// stray NUL-delimited run inside binary payload cannot inject a false symbol.
pub fn gl_symbol_index(gl: &[u8]) -> std::collections::BTreeMap<u32, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut i = 0usize;
    while i < gl.len() {
        // A record's name is a NUL-terminated printable run preceded by a NUL.
        if gl[i] != 0 {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut end = start;
        while end < gl.len() && gl[end] != 0 {
            end += 1;
        }
        if end >= gl.len() || end == start {
            i += 1;
            continue;
        }
        let name_bytes = &gl[start..end];
        let plausible = name_bytes.len() >= 3
            && name_bytes.iter().all(|b| b.is_ascii_graphic())
            && (name_bytes[0] == b'?' || name_bytes[0].is_ascii_alphabetic() || name_bytes[0] == b'_');
        if !plausible {
            i = end.max(i + 1);
            continue;
        }
        // The operand token sits immediately before the leading NUL. Try the
        // 4-byte form first, then the 2-byte one, and keep whichever decodes to
        // a token whose own width lands exactly on that NUL.
        for w in [4usize, 2] {
            if i < w {
                continue;
            }
            let p = i - w;
            if let Some((tok, got)) = read_token_var(gl, p) {
                if got == w {
                    out.entry(tok)
                        .or_insert_with(|| ascii_string(name_bytes));
                    break;
                }
            }
        }
        i = end;
    }
    out
}

/// Extract the source path from `.gl`: a `<letter>:\…\<name>.cpp` NUL-terminated
/// ASCII run (case-insensitive drive + `.cpp` suffix). Provenance only.
pub fn source_path(gl: &[u8]) -> Option<String> {
    let mut i = 0;
    // A candidate must have `:` at its second byte, so scan for `:` and test
    // the byte on each side — the same candidates the old per-byte walk saw, in
    // the same order, with the same resume points (past the NUL run when the
    // `<x>:\` prefix matched, past the candidate start when it did not).
    while i + 2 < gl.len() {
        let Some(k) = memchr_byte(b':', &gl[i + 1..]) else {
            break;
        };
        let start = i + k; // gl[start + 1] == b':'
        if start + 2 >= gl.len() {
            break;
        }
        if gl[start].is_ascii_alphabetic() && gl[start + 2] == b'\\' {
            let end = start
                + memchr_byte(0, &gl[start..]).unwrap_or(gl.len() - start);
            let bytes = &gl[start..end];
            if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                // Case-insensitive `.cpp` suffix, checked on the bytes — same
                // acceptance as lowercasing the whole string, without the two
                // String allocations that cost on the hot parse path.
                if bytes.len() >= 4 && bytes[bytes.len() - 4..].eq_ignore_ascii_case(b".cpp") {
                    return Some(ascii_string(bytes));
                }
            }
            i = end + 1;
        } else {
            i = start + 1;
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

/// Lazily-built `.gl` symbol index (see [`gl_symbol_index`]) — same contents,
/// built on first use. Only the call productions consult it (callee-by-token
/// resolution), so a TU of straight-line leaves never pays for building it;
/// every consumer goes through [`GlIndex::map`], which always yields the full
/// real index, so laziness can never change what is accepted.
struct GlIndex<'a> {
    gl: &'a [u8],
    cell: std::cell::OnceCell<std::collections::BTreeMap<u32, String>>,
}

impl<'a> GlIndex<'a> {
    fn new(gl: &'a [u8]) -> Self {
        GlIndex {
            gl,
            cell: std::cell::OnceCell::new(),
        }
    }
    /// The token → name map, built on first use.
    fn map(&self) -> &std::collections::BTreeMap<u32, String> {
        self.cell.get_or_init(|| gl_symbol_index(self.gl))
    }
}

/// Owned `String` from a byte run the caller has already verified is ASCII
/// (graphic / space). `from_utf8` is then infallible and takes the fast
/// validated path; the lossy fallback is defensive only and never replaces
/// anything for such input, so the result is identical to
/// `String::from_utf8_lossy(bytes).into_owned()` — minus its chunk iterator,
/// which was measurable on the hot parse path.
fn ascii_string(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// Read one IL token, which is **2 or 4 bytes depending on the token itself**,
/// returning `(identity, width)`.
///
/// Token width is per-token, not a per-file constant: the second byte carries a
/// continuation flag in bit 7. Clear → the token is those 2 bytes; set → two
/// more bytes follow. Verified on a real capture of `system/world/Dir.cpp`,
/// where `4F 02` module markers appear as both `4f 02 e3 09` (2-byte) and
/// `4f 02 a4 96 03 00` (4-byte) in the same file, and where applying this rule
/// to every `B9` LOAD site lands on a valid 3-byte operand type at 21443 sites
/// (the residue being a third type class plus `B9` bytes occurring inside data).
///
/// [`detect_token_width`] — which returns a single width for the whole file —
/// is therefore wrong for real translation units, and its misalignment is what
/// produced the artifact census buckets `call-token-0x01…0x05` and
/// `expr-load-type-0N00A6`: a 2-byte read of a 4-byte token leaves the parse
/// standing on the token's own tail bytes. It is kept only for the K1/K2a codec
/// and the existing tests; the parser no longer consults it.
///
/// The returned identity is only ever compared for equality (token → parameter
/// register), so any injective encoding will do. The two widths cannot collide:
/// a 2-byte token's value is `< 0x10000` while a 4-byte token's byte 1 has bit 7
/// set, which lands in bits 23..16 of the result and forces it `>= 0x10000`.
fn read_token_var(ex: &[u8], p: usize) -> Option<(u32, usize)> {
    let b0 = *ex.get(p)? as u32;
    let b1 = *ex.get(p + 1)? as u32;
    if b1 & 0x80 == 0 {
        return Some(((b0 << 8) | b1, 2));
    }
    let b2 = *ex.get(p + 2)? as u32;
    let b3 = *ex.get(p + 3)? as u32;
    Some(((b0 << 24) | (b1 << 16) | (b2 << 8) | b3, 4))
}

/// Forward byte search, word-at-a-time (hand-rolled `memchr`; std-only, no
/// dependency). The `.ex` marker scans walk multi-KB streams whose prefix is
/// zero-fill, and the byte-at-a-time loops were the port's single hottest path
/// (~60% of a small compile). This finds the first candidate byte 8 bytes per
/// step with the classic SWAR zero-byte trick, then lets a plain scan finish
/// inside the matching word — same result as `iter().position`, just faster.
#[inline]
fn memchr_byte(needle: u8, hay: &[u8]) -> Option<usize> {
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;
    #[inline(always)]
    fn has_zero_byte(w: u64) -> bool {
        w.wrapping_sub(LO) & !w & HI != 0
    }
    let bcast = LO.wrapping_mul(needle as u64);
    let mut i = 0;
    // 32 bytes per step while no word contains the needle…
    while i + 32 <= hay.len() {
        let a = u64::from_ne_bytes(hay[i..i + 8].try_into().unwrap()) ^ bcast;
        let b = u64::from_ne_bytes(hay[i + 8..i + 16].try_into().unwrap()) ^ bcast;
        let c = u64::from_ne_bytes(hay[i + 16..i + 24].try_into().unwrap()) ^ bcast;
        let d = u64::from_ne_bytes(hay[i + 24..i + 32].try_into().unwrap()) ^ bcast;
        if has_zero_byte(a) || has_zero_byte(b) || has_zero_byte(c) || has_zero_byte(d) {
            break;
        }
        i += 32;
    }
    // …then 8, then byte-at-a-time to pin the first occurrence.
    while i + 8 <= hay.len() {
        let w = u64::from_ne_bytes(hay[i..i + 8].try_into().unwrap()) ^ bcast;
        if has_zero_byte(w) {
            break;
        }
        i += 8;
    }
    hay[i..].iter().position(|&x| x == needle).map(|k| i + k)
}

fn find_byte(hay: &[u8], b: u8) -> Option<usize> {
    memchr_byte(b, hay)
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    // First-occurrence semantics identical to `windows().position`; the scan
    // just jumps between candidate first bytes word-at-a-time.
    let last_start = hay.len() - needle.len();
    let mut i = 0;
    while i <= last_start {
        let k = memchr_byte(needle[0], &hay[i..=last_start])?;
        let j = i + k;
        if &hay[j..j + needle.len()] == needle {
            return Some(j);
        }
        i = j + 1;
    }
    None
}

/// Read a `.ex` inline **type**: `<tag> <kind> <LEB128 id>`, returning
/// `(tag, kind, id, width)`.
///
/// This is a *third* encoding, distinct from both the operand token
/// ([`read_token_var`], 2 or 4 bytes) and the statement/literal varint
/// ([`read_varint`], 1 or 5 bytes). Total width is 3 bytes for an id below
/// `0x80`, 4 below `0x4000`, 5 below `0x200000`.
///
/// Getting this right matters more than it looks: across the 8628 well-formed
/// call sites of one real TU the return-type width splits 4157 / 3123 / 1358
/// between 3, 4 and 5 bytes, so a fixed-3 or even a "3 or 4" rule mis-parses
/// roughly one call in six. The boundaries are pinned by the fixed one-byte
/// markers that always bracket a type — the `41` result-type marker, the `55`
/// argument push, the `4C 4B` call end — against which a wrong width visibly
/// swallows the next marker.
///
/// The tag always has bit 7 set; its high bits are not understood (`0x86`,
/// `0xA6`, `0x96`, `0xC6` all occur and behave identically here), but a decoder
/// does not need them — the width rule is tag-independent. The `kind` byte is
/// treated as a fixed byte rather than a second LEB because `88 85 41`
/// (`double`) and `88 81 13` (`long long`) have bit 7 set there and would
/// otherwise run on.
fn read_type(seg: &[u8], p: usize) -> Option<(u8, u8, u32, usize)> {
    let tag = *seg.get(p)?;
    if tag & 0x80 == 0 {
        return None;
    }
    let kind = *seg.get(p + 1)?;
    let mut id: u32 = 0;
    let mut shift: u32 = 0;
    let mut i = p + 2;
    loop {
        let b = *seg.get(i)?;
        id |= ((b & 0x7F) as u32) << shift;
        i += 1;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return None; // malformed / not a type
        }
    }
    Some((tag, kind, id, i - p))
}

/// One recognized whole-body shape of a single `.ex` function segment. Every
/// accepted body is *exactly* one of these — the parser (see [`parse_segment`])
/// is a positive whole-stream parse that reaches the segment's end, so anything
/// it does not model produces `None` and the caller reports `NotImplemented`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum BodyShape {
    /// Straight-line all-`int` arithmetic leaf (`return a+b+c`, `return a+5`,
    /// `return 42`, …): a postfix LOAD/LIT/ADD/SUB/MUL stream returning `int`.
    StraightLine { params: Vec<u32>, ops: Vec<IlOp> },
    /// Bare terminal void tail call (`void f(){ g(); }`): exactly one CALL whose
    /// void result is discarded, with **nothing** after its `4C 4B` void
    /// call-end but the return plumbing → codegen emits a single `b <callee>`.
    VoidTailCall { callee_tok: u32 },
    /// Integer tail call `return g(<arg>)` (and the identity-fold `g(a) + 0`):
    /// exactly one int-returning CALL whose single argument is a modeled
    /// sub-expression (`arg_ops`), a `55 <int> 4C` call-end, and a **net-identity
    /// post-op** (absent, or `+ 0` folded away). Codegen computes the argument
    /// into r3 (the leaf selector), then `b <callee>` — a 5-section leaf, the
    /// integer analog of [`BodyShape::VoidTailCall`]. `arg_ops` is a bare
    /// `[Load]` for the passthrough `g(a)`, or e.g. `[Load, Lit, Add]` for the
    /// arg-setup `g(a + 1)` (→ `addi r3,r3,1 ; b g`). `params` are the formals
    /// (token→register mapping the arg-setup needs).
    IntTailCall { params: Vec<u32>, arg_ops: Vec<IlOp>, callee_tok: u32 },
    /// `return g(a1, …, an)` with `n >= 2`, every argument a bare parameter.
    /// `arg_sources[i]` indexes `params` for the value argument slot `i` wants;
    /// codegen turns that into a register permutation plus the tail branch.
    MultiArgTailCall { params: Vec<u32>, arg_sources: Vec<usize>, callee_tok: u32 },
    /// Framed non-leaf `return g(a) + k` (k ≠ 0): exactly one int-returning CALL
    /// whose argument region is exactly the single passthrough LOAD, a `55 <int>`
    /// call-end, then exactly one literal `+ k` (ADD, commutative), returned. A
    /// zero `k` is NOT framed — it folds to [`BodyShape::IntTailCall`].
    FramedCall { add_k: i32, callee_tok: u32 },
    /// W6 comparison leaf: `return <formal> <rel> <literal>;` materialized to a
    /// boolean branchlessly and converted back to `int`/`unsigned`.
    Compare(CompareLeaf),
    /// W13a floating-point leaf: a straight-line chain over float (or double)
    /// *parameters* — no constants, no conversions, no contraction.
    FloatLeaf { params: Vec<u32>, ops: Vec<IlOp>, double: bool },
    /// An **empty function body** (`void f() {}`): the body opens directly on the
    /// `3A` assign of the return plumbing with no expression before it. Emits a
    /// bare `blr`.
    EmptyBody,
    /// An **indirect-load leaf**: the whole body is one load through a pointer
    /// (`return *p;`, `return s->m;`, `return p[k];`, `return mMember;`), which c2
    /// lowers to a single `lwz rD, off(rBase)`. `ops` is always exactly
    /// `[Load(base), LoadInd { off }]` and `params` includes a member function's
    /// `this` at index 0. See [`try_parse_indirect_load_leaf`].
    IndirectLoad { params: Vec<u32>, ops: Vec<IlOp> },
}

/// **Why** a function segment fell outside the modeled class (P2b census).
///
/// The positive parser fails closed at the *first* byte it cannot account for.
/// Recording that point — the grammar production it was in, the offending byte,
/// and the offset — turns an opaque `None` into a rankable census key: over a
/// real workload the histogram of [`Block::feature`] *is* the widening order
/// (docs/ROADMAP.md §G5/P2b). Purely diagnostic: acceptance is unchanged, and
/// [`parse_segment`] still returns a bare `Option`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    /// The grammar production the parse was inside (`"expr"`, `"call-end"`, …).
    pub ctx: &'static str,
    /// The byte that could not be consumed (`None` at end-of-segment).
    pub byte: Option<u8>,
    /// Byte offset within the function segment.
    pub off: usize,
    /// Context payload for the operand-*type* blocks, where the single blocking
    /// byte is uninformative: an operand's 3-byte inline type differs from the
    /// modeled `86 41 74` (int), but the first byte `86` is shared by every
    /// type, so reporting it buckets `unsigned`, `float`, `pointer`, … together.
    /// Packed big-endian in the low 24 bits; 0 when unused.
    pub aux: u32,
}

impl Block {
    /// A short, stable census key naming the blocking *feature*.
    ///
    /// Operand-stream opcodes get a named bucket when the byte's meaning is
    /// verified against a live capture, and a `expr-op-0xNN` bucket otherwise —
    /// the point of the census is to *measure* the unknown vocabulary, so an
    /// honest hex bucket is a result, not a placeholder. Structural blocks
    /// (call-end, return plumbing, formals) name their production instead.
    pub fn feature(self) -> String {
        // Intrinsic-call blocks report their **selector**, which [`Block::aux`]
        // carries as the decoded id (see [`intrinsic_selector`]). This is the
        // whole point of decoding the production: `0x40` alone is one opaque 9 %
        // bucket, while the selector splits it into a handful of named
        // constructs with wildly different lowerings — `fabs` is one
        // instruction, `memcmp` is a 15-instruction loop, and the dominant
        // 2113–2119 class-layout family is a pointer adjustment whose emission
        // depends on its *literal* arguments, not on the id.
        if self.ctx == "expr-intrinsic" || self.ctx == "call-intrinsic" {
            return format!("{}-{}", self.ctx, intrinsic_name(self.aux as i32));
        }
        // Operand-type blocks report the whole 3-byte type: that triple *is* the
        // feature (int vs unsigned vs float vs pointer), and it is what the next
        // widening step must teach `parse_expr` to accept.
        if self.aux != 0 {
            return format!(
                "{}-{:02X}{:02X}{:02X}",
                self.ctx,
                (self.aux >> 16) & 0xFF,
                (self.aux >> 8) & 0xFF,
                self.aux & 0xFF
            );
        }
        let b = match self.byte {
            Some(b) => b,
            None => return format!("{}:eof", self.ctx),
        };
        if self.ctx == "expr" {
            // Operand-stream opcodes VERIFIED against live-toolchain captures
            // (docs/CODEGEN_W6_COMPARE.md pins the relational and logical ones
            // by compiling a probe per relation and reading the emitted byte).
            //
            // Only add a name here once a capture has established it. An earlier
            // revision of this table guessed the relational opcodes from their
            // numeric order and got `!=`, `<=` and `>=` wrong while missing `==`
            // entirely — which silently mislabelled census buckets, the one
            // thing this instrument exists to avoid. A hex bucket is a result;
            // a wrong name is a lie that survives into the roadmap.
            //
            // Signedness is NOT in the opcode: signed and unsigned probes emit
            // the same byte and differ only in the operand type (`86 41 74` int
            // vs `86 42 75` unsigned).
            let named = match b {
                0x1F => Some("cmp-eq"),   // ==
                0x20 => Some("cmp-ne"),   // !=
                0x21 => Some("cmp-le"),   // <=
                0x22 => Some("cmp-lt"),   // <
                0x23 => Some("cmp-ge"),   // >=
                0x24 => Some("cmp-gt"),   // >
                0x1A => Some("not"),      // !
                0x1B => Some("or-or"),    // ||
                0x1C => Some("and-and"),  // &&
                0x09 => Some("shl"),      // <<
                0x0A => Some("shr"),      // >>
                0x0B => Some("bit-and"),  // &
                0x0C => Some("bit-or"),   // |
                0x0D => Some("bit-xor"),  // ^
                0x2C => Some("convert"),  // `2C <TYPE> <varint>` — the real cast
                // `0x40` is a SECOND call token — the intrinsic call — not a
                // cast. It occupies the slot `BD` occupies:
                //   33 <int-TYPE> <selector>  40 <TYPE result>  (<expr> 55 <TYPE>)*  4C
                // An earlier revision of this table guessed "cast" from a single
                // witness where it followed a literal. It follows a bare `int`
                // constant at 6838 of 6839 aligned sites across three real TUs —
                // which is the selector, not a cast operand. Selectors seen:
                // 15 abs, 17 fabs, 159/160 _rotl/_rotr, 164 strcpy, 165 strcmp,
                // 167 strlen, 170 memcmp, 172 memcpy, 173 memset, 1973 sqrt,
                // and the dominant 2113-2119 class-layout adjustment family.
                0x40 => Some("intrinsic-call"),
                // The class-pair descriptor of that same family — NOT a call.
                0x66 => Some("class-descriptor"),
                0x43 => Some("ternary"),  // `43 42 ...` conditional select
                0x26 => Some("call-in-expr"),
                _ => None,
            };
            return match named {
                Some(n) => format!("expr-{n}"),
                None => format!("expr-op-0x{b:02X}"),
            };
        }
        format!("{}-0x{b:02X}", self.ctx)
    }
}

/// Build a [`Block`] at the current parse position.
fn blk(seg: &[u8], p: usize, ctx: &'static str) -> Block {
    Block { ctx, byte: seg.get(p).copied(), off: p, aux: 0 }
}

/// Build an operand-*type* [`Block`]: `p` points at the 3-byte inline type that
/// is not the modeled int (`86 41 74`), `report_at` at the operand it belongs
/// to. Packs the triple into [`Block::aux`] so the census buckets by type.
fn blk_type(seg: &[u8], p: usize, report_at: usize, ctx: &'static str) -> Block {
    let g = |i: usize| seg.get(p + i).copied().unwrap_or(0) as u32;
    Block {
        ctx,
        byte: seg.get(p).copied(),
        off: report_at,
        aux: (g(0) << 16) | (g(1) << 8) | g(2),
    }
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

/// Consume zero or more `4F 01 <varint>` **source-line markers**.
///
/// Two corrections over the previous reading, both from live captures:
///
/// * the payload is a [`read_varint`], not a fixed byte. A function at source line
///   200 emits `4f 01 80 c8 00 00 00` — the escaped four-byte form. Reading one
///   byte therefore desynchronizes the whole token stream for any TU whose
///   functions live past line 127, which is nearly all of them; the parse then
///   fails somewhere arbitrary downstream and the census attributes the block to
///   whatever byte it happened to land on. So this was not only costing coverage,
///   it was corrupting the blocking-feature histogram that the widening order is
///   chosen from.
/// * it is emitted on each line *change*, and two can appear in a row where a
///   declaration line generates no code (`int x;` followed by a statement), so
///   this loops instead of eating at most one.
///
/// Still specific to `4F 01` — it never eats the `4F 12` separator or the `4F 02`
/// module marker.
fn eat_opt_stmt_marker(seg: &[u8], p: &mut usize) {
    while seg.get(*p) == Some(&0x4F) && seg.get(*p + 1) == Some(&0x01) {
        let mut probe = *p + 2;
        if read_varint(seg, &mut probe).is_none() {
            return; // malformed payload: leave `p` put and let the caller block
        }
        *p = probe;
    }
}

/// Read an int literal varint `33 <int-type> <varint>` payload (the `33
/// <int-type>` prefix already consumed): a single byte if `< 0x80` (the value),
/// else `0x80` + a 4-byte LE i32. (Verified: 5→`05`, 42→`2a`, 200→`80 c8000000`,
/// 70000→`80 70110100`.) Any other lead byte → `None`.
fn read_varint(seg: &[u8], p: &mut usize) -> Option<i32> {
    let marker = *seg.get(*p)?;
    if marker == 0x80 {
        // Escape: `80` + a 4-byte LE i32. Used for anything that does not fit
        // the signed short form — including −128, whose byte encoding would
        // otherwise collide with this marker.
        //
        // NOTE: for tag-`0x88` types (`long long`) the escape payload is 8
        // bytes, not 4. Those types are rejected upstream, so this reads only
        // the 4-byte form; widening to 64-bit literals must fix this too.
        let v = i32::from_le_bytes([
            *seg.get(*p + 1)?,
            *seg.get(*p + 2)?,
            *seg.get(*p + 3)?,
            *seg.get(*p + 4)?,
        ]);
        *p += 5;
        Some(v)
    } else {
        // Short form: a **signed** byte, not an unsigned one. `-5` is `fb` and
        // `(char)200` is `c8`. An earlier revision accepted only `00..7F` and
        // rejected `81..FF` outright — fail-closed and safe, but it silently
        // blocked every negative literal in the corpus.
        *p += 1;
        Some(marker as i8 as i32)
    }
}

/// Recognize an **intrinsic-call selector** at `p`: the two-token unit
///
/// ```text
///   33 86 41 74 <varint id>   40
/// ```
///
/// and return the decoded id, or `None` if the bytes are not that shape.
/// Diagnostic only — every caller turns a hit straight into a [`Block`].
///
/// `0x40` is a **second CALL token**, the intrinsic call, occupying exactly the
/// slot `BD` occupies in an ordinary call (`docs/IL_CALL_GRAMMAR.md` §2). Its
/// callee identity is not in the token at all: it is the *preceding* `int`
/// literal, and the token itself is only `40 <TYPE result>` — no
/// calling-convention byte, no function-type id, and (unlike `2C`) **no trailing
/// field**. Two controlled nullary witnesses pin that:
///
/// ```text
///   void n_break()    { __debugbreak(); }
///     33 86 41 74 80 1f 02 00 00  40 82 07 03  4C 4B
///   void *n_retaddr() { return _ReturnAddress(); }
///     33 86 41 74 80 e5 00 00 00  40 86 43 83 08  4C  41 86 43 83 08 …
/// ```
///
/// With zero arguments the `4C` apply sits immediately after the result type, so
/// a `40 <TYPE> <varint>` reading would swallow it and leave the argument list
/// unterminated. See `docs/IL_INTRINSIC_CALL.md` §1.
///
/// Requiring the selector's type to be **exactly** `86 41 74` is deliberate: the
/// residual `expr-intrinsic-call` bucket then measures how often `0x40` is *not*
/// preceded by a plain `int` literal, which is the one structural claim this
/// decode rests on.
fn intrinsic_selector(seg: &[u8], p: usize) -> Option<i32> {
    if seg.get(p)? != &0x33 || seg.get(p + 1..p + 4)? != INT_TYPE {
        return None;
    }
    let mut q = p + 4;
    let id = read_varint(seg, &mut q)?;
    if seg.get(q)? != &0x40 {
        return None;
    }
    Some(id)
}

/// The census name for an intrinsic selector id, or `0xNN` when the id has not
/// been pinned.
///
/// Every name here is pinned by a **controlled fixture** whose `.gl` gave the
/// enclosing function's mangled name and whose reference obj gave the emitted
/// instructions — `fixtures/cpp/il_intrinsic_call.cpp`,
/// `il_intrinsic_nullary.cpp`, `il_intrinsic_bits.cpp` and
/// `il_intrinsic_layout.cpp`, tabulated in `docs/IL_INTRINSIC_CALL.md` §3–§4.
/// Ids observed in the real workload but *not* named there stay hex, for the
/// reason the relational-opcode table gives above: a hex bucket is a result, a
/// wrong name is a lie that survives into the roadmap. The two unnamed ids that
/// actually occur (`0xDE`/`0xDF`, 1758 sites each) are characterized in §5 —
/// trigger and literal pinned, division of labour still UNKNOWN.
///
/// The id space is a c1xx-internal table and is **not enumerable from the IL**;
/// these are the 20 ids that occur across `Dir.cpp`, `App.cpp` and `Game.cpp`
/// plus the ones the fixtures reach.
fn intrinsic_name(id: i32) -> String {
    let named = match id {
        // --- CRT string / memory family (ids 164..173) ---
        164 => "strcpy",
        165 => "strcmp",
        166 => "strcat",
        167 => "strlen",
        170 => "memcmp",
        172 => "memcpy",
        173 => "memset",
        // --- arithmetic / bit helpers ---
        15 => "abs",   // also `labs` — one id serves the whole name family
        17 => "fabs",
        159 => "_rotl",
        160 => "_rotr",
        226 => "_InterlockedIncrement",
        229 => "_ReturnAddress",
        236 => "__emul",
        237 => "__emulu",
        318 => "_InterlockedExchangeAdd",
        543 => "__debugbreak",
        813 => "_rotl64",
        814 => "_rotr64",
        815 => "_abs64",
        839 => "_byteswap_ushort",
        840 => "_byteswap_ulong",
        841 => "_byteswap_uint64",
        850 => "_CountLeadingZeros",
        921 => "_CountLeadingZeros64",
        1935 => "__frsqrte",
        1937 => "__fsel",
        1948 => "__mftb",
        1973 => "sqrt",
        // --- C++ runtime ---
        337 => "throw",
        // --- the class-layout family (2113..2119), the bulk of the bucket ---
        2113 => "this-adjust",       // base adjust for a member call's `this`, UNguarded
        2114 => "base-upcast",       // derived → base, null-guarded
        2115 => "base-downcast",     // base → derived, null-guarded, offset negated
        2116 => "vbase-upcast",      // through a virtual base's vbtable
        2117 => "base-member-addr",  // &member inherited from a non-virtual base
        2118 => "vbase-member-addr", // &member of a virtual base
        2119 => "dynamic-cast",
        _ => return format!("0x{:X}", id as u32),
    };
    named.to_string()
}

/// Consume the shared statement/function-tail plumbing that follows the body
/// expression of *every* accepted shape, and require the parse to reach the end
/// of the segment (the fail-closed terminal — anything trailing rejects). With
/// `has_result_type`, a `41 <int-type>` result annotation is expected first
/// (present for an int return, absent for a void call). Layout (verified):
/// `[41 <int-like>]?` result-type · `3A <label>` branch · `[4F 01 <line>]*` ·
/// `54 02 29 <tok>` return · `4F 12` · `47 54 01 54 00` GT-terminate · then
/// EITHER the segment end (a non-last function, split before the next `4F 1F`) OR
/// the module end `4F 02 20 00 · 4F 01 <line> · 4D` and trailing zero-fill (the
/// last function).
///
/// `3A <tok>` was previously labelled "assign", as if it stored the body
/// expression into a return temporary. It does not: it is an **unconditional
/// branch** and its operand is a label. `void f() { return; }` captures as
/// `53 3a <lbl> 3a <lbl> 54 02 29 <lbl> …` — two of them back to back with no
/// expression anywhere, so there is nothing for a store to store. The same opcode
/// carries `break`, `continue`, `goto` and the if/else join jump. Nothing here
/// depends on the distinction, since this function only skips the token, but the
/// old name would mislead anyone extending it. See `docs/IL_STMT_GRAMMAR.md`.
fn eat_return_plumbing(seg: &[u8], p: &mut usize, has_result_type: bool) -> Result<(), Block> {
    if has_result_type {
        let save = *p;
        if !(eat_byte(seg, p, 0x41) && eat_int_like(seg, p)) {
            *p = save;
            return Err(blk(seg, *p, "result-type"));
        }
    }
    // ASSIGN: 3A <tok>
    if !eat_byte(seg, p, 0x3A) {
        return Err(blk(seg, *p, "assign"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "assign-tok"))?;
    *p += w;
    eat_opt_stmt_marker(seg, p);
    // RETURN: 54 02 29 <tok>
    if !eat(seg, p, &[0x54, 0x02, 0x29]) {
        return Err(blk(seg, *p, "return"));
    }
    let (_, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "return-tok"))?;
    *p += w;
    // Function-tail: 4F 12 · 47 54 01 54 00
    if !eat(seg, p, &[0x4F, 0x12]) || !eat(seg, p, &[0x47, 0x54, 0x01, 0x54, 0x00]) {
        return Err(blk(seg, *p, "fn-tail"));
    }
    // A non-last function's segment ends exactly here (the split cuts before the
    // next `4F 1F`). Otherwise the last function carries the module end.
    if *p == seg.len() {
        return Ok(());
    }
    if !eat(seg, p, &[0x4F, 0x02, 0x20, 0x00]) || !eat(seg, p, &[0x4F, 0x01]) {
        return Err(blk(seg, *p, "module-end"));
    }
    // The module-end marker's payload is the same varint-encoded source line as
    // every other `4F 01`, so it is four bytes longer past line 127.
    read_varint(seg, p).ok_or(blk(seg, *p, "module-end-line"))?;
    if !eat_byte(seg, p, 0x4D) {
        return Err(blk(seg, *p, "module-end"));
    }
    // Trailing zero-fill to the end of `.ex`.
    while seg.get(*p) == Some(&0) {
        *p += 1;
    }
    if *p == seg.len() {
        Ok(())
    } else {
        Err(blk(seg, *p, "trailing"))
    }
}

/// Consume a postfix LOAD/LIT/ADD/SUB/MUL operand sub-stream, stopping (without
/// consuming) at the `stop` byte that begins the following production. Two call
/// sites, same integer-expression class: the straight-line leaf body stops at
/// the `41` result-type marker (the return plumbing); the call-argument region
/// stops at the `55` call-end marker. Fail-closed: any byte that is not a
/// modeled operand/opcode (a comparison `24`, shift `09`, bitwise `0B`, ternary
/// `43 42`, …) rejects the whole function. Requires at least one op.
///
/// `stop` is only ever tested at a token boundary, so it cannot collide with an
/// int-type byte (`86 41 74` — the `41`/`74` are consumed inside the LOAD/LIT
/// arm) or a literal varint (consumed inside the `33` arm).
fn parse_expr(seg: &[u8], p: &mut usize, stop: u8) -> Result<Vec<IlOp>, Block> {
    // Big enough for every fixture body; a longer stream grows normally.
    let mut ops = Vec::with_capacity(16);
    loop {
        let b = *seg.get(*p).ok_or(blk(seg, *p, "expr"))?;
        if b == stop {
            break;
        }
        // An INTRINSIC CALL. Recognized as the two-token unit
        // `33 86 41 74 <id>` + `40` so the census can report *which* intrinsic
        // (`expr-intrinsic-memcpy`, `expr-intrinsic-base-upcast`, …) instead of
        // one 9 %-of-the-workload `expr-intrinsic-call` bucket. **Decoding is not
        // accepting**: this returns `Err` exactly as the old fall-through did, so
        // the gate is byte-for-byte unchanged — only the census key moves. See
        // `docs/IL_INTRINSIC_CALL.md` for why none of the family can be lowered
        // yet (the emission depends on the *literal argument values*, not on the
        // id: id 2114 with offset `00` is nothing at all, with offset `04` it is
        // a null-guarded `addi` plus a control-flow split).
        if let Some(id) = intrinsic_selector(seg, *p) {
            return Err(Block {
                ctx: "expr-intrinsic",
                byte: Some(0x40),
                off: *p,
                aux: id as u32,
            });
        }
        match b {
            0xB9 => {
                // LOAD <token> <int-type>
                let start = *p;
                *p += 1;
                let (tok, w) =
                    read_token_var(seg, *p).ok_or(blk(seg, *p, "expr-load-tok"))?;
                *p += w;
                if !eat_int_like(seg, p) {
                    // non-int-like operand → out of class. Report at the LOAD so
                    // the census bucket reads as a typed-operand gap, not a stray
                    // byte.
                    return Err(blk_type(seg, *p, start, "expr-load-type"));
                }
                ops.push(IlOp::Load(tok));
            }
            0x33 => {
                // LITERAL: 33 <int-type> <varint>
                let start = *p;
                *p += 1;
                if !eat_int_like(seg, p) {
                    return Err(blk_type(seg, *p, start, "expr-lit-type"));
                }
                ops.push(IlOp::Lit(
                    read_varint(seg, p).ok_or(blk(seg, *p, "expr-lit-varint"))?,
                ));
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
            _ => return Err(blk(seg, *p, "expr")),
        }
    }
    if ops.is_empty() {
        Err(blk(seg, *p, "expr-empty"))
    } else {
        Ok(ops)
    }
}

/// Parse the formal-parameter list of a straight-line leaf: after the `46` ('F')
/// marker (before the `LO` marker), a run of `2D <token>` entries emitted in
/// *reverse* of declaration order. An empty list is legitimate (a zero-param
/// `int konst(){return 42;}` still emits `46` immediately before `LO`).
///
/// The marker is located by requiring the region it opens to **end exactly on the
/// `LO` marker** — `46 (2D <tok>)*` and then `lo` — not by taking the first `0x46`
/// byte in the segment. That distinction is load-bearing, and taking the first
/// byte was a live bug:
///
/// * a function on **source line 70** carries the line marker `4F 01 46`, whose
///   payload byte *is* `0x46`. `fixtures/cpp/il_expr_deref.cpp` caught it — one of
///   sixteen otherwise-identical bodies (`ld_ixneg`, at line 70) silently got an
///   **empty** formals list, while its neighbours two lines away parsed fine;
/// * the per-function `4F 33 …` header region before the body is a run of opaque
///   bytes that varies with the function and freely contains `0x46`.
///
/// An empty formals list is not fail-closed: `leaves_ascending` skips tokens that
/// are not formals, so a body whose formals vanished bypasses the reassociation
/// ordering gate entirely. Getting the anchor right is therefore a safety fix, not
/// only a coverage one.
///
/// The earliest candidate that lands exactly on `lo` is taken. No candidate
/// *before* the true marker can span past it unless it lands on `lo`, because the
/// true marker's own `0x46` is neither `0x2D` nor a token continuation there.
fn parse_formals(seg: &[u8], lo: usize) -> Result<Vec<u32>, Block> {
    let mut best: Option<Vec<u32>> = None;
    for f in 0..lo {
        if seg[f] != 0x46 {
            continue;
        }
        let mut p = f + 1;
        let mut rev = Vec::new();
        let mut ok = true;
        while p < lo && seg.get(p) == Some(&0x2D) {
            p += 1;
            match read_token_var(seg, p) {
                Some((tok, w)) => {
                    p += w;
                    rev.push(tok);
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && p == lo {
            rev.reverse();
            best = Some(rev);
            break;
        }
    }
    best.ok_or(Block { ctx: "formals-marker", byte: None, off: lo, aux: 0 })
}

/// **The positive whole-body parser (W4b2-v).** Parse a single `.ex` function
/// segment as *exactly one* of the recognized [`BodyShape`]s, tokenizing
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
///   body   := 'LO'(4C 4F 11) 'SS'(53) stmt?  ( arith | vcall | icall )
///   stmt   := 4F 01 NN                                    (multi-fn only)
///   arith  := expr(→41)  <return int>                     LOAD:=B9 tok INT
///   vcall  := 26 tok  CALL  4C 4B  <return void>          LIT :=33 INT varint
///   icall  := 26 tok  CALL  expr(→55)  55 INT 4C  postop  <return int>
///   postop := ε | 33 INT k 02                             expr:=(LOAD|LIT|02|03|04)+
///   CALL   := BD <ret TYPE> <conv> <varint fn-type-id>    (8-13 bytes, decoded)
/// ```
/// The `CALL` line used to read `BD <3-byte ret type> 00 80 01 10 00 00 (fixed 10
/// bytes)`. That was never an anchor: the trailing value is a per-TU **function-type
/// id**, keyed on the signature and assigned in declaration order of distinct
/// function types, so `0x1001` is merely the first one a single-callee fixture TU
/// happens to create. Every field is self-delimiting and is decoded — see
/// [`parse_call_shape`].
/// `<return …>` is the shared plumbing consumed by [`eat_return_plumbing`]
/// (result-type for int, then assign/return/tail/segment-or-module end). An
/// `icall` is classified by its `postop`: **absent, or `+ 0`** → an integer
/// tail call [`BodyShape::IntTailCall`] (the argument `expr` computed into r3,
/// then `b <callee>`; `g(a)`, `g(a)+0`, `g(a+1)` all land here). A **non-zero
/// `+ k`** over a *bare passthrough* argument (`expr == [Load]`) → the framed
/// [`BodyShape::FramedCall`] (whose `k` fits a signed-16-bit `addi`). A non-zero
/// `+ k` over a *computed* argument (`g(a+1)+1`), or a `* k`/`- k`/wide `k`/a
/// second literal/a second call, all reject. The `callee` name is not in `.ex`;
/// the caller pairs it from `.gl`.
fn parse_segment(seg: &[u8], globals: &GlIndex<'_>) -> Option<BodyShape> {
    parse_segment_detail(seg, globals).ok()
}

/// [`parse_segment`] with the fail-closed *reason* preserved (P2b census).
/// Acceptance is identical — `parse_segment` is `.ok()` of this — so the census
/// can never disagree with the gate about what is in class.
fn parse_segment_detail(seg: &[u8], globals: &GlIndex<'_>) -> Result<BodyShape, Block> {
    let lo = find_subslice(seg, &[0x4C, 0x4F, 0x11]).ok_or(Block {
        ctx: "lo-marker",
        byte: None,
        off: 0,
        aux: 0,
    })?;
    let mut p = lo + 3;
    // 'SS' statement-start, then an optional statement/label marker.
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "stmt-start"));
    }
    eat_opt_stmt_marker(seg, &mut p);

    match *seg.get(p).ok_or(blk(seg, p, "body"))? {
        // An EMPTY body opens directly on the return plumbing's `3A` assign —
        // there is no expression at all. `eat_return_plumbing` still has to
        // reach the segment end, so any trailing statement or unexpected operand
        // fails the function closed exactly as it does for every other shape.
        0x3A => {
            eat_return_plumbing(seg, &mut p, false)?;
            Ok(BodyShape::EmptyBody)
        }
        // `26 <tok>` opens BOTH a call (the callee push) and an assignment
        // statement (the destination push), and the two are told apart by exactly
        // one byte: whether a `BD` CALL opcode follows the pushed token.
        //
        // Dispatching on that byte rather than trying the assignment parse and
        // falling back matters for the *measurement*, not for what is accepted.
        // Falling back meant every assignment-body refusal was re-reported as
        // whatever byte `parse_call_shape` then tripped over — nearly always the
        // RHS's `B9` — so `call-token-0xB9` was a conflated bucket holding pointer
        // operands, casts, `if` statements and more, all filed under a name that
        // described none of them. It has been the #1 entry at ~18% of blocked
        // functions and was directing the widening order at least twice this week.
        // Now each side reports its own reason.
        0x26 => {
            let mut probe = p + 1;
            let is_call = match read_token_var(seg, probe) {
                Some((_, w)) => {
                    probe += w;
                    seg.get(probe) == Some(&0xBD)
                }
                None => false,
            };
            if is_call {
                parse_call_shape(seg, &mut p, lo, None)
            } else {
                try_parse_assign_body_detail(seg, p, lo, globals)
            }
        }
        // Straight-line arithmetic opens with a LOAD or a bare literal — and so
        // does a W6 comparison leaf, which is tried first because its whole-body
        // shape is strictly more specific (a LOAD/LIT pair consumed by a
        // relational opcode). `try_parse_compare` is non-committal: it works on
        // a copy of the cursor and returns None without side effects, so a
        // non-comparison body falls through to the arithmetic parse unchanged.
        0xB9 | 0x33 => {
            if let Some(shape) = try_parse_compare(seg, p, lo) {
                return Ok(shape);
            }
            if let Some(shape) = try_parse_float_leaf(seg, p, lo) {
                return Ok(shape);
            }
            if let Some(shape) = try_parse_indirect_load_leaf(seg, p, lo) {
                return Ok(shape);
            }
            let ops = parse_expr(seg, &mut p, 0x41)?;
            eat_return_plumbing(seg, &mut p, true)?;
            let params = parse_formals(seg, lo)?;
            // A parameter used twice licenses c2's algebraic rewriter.
            if has_repeated_leaf(&ops) {
                return Err(Block { ctx: "expr-repeated-leaf", byte: None, off: p, aux: 0 });
            }
            // Gates that used to live in codegen; see `straight_line_is_out_of_class`.
            if straight_line_is_out_of_class(&ops, &params) {
                return Err(Block { ctx: "expr-out-of-class", byte: None, off: p, aux: 0 });
            }
            let ops = match canonicalize_chain(&ops, &params) {
                Some(c) => c,
                None => {
                    if !leaves_ascending(&ops, &params) {
                        return Err(Block {
                            ctx: "expr-noncanonical-order",
                            byte: None,
                            off: p,
                            aux: 0,
                        });
                    }
                    if !additive_chain_canonical(&ops) {
                        return Err(Block {
                            ctx: "expr-noncanonical-additive",
                            byte: None,
                            off: p,
                            aux: 0,
                        });
                    }
                    ops
                }
            };
            Ok(BodyShape::StraightLine { params, ops })
        }
        _ => Err(blk(seg, p, "body")),
    }
}

/// The largest substituted operand stream accepted, so that a chain of
/// assignments each doubling the previous cannot blow up.
const MAX_SUBST_OPS: usize = 32;

/// True if a straight-line integer body parses cleanly but `select_text` would
/// decline it anyway.
///
/// These gates used to live in codegen, and that broke a stated invariant: the
/// convention is that acceptance is decided in the parser so `function_census` and
/// `PortC2::build` cannot disagree about what is in class. While they sat in
/// codegen every shape below parsed as `straight-line` and was *counted*, then was
/// refused at emission — so the census numerator included functions the port cannot
/// emit. Fail-closed either way, but the census is the public claim and its
/// histogram is the widening order, so an inflated numerator is a real defect
/// rather than a cosmetic one.
///
/// Named rather than inlined so the test can assert the same predicate the parser
/// uses; the previous "census agrees with the gate" test compared `parse_segment`
/// with `parse_segment_detail`, which is `.ok()` of it, and so could not fail.
fn straight_line_is_out_of_class(ops: &[IlOp], params: &[u32]) -> bool {
    // More than eight integer parameters: the ninth is stack-homed.
    if params.len() > ARG_REG_COUNT {
        return true;
    }
    // `return b;` — a bare parameter that is not the first needs a register move.
    // `return a;` is free, since it is already in r3.
    if let [IlOp::Load(t)] = ops {
        if params.first() != Some(t) {
            return true;
        }
    }
    // A bare wide NEGATIVE constant: the `lis`+`ori` pair covers non-negative only.
    if let [IlOp::Lit(k)] = ops {
        if *k < -0x8000 {
            return true;
        }
    }
    // Multiply by a constant strength-reduces to shifts and adds, and `const - reg`
    // needs a `subfic`. The chain is left-associative, so an operator's right-hand
    // operand is the leaf just before it, and a leading `Lit` is the only way a
    // literal reaches an operator's left.
    for (i, op) in ops.iter().enumerate() {
        let rhs_lit = matches!(ops.get(i.wrapping_sub(1)), Some(IlOp::Lit(_)));
        let lhs_lit = matches!(ops.first(), Some(IlOp::Lit(_)));
        match op {
            IlOp::Mul if rhs_lit || (i == 1 && lhs_lit) => return true,
            IlOp::Sub if i == 2 && lhs_lit => return true,
            _ => {}
        }
    }
    false
}

/// True if any operand token is loaded more than once.
///
/// A repeated leaf licenses c2's algebraic rewriter, and it takes the licence:
/// `a + a` does **not** become `add r3,r3,r3`, it becomes `rlwinm r3,r3,1,0,30`
/// (`slwi r3,r3,1`) — byte-identical to what it emits for `a * 2`. So the operand
/// stream stops being a faithful description of the instructions.
///
/// This was a live mis-emit in the straight-line integer class, not a hypothetical:
/// `return a + a;` and `return a + b + a;` both produced wrong bytes, and had done
/// since that class was written, because no fixture used a parameter twice. The FP
/// leaf parser has had the equivalent gate from the start (see
/// [`try_parse_float_leaf`]); the integer path never got one.
///
/// Refusing is the conservative move: the rewrite set is not characterized (only
/// the `x + x` case is captured), so admitting any of it would be guessing.
fn has_repeated_leaf(ops: &[IlOp]) -> bool {
    let mut seen: Vec<u32> = Vec::new();
    for op in ops {
        if let IlOp::Load(t) = op {
            if seen.contains(t) {
                return true;
            }
            seen.push(*t);
        }
    }
    false
}

/// True if the operand LOADs appear in **strictly ascending parameter order** —
/// i.e. in ascending register order, since parameter `i` arrives in `r(3+i)`.
///
/// c2 does not evaluate a commutative chain in source order; it **canonicalizes
/// and reassociates** it by register. Every permutation of `a + b + c` — all five
/// of them — emits exactly `add r11,r3,r4 ; add r3,r11,r5`, and `b + a` emits the
/// same `add r3,r3,r4` as `a + b`. Mixed chains are reassociated too: `a + b - c`
/// and `b - c + a` both emit `subf r11,r5,r3 ; add r3,r11,r4`, which is `(a-c)+b`
/// — neither source grouping.
///
/// The port evaluates in source order, so it emitted numerically-correct but
/// byte-wrong code for every non-canonical chain. A generated differential sweep
/// over 600 integer expressions found ~20 of these, all in the straight-line class
/// that had been "byte-exact" since the MVP.
///
/// This gate is deliberately a **refusal, not a canonicalization**. The rewrite
/// rule is only partly characterized: the additive form looks like "start at the
/// lowest positive term, apply the negative terms in ascending order, then add the
/// remaining positives", but that is inferred from ten captures and implementing it
/// wrong would put the mis-emit straight back. Refusing is exact; a canonicalizer
/// needs its own capture matrix first (docs/GAPS.md).
///
/// Strictly ascending also implies no repeated leaf, so this subsumes
/// [`has_repeated_leaf`]; both are kept because they refuse for different reasons
/// and the census buckets should say which.
/// Rewrite a serial arithmetic chain into **c2's canonical order**, or return
/// `None` if the stream is not a shape this understands.
///
/// c2 does not evaluate a chain left to right. For an additive chain it treats the
/// whole thing as a sum of signed terms and emits, in order: the lowest-numbered
/// positive register, then every negative register ascending, then the remaining
/// positive registers ascending, then the folded literal. For a multiplicative
/// chain it simply sorts ascending. Captured:
///
/// ```text
///   a+b+c, a+c+b, b+a+c, b+c+a, c+b+a  ->  add r11,r3,r4 ; add r3,r11,r5
///   a*c*b                              ->  mullw r11,r3,r4 ; mullw r3,r11,r5
///   a + b - c   and   b - c + a        ->  subf r11,r5,r3 ; add r3,r11,r4
///   a - c - b                          ->  subf r11,r4,r3 ; subf r3,r5,r11
///   a + b - 1                          ->  add r11,r3,r4 ; addi r3,r11,-1
/// ```
///
/// Canonicalizing here rather than refusing is what makes every *permutation* of a
/// chain emit, instead of the one in six that happened to be written in register
/// order. It is done in the parser, not codegen, because the ordering key is the
/// parameter index — which is the register number — and because the census then sees
/// exactly what the emitter will.
///
/// Only a **serial** chain is handled: `leaf (leaf op)*`, all operators from one
/// family. A tree (`(a+b)*(c+d)`) is left untouched for `try_select_depth2_tree`,
/// and a mixed `*` with `+`/`-` is left to be refused downstream.
fn canonicalize_chain(ops: &[IlOp], params: &[u32]) -> Option<Vec<IlOp>> {
    // Recognize `leaf (leaf op)*` and split into signed terms.
    if ops.len() < 3 || ops.len() % 2 == 0 {
        return None;
    }
    let is_leaf = |o: &IlOp| matches!(o, IlOp::Load(_) | IlOp::Lit(_));
    if !is_leaf(&ops[0]) {
        return None;
    }
    let mut terms: Vec<(bool, IlOp)> = Vec::with_capacity(ops.len() / 2 + 1);
    terms.push((true, ops[0])); // (positive?, leaf)
    let mut mul = false;
    let mut addsub = false;
    let mut i = 1;
    while i + 1 < ops.len() + 1 && i + 1 <= ops.len() {
        if i + 1 > ops.len() - 1 {
            break;
        }
        let (leaf, op) = (ops[i], ops[i + 1]);
        if !is_leaf(&leaf) {
            return None;
        }
        match op {
            IlOp::Add => {
                addsub = true;
                terms.push((true, leaf));
            }
            IlOp::Sub => {
                addsub = true;
                terms.push((false, leaf));
            }
            IlOp::Mul => {
                mul = true;
                terms.push((true, leaf));
            }
            _ => return None,
        }
        i += 2;
    }
    if i != ops.len() || (mul && addsub) {
        return None;
    }
    // Order registers by parameter index; a non-parameter token is not orderable.
    let key = |o: &IlOp| match o {
        IlOp::Load(t) => params.iter().position(|p| p == t),
        _ => None,
    };
    if terms
        .iter()
        .any(|(_, l)| matches!(l, IlOp::Load(_)) && key(l).is_none())
    {
        return None;
    }
    // **The acceptance region of a rewrite rule must be a subset of the region that
    // was actually enumerated.** This rule was inferred from captures and is
    // validated by `scripts/expr_sweep.sh`, which enumerates chains of up to four
    // leaves; accepting longer ones would be emitting on extrapolation, which is
    // precisely how the per-chain accumulator bug survived (two rules that coincide
    // on short inputs). The multiplicative path is separately bounded by codegen's
    // r9 scratch floor, but the additive path is not bounded by anything — its
    // accumulator is r11 forever — so the bound has to be here.
    //
    // Raising this requires extending the sweep first, not the other way round.
    const MAX_SWEPT_TERMS: usize = 4;
    if terms.len() > MAX_SWEPT_TERMS {
        return None;
    }

    if mul {
        // A multiplicative chain: registers ascending. A literal factor
        // strength-reduces (shift/add), which is not modeled, so refuse those.
        if terms.iter().any(|(_, l)| matches!(l, IlOp::Lit(_))) {
            return None;
        }
        let mut regs: Vec<IlOp> = terms.iter().map(|(_, l)| *l).collect();
        regs.sort_by_key(|l| key(l));
        let mut out = Vec::with_capacity(ops.len());
        out.push(regs[0]);
        for r in &regs[1..] {
            out.push(*r);
            out.push(IlOp::Mul);
        }
        return Some(out);
    }

    // Additive chain. Fold the literals into one constant; order the registers.
    let mut k: i32 = 0;
    let mut pos: Vec<IlOp> = Vec::with_capacity(terms.len());
    let mut neg: Vec<IlOp> = Vec::with_capacity(terms.len());
    for (positive, leaf) in &terms {
        match leaf {
            IlOp::Lit(v) => {
                k = if *positive {
                    k.checked_add(*v)?
                } else {
                    k.checked_sub(*v)?
                }
            }
            IlOp::Load(_) => {
                if *positive {
                    pos.push(*leaf)
                } else {
                    neg.push(*leaf)
                }
            }
            _ => return None,
        }
    }
    // Needs a positive register to start from: `k - a` is a `subfic` shape that the
    // selector does not model.
    if pos.is_empty() {
        return None;
    }
    pos.sort_by_key(|l| key(l));
    neg.sort_by_key(|l| key(l));
    let mut out = Vec::with_capacity(ops.len() + 2);
    out.push(pos[0]);
    for n in &neg {
        out.push(*n);
        out.push(IlOp::Sub);
    }
    for p in &pos[1..] {
        out.push(*p);
        out.push(IlOp::Add);
    }
    if k != 0 {
        // `i32::MIN.abs()` panics in debug (and wraps in release), so refuse rather
        // than rely on a downstream checked-arithmetic catch. Reachable: the literal
        // varint has a 4-byte escape form, so `a + b + (-2147483648)` is encodable.
        let mag = k.checked_abs()?;
        out.push(IlOp::Lit(mag));
        out.push(if k > 0 { IlOp::Add } else { IlOp::Sub });
    }
    Some(out)
}

/// True if a `+`/`-` chain's source order already *is* c2's canonical order.
///
/// c2 does not evaluate an additive chain left to right. It treats it as a sum of
/// signed terms and emits the **negative** terms first, starting from the lowest
/// positive term, then adds the remaining positives. Captured:
///
/// ```text
///   a + b - c   ->  subf r11,r5,r3 ; add r3,r11,r4     i.e. (a - c) + b
///   b - c + a   ->  subf r11,r5,r3 ; add r3,r11,r4     the same bytes
///   a - c - b   ->  subf r11,r4,r3 ; subf r3,r5,r11    i.e. (a - b) - c
/// ```
///
/// So source order coincides with c2's only when every register subtraction comes
/// *before* every register addition. `a - b + c` and `a - b - c` satisfy that and
/// are byte-exact; `a + b - c` does not, and was a mis-emit — the port computed
/// `(a+b)-c` where c2 computes `(a-c)+b`.
///
/// A subtraction of a **literal** does not count: it folds into the running `addi`
/// immediate rather than emitting an instruction, so `a + b - 1` is fine.
///
/// The chain is left-associative, so in the postfix stream each operator's
/// right-hand operand is the leaf immediately preceding it — which is all the
/// context needed to tell a register operand from a folded literal.
fn additive_chain_canonical(ops: &[IlOp]) -> bool {
    // Depth bound. Even a chain already in canonical order — needing no rewrite, and
    // so taking the pre-canonicalizer path — mis-emits once it is long enough: with
    // three or more register subtractions followed by an addition, c2's intermediate
    // allocation diverges again. Measured: every 4-leaf chain is byte-exact (11,664
    // enumerated), `a - b - c - d` is byte-exact, and `a - b - c - d + e` is not.
    //
    // Nothing else bounded this. The multiplicative path stops at codegen's r9
    // scratch floor, but an additive chain's accumulator is r11 forever, so a chain
    // of any length was accepted on extrapolation from short ones — the same shape as
    // the per-chain accumulator bug. Pure additions and pure multiplications are left
    // alone (5-leaf forms of both are verified); the bound applies only where a
    // subtraction is present, which is where the divergence was found.
    //
    // Raising it requires extending the sweep first.
    const MAX_VERIFIED_LEAVES_WITH_SUB: usize = 4;
    let has_sub = ops.iter().any(|o| matches!(o, IlOp::Sub));
    if has_sub {
        let leaves = ops
            .iter()
            .filter(|o| matches!(o, IlOp::Load(_) | IlOp::Lit(_)))
            .count();
        if leaves > MAX_VERIFIED_LEAVES_WITH_SUB {
            return false;
        }
    }
    let mut reg_add_seen = false;
    for (i, op) in ops.iter().enumerate() {
        let rhs_is_reg = matches!(ops.get(i.wrapping_sub(1)), Some(IlOp::Load(_)));
        match op {
            IlOp::Add if rhs_is_reg => reg_add_seen = true,
            IlOp::Sub if rhs_is_reg && reg_add_seen => return false,
            _ => {}
        }
    }
    true
}

fn leaves_ascending(ops: &[IlOp], params: &[u32]) -> bool {
    let mut last: Option<usize> = None;
    for op in ops {
        if let IlOp::Load(t) = op {
            // A LOAD whose token is not a formal is **refused**, not skipped. The
            // gate orders by parameter index, so an unorderable operand means it
            // cannot do its job — and skipping was a real hole: `parse_formals`
            // used to anchor on the first `0x46` before `LO`, which a source-line
            // marker's payload (`4F 01 46`, a function on line 70) or the
            // per-function header region can supply, and it then returned an
            // *empty* formals list instead of failing. Any body that hit that
            // bypassed this gate entirely. The anchoring is fixed, but the gate
            // must fail closed regardless rather than depend on it.
            let Some(ix) = params.iter().position(|p| p == t) else {
                return false;
            };
            if let Some(prev) = last {
                if ix <= prev {
                    return false;
                }
            }
            last = Some(ix);
        }
    }
    true
}

/// Inline-substitute every `Load(t)` for which `env` has a definition.
///
/// The stream is postfix, so splicing a multi-op definition in place of a single
/// `Load` is valid without any bracketing: `[Load(x), Lit(1), Add]` with
/// `x -> [Load(a), Lit(1), Add]` becomes `[Load(a), Lit(1), Add, Lit(1), Add]`,
/// which is `(a+1)+1`.
///
/// One pass suffices because every `env` entry is *itself* already substituted —
/// entries are recorded at definition time, in terms of parameters only. That is
/// also what makes this correct rather than merely convenient: substituting at
/// definition time captures the operand values as of that point, so a later
/// redefinition of an operand cannot leak backwards. `int x = a; a = a + 1;
/// return x;` yields `x -> [Load(a)]` and returns the *entry* `a`, which is right;
/// substituting lazily at use time would return `a + 1`, which is not.
fn substitute(ops: &[IlOp], env: &[(u32, Vec<IlOp>)]) -> Option<Vec<IlOp>> {
    let mut out = Vec::with_capacity(ops.len());
    for op in ops {
        match op {
            IlOp::Load(t) => match env.iter().find(|(k, _)| k == t) {
                Some((_, def)) => out.extend_from_slice(def),
                None => out.push(*op),
            },
            _ => out.push(*op),
        }
        if out.len() > MAX_SUBST_OPS {
            return None;
        }
    }
    Some(out)
}

/// Try to parse a body that is a **list of assignment statements** followed by a
/// returned expression:
///
/// ```text
///   body := ( `4F 01 <line>`* assign )* `4F 01 <line>`* expr(→41) <return int>
///   assign := `26 <dst>` expr(→32) `32 <TYPE>` `4B`
/// ```
///
/// `26 <dst>` pushes the destination, `32 <TYPE>` stores it (and yields the value,
/// which `4B` then discards). The `<TYPE>` is the destination's own — a conversion
/// is always a separate visible `2C`, so an int-like type here means no conversion.
///
/// **These bodies need no stores at all.** c2 register-allocates locals and
/// coalesces the copies, so the whole class collapses to the expression that
/// actually reaches the `return`. Captured:
///
/// ```text
///   int x; x = a; return x;              -> blr            (x is already r3)
///   int x = a; int y = x; return y;      -> blr
///   a = a + 1; return a;                 -> addi r3,r3,1
///   int x = 0; x = a + 1; return x;      -> addi r3,r3,1    (the x = 0 is dead)
///   a = 7; return a;                     -> li r3,7
/// ```
///
/// So this resolves the statement list by substitution and hands codegen the
/// resulting straight-line expression, which is exactly what the reference emits.
///
/// The destination must be a **formal**, established positively from the `2D` list.
///
/// An earlier version asked whether `.gl` named the destination and refused if so.
/// That looked sound and was not: a file-scope `static int sv` appears there as
/// `$sv`, whose leading `$` `gl_symbol_index` does not accept as an identifier, so
/// the token looked local and the store was silently dropped. Absence from a symbol
/// table proves nothing — it only says the table did not happen to name it.
///
/// Locals are consequently out of class: `.ex` uses the same `26 <tok>` push for
/// parameter, local and global alike, so admitting them needs a positive local
/// signal that does not exist yet.
fn try_parse_assign_body_detail(
    seg: &[u8],
    start: usize,
    lo: usize,
    globals: &GlIndex<'_>,
) -> Result<BodyShape, Block> {
    let mut p = start;
    let mut env: Vec<(u32, Vec<IlOp>)> = Vec::new();
    // Read once for the per-destination check. A body with no formals marker is not
    // rejected here — the destination check below refuses it anyway, and deferring
    // lets the right-hand side report its own reason first, which is what makes the
    // census name the innermost unmodeled construct rather than this outer gate.
    let formals = parse_formals(seg, lo).unwrap_or_default();
    let _ = globals;
    loop {
        eat_opt_stmt_marker(seg, &mut p);
        if *seg.get(p).ok_or(blk(seg, p, "assign-stmt"))? != 0x26 {
            break;
        }
        let mut probe = p + 1;
        let (dst, w) =
            read_token_var(seg, probe).ok_or(blk(seg, probe, "assign-dst-tok"))?;
        probe += w;
        // `BD` here means this `26` was a callee push, not a destination. The caller
        // dispatched on the FIRST one, so reaching this means the right-hand side is
        // itself a call: `int z = g(a); …`. When the very next thing is a return of
        // that same local the whole body is a tail call, which `parse_call_shape`
        // already handles given the bound token — so hand it over rather than refuse.
        //
        // Only valid as the FIRST statement: with `env` non-empty, earlier
        // assignments have already been folded away and would be lost.
        // The right-hand side is a call when it opens with its own `26 <callee>`
        // followed by the `BD` CALL opcode — two tokens along from the destination,
        // not one.
        let rhs_is_call = *seg.get(probe).ok_or(blk(seg, probe, "assign-op"))? == 0x26
            && match read_token_var(seg, probe + 1) {
                Some((_, cw)) => seg.get(probe + 1 + cw) == Some(&0xBD),
                None => false,
            };
        if rhs_is_call {
            if env.is_empty() {
                let mut q = probe;
                return parse_call_shape(seg, &mut q, lo, Some(dst));
            }
            return Err(blk(seg, probe, "assign-rhs-call"));
        }
        p = probe;
        let rhs = parse_expr(seg, &mut p, 0x32)?;
        // The destination must be a **formal**, established positively from the
        // `2D` list — not "absent from `.gl`", which is what this used to test and
        // which does not work.
        //
        // A store to any memory object (a global, or a file-scope `static`) is a
        // real write with a relocation, and treating it as a register copy silently
        // drops it. The absence test failed exactly there: a `static int sv` is in
        // `.gl` as `$sv`, whose leading `$` `gl_symbol_index` does not accept as an
        // identifier, so the token looked local and
        // `static int sv; int f(int a){ sv = a; return a; }` mis-emitted. Found by
        // probing the de-conflated census, not by a fixture.
        //
        // Locals are therefore out of class for now. There is no positive local
        // signal in `.ex` — the statement grammar uses the same `26 <tok>` push for
        // parameter, local and global alike — so admitting them needs a local-symbol
        // production first. The coverage given up is measured at ~0 on the real
        // workload, which is not a reason to keep a mis-emit.
        if !formals.contains(&dst) {
            return Err(Block { ctx: "assign-dst-not-formal", byte: None, off: probe, aux: 0 });
        }
        if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) {
            return Err(blk(seg, p, "assign-store-type"));
        }
        // `4B` ends an expression statement and discards the yielded value. A
        // body that *uses* it (`x = y = a`) does not have one here and refuses.
        if !eat_byte(seg, &mut p, 0x4B) {
            return Err(blk(seg, p, "assign-stmt-end"));
        }
        let rhs = substitute(&rhs, &env)
            .ok_or(Block { ctx: "assign-subst-overflow", byte: None, off: p, aux: 0 })?;
        // Re-assigning shadows the previous definition, which is how a dead store
        // disappears: only the last definition can reach the return.
        env.retain(|(t, _)| *t != dst);
        env.push((dst, rhs));
        if env.len() > MAX_SUBST_OPS {
            return Err(Block { ctx: "assign-too-many-locals", byte: None, off: p, aux: 0 });
        }
    }
    eat_opt_stmt_marker(seg, &mut p);
    let ret = parse_expr(seg, &mut p, 0x41)?;
    let ret = substitute(&ret, &env)
        .ok_or(Block { ctx: "assign-subst-overflow", byte: None, off: p, aux: 0 })?;
    eat_return_plumbing(seg, &mut p, true)?;
    let params = parse_formals(seg, lo)?;
    // After substitution every remaining LOAD must be a parameter. Anything else
    // is a read of something this class cannot account for — an uninitialized
    // local, a global, or a token from a construct not modeled here.
    if !ret.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    }) {
        return Err(Block { ctx: "assign-ret-nonformal", byte: None, off: p, aux: 0 });
    }
    // Substitution is a *source* of repeated leaves even when the written source
    // has none: `int x = a; x = x + x;` substitutes to `a + a`, which c2 emits as
    // `slwi r3,r3,1`. This gate is what keeps that from being wrong bytes.
    if has_repeated_leaf(&ret) {
        return Err(Block { ctx: "assign-repeated-leaf", byte: None, off: p, aux: 0 });
    }
    // Substitution reorders too: `int x = b; return x + a;` resolves to `b + a`.
    if !leaves_ascending(&ret, &params) || !additive_chain_canonical(&ret) {
        return Err(Block { ctx: "assign-noncanonical-order", byte: None, off: p, aux: 0 });
    }
    Ok(BodyShape::StraightLine { params, ops: ret })
}

/// The `unsigned int` operand type encoding inline in the `.ex` body.
/// Distinguished from [`INT_TYPE`] only by its last two bytes; the relational
/// opcodes are sign-agnostic, so this triple is the *only* thing that says a
/// comparison is unsigned.
const UINT_TYPE: [u8; 3] = [0x86, 0x42, 0x75];

/// `long` (`86 41 12`) and `unsigned long` (`86 42 22`). On this target they are
/// 32-bit, and c2 emits **byte-identical** code for them and for `int`/`unsigned`
/// — see `docs/IL_TYPE_TAGS.md` §3.1.
const LONG_TYPE: [u8; 3] = [0x86, 0x41, 0x12];
const ULONG_TYPE: [u8; 3] = [0x86, 0x42, 0x22];

/// The 32-bit integer operand types that are interchangeable *for the operators
/// this parser accepts* (`+`, `-`, `*` and add-immediate folding).
///
/// Signedness is not a distinction PPC's `add`/`subf`/`mullw` make, and the
/// captures bear that out: `a+b+c`, `a-b`, `a*b*c` and `a+7` each produce exactly
/// the same words for `int`, `unsigned`, `long` and a mixed `int`/`unsigned`
/// expression (`docs/IL_TYPE_TAGS.md` §3.1). It is **not** a general licence to
/// ignore signedness — division and the shift-right family do differ, and both
/// are refused elsewhere — nor does it extend to the narrow types, whose
/// extension placement depends on the operator *and* the result type (§3.2).
const INT_LIKE_TYPES: [[u8; 3]; 4] = [INT_TYPE, UINT_TYPE, LONG_TYPE, ULONG_TYPE];

/// Integer argument registers r3..r10. A ninth parameter is stack-homed, which
/// needs a frame; mirrors `c2_core::codegen::ARG_REGS`.
const ARG_REG_COUNT: usize = 8;

/// Consume any one of [`INT_LIKE_TYPES`] at `p`, reporting whether it matched.
fn eat_int_like(seg: &[u8], p: &mut usize) -> bool {
    INT_LIKE_TYPES.iter().any(|t| eat(seg, p, t))
}

/// The `float` operand type (`86 45 40`) and the `double` one (`88 85 41`).
/// Note the *literal* forms differ again ([`FLOAT_LIT_TYPE`] /
/// [`DOUBLE_LIT_TYPE`]).
const FLOAT_TYPE: [u8; 3] = [0x86, 0x45, 0x40];
const DOUBLE_TYPE: [u8; 3] = [0x88, 0x85, 0x41];

/// The *literal* FP type tags, which are distinct from the operand ones above.
/// A float literal carries `86 4a 40`, a double one `88 8a 41`.
const FLOAT_LIT_TYPE: [u8; 3] = [0x86, 0x4A, 0x40];
const DOUBLE_LIT_TYPE: [u8; 3] = [0x88, 0x8A, 0x41];

/// Try to parse a **W13a floating-point leaf**: a straight-line chain over
/// float (or double) *parameters* only.
///
/// ```text
///   ( B9 <tok> <FT> | <op> )+     LOADs and binary ops, all of one FP type
///   41 <FT>                       result type, the SAME FP type
///   <return plumbing>
/// ```
///
/// The gate list is from `docs/CODEGEN_W13_FLOAT.md` §6 and every item is a
/// case where a naive selector emits *wrong* bytes rather than merely running
/// out of range:
///
/// * **No literal.** Every FP constant costs an `.rdata` COMDAT, a REFHI/REFLO
///   relocation pair and a GPR — that is W13b.
/// * **No `2C` convert**, and no mixing of float with double: a mixed-width
///   expression evaluates in double and may need an `frsp`.
/// * **No `*` under `+`/`-`.** Contraction to `fmadds`/`fmsubs`/`fnmsubs` is
///   *mandatory* in c2, so emitting the two separate instructions would be a
///   silent mis-emit. Approximated conservatively here by rejecting any chain
///   that contains both a `Mul` and an `Add`/`Sub`.
/// * **No repeated leaf.** `a + a` is algebraically rewritten to `a * 2.0f`,
///   which is a constant and therefore `.rdata` again.
/// * **No `0x59` marker.** It tracks source parenthesisation and is the only
///   thing distinguishing product shapes c2 flattens from those it does not;
///   its meaning is unknown, so its presence rejects.
fn try_parse_float_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    // The operand type is fixed by the first LOAD and every later one must match.
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    let double = {
        let mut probe = p + 1;
        let (_, w) = read_token_var(seg, probe)?;
        probe += w;
        if seg.get(probe..probe + 3)? == FLOAT_TYPE {
            false
        } else if seg.get(probe..probe + 3)? == DOUBLE_TYPE {
            true
        } else {
            return None;
        }
    };
    let fty = if double { DOUBLE_TYPE } else { FLOAT_TYPE };

    let mut ops: Vec<IlOp> = Vec::new();
    loop {
        match *seg.get(p)? {
            0xB9 => {
                p += 1;
                let (tok, w) = read_token_var(seg, p)?;
                p += w;
                if seg.get(p..p + 3)? != fty {
                    return None; // mixed width, or a non-FP operand
                }
                p += 3;
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
            0x05 => {
                p += 1;
                ops.push(IlOp::Div);
            }
            // W13b: a floating-point literal.
            //
            //   33 <lit-TYPE> <8 bytes: IEEE binary64, little-endian> <u16 size>
            //
            // The payload is a binary64 pattern even for a `float` (already
            // rounded to binary32 precision), and the u16 trailer is the operand
            // *width* — 4 for float, 8 for double — which must agree with the
            // literal tag. Verified byte-for-byte against a live capture of
            // `float k_add(float a){return a + 1.0f;}`:
            //   33 86 4a 40 00 00 00 00 00 00 f0 3f 04 00
            0x33 => {
                p += 1;
                let lty = seg.get(p..p + 3)?;
                let lit_double = if lty == FLOAT_LIT_TYPE {
                    false
                } else if lty == DOUBLE_LIT_TYPE {
                    true
                } else {
                    return None; // an integer (or other) literal: out of class
                };
                // A literal of the other width implies a conversion.
                if lit_double != double {
                    return None;
                }
                p += 3;
                let raw: [u8; 8] = seg.get(p..p + 8)?.try_into().ok()?;
                p += 8;
                let size = u16::from_le_bytes(seg.get(p..p + 2)?.try_into().ok()?);
                p += 2;
                if size as usize != if double { 8 } else { 4 } {
                    return None;
                }
                ops.push(IlOp::FpLit {
                    bits: u64::from_le_bytes(raw),
                    double,
                });
            }
            0x41 => break,
            // 0x2C convert, 0x59 paren marker, 0x08 neg and every other byte
            // reject — see the gate list above.
            _ => return None,
        }
    }
    // Result type must be the same FP type.
    p += 1;
    if seg.get(p..p + 3)? != fty {
        return None;
    }
    p += 3;
    eat_return_plumbing(seg, &mut p, false).ok()?;

    // A `*` mixed with `+`/`-` contracts; reject rather than emit two
    // instructions where c2 emits one.
    let has_mul = ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = ops.iter().any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    if has_mul && has_addsub {
        return None;
    }

    // ---- W13b constant gates ------------------------------------------------
    //
    // These live here, in the parser, rather than in codegen so that the census
    // and the emission gate cannot disagree about what is in class.
    //
    // c2 — not c1xx — evaluates floating-point constants, so the IL still holds
    // every literal the source wrote and the backend is free to fold, reassociate
    // and strength-reduce them. Three captured behaviours the port does not
    // model, each of which would be a silent mis-emit:
    let lits: Vec<(u64, bool)> = ops
        .iter()
        .filter_map(|o| match o {
            IlOp::FpLit { bits, double } => Some((*bits, *double)),
            _ => None,
        })
        .collect();
    if !lits.is_empty() {
        // (1) Two or more literals: c2 folds them where it can (`a*2.0f*b*3.0f`
        //     becomes `(a*b)*6.0f`), and where it cannot it hoists every `addis`
        //     into a prologue group and schedules the loads at first use. Either
        //     way the one-constant lowering is wrong. See `w13b_fpool.cpp`.
        if lits.len() > 1 {
            return None;
        }
        // (2) A constant divisor becomes a reciprocal multiply: `a/2.0f` emits
        //     `fmuls` against `__real@3f000000`, and `a/3.0f/7.0f` collapses to
        //     one `fmuls` by 1/21 — a value that is not even exactly
        //     representable, so this is a numeric transform, not a rewrite.
        if ops.iter().any(|o| matches!(o, IlOp::Div)) {
            return None;
        }
        let (bits, lit_double) = lits[0];
        let v = f64::from_bits(bits);
        // (3) An identity operand disappears entirely — `a + 0.0f`, `a - 0.0f`
        //     and `a * 1.0f` each compile to a bare `blr`, with no constant
        //     pooled at all. (`a * 0.0f` is *not* folded: it really does load
        //     zero and multiply.) Refuse when the literal is an identity for any
        //     operator in the body; slight over-refusal beats emitting three
        //     instructions where c2 emits none.
        if v == 0.0 && has_addsub {
            return None;
        }
        if v == 1.0 && has_mul {
            return None;
        }
        // (4) A `float` literal is carried as a binary64 pattern already rounded
        //     to binary32. If it does not narrow exactly, the four bytes we would
        //     pool are not the ones c2 pooled.
        if !lit_double && f64::from(v as f32).to_bits() != bits {
            return None;
        }
    }
    // FP chains are canonicalized by register exactly as integer ones are: `b + a`
    // and `b * a` emit the operands in ascending order, and every permutation of
    // `a + b + c` emits one stream. The port evaluated source order, so all of those
    // were mis-emits until the generated sweep found them.
    //
    // Division is tighter still. One division as the *only* operator is byte-exact
    // (`a / b`, `b / a` — it is non-commutative, so order is preserved), but two
    // divisions (`a / b / c`) or a division mixed with anything else (`a + b / c`)
    // are not what the serial model emits. Both refuse.
    let n_binops = ops
        .iter()
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. }))
        .count();
    if ops.iter().any(|o| matches!(o, IlOp::Div)) && n_binops != 1 {
        return None;
    }
    // A repeated leaf can trigger algebraic rewriting into a constant.
    let mut seen: Vec<u32> = Vec::new();
    for o in &ops {
        if let IlOp::Load(t) = o {
            if seen.contains(t) {
                return None;
            }
            seen.push(*t);
        }
    }
    let params = parse_formals(seg, lo).ok()?;
    if params.len() > 13 || !seen.iter().all(|t| params.contains(t)) {
        return None;
    }
    // c2 canonicalizes a chain containing a **commutative** operator by register,
    // exactly as it does an integer one, so such a chain must already be written in
    // ascending order. A chain with only non-commutative operators is left alone —
    // `b - a` and `b / a` really do emit their operands in source order, and gating
    // them would refuse bodies that are byte-exact today.
    let has_commutative = ops
        .iter()
        .any(|o| matches!(o, IlOp::Add | IlOp::Mul));
    if has_commutative && !leaves_ascending(&ops, &params) {
        return None;
    }
    Some(BodyShape::FloatLeaf { params, ops, double })
}

/// A TYPE's implied width in bytes. The tag's low nibble is
/// `2 * (log2(size) + 1)`, so `…2`→1, `…4`→2, `…6`→4, `…8`→8, and it is the same
/// in every observed tag family (`86`, `A6` const, `96` volatile, `82`/`84`/`88`).
/// Verified across every triple in `docs/IL_TYPE_TAGS.md` §2 and against the
/// pointee-width tags this document's `27` captures produced
/// (`27 82 43 f0 08` for a `char` member, `27 88 43 c1 08` for a `double` one).
fn type_width(tag: u8) -> Option<u32> {
    match tag & 0x0F {
        0x2 => Some(1),
        0x4 => Some(2),
        0x6 => Some(4),
        0x8 => Some(8),
        _ => None,
    }
}

/// True for a TYPE naming a **4-byte integer** value in any cv-qualification:
/// `kind`'s low nibble is 1 (signed) or 2 (unsigned) and the tag says 4 bytes.
/// Captured witnesses: `86 41 74` int, `86 42 75` unsigned, `86 41 12` long,
/// `86 42 22` unsigned long, `A6 41 84 20` const int, `96 41 86 20` volatile int.
///
/// This admits exactly the set over which a following `2C` to an int-like target
/// is *provably* a no-op (`docs/IL_CAST_CONVERT.md` §2.2/§4.2: int↔unsigned and
/// cv-strip at the same width emit nothing), which is what lets the cv-qualified
/// forms be accepted at all — a `const` member getter's load carries
/// `30 A6 41 …` followed by `2C 86 41 74 00`, and both captures
/// (`const int *`, `volatile int *`) emit a bare `lwz` with nothing added.
fn is_int4_type(tag: u8, kind: u8) -> bool {
    type_width(tag) == Some(4) && matches!(kind & 0x0F, 0x1 | 0x2)
}

/// True for a TYPE naming a **pointer to a 4-byte object**: `kind`'s low nibble
/// is 3 and the tag says 4. In a `B9` operand position a pointer's tag is the
/// *pointer's* own width (`86 43 f4 08` = `int *`); in the `27` byte-offset-add
/// position it is the **pointee's** width instead (`82 43 f0 08` for `char *`,
/// `88 43 c1 08` for `double *`), which is why this is applied to the `27` type
/// and not only to the base LOAD.
fn is_ptr_to_4(tag: u8, kind: u8) -> bool {
    type_width(tag) == Some(4) && (kind & 0x0F) == 0x3
}

/// The member-function `this` token, when this segment's pre-body region binds
/// one: `53 53 26 <fn> B9 <this> <TYPE> 99 <TYPE> 00 46`.
///
/// `this` is **not** in the `2D` formals list, and it occupies r3 — so every
/// explicit formal of a member function is one register higher than
/// [`parse_formals`]'s index implies. Captured, and it is a live off-by-one trap
/// for anything that maps formals to registers:
///
/// ```text
/// int C::g(int* q) const        { return *q; }   -> lwz r3,0(r4)   q is r4, not r3
/// int C::i(int v, int* q) const { return *q; }   -> lwz r3,0(r5)   q is r5, not r4
/// int D::s(int* q)              { return *q; }   -> lwz r3,0(r3)   static: no `this`
/// ```
///
/// Located by parsing *backwards to a fixed end*: the candidate must decode as
/// `B9 <tok> <TYPE> 99 <TYPE> 00` and finish **exactly** on the `46` formals
/// marker. If no candidate or more than one does, this returns `None` and the
/// caller refuses — a guessed `this` would pick the wrong base register.
///
/// Note that `99`'s trailing field is a one-byte varint while the visually
/// similar `9B`'s is a whole `read_token_var`; see `docs/IL_EXPR_LAYER.md` §7.
fn parse_this_token(seg: &[u8], lo: usize) -> Option<u32> {
    let f = find_byte(&seg[..lo], 0x46)?;
    let mut found: Option<u32> = None;
    for q in 0..f {
        if seg[q] != 0xB9 {
            continue;
        }
        let mut p = q + 1;
        let (tok, w) = match read_token_var(seg, p) {
            Some(x) => x,
            None => continue,
        };
        p += w;
        let tw = match read_type(seg, p) {
            Some((_, _, _, w)) => w,
            None => continue,
        };
        p += tw;
        if seg.get(p) != Some(&0x99) {
            continue;
        }
        p += 1;
        let tw = match read_type(seg, p) {
            Some((_, _, _, w)) => w,
            None => continue,
        };
        p += tw;
        if seg.get(p) != Some(&0x00) {
            continue;
        }
        p += 1;
        if p != f {
            continue;
        }
        if found.is_some() {
            return None; // ambiguous: refuse rather than pick
        }
        found = Some(tok);
    }
    found
}

/// Try to parse an **indirect-load leaf**: a whole body that is one load through
/// a pointer, `return *p;` / `return s->m;` / `return p[k];` and nothing else.
///
/// ```text
///   B9 <base-tok> <PTR-TYPE>                     the base pointer
///   [ 33 <int-like> <off>  27 <PTR-TYPE> ]       ONE member byte-offset add, or
///   [ 33 <long>     <off>  28 00 00      ]       ONE subscript byte-offset add
///   30 <INT4-TYPE>                               the indirect load
///   [ 2C <int-like> 00 ]                         a cv-qualification strip
///   41 <int-like>                                result type
///   <return plumbing, reaching the segment end>
/// ```
///
/// c2 lowers all of it to **one `lwz rD, off(rBase)`** plus the `blr`, folding the
/// offset into the displacement. Captured, one instruction each:
///
/// ```text
/// int f(int* p)                { return *p; }      -> lwz r3,0(r3)
/// int f(int a, int* p)         { return *p; }      -> lwz r3,0(r4)
/// int f(int a, int b, int* p)  { return *p; }      -> lwz r3,0(r5)
/// int f(S* s)                  { return s->d; }    -> lwz r3,16(r3)     (27, off 0x10)
/// int f(int* p)                { return p[3]; }    -> lwz r3,12(r3)     (28, off 0x0c)
/// int f(int* p)                { return p[-1]; }   -> lwz r3,-4(r3)     (off 0xfc = -4)
/// int f(int* p)                { return p[8000]; } -> lwz r3,32000(r3)
/// int C::f() const             { return b; }       -> lwz r3,4(r3)      (`this`)
/// unsigned/long/const/volatile int *                -> the same bare `lwz`
/// ```
///
/// Why every gate below is load-bearing rather than defensive — each is a
/// *captured* case where the same-looking IL lowers differently:
///
/// * **Exactly one offset add.** `p[i][j]` chains two of them and needs
///   `slwi ; add ; slwi ; lwzx`; `p[i].b` chains a `28` and a `27`.
/// * **The offset must fit the 16-bit displacement.** `p[100000]` (offset 400000)
///   is `lis r11,6 ; ori r11,r11,0x1a80 ; lwzx r3,r3,r11` instead.
/// * **The offset must be a literal.** A variable index is
///   `slwi r11,r4,2 ; lwzx r3,r11,r3` — a different instruction, an extra one, and
///   a scratch register.
/// * **The `28` payload must be exactly `00 00`.** Those two bytes are `00 00` at
///   every site captured (constant and variable indices, 1/4/8-byte elements,
///   negative indices, 2-D arrays, bitfields) and their meaning is UNKNOWN, so
///   anything else refuses.
/// * **The loaded type must be a 4-byte integer.** `char *` is `lbz`, `short *`
///   is `lhz`, `float *` is `lfs`, `double *` is `lfd` — all captured, all
///   different instructions.
/// * **Nothing may follow the load but the return.** `*p + 1` puts the load in
///   r11, and `*p * 3` is strength-reduced; see [`IlOp::LoadInd`].
/// * **A `this`-bearing function must have its `this` found**, because `this`
///   takes r3 and shifts every explicit formal up one
///   ([`parse_this_token`]).
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
fn try_parse_indirect_load_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // The base pointer LOAD.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (base_tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_to_4(tag, kind) {
        return None;
    }
    p += tw;

    // At most ONE byte-offset add, in either of its two forms. Both push a
    // byte offset as a literal and add it to the designator; `27` re-types the
    // result and `28` does not (`docs/IL_EXPR_LAYER.md` §4).
    let mut off: i32 = 0;
    if *seg.get(p)? == 0x33 {
        let mut probe = p + 1;
        // The literal's own type: `86 41 74` (int) for a member offset,
        // `86 41 12` (long) for a subscript offset. Both are int-like.
        if !eat_int_like(seg, &mut probe) {
            return None;
        }
        let k = read_varint(seg, &mut probe)?;
        match *seg.get(probe)? {
            0x27 => {
                probe += 1;
                let (tag, kind, _, tw) = read_type(seg, probe)?;
                if !is_ptr_to_4(tag, kind) {
                    return None;
                }
                probe += tw;
            }
            0x28 => {
                // The two trailing bytes are `00 00` at every captured site and
                // are not understood; anything else refuses.
                probe += 1;
                if !eat(seg, &mut probe, &[0x00, 0x00]) {
                    return None;
                }
            }
            _ => return None,
        }
        off = k;
        p = probe;
    }
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }

    // The indirect load itself.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_int4_type(tag, kind) {
        return None;
    }
    p += tw;

    // An optional cv-qualification strip. Provably free over a 4-byte integer
    // source (see [`is_int4_type`]); the target must still be int-like, and the
    // trailing varint must be the `00` observed at all 14,098 aligned sites.
    if *seg.get(p)? == 0x2C {
        let mut probe = p + 1;
        if !eat_int_like(seg, &mut probe) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    // Result type, then the shared plumbing, which must reach the segment end.
    if !eat_byte(seg, &mut p, 0x41) || !eat_int_like(seg, &mut p) {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false).ok()?;

    // Bind the base to its argument register. `this` is argument 0, and when it
    // is present every explicit formal shifts up one.
    let formals = parse_formals(seg, lo).ok()?;
    let params = match parse_this_token(seg, lo) {
        Some(this_tok) => {
            let mut v = vec![this_tok];
            v.extend_from_slice(&formals);
            v
        }
        None => formals,
    };
    let ix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    Some(BodyShape::IndirectLoad {
        params,
        ops: vec![IlOp::Load(base_tok), IlOp::LoadInd { off }],
    })
}

/// Try to parse a **W6 comparison leaf** body: `return <formal> <rel> <k>;`.
///
/// ```text
///   B9 <tok> <T>        LOAD the formal          T ∈ {int, unsigned}
///   33 <T> <varint>     LITERAL k, same type T
///   <rel>               1F|20|21|22|23|24
///   2C <R> 00           convert bool → R         R ∈ {int, unsigned}
///   41 <R>              result type
///   <return plumbing>
/// ```
///
/// Fail-closed specifics that are load-bearing rather than incidental:
///
/// * The two operand types must be **equal**. c1xx always inserts a conversion
///   first, so a mismatch has never been observed; rejecting it is a cheap
///   assertion, not a dropped feature.
/// * The `2C` convert is accepted **only here**, directly over a comparison
///   result. The identical token over a narrow-integer LOAD is a real
///   `extsb`/`extsh` sign-extension, so a blanket "`2C` is free" rule would
///   silently drop those instructions.
/// * The parse must reach the segment end via the shared return plumbing, so a
///   trailing statement, a second comparison, or an arithmetic post-op (e.g.
///   `return (a > 7) + 1;`, which retargets the spine's last instruction) all
///   reject the whole function.
///
/// Returns `None` — leaving the caller's cursor untouched — for anything that is
/// not exactly this shape.
fn try_parse_compare(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // LOAD <formal> <T>
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (param, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let operand_type = if signed { INT_TYPE } else { UINT_TYPE };

    // LITERAL k, of the SAME type as the loaded operand.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &operand_type) {
        return None;
    }
    let k = read_varint(seg, &mut p)?;

    // The relational opcode.
    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // `2C <R> 00` — convert the bool result to the return type.
    if !eat_byte(seg, &mut p, 0x2C) {
        return None;
    }
    let ret_is_int = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    if !eat_byte(seg, &mut p, 0x00) {
        return None;
    }

    // Result type + the shared return plumbing, which must reach the segment end.
    let ret_type = if ret_is_int { INT_TYPE } else { UINT_TYPE };
    if !eat_byte(seg, &mut p, 0x41) || !eat(seg, &mut p, &ret_type) {
        return None;
    }
    // Result type already consumed above, so `has_result_type` is false here.
    eat_return_plumbing(seg, &mut p, false).ok()?;

    // The compared value must be the function's FIRST formal: the spine reads it
    // from r3, and nothing here models a register move.
    let params = parse_formals(seg, lo).ok()?;
    if params.first() != Some(&param) || params.len() != 1 {
        return None;
    }

    // Gates moved here from `compare_leaf_text`, so the census counts only what the
    // emitter can emit. A literal outside the signed 16-bit immediate needs
    // `lis`+`ori` and the extra temp slot that consumes; and `==`/`!=` form `a - k`
    // as `addi r11,a,-k`, so at `k == i16::MIN` the negation itself overflows.
    if i16::try_from(k).is_err() {
        return None;
    }
    if matches!(rel, Rel::Eq | Rel::Ne) && k == i32::from(i16::MIN) {
        return None;
    }
    Some(BodyShape::Compare(CompareLeaf { param, rel, signed, k }))
}

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call, an integer tail call `return g(<arg>)` (passthrough
/// or arg-setup, plus the `g(a)+0` identity fold), or the framed
/// `return g(a) + k` (k ≠ 0). See [`parse_segment`] for the grammar;
/// fail-closed at every step. `lo` locates the formals for the arg-setup.
fn parse_call_shape(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    bound_to: Option<u32>,
) -> Result<BodyShape, Block> {
    // 26 <tok> function/result ref.
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, "call-ref"));
    }
    // The `26 <tok>` symbol push NAMES THE CALLEE. The CALL token that follows
    // carries only a function-*type* id, so this token is the only thing that
    // distinguishes one callee from another; it is resolved through the `.gl`
    // symbol index (see `gl_symbol_index`).
    let (callee_tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "call-ref-tok"))?;
    *p += w;
    // The CALL token: `BD <TYPE ret> <flags> <varint fn-type-id>`. Nothing in it
    // is fixed but the `BD` — it is 8 to 13 bytes and self-delimiting field by
    // field, so it is decoded rather than matched.
    //
    // This replaces a hardcoded 6-byte "callee anchor" `00 80 01 10 00 00`,
    // which was never an anchor: it is `flags = 0` followed by the varint
    // `0x1001`, and `0x1001` is merely the first function type a single-function
    // fixture TU happens to create. True of every MVP fixture and of almost
    // nothing else — which is precisely what the `call-anchor-*` census buckets
    // were measuring.
    if !eat_byte(seg, p, 0xBD) {
        // `26 <sym>` followed by an INTRINSIC CALL rather than a `BD`. This is the
        // other half of the `0x40` production's footprint and it was the whole of
        // the `call-token-0x33` census bucket (7.4 % of blocked functions): a
        // member call whose `this` is an adjusted base pointer opens
        // `26 <method> 33 86 41 74 <2113> 40 …`, and an intrinsic result stored to
        // a symbol opens `26 <dest> 33 86 41 74 <id> 40 …`. Reported with the
        // selector so the two footprints can be summed; still `Err`, so the gate
        // is unchanged.
        if let Some(id) = intrinsic_selector(seg, *p) {
            return Err(Block {
                ctx: "call-intrinsic",
                byte: Some(0x40),
                off: *p,
                aux: id as u32,
            });
        }
        return Err(blk(seg, *p, "call-token"));
    }
    let (_, _, _, ret_w) = read_type(seg, *p).ok_or(blk(seg, *p, "call-ret-type"))?;
    *p += ret_w;
    // Calling convention: 0x00 = cdecl/stdcall, 0x04 = fastcall, 0x40 = varargs.
    // Only cdecl is in class — the others need argument-passing the port does
    // not implement, and accepting them would mis-emit rather than refuse.
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "call-conv")),
    }
    // The function-type id. NOT the callee: three different callees sharing one
    // signature produce byte-identical CALL tokens. The callee is bound from the
    // `26 <tok>` symbol push instead, so this field is decoded only to find the
    // token's end, then discarded.
    read_varint(seg, p).ok_or(blk(seg, *p, "call-fn-type-id"))?;

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type). `g();g();` and `g();return a+1;` fail here — a
    // second `26` call or a `B9` statement stands where the return plumbing must.
    if eat(seg, p, &[0x4C, 0x4B]) {
        eat_return_plumbing(seg, p, false)?;
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }

    // INT call. The argument region is a **repetition**, not a single argument:
    //
    //     args := ( expr `55` <TYPE> )*  `4C`
    //
    // Each argument is a modeled sub-expression — a passthrough `B9 a INT`
    // (→ `[Load]`) or an arg-setup like `a + 1` (→ `[Load, Lit, Add]`) — followed
    // by `55 <TYPE>` carrying the *formal's* declared type, and the whole list is
    // terminated by `4C`. Arguments appear in **reverse source order**, rightmost
    // first (anchored on `parse_formals`, which reverses the `2D` stream so
    // `params[0]` is its last token; `fixtures/cpp/il_call_args2.cpp` holds the
    // `g2(a,b)` / `g2(b,a)` pair that separates the two readings).
    //
    // This used to accept exactly one argument, so every real call site blocked at
    // the second `B9` — the largest single census bucket.
    let mut args: Vec<Vec<IlOp>> = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            break;
        }
        let ops = parse_expr(seg, p, 0x55)?;
        if !eat_byte(seg, p, 0x55) || !eat_int_like(seg, p) {
            // an argument whose terminator or formal type we do not model
            return Err(blk(seg, *p, "call-end"));
        }
        args.push(ops);
        if args.len() > 8 {
            // Past the eighth the arguments are stack-homed, which needs a frame.
            return Err(Block { ctx: "call-args-overflow", byte: None, off: *p, aux: 0 });
        }
    }
    if args.is_empty() {
        // A zero-argument int call (`return g();`). The value-consuming shapes
        // below all assume an argument region, so refuse rather than guess.
        return Err(Block { ctx: "call-args-none", byte: None, off: *p, aux: 0 });
    }
    // A call whose result is bound to a local that is then returned immediately —
    // `int z = g(a); return z;` — is byte-identical to `return g(a);`. c2
    // register-allocates the local and coalesces the copy, so both are a bare
    // `b <callee>`; captured on the one-, two- and three-argument forms.
    //
    // This is the `expr-call-in-expr` census bucket, and after the gate migration it
    // is the largest single blocker at 12.3% of blocked functions. It needs no new
    // codegen at all — only the IL model — so it routes to the existing tail-call
    // productions rather than growing a shape of its own.
    //
    // The local never becomes a memory object here, which is why this does not
    // reopen the store question `il_stmt_static.cpp` closed: the value is returned,
    // never written anywhere, and the shape below admits nothing between the store
    // and the return.
    if let Some(dst) = bound_to {
        //  32 <TYPE> 4B          store the call result into `dst`, discard the value
        //  [4F 01 <line>]*       a line change between the two statements
        //  B9 <dst> <TYPE> 41    load it straight back and return it
        if !eat_byte(seg, p, 0x32) || !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-store"));
        }
        if !eat_byte(seg, p, 0x4B) {
            return Err(blk(seg, *p, "call-bound-stmt-end"));
        }
        eat_opt_stmt_marker(seg, p);
        if !eat_byte(seg, p, 0xB9) {
            return Err(blk(seg, *p, "call-bound-reload"));
        }
        let (back, w) =
            read_token_var(seg, *p).ok_or(blk(seg, *p, "call-bound-reload-tok"))?;
        *p += w;
        // Anything other than reading back the very token just written is a
        // different program.
        if back != dst {
            return Err(Block { ctx: "call-bound-other-token", byte: None, off: *p, aux: 0 });
        }
        if !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-reload-type"));
        }
        eat_return_plumbing(seg, p, true)?;
        let params = parse_formals(seg, lo)?;
        if args.len() > 1 {
            let mut arg_sources = Vec::with_capacity(args.len());
            for slot in 0..args.len() {
                let ops = &args[args.len() - 1 - slot];
                let tok = match ops.as_slice() {
                    [IlOp::Load(t)] => *t,
                    _ => {
                        return Err(Block {
                            ctx: "call-arg-computed",
                            byte: None,
                            off: *p,
                            aux: 0,
                        })
                    }
                };
                match params.iter().position(|&t| t == tok) {
                    Some(ix) => arg_sources.push(ix),
                    None => {
                        return Err(Block {
                            ctx: "call-arg-nonformal",
                            byte: None,
                            off: *p,
                            aux: 0,
                        })
                    }
                }
            }
            for (i, src) in arg_sources.iter().enumerate() {
                if arg_sources[..i].contains(src) {
                    return Err(Block {
                        ctx: "call-arg-duplicated",
                        byte: None,
                        off: *p,
                        aux: 0,
                    });
                }
            }
            let n = arg_sources.len();
            let mut seen = vec![false; n];
            let mut cycles = 0usize;
            for start in 0..n {
                if seen[start] || arg_sources[start] == start {
                    seen[start] = true;
                    continue;
                }
                let mut at = start;
                while !seen[at] {
                    seen[at] = true;
                    at = arg_sources[at];
                }
                cycles += 1;
            }
            if cycles > 1 {
                return Err(Block { ctx: "call-arg-multicycle", byte: None, off: *p, aux: 0 });
            }
            return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
        }
        let arg_ops = args.pop().expect("exactly one argument");
        if has_repeated_leaf(&arg_ops) {
            return Err(Block { ctx: "call-arg-repeated-leaf", byte: None, off: *p, aux: 0 });
        }
        if !additive_chain_canonical(&arg_ops) {
            return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
        }
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled, and
        // only as a tail call. Every argument must be a bare parameter LOAD — a
        // computed argument would need its own register and interacts with the
        // permutation temp in ways no capture covers yet.
        let params = parse_formals(seg, lo)?;
        let mut arg_sources = Vec::with_capacity(args.len());
        // Stream order is reverse source order, so slot `i` is stream `n-1-i`.
        for slot in 0..args.len() {
            let ops = &args[args.len() - 1 - slot];
            let tok = match ops.as_slice() {
                [IlOp::Load(t)] => *t,
                _ => {
                    return Err(Block {
                        ctx: "call-arg-computed",
                        byte: None,
                        off: *p,
                        aux: 0,
                    })
                }
            };
            match params.iter().position(|&t| t == tok) {
                Some(ix) => arg_sources.push(ix),
                // An argument that is not one of this function's formals (a local,
                // a global, a nested call result).
                None => {
                    return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 })
                }
            }
        }
        // The two permutation shapes codegen cannot lower are rejected HERE rather
        // than there, so the census and the emission gate cannot disagree about
        // what is in class (the same reason the FP contraction and constant gates
        // live in this file). Both are captured in `fixtures/cpp/il_call_multi.cpp`
        // and explained at `c2_core::codegen::permute_args_text`.
        //
        // A value passed twice: c2 emits a dead `mr` through the temp, which no
        // live-value-driven solver produces.
        for (i, s) in arg_sources.iter().enumerate() {
            if arg_sources[..i].contains(s) {
                return Err(Block { ctx: "call-arg-duplicated", byte: None, off: *p, aux: 0 });
            }
        }
        // Two or more disjoint cycles: c2 hoists every save (r11, then r10) and
        // then has several clobber-free orders to choose between, which the one
        // available capture does not pin down.
        {
            let n = arg_sources.len();
            let mut seen = vec![false; n];
            let mut cycles = 0usize;
            for start in 0..n {
                if seen[start] || arg_sources[start] == start {
                    seen[start] = true;
                    continue;
                }
                let mut at = start;
                while !seen[at] {
                    seen[at] = true;
                    at = arg_sources[at];
                }
                cycles += 1;
            }
            if cycles > 1 {
                return Err(Block { ctx: "call-arg-multicycle", byte: None, off: *p, aux: 0 });
            }
        }
        // Only a terminal tail call: a post-op would consume the result and need
        // the framed path, which does not model argument setup at all.
        if seg.get(*p) != Some(&0x41) {
            return Err(Block { ctx: "call-multiarg-postop", byte: None, off: *p, aux: 0 });
        }
        eat_return_plumbing(seg, p, true)?;
        return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
    }
    let arg_ops = args.pop().expect("exactly one argument");
    // The single call argument is an ordinary operand stream, so it is subject to
    // the same rewriter: `g(a + a)` is not `add` + branch.
    if has_repeated_leaf(&arg_ops) {
        return Err(Block { ctx: "call-arg-repeated-leaf", byte: None, off: *p, aux: 0 });
    }
    // And to the same reassociation: `g(b + a)` is not the source order either.
    // `parse_formals` may legitimately fail here — the framed-call class carries no
    // formals — and that must not turn into a rejection, so fall back to an empty
    // list, against which `leaves_ascending` simply has nothing to compare.
    // The ordering gate needs formals to order against, and the framed-call class
    // legitimately has none. It is also vacuous for a single operand — and the
    // framed path accepts only a bare passthrough `[Load]`, which cannot be out of
    // order — so skip it there rather than weakening the gate itself.
    let n_loads = arg_ops.iter().filter(|o| matches!(o, IlOp::Load(_))).count();
    if n_loads > 1 {
        let formals = parse_formals(seg, lo)?;
        if !leaves_ascending(&arg_ops, &formals) {
            return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
        }
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
    }

    // Post-op region. EITHER the return plumbing begins directly at its `41`
    // result-type marker (no post-op → an integer tail call `return g(<arg>)`),
    // OR exactly one literal `33 <int> k` + ADD (`return g(a) + k`, framed).
    if seg.get(*p) == Some(&0x41) {
        // No post-op → integer tail call: compute the argument into r3, then
        // `b <callee>` (5-section leaf). The int analog of the void tail call;
        // `g(a)` is a bare `b g`, `g(a+1)` prepends `addi r3,r3,1`.
        eat_return_plumbing(seg, p, true)?;
        let params = parse_formals(seg, lo)?;
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // Post-op `+ k`: EXACTLY one literal `33 <int> k` immediately followed by
    // ADD. A second call (`g(a)+g(1)` → `26 …`), a second literal (`g(a)+1+2` →
    // a second `33 …`), or SUB/MUL (`03`/`04`) all fail one of these `eat`s.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return Err(blk(seg, *p, "call-postop"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "call-postop-varint"))?;
    if !eat_byte(seg, p, 0x02) {
        // non-ADD post-op → non-commutative / strength-reduced
        return Err(blk(seg, *p, "call-postop-op"));
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(Block { ctx: "call-postop-wide", byte: None, off: *p, aux: 0 });
    }
    eat_return_plumbing(seg, p, true)?;

    // W4b2-vi identity fold: a net post-op of 0 is NOT a framed call. `g(a)+0`
    // == `g(a)`, and the optimizer folds it to the bare `b g` (verified: the
    // `g(a)+0` obj is byte-identical to `g(a)`'s). Route it to the integer
    // tail-call production so it takes the 5-section leaf path — never the
    // 6-section framed obj (which would mis-emit a frame the reference elides).
    if k == 0 {
        let params = parse_formals(seg, lo)?;
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // A genuine `+ k` (k ≠ 0) is a framed non-leaf call — but the 6-section
    // framed path models only a **bare passthrough argument** (`g(a) + k`), not
    // arg-setup. `g(a+1) + 1` (a computed argument AND a framed post-op) is out
    // of class → reject (fail closed), never a mis-emitted framed obj.
    if matches!(arg_ops.as_slice(), [IlOp::Load(_)]) {
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok });
    }
    Err(Block { ctx: "framed-computed-arg", byte: None, off: *p, aux: 0 })
}

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
const FN_START: [u8; 2] = [0x4F, 0x1F];

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
/// One function's census verdict (P2b). Either the modeled shape it parsed as,
/// or the first feature that blocked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FnVerdict {
    /// Parsed as a modeled shape. The string is a stable shape label
    /// (`straight-line`, `void-tail-call`, `int-tail-call`, `framed-call`).
    InClass(&'static str),
    /// Blocked at the first unmodeled feature.
    Blocked(Block),
}

impl FnVerdict {
    /// The census bucket key: the shape label when in class, else the blocking
    /// feature (see [`Block::feature`]).
    pub fn key(&self) -> String {
        match self {
            FnVerdict::InClass(s) => (*s).to_string(),
            FnVerdict::Blocked(b) => b.feature(),
        }
    }
    pub fn in_class(&self) -> bool {
        matches!(self, FnVerdict::InClass(_))
    }
}

/// One census row: a function segment and how it classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnCensus {
    /// Index of the function within the TU (`.ex` segment order).
    pub index: usize,
    /// Mangled name, when `.gl` has one at this position.
    pub name: Option<String>,
    /// Segment length in bytes (a rough proxy for function size).
    pub seg_len: usize,
    pub verdict: FnVerdict,
    /// Raw bytes around the blocking site, for grammar work: the segment window
    /// `[off - CENSUS_HEX_BACK, off + CENSUS_HEX_FWD)` clamped to the segment,
    /// plus the index of the blocking byte within that window. Empty when the
    /// function is in class.
    pub hex: Vec<u8>,
    /// Index of the blocking byte inside [`FnCensus::hex`].
    pub hex_mark: usize,
}

/// Bytes of context kept before / after a blocking site.
pub const CENSUS_HEX_BACK: usize = 16;
pub const CENSUS_HEX_FWD: usize = 24;

/// The `.ex` body marker `4C 4F 11` (`LO`) that opens every function body.
const LO_MARKER: [u8; 3] = [0x4C, 0x4F, 0x11];

/// Split `.ex` into one segment per **function body**, anchored on the `LO`
/// marker rather than the `4F 1F` function-start marker (P2b).
///
/// `4F 1F` is only two bytes and also occurs inside token and varint payloads,
/// so a raw marker scan over a real translation unit over-counts: measured on
/// `system/world/Dir.cpp` (1.5 MB `.ex`), 5340 `4F 1F` against 5239 `LO` body
/// markers and 5243 function tails (`4F 12 47 54 01 54 00`) — the latter two
/// agree to 0.08%, the first is ~2% high. Anchoring on `LO` keeps the count
/// honest without inventing a denominator.
///
/// Each segment starts at the `4F 1F` immediately preceding its `LO` (so the
/// formals region stays inside the segment, where [`parse_formals`] looks for
/// it) and runs to the next segment's start. Two bodies sharing one preceding
/// `4F 1F` would collide; the later one then starts at its own `LO`, which
/// simply blocks it at `formals-marker` — an honest miss, never a merge that
/// would silently drop a function from the denominator.
fn split_function_bodies(ex: &[u8]) -> Vec<&[u8]> {
    // Body markers, in file order. Same walk as the old byte loop (a match
    // consumes 3 bytes, a miss 1); candidates are found word-at-a-time.
    let mut los: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 3 <= ex.len() {
        let Some(k) = memchr_byte(LO_MARKER[0], &ex[i..ex.len() - 2]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == LO_MARKER[1] && ex[j + 2] == LO_MARKER[2] {
            los.push(j);
            i = j + 3;
        } else {
            i = j + 1;
        }
    }
    if los.is_empty() {
        return Vec::new();
    }
    // Function-start markers, in file order, for the "nearest preceding" lookup.
    let mut starts: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 2 <= ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }

    let mut segs_start: Vec<usize> = Vec::with_capacity(los.len());
    for &lo in &los {
        // Greatest `4F 1F` offset strictly below this body marker.
        let cand = match starts.partition_point(|&s| s < lo) {
            0 => lo,
            k => starts[k - 1],
        };
        // Never reuse a start (would merge two bodies into one segment).
        let cand = if segs_start.last() == Some(&cand) { lo } else { cand };
        segs_start.push(cand);
    }
    (0..segs_start.len())
        .map(|k| {
            let start = segs_start[k];
            let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
            &ex[start..end.max(start)]
        })
        .collect()
}

/// True iff `.ex` positively declares a module with **no function bodies**
/// (R1): it carries neither a body marker (`4C 4F 11`) nor a function-start
/// marker (`4F 1F`).
///
/// Both signals are required. `4F 1F` alone is two bytes and collides inside
/// payloads (so its *absence* is meaningful but its presence is not), while
/// `LO` is the marker every real body opens with — on a 1.5 MB real `.ex` the
/// `LO` count tracked the function-tail count to 0.08%. A capture with zero of
/// each has nothing that could be a function.
///
/// Verified against the live toolchain: a TU containing only a typedef captures
/// a 2691-byte `.ex` that is entirely zero-fill apart from a 4-byte head and a
/// 46-byte module-metadata tail, with 0 `LO` and 0 `4F 1F`, and c2 emits a
/// 720-byte four-section obj for it.
pub fn is_empty_module(ex: &[u8]) -> bool {
    let has_lo = find_subslice(ex, &LO_MARKER).is_some();
    let has_fn_start = find_subslice(ex, &FN_START).is_some();
    !has_lo && !has_fn_start
}

fn split_functions(ex: &[u8]) -> Vec<&[u8]> {
    split_functions_at(ex).1
}

/// [`split_functions`], keeping the `4F 1F` offsets alongside the segments. The
/// offsets are what `.gl`'s framed body-start fields are matched against, so the
/// name binding is per record rather than per position (see
/// [`gl_defined_names`]).
fn split_functions_at(ex: &[u8]) -> (Vec<usize>, Vec<&[u8]>) {
    let mut starts = Vec::new();
    let mut i = 0;
    // Same walk as the old byte loop (a match consumes 2 bytes, a miss 1);
    // candidates are found word-at-a-time.
    while i + 1 < ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }
    let mut segs = Vec::with_capacity(starts.len());
    for k in 0..starts.len() {
        let end = if k + 1 < starts.len() { starts[k + 1] } else { ex.len() };
        segs.push(&ex[starts[k]..end]);
    }
    (starts, segs)
}

impl IlBundle {
    /// **Function-level census (P2b).** Classify *every* function in the bundle
    /// independently, so a TU whose 700th function uses an unmodeled opcode
    /// still reports the other 699 as in-class.
    ///
    /// This is the measurement [`IlBundle::functions`] cannot give: that method
    /// is all-or-nothing per TU (correctly — the port must emit a whole obj or
    /// nothing), so over a real workload it reports one `vocab-gap` per TU and
    /// cannot rank the missing classes. The census runs the *same*
    /// [`parse_segment_detail`] per segment and keeps the first blocking
    /// feature, so the histogram of [`FnVerdict::key`] over a corpus is the
    /// widening order (docs/ROADMAP.md §G5).
    ///
    /// Diagnostic only — never a gate, and never consulted by the emitter.
    /// Returns `None` only when the bundle lacks the required files.
    pub fn function_census(&self) -> Option<Vec<FnCensus>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let names = mangled_names(gl);
        let segs = split_function_bodies(ex);
        // Names are paired positionally, which is only meaningful when `.gl`
        // yields exactly one name per body. On a real TU `mangled_names` finds
        // far fewer (it accepts only `?…@@…` forms, and `.gl` also lists
        // externals), so pairing there would attach wrong names to functions —
        // report none rather than a plausible-looking lie.
        let paired = names.len() == segs.len();
        // The symbol index is threaded into the parse but is no longer consulted on
        // this path: the assignment class used to decide "is this destination a
        // global?" by asking whether `.gl` named it, and that was wrong (a file-scope
        // `static` is `$sv`, which the index does not accept as an identifier), so
        // the destination is now established positively from the formals list.
        //
        // It stays threaded because modelling locals will need a symbol view again
        // and the plumbing is the awkward part — but do not restore the absence test.
        // `GlIndex` builds the map lazily, so a TU with no call shape never pays for
        // it at all; the contents are identical when it is built, so laziness cannot
        // change acceptance.
        let globals = GlIndex::new(gl);
        Some(
            segs.iter()
                .enumerate()
                .map(|(i, seg)| {
                    let verdict = match parse_segment_detail(seg, &globals) {
                        Ok(BodyShape::StraightLine { .. }) => FnVerdict::InClass("straight-line"),
                        Ok(BodyShape::VoidTailCall { .. }) => FnVerdict::InClass("void-tail-call"),
                        Ok(BodyShape::IntTailCall { .. }) => FnVerdict::InClass("int-tail-call"),
                        Ok(BodyShape::MultiArgTailCall { .. }) => {
                            FnVerdict::InClass("multiarg-tail-call")
                        }
                        Ok(BodyShape::FramedCall { .. }) => FnVerdict::InClass("framed-call"),
                        Ok(BodyShape::Compare(_)) => FnVerdict::InClass("compare-leaf"),
                        Ok(BodyShape::EmptyBody) => FnVerdict::InClass("empty-body"),
                        Ok(BodyShape::IndirectLoad { .. }) => {
                            FnVerdict::InClass("indirect-load-leaf")
                        }
                        Ok(BodyShape::FloatLeaf { double, .. }) => {
                            FnVerdict::InClass(if double { "double-leaf" } else { "float-leaf" })
                        }
                        Err(b) => FnVerdict::Blocked(b),
                    };
                    // Keep the raw bytes around the blocking site: decoding a new
                    // grammar production always starts by staring at exactly this
                    // window, and having it in the census means that work is a
                    // report away instead of a one-off script.
                    let (hex, hex_mark) = match &verdict {
                        FnVerdict::InClass(_) => (Vec::new(), 0),
                        FnVerdict::Blocked(b) => {
                            let start = b.off.saturating_sub(CENSUS_HEX_BACK);
                            let end = (b.off + CENSUS_HEX_FWD).min(seg.len());
                            let start = start.min(end);
                            (seg[start..end].to_vec(), b.off - start)
                        }
                    };
                    FnCensus {
                        index: i,
                        name: if paired { names.get(i).cloned() } else { None },
                        seg_len: seg.len(),
                        verdict,
                        hex,
                        hex_mark,
                    }
                })
                .collect(),
        )
    }

    /// Parse this bundle as a sequence of straight-line add-chain functions
    /// (the MVP class, generalized to a multi-function TU). Returns `None` if
    /// the required files are absent, or if the `.gl` name count does not match
    /// the `.ex` function count, or if ANY function body is outside the class
    /// (the caller — `PortC2` — then reports `NotImplemented` for the whole TU).
    ///
    /// Bodies come from `.ex` split at each `4F 1F`; each body's name comes from
    /// the `.gl` record whose framed body-start offset **is** that split point
    /// ([`gl_defined_names`]) — a per-record binding, not a positional one. Any
    /// `.gl` symbol no record claimed must be a resolved callee, or the TU is
    /// refused: an unclaimed symbol is one the real obj defines and the port does
    /// not model.
    pub fn functions(&self) -> Option<Vec<IlFunction>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let src = source_path(gl);

        // R1: a TU that defines no functions is in class, and its obj is the
        // fixed four-section shell with no `.text`. Recognized **positively**
        // (no body markers AND no function-start markers), never as "the split
        // returned nothing" — the latter would also fire on a bundle we merely
        // failed to split, and emitting an empty obj for a TU that really has
        // code is precisely the mis-emit the fail-closed rule forbids.
        //
        // Evaluated in one pass over `.ex` instead of calling
        // [`is_empty_module`] up front: the split already proves whether any
        // `4F 1F` exists, so only the no-start case still needs the body-marker
        // probe. The predicate is unchanged — all four (LO?, 4F1F?) cases land
        // exactly where they did:
        //   neither        → empty module (was: is_empty_module → Some([]))
        //   LO only        → None         (was: not empty; split empty → None)
        //   4F 1F, any LO  → parse        (was: not empty; split non-empty)
        let (starts, segs) = split_functions_at(ex);
        if segs.is_empty() {
            return if find_subslice(ex, &LO_MARKER).is_none() {
                Some(Vec::new())
            } else {
                None
            };
        }
        // Per-record name binding, gated fail-closed: the `.gl` records' framed
        // body-start offsets must be exactly the `.ex` split points, in order and
        // 1:1. A disagreement means either `.gl` has a record shape we cannot
        // frame or the splitter miscounted bodies, and in both cases every name
        // after the divergence would be wrong — so bind none of them.
        //
        // A *defined* function's own name comes from here. Callee names do NOT:
        // they are resolved by token through the `.gl` symbol index, because the
        // CALL token carries only a function-type id and cannot distinguish two
        // callees with the same signature.
        let (bound, unclaimed) = gl_defined_names(gl);
        if bound.len() != segs.len()
            || bound
                .iter()
                .zip(&starts)
                .any(|(&(off, _), &s)| off as usize != s)
        {
            return None;
        }
        let names: Vec<String> = bound.into_iter().map(|(_, n)| n).collect();
        let n_defined = segs.len();
        // Lazily built: only the call productions resolve through it, so a TU
        // of straight-line leaves never constructs the index at all.
        let symbols = GlIndex::new(gl);
        let resolve = |tok: u32| -> Option<String> { symbols.map().get(&tok).cloned() };

        let mut funcs = Vec::with_capacity(n_defined);
        for (name, seg) in names.iter().take(n_defined).zip(segs) {
            match parse_segment(seg, &symbols)? {
                // An indirect-load leaf reaches the ordinary integer selector,
                // which pattern-matches its exact two-op stream; `params` carries
                // a member function's `this` at index 0 so the base register comes
                // out right.
                BodyShape::IndirectLoad { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::StraightLine { params, ops } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // Tail calls: the callee is resolved BY TOKEN through the `.gl`
                // symbol index. An unresolvable token rejects the whole TU
                // rather than falling back to a positional guess — a wrong
                // callee name is a relocation against the wrong symbol, which is
                // a mis-emit, not a gap.
                BodyShape::VoidTailCall { callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::IntTailCall { params, arg_ops, callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops: arg_ops,
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // A multi-argument tail call is still a tail call — same resolved
                // callee, same `b <callee>` — but its argument setup is a register
                // permutation rather than an operand stream, so `ops` stays empty
                // and `arg_sources` carries the mapping.
                BodyShape::MultiArgTailCall { params, arg_sources, callee_tok } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops: Vec::new(),
                        tail_call: Some(resolve(callee_tok)?),
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: Some(arg_sources),
                    });
                }
                // The framed non-leaf path stays SINGLE-FUNCTION. Its obj carries
                // `.pdata` with compiler label symbols ($M2545/$M2546/$T2547)
                // whose counters are a fixed toolchain seed for the first
                // function and shift once preceding functions consume slots
                // (W-UNW-1, docs/CODEGEN_PPC_MVP.md), so a multi-function TU
                // containing one would be mis-numbered.
                BodyShape::FramedCall { add_k, callee_tok } => {
                    if n_defined != 1 {
                        return None;
                    }
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: Some(FramedCall {
                            callee: resolve(callee_tok)?,
                            add_k,
                        }),
                        compare: None,
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                // W6: a comparison leaf carries no op stream — codegen emits its
                // spine from the decoded relation instead.
                BodyShape::EmptyBody => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: Vec::new(),
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: true,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
                BodyShape::FloatLeaf { params, ops, double } => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params,
                        ops,
                        tail_call: None,
                        framed_call: None,
                        compare: None,
                        empty_body: false,
                        float_leaf: Some(double),
                        arg_sources: None,
                    });
                }
                BodyShape::Compare(cmp) => {
                    funcs.push(IlFunction {
                        mangled_name: name.clone(),
                        source_path: src.clone(),
                        params: vec![cmp.param],
                        ops: Vec::new(),
                        tail_call: None,
                        framed_call: None,
                        compare: Some(cmp),
                        empty_body: false,
                        float_leaf: None,
                        arg_sources: None,
                    });
                }
            }
        }
        // Account for every `.gl` symbol no record claimed. The port emits
        // exactly the `n_defined` bodies plus an external symbol per resolved
        // callee, so an unclaimed name is a symbol the real obj has and this obj
        // would not — and for a *data* definition it is a whole extra section.
        // `int gv; int f(int a){return a+1;}` mismatched at file offset 2, the
        // section count, for exactly this reason: `?gv@@3HA` was invisible to the
        // emitter. A defined static member (`?sm@S@@2HA`) did the same.
        //
        // Extern data cannot be told from defined data by mangling — `extern int
        // g;` and `int g;` both appear as `?g@@3HA` — so this refuses both. That
        // costs nothing today: reading a global is already out of class, so an
        // extern that is never referenced is one c2 would not have listed.
        let mut accounted: Vec<&str> = names.iter().map(String::as_str).collect();
        for f in &funcs {
            if let Some(c) = &f.tail_call {
                accounted.push(c);
            }
            if let Some(fc) = &f.framed_call {
                accounted.push(&fc.callee);
            }
        }
        if unclaimed.iter().any(|n| !accounted.contains(&n.as_str())) {
            return None;
        }
        // A callee that is also DEFINED here is out of class: c2 may inline it,
        // and the port cannot. `int f(int); int use(int a){return f(a);}
        // int f(int a){return a+1;}` gets a `.text` of *two* copies of
        // `addi r3,r3,1 ; blr` and **no relocations** — c2 cloned `f` into `use`
        // rather than branching to it. The port emitted `b ?f` against an
        // undefined external and mismatched at file offset 8.
        //
        // Refused wholesale rather than by callee size, because what makes c2
        // inline (and what it does to the symbol table and `.pdata` when it does)
        // is uncharacterized. Calls to true externals are unaffected — those are
        // the tail calls the class was built on.
        if funcs.iter().any(|f| {
            let callee = f
                .tail_call
                .as_deref()
                .or(f.framed_call.as_ref().map(|c| c.callee.as_str()));
            callee.is_some_and(|c| names.iter().any(|n| n == c))
        }) {
            return None;
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
    /// These pinned segments are synthetic and contain no global stores, so an
    /// empty symbol index is the honest input: nothing here is a global.
    fn no_globals() -> GlIndex<'static> {
        GlIndex::new(&[])
    }

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
            parse_segment(seg, &no_globals()),
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
            parse_segment(konst, &no_globals()),
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
            parse_segment(kw, &no_globals()),
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
            parse_segment(seg, &no_globals()),
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
        assert_eq!(
            parse_segment(MVP_CALL, &no_globals()),
            Some(BodyShape::VoidTailCall { callee_tok: 0xE309 })
        );
    }

    #[test]
    fn parse_segment_accepts_framed_call() {
        // `int f(int a){ return g(a) + 1; }` (mvp_framed): int call, single
        // passthrough arg, `55` call-end, exactly one `+1` post-op.
        assert_eq!(
            parse_segment(MVP_FRAMED, &no_globals()),
            Some(BodyShape::FramedCall { add_k: 1, callee_tok: 0xE409 })
        );
    }

    #[test]
    fn parse_segment_accepts_int_tail_call_family() {
        // The three int tail-call shapes (formals `46 2d e509` = param a → r3):
        //   passthrough `g(a)` and identity-fold `g(a)+0` → arg `[Load a]`;
        //   arg-setup `g(a+1)` → arg `[Load a, Lit 1, Add]`. All are
        //   `IntTailCall` (a net-identity post-op is a tail call, not framed).
        assert_eq!(
            parse_segment(INT_TAILRET, &no_globals()),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "passthrough g(a)"
        );
        assert_eq!(
            parse_segment(INT_PLUS0, &no_globals()),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509)],
                callee_tok: 0xE409,
            }),
            "identity-fold g(a)+0 routes to a tail call, not FramedCall{{add_k:0}}"
        );
        assert_eq!(
            parse_segment(INT_ARGTAIL, &no_globals()),
            Some(BodyShape::IntTailCall {
                params: vec![0xE509],
                arg_ops: vec![IlOp::Load(0xE509), IlOp::Lit(1), IlOp::Add],
                callee_tok: 0xE409,
            }),
            "arg-setup g(a+1)"
        );
    }

    #[test]
    fn parse_segment_routes_framed_nonzero_but_folds_zero_k() {
        // Routing contrast at the post-op: a NON-zero `+k` over a bare
        // passthrough arg is FramedCall (6-section frame); a ZERO `+k` folds to
        // an IntTailCall (5-section leaf). Same shape but for the immediate.
        assert_eq!(
            parse_segment(MVP_FRAMED, &no_globals()),
            Some(BodyShape::FramedCall { add_k: 1, callee_tok: 0xE409 }),
            "g(a)+1 is framed"
        );
        assert!(
            matches!(parse_segment(INT_PLUS0, &no_globals()), Some(BodyShape::IntTailCall { .. })),
            "g(a)+0 must NOT be FramedCall{{add_k:0}}"
        );
    }

    #[test]
    fn parse_segment_rejects_all_out_of_class_call_shapes() {
        // The W4b2-i/-v out-of-class probes — each a real captured segment the
        // positive parse must reject at the parser level (→ None →
        // NotImplemented), never mis-emit. Named by their `.cpp`. (The bare
        // arg-setup tail calls `g(a)`/`g(a)+0`/`g(a+1)` are now ACCEPTED —
        // see `parse_segment_accepts_int_tail_call_family`.)
        let cases: &[(&str, &[u8])] = &[
            ("g(a) - 1 (submod)", GA_SUBMOD),
            ("g(a) * 5 (mulmod)", GA_MULMOD),
            ("g(a) + 70000 (widemod)", GA_WIDEMOD),
            ("g(); g(); (two_calls)", TWO_CALLS),
            ("g(); return a+1; (call_then_stmt)", CALL_THEN_STMT),
            ("g(a + 1) + 1 (argframed_plusk)", ARGFRAMED_PLUSK),
            ("g(a) + g(a + 1) (two_framed_calls)", TWO_FRAMED_CALLS),
            ("g(a) + 1 + 2 (plus1plus2)", PLUS1PLUS2),
        ];
        for (label, seg) in cases {
            assert_eq!(parse_segment(seg, &no_globals()), None, "must reject: {label}");
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
        assert_eq!(parse_segment(cmp, &no_globals()), None);
    }

    // ---- variable-width tokens ----------------------------------------------

    #[test]
    fn token_is_two_bytes_when_the_continuation_bit_is_clear() {
        // Every fixture token is of this form (`e3 09`, `e5 09`, …) — bit 7 of
        // the second byte clear.
        assert_eq!(read_token_var(&[0xE3, 0x09, 0xFF], 0), Some((0xE309, 2)));
        assert_eq!(read_token_var(&[0x00, 0x7F], 0), Some((0x007F, 2)));
    }

    #[test]
    fn token_is_four_bytes_when_the_continuation_bit_is_set() {
        // Real-TU form, e.g. the module marker payload `a4 96 03 00`.
        assert_eq!(
            read_token_var(&[0xA4, 0x96, 0x03, 0x00], 0),
            Some((0xA496_0300, 4))
        );
        // Truncated 4-byte token → None, never a short read.
        assert_eq!(read_token_var(&[0xA4, 0x96, 0x03], 0), None);
    }

    #[test]
    fn token_identities_cannot_collide_across_widths() {
        // The parser compares token identities for equality (token → parameter
        // register), so a 2-byte and a 4-byte token must never produce the same
        // value. A 4-byte token's byte 1 has bit 7 set, which lands in bits
        // 23..16 and forces the value >= 0x10000; 2-byte values are < 0x10000.
        for b0 in [0x00u8, 0x7F, 0x80, 0xFF] {
            for b1 in [0x00u8, 0x7F, 0x80, 0xFF] {
                let (v, w) = read_token_var(&[b0, b1, 0x00, 0x00], 0).unwrap();
                if w == 2 {
                    assert!(v < 0x10000, "2-byte token {v:#X} must stay narrow");
                } else {
                    assert!(v >= 0x10000, "4-byte token {v:#X} must not alias a narrow one");
                }
            }
        }
    }

    #[test]
    fn a_four_byte_token_parses_as_one_operand_not_two() {
        // The misalignment this fixes: reading a 4-byte token as 2 bytes leaves
        // the parse standing on the token's own tail, which then looks like an
        // unknown opcode. Build a straight-line body whose single LOAD carries a
        // wide token and check it decodes as exactly one Load of that token.
        let seg: &[u8] = &[
            0x46, 0x2D, 0xA4, 0x96, 0x03, 0x00, // formals: one wide token
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xA4, 0x96, 0x03, 0x00, 0x86, 0x41, 0x74, // LOAD <wide> int
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
        ];
        assert_eq!(
            parse_segment(seg, &no_globals()),
            Some(BodyShape::StraightLine {
                params: vec![0xA496_0300],
                ops: vec![IlOp::Load(0xA496_0300)],
            })
        );
    }

    #[test]
    fn varint_short_form_is_signed() {
        // `-5` is `fb`, not a rejected byte. An earlier revision accepted only
        // 00..7F, which was fail-closed but silently blocked every negative
        // literal in the corpus.
        let cases: &[(&[u8], i32, usize)] = &[
            (&[0x00], 0, 1),
            (&[0x05], 5, 1),
            (&[0x7F], 127, 1),
            (&[0xFB], -5, 1),
            (&[0xFF], -1, 1),
            (&[0x81], -127, 1),
            // Escape form: `80` + 4-byte LE i32.
            (&[0x80, 0x70, 0x11, 0x01, 0x00], 70000, 5),
            // -128 cannot use the short form (0x80 is the marker), so it is
            // forced to the escape.
            (&[0x80, 0x80, 0xFF, 0xFF, 0xFF], -128, 5),
        ];
        for (bytes, want, width) in cases {
            let mut p = 0usize;
            assert_eq!(read_varint(bytes, &mut p), Some(*want), "{bytes:02X?}");
            assert_eq!(p, *width, "width for {bytes:02X?}");
        }
    }

    // ---- `.gl` name → body binding ------------------------------------------

    /// One `.gl` function record: a name run, then the framing
    /// `80 XX 10 00 00 00 00` that `codec::gl_offset_framed` recognizes, then the
    /// `80 <LE32>` body-start offset.
    fn gl_record(name: &str, body_off: u32) -> Vec<u8> {
        let mut v = vec![0u8];
        v.extend_from_slice(name.as_bytes());
        v.push(0);
        v.extend_from_slice(&[0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80]);
        v.extend_from_slice(&body_off.to_le_bytes());
        v
    }

    #[test]
    fn gl_names_bind_to_their_own_record_not_their_position() {
        // The `il_gl_record_order.cpp` layout: a `??`-prefixed thunk first, then a
        // function, then a data symbol, then an external constructor. Positional
        // pairing over `mangled_names` (which cannot see either `??` name) would
        // pair `?w_add` with the thunk's body and the *variable* with the second.
        let mut gl = Vec::new();
        gl.extend_from_slice(&gl_record("??__Egs@@YAXXZ", 2644));
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2753));
        gl.push(0);
        gl.extend_from_slice(b"?gs@@3US@@A");
        gl.push(0);
        gl.push(0);
        gl.extend_from_slice(b"??0S@@QAA@XZ");
        gl.push(0);

        let (bound, unclaimed) = gl_defined_names(&gl);
        assert_eq!(
            bound,
            vec![
                (2644, "??__Egs@@YAXXZ".to_string()),
                (2753, "?w_add@@YAHH@Z".to_string()),
            ],
            "each name must come from the record carrying its own body offset"
        );
        // The data symbol and the external are unclaimed; the caller must account
        // for each as a resolved callee or refuse the TU.
        assert_eq!(
            unclaimed,
            vec!["?gs@@3US@@A".to_string(), "??0S@@QAA@XZ".to_string()]
        );
        // And the narrow scan is exactly what missed the two `??` names.
        assert_eq!(
            mangled_names(&gl),
            vec!["?w_add@@YAHH@Z".to_string(), "?gs@@3US@@A".to_string()],
            "regression guard: mangled_names drops ?? names, so it cannot bind bodies"
        );
    }

    #[test]
    fn gl_symbol_runs_ignore_non_symbol_strings() {
        // A source path is a NUL-delimited printable run too. `@@` is what keeps
        // it out — without that test the accounting rule would refuse every TU.
        let mut gl = vec![0u8];
        gl.extend_from_slice(b"e:\\lazer_build_gmc1\\x.cpp");
        gl.push(0);
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2644));
        let (bound, unclaimed) = gl_defined_names(&gl);
        assert_eq!(bound, vec![(2644, "?w_add@@YAHH@Z".to_string())]);
        assert!(unclaimed.is_empty(), "got {unclaimed:?}");
    }

    #[test]
    fn gl_framed_offset_without_a_name_binds_nothing() {
        // Fail closed on a record shape we cannot name: binding nothing makes
        // `functions()` refuse, rather than emitting a nameless function or
        // borrowing the name of a following record.
        let mut gl = vec![0x80, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x80];
        gl.extend_from_slice(&2644u32.to_le_bytes());
        gl.extend_from_slice(&gl_record("?w_add@@YAHH@Z", 2753));
        assert_eq!(gl_defined_names(&gl), (Vec::new(), Vec::new()));
    }

    // ---- `.gl` symbol index -------------------------------------------------

    #[test]
    fn gl_symbol_index_binds_tokens_to_names() {
        // A `.gl` record is `<kind> <token> 00 <name> 00 <TYPE> …`. Transcribed
        // from a controlled fixture with three externals declared a, b, c —
        // tokens are assigned in DECLARATION order (0x09E3, 0x09E4, 0x09E5),
        // which is what makes a positional pairing with call order wrong.
        let mut gl = Vec::new();
        for (tok, name) in [
            ([0xE3u8, 0x09], "?a@@YAXXZ"),
            ([0xE4, 0x09], "?b@@YAXXZ"),
            ([0xE5, 0x09], "?c@@YAXXZ"),
        ] {
            gl.push(0x04); // kind
            gl.extend_from_slice(&tok);
            gl.push(0x00);
            gl.extend_from_slice(name.as_bytes());
            gl.push(0x00);
            gl.extend_from_slice(&[0x82, 0x07, 0x04]); // TYPE
        }
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?a@@YAXXZ"));
        assert_eq!(idx.get(&0xE409).map(String::as_str), Some("?b@@YAXXZ"));
        assert_eq!(idx.get(&0xE509).map(String::as_str), Some("?c@@YAXXZ"));
        // An unknown token must not resolve — the caller rejects rather than
        // guessing, since a wrong callee is a relocation against a wrong symbol.
        assert!(idx.get(&0xFFFF).is_none());
    }

    #[test]
    fn gl_symbol_index_ignores_non_identifier_runs() {
        // Binary payload between NULs must not become a symbol.
        let gl = b"\x00\x01\x02\x03\x00\x04\xE3\x09\x00?ok@@YAXXZ\x00".to_vec();
        let idx = gl_symbol_index(&gl);
        assert_eq!(idx.get(&0xE309).map(String::as_str), Some("?ok@@YAXXZ"));
        assert_eq!(idx.len(), 1, "only the identifier-shaped run is indexed");
    }

    // ---- inline type encoding (LEB128) --------------------------------------

    #[test]
    fn read_type_widths_match_the_captured_boundaries() {
        // Each of these is pinned in a live capture by the fixed one-byte marker
        // that follows it (`41` result-type, `55` arg push, `4C 4B` call end),
        // so a wrong width visibly swallows the next marker.
        let cases: &[(&[u8], u8, u8, u32, usize)] = &[
            (&[0x86, 0x41, 0x74], 0x86, 0x41, 116, 3),        // int
            (&[0x86, 0x42, 0x75], 0x86, 0x42, 117, 3),        // unsigned
            (&[0x82, 0x07, 0x03], 0x82, 0x07, 3, 3),          // void
            (&[0x86, 0x43, 0x83, 0x08], 0x86, 0x43, 1027, 4), // void*
            (&[0x86, 0x43, 0x82, 0x20], 0x86, 0x43, 4098, 4), // int**
            (&[0x88, 0x85, 0x41], 0x88, 0x85, 65, 3),         // double
            (&[0x88, 0x81, 0x13], 0x88, 0x81, 19, 3),         // long long
            (&[0x86, 0x43, 0x9B, 0xB9, 0x02], 0x86, 0x43, 40091, 5), // 5-byte id
        ];
        for (bytes, tag, kind, id, w) in cases {
            assert_eq!(
                read_type(bytes, 0),
                Some((*tag, *kind, *id, *w)),
                "type {bytes:02X?}"
            );
        }
        // A tag without bit 7 set is not a type.
        assert_eq!(read_type(&[0x41, 0x86, 0x41], 0), None);
    }

    #[test]
    fn call_token_is_decoded_not_anchor_matched() {
        // The old model hardcoded `00 80 01 10 00 00` as a fixed "callee anchor".
        // It is really flags=0 + varint(0x1001), so any TU whose callee function
        // type is not the first one created failed to parse. All three of these
        // are real captured CALL tokens with different fn-type ids and return
        // types; all must now decode.
        let cases: &[(&[u8], &str)] = &[
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00], "void, id 0x1001"),
            (&[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x80, 0x10, 0x00, 0x00], "void, id 0x1080"),
            (
                &[0xBD, 0x86, 0x43, 0x83, 0x08, 0x00, 0x80, 0xAE, 0x10, 0x00, 0x00],
                "void* (4-byte type), id 0x10AE",
            ),
        ];
        for (bytes, label) in cases {
            let mut p = 1; // past the BD
            let (_, _, _, w) = read_type(bytes, p).expect(label);
            p += w;
            assert_eq!(bytes.get(p), Some(&0x00), "{label}: cdecl flags byte");
            p += 1;
            assert!(read_varint(bytes, &mut p).is_some(), "{label}: fn-type id");
            assert_eq!(p, bytes.len(), "{label}: token must end exactly here");
        }
    }

    // ---- P2b function-level census ------------------------------------------

    #[test]
    fn census_agrees_with_the_gate_on_every_pinned_segment() {
        // This used to compare `parse_segment` with `parse_segment_detail` and
        // could not fail: the former is literally `.ok()` of the latter, so it
        // asserted a function equals itself. It protected only against someone
        // re-forking the two, which is worth keeping — hence the first assertion —
        // but it never checked the invariant its name claims.
        //
        // The invariant that matters is that **everything the parser accepts, the
        // emitter can emit**. That cannot be tested from this crate (c2-il cannot
        // depend on c2-core), so what is pinned here is the half that can be: the
        // specific shapes whose emission gates used to live in codegen, and which
        // the census therefore counted as in-class while the port refused them.
        // Each must now be refused by the parser. The other half — that no
        // *accepted* shape is refused downstream — is guarded by the fixture
        // differential and `scripts/expr_sweep.sh`.
        let all: &[&[u8]] = &[
            MVP_CALL, MVP_FRAMED, INT_TAILRET, INT_PLUS0, INT_ARGTAIL, GA_SUBMOD, GA_MULMOD,
            GA_WIDEMOD, TWO_CALLS, CALL_THEN_STMT, ARGFRAMED_PLUSK, TWO_FRAMED_CALLS, PLUS1PLUS2,
        ];
        for seg in all {
            assert_eq!(
                parse_segment(seg, &no_globals()).is_some(),
                parse_segment_detail(seg, &no_globals()).is_ok(),
                "the two entry points have been re-forked"
            );
        }

        // Shapes that parse as a well-formed straight-line body but that
        // `select_text` declines. Each is refused in the parser now.
        let params = vec![0x10u32, 0x11];
        let a = IlOp::Load(0x10);
        let b = IlOp::Load(0x11);
        for (ops, why) in [
            (vec![a, IlOp::Lit(3), IlOp::Mul], "multiply by a constant"),
            (vec![b], "bare non-first parameter"),
            (vec![IlOp::Lit(5), a, IlOp::Sub], "const - reg needs subfic"),
            (vec![IlOp::Lit(-70000)], "negative wide constant"),
        ] {
            assert!(
                straight_line_is_out_of_class(&ops, &params),
                "parser must refuse: {why}"
            );
        }
        // ...and the neighbours that really do emit must stay accepted.
        for (ops, why) in [
            (vec![a, b, IlOp::Add], "a + b"),
            (vec![a], "bare first parameter"),
            (vec![IlOp::Lit(70000)], "positive wide constant"),
        ] {
            assert!(
                !straight_line_is_out_of_class(&ops, &params),
                "parser must accept: {why}"
            );
        }
    }

    // ---- indirect-load leaf -------------------------------------------------
    //
    // Every byte below is transcribed from a live capture of
    // `fixtures/cpp/il_expr_deref.cpp` / `il_expr_member.cpp`
    // (`c2rs census <cpp> --keep-il <dir>`), not derived.

    /// `int ld_p(int* p) { return *p; }` — one formal, no offset add.
    const IND_DEREF: &[u8] = &[
        0x46, 0x2D, 0xEE, 0x09, // formals: p
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0xEE, 0x09, 0x86, 0x43, 0xF4, 0x08, // LOAD p (int *)
        0x30, 0x86, 0x41, 0x74, // indirect load -> int
        0x41, 0x86, 0x41, 0x74, // result type int
        0x3A, 0xF0, 0x09, 0x54, 0x02, 0x29, 0xF0, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // fn tail = segment end
    ];

    /// `int ld_m0(S* s) { return s->a; }` — a `27` byte-offset add of 0.
    const IND_MEMBER0: &[u8] = &[
        0x46, 0x2D, 0xFE, 0x09, // formals: s
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0xFE, 0x09, 0x86, 0x43, 0x81, 0x20, // LOAD s (S *)
        0x33, 0x86, 0x41, 0x74, 0x00, // LITERAL int 0 (byte offset)
        0x27, 0x86, 0x43, 0xF4, 0x08, // byte-offset add -> int *
        0x30, 0x86, 0x41, 0x74, // indirect load -> int
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x00, 0x0A, 0x54, 0x02, 0x29, 0x00, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int ld_ixneg(int* p) { return p[-1]; }` — a `28 00 00` subscript add whose
    /// offset is the **signed** short form `FC` = −4, typed `long` not `int`.
    const IND_SUBSCRIPT_NEG: &[u8] = &[
        0x46, 0x2D, 0x10, 0x0A, // formals: p
        0x4C, 0x4F, 0x11, 0x53, //
        0xB9, 0x10, 0x0A, 0x86, 0x43, 0xF4, 0x08, // LOAD p (int *)
        0x33, 0x86, 0x41, 0x12, 0xFC, // LITERAL long -4
        0x28, 0x00, 0x00, // subscript add
        0x30, 0x86, 0x41, 0x74, //
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0x12, 0x0A, 0x54, 0x02, 0x29, 0x12, 0x0A, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int C::get_b() const { return b; }` — the `this` form: the pre-body
    /// region binds `this` with `B9 <tok> <TYPE> 99 <TYPE> 00` and the `2D`
    /// formals list is EMPTY, so `this` must come from that binding or the base
    /// register is wrong. The load type is `const int` and is stripped by a `2C`.
    const IND_THIS_GETTER: &[u8] = &[
        0x53, 0x53, 0x26, 0xE7, 0x09, // fn symbol push
        0xB9, 0xF8, 0x09, 0xA6, 0x43, 0x82, 0x20, // LOAD this (C * const)
        0x99, 0x86, 0x43, 0x84, 0x20, 0x00, // bind-member, offset 0
        0x46, 0x4C, 0x4F, 0x11, 0x53, // formals (none) LO SS
        0xB9, 0xF8, 0x09, 0xA6, 0x43, 0x82, 0x20, // LOAD this
        0x33, 0x86, 0x41, 0x74, 0x04, // LITERAL int 4
        0x27, 0xA6, 0x43, 0x8E, 0x20, // byte-offset add -> const int *
        0x30, 0xA6, 0x41, 0x8D, 0x20, // indirect load -> const int
        0x2C, 0x86, 0x41, 0x74, 0x00, // cv strip -> int
        0x41, 0x86, 0x41, 0x74, //
        0x3A, 0xF9, 0x09, 0x54, 0x02, 0x29, 0xF9, 0x09, //
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn indirect_load_leaf_decodes_deref_member_and_subscript() {
        assert_eq!(
            parse_segment(IND_DEREF, &no_globals()),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            })
        );
        assert_eq!(
            parse_segment(IND_MEMBER0, &no_globals()),
            Some(BodyShape::IndirectLoad {
                params: vec![0xFE09],
                ops: vec![IlOp::Load(0xFE09), IlOp::LoadInd { off: 0 }],
            })
        );
        // The offset is a SIGNED short-form byte, and `-1` on an `int *` is −4
        // bytes — the scale is already applied by the front end.
        assert_eq!(
            parse_segment(IND_SUBSCRIPT_NEG, &no_globals()),
            Some(BodyShape::IndirectLoad {
                params: vec![0x100A],
                ops: vec![IlOp::Load(0x100A), IlOp::LoadInd { off: -4 }],
            })
        );
    }

    #[test]
    fn indirect_load_leaf_binds_this_as_argument_zero() {
        // `this` is not in the `2D` list, so `params` must be built from the
        // pre-body binding — otherwise the base register is unknown (or, worse,
        // an explicit formal is mapped one register low).
        assert_eq!(parse_this_token(IND_THIS_GETTER, 21), Some(0xF809));
        assert_eq!(
            parse_segment(IND_THIS_GETTER, &no_globals()),
            Some(BodyShape::IndirectLoad {
                params: vec![0xF809],
                ops: vec![IlOp::Load(0xF809), IlOp::LoadInd { off: 4 }],
            })
        );
    }

    #[test]
    fn indirect_load_leaf_refuses_the_adjacent_shapes() {
        // Splice one field of IND_DEREF at a time. Each variant is a construct
        // whose reference codegen differs (see fixtures/cpp/il_expr_load_neg.cpp).
        let bad = |patch: &[(usize, u8)]| {
            let mut s = IND_DEREF.to_vec();
            for &(i, b) in patch {
                s[i] = b;
            }
            parse_segment(&s, &no_globals())
        };
        // A `char` pointee is `lbz`, not `lwz` (`30 82 11 70`).
        assert_eq!(bad(&[(16, 0x82), (17, 0x11), (18, 0x70), (20, 0x82), (21, 0x11), (22, 0x70)]), None);
        // A `float` pointee is `lfs` (`30 86 45 40`).
        assert_eq!(bad(&[(17, 0x45), (18, 0x40), (21, 0x45), (22, 0x40)]), None);
        // A pointer pointee (`int **`) emits the same word but stays refused.
        assert_eq!(bad(&[(17, 0x43)]), None);

        // Arithmetic after the load: the load lands in the scratch register, so
        // this must not reach the affine selector.
        let mut with_add = IND_DEREF[..19].to_vec();
        with_add.extend_from_slice(&[0x33, 0x86, 0x41, 0x74, 0x01, 0x02]); // + 1
        with_add.extend_from_slice(&IND_DEREF[19..]);
        assert_eq!(parse_segment(&with_add, &no_globals()), None);

        // A `28` payload other than `00 00` is unexplained and must refuse.
        let mut sub_bad = IND_SUBSCRIPT_NEG.to_vec();
        sub_bad[21] = 0x01;
        assert_eq!(parse_segment(&sub_bad, &no_globals()), None);

        // An offset past the 16-bit displacement materializes an index register.
        let mut wide = IND_SUBSCRIPT_NEG[..19].to_vec();
        wide.extend_from_slice(&[0x80, 0x80, 0x1A, 0x06, 0x00]); // 400000
        wide.extend_from_slice(&IND_SUBSCRIPT_NEG[20..]);
        assert_eq!(parse_segment(&wide, &no_globals()), None);
    }

    #[test]
    fn parse_formals_anchors_on_the_marker_that_reaches_lo() {
        // A function on source line 70 emits the line marker `4F 01 46`, whose
        // payload byte is `0x46`. Taking the first `0x46` in the segment finds
        // *that* and silently yields an empty formals list — which is not
        // fail-closed, because `leaves_ascending` skips non-formal tokens.
        let mut seg = vec![0x4F, 0x01, 0x46]; // line 70
        seg.extend_from_slice(IND_DEREF);
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        assert_eq!(parse_formals(&seg, lo), Ok(vec![0xEE09]));
        // And the whole body still parses, base register included.
        assert_eq!(
            parse_segment(&seg, &no_globals()),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            })
        );
    }

    #[test]
    fn chains_canonicalize_to_c2s_register_order() {
        let p = vec![0x10, 0x11, 0x12]; // a -> r3, b -> r4, c -> r5
        let (a, b, c) = (IlOp::Load(0x10), IlOp::Load(0x11), IlOp::Load(0x12));
        let canon = |ops: Vec<IlOp>| canonicalize_chain(&ops, &p).unwrap();

        // Every permutation of `a + b + c` collapses to the same stream, because c2
        // emits the same `add r11,r3,r4 ; add r3,r11,r5` for all five.
        let want = vec![a, b, IlOp::Add, c, IlOp::Add];
        for perm in [
            vec![a, b, IlOp::Add, c, IlOp::Add],
            vec![a, c, IlOp::Add, b, IlOp::Add],
            vec![b, a, IlOp::Add, c, IlOp::Add],
            vec![b, c, IlOp::Add, a, IlOp::Add],
            vec![c, b, IlOp::Add, a, IlOp::Add],
        ] {
            assert_eq!(canon(perm), want);
        }
        // `b + a` -> `a + b`.
        assert_eq!(canon(vec![b, a, IlOp::Add]), vec![a, b, IlOp::Add]);
        // A multiplicative chain sorts ascending.
        assert_eq!(
            canon(vec![a, c, IlOp::Mul, b, IlOp::Mul]),
            vec![a, b, IlOp::Mul, c, IlOp::Mul]
        );

        // Additive: negatives first, from the lowest positive. `a + b - c` and
        // `b - c + a` both become `(a - c) + b`.
        let want_mixed = vec![a, c, IlOp::Sub, b, IlOp::Add];
        assert_eq!(canon(vec![a, b, IlOp::Add, c, IlOp::Sub]), want_mixed);
        assert_eq!(canon(vec![b, c, IlOp::Sub, a, IlOp::Add]), want_mixed);
        // Two negatives sort ascending: `a - c - b` becomes `(a - b) - c`.
        assert_eq!(
            canon(vec![a, c, IlOp::Sub, b, IlOp::Sub]),
            vec![a, b, IlOp::Sub, c, IlOp::Sub]
        );
        // Literals fold into one constant applied last, so they never affect order.
        assert_eq!(
            canon(vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Sub]),
            vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Sub]
        );
        assert_eq!(
            canon(vec![a, IlOp::Lit(1), IlOp::Add, IlOp::Lit(2), IlOp::Sub]),
            vec![a, IlOp::Lit(1), IlOp::Sub]
        );

        // Shapes it must decline rather than mangle: a tree (so
        // `try_select_depth2_tree` still sees it), a `*` mixed with `+`, a
        // multiply by a constant (which strength-reduces), and a chain with no
        // positive register to start from.
        assert!(canonicalize_chain(&[a, b, IlOp::Add, c, a, IlOp::Add, IlOp::Mul], &p).is_none());
        assert!(canonicalize_chain(&[a, b, IlOp::Mul, c, IlOp::Add], &p).is_none());
        assert!(canonicalize_chain(&[a, IlOp::Lit(2), IlOp::Mul], &p).is_none());
        assert!(canonicalize_chain(&[IlOp::Lit(1), a, IlOp::Sub], &p).is_none());
    }

    #[test]
    fn reassociation_gates_separate_canonical_from_rewritten_chains() {
        // params[i] is register r(3+i), so ascending index == ascending register.
        let p = vec![0x10, 0x11, 0x12]; // a, b, c
        let ld = |t: u32| IlOp::Load(t);

        // Commutative chains are canonicalized by register: every permutation of
        // `a + b + c` emits the same bytes, so only the ascending one may be
        // accepted in source order.
        assert!(leaves_ascending(&[ld(0x10), ld(0x11), IlOp::Add], &p)); // a + b
        assert!(!leaves_ascending(&[ld(0x11), ld(0x10), IlOp::Add], &p)); // b + a
        assert!(leaves_ascending(
            &[ld(0x10), ld(0x11), IlOp::Add, ld(0x12), IlOp::Add],
            &p
        )); // a + b + c
        assert!(!leaves_ascending(
            &[ld(0x10), ld(0x12), IlOp::Add, ld(0x11), IlOp::Add],
            &p
        )); // a + c + b
        // Literals do not participate in the ordering.
        assert!(leaves_ascending(&[ld(0x10), IlOp::Lit(1), IlOp::Add, ld(0x11), IlOp::Add], &p));

        // A mixed chain is reassociated even when the operands ARE in register
        // order: c2 applies the negative terms first. `a - b + c` is already that
        // order and is byte-exact; `a + b - c` is not.
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Sub,
            ld(0x12),
            IlOp::Add
        ])); // a - b + c
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Sub,
            ld(0x12),
            IlOp::Sub
        ])); // a - b - c
        assert!(!additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Add,
            ld(0x12),
            IlOp::Sub
        ])); // a + b - c  -> c2 emits (a - c) + b
        // Subtracting a LITERAL folds into the `addi` immediate and emits no
        // instruction, so it can never be out of order.
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Add,
            IlOp::Lit(1),
            IlOp::Sub
        ])); // a + b - 1
    }

    #[test]
    fn repeated_leaves_are_refused_before_and_after_substitution() {
        // Written directly: `a + a`. c2 emits `slwi r3,r3,1`, not `add r3,r3,r3`,
        // so accepting this is wrong bytes rather than a missing feature.
        assert!(has_repeated_leaf(&[IlOp::Load(1), IlOp::Load(1), IlOp::Add]));
        assert!(has_repeated_leaf(&[
            IlOp::Load(1),
            IlOp::Load(2),
            IlOp::Add,
            IlOp::Load(1),
            IlOp::Add
        ]));
        // Distinct operands, and a literal reused, are both fine.
        assert!(!has_repeated_leaf(&[IlOp::Load(1), IlOp::Load(2), IlOp::Add]));
        assert!(!has_repeated_leaf(&[
            IlOp::Load(1),
            IlOp::Lit(1),
            IlOp::Add,
            IlOp::Lit(1),
            IlOp::Add
        ]));

        // Substitution CREATES repetition that the source did not have:
        // `int x = a; x = x + x;` has no repeated operand written anywhere, but
        // resolves to `a + a`. This is why the gate runs on the resolved stream.
        let env = vec![(0x100, vec![IlOp::Load(1)])];
        let resolved = substitute(&[IlOp::Load(0x100), IlOp::Load(0x100), IlOp::Add], &env).unwrap();
        assert_eq!(resolved, vec![IlOp::Load(1), IlOp::Load(1), IlOp::Add]);
        assert!(has_repeated_leaf(&resolved));
    }

    #[test]
    fn substitution_captures_operands_at_definition_time() {
        // `int x = a; a = a + 1; return x;` must return the ENTRY `a`. Recording
        // definitions already-substituted is what guarantees it: a later
        // redefinition of `a` cannot reach backwards into `x`'s definition.
        // Substituting lazily at use time would wrongly yield `a + 1`.
        let mut env: Vec<(u32, Vec<IlOp>)> = Vec::new();
        // int x = a;
        env.push((0x100, substitute(&[IlOp::Load(1)], &env).unwrap()));
        // a = a + 1;
        let rhs = substitute(&[IlOp::Load(1), IlOp::Lit(1), IlOp::Add], &env).unwrap();
        env.retain(|(t, _)| *t != 1);
        env.push((1, rhs));
        // return x;
        assert_eq!(substitute(&[IlOp::Load(0x100)], &env).unwrap(), vec![IlOp::Load(1)]);
        // return a; would instead be the incremented value.
        assert_eq!(
            substitute(&[IlOp::Load(1)], &env).unwrap(),
            vec![IlOp::Load(1), IlOp::Lit(1), IlOp::Add]
        );
    }

    #[test]
    fn census_names_the_first_blocking_opcode() {
        // A comparison (`24` GT) in the operand stream buckets as `expr-cmp-gt`,
        // and the offset points at the `24` itself — not at some later byte.
        let cmp: &[u8] = &[
            0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01, 0x10,
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, 0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74,
            0x24, // GT
            0x43, 0x42, 0x00, 0x00, 0x41, 0x86, 0x41, 0x74,
        ];
        let b = parse_segment_detail(cmp, &no_globals()).unwrap_err();
        assert_eq!(b.feature(), "expr-cmp-gt");
        assert_eq!(cmp[b.off], 0x24);
    }

    #[test]
    fn census_reports_the_whole_operand_type_not_its_shared_first_byte() {
        // An `unsigned` operand's inline type shares its first byte (`86`) with
        // `int`, so bucketing on that byte would merge every non-int type into
        // one meaningless class. The bucket must carry the full triple.
        let mut seg = INT_TAILRET.to_vec();
        // Corrupt the argument LOAD's type `86 41 74` → `86 41 75` (a distinct
        // type), leaving everything else intact.
        let load = seg.windows(6).position(|w| w == [0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74]).unwrap();
        seg[load + 5] = 0x75;
        let b = parse_segment_detail(&seg, &no_globals()).unwrap_err();
        assert_eq!(b.feature(), "expr-load-type-864175");
        assert_eq!(seg[b.off], 0xB9, "reported at the LOAD, not mid-type");
    }

    #[test]
    fn census_classifies_each_function_independently() {
        // The point of P2b: one blocked function does not hide the in-class
        // ones. `functions()` (the gate) is all-or-nothing and returns None.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&FN_START);
        ex.extend_from_slice(TWO_CALLS);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), ex),
                ("gl".to_string(), b"?f@@YAXXZ\x00?g@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2);
        assert_eq!(census[0].verdict, FnVerdict::InClass("void-tail-call"));
        assert!(!census[1].verdict.in_class());
        assert_eq!(census[0].name.as_deref(), Some("?f@@YAXXZ"));
        // In-class functions carry no hex window; blocked ones point at the
        // offending byte inside theirs.
        assert!(census[0].hex.is_empty());
        let FnVerdict::Blocked(b) = census[1].verdict else {
            panic!("expected a block");
        };
        assert_eq!(census[1].hex[census[1].hex_mark], b.byte.unwrap());
    }

    #[test]
    fn census_hex_window_is_clamped_to_the_segment() {
        // A block at offset 0 must not underflow, and one near the end must not
        // run past it — the window is diagnostic and must never panic.
        let tiny: &[u8] = &[0x4C, 0x4F, 0x11, 0xFF];
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), tiny.to_vec()),
                ("gl".to_string(), b"?f@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 1);
        let c = &census[0];
        assert!(!c.verdict.in_class());
        assert!(c.hex_mark < c.hex.len().max(1));
        assert!(c.hex.len() <= CENSUS_HEX_BACK + CENSUS_HEX_FWD);
    }

    // ---- intrinsic-call (`0x40`) decode -------------------------------------
    //
    // Every byte array below is transcribed verbatim from a live-toolchain `.ex`
    // capture of a tracked fixture (`c2rs census <fixture> --keep-il <dir>`), not
    // hand-assembled — the whole point of the production is that its field widths
    // were guessed wrong twice before a capture settled them.

    /// `double t_fabs(double a){ return fabs(a); }`
    /// (`fixtures/cpp/il_intrinsic_call.cpp`, `?t_fabs@@YANN@Z`). Selector 17.
    const INTR_FABS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x11, 0x40, 0x88, 0x85, 0x41, 0xB9, 0x17,
        0x0A, 0x88, 0x85, 0x41, 0x55, 0x88, 0x85, 0x41, 0x4C, 0x41, 0x88, 0x85, 0x41, 0x3A, 0x19,
        0x0A, 0x54, 0x02, 0x29, 0x19, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `void n_break(){ __debugbreak(); }`
    /// (`fixtures/cpp/il_intrinsic_nullary.cpp`, `?n_break@@YAXXZ`). Selector 543,
    /// **zero arguments** — the witness that `40 <TYPE>` carries no trailing field.
    const INTR_NULLARY: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x80, 0x1F, 0x02, 0x00, 0x00, 0x40, 0x82,
        0x07, 0x03, 0x4C, 0x4B, 0x3A, 0xFF, 0x09, 0x54, 0x02, 0x29, 0xFF, 0x09, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];
    /// `A2 *l_up2(M *m){ return m; }`
    /// (`fixtures/cpp/il_intrinsic_layout.cpp`). Selector 2114, offset literal `08`.
    const INTR_UPCAST: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x80, 0x42, 0x08, 0x00, 0x00, 0x40, 0x86,
        0x43, 0xB1, 0x20, 0x66, 0x02, 0x92, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86,
        0x41, 0x74, 0x08, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x41, 0x0A, 0x86, 0x43, 0xB0, 0x20, 0x55,
        0x86, 0x43, 0xB0, 0x20, 0x4C, 0x41, 0x86, 0x43, 0xB1, 0x20, 0x3A, 0x43, 0x0A, 0x54, 0x02,
        0x29, 0x43, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `void l_this2(M *m){ m->mb(); }`
    /// (`fixtures/cpp/il_intrinsic_layout.cpp`). Selector 2113, offset literal `08`
    /// — byte-for-byte the same descriptor and offset as [`INTR_UPCAST`], reached
    /// through the `26 <sym>` path instead.
    const INTR_THIS_ADJUST: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF2, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00,
        0x00, 0x40, 0xA6, 0x43, 0x96, 0x20, 0x66, 0x02, 0x92, 0x20, 0x93, 0x20, 0x55, 0x86, 0x41,
        0x74, 0x33, 0x86, 0x41, 0x74, 0x08, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x48, 0x0A, 0x86, 0x43,
        0xB0, 0x20, 0x55, 0x86, 0x43, 0xB0, 0x20, 0x4C, 0x99, 0x86, 0x43, 0x97, 0x20, 0x00, 0xBD,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x17, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0x4A, 0x0A, 0x54,
        0x02, 0x29, 0x4A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn intrinsic_call_census_reports_the_selector_not_the_opcode() {
        // The whole `0x40` production is one census bucket only because the
        // selector was never decoded. Every site must name the intrinsic.
        //
        // `INTR_THIS_ADJUST` reports `expr-` rather than `call-` since the body
        // dispatch keys on whether a `BD` follows the first `26 <tok>` immediately.
        // Here it does not — the `BD` is fifty bytes later, behind argument-shaped
        // material — so the body goes to the assignment parser and the intrinsic is
        // named from the expression it sits in. That is the claim I can support from
        // these bytes; asserting the enclosing construct is a call would be
        // asserting more. The selector is named either way, so the histogram is
        // unaffected in aggregate, and `intrinsic_call_decode_does_not_accept`
        // pins that both routings still refuse.
        for (seg, want) in [
            (INTR_FABS, "expr-intrinsic-fabs"),
            (INTR_NULLARY, "expr-intrinsic-__debugbreak"),
            (INTR_UPCAST, "expr-intrinsic-base-upcast"),
            (INTR_THIS_ADJUST, "expr-intrinsic-this-adjust"),
        ] {
            let b = parse_segment_detail(seg, &no_globals()).unwrap_err();
            assert_eq!(b.feature(), want);
            // The block is reported at the selector literal, whose `40` follows.
            assert_eq!(seg[b.off], 0x33, "{want}");
        }
    }

    #[test]
    fn intrinsic_call_decode_does_not_accept() {
        // Decoding is not accepting. Every one of these still fails closed, so
        // the census and the emission gate cannot disagree — the same invariant
        // `census_agrees_with_the_gate_on_every_pinned_segment` checks globally.
        for seg in [INTR_FABS, INTR_NULLARY, INTR_UPCAST, INTR_THIS_ADJUST] {
            assert!(parse_segment(seg, &no_globals()).is_none());
        }
    }

    #[test]
    fn intrinsic_call_token_has_no_trailing_field() {
        // `40 <TYPE>` and nothing else: in the nullary capture the `4C` apply sits
        // immediately after the `void` result type, so a `40 <TYPE> <varint>`
        // reading (the shape `2C`/`99`/`9B`/`5C` have, and the one an earlier
        // session assumed) would swallow the terminator.
        let p = 4; // the selector literal, right after `4C 4F 11 53`
        assert_eq!(intrinsic_selector(INTR_NULLARY, p), Some(543));
        let tok = p + 9; // `33 86 41 74` + the 5-byte escaped varint
        assert_eq!(INTR_NULLARY[tok], 0x40);
        let (_, _, _, w) = read_type(INTR_NULLARY, tok + 1).unwrap();
        assert_eq!(&INTR_NULLARY[tok + 1..tok + 1 + w], &[0x82, 0x07, 0x03]); // void
        assert_eq!(INTR_NULLARY[tok + 1 + w], 0x4C); // the apply, with no field between
    }

    #[test]
    fn same_descriptor_and_offset_different_selector_is_a_different_emission() {
        // 2113 and 2114 carry an identical `66 02 92 20 93 20` class-pair
        // descriptor and an identical offset literal `08`, and c2 emits
        // `addi r3,r3,8` for one and a null-guarded five-instruction form for the
        // other (see `fixtures/cpp/il_intrinsic_layout.cpp`). So the census must
        // separate them, and a lowering keyed on the offset alone would be wrong.
        let up = parse_segment_detail(INTR_UPCAST, &no_globals()).unwrap_err();
        let this = parse_segment_detail(INTR_THIS_ADJUST, &no_globals()).unwrap_err();
        assert_ne!(up.feature(), this.feature());
        assert_eq!(up.aux, 2114);
        assert_eq!(this.aux, 2113);
        // Both offset literals really are the same byte.
        assert_eq!(INTR_UPCAST[32], 0x08);
        assert_eq!(INTR_THIS_ADJUST[35], 0x08);
    }

    #[test]
    fn selector_must_be_exactly_int_typed_or_the_decode_declines() {
        // The one structural claim the decode rests on is that `0x40` is always
        // preceded by an `int`-typed literal. Retype the `t_fabs` selector to
        // `unsigned` (`86 42 75`) and the decode must decline rather than report a
        // selector it cannot vouch for — falling back to the honest
        // `expr-intrinsic-call` residue, which is what measures the claim over the
        // real workload (measured: 0 of 213,411 sites land in the residue).
        let mut seg = INTR_FABS.to_vec();
        seg[5] = 0x86;
        seg[6] = 0x42;
        seg[7] = 0x75;
        assert_eq!(intrinsic_selector(&seg, 4), None);
        let b = parse_segment_detail(&seg, &no_globals()).unwrap_err();
        assert_eq!(b.feature(), "expr-intrinsic-call");
    }

    #[test]
    fn unpinned_selector_ids_stay_hex() {
        // A hex bucket is a result; a wrong name is a lie that survives into the
        // roadmap. 222/223 occur 1758 times each on the real workload and their
        // trigger is pinned (`fixtures/cpp/il_intrinsic_byval.cpp`) while their
        // individual semantics are not, so they must not be named.
        assert_eq!(intrinsic_name(222), "0xDE");
        assert_eq!(intrinsic_name(223), "0xDF");
        assert_eq!(intrinsic_name(2120), "0x848");
        assert_eq!(intrinsic_name(17), "fabs");
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
    // The three ACCEPTED integer tail-call segments (transcribed from live
    // 16.00.11886.00 `.ex` captures of `return g(a)`, `g(a)+0`, `g(a+1)`).
    // Unlike the void/framed constants above, these start at the `46` formals
    // marker (param a = token 0xE509) — the arg-setup codegen maps the argument
    // tokens to registers, so the formal list must be present.

    /// `int f(int a){ return g(a); }` — passthrough: arg region is the bare
    /// LOAD, no post-op → integer tail call (bare `b g`).
    const INT_TAILRET: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a (the argument)
        0x55, 0x86, 0x41, 0x74, 0x4C, // 55 <int> 4C call-end
        0x41, 0x86, 0x41, 0x74, // result-type int (no post-op → tail call)
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int f(int a){ return g(a) + 0; }` — identity fold: same arg LOAD, then a
    /// real `33 86 41 74 00 02` (LIT 0 + ADD) post-op that folds to a tail call.
    const INT_PLUS0: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0x55, 0x86, 0x41, 0x74, 0x4C, // call-end
        0x33, 0x86, 0x41, 0x74, 0x00, 0x02, // post-op LIT 0 + ADD (folds away)
        0x41, 0x86, 0x41, 0x74, // result-type int
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];
    /// `int f(int a){ return g(a + 1); }` — arg-setup: the `+1` is IN the
    /// argument (LOAD+LIT+ADD before `55`), no post-op → integer tail call
    /// (`addi r3,r3,1 ; b g`). Not to be mistaken for framed `g(a)+1`.
    const INT_ARGTAIL: &[u8] = &[
        0x46, 0x2D, 0xE5, 0x09, // formals: a = e509
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // 26 CALL
        0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD a
        0x33, 0x86, 0x41, 0x74, 0x01, 0x02, // LIT 1 + ADD (computes a+1 into the arg)
        0x55, 0x86, 0x41, 0x74, 0x4C, // call-end
        0x41, 0x86, 0x41, 0x74, // result-type int (no post-op → tail call)
        0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, // assign + return
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
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
