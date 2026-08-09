//! IL bundle container model.
//!
//! The MSVC Xbox 360 pipeline is `c1xx.dll` (C++ front-end) → **IL files** →
//! `c2.dll` (PPC back-end). One compilation emits a 5-file bundle with a common
//! base name `_CL_<hash>` and the suffixes (no dot): `ex gl sy in db`. `.ex` is
//! the main IL bytecode stream; the others carry globals, symbols, imports, and
//! debug info.
//!
//! This crate is the **container** only: raw bytes keyed by suffix, plus load /
//! write / round-trip and a couple of cheap header sniffs. It is deliberately
//! NOT an IL disassembler — decoding the `.ex` opcode grammar, type encodings,
//! and symbol tables is a separate, much larger workstream.
//!
//! The reference Python decoder that this will eventually mirror lives at
//! `dc3-decomp/msvc-src/tools/il_parser.py` (opcode table, `try_parse_type`,
//! `ILFunction`, `ILSymbols`, …).
//!
//! # K1 lossless codec ([`mod@codec`])
//!
//! [`IlModel::parse`] / [`IlModel::encode`] are the **round-trip-gated container
//! codec**: they decode a bundle into typed islands (the `.ex` operand-stream
//! tokens, the `.ex` per-function metadata prefix — FnHeader / block-start /
//! `53 53` / result-ref / formals, added in K2a — and the `.gl` `80 <LE32>`
//! body-start offset field, K2a-located by record framing and gated 1:1 with the
//! function count) over opaque spans for everything not yet decoded, with the
//! invariant `encode(parse(b)) == b` byte-for-byte or a fail-closed
//! [`CodecError`]. Still opaque, hence the remaining K2 backlog: the `.ex`
//! header/index, the FnHeader interior, the rest of `.gl`, and all of
//! `.sy`/`.in`/`.db` (coverage map in `docs/IL_BUNDLE_MVP.md`).

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

pub mod codec;
pub mod func;
pub use codec::{CodecError, EditError, EditReport, ExToken, FileModel, IlModel, Span};
pub use func::{
    chain_form, gl_body_record_names, slot_sources, ChainForm, EmitBinding, FP_SCRATCH,
    detect_token_width, gl_alias_table, gl_alias_table_shifted, gl_symbol_conflicts,
    gl_symbol_index, GlAliasStats, GlAliasTable, is_empty_module, label_counter,
    mangled_name, mangled_names, opt_word_mode, source_path, Block, FnCensus, OptWordMode,
    OPT_WORD_O1, OPT_WORD_OX, OPT_WORD_SPECIAL_MEMBER,
    FnVerdict,
    InInitReport, InInitResidue, InSymbolRef,
    CallSeq, CmpShiftOr, CompareLeaf, DataObject, DataTu, DynInitTu, FpTail, FramedCall, GlDataRow, InAliasReport,
    IlFunction, IlOp,
    DivModLeaf,
    FloatWalkLoop, FloatWalkOp, FloatWalkShape,
    FpDiamondConstStore, FpDiamondDiv, FpStoreDiamond, CtorForwardCall,
    IfCallJoin,
    AllocInitOrFail,
    AllocInitOrFailFn,
    GuardRetChain,
    // W-XTEA2 — the whole-body `memcpy` tail branch (`EncryptXTEA.cpp`).
    MemcpyTail,
    GuardRetGuard,
    GuardRetSpine,
    OsfHandleGuard,
    OsfHandleGuardFn,
    JsonUtf8Copy,
    JsonUtf8CopyFn,
    XlrcCreateGuard,
    XlrcCreateGuardFn,
    GuardChainSharedTailFn,
    IfCallJoinFn,
    PtrWalkModLoop,
    // W-POOL2 — the intrusive free list: the two guarded leaves and the
    // constructor that builds the chain.
    PoolCtorChain, PoolFreeList, PoolFreeListOp,
    // W-BDNZ — the counted-`for` accumulate loop's three free fields.
    CountedAccumLoop, CountedAccumOp,
    ChainOp, ChainOpKind, ChainRhs, PtrWalkChainLoop,
    Rel, SeqCall, SeqCmp, SeqEarlyReturn, SeqGuard, SeqTail, StoreRunPrefix,
    SlotArg, LINK_FIRST_SLOT,
    // W8 — the two-arm conditional tail call and its register schedule. The
    // schedule is exported because the emitter must run the *same* planner the
    // parser gated on, never a copy (`docs/GAPS.md` §6 instance #9).
    plan_cond_pair, CondArm, CondPlan, CondStep, CondTailPair, COND_PARK_REG,
    // W42 / W43 — the two folds this port derives at PARSE time so the census
    // and the emitter cannot disagree about which literals are in class.
    shift_mask_rlwinm, shift_or_rlwimi,
};

/// The five IL suffixes (no leading dot), in canonical order. `.ex` first
/// because it is the main IL stream; mirrors `IL_SUFFIXES` in il_parser.py.
pub const IL_SUFFIXES: [&str; 5] = ["ex", "gl", "sy", "in", "db"];

/// `.ex` header magic — the first four bytes of a real IL bytecode stream.
/// (`5B 80 54 0A`, per `msvc-src/docs/IL_FORMAT.md`.)
pub const EX_MAGIC: [u8; 4] = [0x5B, 0x80, 0x54, 0x0A];

/// Returns true iff `data` begins with the `.ex` header magic `5B 80 54 0A`.
pub fn is_ex_magic(data: &[u8]) -> bool {
    data.len() >= EX_MAGIC.len() && data[..EX_MAGIC.len()] == EX_MAGIC
}

/// A captured IL bundle: the raw bytes of the (up to) five `_CL_*` files,
/// keyed by suffix, plus the suffix-free base name (e.g. `_CL_291e984a`).
///
/// Missing files are simply absent from `files` — the container does not
/// require all five to be present, though a real capture always yields all
/// five and callers generally require a non-empty `.ex`.
#[derive(Clone, Debug, Default)]
pub struct IlBundle {
    /// Bundle base, suffix-free, e.g. `_CL_291e984a`.
    pub base_name: String,
    /// suffix (`"ex"`, `"gl"`, …) -> raw file bytes.
    pub files: BTreeMap<String, Vec<u8>>,
}

impl IlBundle {
    /// An empty bundle with the given base name.
    pub fn new(base_name: impl Into<String>) -> Self {
        IlBundle {
            base_name: base_name.into(),
            files: BTreeMap::new(),
        }
    }

    /// The canonical suffix list, `["ex","gl","sy","in","db"]`.
    pub fn suffixes(&self) -> &'static [&'static str] {
        &IL_SUFFIXES
    }

    /// Raw bytes for a suffix, if present.
    pub fn get(&self, suffix: &str) -> Option<&[u8]> {
        self.files.get(suffix).map(|v| v.as_slice())
    }

    /// Insert/replace one file's bytes.
    pub fn set(&mut self, suffix: impl Into<String>, bytes: Vec<u8>) {
        self.files.insert(suffix.into(), bytes);
    }

    /// The main `.ex` IL stream, if present.
    pub fn ex(&self) -> Option<&[u8]> {
        self.get("ex")
    }

    /// True iff the `.ex` stream is present and starts with the header magic.
    pub fn has_ex_magic(&self) -> bool {
        self.ex().map(is_ex_magic).unwrap_or(false)
    }

    /// The compiler label counter from `.gl` — see [`func::label_counter`].
    /// `None` when `.gl` is absent or its header does not match, in which case
    /// a TU containing a framed function must be refused rather than emitted.
    pub fn label_counter(&self) -> Option<u32> {
        self.get("gl").and_then(func::label_counter)
    }

    /// Heuristic token width for the `.ex` stream: 4 bytes for real
    /// compilations, 2 for the tiny hand-authored test files il_parser.py was
    /// originally built against.
    ///
    /// HEURISTIC ONLY — the real detector (il_parser.py `_detect_token_width`)
    /// finds the first `4F 02` and measures the gap to the next `4F`. Here we
    /// use a crude size proxy so callers have *something* to key on. Port the
    /// real detector when the codec lands.
    // TODO(A2 codec): replace with the 4F-02 gap measurement from il_parser.py.
    pub fn token_width(&self) -> u32 {
        match self.ex() {
            Some(ex) if ex.len() >= 512 => 4,
            _ => 2,
        }
    }

    /// Load a bundle from `dir` given the suffix-free `base` (e.g.
    /// `_CL_291e984a`). Files that are not present are skipped; other I/O
    /// errors propagate. The `.ex` may legitimately be absent here — callers
    /// that require it should check [`IlBundle::ex`] afterwards.
    pub fn load_from_dir(dir: &Path, base: &str) -> io::Result<IlBundle> {
        let mut bundle = IlBundle::new(base.to_string());
        for suffix in IL_SUFFIXES {
            let path = dir.join(format!("{base}{suffix}"));
            match std::fs::read(&path) {
                Ok(bytes) => {
                    bundle.set(suffix, bytes);
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
        Ok(bundle)
    }

    /// Write every present file to `dir` as `{base}{suffix}`. Creates `dir`.
    pub fn write_to_dir(&self, dir: &Path, base: &str) -> io::Result<()> {
        std::fs::create_dir_all(dir)?;
        for suffix in IL_SUFFIXES {
            if let Some(bytes) = self.get(suffix) {
                let path = dir.join(format!("{base}{suffix}"));
                std::fs::write(&path, bytes)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn tmpdir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!(
            "c2rs-il-{tag}-{}-{}-{}",
            std::process::id(),
            nanos,
            n
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn synthetic() -> IlBundle {
        let mut b = IlBundle::new("_CL_deadbeef");
        // .ex begins with header magic, then some filler.
        let mut ex = EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 32]);
        ex.extend_from_slice(b"\x4f\x02\x20\x00\x4f\x01\x00\x4d");
        b.set("ex", ex);
        b.set("gl", b"?add3@@YAHHHH@Z\x00".to_vec());
        b.set("sy", b"a\x00b\x00c\x00".to_vec());
        b.set("in", vec![0x86, 0x41, 0x74]);
        b.set("db", vec![0x01, 0x02, 0x03]);
        b
    }

    #[test]
    fn is_ex_magic_matches() {
        assert!(is_ex_magic(&EX_MAGIC));
        assert!(is_ex_magic(&[0x5B, 0x80, 0x54, 0x0A, 0xFF]));
        assert!(!is_ex_magic(&[0x5B, 0x80, 0x54]));
        assert!(!is_ex_magic(&[0x00, 0x80, 0x54, 0x0A]));
        assert!(!is_ex_magic(&[]));
    }

    #[test]
    fn round_trip_load_write_load_byte_identical() {
        let dir = tmpdir("rt");
        let base = "_CL_deadbeef";
        let orig = synthetic();

        orig.write_to_dir(&dir, base).unwrap();
        let loaded = IlBundle::load_from_dir(&dir, base).unwrap();

        assert_eq!(loaded.base_name, base);
        assert_eq!(loaded.files, orig.files);
        assert!(loaded.has_ex_magic());

        // Second cycle must be byte-identical too.
        let dir2 = tmpdir("rt2");
        loaded.write_to_dir(&dir2, base).unwrap();
        let loaded2 = IlBundle::load_from_dir(&dir2, base).unwrap();
        assert_eq!(loaded2.files, orig.files);

        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&dir2).ok();
    }

    #[test]
    fn missing_files_are_skipped_not_errors() {
        let dir = tmpdir("partial");
        let base = "_CL_partial";
        let mut b = IlBundle::new(base);
        b.set("ex", EX_MAGIC.to_vec());
        b.write_to_dir(&dir, base).unwrap();

        let loaded = IlBundle::load_from_dir(&dir, base).unwrap();
        assert_eq!(loaded.files.len(), 1);
        assert!(loaded.get("ex").is_some());
        assert!(loaded.get("gl").is_none());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn token_width_heuristic() {
        let mut small = IlBundle::new("_CL_small");
        small.set("ex", EX_MAGIC.to_vec());
        assert_eq!(small.token_width(), 2);

        let mut big = IlBundle::new("_CL_big");
        big.set("ex", vec![0u8; 1024]);
        assert_eq!(big.token_width(), 4);
    }
}
