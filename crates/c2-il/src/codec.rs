//! **K1 — the lossless IL container codec with typed islands.**
//!
//! [`IlModel::parse`] walks the five files of an [`IlBundle`] and produces a
//! structured model whose leaves are either (a) **typed, decoded tokens** for
//! the classes the grammar is known for (the `.ex` operand stream that
//! [`crate::func`] already recognizes, the `.ex` per-function **metadata prefix**
//! — FnHeader preamble, block-start, `53 53`, result-ref, formals — and the
//! `.gl` `80 <LE32>` body-start offset field), or (b) **opaque byte spans** for
//! every not-yet-decoded region (the `.ex` header/index, the FnHeader interior,
//! the rest of `.gl`, and all of `.sy`/`.in`/`.db`). [`IlModel::encode`]
//! serializes the model back to bytes.
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
//! Per the P0.6a probe series, each function's `.gl` record ends `80 <LE32>` where
//! the LE32 is the **`.ex` byte offset of that function's `4F 1F` body-start
//! marker**. That is the one length-bearing field K3 must rewrite when an `.ex`
//! function changes length, so it is typed as [`Span::GlOffset`] (a `u32`), not
//! opaque. K1 only round-trips it unchanged. K2a locates the field **by its
//! record framing** (`80 XX 10 00 00 00 00` precedes it) rather than by value,
//! then gates it fail-closed: the framed offsets must equal the `.ex` `4F 1F`
//! offsets 1:1 and in function order, or none are typed (see [`parse_gl`]).

use std::collections::BTreeSet;

use crate::{detect_token_width, IlBundle};

/// The int type encoding inline in the `.ex` body (`86 41 74`). Mirrors
/// `func::INT_TYPE`; duplicated here to keep [`crate::func`] untouched.
const INT_TYPE: [u8; 3] = [0x86, 0x41, 0x74];

/// The **float** type encoding inline in the `.ex` body (`86 45 40`) — the
/// float analog of [`INT_TYPE`], per the reference decoder's type table
/// (`dc3-decomp/msvc-src/tools/il_parser.py` `KNOWN_TYPES`: `86 45 40` = float).
/// This is the "float type-annotation" the float-leaf codec widening decodes:
/// it appears in a float LOAD (`B9 <tok> 86 45 40`), the CAST/STORE that
/// materialize a float temp, and the float result-type annotation
/// (`41 86 45 40`). Verified against a live 16.00.11886.00 capture of a
/// `Box::Volume`-class float leaf (see `VOLF_SEGMENT`).
const FLOAT_TYPE: [u8; 3] = [0x86, 0x45, 0x40];

/// The 2-byte **pointer** type prefix (`86 43`) — a pointer type is
/// `86 43 XX XX` (4 bytes; the two low bytes name the pointee), per the
/// reference decoder (`try_parse_type`: prefix `86` with `43` → pointer). It
/// leads a pointer LOAD (`B9 <tok> 86 43 XX XX`) and the MEMBER_PTR op.
const PTR_TYPE_PREFIX: [u8; 2] = [0x86, 0x43];

/// The class/struct-pointer type prefix (`A6`) — an `A6 XX XX XX` 4-byte type
/// (per `try_parse_type`: prefix `A6` → class-pointer). It types the DEREF
/// (`30 A6 XX XX XX`) that loads a struct member through the member pointer.
const CLASS_PTR_PREFIX: u8 = 0xA6;

/// The 6 bytes this codec still writes as a CALL token's tail (`00 80 01 10 00 00`).
///
/// **Not an anchor, and no longer mirrored anywhere.** `func.rs` used to hardcode
/// the same constant and now decodes the field properly: the trailing value is a
/// per-TU function-type id, keyed on the signature, so `0x1001` is merely the first
/// one a single-callee TU creates. This codec keeps the literal because it has not
/// been ported to the variable-width reads yet (ROADMAP item 14).
///
/// That is safe rather than merely tolerated: the codec is round-trip gated, so a
/// CALL token whose tail differs simply fails to match here and falls through to an
/// opaque span that re-encodes byte-for-byte. It costs the IL-mutation search
/// coverage, never correctness.
const CALL_CALLEE_ANCHOR: [u8; 6] = [0x00, 0x80, 0x01, 0x10, 0x00, 0x00];

/// The `.ex` per-function start marker (`4F 1F`). Mirrors `func::FN_START`.
const FN_START: [u8; 2] = [0x4F, 0x1F];

/// The one-byte `4C` 'LO' body-start token — the point from which the `.ex`
/// operand stream of a function is a typed token sequence.
///
/// **`4F 11` is a separate, optional record beside it** ([`LO_RECORD`]), not part
/// of the token: a `??__E`/`??__F` dynamic-initializer thunk opens `4C 53` where
/// a source function opens `4C 4F 11 53` (ROADMAP §10.12). The composed form is
/// `func::bundle::LO_MARKER`; the locator that handles both is
/// `func::bundle::body_start`, and this module calls THAT rather than keeping a
/// second copy of the rule — a private re-derivation of a rule the crate already
/// owns is not a shortcut, it is a second rule that agrees until it matters
/// (§10.14).
const LO: u8 = 0x4C;

/// The optional `4F 11` record between [`LO`] and the body's first `53`.
const LO_RECORD: [u8; 2] = [0x4F, 0x11];

/// The `4F 02 20 00` per-function block-start marker prefix. In the metadata
/// prefix it is followed by `4F 01 NN` (block index) then `53 53`; at the end of
/// the last function it is `4F 02 20 00 4F 01 NN 4D` (the module end). Mirrors
/// the leading bytes of `func`'s module-end sequence.
const BLOCK_START: [u8; 4] = [0x4F, 0x02, 0x20, 0x00];

/// A single decoded `.ex` operand-stream token. Every variant re-encodes to
/// *exactly* the bytes it was parsed from (see [`ExToken::encode_into`]), so a
/// span list of these plus [`Span::Opaque`] runs round-trips byte-identically.
///
/// The token classes mirror the grammar in [`crate::func`] (`parse_segment`)
/// and `docs/IL_BUNDLE_MVP.md`. All tokens are decoded at token width 2 (every
/// captured bundle); a stream at another width is left fully opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExToken {
    /// `4C` — 'LO' load-operands body-start token.
    ///
    /// One byte. The `4F 11` that follows it in a source function's body is
    /// [`ExToken::LoRecord`], a separate optional record — see [`LO`].
    Lo,
    /// `4F 11` — the optional record between [`ExToken::Lo`] and the body's first
    /// `53`. Present in every source function measured, absent in every
    /// `??__E`/`??__F` thunk measured (ROADMAP §10.12).
    LoRecord,
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
    /// The per-function **metadata-header preamble**: the fixed bytes from the
    /// `4F 1F` function-start marker up to (not including) the `4F 02 20 00`
    /// block-start marker — the `4F 1F`/`4F 20` descriptors, the length-prefixed
    /// `4F 33 <len>` metadata record, the `42 45` ('BE') block-entry, and the
    /// trailing `0F`. Byte-identical across every captured function (return type,
    /// formal count, and body shape do not alter it), so it is recognized as a
    /// bounded typed island — its start (`4F 1F`) and end (the block-start) are
    /// structural; its interior sub-records are captured verbatim, not yet
    /// individually field-typed (a further K2 shrink). K3 reads it to know where
    /// the fixed header ends and the length-relevant body structure begins.
    FnHeader(Vec<u8>),
    /// `4F 02 20 00 4F 01 NN` — the per-function **block-start marker** (`NN` is
    /// the statement/block index, distinct per function). Structurally the same
    /// `4F 02 20 00 4F 01 NN` sequence [`ExToken::ModuleEnd`] carries, minus the
    /// trailing `4D`; located here in the metadata prefix, before `53 53`.
    BlockStart(u8),

    // ---- float-leaf vocabulary (float arithmetic + struct-member loads) ----
    //
    // The `Box::Volume`-class float leaf (`float x=a->x-b->x; … return x*y*z;`)
    // parses to a float-arith stream over struct-member loads. Its operand forms
    // are the float analogs of the int LOAD/result-type plus the member-access
    // idiom `LOAD ptr ; LIT offset ; MEMBER_PTR ; DEREF`. These are the tokens
    // that were interleaved-opaque before the widening — decoded here so the body
    // is a *contiguous* typed run (K3a-editable). Byte evidence: a live
    // 16.00.11886.00 `/Bd /d2nop /Ox /GS- /c` capture of `float volf(const V*a,
    // const V*b){ float x=a->x-b->x; float y=…; float z=…; return x*y*z; }` — see
    // the `VOLF_SEGMENT` test fixture. (The int-typed `Lit`, `Sub`, `Mul` in the
    // body were already typed — `Sub`/`Mul` are type-agnostic single bytes — so
    // no float `Sub`/`Mul` variant is needed; they surface as the existing
    // [`ExToken::Sub`]/[`ExToken::Mul`] once the body is contiguous.)

    /// `B9 <tok> 86 45 40` — LOAD a **float** operand (the float analog of
    /// [`ExToken::Load`], which is int `B9 <tok> 86 41 74`).
    FloatLoad(u16),
    /// `B9 <tok> 86 43 XX XX` — LOAD a **pointer** operand (a `this`/argument
    /// pointer feeding a member access). `86 43` is the pointer-type prefix; the
    /// two low bytes name the pointee and are preserved for byte-exact re-encode.
    PtrLoad { tok: u16, ty: [u8; 2] },
    /// `27 86 43 XX XX` — MEMBER_PTR: pointer + offset literal → a typed member
    /// pointer (`a` + `offsetof(x)`). The two low type bytes are preserved.
    MemberPtr([u8; 2]),
    /// `30 A6 XX XX XX` — DEREF: load-indirect the float member through the
    /// member pointer (`A6` = class/struct-pointer type; 3 payload bytes).
    Deref([u8; 3]),
    /// `2C 86 45 40 00` — CAST to float: materialize the float sub-expression
    /// result (trailing `00` per the reference `CAST: 2C type 00`).
    CastFloat,
    /// `32 86 45 40 4B` — STORE the float temp (trailing `4B` end marker, per the
    /// reference `STORE: 32 type 4B`).
    StoreFloat,
    /// `41 86 45 40` — **float** result-type annotation (the float analog of
    /// [`ExToken::ResultType`], which is int `41 86 41 74`).
    ResultTypeFloat,
}

impl ExToken {
    /// Append this token's exact byte encoding to `out`.
    fn encode_into(&self, out: &mut Vec<u8>) {
        let tok = |out: &mut Vec<u8>, t: u16| {
            out.push((t >> 8) as u8);
            out.push((t & 0xFF) as u8);
        };
        match *self {
            ExToken::Lo => out.push(LO),
            ExToken::LoRecord => out.extend_from_slice(&LO_RECORD),
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
            ExToken::FnHeader(ref b) => out.extend_from_slice(b),
            ExToken::BlockStart(nn) => {
                out.extend_from_slice(&BLOCK_START);
                out.extend_from_slice(&[0x4F, 0x01, nn]);
            }
            ExToken::FloatLoad(t) => {
                out.push(0xB9);
                tok(out, t);
                out.extend_from_slice(&FLOAT_TYPE);
            }
            ExToken::PtrLoad { tok: t, ty } => {
                out.push(0xB9);
                tok(out, t);
                out.extend_from_slice(&PTR_TYPE_PREFIX);
                out.extend_from_slice(&ty);
            }
            ExToken::MemberPtr(ty) => {
                out.push(0x27);
                out.extend_from_slice(&PTR_TYPE_PREFIX);
                out.extend_from_slice(&ty);
            }
            ExToken::Deref(ty) => {
                out.push(0x30);
                out.push(CLASS_PTR_PREFIX);
                out.extend_from_slice(&ty);
            }
            ExToken::CastFloat => {
                out.push(0x2C);
                out.extend_from_slice(&FLOAT_TYPE);
                out.push(0x00);
            }
            ExToken::StoreFloat => {
                out.push(0x32);
                out.extend_from_slice(&FLOAT_TYPE);
                out.push(0x4B);
            }
            ExToken::ResultTypeFloat => {
                out.push(0x41);
                out.extend_from_slice(&FLOAT_TYPE);
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
        // The `.ex` function-start (`4F 1F`) offsets, in file order — the
        // fail-closed cross-check that confirms the structurally-framed `.gl`
        // body-start offset fields are 1:1 and in function order with `.ex`.
        let ex = bundle.get("ex").unwrap_or(&[]);
        let ex_offsets: Vec<u32> = ex_fn_start_offsets(ex).into_iter().collect();

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

    /// The number of `.ex` functions — the count of `4F 1F` function-start
    /// markers. K3's invariant: the typed `.gl` body-start offsets are 1:1 with
    /// these (`gl_body_start_offsets().len() == ex_function_count()`), enforced
    /// fail-closed by [`parse_gl`]'s structural cross-check.
    pub fn ex_function_count(&self) -> usize {
        self.files
            .iter()
            .find(|f| f.suffix == "ex")
            .map(|f| ex_fn_start_offsets(&f.encode()).len())
            .unwrap_or(0)
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

// ===========================================================================
// K3a — the length-consistent IL edit primitive.
// ===========================================================================
//
// K1/K2a made [`IlModel`] a lossless *read* codec. K3a turns it into a verified
// *edit* substrate: a statement-grain mutation of a target function's `.ex`
// operand stream that changes its byte length and re-emits the one length-bearing
// field the change obligates — the per-function `.gl` body-start offset column
// (`80 <LE32>` = the `.ex` offset of each function's `4F 1F` marker).
//
// Proven live in the P0.6a probe series: `.ex` is length-plastic — grown/shrunk IL
// is re-optimized by c2 byte-exact to a native capture of the equivalent source,
// under ONE obligation: on any `.ex` length change, every function AFTER the edit
// point has its `.gl` body-start offset bumped by the byte delta (the edited and
// preceding functions are unchanged; a single-fn / last-fn edit needs no `.gl`
// patch at all — P0.6a's zero-bookkeeping regime). Skip the re-emit on a non-last
// edit and c2 seeks a stale offset and SIGSEGVs (P0.6a experiment C).
//
// Scope (K3a): statement-grain length edits *within one function's body* —
//   * varint literal widen/narrow (same value, pure length change; P0.6a A/B),
//   * operand-stream token insert/delete (an arithmetic term added/removed;
//     P0.6a E `(a+5)+5`, F `a+b+c`→`a+b`).
// Whole-function add/remove is OUT of scope: it needs coordinated `.gl` record
// and `.sy` record framing (K3b), and violating the `.gl`/`.ex` function-set can
// make c2 *hang* (P0.6a G). Every edit here is **fail-closed**: an edit that would
// change the function set, or a non-last edit whose `.gl` offset column is not
// modeled (so the obligation cannot be discharged), refuses with a typed
// [`EditError`] and leaves the model untouched — it never emits a
// hang/crash-inducing bundle.

use std::ops::Range;

/// The outcome of one successful length edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditReport {
    /// The edited function's index (0-based, in `.ex` function order).
    pub fn_index: usize,
    /// Signed change in the edited function's segment byte length (and in the
    /// whole `.ex` length): `new_len - old_len`. Downstream `.gl` offsets shift
    /// by exactly this.
    pub byte_delta: i64,
    /// The re-emitted `.gl` body-start offset column after the edit, in function
    /// order. Empty iff `.gl` carried no typed offsets — legal only for a
    /// single/last-function edit, which needs no re-emit.
    pub gl_offsets: Vec<u32>,
}

/// A fail-closed edit rejection. The model is never mutated when one is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditError {
    /// The bundle has no `.ex` file to edit.
    NoExFile,
    /// `fn_index` is out of range (`count` functions present).
    NoSuchFunction { index: usize, count: usize },
    /// The target function's `.ex` spans are not a clean leading-opaque + typed
    /// token run (an opaque region sits *between* typed tokens), so the edit
    /// cannot be token-addressed. Out of K3a scope; refuse rather than guess.
    OpaqueFunctionBody { fn_index: usize },
    /// A splice range `[start, end)` is outside the function's token sequence.
    TokenRange {
        fn_index: usize,
        start: usize,
        end: usize,
        ntokens: usize,
    },
    /// A widen/narrow target token is not an [`ExToken::Lit`].
    NotALiteral { fn_index: usize, token_index: usize },
    /// A narrow (`wide → false`) target's value does not fit the 1-byte varint
    /// form (`0..=0x7F`), so narrowing would change its value.
    ValueNotNarrowable { value: i32 },
    /// The edit changed the `.ex` function count (a `4F 1F` marker created or
    /// destroyed) — whole-function add/remove, which is K3b, not K3a.
    FunctionSetChanged { before: usize, after: usize },
    /// The edit moved the edited-or-preceding function's start offset — a
    /// structural surprise (the edit was supposed to stay within one body); the
    /// model is left untouched.
    PrecedingOffsetShifted { fn_index: usize },
    /// A downstream function's start did not shift by exactly the byte delta — a
    /// marker moved unexpectedly (e.g. the replacement encoded a stray `4F 1F`).
    DownstreamOffsetDesync { fn_index: usize },
    /// A non-last-function length edit, but the `.gl` body-start offset column is
    /// not modeled (all opaque), so the mandatory re-emit cannot be discharged.
    /// Editing this function would strand a stale `.gl` offset → SIGSEGV.
    GlOffsetsNotTyped { fn_index: usize },
    /// The `.gl` typed offset count disagrees with the `.ex` function count.
    GlOffsetCountMismatch { gl: usize, ex: usize },
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::NoExFile => write!(f, "bundle has no .ex file to edit"),
            EditError::NoSuchFunction { index, count } => {
                write!(f, "no such function {index} (.ex has {count})")
            }
            EditError::OpaqueFunctionBody { fn_index } => write!(
                f,
                "function {fn_index} has opaque bytes between typed tokens — not token-addressable (out of K3a scope)"
            ),
            EditError::TokenRange {
                fn_index,
                start,
                end,
                ntokens,
            } => write!(
                f,
                "splice range {start}..{end} out of function {fn_index}'s {ntokens} tokens"
            ),
            EditError::NotALiteral {
                fn_index,
                token_index,
            } => write!(
                f,
                "token {token_index} of function {fn_index} is not an int literal"
            ),
            EditError::ValueNotNarrowable { value } => write!(
                f,
                "value {value} does not fit the 1-byte varint form (0..=127); narrowing would change it"
            ),
            EditError::FunctionSetChanged { before, after } => write!(
                f,
                "edit changed the .ex function count {before} -> {after} (whole-function add/remove is K3b, not K3a)"
            ),
            EditError::PrecedingOffsetShifted { fn_index } => write!(
                f,
                "edit moved the edited/preceding start offset for function {fn_index}"
            ),
            EditError::DownstreamOffsetDesync { fn_index } => write!(
                f,
                "a function after {fn_index} did not shift by the byte delta (stray marker?)"
            ),
            EditError::GlOffsetsNotTyped { fn_index } => write!(
                f,
                "non-last edit of function {fn_index} but .gl offset column is not modeled — cannot re-emit (would strand a stale offset -> SIGSEGV)"
            ),
            EditError::GlOffsetCountMismatch { gl, ex } => {
                write!(f, ".gl typed offsets ({gl}) != .ex functions ({ex})")
            }
        }
    }
}

impl std::error::Error for EditError {}

impl IlModel {
    /// Index of the `.ex` file model, if present.
    fn ex_file_index(&self) -> Option<usize> {
        self.files.iter().position(|f| f.suffix == "ex")
    }

    /// The `.ex` `4F 1F` function-start byte offsets, in file order.
    fn ex_start_offsets_vec(&self) -> Vec<u32> {
        self.ex_file_index()
            .map(|i| {
                ex_fn_start_offsets(&self.files[i].encode())
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The typed `.ex` tokens of function `fn_index`, in stream order (the tokens
    /// a caller addresses when locating a splice / widen point). Errors if the
    /// function is out of range or is not token-addressable (opaque interior).
    pub fn function_tokens(&self, fn_index: usize) -> Result<Vec<ExToken>, EditError> {
        let (_, tokens, _, _) = self.function_parts(fn_index)?;
        Ok(tokens)
    }

    /// Decompose function `fn_index`'s `.ex` spans into
    /// `(span_range, tokens, leading_opaque, trailing_opaque)`:
    /// * `span_range` — the `[start, end)` span indices of the function,
    /// * `tokens` — its contiguous run of typed [`ExToken`]s,
    /// * `leading_opaque` / `trailing_opaque` — the opaque spans before / after
    ///   that run (a captured function has none before and at most a module-tail
    ///   after; a hand-built one may have an opaque descriptor prefix).
    ///
    /// Fails closed if a function is out of range or has an opaque span *between*
    /// two typed tokens (not token-addressable).
    fn function_parts(
        &self,
        fn_index: usize,
    ) -> Result<(Range<usize>, Vec<ExToken>, Vec<Span>, Vec<Span>), EditError> {
        let exi = self.ex_file_index().ok_or(EditError::NoExFile)?;
        let spans = &self.files[exi].spans;
        let ranges = ex_function_span_ranges(spans);
        let count = ranges.len();
        let range = ranges
            .get(fn_index)
            .cloned()
            .ok_or(EditError::NoSuchFunction {
                index: fn_index,
                count,
            })?;
        let fn_spans = &spans[range.clone()];
        // Split into leading opaque, the typed-token run, trailing opaque.
        let first_ex = fn_spans.iter().position(|s| matches!(s, Span::Ex(_)));
        let Some(first_ex) = first_ex else {
            // No typed tokens at all — nothing to address.
            return Err(EditError::OpaqueFunctionBody { fn_index });
        };
        let last_ex = fn_spans
            .iter()
            .rposition(|s| matches!(s, Span::Ex(_)))
            .unwrap();
        // The [first_ex, last_ex] window must be *all* typed (no opaque between).
        let mut tokens = Vec::new();
        for s in &fn_spans[first_ex..=last_ex] {
            match s {
                Span::Ex(t) => tokens.push(t.clone()),
                _ => return Err(EditError::OpaqueFunctionBody { fn_index }),
            }
        }
        let leading = fn_spans[..first_ex].to_vec();
        let trailing = fn_spans[last_ex + 1..].to_vec();
        Ok((range, tokens, leading, trailing))
    }

    /// **The K3a length-edit primitive.** Replace the tokens `[range]` of function
    /// `fn_index`'s `.ex` operand stream with `replacement`, recompute the
    /// function's segment length, and re-emit the `.gl` body-start offset column
    /// from the new `4F 1F` marker positions (functions ≤ `fn_index` unchanged;
    /// functions after shift by the byte delta). Insert = empty `range`; delete =
    /// empty `replacement`; substitute = both non-empty.
    ///
    /// Fail-closed: if the edit changes the `.ex` function set, or a downstream
    /// function's start does not shift by exactly the delta, or a non-last edit
    /// cannot re-emit its `.gl` offsets, the model is left **untouched** and a
    /// typed [`EditError`] is returned — never a hang/crash-inducing bundle.
    pub fn splice_function_tokens(
        &mut self,
        fn_index: usize,
        range: Range<usize>,
        replacement: Vec<ExToken>,
    ) -> Result<EditReport, EditError> {
        let exi = self.ex_file_index().ok_or(EditError::NoExFile)?;
        let old_offsets = self.ex_start_offsets_vec();
        let old_count = old_offsets.len();
        let old_ex_len = self.files[exi].encode().len();

        let (span_range, tokens, leading, trailing) = self.function_parts(fn_index)?;
        if range.start > range.end || range.end > tokens.len() {
            return Err(EditError::TokenRange {
                fn_index,
                start: range.start,
                end: range.end,
                ntokens: tokens.len(),
            });
        }

        // Build the edited token sequence for this function.
        let mut new_tokens: Vec<ExToken> = Vec::with_capacity(
            tokens.len() - (range.end - range.start) + replacement.len(),
        );
        new_tokens.extend_from_slice(&tokens[..range.start]);
        new_tokens.extend(replacement.iter().cloned());
        new_tokens.extend_from_slice(&tokens[range.end..]);

        // Reassemble the function's spans: leading opaque, typed run, trailing.
        let mut new_fn_spans: Vec<Span> = Vec::with_capacity(new_tokens.len() + 2);
        new_fn_spans.extend(leading.iter().cloned());
        new_fn_spans.extend(new_tokens.into_iter().map(Span::Ex));
        new_fn_spans.extend(trailing.iter().cloned());

        // Splice the new function spans into a CANDIDATE `.ex` span list (no
        // mutation of `self` yet — we validate the candidate first).
        let mut cand_ex_spans = self.files[exi].spans.clone();
        cand_ex_spans.splice(span_range.clone(), new_fn_spans);

        // Re-derive the new `.ex` function-start offsets from the candidate bytes.
        let cand_ex_bytes = encode_spans(&cand_ex_spans);
        let new_offsets: Vec<u32> = ex_fn_start_offsets(&cand_ex_bytes).into_iter().collect();
        let delta = cand_ex_bytes.len() as i64 - old_ex_len as i64;

        // Fail-closed structural checks.
        if new_offsets.len() != old_count {
            return Err(EditError::FunctionSetChanged {
                before: old_count,
                after: new_offsets.len(),
            });
        }
        for j in 0..=fn_index {
            if new_offsets[j] != old_offsets[j] {
                return Err(EditError::PrecedingOffsetShifted { fn_index });
            }
        }
        for j in (fn_index + 1)..old_count {
            if new_offsets[j] as i64 != old_offsets[j] as i64 + delta {
                return Err(EditError::DownstreamOffsetDesync { fn_index });
            }
        }

        // Re-emit the `.gl` offset column (the K3a obligation). Build a candidate
        // `.gl` span list; commit both files together only if everything holds.
        let gli = self.files.iter().position(|f| f.suffix == "gl");
        let has_downstream = fn_index + 1 < old_count;
        let mut cand_gl: Option<(usize, Vec<Span>)> = None;
        if let Some(gli) = gli {
            let typed = self.files[gli]
                .spans
                .iter()
                .filter(|s| matches!(s, Span::GlOffset(_)))
                .count();
            if typed > 0 {
                if typed != old_count {
                    return Err(EditError::GlOffsetCountMismatch {
                        gl: typed,
                        ex: old_count,
                    });
                }
                // Rewrite each typed offset from the new `4F 1F` positions.
                let mut spans = self.files[gli].spans.clone();
                let mut k = 0;
                for s in spans.iter_mut() {
                    if let Span::GlOffset(v) = s {
                        *v = new_offsets[k];
                        k += 1;
                    }
                }
                cand_gl = Some((gli, spans));
            } else if has_downstream {
                return Err(EditError::GlOffsetsNotTyped { fn_index });
            }
        } else if has_downstream {
            return Err(EditError::GlOffsetsNotTyped { fn_index });
        }

        // Commit.
        self.files[exi].spans = cand_ex_spans;
        let gl_offsets = if let Some((gli, spans)) = cand_gl {
            self.files[gli].spans = spans;
            new_offsets.clone()
        } else {
            Vec::new()
        };

        Ok(EditReport {
            fn_index,
            byte_delta: delta,
            gl_offsets,
        })
    }

    /// Widen (`wide = true`) or narrow (`wide = false`) the varint form of the
    /// int literal at `token_index` of function `fn_index` — same value, pure
    /// length change (P0.6a A/B). Built on [`IlModel::splice_function_tokens`], so
    /// it re-emits the `.gl` offset column and is fail-closed identically.
    pub fn set_literal_wide(
        &mut self,
        fn_index: usize,
        token_index: usize,
        wide: bool,
    ) -> Result<EditReport, EditError> {
        let tokens = self.function_tokens(fn_index)?;
        let tok = tokens.get(token_index).ok_or(EditError::TokenRange {
            fn_index,
            start: token_index,
            end: token_index + 1,
            ntokens: tokens.len(),
        })?;
        let ExToken::Lit { value, wide: _ } = *tok else {
            return Err(EditError::NotALiteral {
                fn_index,
                token_index,
            });
        };
        if !wide && !(0..=0x7F).contains(&value) {
            return Err(EditError::ValueNotNarrowable { value });
        }
        self.splice_function_tokens(
            fn_index,
            token_index..token_index + 1,
            vec![ExToken::Lit { value, wide }],
        )
    }
}

/// Concatenate a span list to bytes.
fn encode_spans(spans: &[Span]) -> Vec<u8> {
    let mut out = Vec::new();
    for s in spans {
        s.encode_into(&mut out);
    }
    out
}

/// Partition an `.ex` file's spans into per-function span-index ranges. A
/// function begins at each span boundary whose cumulative byte offset is a
/// `4F 1F` marker offset. Function boundaries always align with span boundaries
/// (`parse_ex` splits every segment at a `4F 1F`), so each range is a clean run
/// of that function's spans; the leading header/index spans (offset 0 up to the
/// first `4F 1F`) are excluded.
fn ex_function_span_ranges(spans: &[Span]) -> Vec<Range<usize>> {
    // Cumulative byte offset at each span boundary (offs[i] = bytes before span i).
    let mut offs = Vec::with_capacity(spans.len() + 1);
    let mut acc = 0usize;
    offs.push(0usize);
    for s in spans {
        let mut tmp = Vec::new();
        s.encode_into(&mut tmp);
        acc += tmp.len();
        offs.push(acc);
    }
    let bytes = encode_spans(spans);
    let starts = ex_fn_start_offsets(&bytes);
    // The span indices at which a function starts (offset aligns to a boundary).
    let start_spans: Vec<usize> = (0..spans.len())
        .filter(|&i| starts.contains(&(offs[i] as u32)))
        .collect();
    let mut ranges = Vec::with_capacity(start_spans.len());
    for (k, &si) in start_spans.iter().enumerate() {
        let end = start_spans.get(k + 1).copied().unwrap_or(spans.len());
        ranges.push(si..end);
    }
    ranges
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
/// function a **typed metadata prefix** (`4F 1F` … up to the `4C 4F 11` 'LO'
/// marker) followed by a typed walk of the body from 'LO' to the segment end.
/// Regions the walkers do not recognize become opaque bytes, so the whole file
/// round-trips regardless of what is decoded.
fn parse_ex(ex: &[u8]) -> Vec<Span> {
    let mut spans = Vec::new();
    let starts: Vec<usize> = ex_fn_start_offsets(ex).iter().map(|&o| o as usize).collect();
    if starts.is_empty() {
        return vec![opaque(ex)];
    }
    // Opaque header/index region before the first function (K2 backlog).
    if starts[0] > 0 {
        spans.push(opaque(&ex[..starts[0]]));
    }
    let tw = detect_token_width(ex);
    for (k, &s) in starts.iter().enumerate() {
        let e = starts.get(k + 1).copied().unwrap_or(ex.len());
        let seg = &ex[s..e];
        // ONE locator for both forms of the body start, and it lives in
        // `func::bundle` — the crate's own rule, called rather than re-derived.
        match crate::func::body_start(seg) {
            Some(lo) => {
                // The metadata prefix `seg[..lo]` (`4F 1F` … 'LO') decodes into
                // the FnHeader preamble, the `4F 02 20 00 4F 01 NN` block-start,
                // `53 53`, the `26 <tok>` result-ref, and the `46 (2D <tok>)*`
                // formals; anything unrecognized coalesces into opaque bytes.
                walk_ex_prefix(&seg[..lo], tw, &mut spans);
                walk_ex_body(&seg[lo..], tw, &mut spans);
            }
            // No body marker in this segment — keep it wholly opaque.
            None => spans.push(opaque(seg)),
        }
    }
    spans
}

/// Greedy typed-token walk of an `.ex` function **metadata prefix** (`4F 1F` up
/// to the 'LO' marker). Recognizes the FnHeader preamble (a bounded island from
/// `4F 1F` to the block-start), the `4F 02 20 00 4F 01 NN` block-start, `53`
/// statement bytes, the `26 <tok>` result-ref, and the `46 (2D <tok>)*` formal
/// list. Unrecognized bytes coalesce into opaque runs; token reads assume width
/// 2 (at any other width the prefix is left fully opaque — honest, undecoded).
fn walk_ex_prefix(prefix: &[u8], tw: usize, spans: &mut Vec<Span>) {
    if tw != 2 {
        spans.push(opaque(prefix));
        return;
    }
    let mut pending: Vec<u8> = Vec::new();
    let mut p = 0;
    while p < prefix.len() {
        if let Some((tok, len)) = try_prefix_token(prefix, p) {
            if !pending.is_empty() {
                spans.push(Span::Opaque(std::mem::take(&mut pending)));
            }
            spans.push(Span::Ex(tok));
            p += len;
        } else {
            pending.push(prefix[p]);
            p += 1;
        }
    }
    if !pending.is_empty() {
        spans.push(Span::Opaque(pending));
    }
}

/// Try to decode one metadata-prefix token at `prefix[p]` (width 2). Only the
/// prefix token classes are recognized here (never body ops), so a stray header
/// byte can never be mis-read as a LOAD/LIT/ADD. Returns the token and the bytes
/// it consumes, or `None`.
fn try_prefix_token(prefix: &[u8], p: usize) -> Option<(ExToken, usize)> {
    // The FnHeader preamble is only ever the first token of a segment prefix:
    // it runs from the leading `4F 1F` up to the `4F 02 20 00` block-start. If
    // there is no block-start ahead (e.g. a minimal hand-built segment), it is
    // not recognized and the `4F 1F` bytes fall through to an opaque run.
    if p == 0 && starts_with(prefix, 0, &FN_START) {
        if let Some(bs) = find_subslice(prefix, &BLOCK_START) {
            if bs > 0 {
                return Some((ExToken::FnHeader(prefix[..bs].to_vec()), bs));
            }
        }
    }
    match *prefix.get(p)? {
        0x4F => {
            // Block-start: 4F 02 20 00 4F 01 NN (no trailing 4D — that is the
            // module end, which lives in the body, not the prefix).
            if starts_with(prefix, p, &BLOCK_START) && starts_with(prefix, p + 4, &[0x4F, 0x01]) {
                let nn = *prefix.get(p + 6)?;
                Some((ExToken::BlockStart(nn), 7))
            } else {
                None
            }
        }
        0x53 => Some((ExToken::Ss, 1)),
        0x26 => {
            let t = tok16(prefix, p + 1)?;
            Some((ExToken::ResultRef(t), 3))
        }
        0x46 => Some((ExToken::Formals, 1)),
        0x2D => {
            let t = tok16(prefix, p + 1)?;
            Some((ExToken::Formal(t), 3))
        }
        _ => None,
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
            // `4C 4B` is VoidCallEnd and is checked FIRST, unchanged: the
            // body-start `4C` is followed by `4F 11` or by `53`, never by `4B`,
            // so no stream that decoded as VoidCallEnd before decodes as `Lo`
            // now. Everything else beginning `4C` is the one-byte token.
            if starts_with(body, p, &[0x4C, 0x4B]) {
                Some((ExToken::VoidCallEnd, 2))
            } else {
                Some((ExToken::Lo, 1))
            }
        }
        0x53 => Some((ExToken::Ss, 1)),
        0x4F => {
            if starts_with(body, p, &LO_RECORD) {
                Some((ExToken::LoRecord, 2))
            } else if starts_with(body, p, &[0x4F, 0x01]) {
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
            } else if starts_with(body, p + 3, &FLOAT_TYPE) {
                Some((ExToken::FloatLoad(t), 6))
            } else if starts_with(body, p + 3, &PTR_TYPE_PREFIX) {
                // Pointer LOAD: `B9 <tok> 86 43 XX XX` (4-byte pointer type).
                let ty = [*body.get(p + 5)?, *body.get(p + 6)?];
                Some((ExToken::PtrLoad { tok: t, ty }, 7))
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
            } else if starts_with(body, p + 1, &FLOAT_TYPE) {
                Some((ExToken::ResultTypeFloat, 4))
            } else {
                None
            }
        }
        0x27 => {
            // MEMBER_PTR: `27 86 43 XX XX` (pointer type; pointer + offset).
            if starts_with(body, p + 1, &PTR_TYPE_PREFIX) {
                let ty = [*body.get(p + 3)?, *body.get(p + 4)?];
                Some((ExToken::MemberPtr(ty), 5))
            } else {
                None
            }
        }
        0x30 => {
            // DEREF: `30 A6 XX XX XX` (class/struct-pointer type; load-indirect).
            if body.get(p + 1) == Some(&CLASS_PTR_PREFIX) {
                let ty = [*body.get(p + 2)?, *body.get(p + 3)?, *body.get(p + 4)?];
                Some((ExToken::Deref(ty), 5))
            } else {
                None
            }
        }
        0x2C => {
            // CAST to float: `2C 86 45 40 00`.
            if starts_with(body, p + 1, &FLOAT_TYPE) && body.get(p + 4) == Some(&0x00) {
                Some((ExToken::CastFloat, 5))
            } else {
                None
            }
        }
        0x32 => {
            // STORE float temp: `32 86 45 40 4B`.
            if starts_with(body, p + 1, &FLOAT_TYPE) && body.get(p + 4) == Some(&0x4B) {
                Some((ExToken::StoreFloat, 5))
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

/// True iff a `.gl` body-start offset field's `80 <LE32>` at `o` sits in its
/// record framing: `80 XX 10 00 00 00 00` immediately precedes it (a `80`-field
/// with a `10 00 00` body, then two zero bytes). Verified byte-exact ahead of
/// every offset field across the full fixture spread — this locates the field
/// **by position within the record**, not by what its value happens to be.
pub(crate) fn gl_offset_framed(gl: &[u8], o: usize) -> bool {
    o >= 7
        && gl[o] == 0x80
        && gl[o - 7] == 0x80
        && gl[o - 5] == 0x10
        && gl[o - 4] == 0x00
        && gl[o - 3] == 0x00
        && gl[o - 2] == 0x00
        && gl[o - 1] == 0x00
}

/// Model `.gl`: type each function's `80 <LE32>` body-start offset field as a
/// [`Span::GlOffset`]; everything else is opaque.
///
/// **Structural identification (K2a).** The offset fields are located by their
/// record framing (see [`gl_offset_framed`]) — a position-based decode, not the
/// K1 value-membership heuristic — and then gated fail-closed against `.ex`: the
/// framed offsets, in `.gl` order, must equal the `.ex` `4F 1F` function-start
/// offsets exactly (same values, same order, 1:1). If they do not (a coincidental
/// frame, or a record we failed to frame), NONE are typed — K3 is never handed a
/// false rewrite site. `ex_offsets_ordered` is the `.ex` `4F 1F` offsets in file
/// order.
///
/// Residual risk: a coincidental `80 <LE32>` carrying the exact framing AND whose
/// value equals the matching function's offset AND appearing in function order
/// would still be accepted. Full per-record `.gl` framing (K2 backlog) would
/// remove even that; the framing + order + 1:1 gate makes it vanishingly small.
fn parse_gl(gl: &[u8], ex_offsets_ordered: &[u32]) -> Vec<Span> {
    // Locate offset fields structurally, in file order.
    let mut framed: Vec<(usize, u32)> = Vec::new();
    let mut p = 0;
    while p + 5 <= gl.len() {
        if gl_offset_framed(gl, p) {
            let v = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]);
            framed.push((p, v));
        }
        p += 1;
    }
    // Fail-closed cross-check: the framed offsets must match `.ex` exactly.
    let values: Vec<u32> = framed.iter().map(|&(_, v)| v).collect();
    let offsets: BTreeSet<usize> = if values == ex_offsets_ordered {
        framed.iter().map(|&(pos, _)| pos).collect()
    } else {
        BTreeSet::new()
    };

    let mut spans = Vec::new();
    let mut pending: Vec<u8> = Vec::new();
    let mut p = 0;
    while p < gl.len() {
        if offsets.contains(&p) {
            let v = u32::from_le_bytes([gl[p + 1], gl[p + 2], gl[p + 3], gl[p + 4]]);
            if !pending.is_empty() {
                spans.push(Span::Opaque(std::mem::take(&mut pending)));
            }
            spans.push(Span::GlOffset(v));
            p += 5;
            continue;
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
            // `Lo` is ONE byte and `LoRecord` is the optional record beside it
            // (§10.12) — the two together re-encode to the old three-byte atom,
            // which is what every source function's body still opens with.
            (ExToken::Lo, &[0x4C]),
            (ExToken::LoRecord, &[0x4F, 0x11]),
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
            // Float-leaf vocabulary (byte evidence from the volf capture).
            (ExToken::FloatLoad(0xF109), &[0xB9, 0xF1, 0x09, 0x86, 0x45, 0x40]),
            (
                ExToken::PtrLoad {
                    tok: 0xED09,
                    ty: [0x82, 0x20],
                },
                &[0xB9, 0xED, 0x09, 0x86, 0x43, 0x82, 0x20],
            ),
            (ExToken::MemberPtr([0x86, 0x20]), &[0x27, 0x86, 0x43, 0x86, 0x20]),
            (ExToken::Deref([0x45, 0x85, 0x20]), &[0x30, 0xA6, 0x45, 0x85, 0x20]),
            (ExToken::CastFloat, &[0x2C, 0x86, 0x45, 0x40, 0x00]),
            (ExToken::StoreFloat, &[0x32, 0x86, 0x45, 0x40, 0x4B]),
            (ExToken::ResultTypeFloat, &[0x41, 0x86, 0x45, 0x40]),
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
                ExToken::LoRecord,
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
    fn parse_gl_types_body_start_offset_by_record_framing() {
        // A .gl fragment carrying TWO `80 54 0A 00 00` fields whose value is the
        // real .ex offset 0x0A54: one WITHOUT the record framing (a value
        // collision that structural framing must reject) and one WITH the framing
        // `80 01 10 00 00 00 00` immediately before it (the real offset field).
        // Only the framed one is typed — proving location by position, not value.
        let gl: &[u8] = &[
            0xAA, 0xBB, // pad
            0x80, 0x54, 0x0A, 0x00, 0x00, // value collision, UNFRAMED -> opaque
            0xCC, //
            0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00, // the 7-byte offset framing
            0x80, 0x54, 0x0A, 0x00, 0x00, // the real framed offset field
            0xDD,
        ];
        let spans = parse_gl(gl, &[0x0A54]);
        // Exactly one typed offset, value 0x0A54 — the framed one.
        let offs: Vec<u32> = spans
            .iter()
            .filter_map(|s| match s {
                Span::GlOffset(v) => Some(*v),
                _ => None,
            })
            .collect();
        assert_eq!(offs, vec![0x0A54]);
        // The unframed value-collision at offset 2 stayed opaque.
        assert!(spans
            .iter()
            .any(|s| matches!(s, Span::Opaque(b) if b.contains(&0xAA))));
        // Round-trips.
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, gl);
    }

    #[test]
    fn parse_gl_rejects_when_framed_offsets_disagree_with_ex() {
        // A framed offset field for 0x0A54, but `.ex` declares a DIFFERENT
        // function set {0x0B00}. The 1:1/order cross-check fails, so NONE is
        // typed (fail-closed — K3 is never handed a mismatched site).
        let gl: &[u8] = &[
            0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00, // framing
            0x80, 0x54, 0x0A, 0x00, 0x00, // framed offset 0x0A54
            0x00,
        ];
        let spans = parse_gl(gl, &[0x0B00]);
        assert!(spans.iter().all(|s| matches!(s, Span::Opaque(_))));
        assert_eq!(spans, vec![Span::Opaque(gl.to_vec())]);
    }

    #[test]
    fn parse_gl_all_opaque_when_no_ex_offsets() {
        let gl: &[u8] = &[0x80, 0x54, 0x0A, 0x00, 0x00, 0x01, 0x02];
        let spans = parse_gl(gl, &[]);
        assert_eq!(spans, vec![Span::Opaque(gl.to_vec())]);
    }

    #[test]
    fn full_model_round_trips_and_opaque_files_preserved() {
        let mut bundle = IlBundle::new("_CL_synthetic");
        let ex = synthetic_add3_ex();
        // Place the offset field in .gl matching the .ex 4F 1F offset, in its
        // record framing (`80 XX 10 00 00 00 00` precedes it), so the structural
        // K2a identification accepts it.
        let mods = ex_fn_start_offsets(&ex);
        let off = *mods.iter().next().unwrap();
        let mut gl = b"?add3@@YAHHHH@Z\x00".to_vec();
        gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]); // framing
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

    // ---- `.ex` per-function metadata prefix (K2a) ---------------------------

    /// A REAL `int add3(int,int,int){return a+b+c;}` `.ex` segment (prefix +
    /// body), transcribed from a live 16.00.11886.00 `/Bd /d2nop /Ox /GS- /c`
    /// capture from the `4F 1F` function-start marker. Only `.ex` opcode bytes —
    /// no host path (those live in `.gl`, which is why bundles are not committed).
    const ADD3_SEGMENT: &[u8] = &[
        // FnHeader preamble (4F 1F .. before the 4F 02 20 00 block-start):
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, // 4F 1F fn-start + descriptor
        0x4F, 0x20, 0x80, 0xFE, 0x00, // 4F 20 descriptor
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03,
        0x0F, // 4F 33 <len=0D> + 13 metadata bytes
        0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, // trailing metadata blob
        0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, // 42 45 'BE' block-entry
        0x0F, // 0F
        // block-start marker + SS SS + result-ref + formals:
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x01, // 4F 02 20 00 4F 01 NN block-start (NN=01)
        0x53, 0x53, // SS SS
        0x26, 0xE6, 0x09, // result-ref
        0x46, 0x2D, 0xE5, 0x09, 0x2D, 0xE4, 0x09, 0x2D, 0xE3, 0x09, // formals c,b,a
        // body from the LO marker:
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
    ];

    /// The FnHeader preamble bytes (from `4F 1F` up to the `4F 02 20 00`
    /// block-start) — the fixed, structurally-bounded island.
    fn add3_fn_header() -> Vec<u8> {
        let bs = find_subslice(ADD3_SEGMENT, &BLOCK_START).unwrap();
        ADD3_SEGMENT[..bs].to_vec()
    }

    #[test]
    fn prefix_decodes_header_blockstart_ss_resultref_formals() {
        let lo = crate::func::body_start(ADD3_SEGMENT).unwrap();
        let mut spans = Vec::new();
        walk_ex_prefix(&ADD3_SEGMENT[..lo], 2, &mut spans);
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
                ExToken::FnHeader(add3_fn_header()),
                ExToken::BlockStart(0x01),
                ExToken::Ss,
                ExToken::Ss,
                ExToken::ResultRef(0xE609),
                ExToken::Formals,
                ExToken::Formal(0xE509),
                ExToken::Formal(0xE409),
                ExToken::Formal(0xE309),
            ]
        );
        // No opaque bytes remain in the prefix — it is fully typed.
        assert!(
            !spans.iter().any(|s| matches!(s, Span::Opaque(_))),
            "the add3 metadata prefix should be fully typed, no opaque residue"
        );
        // And it round-trips.
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, &ADD3_SEGMENT[..lo]);
    }

    #[test]
    fn block_start_token_round_trips() {
        let bytes: &[u8] = &[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07];
        let mut out = Vec::new();
        ExToken::BlockStart(0x07).encode_into(&mut out);
        assert_eq!(out, bytes);
        assert_eq!(
            try_prefix_token(bytes, 0),
            Some((ExToken::BlockStart(0x07), 7))
        );
        // A trailing 4D (module end, not block-start) is NOT a BlockStart here.
        let modend: &[u8] = &[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x4D];
        assert_eq!(try_prefix_token(modend, 0), Some((ExToken::BlockStart(0x07), 7)));
        // (the 4D is left for the body walker's ModuleEnd — prefix never sees it)
    }

    #[test]
    fn fn_header_falls_back_to_opaque_without_block_start() {
        // A minimal `4F 1F` prefix with no block-start ahead: FnHeader must not
        // fire; the bytes stay opaque and still round-trip.
        let prefix: &[u8] = &[0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x46, 0x2D, 0xE5, 0x09];
        let mut spans = Vec::new();
        walk_ex_prefix(prefix, 2, &mut spans);
        assert!(!spans
            .iter()
            .any(|s| matches!(s, Span::Ex(ExToken::FnHeader(_)))));
        assert!(matches!(spans.first(), Some(Span::Opaque(b)) if b.starts_with(&FN_START)));
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, prefix);
    }

    #[test]
    fn realistic_ex_round_trips_and_shrinks_opaque() {
        // A full single-function `.ex`: header pad + the real add3 segment.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 12]);
        ex.extend_from_slice(ADD3_SEGMENT);
        let spans = parse_ex(&ex);
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, ex);
        // The prefix is now typed: exactly one FnHeader + one BlockStart present.
        assert_eq!(
            spans
                .iter()
                .filter(|s| matches!(s, Span::Ex(ExToken::FnHeader(_))))
                .count(),
            1
        );
        assert_eq!(
            spans
                .iter()
                .filter(|s| matches!(s, Span::Ex(ExToken::BlockStart(_))))
                .count(),
            1
        );
        // The only opaque residue is the header/index pad before the function.
        let opaque_bytes: usize = spans
            .iter()
            .filter_map(|s| match s {
                Span::Opaque(b) => Some(b.len()),
                _ => None,
            })
            .sum();
        assert_eq!(opaque_bytes, 16, "only the 16-byte header pad stays opaque");
    }

    #[test]
    fn gl_offsets_are_one_to_one_with_ex_function_count() {
        // A two-function bundle: two `4F 1F` segments in `.ex`, two framed
        // offset fields in `.gl` — the `== function_count` invariant K3 relies on.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]);
        let f0 = ex.len() as u32;
        ex.extend_from_slice(ADD3_SEGMENT);
        let f1 = ex.len() as u32;
        ex.extend_from_slice(ADD3_SEGMENT);

        // .gl: two framed offset fields, in function order.
        let mut gl = b"?add3@@YAHHHH@Z\x00".to_vec();
        for off in [f0, f1] {
            gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]);
            gl.push(0x80);
            gl.extend_from_slice(&off.to_le_bytes());
        }

        let mut bundle = IlBundle::new("_CL_two");
        bundle.set("ex", ex);
        bundle.set("gl", gl);
        let model = IlModel::parse(&bundle).expect("round-trips");
        assert_eq!(model.encode().files, bundle.files);
        assert_eq!(model.ex_function_count(), 2);
        assert_eq!(model.gl_body_start_offsets(), vec![f0, f1]);
        assert_eq!(
            model.gl_body_start_offsets().len(),
            model.ex_function_count(),
            "typed .gl offsets must be 1:1 with .ex functions"
        );
    }

    // ---- float-leaf codec widening (Box::Volume class) ---------------------
    //
    // A REAL single-function `.ex` segment captured from the live
    // 16.00.11886.00 toolchain (`/Bd /d2nop /Ox /GS- /c`) of the faithful
    // `Box::Volume` reduction
    //   `float volf(const V* a, const V* b) {
    //        float x = a->x - b->x; float y = a->y - b->y;
    //        float z = a->z - b->z; return x * y * z; }`
    // (`?volf@@YAMPBUV@@0@Z`; `V { float x,y,z; }`), transcribed from the
    // `4F 1F` function-start marker. Only `.ex` opcode bytes — the host path
    // lives in `.gl`, which is why bundles are not committed. This is the
    // float-leaf class the near-miss lane needs: float arithmetic (`03` SUB,
    // `04` MUL) over struct-member loads (`LOAD ptr ; LIT off ; MEMBER_PTR ;
    // DEREF`), materialized through CAST/STORE and returned float. Before the
    // widening it parsed with ~21 interleaved opaque runs (the float/pointer
    // loads + member-access ops) and yielded ZERO K3a neighbors; after it, the
    // body is a *contiguous* typed run.
    const VOLF_SEGMENT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
        0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
        0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEF, 0x09,
        0x46, 0x2D, 0xEE, 0x09, 0x2D, 0xED, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F,
        0x01, 0x03, 0x26, 0xF1, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43, 0x82, 0x20,
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6,
        0x45, 0x85, 0x20, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x33, 0x86,
        0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6, 0x45, 0x85,
        0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32, 0x86, 0x45, 0x40, 0x4B,
        0x4F, 0x01, 0x04, 0x26, 0xF2, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43, 0x82,
        0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30,
        0xA6, 0x45, 0x85, 0x20, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x33,
        0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6, 0x45,
        0x85, 0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32, 0x86, 0x45, 0x40,
        0x4B, 0x4F, 0x01, 0x05, 0x26, 0xF3, 0x09, 0xB9, 0xED, 0x09, 0x86, 0x43,
        0x82, 0x20, 0x33, 0x86, 0x41, 0x74, 0x08, 0x27, 0x86, 0x43, 0x86, 0x20,
        0x30, 0xA6, 0x45, 0x85, 0x20, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x82, 0x20,
        0x33, 0x86, 0x41, 0x74, 0x08, 0x27, 0x86, 0x43, 0x86, 0x20, 0x30, 0xA6,
        0x45, 0x85, 0x20, 0x03, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x32, 0x86, 0x45,
        0x40, 0x4B, 0x4F, 0x01, 0x06, 0xB9, 0xF1, 0x09, 0x86, 0x45, 0x40, 0xB9,
        0xF2, 0x09, 0x86, 0x45, 0x40, 0x04, 0xB9, 0xF3, 0x09, 0x86, 0x45, 0x40,
        0x04, 0x41, 0x86, 0x45, 0x40, 0x3A, 0xF0, 0x09, 0x4F, 0x01, 0x07, 0x54,
        0x02, 0x29, 0xF0, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F,
        0x02, 0x20, 0x00, 0x4F, 0x01, 0x08, 0x4D,
    ];

    /// A full single-function `.ex` for the volf float leaf: header pad + the
    /// captured segment (mirrors [`build_bundle`]'s `.ex` shape).
    fn volf_ex() -> Vec<u8> {
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 12]);
        ex.extend_from_slice(VOLF_SEGMENT);
        ex
    }

    #[test]
    fn volf_float_leaf_round_trips_and_is_fully_typed() {
        let ex = volf_ex();
        let spans = parse_ex(&ex);
        // Byte-exact round-trip (the standing K1 invariant).
        let mut out = Vec::new();
        for s in &spans {
            s.encode_into(&mut out);
        }
        assert_eq!(out, ex, "volf .ex must round-trip byte-identically");

        // The ONLY opaque residue is the 16-byte header pad — the entire float
        // leaf body (float/pointer loads, member-access ops, CAST/STORE, float
        // result-type) is now typed, no interleaved opaque runs.
        let opaque_bytes: usize = spans
            .iter()
            .filter_map(|s| match s {
                Span::Opaque(b) => Some(b.len()),
                _ => None,
            })
            .sum();
        assert_eq!(
            opaque_bytes, 16,
            "only the 16-byte header pad stays opaque; the float leaf is fully typed"
        );

        // The new float-leaf tokens are present with the expected multiplicity
        // (6 member accesses × {PtrLoad, MemberPtr, Deref}, 3 × {CastFloat,
        // StoreFloat}, 3 float value loads, 1 float result-type).
        let toks: Vec<ExToken> = spans
            .iter()
            .filter_map(|s| match s {
                Span::Ex(t) => Some(t.clone()),
                _ => None,
            })
            .collect();
        let count = |pred: &dyn Fn(&ExToken) -> bool| toks.iter().filter(|t| pred(t)).count();
        assert_eq!(count(&|t| matches!(t, ExToken::PtrLoad { .. })), 6);
        assert_eq!(count(&|t| matches!(t, ExToken::MemberPtr(_))), 6);
        assert_eq!(count(&|t| matches!(t, ExToken::Deref(_))), 6);
        assert_eq!(count(&|t| matches!(t, ExToken::CastFloat)), 3);
        assert_eq!(count(&|t| matches!(t, ExToken::StoreFloat)), 3);
        assert_eq!(count(&|t| matches!(t, ExToken::FloatLoad(_))), 3);
        assert_eq!(count(&|t| matches!(t, ExToken::ResultTypeFloat)), 1);
        // The float arithmetic itself: 3 SUBs (the member diffs), 2 MULs (the
        // product) — already the type-agnostic `03`/`04`, now inside the
        // contiguous typed run rather than stranded among opaque bytes.
        assert_eq!(count(&|t| matches!(t, ExToken::Sub)), 3);
        assert_eq!(count(&|t| matches!(t, ExToken::Mul)), 2);
        // And the 6 int member-offset literals (0,0,4,4,8,8) — editable Lits.
        assert_eq!(count(&|t| matches!(t, ExToken::Lit { .. })), 6);
    }

    #[test]
    fn volf_float_leaf_body_is_token_addressable_with_editable_ops() {
        // THE GATE (codec side): the float-leaf body is a contiguous typed run,
        // so `function_tokens` succeeds (no `OpaqueFunctionBody`) — the exact
        // precondition the stuck-dc3 attempt failed (interleaved opaque → zero
        // K3a neighbors). The move set derives K3a neighbors from these tokens;
        // this asserts the codec now exposes editable float ops + literals the
        // move set's rules (`is_binop` for SUB/MUL, literal widen for the
        // offsets) will act on. The neighbor COUNT itself is asserted in the
        // harness (`search_differential.rs`), which owns `MoveSet`.
        let mut bundle = IlBundle::new("_CL_volf");
        bundle.set("ex", volf_ex());
        let model = IlModel::parse(&bundle).expect("volf bundle round-trips");
        assert_eq!(model.ex_function_count(), 1);

        let tokens = model
            .function_tokens(0)
            .expect("float-leaf body is token-addressable (contiguous typed run)");

        // Editable float ops the move set accepts: SUB/MUL are `is_binop`.
        let binops = tokens
            .iter()
            .filter(|t| matches!(t, ExToken::Sub | ExToken::Mul))
            .count();
        assert!(binops >= 1, "≥1 editable float binop (SUB/MUL) for the move set");
        // Editable literals (the member offsets) — the widen/narrow move site.
        let lits = tokens
            .iter()
            .filter(|t| matches!(t, ExToken::Lit { .. }))
            .count();
        assert!(lits >= 1, "≥1 editable literal (member offset) for widen/narrow");
    }

    // ---- K3a length-edit primitive -----------------------------------------

    /// A fully-typed single-function `addk`-shaped segment: the real add3
    /// FnHeader preamble + block-start + SS + result-ref + one formal, then a
    /// body `LOAD a ; LIT 5 ; ADD` (i.e. `a + 5`) with a NARROW literal, closed
    /// by the result-type / assign / return / fn-tail / module-end. Parses with
    /// no opaque residue (like the real capture), so it is token-addressable.
    fn addk_segment() -> Vec<u8> {
        let mut seg = add3_fn_header(); // `4F 1F` … up to the block-start
        seg.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x01]); // BlockStart(01)
        seg.push(0x53); // Ss
        seg.extend_from_slice(&[0x26, 0xE6, 0x09]); // ResultRef
        seg.extend_from_slice(&[0x46, 0x2D, 0xE3, 0x09]); // Formals + formal a
        seg.extend_from_slice(&[0x4C, 0x4F, 0x11, 0x53]); // LO SS
        seg.extend_from_slice(&[0xB9, 0xE3, 0x09, 0x86, 0x41, 0x74]); // LOAD a
        seg.extend_from_slice(&[0x33, 0x86, 0x41, 0x74, 0x05]); // LIT 5 (narrow, +4 to widen)
        seg.push(0x02); // ADD
        seg.extend_from_slice(&[0x41, 0x86, 0x41, 0x74]); // result-type
        seg.extend_from_slice(&[0x3A, 0xE7, 0x09]); // ASSIGN
        seg.extend_from_slice(&[0x54, 0x02, 0x29, 0xE7, 0x09]); // RETURN
        seg.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
        seg.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x4D]); // ModuleEnd
        seg
    }

    /// Build a bundle from the given `.ex` function segments (a header pad is
    /// prepended), optionally with a framed `.gl` offset column that is 1:1 with
    /// the `.ex` `4F 1F` positions (so K2a types it).
    fn build_bundle(segments: &[Vec<u8>], with_gl: bool) -> IlBundle {
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]); // opaque header pad
        let mut offs = Vec::new();
        for seg in segments {
            offs.push(ex.len() as u32);
            ex.extend_from_slice(seg);
        }
        let mut bundle = IlBundle::new("_CL_edit");
        bundle.set("ex", ex);
        if with_gl {
            let mut gl = b"?fn@@YAHH@Z\x00".to_vec();
            for off in &offs {
                gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]); // framing
                gl.push(0x80);
                gl.extend_from_slice(&off.to_le_bytes());
            }
            bundle.set("gl", gl);
        }
        bundle
    }

    /// The token index of the first `Lit` in a function.
    fn lit_index(model: &IlModel, fn_index: usize) -> usize {
        model
            .function_tokens(fn_index)
            .unwrap()
            .iter()
            .position(|t| matches!(t, ExToken::Lit { .. }))
            .expect("a literal in this function")
    }

    #[test]
    fn widen_nonlast_fn_bumps_downstream_gl_by_delta() {
        let bundle = build_bundle(&[addk_segment(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let [f0, f1] = model.gl_body_start_offsets()[..] else {
            panic!("expected two typed offsets");
        };
        let old_ex_len = bundle.get("ex").unwrap().len();

        let idx = lit_index(&model, 0);
        let report = model.set_literal_wide(0, idx, true).expect("widen fn0");

        // +4 B (1-byte varint -> `80`+LE32), downstream fn shifted by exactly +4.
        assert_eq!(report.byte_delta, 4);
        assert_eq!(report.fn_index, 0);
        assert_eq!(model.gl_body_start_offsets(), vec![f0, f1 + 4]);
        assert_eq!(report.gl_offsets, vec![f0, f1 + 4]);
        // The edited/preceding function's own start is unchanged.
        assert_eq!(model.ex_start_offsets_vec()[0], f0);
        // Internally consistent: `.ex` grew by the delta and the edited bundle
        // itself re-parses (the re-emitted `.gl` matches the new `4F 1F` set).
        assert_eq!(model.files.iter().find(|f| f.suffix == "ex").unwrap().encode().len(),
                   old_ex_len + 4);
        let reparsed = IlModel::parse(&model.encode()).expect("edited bundle re-parses");
        assert_eq!(reparsed.gl_body_start_offsets(), vec![f0, f1 + 4]);
    }

    #[test]
    fn widen_last_fn_leaves_gl_unchanged() {
        // The P0.6a zero-bookkeeping regime: a last-function edit needs no `.gl`
        // patch — the downstream set is empty, so no offset moves.
        let bundle = build_bundle(&[addk_segment(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let before = model.gl_body_start_offsets();

        let idx = lit_index(&model, 1);
        let report = model.set_literal_wide(1, idx, true).expect("widen fn1 (last)");

        assert_eq!(report.byte_delta, 4);
        // No offset changed (fn1's own start is fixed; nothing follows it).
        assert_eq!(model.gl_body_start_offsets(), before);
        IlModel::parse(&model.encode()).expect("edited bundle re-parses");
    }

    #[test]
    fn insert_arith_term_grows_and_shifts_gl() {
        // P0.6a E: splice `LIT 5 ; ADD` after the existing add -> `(a+5)+5`, +6 B.
        let bundle = build_bundle(&[addk_segment(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let [f0, f1] = model.gl_body_start_offsets()[..] else {
            panic!("two offsets");
        };
        // Insert right after the body's ADD (which follows the first Lit).
        let toks = model.function_tokens(0).unwrap();
        let add_after_lit = toks
            .iter()
            .position(|t| matches!(t, ExToken::Add))
            .expect("an ADD");
        let report = model
            .splice_function_tokens(
                0,
                add_after_lit + 1..add_after_lit + 1,
                vec![ExToken::Lit { value: 5, wide: false }, ExToken::Add],
            )
            .expect("insert term");
        assert_eq!(report.byte_delta, 6); // LIT(5B) + ADD(1B)
        assert_eq!(model.gl_body_start_offsets(), vec![f0, f1 + 6]);
        IlModel::parse(&model.encode()).expect("re-parses");
    }

    #[test]
    fn delete_arith_term_shrinks_and_shifts_gl() {
        // P0.6a F: drop `LOAD c ; ADD` from `a+b+c` -> `a+b`, -7 B. add3 is the
        // first of two functions so the change is a non-last edit (`.gl` re-emit).
        let bundle = build_bundle(&[ADD3_SEGMENT.to_vec(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let [f0, f1] = model.gl_body_start_offsets()[..] else {
            panic!("two offsets");
        };
        // Find the LAST `Load` in add3's body (that is `c`) and the ADD after it.
        let toks = model.function_tokens(0).unwrap();
        let last_load = toks
            .iter()
            .rposition(|t| matches!(t, ExToken::Load(_)))
            .expect("a load");
        assert!(matches!(toks[last_load + 1], ExToken::Add), "LOAD c then ADD");
        let report = model
            .splice_function_tokens(0, last_load..last_load + 2, vec![])
            .expect("delete term");
        assert_eq!(report.byte_delta, -7); // LOAD(6B) + ADD(1B)
        assert_eq!(model.gl_body_start_offsets(), vec![f0, (f1 as i64 - 7) as u32]);
        IlModel::parse(&model.encode()).expect("re-parses");
    }

    #[test]
    fn edit_that_creates_a_function_is_refused() {
        // Splicing a token whose bytes carry a fresh `4F 1F` would add a function
        // — out of K3a scope; refuse fail-closed and leave the model untouched.
        let bundle = build_bundle(&[addk_segment(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let before_gl = model.gl_body_start_offsets();
        let toks = model.function_tokens(0).unwrap();
        let add = toks.iter().position(|t| matches!(t, ExToken::Add)).unwrap();
        let err = model
            .splice_function_tokens(
                0,
                add + 1..add + 1,
                vec![ExToken::FnHeader(vec![0x4F, 0x1F, 0x00])], // stray marker
            )
            .expect_err("must refuse");
        assert!(matches!(err, EditError::FunctionSetChanged { before: 2, after: 3 }));
        // Untouched.
        assert_eq!(model.gl_body_start_offsets(), before_gl);
        assert_eq!(model.encode().files, bundle.files);
    }

    #[test]
    fn nonlast_edit_without_typed_gl_is_refused_but_last_is_allowed() {
        // No `.gl` at all: a non-last edit cannot discharge the offset re-emit, so
        // it is refused (a stale downstream offset would SIGSEGV). A LAST-function
        // edit needs no re-emit, so it is allowed even with no `.gl`.
        let bundle = build_bundle(&[addk_segment(), addk_segment()], false);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let idx0 = lit_index(&model, 0);
        let err = model.set_literal_wide(0, idx0, true).expect_err("refuse non-last");
        assert!(matches!(err, EditError::GlOffsetsNotTyped { fn_index: 0 }));
        assert_eq!(model.encode().files, bundle.files); // untouched

        let idx1 = lit_index(&model, 1);
        let report = model.set_literal_wide(1, idx1, true).expect("last-fn ok");
        assert_eq!(report.byte_delta, 4);
        assert!(report.gl_offsets.is_empty()); // nothing to re-emit
    }

    #[test]
    fn narrow_of_a_wide_literal_shrinks_by_four() {
        // First widen fn1 (last), then narrow it back — the P0.6a A/B pair.
        let bundle = build_bundle(&[addk_segment(), addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        let idx = lit_index(&model, 1);
        model.set_literal_wide(1, idx, true).expect("widen");
        let report = model.set_literal_wide(1, idx, false).expect("narrow back");
        assert_eq!(report.byte_delta, -4);
        // Back to the exact original bytes.
        assert_eq!(model.encode().files, bundle.files);
    }

    #[test]
    fn edit_errors_are_typed_and_nonmutating() {
        let bundle = build_bundle(&[addk_segment()], true);
        let mut model = IlModel::parse(&bundle).expect("round-trips");
        // Out-of-range function.
        assert!(matches!(
            model.function_tokens(9),
            Err(EditError::NoSuchFunction { index: 9, count: 1 })
        ));
        // Widen a non-literal token (the leading FnHeader at index 0).
        assert!(matches!(
            model.set_literal_wide(0, 0, true),
            Err(EditError::NotALiteral { fn_index: 0, token_index: 0 })
        ));
        // Narrow a value that does not fit the 1-byte form.
        let idx = lit_index(&model, 0);
        model.set_literal_wide(0, idx, true).expect("widen");
        // Rewrite the value wide-only to 0x1000 via a splice, then try to narrow.
        model
            .splice_function_tokens(0, idx..idx + 1, vec![ExToken::Lit { value: 0x1000, wide: true }])
            .expect("set wide value");
        assert!(matches!(
            model.set_literal_wide(0, idx, false),
            Err(EditError::ValueNotNarrowable { value: 0x1000 })
        ));
    }
}
