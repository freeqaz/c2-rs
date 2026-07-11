//! **K1 — the lossless IL container codec with typed islands.**
//!
//! [`IlModel::parse`] walks the five files of an [`IlBundle`] and produces a
//! structured model whose leaves are either (a) **typed, decoded tokens** for
//! the classes the grammar is known for (the `.ex` operand stream that
//! [`crate::func`] already recognizes, and the `.gl` `80 <LE32>` body-start
//! offset field), or (b) **opaque byte spans** for every not-yet-decoded region
//! (the `.ex` header + per-function metadata, the rest of `.gl`, and all of
//! `.sy`/`.in`/`.db`). [`IlModel::encode`] serializes the model back to bytes.
//!
//! **Invariant (fail-closed):** `encode(parse(bundle)) == bundle` byte-for-byte
//! for every file, or [`IlModel::parse`] returns [`CodecError::CannotRoundTrip`]
//! — it never silently loses or reorders a byte. This holds *by construction*:
//! every byte is either consumed by a typed token that re-encodes to exactly the
//! bytes it consumed, or coalesced into an [`Span::Opaque`] run that is emitted
//! verbatim. [`IlModel::parse`] additionally re-encodes each file and compares
//! before returning, so a decoding bug surfaces as an error, never a mis-emit.
//!
//! This is deliberately a **container codec**, not a full IL disassembler: an
//! undecoded region kept opaque is correct; a mis-decoded region that does not
//! round-trip is a bug. New decoded structure is added by teaching the walker a
//! new typed token (which shrinks the opaque map), never by loosening the gate.
//!
//! ## The `.gl` body-start offset — modeled first-class
//!
//! Per `il-witness` P0.6a, each function's `.gl` record ends `80 <LE32>` where
//! the LE32 is the **`.ex` byte offset of that function's `4F 1F` body-start
//! marker**. That is the one length-bearing field K3 must rewrite when an `.ex`
//! function changes length, so it is typed as [`Span::GlOffset`] (a `u32`), not
//! opaque. K1 only round-trips it unchanged; the field is located robustly by
//! cross-checking against the actual set of `4F 1F` offsets in `.ex`.

use std::collections::BTreeSet;

use crate::{detect_token_width, IlBundle};

/// The int type encoding inline in the `.ex` body (`86 41 74`). Mirrors
/// `func::INT_TYPE`; duplicated here to keep [`crate::func`] untouched.
const INT_TYPE: [u8; 3] = [0x86, 0x41, 0x74];

/// The fixed 6-byte callee-reference tail of a CALL token
/// (`00 80 01 10 00 00`). Mirrors `func::CALL_CALLEE_ANCHOR`.
const CALL_CALLEE_ANCHOR: [u8; 6] = [0x00, 0x80, 0x01, 0x10, 0x00, 0x00];

/// The `.ex` per-function start marker (`4F 1F`). Mirrors `func::FN_START`.
const FN_START: [u8; 2] = [0x4F, 0x1F];

/// The `4C 4F 11` 'LO' body-start marker — the point from which the `.ex`
/// operand stream of a function is a typed token sequence.
const LO_MARKER: [u8; 3] = [0x4C, 0x4F, 0x11];

/// A single decoded `.ex` operand-stream token. Every variant re-encodes to
/// *exactly* the bytes it was parsed from (see [`ExToken::encode_into`]), so a
/// span list of these plus [`Span::Opaque`] runs round-trips byte-identically.
///
/// The token classes mirror the grammar in [`crate::func`] (`parse_segment`)
/// and `docs/IL_BUNDLE_MVP.md`. All tokens are decoded at token width 2 (every
/// captured bundle); a stream at another width is left fully opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExToken {
    /// `4C 4F 11` — 'LO' load-operands body-start marker.
    Lo,
    /// `53` — 'SS' statement-start.
    Ss,
    /// `4F 01 NN` — per-statement sequence/label marker (multi-fn TUs).
    Stmt(u8),
    /// `26 <tok>` — function/result-temp reference (precedes a CALL).
    ResultRef(u16),
    /// `BD <3-byte ret type> 00 80 01 10 00 00` — the fixed 10-byte CALL token.
    Call([u8; 3]),
    /// `B9 <tok> 86 41 74` — LOAD an int operand.
    Load(u16),
    /// `33 86 41 74 <varint>` — push an int literal. `wide` selects the varint
    /// form (`false` = single byte `< 0x80`; `true` = `80` + 4-byte LE i32) so a
    /// value representable narrowly but encoded wide (a P0.6a length pad) is
    /// preserved exactly.
    Lit { value: i32, wide: bool },
    /// `02` — ADD (postfix, commutative).
    Add,
    /// `03` — SUB (postfix, NON-commutative).
    Sub,
    /// `04` — MUL (postfix, commutative).
    Mul,
    /// `55 86 41 74 4C` — int call-end (consumed value) + its 'L' marker.
    IntCallEnd,
    /// `4C 4B` — void call-end (discarded value).
    VoidCallEnd,
    /// `41 86 41 74` — int result-type annotation.
    ResultType,
    /// `3A <tok>` — ASSIGN to a temp.
    Assign(u16),
    /// `54 02 29 <tok>` — RETURN a temp.
    Return(u16),
    /// `4F 12 47 54 01 54 00` — function-tail separator + 'GT' terminate.
    FnTail,
    /// `4F 02 20 00 4F 01 NN 4D` — module end (last function only).
    ModuleEnd(u8),
    /// `46` — 'F' formal-parameter list marker.
    Formals,
    /// `2D <tok>` — one formal-parameter entry.
    Formal(u16),
}

impl ExToken {
    /// Append this token's exact byte encoding to `out`.
    fn encode_into(&self, out: &mut Vec<u8>) {
        let tok = |out: &mut Vec<u8>, t: u16| {
            out.push((t >> 8) as u8);
            out.push((t & 0xFF) as u8);
        };
        match *self {
            ExToken::Lo => out.extend_from_slice(&LO_MARKER),
            ExToken::Ss => out.push(0x53),
            ExToken::Stmt(nn) => out.extend_from_slice(&[0x4F, 0x01, nn]),
            ExToken::ResultRef(t) => {
                out.push(0x26);
                tok(out, t);
            }
            ExToken::Call(ret) => {
                out.push(0xBD);
                out.extend_from_slice(&ret);
                out.extend_from_slice(&CALL_CALLEE_ANCHOR);
            }
            ExToken::Load(t) => {
                out.push(0xB9);
                tok(out, t);
                out.extend_from_slice(&INT_TYPE);
            }
            ExToken::Lit { value, wide } => {
                out.push(0x33);
                out.extend_from_slice(&INT_TYPE);
                if wide {
                    out.push(0x80);
                    out.extend_from_slice(&value.to_le_bytes());
                } else {
                    out.push(value as u8);
                }
            }
            ExToken::Add => out.push(0x02),
            ExToken::Sub => out.push(0x03),
            ExToken::Mul => out.push(0x04),
            ExToken::IntCallEnd => {
                out.push(0x55);
                out.extend_from_slice(&INT_TYPE);
                out.push(0x4C);
            }
            ExToken::VoidCallEnd => out.extend_from_slice(&[0x4C, 0x4B]),
            ExToken::ResultType => {
                out.push(0x41);
                out.extend_from_slice(&INT_TYPE);
            }
            ExToken::Assign(t) => {
                out.push(0x3A);
                tok(out, t);
            }
            ExToken::Return(t) => {
                out.extend_from_slice(&[0x54, 0x02, 0x29]);
                tok(out, t);
            }
            ExToken::FnTail => out.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]),
            ExToken::ModuleEnd(nn) => {
                out.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, nn, 0x4D]);
            }
            ExToken::Formals => out.push(0x46),
            ExToken::Formal(t) => {
                out.push(0x2D);
                tok(out, t);
            }
        }
    }
}

/// One leaf of a file's model: a decoded typed island or an opaque byte run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    /// A not-yet-decoded region, preserved verbatim.
    Opaque(Vec<u8>),
    /// A decoded `.ex` operand-stream token.
    Ex(ExToken),
    /// The `.gl` `80 <LE32>` body-start offset field: the `.ex` byte offset of a
    /// function's `4F 1F` marker. The one field K3 rewrites on `.ex` length
    /// change; K1 round-trips it unchanged.
    GlOffset(u32),
}

impl Span {
    fn encode_into(&self, out: &mut Vec<u8>) {
        match self {
            Span::Opaque(b) => out.extend_from_slice(b),
            Span::Ex(t) => t.encode_into(out),
            Span::GlOffset(v) => {
                out.push(0x80);
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
}

/// The decoded model of one bundle file: its suffix and an ordered span list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileModel {
    /// Suffix, e.g. `"ex"`, `"gl"`.
    pub suffix: String,
    /// Ordered leaves; concatenating [`Span::encode_into`] reproduces the file.
    pub spans: Vec<Span>,
}

impl FileModel {
    /// Serialize this file's spans back to bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for s in &self.spans {
            s.encode_into(&mut out);
        }
        out
    }
}

/// The decoded model of a whole [`IlBundle`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IlModel {
    /// Bundle base, suffix-free, e.g. `_CL_dfd7b253`.
    pub base_name: String,
    /// One [`FileModel`] per present file, in the bundle's suffix order.
    pub files: Vec<FileModel>,
}

/// A codec failure. Today the only variant is the fail-closed round-trip guard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    /// A file did not re-encode to its exact input bytes — a decoding bug. The
    /// codec refuses to hand back a model it cannot losslessly serialize.
    CannotRoundTrip {
        /// The offending file's suffix.
        suffix: String,
        /// First byte offset at which re-encoding diverged.
        first_offset: usize,
        /// Original file length.
        orig_len: usize,
        /// Re-encoded length.
        encoded_len: usize,
    },
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::CannotRoundTrip {
                suffix,
                first_offset,
                orig_len,
                encoded_len,
            } => write!(
                f,
                "codec cannot round-trip .{suffix}: diverged at offset {first_offset} \
                 (orig {orig_len} B, re-encoded {encoded_len} B)"
            ),
        }
    }
}

impl std::error::Error for CodecError {}

impl IlModel {
    /// Parse `bundle` into a structured, losslessly-re-encodable model.
    ///
    /// Fail-closed: after building the model, each file is re-encoded and
    /// compared to its input; a mismatch returns [`CodecError::CannotRoundTrip`]
    /// rather than a model that would silently corrupt on [`IlModel::encode`].
    pub fn parse(bundle: &IlBundle) -> Result<IlModel, CodecError> {
        // The set of `.ex` function-start (`4F 1F`) offsets — the discriminator
        // that tells a real `.gl` body-start offset field from a coincidental
        // `80 <LE32>` elsewhere in `.gl`.
        let ex = bundle.get("ex").unwrap_or(&[]);
        let ex_offsets = ex_fn_start_offsets(ex);

        let mut files = Vec::new();
        // Iterate the bundle's own file set (BTreeMap → suffix-sorted) so the
        // model mirrors exactly what is present, in a stable order.
        for (suffix, bytes) in &bundle.files {
            let spans = match suffix.as_str() {
                "ex" => parse_ex(bytes),
                "gl" => parse_gl(bytes, &ex_offsets),
                // `.sy`, `.in`, `.db` are not decoded yet (K2 backlog).
                _ => vec![opaque(bytes)],
            };
            let fm = FileModel {
                suffix: suffix.clone(),
                spans,
            };
            // Fail-closed round-trip guard for this file.
            let re = fm.encode();
            if re != *bytes {
                let first_offset = re
                    .iter()
                    .zip(bytes.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(re.len().min(bytes.len()));
                return Err(CodecError::CannotRoundTrip {
                    suffix: suffix.clone(),
                    first_offset,
                    orig_len: bytes.len(),
                    encoded_len: re.len(),
                });
            }
            files.push(fm);
        }

        Ok(IlModel {
            base_name: bundle.base_name.clone(),
            files,
        })
    }

    /// Serialize the model back to an [`IlBundle`]. `encode(parse(b)) == b`.
    pub fn encode(&self) -> IlBundle {
        let mut bundle = IlBundle::new(self.base_name.clone());
        for fm in &self.files {
            bundle.set(fm.suffix.clone(), fm.encode());
        }
        bundle
    }

    /// The typed `.gl` body-start offsets, in file order — the `.ex` `4F 1F`
    /// offsets K3 will rewrite. Empty if `.gl` is absent or carried none.
    pub fn gl_body_start_offsets(&self) -> Vec<u32> {
        self.files
            .iter()
            .find(|f| f.suffix == "gl")
            .map(|f| {
                f.spans
                    .iter()
                    .filter_map(|s| match s {
                        Span::GlOffset(v) => Some(*v),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The decoded `.ex` tokens, in stream order (opaque regions skipped).
    pub fn ex_tokens(&self) -> Vec<ExToken> {
        self.files
            .iter()
            .find(|f| f.suffix == "ex")
            .map(|f| {
                f.spans
                    .iter()
                    .filter_map(|s| match s {
                        Span::Ex(t) => Some(t.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn opaque(b: &[u8]) -> Span {
    Span::Opaque(b.to_vec())
}

/// Byte offsets of every `4F 1F` function-start marker in `.ex`.
fn ex_fn_start_offsets(ex: &[u8]) -> BTreeSet<u32> {
    let mut set = BTreeSet::new();
    let mut i = 0;
    while i + 1 < ex.len() {
        if ex[i] == FN_START[0] && ex[i + 1] == FN_START[1] {
            set.insert(i as u32);
            i += 2;
        } else {
            i += 1;
        }
    }
    set
}

/// Model the `.ex` stream: an opaque header (up to the first `4F 1F`), then per
/// function an opaque metadata prefix (`4F 1F` … up to the `4C 4F 11` 'LO'
/// marker) followed by a typed walk of the body from 'LO' to the segment end.
/// Regions the walk does not recognize become opaque bytes, so the whole file
/// round-trips regardless of what is decoded.
fn parse_ex(ex: &[u8]) -> Vec<Span> {
    let mut spans = Vec::new();
    let starts: Vec<usize> = ex_fn_start_offsets(ex).iter().map(|&o| o as usize).collect();
    if starts.is_empty() {
        return vec![opaque(ex)];
    }
    // Opaque header before the first function.
    if starts[0] > 0 {
        spans.push(opaque(&ex[..starts[0]]));
    }
    let tw = detect_token_width(ex);
    for (k, &s) in starts.iter().enumerate() {
        let e = starts.get(k + 1).copied().unwrap_or(ex.len());
        let seg = &ex[s..e];
        match find_subslice(seg, &LO_MARKER) {
            Some(lo) => {
                // The formal-parameter list `46 (2D <tok>)*` sits in the metadata
                // prefix, anchored immediately before the LO marker. Type it (so
                // the model exposes the formals), keep the rest of the prefix
                // opaque. `fstart` is the `46` marker index, or `lo` if absent.
                let fstart = if tw == 2 {
                    formals_marker(seg, lo, tw)
                } else {
                    lo
                };
                if fstart > 0 {
                    spans.push(opaque(&seg[..fstart]));
                }
                let mut q = fstart;
                if q < lo {
                    // fstart points at `46`; emit Formals then each `2D <tok>`.
                    spans.push(Span::Ex(ExToken::Formals));
                    q += 1;
                    while q + 3 <= lo && seg[q] == 0x2D {
                        let t = ((seg[q + 1] as u16) << 8) | seg[q + 2] as u16;
                        spans.push(Span::Ex(ExToken::Formal(t)));
                        q += 3;
                    }
                }
                // Any bytes between the formals run and LO (none in practice)
                // stay opaque so the boundary is never dropped.
                if q < lo {
                    spans.push(opaque(&seg[q..lo]));
                }
                walk_ex_body(&seg[lo..], tw, &mut spans);
            }
            // No body marker in this segment — keep it wholly opaque.
            None => spans.push(opaque(seg)),
        }
    }
    spans
}

/// Locate the `46` formal-parameter marker in a segment by walking back from
/// the LO marker over `(2D <tok>)*` groups: the formals run is `46 (2D <tok>)*`
/// ending immediately before LO (an empty list is a bare `46` before LO).
/// Returns the `46` index, or `lo` if no formals marker anchors there.
fn formals_marker(seg: &[u8], lo: usize, _tw: usize) -> usize {
    let mut end = lo;
    while end >= 3 && seg[end - 3] == 0x2D {
        end -= 3;
    }
    if end >= 1 && seg[end - 1] == 0x46 {
        end - 1
    } else {
        lo
    }
}

/// Greedy typed-token walk of an `.ex` function body (from the 'LO' marker).
/// Unrecognized bytes coalesce into opaque runs. Token reads assume width 2;
/// at any other width the body is left fully opaque (honest — undecoded).
fn walk_ex_body(body: &[u8], tw: usize, spans: &mut Vec<Span>) {
    if tw != 2 {
        spans.push(opaque(body));
        return;
    }
    let mut pending: Vec<u8> = Vec::new();
    let mut p = 0;
    while p < body.len() {
        if let Some((tok, len)) = try_ex_token(body, p) {
            if !pending.is_empty() {
                spans.push(Span::Opaque(std::mem::take(&mut pending)));
            }
            spans.push(Span::Ex(tok));
            p += len;
        } else {
            pending.push(body[p]);
            p += 1;
        }
    }
    if !pending.is_empty() {
        spans.push(Span::Opaque(pending));
    }
}

/// Read a big-endian 2-byte token at `p`, if in range.
fn tok16(b: &[u8], p: usize) -> Option<u16> {
    if p + 2 <= b.len() {
        Some(((b[p] as u16) << 8) | b[p + 1] as u16)
    } else {
        None
    }
}

fn starts_with(b: &[u8], p: usize, pat: &[u8]) -> bool {
    b.len() >= p + pat.len() && &b[p..p + pat.len()] == pat
}

/// Try to decode one typed token at `body[p]` (width 2). Returns the token and
/// the number of bytes it consumes, or `None` if no known token matches here.
fn try_ex_token(body: &[u8], p: usize) -> Option<(ExToken, usize)> {
    let b0 = *body.get(p)?;
    match b0 {
        0x4C => {
            if starts_with(body, p, &LO_MARKER) {
                Some((ExToken::Lo, 3))
            } else if starts_with(body, p, &[0x4C, 0x4B]) {
                Some((ExToken::VoidCallEnd, 2))
            } else {
                None
            }
        }
        0x53 => Some((ExToken::Ss, 1)),
        0x4F => {
            if starts_with(body, p, &[0x4F, 0x01]) {
                let nn = *body.get(p + 2)?;
                Some((ExToken::Stmt(nn), 3))
            } else if starts_with(body, p, &[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]) {
                Some((ExToken::FnTail, 7))
            } else if starts_with(body, p, &[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01]) {
                let nn = *body.get(p + 6)?;
                if body.get(p + 7) == Some(&0x4D) {
                    Some((ExToken::ModuleEnd(nn), 8))
                } else {
                    None
                }
            } else {
                None
            }
        }
        0x26 => {
            let t = tok16(body, p + 1)?;
            Some((ExToken::ResultRef(t), 3))
        }
        0xBD => {
            // BD <3-byte ret type> <6-byte anchor>.
            if p + 10 <= body.len() && starts_with(body, p + 4, &CALL_CALLEE_ANCHOR) {
                let ret = [body[p + 1], body[p + 2], body[p + 3]];
                Some((ExToken::Call(ret), 10))
            } else {
                None
            }
        }
        0xB9 => {
            let t = tok16(body, p + 1)?;
            if starts_with(body, p + 3, &INT_TYPE) {
                Some((ExToken::Load(t), 6))
            } else {
                None
            }
        }
        0x33 => {
            if !starts_with(body, p + 1, &INT_TYPE) {
                return None;
            }
            let vp = p + 4;
            let marker = *body.get(vp)?;
            if marker < 0x80 {
                Some((
                    ExToken::Lit {
                        value: marker as i32,
                        wide: false,
                    },
                    5,
                ))
            } else if marker == 0x80 && vp + 5 <= body.len() {
                let value =
                    i32::from_le_bytes([body[vp + 1], body[vp + 2], body[vp + 3], body[vp + 4]]);
                Some((ExToken::Lit { value, wide: true }, 9))
            } else {
                None
            }
        }
        0x02 => Some((ExToken::Add, 1)),
        0x03 => Some((ExToken::Sub, 1)),
        0x04 => Some((ExToken::Mul, 1)),
        0x55 => {
            if starts_with(body, p + 1, &INT_TYPE) && body.get(p + 4) == Some(&0x4C) {
                Some((ExToken::IntCallEnd, 5))
            } else {
                None
            }
        }
        0x41 => {
            if starts_with(body, p + 1, &INT_TYPE) {
                Some((ExToken::ResultType, 4))
            } else {
                None
            }
        }
        0x3A => {
            let t = tok16(body, p + 1)?;
            Some((ExToken::Assign(t), 3))
        }
        0x54 => {
            if starts_with(body, p, &[0x54, 0x02, 0x29]) {
                let t = tok16(body, p + 3)?;
                Some((ExToken::Return(t), 5))
            } else {
                None
            }
        }
        0x46 => Some((ExToken::Formals, 1)),
        0x2D => {
            let t = tok16(body, p + 1)?;
            Some((ExToken::Formal(t), 3))
        }
        _ => None,
    }
}

/// Model `.gl`: type every `80 <LE32>` whose LE32 is a real `.ex` `4F 1F`
/// offset as a [`Span::GlOffset`] (the body-start offset field); everything else
/// is opaque. The cross-check against `ex_offsets` distinguishes the offset
/// field from unrelated `80`-prefixed data (CALL anchors, wide literals).
fn parse_gl(gl: &[u8], ex_offsets: &BTreeSet<u32>) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut pending: Vec<u8> = Vec::new();
    let mut p = 0;
    while p < gl.len() {
        if gl[p] == 0x80 && p + 5 <= gl.len() {
            let v = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]);
            if ex_offsets.contains(&v) {
                if !pending.is_empty() {
                    spans.push(Span::Opaque(std::mem::take(&mut pending)));
                }
                spans.push(Span::GlOffset(v));
                p += 5;
                continue;
            }
        }
        pending.push(gl[p]);
        p += 1;
    }
    if !pending.is_empty() {
        spans.push(Span::Opaque(pending));
    }
    spans
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- typed token encodings ---------------------------------------------

    #[test]
    fn ex_token_encodings_are_exact() {
        let cases: &[(ExToken, &[u8])] = &[
            (ExToken::Lo, &[0x4C, 0x4F, 0x11]),
            (ExToken::Ss, &[0x53]),
            (ExToken::Stmt(0x0E), &[0x4F, 0x01, 0x0E]),
            (ExToken::ResultRef(0xE609), &[0x26, 0xE6, 0x09]),
            (
                ExToken::Call([0x82, 0x07, 0x03]),
                &[0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00],
            ),
            (ExToken::Load(0xE309), &[0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74]),
            (
                ExToken::Lit {
                    value: 42,
                    wide: false,
                },
                &[0x33, 0x86, 0x41, 0x74, 0x2A],
            ),
            (
                ExToken::Lit {
                    value: 70000,
                    wide: true,
                },
                &[0x33, 0x86, 0x41, 0x74, 0x80, 0x70, 0x11, 0x01, 0x00],
            ),
            // A narrow-representable value forced wide (a P0.6a length pad).
            (
                ExToken::Lit {
                    value: 5,
                    wide: true,
                },
                &[0x33, 0x86, 0x41, 0x74, 0x80, 0x05, 0x00, 0x00, 0x00],
            ),
            (ExToken::Add, &[0x02]),
            (ExToken::Sub, &[0x03]),
            (ExToken::Mul, &[0x04]),
            (ExToken::IntCallEnd, &[0x55, 0x86, 0x41, 0x74, 0x4C]),
            (ExToken::VoidCallEnd, &[0x4C, 0x4B]),
            (ExToken::ResultType, &[0x41, 0x86, 0x41, 0x74]),
            (ExToken::Assign(0xE709), &[0x3A, 0xE7, 0x09]),
            (ExToken::Return(0xE709), &[0x54, 0x02, 0x29, 0xE7, 0x09]),
            (ExToken::FnTail, &[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]),
            (
                ExToken::ModuleEnd(0x02),
                &[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D],
            ),
            (ExToken::Formals, &[0x46]),
            (ExToken::Formal(0xE509), &[0x2D, 0xE5, 0x09]),
        ];
        for (tok, bytes) in cases {
            let mut out = Vec::new();
            tok.encode_into(&mut out);
            assert_eq!(&out, bytes, "encode mismatch for {tok:?}");
            // And the walker decodes those exact bytes back to the token.
            assert_eq!(
                try_ex_token(bytes, 0),
                Some((tok.clone(), bytes.len())),
                "decode mismatch for {tok:?}"
            );
        }
    }

    // A minimal but complete single-function `.ex`: header, one `4F 1F` segment
    // whose body is the real add3 straight-line stream + module end.
    fn synthetic_add3_ex() -> Vec<u8> {
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 12]); // opaque header filler
        let module_start = ex.len();
        // 4F 1F metadata prefix (opaque), then the LO body.
        ex.extend_from_slice(&[
            0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, // fn start marker + descr
            0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // 46 formals c,b,a
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74, // LOAD a
            0xB9, 0xE4, 0x09, 0x86, 0x41, 0x74, // LOAD b
            0x02, // ADD
            0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, // LOAD c
            0x02, // ADD
            0x41, 0x86, 0x41, 0x74, // result-type
            0x3A, 0xE7, 0x09, // ASSIGN
            0x54, 0x02, 0x29, 0xE7, 0x09, // RETURN
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // FnTail
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D, // ModuleEnd
        ]);
        (ex, module_start).0
    }

    #[test]
    fn parse_ex_round_trips_and_types_the_body() {
        let ex = synthetic_add3_ex();
        let spans = parse_ex(&ex);
        // Re-encode == input.
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, ex);
        // The body tokens are decoded (order-checked).
        let toks: Vec<ExToken> = spans
            .iter()
            .filter_map(|s| match s {
                Span::Ex(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            toks,
            vec![
                ExToken::Formals,
                ExToken::Formal(0xE509),
                ExToken::Formal(0xE409),
                ExToken::Formal(0xE309),
                ExToken::Lo,
                ExToken::Ss,
                ExToken::Load(0xE309),
                ExToken::Load(0xE409),
                ExToken::Add,
                ExToken::Load(0xE509),
                ExToken::Add,
                ExToken::ResultType,
                ExToken::Assign(0xE709),
                ExToken::Return(0xE709),
                ExToken::FnTail,
                ExToken::ModuleEnd(0x02),
            ]
        );
        // The `4F 1F …` metadata prefix stays opaque.
        assert!(spans.iter().any(|s| matches!(s, Span::Opaque(b) if b.starts_with(&FN_START))));
    }

    #[test]
    fn parse_gl_types_body_start_offset_by_ex_cross_check() {
        // A .gl fragment: some bytes, the offset field `80 54 0A 00 00` (0x0A54),
        // a decoy `80 01 10 00 00` (0x00100100 — not an .ex offset), more bytes.
        let gl: &[u8] = &[
            0xAA, 0xBB, 0x80, 0x54, 0x0A, 0x00, 0x00, 0xCC, 0x80, 0x01, 0x10, 0x00, 0x00, 0xDD,
        ];
        let mut ex_offsets = BTreeSet::new();
        ex_offsets.insert(0x0A54);
        let spans = parse_gl(gl, &ex_offsets);
        // Exactly one typed offset, value 0x0A54.
        let offs: Vec<u32> = spans
            .iter()
            .filter_map(|s| match s {
                Span::GlOffset(v) => Some(*v),
                _ => None,
            })
            .collect();
        assert_eq!(offs, vec![0x0A54]);
        // Round-trips (decoy left opaque).
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, gl);
    }

    #[test]
    fn parse_gl_all_opaque_when_no_ex_offsets() {
        let gl: &[u8] = &[0x80, 0x54, 0x0A, 0x00, 0x00, 0x01, 0x02];
        let spans = parse_gl(gl, &BTreeSet::new());
        assert_eq!(spans, vec![Span::Opaque(gl.to_vec())]);
    }

    #[test]
    fn full_model_round_trips_and_opaque_files_preserved() {
        let mut bundle = IlBundle::new("_CL_synthetic");
        let ex = synthetic_add3_ex();
        // Place the offset field in .gl matching the .ex 4F 1F offset.
        let mods = ex_fn_start_offsets(&ex);
        let off = *mods.iter().next().unwrap();
        let mut gl = b"?add3@@YAHHHH@Z\x00".to_vec();
        gl.push(0x80);
        gl.extend_from_slice(&off.to_le_bytes());
        bundle.set("ex", ex.clone());
        bundle.set("gl", gl.clone());
        bundle.set("sy", vec![0x03, 0x01, 0xE7, 0x09, 0x00]); // opaque island
        bundle.set("in", vec![0x86, 0x41, 0x74]);
        bundle.set("db", vec![0x01, 0x02, 0x03]);

        let model = IlModel::parse(&bundle).expect("round-trips");
        let back = model.encode();
        assert_eq!(back.base_name, bundle.base_name);
        assert_eq!(back.files, bundle.files);

        // The typed body-start offset surfaces, cross-checked against `.ex`.
        assert_eq!(model.gl_body_start_offsets(), vec![off]);
        // `.sy`/`.in`/`.db` are single opaque spans (K2 backlog).
        for suffix in ["sy", "in", "db"] {
            let fm = model.files.iter().find(|f| f.suffix == suffix).unwrap();
            assert!(matches!(fm.spans.as_slice(), [Span::Opaque(_)]));
        }
    }

    #[test]
    fn out_of_class_bytes_stay_opaque_but_round_trip() {
        // A body with an unmodeled op (comparison `24`) between two loads. The
        // walker must keep `24` opaque, not choke, and still round-trip.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 4]);
        ex.extend_from_slice(&[
            0x4F, 0x1F, 0x00, // fn start (opaque prefix)
            0x4C, 0x4F, 0x11, 0x53, // LO SS
            0xB9, 0xED, 0x09, 0x86, 0x41, 0x74, // LOAD
            0x24, // GT — unmodeled → opaque
            0xB9, 0xEE, 0x09, 0x86, 0x41, 0x74, // LOAD
        ]);
        let spans = parse_ex(&ex);
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, ex);
        // `24` survived as an opaque byte island between the two typed LOADs.
        assert!(spans
            .iter()
            .any(|s| matches!(s, Span::Opaque(b) if b == &[0x24])));
        let loads = spans
            .iter()
            .filter(|s| matches!(s, Span::Ex(ExToken::Load(_))))
            .count();
        assert_eq!(loads, 2);
    }
}
