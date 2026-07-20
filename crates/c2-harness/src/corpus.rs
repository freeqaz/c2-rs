//! **P1.2 — corpus generator.** A reproducible tool that emits a corpus of
//! `(source, IL-bundle, obj)` triples by compiling generated C++ functions
//! through the real toolchain, capturing the c1xx→c2 IL bundle and the c2 obj,
//! and indexing every triple in a queryable manifest for the downstream
//! retrieval baseline (P1.3) and IL-space search (T-A).
//!
//! # What one triple is
//!
//! * **source** — a deterministically generated, self-contained `.cpp` of
//!   straight-line integer functions (the class the port + codec already
//!   handle: arithmetic chains over parameters and narrow/wide literals,
//!   single- and multi-function TUs).
//! * **IL bundle** — the surviving `_CL_*{ex,gl,sy,in,db}` files, captured with
//!   the P0.1 `strace`+`/Bd` recipe (real c2 runs → real obj, and the
//!   `unlink`-inject keeps the bundle). The codec's round-trippable
//!   [`IlModel`](c2_il::IlModel) is asserted per triple
//!   (`encode(parse(bundle)) == bundle`); a triple that fails the round-trip is
//!   flagged in the manifest, never silently stored.
//! * **obj** — the c2 object for that IL, stored timestamp-normalized (the COFF
//!   `TimeDateStamp` zeroed, [`ObjImage::normalized`]).
//!
//! # Determinism
//!
//! Source is a pure function of `(seed, index)` — a hand-rolled splitmix64
//! stream, no `Date.now()` / OS entropy. `gen_source(seed, i)` reproduces byte
//! for byte on re-run. The obj additionally embeds its `/Fo` output path
//! (MSVC bakes the path into `S_OBJNAME`), so obj bytes are a deterministic
//! function of `(source, output-path)` — reproducible for a fixed corpus root,
//! exactly as `oracle_selftest`'s determinism note documents.
//!
//! # Committability (K1 finding, binds here)
//!
//! A captured `.gl` embeds the host source path (`z:\home\…`; wibo maps
//! `/`→`Z:\`), and the obj embeds its output path — so **captured bundles/objs
//! are NOT committable**. The generated corpus dir is therefore a gitignored
//! artifact (`/corpus/` in `.gitignore`); only the generator, the schema doc
//! (`docs/CORPUS_MVP.md`), and a tiny hand-built *synthetic* sample
//! (`crates/c2-harness/tests/corpus_sample/`, path-free) are committed. The
//! generator regenerates the full corpus on demand.
//!
//! # Timeout
//!
//! Every toolchain invocation is bounded by a wall-clock deadline (P0.6(a): a
//! malformed IL fn-set can *hang* c2, not just crash). A capture that times out
//! is recorded as a `capture_timeout` skip and the run continues — never a hang
//! of the whole corpus.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use c2_il::{IlBundle, IlModel, Span};
use c2_obj::ObjImage;
use c2_reference::{to_wibo_path, Toolchain};

/// Bumped whenever the source-generation grammar changes in a way that alters
/// the emitted corpus for a fixed `(seed, index)`. Recorded in `config.json`.
pub const GENERATOR_VERSION: &str = "straightline_int_v1";

/// Default per-capture wall-clock timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

// ------------------------------------------------------------------------
// Deterministic source generation
// ------------------------------------------------------------------------

/// splitmix64 — a tiny, dependency-free deterministic mixer/PRNG.
fn mix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A deterministic value stream seeded by `(seed, index)`.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64, index: usize) -> Self {
        Rng {
            state: mix64(seed ^ mix64(index as u64).wrapping_add(0x1234_5678)),
        }
    }
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix64(self.state)
    }
    fn upto(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// A generated translation unit: its source text and the function names in it.
#[derive(Clone, Debug)]
pub struct GeneratedTu {
    pub source: String,
    pub functions: Vec<String>,
}

const PARAM_NAMES: [&str; 4] = ["a", "b", "c", "d"];
const OPS: [(&str, &str); 3] = [("+", "add"), ("-", "sub"), ("*", "mul")];

/// Generate the C++ translation unit for `(seed, index)` — a pure function of
/// its inputs (no entropy), so a re-run reproduces it byte for byte.
///
/// Each TU holds 1–3 straight-line integer functions; each function is a
/// fully-parenthesized left-associative chain of 1–4 binary ops (`+ - *`) over
/// its parameters and narrow (`1..=99`) / wide (`> 2^16`) integer literals.
pub fn gen_source(seed: u64, index: usize) -> GeneratedTu {
    let mut rng = Rng::new(seed, index);
    let n_fns = 1 + rng.upto(3) as usize; // 1..=3
    let mut functions = Vec::with_capacity(n_fns);
    let mut body = String::new();
    body.push_str(&format!(
        "// Generated straight-line int TU (seed={seed}, index={index}).\n\
         // Reproduce with: c2rs corpus gen --seed {seed} ...\n"
    ));
    for f in 0..n_fns {
        let name = format!("f{index}_{f}");
        let arity = 1 + rng.upto(4) as usize; // 1..=4 params
        let chain = 1 + rng.upto(4) as usize; // 1..=4 ops
        let params: Vec<&str> = PARAM_NAMES[..arity].to_vec();
        // First operand is always a parameter, so the body depends on its args.
        let mut expr = params[0].to_string();
        for _ in 0..chain {
            let (op, _) = OPS[rng.upto(3) as usize];
            // Operand: weight toward params (0,1 => param), else a literal.
            let operand = match rng.upto(4) {
                0 | 1 => params[rng.upto(arity as u64) as usize].to_string(),
                2 => format!("{}", 1 + rng.upto(99)), // narrow literal
                _ => format!("{}", 65536 + rng.upto(200_000)), // wide literal
            };
            expr = format!("({expr} {op} {operand})");
        }
        let sig_params = params
            .iter()
            .map(|p| format!("int {p}"))
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!("int {name}({sig_params}) {{ return {expr}; }}\n"));
        functions.push(name);
    }
    GeneratedTu {
        source: body,
        functions,
    }
}

// ------------------------------------------------------------------------
// Triple records + manifest
// ------------------------------------------------------------------------

/// Status of one triple in the manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TripleStatus {
    /// Captured, codec round-tripped, obj stored.
    Ok,
    /// The codec refused the captured bundle (round-trip failed) — flagged, and
    /// the raw bundle is still stored for inspection.
    CodecFail,
    /// The capture exceeded the wall-clock deadline (recorded as a skip).
    CaptureTimeout,
    /// The capture/compile errored (toolchain I/O, empty bundle, …).
    CaptureError,
}

impl TripleStatus {
    fn as_str(&self) -> &'static str {
        match self {
            TripleStatus::Ok => "ok",
            TripleStatus::CodecFail => "codec_fail",
            TripleStatus::CaptureTimeout => "capture_timeout",
            TripleStatus::CaptureError => "capture_error",
        }
    }
}

/// One manifest row: everything P1.3 retrieval / T-A seeding needs to locate and
/// key on a triple, plus the codec fidelity stats for the IL side.
#[derive(Clone, Debug)]
pub struct TripleRecord {
    pub id: String,
    pub index: usize,
    pub seed: u64,
    pub status: TripleStatus,
    pub error: Option<String>,
    pub source_rel: String,
    pub source_sha256: String,
    pub functions: Vec<String>,
    // IL side (present on ok / codec_fail).
    pub il_dir_rel: Option<String>,
    pub il_base: Option<String>,
    pub il_files: Vec<(String, usize)>, // (suffix, len) in canonical order
    pub codec_roundtrip: Option<bool>,
    pub ex_token_count: Option<usize>,
    pub ex_typed_bytes: Option<usize>,
    pub ex_opaque_bytes: Option<usize>,
    pub gl_offsets: Option<Vec<u32>>,
    // obj side (present on ok).
    pub obj_rel: Option<String>,
    pub obj_len: Option<usize>,
    pub obj_sha256_norm: Option<String>,
}

impl TripleRecord {
    /// Serialize to a single compact JSON object (one manifest line).
    pub fn to_json(&self) -> String {
        let mut j = JsonWriter::new();
        j.begin();
        j.str("id", &self.id);
        j.num("index", self.index as i64);
        j.num("seed", self.seed as i64);
        j.raw("status", &jstr(self.status.as_str()));
        if let Some(e) = &self.error {
            j.str("error", e);
        }
        j.str("source_rel", &self.source_rel);
        j.str("source_sha256", &self.source_sha256);
        j.str_array("functions", &self.functions);
        if let Some(d) = &self.il_dir_rel {
            j.str("il_dir_rel", d);
        }
        if let Some(b) = &self.il_base {
            j.str("il_base", b);
        }
        if !self.il_files.is_empty() {
            let obj = self
                .il_files
                .iter()
                .map(|(s, n)| format!("{}:{}", jstr(s), n))
                .collect::<Vec<_>>()
                .join(",");
            j.raw("il_files", &format!("{{{obj}}}"));
        }
        if let Some(rt) = self.codec_roundtrip {
            j.raw("codec_roundtrip", if rt { "true" } else { "false" });
        }
        if let Some(n) = self.ex_token_count {
            j.num("ex_token_count", n as i64);
        }
        if let Some(n) = self.ex_typed_bytes {
            j.num("ex_typed_bytes", n as i64);
        }
        if let Some(n) = self.ex_opaque_bytes {
            j.num("ex_opaque_bytes", n as i64);
        }
        if let Some(offs) = &self.gl_offsets {
            let arr = offs
                .iter()
                .map(|o| o.to_string())
                .collect::<Vec<_>>()
                .join(",");
            j.raw("gl_offsets", &format!("[{arr}]"));
        }
        if let Some(o) = &self.obj_rel {
            j.str("obj_rel", o);
        }
        if let Some(n) = self.obj_len {
            j.num("obj_len", n as i64);
        }
        if let Some(h) = &self.obj_sha256_norm {
            j.str("obj_sha256_norm", h);
        }
        j.end();
        j.done()
    }
}

/// Summary of a generation run.
#[derive(Clone, Debug, Default)]
pub struct CorpusSummary {
    pub ok: usize,
    pub codec_fail: usize,
    pub timeout: usize,
    pub error: usize,
    pub distinct_sources: usize,
}

impl CorpusSummary {
    pub fn total(&self) -> usize {
        self.ok + self.codec_fail + self.timeout + self.error
    }
}

/// Config for a generation run.
#[derive(Clone, Debug)]
pub struct CorpusConfig {
    pub seed: u64,
    pub count: usize,
    pub timeout: Duration,
}

impl Default for CorpusConfig {
    fn default() -> Self {
        CorpusConfig {
            seed: 0,
            count: 32,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

// ------------------------------------------------------------------------
// Generation (toolchain path)
// ------------------------------------------------------------------------

/// Generate a corpus under `root` using the real toolchain `tc`.
///
/// Enumerates `(seed, index)` from `index = 0`, dedups by source text so the
/// corpus holds exactly `cfg.count` **distinct** TUs (skipping the rare
/// enumeration collision), captures each through the P0.1 recipe, stores the
/// triple, and appends a manifest row. Requires strace (checked by the caller);
/// individual captures degrade to a flagged skip, never a run-fatal hang.
pub fn generate(root: &Path, tc: &Toolchain, cfg: &CorpusConfig) -> std::io::Result<CorpusSummary> {
    let triples_dir = root.join("triples");
    std::fs::create_dir_all(&triples_dir)?;
    let manifest_path = root.join("manifest.jsonl");
    let mut manifest = std::fs::File::create(&manifest_path)?;
    write_config(root, cfg)?;

    let mut summary = CorpusSummary::default();
    let mut seen_sources: BTreeSet<String> = BTreeSet::new();
    let mut index = 0usize;
    // Bound the enumeration so a degenerate collision run cannot loop forever.
    let max_index = cfg.count.saturating_mul(16).max(cfg.count + 64);

    while summary.total() < cfg.count && index < max_index {
        let tu = gen_source(cfg.seed, index);
        let idx = index;
        index += 1;
        if !seen_sources.insert(tu.source.clone()) {
            continue; // duplicate TU — skip without consuming a corpus slot
        }
        summary.distinct_sources += 1;

        let id = format!("t{:05}", summary.total());
        let record = build_triple(root, &triples_dir, &id, idx, cfg, tc, &tu)?;
        match record.status {
            TripleStatus::Ok => summary.ok += 1,
            TripleStatus::CodecFail => summary.codec_fail += 1,
            TripleStatus::CaptureTimeout => summary.timeout += 1,
            TripleStatus::CaptureError => summary.error += 1,
        }
        writeln!(manifest, "{}", record.to_json())?;
    }
    Ok(summary)
}

/// Capture + store one triple, returning its manifest record. Writes the source
/// unconditionally; writes the IL bundle + obj on a successful capture.
fn build_triple(
    root: &Path,
    triples_dir: &Path,
    id: &str,
    index: usize,
    cfg: &CorpusConfig,
    tc: &Toolchain,
    tu: &GeneratedTu,
) -> std::io::Result<TripleRecord> {
    let tdir = triples_dir.join(id);
    std::fs::create_dir_all(&tdir)?;
    let src_path = tdir.join("source.cpp");
    std::fs::write(&src_path, tu.source.as_bytes())?;
    let source_rel = format!("triples/{id}/source.cpp");
    let source_sha = sha256_hex(tu.source.as_bytes());

    let mut rec = TripleRecord {
        id: id.to_string(),
        index,
        seed: cfg.seed,
        status: TripleStatus::CaptureError,
        error: None,
        source_rel,
        source_sha256: source_sha,
        functions: tu.functions.clone(),
        il_dir_rel: None,
        il_base: None,
        il_files: Vec::new(),
        codec_roundtrip: None,
        ex_token_count: None,
        ex_typed_bytes: None,
        ex_opaque_bytes: None,
        gl_offsets: None,
        obj_rel: None,
        obj_len: None,
        obj_sha256_norm: None,
    };

    let work = tdir.join("_work");
    let outcome = capture_triple(tc, &src_path, &work, cfg.timeout);
    // The scratch work dir is not part of the corpus artifact.
    let _ = std::fs::remove_dir_all(&work);

    match outcome {
        CaptureOutcome::Timeout => {
            rec.status = TripleStatus::CaptureTimeout;
            rec.error = Some("capture exceeded wall-clock deadline".into());
        }
        CaptureOutcome::Error(msg) => {
            rec.status = TripleStatus::CaptureError;
            rec.error = Some(first_line(&msg).to_string());
        }
        CaptureOutcome::Ok { bundle, obj_bytes } => {
            store_triple(&tdir, id, root, &mut rec, &bundle, &obj_bytes)?;
        }
    }
    Ok(rec)
}

/// Persist a captured bundle + obj into the triple dir and fill in `rec`.
fn store_triple(
    tdir: &Path,
    id: &str,
    _root: &Path,
    rec: &mut TripleRecord,
    bundle: &IlBundle,
    obj_bytes: &[u8],
) -> std::io::Result<()> {
    // IL bundle (raw). Dir is inside the gitignored corpus tree.
    let il_dir = tdir.join("il");
    bundle.write_to_dir(&il_dir, &bundle.base_name)?;
    rec.il_dir_rel = Some(format!("triples/{id}/il"));
    rec.il_base = Some(bundle.base_name.clone());
    rec.il_files = canonical_files(bundle);

    // obj, timestamp-normalized. Stored as `obj.bin` (not `*.obj`, so a scrubbed
    // sample can be committed without tripping the `*.obj` gitignore rule).
    let obj_norm = ObjImage::new(obj_bytes.to_vec()).normalized();
    let obj_path = tdir.join("obj.bin");
    std::fs::write(&obj_path, &obj_norm)?;
    rec.obj_rel = Some(format!("triples/{id}/obj.bin"));
    rec.obj_len = Some(obj_norm.len());
    rec.obj_sha256_norm = Some(sha256_hex(&obj_norm));

    // Codec fidelity: parse (fail-closed re-encode inside), record stats.
    fill_codec_stats(rec, bundle);
    Ok(())
}

/// Parse the bundle through the K1 codec and record round-trip + coverage stats.
fn fill_codec_stats(rec: &mut TripleRecord, bundle: &IlBundle) {
    match IlModel::parse(bundle) {
        Ok(model) => {
            // Round-trip is guaranteed by `parse` (fail-closed), but assert the
            // encode is byte-identical here too, defensively.
            let rt = model.encode().files == bundle.files;
            rec.codec_roundtrip = Some(rt);
            rec.status = if rt {
                TripleStatus::Ok
            } else {
                TripleStatus::CodecFail
            };
            rec.ex_token_count = Some(model.ex_tokens().len());
            rec.gl_offsets = Some(model.gl_body_start_offsets());
            let (typed, opaque) = ex_typed_opaque(&model);
            rec.ex_typed_bytes = Some(typed);
            rec.ex_opaque_bytes = Some(opaque);
        }
        Err(e) => {
            rec.codec_roundtrip = Some(false);
            rec.status = TripleStatus::CodecFail;
            rec.error = Some(format!("codec: {e}"));
        }
    }
}

/// (typed, opaque) byte counts for the `.ex` file of a parsed model. Opaque
/// bytes are summed directly from [`Span::Opaque`]; typed = file len − opaque.
fn ex_typed_opaque(model: &IlModel) -> (usize, usize) {
    for fm in &model.files {
        if fm.suffix == "ex" {
            let total: usize = fm.encode().len();
            let opaque: usize = fm
                .spans
                .iter()
                .map(|s| match s {
                    Span::Opaque(b) => b.len(),
                    _ => 0,
                })
                .sum();
            return (total.saturating_sub(opaque), opaque);
        }
    }
    (0, 0)
}

/// The bundle's present files in canonical suffix order, with byte lengths.
fn canonical_files(bundle: &IlBundle) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for suffix in c2_il::IL_SUFFIXES {
        if let Some(b) = bundle.get(suffix) {
            out.push((suffix.to_string(), b.len()));
        }
    }
    out
}

// ------------------------------------------------------------------------
// Timeout-bounded capture (P0.1 recipe: strace + /Bd → real obj + surviving IL)
// ------------------------------------------------------------------------

enum CaptureOutcome {
    Ok { bundle: IlBundle, obj_bytes: Vec<u8> },
    Timeout,
    Error(String),
}

/// Run one `/Bd /Ox /GS- /c` pipeline compile under `strace` (with
/// `unlink`/`unlinkat` injected to no-op) so **c2 runs for real** — producing
/// the obj at `<work>/out.obj` — **and** the `_CL_*` IL bundle survives. Bounded
/// by `timeout`: on expiry the child is killed and [`CaptureOutcome::Timeout`]
/// is returned.
///
/// Mirrors the recipe in `c2-reference::Toolchain::capture_reference` but adds
/// the wall-clock kill and stores only what a corpus triple needs (bundle +
/// obj); it does not scrape the c2 argv (unneeded here).
fn capture_triple(tc: &Toolchain, cpp: &Path, work: &Path, timeout: Duration) -> CaptureOutcome {
    let strace = match &tc.strace {
        Some(s) => s.clone(),
        None => return CaptureOutcome::Error("strace absent".into()),
    };
    if let Err(e) = std::fs::create_dir_all(work) {
        return CaptureOutcome::Error(format!("create work dir: {e}"));
    }
    let work_abs = match std::fs::canonicalize(work) {
        Ok(p) => p,
        Err(e) => return CaptureOutcome::Error(format!("canonicalize work: {e}")),
    };
    let cpp_abs = match std::fs::canonicalize(cpp) {
        Ok(p) => p,
        Err(e) => return CaptureOutcome::Error(format!("canonicalize src: {e}")),
    };
    let out_obj = work_abs.join("out.obj");
    let _ = std::fs::remove_file(&out_obj);

    let z_src = to_wibo_path(&cpp_abs);
    let z_obj = to_wibo_path(&out_obj);

    let mut child = match Command::new(&strace)
        .arg("-f")
        .arg("-e")
        .arg("trace=unlink,unlinkat")
        .arg("-e")
        .arg("inject=unlink,unlinkat:retval=0")
        .arg("-o")
        .arg("/dev/null")
        .arg(&tc.wibo)
        .arg(&tc.cl_exe)
        .arg("/Bd")
        .arg("/Ox")
        .arg("/GS-")
        .arg("/c")
        .arg(format!("/Fo{z_obj}"))
        .arg(&z_src)
        .env("TMP", &work_abs)
        .env("TEMP", &work_abs)
        .env("WIBO_FS_CACHE", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return CaptureOutcome::Error(format!("spawn strace/wibo: {e}")),
    };

    // Poll until the child exits or the deadline passes.
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return CaptureOutcome::Timeout;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return CaptureOutcome::Error(format!("wait child: {e}")),
        }
    }

    if !out_obj.exists() {
        return CaptureOutcome::Error(format!("no obj produced at {}", out_obj.display()));
    }
    let obj_bytes = match std::fs::read(&out_obj) {
        Ok(b) => b,
        Err(e) => return CaptureOutcome::Error(format!("read obj: {e}")),
    };

    let base = match find_bundle_base(&work_abs) {
        Some(b) => b,
        None => return CaptureOutcome::Error("no surviving _CL_*ex bundle".into()),
    };
    let bundle = match IlBundle::load_from_dir(&work_abs, &base) {
        Ok(b) => b,
        Err(e) => return CaptureOutcome::Error(format!("load bundle: {e}")),
    };
    match bundle.ex() {
        Some(ex) if !ex.is_empty() => CaptureOutcome::Ok { bundle, obj_bytes },
        _ => CaptureOutcome::Error(format!("captured bundle {base} has empty .ex")),
    }
}

/// The surviving IL bundle base in `dir`: the first `_CL_<hex>ex` file, `ex`
/// stripped. Deterministic (smallest name) when several are present.
fn find_bundle_base(dir: &Path) -> Option<String> {
    let mut found: Option<String> = None;
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("_CL_") && name.ends_with("ex") {
            let base = name[..name.len() - 2].to_string();
            match &found {
                Some(prev) if prev <= &base => {}
                _ => found = Some(base),
            }
        }
    }
    found
}

// ------------------------------------------------------------------------
// config.json
// ------------------------------------------------------------------------

fn write_config(root: &Path, cfg: &CorpusConfig) -> std::io::Result<()> {
    let mut j = JsonWriter::new();
    j.begin();
    j.str("generator", GENERATOR_VERSION);
    j.num("seed", cfg.seed as i64);
    j.num("count", cfg.count as i64);
    j.num("timeout_secs", cfg.timeout.as_secs() as i64);
    j.end();
    std::fs::write(root.join("config.json"), j.done())
}

// ------------------------------------------------------------------------
// Manifest reader (minimal, for the portable sample test + `stats`)
// ------------------------------------------------------------------------

/// A parsed manifest row (only the fields consumers key on). Absent fields are
/// `None`.
#[derive(Clone, Debug, Default)]
pub struct ManifestRow {
    pub id: String,
    pub status: String,
    pub source_rel: Option<String>,
    pub il_dir_rel: Option<String>,
    pub il_base: Option<String>,
    pub il_files: Vec<(String, i64)>,
    pub codec_roundtrip: Option<bool>,
    pub ex_token_count: Option<i64>,
    pub gl_offsets: Vec<i64>,
    pub obj_rel: Option<String>,
    pub obj_len: Option<i64>,
    pub obj_sha256_norm: Option<String>,
}

/// Load and parse `<root>/manifest.jsonl`.
pub fn load_manifest(root: &Path) -> std::io::Result<Vec<ManifestRow>> {
    let text = std::fs::read_to_string(root.join("manifest.jsonl"))?;
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let obj = json::parse(line)
            .and_then(|v| v.into_object())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "bad manifest JSON line")
            })?;
        let mut row = ManifestRow::default();
        for (k, v) in obj {
            match k.as_str() {
                "id" => row.id = v.as_str().unwrap_or_default().to_string(),
                "status" => row.status = v.as_str().unwrap_or_default().to_string(),
                "source_rel" => row.source_rel = v.as_str().map(str::to_string),
                "il_dir_rel" => row.il_dir_rel = v.as_str().map(str::to_string),
                "il_base" => row.il_base = v.as_str().map(str::to_string),
                "il_files" => {
                    if let json::Json::Obj(pairs) = v {
                        for (fk, fv) in pairs {
                            if let Some(n) = fv.as_i64() {
                                row.il_files.push((fk, n));
                            }
                        }
                    }
                }
                "codec_roundtrip" => row.codec_roundtrip = v.as_bool(),
                "ex_token_count" => row.ex_token_count = v.as_i64(),
                "gl_offsets" => {
                    if let json::Json::Arr(items) = v {
                        row.gl_offsets = items.iter().filter_map(|x| x.as_i64()).collect();
                    }
                }
                "obj_rel" => row.obj_rel = v.as_str().map(str::to_string),
                "obj_len" => row.obj_len = v.as_i64(),
                "obj_sha256_norm" => row.obj_sha256_norm = v.as_str().map(str::to_string),
                _ => {}
            }
        }
        rows.push(row);
    }
    Ok(rows)
}

// ------------------------------------------------------------------------
// Synthetic sample (committed, portable, path-free)
// ------------------------------------------------------------------------

/// A hand-built, path-free synthetic bundle that decodes to typed tokens — the
/// committable stand-in for a real capture (a real `.gl` embeds a host path).
/// Mirrors the shape of a one-function add-chain body so the codec exposes an
/// `.ex` token walk and one `.gl` body-start offset.
pub fn synthetic_bundle(base: &str) -> IlBundle {
    let mut b = IlBundle::new(base.to_string());

    // .ex: magic + opaque header, then one function: `4F 1F` start, opaque
    // metadata, formals `46 (2D <tok>)*`, `4C 4F 11` LO, a small typed body,
    // then the function-tail + module-end tokens.
    let mut ex: Vec<u8> = Vec::new();
    ex.extend_from_slice(&c2_il::EX_MAGIC); // 5B 80 54 0A
    ex.extend_from_slice(&[0x00; 8]); // opaque header filler
    let fn_start = ex.len() as u32; // offset of the 4F 1F marker
    ex.extend_from_slice(&[0x4F, 0x1F]); // function start
    ex.extend_from_slice(&[0x11, 0x22]); // opaque per-function metadata
    ex.push(0x46); // Formals
    ex.extend_from_slice(&[0x2D, 0xE3, 0x01]); // Formal(0xE301)
    ex.extend_from_slice(&[0x2D, 0xE3, 0x02]); // Formal(0xE302)
    ex.extend_from_slice(&[0x4C, 0x4F, 0x11]); // LO marker
    ex.push(0x53); // Ss (statement start)
    ex.extend_from_slice(&[0xB9, 0xE3, 0x01, 0x86, 0x41, 0x74]); // Load(0xE301)
    ex.extend_from_slice(&[0xB9, 0xE3, 0x02, 0x86, 0x41, 0x74]); // Load(0xE302)
    ex.push(0x02); // Add
    ex.extend_from_slice(&[0x54, 0x02, 0x29, 0xE3, 0x00]); // Return(0xE300)
    ex.extend_from_slice(&[0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00]); // FnTail
    ex.extend_from_slice(&[0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x00, 0x4D]); // ModuleEnd(0)
    b.set("ex", ex);

    // .gl: opaque name bytes, then the record framing the codec keys the
    // body-start offset field off of (`80 XX 10 00 00 00 00`), then the
    // `80 <LE32>` offset pointing at the `.ex` 4F 1F. No host path — that is the
    // whole point of the synthetic form (a real `.gl` embeds `z:\home\…`).
    let mut gl: Vec<u8> = Vec::new();
    gl.extend_from_slice(b"?fadd@@YAHHH@Z\x00"); // scrubbed in-class mangled name
    gl.extend_from_slice(&[0x80, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00]); // record frame
    gl.push(0x80);
    gl.extend_from_slice(&fn_start.to_le_bytes()); // body-start offset field
    b.set("gl", gl);

    // The remaining files are opaque to the codec; small path-free fillers.
    b.set("sy", b"a\x00b\x00\x00".to_vec());
    b.set("in", vec![0x86, 0x41, 0x74, 0x00]);
    b.set("db", Vec::new());
    b
}

/// A synthetic obj: a minimal COFF-ish blob (machine word, section count, a
/// timestamp we will normalize, then filler). Path-free and stable.
fn synthetic_obj() -> Vec<u8> {
    let mut v = vec![0xF2, 0x01, 0x02, 0x00]; // POWERPCBE machine + 2 sections
    v.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // TimeDateStamp (normalized away)
    v.extend_from_slice(b"synthetic-c2-obj-payload\x00\x00");
    v
}

/// Write the committed synthetic sample corpus to `root` using the exact same
/// record/manifest code path as [`generate`] — so the sample's schema can never
/// drift from a real run. No toolchain required; fully deterministic.
pub fn write_synthetic_sample(root: &Path) -> std::io::Result<CorpusSummary> {
    let triples_dir = root.join("triples");
    std::fs::create_dir_all(&triples_dir)?;
    let mut manifest = std::fs::File::create(root.join("manifest.jsonl"))?;
    let cfg = CorpusConfig {
        seed: 0,
        count: 2,
        timeout: DEFAULT_TIMEOUT,
    };
    write_config(root, &cfg)?;

    let mut summary = CorpusSummary::default();
    for i in 0..2usize {
        let id = format!("t{i:05}");
        let tu = gen_source(0, i);
        let tdir = triples_dir.join(&id);
        std::fs::create_dir_all(&tdir)?;
        std::fs::write(tdir.join("source.cpp"), tu.source.as_bytes())?;

        let mut rec = TripleRecord {
            id: id.clone(),
            index: i,
            seed: 0,
            status: TripleStatus::CaptureError,
            error: None,
            source_rel: format!("triples/{id}/source.cpp"),
            source_sha256: sha256_hex(tu.source.as_bytes()),
            functions: tu.functions.clone(),
            il_dir_rel: None,
            il_base: None,
            il_files: Vec::new(),
            codec_roundtrip: None,
            ex_token_count: None,
            ex_typed_bytes: None,
            ex_opaque_bytes: None,
            gl_offsets: None,
            obj_rel: None,
            obj_len: None,
            obj_sha256_norm: None,
        };
        let base = format!("sample{i:02}"); // NOT `_CL_*` (would be gitignored)
        let bundle = synthetic_bundle(&base);
        let obj = synthetic_obj();
        store_triple(&tdir, &id, root, &mut rec, &bundle, &obj)?;
        summary.distinct_sources += 1;
        match rec.status {
            TripleStatus::Ok => summary.ok += 1,
            TripleStatus::CodecFail => summary.codec_fail += 1,
            TripleStatus::CaptureTimeout => summary.timeout += 1,
            TripleStatus::CaptureError => summary.error += 1,
        }
        writeln!(manifest, "{}", rec.to_json())?;
    }
    Ok(summary)
}

// ------------------------------------------------------------------------
// helpers
// ------------------------------------------------------------------------

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

/// JSON-escape and quote a string.
pub(crate) fn jstr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A tiny compact JSON-object writer.
struct JsonWriter {
    buf: String,
    first: bool,
}
impl JsonWriter {
    fn new() -> Self {
        JsonWriter {
            buf: String::new(),
            first: true,
        }
    }
    fn begin(&mut self) {
        self.buf.push('{');
        self.first = true;
    }
    fn sep(&mut self) {
        if self.first {
            self.first = false;
        } else {
            self.buf.push(',');
        }
    }
    fn str(&mut self, key: &str, val: &str) {
        self.sep();
        self.buf.push_str(&jstr(key));
        self.buf.push(':');
        self.buf.push_str(&jstr(val));
    }
    fn num(&mut self, key: &str, val: i64) {
        self.sep();
        self.buf.push_str(&jstr(key));
        self.buf.push(':');
        self.buf.push_str(&val.to_string());
    }
    fn raw(&mut self, key: &str, raw: &str) {
        self.sep();
        self.buf.push_str(&jstr(key));
        self.buf.push(':');
        self.buf.push_str(raw);
    }
    fn str_array(&mut self, key: &str, vals: &[String]) {
        let arr = vals.iter().map(|s| jstr(s)).collect::<Vec<_>>().join(",");
        self.raw(key, &format!("[{arr}]"));
    }
    fn end(&mut self) {
        self.buf.push('}');
    }
    fn done(self) -> String {
        self.buf
    }
}

/// A minimal recursive-descent JSON parser — just enough for the manifest lines
/// (objects, arrays, strings, integers, bools, null). No floats. std-only.
pub(crate) mod json {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Json {
        Str(String),
        Int(i64),
        Bool(bool),
        Null,
        Arr(Vec<Json>),
        Obj(Vec<(String, Json)>),
    }

    impl Json {
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Json::Str(s) => Some(s),
                _ => None,
            }
        }
        pub fn as_i64(&self) -> Option<i64> {
            match self {
                Json::Int(n) => Some(*n),
                _ => None,
            }
        }
        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Json::Bool(b) => Some(*b),
                _ => None,
            }
        }
        pub fn into_object(self) -> Option<Vec<(String, Json)>> {
            match self {
                Json::Obj(o) => Some(o),
                _ => None,
            }
        }
    }

    pub fn parse(s: &str) -> Option<Json> {
        let bytes: Vec<char> = s.chars().collect();
        let mut p = Parser { b: bytes, i: 0 };
        p.ws();
        let v = p.value()?;
        p.ws();
        if p.i == p.b.len() {
            Some(v)
        } else {
            None
        }
    }

    struct Parser {
        b: Vec<char>,
        i: usize,
    }
    impl Parser {
        fn peek(&self) -> Option<char> {
            self.b.get(self.i).copied()
        }
        fn ws(&mut self) {
            while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
                self.i += 1;
            }
        }
        fn eat(&mut self, c: char) -> bool {
            if self.peek() == Some(c) {
                self.i += 1;
                true
            } else {
                false
            }
        }
        fn value(&mut self) -> Option<Json> {
            self.ws();
            match self.peek()? {
                '"' => self.string().map(Json::Str),
                '{' => self.object(),
                '[' => self.array(),
                't' | 'f' => self.boolean(),
                'n' => {
                    for c in ['n', 'u', 'l', 'l'] {
                        if !self.eat(c) {
                            return None;
                        }
                    }
                    Some(Json::Null)
                }
                _ => self.number(),
            }
        }
        fn string(&mut self) -> Option<String> {
            if !self.eat('"') {
                return None;
            }
            let mut out = String::new();
            loop {
                let c = self.peek()?;
                self.i += 1;
                match c {
                    '"' => return Some(out),
                    '\\' => {
                        let e = self.peek()?;
                        self.i += 1;
                        match e {
                            '"' => out.push('"'),
                            '\\' => out.push('\\'),
                            '/' => out.push('/'),
                            'n' => out.push('\n'),
                            'r' => out.push('\r'),
                            't' => out.push('\t'),
                            'u' => {
                                let mut code = 0u32;
                                for _ in 0..4 {
                                    let h = self.peek()?;
                                    self.i += 1;
                                    code = code * 16 + h.to_digit(16)?;
                                }
                                out.push(char::from_u32(code)?);
                            }
                            _ => return None,
                        }
                    }
                    c => out.push(c),
                }
            }
        }
        fn number(&mut self) -> Option<Json> {
            let start = self.i;
            if self.peek() == Some('-') {
                self.i += 1;
            }
            while matches!(self.peek(), Some('0'..='9')) {
                self.i += 1;
            }
            let s: String = self.b[start..self.i].iter().collect();
            s.parse::<i64>().ok().map(Json::Int)
        }
        fn boolean(&mut self) -> Option<Json> {
            if self.peek() == Some('t') {
                for c in ['t', 'r', 'u', 'e'] {
                    if !self.eat(c) {
                        return None;
                    }
                }
                Some(Json::Bool(true))
            } else {
                for c in ['f', 'a', 'l', 's', 'e'] {
                    if !self.eat(c) {
                        return None;
                    }
                }
                Some(Json::Bool(false))
            }
        }
        fn array(&mut self) -> Option<Json> {
            self.eat('[');
            let mut items = Vec::new();
            self.ws();
            if self.eat(']') {
                return Some(Json::Arr(items));
            }
            loop {
                items.push(self.value()?);
                self.ws();
                if self.eat(']') {
                    return Some(Json::Arr(items));
                }
                if !self.eat(',') {
                    return None;
                }
            }
        }
        fn object(&mut self) -> Option<Json> {
            self.eat('{');
            let mut pairs = Vec::new();
            self.ws();
            if self.eat('}') {
                return Some(Json::Obj(pairs));
            }
            loop {
                self.ws();
                let key = self.string()?;
                self.ws();
                if !self.eat(':') {
                    return None;
                }
                let val = self.value()?;
                pairs.push((key, val));
                self.ws();
                if self.eat('}') {
                    return Some(Json::Obj(pairs));
                }
                if !self.eat(',') {
                    return None;
                }
            }
        }
    }
}

// ------------------------------------------------------------------------
// SHA-256 (std-only; the workspace forbids external crates)
// ------------------------------------------------------------------------

/// Lowercase-hex SHA-256 of `data`. Small, self-contained; used for content
/// digests in the manifest (integrity + dedup keys for retrieval).
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, wi) in w.iter_mut().enumerate().take(16) {
            *wi = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ ((!v[4]) & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }

    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{word:08x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn gen_source_is_deterministic() {
        for i in 0..64 {
            let a = gen_source(0, i);
            let b = gen_source(0, i);
            assert_eq!(a.source, b.source, "source not reproducible at index {i}");
            assert_eq!(a.functions, b.functions);
            assert!(!a.functions.is_empty());
            assert!(a.source.contains("int f"));
            assert!(a.source.contains("return"));
        }
    }

    #[test]
    fn gen_source_varies_across_seeds_and_indices() {
        // Enough distinct TUs in the first 128 indices to be a real corpus.
        let mut set = BTreeSet::new();
        for i in 0..128 {
            set.insert(gen_source(0, i).source);
        }
        assert!(
            set.len() >= 64,
            "expected many distinct TUs, got {}",
            set.len()
        );
        // A different seed produces a different stream.
        assert_ne!(gen_source(0, 0).source, gen_source(1, 0).source);
    }

    #[test]
    fn synthetic_bundle_roundtrips_and_is_path_free() {
        let b = synthetic_bundle("sample00");
        let model = IlModel::parse(&b).expect("codec parse");
        assert_eq!(model.encode().files, b.files, "synthetic must round-trip");
        assert!(!model.ex_tokens().is_empty());
        assert_eq!(model.gl_body_start_offsets().len(), 1);
        for bytes in b.files.values() {
            let lower = String::from_utf8_lossy(bytes).to_lowercase();
            assert!(!lower.contains("z:\\"), "synthetic bundle must be path-free");
            assert!(!lower.contains("/home/"), "synthetic bundle must be path-free");
        }
    }

    #[test]
    fn json_roundtrips_a_manifest_row() {
        let rec = TripleRecord {
            id: "t00000".into(),
            index: 0,
            seed: 0,
            status: TripleStatus::Ok,
            error: None,
            source_rel: "triples/t00000/source.cpp".into(),
            source_sha256: sha256_hex(b"x"),
            functions: vec!["f0_0".into(), "f0_1".into()],
            il_dir_rel: Some("triples/t00000/il".into()),
            il_base: Some("_CL_abc".into()),
            il_files: vec![("ex".into(), 100), ("gl".into(), 20)],
            codec_roundtrip: Some(true),
            ex_token_count: Some(7),
            ex_typed_bytes: Some(30),
            ex_opaque_bytes: Some(70),
            gl_offsets: Some(vec![12, 44]),
            obj_rel: Some("triples/t00000/obj.bin".into()),
            obj_len: Some(842),
            obj_sha256_norm: Some(sha256_hex(b"y")),
        };
        let line = rec.to_json();
        let parsed = json::parse(&line).and_then(|v| v.into_object()).unwrap();
        let get = |k: &str| parsed.iter().find(|(key, _)| key == k).map(|(_, v)| v);
        assert_eq!(get("id").unwrap().as_str(), Some("t00000"));
        assert_eq!(get("status").unwrap().as_str(), Some("ok"));
        assert_eq!(get("codec_roundtrip").unwrap().as_bool(), Some(true));
        assert_eq!(get("obj_len").unwrap().as_i64(), Some(842));
        assert!(matches!(get("gl_offsets").unwrap(), json::Json::Arr(_)));
        assert!(matches!(get("il_files").unwrap(), json::Json::Obj(_)));
    }

    #[test]
    fn write_synthetic_sample_is_loadable_and_consistent() {
        let dir = std::env::temp_dir().join(format!(
            "c2rs-corpus-sample-test-{}-{}",
            std::process::id(),
            SAMPLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let summary = write_synthetic_sample(&dir).unwrap();
        assert_eq!(summary.ok, 2);
        let rows = load_manifest(&dir).unwrap();
        assert_eq!(rows.len(), 2);
        for row in &rows {
            assert_eq!(row.status, "ok");
            assert_eq!(row.codec_roundtrip, Some(true));
            // The recorded IL + obj files exist and match their manifest sizes.
            let il_dir = dir.join(row.il_dir_rel.as_ref().unwrap());
            let base = row.il_base.as_ref().unwrap();
            for (suffix, len) in &row.il_files {
                let p = il_dir.join(format!("{base}{suffix}"));
                assert_eq!(std::fs::metadata(&p).unwrap().len() as i64, *len);
            }
            let obj = dir.join(row.obj_rel.as_ref().unwrap());
            let bytes = std::fs::read(&obj).unwrap();
            assert_eq!(bytes.len() as i64, row.obj_len.unwrap());
            assert_eq!(sha256_hex(&bytes), row.obj_sha256_norm.clone().unwrap());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    static SAMPLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
}
