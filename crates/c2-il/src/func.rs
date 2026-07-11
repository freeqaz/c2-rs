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

/// A single straight-line IL operation in the MVP add-chain class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IlOp {
    /// Load a named variable (by IL token) onto the expression stack.
    Load(u16),
    /// Pop two operands, push their integer sum.
    Add,
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
    /// Straight-line body op stream (loads + adds).
    pub ops: Vec<IlOp>,
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
    if formals_rev.is_empty() {
        return None;
    }
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

impl IlBundle {
    /// Parse this bundle as an MVP straight-line add-chain function. Returns
    /// `None` if the required files are absent or the body is outside the MVP
    /// class (the caller — `PortC2` — then reports `NotImplemented`).
    pub fn mvp_function(&self) -> Option<IlFunction> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let mangled = mangled_name(gl)?;
        let src = source_path(gl);
        let tw = detect_token_width(ex);
        let (params, ops) = parse_body(ex, tw)?;
        Some(IlFunction {
            mangled_name: mangled,
            source_path: src,
            params,
            ops,
        })
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
