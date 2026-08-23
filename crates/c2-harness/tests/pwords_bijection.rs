//! **`w-pwords` — can `.pdata`'s `prolog_words` REPAIR S1's demoted bijection?**
//!
//! `docs/ROADMAP_SLICING_2026-08-21.md` §5's **AMENDED** block demoted
//! `w-ildecode`'s `the_final_tuple_order_reproduces_the_text_words` from an
//! *equality* to a *per-function ratio*, on the ground that the final expansion
//! rewrites the prologue pseudo-op into a word count nobody could predict.
//! Board **#3431** found the count **is written into the object**: `.pdata`'s
//! unwind word carries `prolog_words` in its low 8 bits
//! (`crates/c2-core/src/coff/pdata.rs:71`). This file is the seam that
//! confronts the port-side tuple count with that oracle-side field.
//!
//! **WHAT THIS IS NOT.** It is a *measurement seam*. It licenses no emit, it is
//! never a gate, and it never stands in for the byte judge — real `c2` under
//! wibo plus a byte-exact obj compare (`CLAUDE.md` § "The one correctness
//! rule"). Its output is characterization under goal (1), nothing else.
//!
//! **THE THING NOBODY HAD DONE.** The bijection's graded population is *three
//! functions, nine words, all leaf, all frameless* — the AMENDED block says so
//! itself. So the equality had **never been evaluated on a single framed
//! function**, and its breakage was a *prediction*. `h0_*` below is that
//! measurement.
//!
//! Prereg: `docs/rungs/_2026-08-23-w-pwords-prereg.md` (frozen `c518532c7`,
//! before any of this ran). The three forms, verbatim from §2.2:
//!
//! ```text
//!   H0   T(f)                        == W(f)      the demoted equality itself
//!   H1   T(f) - I(f) + P(f)          == W(f)      the prologue correction
//!   H2   R(f) = W(f) - (T(f) - I(f) + P(f))       the residual: what .pdata CANNOT see
//! ```
//!
//! `R` carries the **ungraded epilogue term** plus every other unbounded
//! expansion arm (`nopalign 0x27b`, `0x2e5`, `retaddr 0x28f`, and the
//! long-branch `bc`). This file **publishes `R`'s distribution and does not fit
//! a constant to it.**

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use c2_obj::ObjImage;
use c2_reference::stage::STAGE_SITES;
use c2_reference::Toolchain;

/// The capture profile. Identical to the one the bijection itself uses
/// (`middle_interfaces.rs:66`), so this lane's population is comparable with
/// the three functions the bijection already grades.
const FLAGS: [&str; 3] = ["/Ox", "/GS-", "/c"];

/// The site: **after the final schedule**, the order that actually reaches the
/// encoder. A count taken at `sched0` would be the last schedule's *input*.
const PHASE: &str = "after0";

// The pseudo-op opcodes this lane names, all from `docs/whitebox/ref/P_EXPAND.md`.
/// Prologue arms (§4.1): `0x2f0` → `FUN_10c21719`, `0x2f4` → `FUN_10c216f5`.
const OP_PROLOGUE: [u32; 2] = [0x2f0, 0x2f4];
/// The restore/epilogue arm (§4.1): `0x2f6` → `FUN_10bffb72`. **Its word count
/// is in no field of the obj** — this is #3431's caveat 2, the ungraded term.
const OP_EPILOGUE: u32 = 0x2f6;
/// The genuinely **unbounded** arms (§3 reading 3): alignment padding is a
/// loop, so no constant describes it. Any instrument asserting a word count
/// must special-case them — this one *strata-fies* on them instead.
const OP_UNBOUNDED: [u32; 3] = [0x27b, 0x2e5, 0x28f];

// ---------------------------------------------------------------------------
// COFF reading — `.text` functions and `.pdata` records
// ---------------------------------------------------------------------------
//
// `crates/c2-obj` has **no** pub accessor for a section's raw bytes
// (`section_names()` returns names only; `coff_layout()` is private), and
// `.pdata` appears there only as emitter material. Rather than widen a shared
// crate mid-wave, this lane keeps the slicing local to the harness — the
// attribution logic below is measurement-specific, not a general accessor.
// Recorded in the rung as the reason.

struct Coff<'a> {
    b: &'a [u8],
    nsec: usize,
    sect: usize,
    symptr: usize,
    nsym: usize,
}

impl<'a> Coff<'a> {
    fn new(obj: &'a ObjImage) -> Coff<'a> {
        let b = obj.as_bytes();
        let g16 = |o: usize| u16::from_le_bytes(b[o..o + 2].try_into().unwrap());
        let g32 = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let nsec = g16(2) as usize;
        let symptr = g32(8) as usize;
        let nsym = g32(12) as usize;
        let opt = g16(16) as usize;
        Coff { b, nsec, sect: 20 + opt, symptr, nsym }
    }
    fn g16(&self, o: usize) -> u16 {
        u16::from_le_bytes(self.b[o..o + 2].try_into().unwrap())
    }
    fn g32(&self, o: usize) -> u32 {
        u32::from_le_bytes(self.b[o..o + 4].try_into().unwrap())
    }
    /// `(section index 1-based, raw ptr, raw size)` for the first section whose
    /// name starts with `pfx`.
    fn section(&self, pfx: &str) -> Option<(usize, usize, usize)> {
        for i in 0..self.nsec {
            let o = self.sect + i * 40;
            let name =
                String::from_utf8_lossy(&self.b[o..o + 8]).trim_end_matches('\0').to_string();
            if name.starts_with(pfx) {
                return Some((i + 1, self.g32(o + 20) as usize, self.g32(o + 16) as usize));
            }
        }
        None
    }
}

/// The inter-function padding word. `pdata.rs:24` is explicit that `.pdata`'s
/// `FuncLen` is the function length *"**excluding** any inter-function
/// padding"*, so a `W` sliced naively from consecutive symbol `Value`s
/// **includes** padding the oracle field does not — which breaks both the
/// equality and the `FuncLen` cross-check that verifies attribution.
///
/// **Measured, not anticipated, and the first guess was wrong.** This lane
/// first assumed PPC `nop` (`ori r0,r0,0` = `0x60000000`), which stripped
/// nothing and left 7 functions unattributable. The failing tails, printed
/// rather than reasoned about, read `7d8803a6 4e800020 00000000` — `mtlr r12`,
/// `blr`, then a **zero** word. c2 pads with zeros here. Zero is not a valid
/// PPC primary opcode, so a trailing zero word is unambiguously padding and
/// never a truncated instruction.
const PAD_WORD: u32 = 0x0000_0000;

/// One `.text` function.
#[derive(Clone, Debug)]
struct TextFn {
    name: String,
    /// Byte offset of the function within `.text` — the value a `.pdata`
    /// `BeginAddress` relocation resolves to.
    start: u32,
    /// Word count, **excluding** trailing inter-function `nop` padding.
    words: usize,
    /// Padding words stripped.
    pad: usize,
    /// The function's leading words, capped — enough to classify a prologue
    /// (`prolog_words` is ≤ 8 on 100 % of #3431's 12,610).
    head: Vec<u32>,
}

/// Classify one PPC word far enough to name a prologue shape. Transcribed from
/// `docs/whitebox/scripts/probe_prolog_words.py::classify` so the two agree by
/// construction — this lane extends that probe's shape population rather than
/// competing with it.
fn classify(w: u32) -> &'static str {
    let op = w >> 26;
    let xo = (w >> 1) & 0x3FF;
    match (op, xo) {
        (31, 339) => "mfspr", // `mflr r12` is `mfspr r12,LR`
        (31, 467) => "mtspr",
        (31, 183) => "stwux",
        (31, 444) => "or", // `mr`
        (18, _) => {
            if w & 1 != 0 {
                "bl"
            } else {
                "b"
            }
        }
        (16, _) => "bc",
        (37, _) => "stwu",
        (36, _) => "stw",
        (54, _) => "stfd",
        (62, _) => "std",
        (14, _) => "addi",
        (15, _) => "addis",
        _ => "other",
    }
}

/// The obj's `.text` functions, in address order.
///
/// Slices on the function symbols' `Value` (at **+8**, not +4 — +4 is the
/// second half of the 8-byte `Name` union). These fixtures compile to a
/// non-COMDAT `.text` (`Characteristics = 0x60400020`), so
/// `ObjImage::text_comdat_functions_with_bytes` correctly returns nothing and
/// an instrument built on it would grade zero while printing a pass.
fn text_functions(obj: &ObjImage) -> Vec<TextFn> {
    let c = Coff::new(obj);
    let Some((idx, rawptr, rawsize)) = c.section(".text") else { return Vec::new() };
    let strtab = c.symptr + c.nsym * 18;
    let mut syms: Vec<(u32, String)> = Vec::new();
    let mut i = 0usize;
    while i < c.nsym {
        let o = c.symptr + i * 18;
        let naux = c.b[o + 17] as usize;
        if c.g16(o + 12) as usize == idx && c.b[o + 16] == 2 && c.g16(o + 14) == 0x20 {
            let name = if c.g32(o) == 0 {
                let so = strtab + c.g32(o + 4) as usize;
                let end = c.b[so..].iter().position(|&x| x == 0).unwrap_or(0) + so;
                String::from_utf8_lossy(&c.b[so..end]).to_string()
            } else {
                String::from_utf8_lossy(&c.b[o..o + 8]).trim_end_matches('\0').to_string()
            };
            syms.push((c.g32(o + 8), name));
        }
        i += 1 + naux;
    }
    syms.sort_by_key(|(v, _)| *v);
    let mut out = Vec::new();
    for (k, (val, name)) in syms.iter().enumerate() {
        let start = *val as usize;
        let end = syms.get(k + 1).map(|(v, _)| *v as usize).unwrap_or(rawsize);
        let words: Vec<u32> = c.b[rawptr + start..rawptr + end]
            .chunks_exact(4)
            .map(|x| u32::from_be_bytes(x.try_into().unwrap()))
            .collect();
        // Strip trailing inter-function `nop` padding, to match `FuncLen`.
        let mut n = words.len();
        while n > 0 && words[n - 1] == PAD_WORD {
            n -= 1;
        }
        out.push(TextFn {
            name: name.clone(),
            start: *val,
            words: n,
            pad: words.len() - n,
            head: words.iter().take(8).copied().collect(),
        });
    }
    out
}

/// Attribute each `.pdata` record to a `.text` function **through the record's
/// own `BeginAddress` relocation**, which is the only exact answer.
///
/// `probe_prolog_words.py` punted on this — it restricted its shape
/// sub-population to single-`.text` objs "so the words can be attributed
/// without resolving the `BeginAddress` relocation", which is why that
/// sub-population is 2.2 %. Resolving it is not hard and it removes the whole
/// question: a record's first dword carries an `ADDR32` relocation naming
/// either the function symbol itself or the `.text` section symbol plus an
/// addend, and both forms land on exactly one function.
///
/// Returns `None` if anything does not resolve — a *guessed* attribution is a
/// wrong `P` silently bound to the wrong function, which is worse than a
/// counted refusal.
fn attribute(
    obj: &ObjImage,
    funcs: &[TextFn],
    recs: &[PdataRec],
) -> Result<Vec<Option<PdataRec>>, String> {
    let c = Coff::new(obj);
    let (text_idx, _tp, _ts) = c.section(".text").ok_or("no .text")?;
    // An obj with no `.pdata` section is an ALL-LEAF TU, not an attribution
    // failure: every function correctly has `P = None`. Treating it as a
    // failure is how the leaf stratum — the majority of the obj corpus,
    // 3,608 of 6,000 objs — would have been thrown away as noise.
    if recs.is_empty() {
        return Ok(vec![None; funcs.len()]);
    }
    let (pd_idx, pd_ptr, _pd_size) = c.section(".pdata").ok_or("no .pdata")?;
    let relocs = obj.relocations().ok_or("relocations() refused")?;

    // Symbol table: index -> (section number, value).
    let mut sym: Vec<(usize, u32)> = Vec::with_capacity(c.nsym);
    let mut i = 0usize;
    while i < c.nsym {
        let o = c.symptr + i * 18;
        let naux = c.b[o + 17] as usize;
        let ent = (c.g16(o + 12) as usize, c.g32(o + 8));
        for _ in 0..=naux {
            sym.push(ent);
        }
        i += 1 + naux;
    }

    let start_of: BTreeMap<u32, usize> =
        funcs.iter().enumerate().map(|(k, f)| (f.start, k)).collect();
    let mut out: Vec<Option<PdataRec>> = vec![None; funcs.len()];
    let mut seen = 0usize;
    for r in relocs.iter().filter(|r| r.section + 1 == pd_idx) {
        // Only the `BeginAddress` dword (offset 0 of an 8-byte record).
        if r.va % 8 != 0 {
            continue;
        }
        let k = (r.va / 8) as usize;
        let rec = *recs.get(k).ok_or_else(|| format!("reloc va {} past record {}", r.va, recs.len()))?;
        let s = *sym.get(r.sym as usize).ok_or("reloc sym index out of range")?;
        if s.0 != text_idx {
            return Err(format!("record {k} sym in section {} not .text {text_idx}", s.0));
        }
        // The raw dword is the addend; for a function symbol it is 0 and the
        // symbol's own Value locates the function, for a section symbol the
        // Value is 0 and the addend does. Adding them is correct for both.
        let addend = u32::from_be_bytes(
            c.b[pd_ptr + r.va as usize..pd_ptr + r.va as usize + 4].try_into().map_err(|_| "short .pdata")?,
        );
        let want = s.1.wrapping_add(addend);
        let fi = *start_of.get(&want).ok_or_else(|| {
            format!("record {k} -> .text+{want} (sym val {} addend {addend}) is no function start", s.1)
        })?;
        if out[fi].is_some() {
            return Err(format!("two records for function {}", funcs[fi].name));
        }
        out[fi] = Some(rec);
        seen += 1;
    }
    // Every record must have been placed. A record with no relocation is a
    // record this reader did not understand, not an absent one.
    if seen != recs.len() {
        return Err(format!("{seen} of {} records had a BeginAddress relocation", recs.len()));
    }
    // Cross-check: the oracle's own `FuncLen` must equal the word count sliced
    // out of `.text`. This is the self-check that makes the attribution
    // falsifiable rather than merely plausible.
    for (f, r) in funcs.iter().zip(out.iter()) {
        if let Some(r) = r {
            if r.func_words as usize != f.words {
                let (tp, _) = (c.section(".text").unwrap().1, 0);
                let tail: Vec<String> = (0..f.words.min(3))
                    .rev()
                    .map(|k| {
                        let o = tp + f.start as usize + (f.words - 1 - k) * 4;
                        format!("{:08x}", u32::from_be_bytes(c.b[o..o + 4].try_into().unwrap()))
                    })
                    .collect();
                return Err(format!(
                    "FuncLen {} != sliced words {} for {} (pad {}, tail {})",
                    r.func_words,
                    f.words,
                    f.name,
                    f.pad,
                    tail.join(" ")
                ));
            }
        }
    }
    Ok(out)
}

/// One `.pdata` record, decoded per `crates/c2-core/src/coff/pdata.rs:71`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PdataRec {
    /// `bits 7..0` — prologue length **in instructions**. Already words; no
    /// division. This is `P(f)`.
    prolog_words: u32,
    /// `bits 29..8` — function length in instructions.
    func_words: u32,
    /// `bit 31` — the function has EH data. The port refuses this case, and a
    /// `try`/`catch` body produces **several** records, which breaks the
    /// one-record-per-function assumption attribution rests on.
    eh: bool,
}

/// Every `.pdata` record in the obj, in section order (which is `.text` order —
/// `pdata.rs:88`). Big-endian, like `.text` and unlike every COFF header field.
fn pdata_records(obj: &ObjImage) -> Vec<PdataRec> {
    let c = Coff::new(obj);
    let Some((_i, ptr, size)) = c.section(".pdata") else { return Vec::new() };
    let mut out = Vec::new();
    let raw = &c.b[ptr..ptr + size];
    for r in raw.chunks_exact(8) {
        let w = u32::from_be_bytes(r[4..8].try_into().unwrap());
        out.push(PdataRec {
            prolog_words: w & 0xFF,
            func_words: (w >> 8) & 0x3F_FFFF,
            eh: (w >> 31) & 1 == 1,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Tuple rows
// ---------------------------------------------------------------------------

/// The tuple SPINE: `<opcode> <cat> <flags> <cc>`, funcwalk flavour (no leading
/// index).
#[derive(Clone, Copy, Debug)]
struct Tuple {
    opcode: u32,
    flags: u32,
}

impl Tuple {
    fn parse(row: &str) -> Option<Tuple> {
        let spine = row.split_once(" | ").map(|(s, _)| s).unwrap_or(row);
        let mut it = spine.split_whitespace();
        let opcode = u32::from_str_radix(it.next()?, 16).ok()?;
        let _cat = it.next()?;
        let flags = u32::from_str_radix(it.next()?, 16).ok()?;
        Some(Tuple { opcode, flags })
    }
    /// Tuple `+0x9` bit 0 — R2's invariant, reproduced from the constructor end
    /// in `ref/P_EXPAND.md` §2 ("every one … ORs bit 0 into `node+9`").
    fn is_instruction(self) -> bool {
        self.flags & 1 != 0
    }
}

// ---------------------------------------------------------------------------
// The per-function measurement
// ---------------------------------------------------------------------------

/// Which stratum a function landed in. Registered in the prereg §5 **before**
/// measuring, so a stratum cannot be invented to explain a bad number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stratum {
    /// No `.pdata` record — a leaf. `P(f)` is `None`; **absent from the
    /// denominator, never counted as zero.**
    Leaf,
    /// Has a record, and no unbounded-arm pseudo-op appears in its tuples.
    FramedClean,
    /// Has a record **and** ≥1 of `nopalign`/`0x2e5`/`retaddr`.
    FramedUnbounded,
    /// **The tap's function ordinal could not be verified against `.text`
    /// address order.** This instrument pairs funcwalk `func == i+1` with the
    /// i-th function in `.text` address order. That is an *assumption*: c2's
    /// ordinal is its own processing order, and nothing in the funcwalk payload
    /// carries a name to check it with. When the TU's funcwalk count and its
    /// `.text` function count disagree, the pairing is provably unsafe and
    /// every row of that TU goes here rather than into a hold-rate.
    OrdinalUnverified,
    /// Record↔function match did not verify. Not guessed at.
    Unattributable,
}

#[derive(Clone, Debug)]
struct Row {
    fixture: String,
    func: String,
    stratum: Stratum,
    /// `T(f)`
    t: usize,
    /// `W(f)`
    w: usize,
    /// `I(f)` — prologue pseudo-op tuples carrying the real-instruction bit.
    i: usize,
    /// `P(f)` — `None` for a leaf.
    p: Option<u32>,
    /// Count of epilogue (`0x2f6`) tuples, however flagged.
    n_epi: usize,
    /// Count of prologue-family tuples, however flagged.
    n_pro: usize,
    /// Trailing inter-function `nop` padding words stripped from `w`.
    pad: usize,
    /// `T` counted at EVERY site that produced a funcwalk for this function,
    /// not only at `after0`. This is what locates the expansion: if the
    /// equality holds at `after0` but not upstream, the prologue was expanded
    /// somewhere between.
    t_by_phase: Vec<(String, usize)>,
    /// The classified prologue: the first `P` words of `.text`, by shape.
    /// Empty for a leaf.
    shape: Vec<&'static str>,
    /// Opcodes of the tuples NOT carrying the real-instruction bit — i.e. the
    /// pseudo-ops that survive to `after0`. Small; kept so the rung can say
    /// *which* pseudo-ops are still there rather than only how many.
    pseudo: Vec<u32>,
}

impl Row {
    fn h0(&self) -> bool {
        self.t == self.w
    }
    /// `T - I + P == W`, only defined where a record exists.
    fn h1(&self) -> Option<bool> {
        self.p.map(|p| self.t - self.i + p as usize == self.w)
    }
    /// `R = W - (T - I + P)`, signed: it can be negative.
    fn residual(&self) -> Option<i64> {
        self.p.map(|p| self.w as i64 - (self.t as i64 - self.i as i64 + p as i64))
    }
}

/// How the instrument is deliberately broken, to be **watched failing** before
/// any number is trusted (prereg §3; `CLAUDE.md`'s formatter rule generalized —
/// *a fence never seen refusing is not a fence*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Perturb {
    None,
    /// Add 1 to every `P`. H1's hold-rate must COLLAPSE.
    PlusOneP,
    /// Ignore the real-instruction bit. H0 must go red where it was green.
    NoInsnFilter,
}

fn measure_one(
    tc: &Toolchain,
    cpp: &Path,
    work: &Path,
    perturb: Perturb,
) -> Result<Vec<Row>, String> {
    let abs = cpp.canonicalize().map_err(|e| e.to_string())?;
    let flags: Vec<String> = FLAGS.iter().map(|s| (*s).to_string()).collect();
    let cap = tc
        .capture_reference_with(&c2_reference::to_wibo_path(&abs), &work.join("cap"), &flags, None)
        .map_err(|e| format!("capture failed: {e}"))?;
    let out = cap.ref_obj_path.clone();
    let (obj, rep) = tc
        .replay_tapped_probe(&cap, &work.join("il"), &out, STAGE_SITES, true, true)
        .map_err(|e| format!("tapped replay failed: {e}"))?;

    // POSITIVE CHECK before a single row is read. An unarmed run yields an
    // empty walk, and "0 agreed with 0" is the vacuous green this whole family
    // of instruments exists not to print.
    if !rep.armed_and_fired() {
        return Err(format!("tap did not arm and fire (refused={:?})", rep.refused));
    }
    // Bounds honesty: WALK_MAX 4096 / BLK_MAX 4096 / OPS_MAX 128 / ARENA 4 MiB.
    // A truncated payload makes every count below a FLOOR, so it is excluded
    // and counted, never silently floored.
    if !rep.walk_refusals.is_empty() {
        return Err(format!("walk refused (TRUNCATED): {:?}", rep.walk_refusals));
    }

    let funcs = text_functions(&obj);
    if funcs.is_empty() {
        return Err("no .text functions".into());
    }
    let recs = pdata_records(&obj);
    // A record with the EH bit splits a function into SEVERAL records, which
    // breaks the one-record-per-function assumption attribution rests on.
    let eh_present = recs.iter().any(|r| r.eh);
    let attributed = if eh_present {
        Err("EH bit set: a function splits into several records".to_string())
    } else {
        attribute(&obj, &funcs, &recs)
    };
    let why = attributed.as_ref().err().cloned();
    let ok = attributed.is_ok();
    let rec_for = attributed.unwrap_or_else(|_| vec![None; funcs.len()]);
    if let Some(w) = &why {
        if std::env::var_os("C2RS_PWORDS_DUMP").is_some() {
            eprintln!("  ATTRIB-FAIL {}: {w}", cpp.file_name().unwrap().to_string_lossy());
        }
    }

    // The ordinal fence. `func == i+1` is paired with the i-th `.text`
    // function in address order; if c2 walked a different NUMBER of functions
    // than `.text` contains, that pairing is provably unsafe for this TU.
    // Measured, not anticipated: `wkg_splice_pos.cpp` produced six "failures"
    // with mismatches in both directions (T=73 against W=14 beside T=6 against
    // W=72) — the signature of a permuted pairing, not of a compiler that
    // sometimes emits 59 extra words.
    let n_walks = rep.funcs.iter().filter(|x| x.phase == PHASE).count();
    let ordinals_verified = n_walks == funcs.len();

    let mut out_rows = Vec::new();
    for (fi, tf) in funcs.iter().enumerate() {
        let (name, w, pad) = (&tf.name, &tf.words, &tf.pad);
        let f = (fi + 1) as u32;
        let Some(fw) = rep.funcs.iter().find(|x| x.phase == PHASE && x.func == f) else {
            continue;
        };
        let rows = fw.rows();
        let tuples: Vec<Tuple> = rows.iter().filter_map(|r| Tuple::parse(r)).collect();
        let t = match perturb {
            Perturb::NoInsnFilter => tuples.len(),
            _ => tuples.iter().filter(|x| x.is_instruction()).count(),
        };
        let i = tuples
            .iter()
            .filter(|x| OP_PROLOGUE.contains(&x.opcode) && x.is_instruction())
            .count();
        let n_pro = tuples.iter().filter(|x| OP_PROLOGUE.contains(&x.opcode)).count();
        let n_epi = tuples.iter().filter(|x| x.opcode == OP_EPILOGUE).count();
        let unbounded = tuples.iter().any(|x| OP_UNBOUNDED.contains(&x.opcode));

        let p = rec_for[fi].map(|r| {
            let base = r.prolog_words;
            match perturb {
                Perturb::PlusOneP => base + 1,
                _ => base,
            }
        });
        let stratum = if !ordinals_verified {
            Stratum::OrdinalUnverified
        } else if !ok {
            Stratum::Unattributable
        } else if p.is_none() {
            Stratum::Leaf
        } else if unbounded {
            Stratum::FramedUnbounded
        } else {
            Stratum::FramedClean
        };
        out_rows.push(Row {
            fixture: cpp.file_name().unwrap().to_string_lossy().to_string(),
            func: name.clone(),
            stratum,
            t,
            w: *w,
            i,
            p,
            n_epi,
            n_pro,
            pad: *pad,
            t_by_phase: {
                let mut v: Vec<(String, usize)> = rep
                    .funcs
                    .iter()
                    .filter(|x| x.func == f)
                    .map(|x| {
                        (
                            x.phase.clone(),
                            x.rows()
                                .iter()
                                .filter_map(|r| Tuple::parse(r))
                                .filter(|t| t.is_instruction())
                                .count(),
                        )
                    })
                    .collect();
                v.sort();
                v.dedup();
                v
            },
            shape: match p {
                Some(pw) => tf.head.iter().take(pw as usize).map(|w| classify(*w)).collect(),
                None => Vec::new(),
            },
            pseudo: tuples.iter().filter(|x| !x.is_instruction()).map(|x| x.opcode).collect(),
        });
    }
    Ok(out_rows)
}

// ---------------------------------------------------------------------------
// Corpus driver
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn all_fixtures() -> Vec<PathBuf> {
    let d = repo_root().join("fixtures/cpp");
    let mut v: Vec<PathBuf> = std::fs::read_dir(&d)
        .expect("fixtures/cpp unreadable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "cpp").unwrap_or(false))
        .collect();
    v.sort();
    v
}

struct Corpus {
    rows: Vec<Row>,
    /// `(fixture, reason)` — captures/replays that did not produce rows.
    excluded: Vec<(String, String)>,
}

fn run_corpus(tc: &Toolchain, fixtures: &[PathBuf], perturb: Perturb, jobs: usize) -> Corpus {
    let rows = Mutex::new(Vec::new());
    let excluded = Mutex::new(Vec::new());
    let next = AtomicUsize::new(0);
    let base = std::env::temp_dir().join(format!("c2rs-pwords-{}", std::process::id()));
    std::thread::scope(|s| {
        for j in 0..jobs {
            let (rows, excluded, next, base) = (&rows, &excluded, &next, &base);
            s.spawn(move || {
                let w = base.join(format!("j{j}"));
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= fixtures.len() {
                        break;
                    }
                    let _ = std::fs::remove_dir_all(&w);
                    let name = fixtures[i].file_name().unwrap().to_string_lossy().to_string();
                    match measure_one(tc, &fixtures[i], &w, perturb) {
                        Ok(r) if r.is_empty() => {
                            excluded.lock().unwrap().push((name, "no graded function".into()))
                        }
                        Ok(r) => rows.lock().unwrap().extend(r),
                        Err(e) => excluded.lock().unwrap().push((name, e)),
                    }
                }
                let _ = std::fs::remove_dir_all(&w);
            });
        }
    });
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by(|a, b| (&a.fixture, &a.func).cmp(&(&b.fixture, &b.func)));
    let mut excluded = excluded.into_inner().unwrap();
    excluded.sort();
    let _ = std::fs::remove_dir_all(&base);
    Corpus { rows, excluded }
}

fn ready() -> Option<Toolchain> {
    match Toolchain::locate() {
        Some(tc) if tc.strace.is_some() => Some(tc),
        other => {
            let why = if other.is_none() { "toolchain absent" } else { "strace absent" };
            if std::env::var_os("C2RS_REQUIRE_TOOLCHAIN").is_some() {
                panic!("C2RS_REQUIRE_TOOLCHAIN is set but {why}");
            }
            eprintln!("SKIP: {why}");
            None
        }
    }
}

fn jobs() -> usize {
    std::env::var("C2RS_PWORDS_JOBS").ok().and_then(|v| v.parse().ok()).unwrap_or(8)
}

/// How many fixtures to drive. `0` = all of them. Defaults to a bounded slice
/// so a routine `cargo test` is not a ten-minute corpus walk; the lane's own
/// evidence run sets `C2RS_PWORDS_LIMIT=0` and the rung quotes that command.
fn limit() -> usize {
    std::env::var("C2RS_PWORDS_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(48)
}

fn population() -> Vec<PathBuf> {
    let all = all_fixtures();
    let n = limit();
    if n == 0 || n >= all.len() {
        all
    } else {
        // Spread across the (sorted) corpus rather than taking a prefix, so the
        // bounded default is not just the `il_*` fixtures.
        let step = all.len() / n;
        all.into_iter().step_by(step.max(1)).take(n).collect()
    }
}

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

fn pct(a: usize, b: usize) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

fn report(c: &Corpus) {
    let mut by: BTreeMap<Stratum, Vec<&Row>> = BTreeMap::new();
    for r in &c.rows {
        by.entry(r.stratum).or_default().push(r);
    }
    eprintln!("\n===== w-pwords: the corrected bijection =====");
    eprintln!(
        "graded {} functions over {} fixtures; {} fixtures excluded",
        c.rows.len(),
        c.rows.iter().map(|r| &r.fixture).collect::<std::collections::BTreeSet<_>>().len(),
        c.excluded.len()
    );

    eprintln!("\n-- strata --");
    for (s, v) in &by {
        eprintln!("  {:<18?} {:>5}  ({:>5.1}%)", s, v.len(), pct(v.len(), c.rows.len()));
    }

    eprintln!("\n-- H0: T == W  (the demoted equality itself) --");
    for (s, v) in &by {
        let h = v.iter().filter(|r| r.h0()).count();
        eprintln!("  {:<18?} {:>5}/{:<5} = {:>6.2}%", s, h, v.len(), pct(h, v.len()));
    }

    eprintln!("\n-- H1: T - I + P == W  (the prologue correction) --");
    for (s, v) in &by {
        let d: Vec<&&Row> = v.iter().filter(|r| r.h1().is_some()).collect();
        let h = d.iter().filter(|r| r.h1() == Some(true)).count();
        eprintln!("  {:<18?} {:>5}/{:<5} = {:>6.2}%", s, h, d.len(), pct(h, d.len()));
    }

    eprintln!("\n-- H2: R = W - (T - I + P), the residual .pdata CANNOT see --");
    for (s, v) in &by {
        if *s == Stratum::Leaf {
            continue;
        }
        let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
        for r in v.iter().filter_map(|r| r.residual()) {
            *hist.entry(r).or_default() += 1;
        }
        if hist.is_empty() {
            continue;
        }
        let tot: usize = hist.values().sum();
        let modal = hist.iter().max_by_key(|(_, n)| **n).unwrap();
        eprintln!(
            "  {:<18?} {} distinct, modal R={} at {:.1}%",
            s,
            hist.len(),
            modal.0,
            pct(*modal.1, tot)
        );
        for (k, n) in &hist {
            eprintln!("      R = {:>4} : {:>5}  ({:>5.1}%)", k, n, pct(*n, tot));
        }
    }

    // P6: is R an exact FUNCTION of P? If it is, the equality closes with the
    // obj field alone and the demotion is fully repairable.
    eprintln!("\n-- is R an exact function of P? (P6) --");
    let mut map: BTreeMap<u32, BTreeMap<i64, usize>> = BTreeMap::new();
    for r in c.rows.iter().filter(|r| r.stratum != Stratum::Leaf) {
        if let (Some(p), Some(res)) = (r.p, r.residual()) {
            *map.entry(p).or_default().entry(res).or_default() += 1;
        }
    }
    let mut functional = true;
    for (p, rs) in &map {
        if rs.len() > 1 {
            functional = false;
        }
        let tot: usize = rs.values().sum();
        eprintln!(
            "  P={} n={:<5} R values: {}",
            p,
            tot,
            rs.iter().map(|(k, n)| format!("{k}×{n}")).collect::<Vec<_>>().join(" ")
        );
    }
    eprintln!(
        "  => R is {} an exact function of P",
        if functional { "**EXACTLY**" } else { "**NOT**" }
    );

    // I(f): does the prologue pseudo-op carry the real-instruction bit? (P3)
    let framed: Vec<&Row> = c.rows.iter().filter(|r| r.p.is_some()).collect();
    let with_i = framed.iter().filter(|r| r.i > 0).count();
    let with_pro = framed.iter().filter(|r| r.n_pro > 0).count();
    let with_epi = framed.iter().filter(|r| r.n_epi > 0).count();
    eprintln!("\n-- the pseudo-op tuples themselves (P3) --");
    eprintln!("  framed functions                              {}", framed.len());
    eprintln!("  ... with >=1 prologue-family tuple (any flag) {with_pro}");
    eprintln!("  ... with >=1 prologue tuple FLAGGED real (I>0) {with_i}");
    eprintln!("  ... with >=1 epilogue 0x2f6 tuple             {with_epi}");

    // WHICH pseudo-ops survive to `after0` at all. If the prologue family is
    // absent here, the expansion happened UPSTREAM of the tap and the AMENDED
    // block's premise — "one tuple that becomes many words" — is not a
    // description of what `after0` sees.
    eprintln!("\n-- pseudo-op tuples surviving to {PHASE} (not flagged real) --");
    let mut ps: BTreeMap<u32, usize> = BTreeMap::new();
    for r in &c.rows {
        for o in &r.pseudo {
            *ps.entry(*o).or_default() += 1;
        }
    }
    if ps.is_empty() {
        eprintln!("  NONE — every tuple at {PHASE} carries the real-instruction bit");
    }
    for (o, n) in ps.iter().take(24) {
        let tag = if OP_PROLOGUE.contains(o) {
            " <- PROLOGUE"
        } else if *o == OP_EPILOGUE {
            " <- EPILOGUE"
        } else if OP_UNBOUNDED.contains(o) {
            " <- UNBOUNDED ARM"
        } else {
            ""
        };
        eprintln!("  {:#06x} : {:>5}{}", o, n, tag);
    }

    // R6's caveat 1 was that the prologue SHAPE is confirmed on 282 records
    // (2.2 %) only, because `probe_prolog_words.py` could not attribute a
    // record to a function without resolving the `BeginAddress` relocation.
    // This lane resolves it, so every framed function here is shape-attributable.
    let framed_rows: Vec<&Row> = c
        .rows
        .iter()
        .filter(|r| r.p.is_some() && r.stratum != Stratum::OrdinalUnverified)
        .collect();
    eprintln!(
        "\n-- prologue SHAPE on {} exactly-attributed framed functions (R6's caveat 1) --",
        framed_rows.len()
    );
    let mut shapes: BTreeMap<String, usize> = BTreeMap::new();
    for r in &framed_rows {
        *shapes.entry(r.shape.join("|")).or_default() += 1;
    }
    let mut sv: Vec<(&String, &usize)> = shapes.iter().collect();
    sv.sort_by(|a, b| b.1.cmp(a.1));
    for (k, n) in sv.iter().take(12) {
        eprintln!("  {:>5}  {}", n, k);
    }
    let with_bl = framed_rows.iter().filter(|r| r.shape.contains(&"bl")).count();
    eprintln!(
        "  prologues containing a `bl` (register-save helper): {}/{} = {:.1}%",
        with_bl,
        framed_rows.len(),
        pct(with_bl, framed_rows.len())
    );

    // WHERE does the expansion happen? Counting `T` at every site that walked
    // the function turns "the equality holds at after0" into a statement about
    // the pass order.
    eprintln!("\n-- H0 (T == W) by TAP SITE, framed functions only --");
    let mut per: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for r in c.rows.iter().filter(|r| r.p.is_some() && r.stratum != Stratum::OrdinalUnverified) {
        for (ph, t) in &r.t_by_phase {
            let e = per.entry(ph.clone()).or_default();
            e.1 += 1;
            if *t == r.w {
                e.0 += 1;
            }
        }
    }
    for (ph, (h, n)) in &per {
        eprintln!("  {:<10} {:>5}/{:<5} = {:>6.2}%{}", ph, h, n, pct(*h, *n),
            if ph == PHASE { "   <- the site the bijection uses" } else { "" });
    }

    let padded = c.rows.iter().filter(|r| r.pad > 0).count();
    eprintln!(
        "\n-- inter-function `nop` padding: {} of {} functions carried some ({:.1}%) --",
        padded,
        c.rows.len(),
        pct(padded, c.rows.len())
    );

    if std::env::var_os("C2RS_PWORDS_DUMP").is_some() {
        eprintln!("\n-- per-function rows (H0 failures first) --");
        let mut v: Vec<&Row> = c.rows.iter().collect();
        v.sort_by_key(|r| (r.h0(), r.fixture.clone(), r.func.clone()));
        for r in v.iter().take(120) {
            eprintln!(
                "  {} {:<28} {:<16?} T={:<4} W={:<4} I={} P={:<5} pad={} epi={} pro={} {}",
                if r.h0() { "  " } else { "H0" },
                r.fixture,
                r.stratum,
                r.t,
                r.w,
                r.i,
                r.p.map(|x| x as i64).unwrap_or(-1),
                r.pad,
                r.n_epi,
                r.n_pro,
                r.func
            );
        }
    }

    if !c.excluded.is_empty() {
        eprintln!("\n-- excluded fixtures (counted, never silently floored) --");
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        for (_f, e) in &c.excluded {
            let k = e.split(':').next().unwrap_or(e).to_string();
            *why.entry(k).or_default() += 1;
        }
        for (k, n) in &why {
            eprintln!("  {:>4}  {}", n, k);
        }
    }
    eprintln!("=============================================\n");
}

// ---------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------

/// **The measurement.** Drives the fixture corpus and publishes H0/H1/H2 with
/// denominators and strata.
///
/// This test asserts only what must be true for the *numbers to mean anything*
/// — that a real population graded, and that the leaf/framed split is not
/// degenerate. **It deliberately does not assert a hold-rate**: the hold-rate
/// is the finding, and a test that pinned it would convert a measurement into
/// a gate, which prereg §0 forbids.
#[test]
fn pwords_corrected_bijection_over_the_fixture_corpus() {
    let Some(tc) = ready() else { return };
    let pop = population();
    let c = run_corpus(&tc, &pop, Perturb::None, jobs());
    report(&c);

    assert!(
        c.rows.len() >= 40,
        "only {} functions graded — prereg §5's FAILED threshold is 40",
        c.rows.len()
    );
    let framed = c.rows.iter().filter(|r| r.p.is_some()).count();
    let leaf = c.rows.iter().filter(|r| r.stratum == Stratum::Leaf).count();
    assert!(
        framed > 0,
        "ZERO framed functions graded — this lane would be measuring exactly the \
         population the bijection already covers, which is the failure the AMENDED \
         block's 3-function fence describes"
    );
    assert!(leaf > 0, "ZERO leaf functions — the baseline stratum is missing");
}

/// **The fence, watched refusing** (prereg §3; `CLAUDE.md`'s formatter rule).
///
/// A hold-rate nobody has seen move is not a measurement. Two deliberate
/// corruptions, each of which MUST change the numbers:
///
/// 1. `P += 1` on every framed function — H1's hold-rate must **collapse**.
/// 2. drop the `is_instruction()` filter — H0 must go **red** where it was green.
#[test]
fn the_instrument_fails_on_deliberately_broken_input() {
    let Some(tc) = ready() else { return };
    // A small population: this test is about the instrument's sensitivity, not
    // about the corpus.
    let pop: Vec<PathBuf> = population().into_iter().take(16).collect();

    let base = run_corpus(&tc, &pop, Perturb::None, jobs());
    let h1_ok = |c: &Corpus| {
        let d: Vec<&Row> = c.rows.iter().filter(|r| r.h1().is_some()).collect();
        (d.iter().filter(|r| r.h1() == Some(true)).count(), d.len())
    };
    let h0_ok = |c: &Corpus| {
        (c.rows.iter().filter(|r| r.h0()).count(), c.rows.len())
    };

    let (b1, bn1) = h1_ok(&base);
    let (b0, bn0) = h0_ok(&base);
    assert!(bn1 > 0, "no framed function in the control population — nothing to break");
    assert!(b0 > 0, "H0 holds NOWHERE in the control — check 2 could not detect a change");
    eprintln!("BROKEN-INPUT control: H1 {b1}/{bn1}, H0 {b0}/{bn0}");

    // Check 1 could NOT be "H1's hold-rate collapses", because this lane
    // measured H1 at 0/45 — a hold count of zero cannot be reduced, so that
    // assertion would pass while reading nothing. The sensitivity check that
    // survives the actual result is on the RESIDUAL: `P += 1` must shift every
    // `R` by exactly -1.
    let resid = |c: &Corpus| {
        let mut v: Vec<i64> = c.rows.iter().filter_map(|r| r.residual()).collect();
        v.sort();
        v
    };
    let plus = run_corpus(&tc, &pop, Perturb::PlusOneP, jobs());
    let (rb, rp) = (resid(&base), resid(&plus));
    eprintln!(
        "BROKEN-INPUT  P+=1  : residuals {} -> {} (first few {:?} -> {:?})",
        rb.len(),
        rp.len(),
        &rb[..rb.len().min(4)],
        &rp[..rp.len().min(4)]
    );
    assert_eq!(rb.len(), rp.len(), "perturbation changed the graded population, not just P");
    assert!(!rb.is_empty(), "no residuals — nothing to break");
    assert_ne!(rb, rp, "P += 1 did NOT move a single residual — `prolog_words` is not being read");
    for (a, b) in rb.iter().zip(rp.iter()) {
        assert_eq!(
            *b,
            *a - 1,
            "P += 1 must shift every residual by exactly -1; saw {a} -> {b}. The \
             instrument is not reading `prolog_words` where it claims to."
        );
    }

    let noflag = run_corpus(&tc, &pop, Perturb::NoInsnFilter, jobs());
    let (n0, nn0) = h0_ok(&noflag);
    eprintln!("BROKEN-INPUT no-flag: H0 {n0}/{nn0}  (control {b0}/{bn0})");
    assert!(
        n0 < b0,
        "dropping the real-instruction filter did NOT reduce H0's hold count \
         ({n0} vs {b0}) — the flags bit is not being read, so `T` is not the \
         quantity this instrument claims to measure"
    );
}

/// The `.pdata` decode, pinned against `c2-core`'s own emitter witnesses
/// (`crates/c2-core/src/coff/pdata.rs:56-64`) — read there, transcribed here.
/// A decoder that drifts from the emitter would silently re-interpret every
/// `P` in this file.
#[test]
fn the_pdata_decode_matches_the_emitters_own_witnesses() {
    // (unwind word, func_words, prolog_words, eh)
    let cases: [(u32, u32, u32, bool); 6] = [
        (0x4000_0903, 9, 3, false),
        (0x4000_1205, 18, 5, false),
        (0x4000_1607, 22, 7, false),
        (0x4000_2203, 34, 3, false),
        (0x4000_0f06, 15, 6, false),
        (0xc000_1306, 19, 6, true),
    ];
    for (w, fw, pw, eh) in cases {
        assert_eq!(w & 0xFF, pw, "prolog_words of {w:#010x}");
        assert_eq!((w >> 8) & 0x3F_FFFF, fw, "func_words of {w:#010x}");
        assert_eq!((w >> 31) & 1 == 1, eh, "EH bit of {w:#010x}");
    }
}
