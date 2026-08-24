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
use c2_reference::stage::{OrdinalVerdict, STAGE_SITES};
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
    /// address order.** Under [`Pairing::Ordinal`] this instrument pairs
    /// funcwalk `func == i+1` with the i-th function in `.text` address order,
    /// which is an *assumption*: c2's ordinal is its own processing order.
    /// Before **#3459** nothing in the payload could check it, so the fence was
    /// the TU's funcwalk count against its `.text` function count.
    ///
    /// **Since #3459 this stratum means one thing only: the payload carried NO
    /// IDENTITY** (an old tap, or the tap refusing the read on every function).
    /// It is kept rather than deleted so that a run against an identity-free
    /// payload degrades to the old, honestly-labelled behaviour instead of
    /// silently pairing on nothing.
    OrdinalUnverified,
    /// Record↔function match did not verify. Not guessed at.
    Unattributable,
}

/// How a `.text` function is bound to a funcwalk.
///
/// **Both arms are kept live, and the reason is the evidence**: `Ordinal` is
/// exactly the pre-#3459 rule, so the two can be run back to back on ONE
/// binary and ONE corpus and the difference attributed to the pairing rule
/// alone. A base-binary-versus-tip-binary diff could not say that.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pairing {
    /// `funcwalk.func == i + 1` against `.text` address order. The assumption
    /// board #3459 is about.
    Ordinal,
    /// `funcwalk.identity() == .text symbol name`. The read.
    Identity,
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

/// One fixture's result: the rows, plus everything #3459 needs said ABOUT the
/// pairing rather than folded into it.
#[derive(Default)]
struct Measured {
    rows: Vec<Row>,
    /// `pair_by_identity`'s verdict label at [`PHASE`].
    verdict: String,
    /// `Some(true/false)` when the verdict was `Verified` — did c2's ordinal
    /// order actually equal `.text` address order on this TU? **This is the
    /// pre-#3459 assumption, now measured per TU.**
    ord_agrees: Option<bool>,
    /// `.text` functions the tap named nothing for. Counted and NAMED, never
    /// folded into a denominator.
    unpaired: Vec<String>,
    /// Funcwalk identities with no `.text` function of that name.
    only_in_tap: Vec<String>,
    /// F2a fired: a deliberately corrupted expected-name list was REFUSED.
    fence_wrongname_fired: bool,
    /// F2b fired: a rotated expected-name list turned a `Verified{agrees:true}`
    /// into `Verified{agrees:false}`.
    fence_rotate_fired: bool,
}

/// **The live fence (prereg §3 F2), run on every fixture at no extra capture
/// cost.**
///
/// The prereg registered ONE check here — *"the rotated verdict MUST NOT be
/// `Verified`"* — and it is **wrong about the design it was fencing**, in
/// exactly the way `w-pwords` §6.1 warns a registered check can be. A rotation
/// is a permutation of the SAME name set, and `pair_by_identity` is
/// order-independent by construction, so the rotated verdict is `Verified` and
/// *must* be: that is the fix working. The rotation's real signal is
/// `ordinal_order_agrees` flipping to false.
///
/// So the check splits in two, and both are watched:
///
/// * **F2a** — corrupt one expected NAME. The verdict must stop being
///   `Verified`. This is the fence on the pairing itself.
/// * **F2b** — ROTATE the expected names. The pairing must survive (`Verified`)
///   and `ordinal_order_agrees` must go false. This is the fence on the ordinal
///   check, i.e. on the thing #3459 is actually about.
fn run_pairing_fences(
    rep: &c2_reference::stage::TapReport,
    text_names: &[String],
    m: &mut Measured,
) {
    if text_names.len() < 2 {
        return; // a rotation of one element is the identity map: nothing to read
    }
    // F2a — one name replaced by a fiction.
    let mut wrong = text_names.to_vec();
    wrong[0] = format!("{}$W-ORDID-NOT-A-REAL-NAME", wrong[0]);
    let vw = rep.verify_ordinals(PHASE, &wrong);
    assert!(
        !vw.is_verified(),
        "FENCE F2a DID NOT REFUSE: a fabricated function name verified as {vw:?}. The \
         identity check is absorbing wrong input, so every `verified` above is worthless."
    );
    m.fence_wrongname_fired = true;

    // F2b — the names rotated by one.
    if m.ord_agrees == Some(true) {
        let mut rot = text_names.to_vec();
        rot.rotate_left(1);
        match rep.verify_ordinals(PHASE, &rot) {
            OrdinalVerdict::Verified { ordinal_order_agrees, .. } => {
                assert!(
                    !ordinal_order_agrees,
                    "FENCE F2b DID NOT REFUSE: the expected names were ROTATED and the \
                     ordinal check still reported agreement. `ordinal_order_agrees` is a \
                     rubber stamp, which is precisely the defect #3459 names."
                );
                m.fence_rotate_fired = true;
            }
            other => panic!(
                "FENCE F2b is MISCONFIGURED: rotating a permutation of the same name set \
                 must still PAIR (the pairing is order-independent — that is the fix), but \
                 the verdict was {other:?}"
            ),
        }
    }
}

fn measure_one(
    tc: &Toolchain,
    cpp: &Path,
    work: &Path,
    perturb: Perturb,
    pairing: Pairing,
) -> Result<Measured, String> {
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

    // ---- THE PAIRING (board #3459) ----
    //
    // Before this, the only available fence was a COUNT: `func == i+1` was
    // paired with the i-th `.text` function in address order, and if c2 walked
    // a different NUMBER of functions than `.text` contains the whole TU was
    // quarantined. Measured, not anticipated: `wkg_splice_pos.cpp` produced six
    // "failures" with mismatches in both directions (T=73 against W=14 beside
    // T=6 against W=72) — the signature of a permuted pairing, not of a
    // compiler that sometimes emits 59 extra words.
    //
    // The payload now carries the function's own name, so the pairing is READ.
    let text_names: Vec<String> = funcs.iter().map(|f| f.name.clone()).collect();
    let (by_identity, verdict) = rep.pair_by_identity(PHASE, &text_names);
    let mut m = Measured { verdict: verdict.label().to_string(), ..Default::default() };
    if let OrdinalVerdict::Verified { ordinal_order_agrees, .. } = verdict {
        m.ord_agrees = Some(ordinal_order_agrees);
    }
    if let OrdinalVerdict::Unmatched { only_in_tap, .. } = &verdict {
        m.only_in_tap = only_in_tap.clone();
    }
    run_pairing_fences(&rep, &text_names, &mut m);

    // The one case where the ordinal rule is still the honest answer: the
    // payload offered NO identity at all (a tap without #3459's field). Falling
    // back is correct; falling back SILENTLY is not, so every row of such a TU
    // is labelled `OrdinalUnverified` exactly as it was before.
    let no_identity = matches!(verdict, OrdinalVerdict::NoIdentity { .. } | OrdinalVerdict::Empty);
    let n_walks = rep.funcs.iter().filter(|x| x.phase == PHASE).count();
    let ordinals_verified = match pairing {
        // EXACTLY the pre-#3459 rule, kept executable so the two pairings can
        // be diffed on one binary and one corpus.
        Pairing::Ordinal => n_walks == funcs.len(),
        Pairing::Identity => !no_identity,
    };

    let by_ordinal = |fi: usize| {
        rep.funcs.iter().find(|x| x.phase == PHASE && x.func == (fi + 1) as u32)
    };

    let mut out_rows = Vec::new();
    for (fi, tf) in funcs.iter().enumerate() {
        let (name, w, pad) = (&tf.name, &tf.words, &tf.pad);
        let chosen = match pairing {
            Pairing::Ordinal => by_ordinal(fi),
            Pairing::Identity if no_identity => by_ordinal(fi),
            Pairing::Identity => by_identity[fi],
        };
        let Some(fw) = chosen else {
            // Counted and NAMED. Under `Identity` an absent pairing is a real
            // finding about this TU; under `Ordinal` it is the old rule running
            // off the end of the walk list, which is the same finding wearing
            // the old rule's clothes.
            m.unpaired.push(name.clone());
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
                // The cross-phase selection is the same question one level up:
                // under `Identity` a phase's walk belongs to this function when
                // it CARRIES ITS NAME, not when its ordinal happens to match.
                // `sched1` is where `g_fn` is incremented, so a phase-indexed
                // selection by ordinal is exactly where a skipped `sched1`
                // would misattribute an upstream count.
                let mine = |x: &c2_reference::stage::FuncWalk| match pairing {
                    Pairing::Identity if !no_identity => x.identity() == Some(name.as_str()),
                    _ => x.func == fw.func,
                };
                let mut v: Vec<(String, usize)> = rep
                    .funcs
                    .iter()
                    .filter(|x| mine(x))
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
    m.rows = out_rows;
    Ok(m)
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

#[derive(Default)]
struct Corpus {
    rows: Vec<Row>,
    /// `(fixture, reason)` — captures/replays that did not produce rows.
    excluded: Vec<(String, String)>,
    /// `(fixture, verdict label)` from `pair_by_identity` at [`PHASE`].
    verdicts: Vec<(String, String)>,
    /// `(fixture, did c2's ordinal order equal `.text` address order?)` —
    /// **the pre-#3459 assumption, measured**.
    ord_agrees: Vec<(String, bool)>,
    /// `(fixture, .text function name)` the tap named nothing for.
    unpaired: Vec<(String, String)>,
    /// `(fixture, tap identity)` with no `.text` function of that name.
    only_in_tap: Vec<(String, String)>,
    /// Fixtures on which F2a / F2b were exercised and refused.
    fence_wrongname: usize,
    fence_rotate: usize,
}

fn run_corpus(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    perturb: Perturb,
    jobs: usize,
    pairing: Pairing,
) -> Corpus {
    let rows = Mutex::new(Vec::new());
    let excluded = Mutex::new(Vec::new());
    let meta: Mutex<Corpus> = Mutex::new(Corpus::default());
    let next = AtomicUsize::new(0);
    // The work root must be unique per CALL, not per process. Keyed on the pid
    // alone, the two tests in this file — which cargo runs as parallel threads
    // of ONE process — shared `j0..jN` and clobbered each other's captures
    // mid-flight. The symptom was not a crash: the control pass graded 37
    // framed functions and the perturbed pass 76, from the same fixture list,
    // and the sensitivity check died on a population mismatch it reported as
    // "perturbation changed the graded population". The single-test evidence
    // runs never hit it because a filtered run has only one caller.
    static CALL: AtomicUsize = AtomicUsize::new(0);
    let base = std::env::temp_dir().join(format!(
        "c2rs-pwords-{}-{}",
        std::process::id(),
        CALL.fetch_add(1, Ordering::Relaxed)
    ));
    std::thread::scope(|s| {
        for j in 0..jobs {
            let (rows, excluded, next, base, meta) = (&rows, &excluded, &next, &base, &meta);
            s.spawn(move || {
                let w = base.join(format!("j{j}"));
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= fixtures.len() {
                        break;
                    }
                    let _ = std::fs::remove_dir_all(&w);
                    let name = fixtures[i].file_name().unwrap().to_string_lossy().to_string();
                    match measure_one(tc, &fixtures[i], &w, perturb, pairing) {
                        Ok(m) => {
                            {
                                let mut g = meta.lock().unwrap();
                                g.verdicts.push((name.clone(), m.verdict.clone()));
                                if let Some(a) = m.ord_agrees {
                                    g.ord_agrees.push((name.clone(), a));
                                }
                                for u in &m.unpaired {
                                    g.unpaired.push((name.clone(), u.clone()));
                                }
                                for u in &m.only_in_tap {
                                    g.only_in_tap.push((name.clone(), u.clone()));
                                }
                                g.fence_wrongname += usize::from(m.fence_wrongname_fired);
                                g.fence_rotate += usize::from(m.fence_rotate_fired);
                            }
                            if m.rows.is_empty() {
                                excluded.lock().unwrap().push((name, "no graded function".into()))
                            } else {
                                rows.lock().unwrap().extend(m.rows);
                            }
                        }
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
    let mut c = meta.into_inner().unwrap();
    c.rows = rows;
    c.excluded = excluded;
    c.verdicts.sort();
    c.ord_agrees.sort();
    c.unpaired.sort();
    c.only_in_tap.sort();
    c
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

    // ---- #3459: the pairing, reported rather than assumed ----
    eprintln!("\n-- the ordinal->function pairing (board #3459) --");
    let mut vk: BTreeMap<&str, usize> = BTreeMap::new();
    for (_f, v) in &c.verdicts {
        *vk.entry(v.as_str()).or_default() += 1;
    }
    for (k, n) in &vk {
        eprintln!("  verdict {:<14} {:>4}/{:<4} fixtures", k, n, c.verdicts.len());
    }
    let agree = c.ord_agrees.iter().filter(|(_, a)| *a).count();
    eprintln!(
        "  ordinal order == .text address order on {}/{} verified fixtures ({:.1}%)",
        agree,
        c.ord_agrees.len(),
        pct(agree, c.ord_agrees.len())
    );
    if agree < c.ord_agrees.len() {
        eprintln!("  ... where it does NOT (the pre-#3459 assumption, live):");
        for (f, _) in c.ord_agrees.iter().filter(|(_, a)| !*a) {
            eprintln!("      {f}");
        }
    }
    eprintln!(
        "  FENCE: F2a (fabricated name refused) fired on {} fixtures; \
         F2b (rotation caught) on {}",
        c.fence_wrongname, c.fence_rotate
    );
    // Counted and NAMED, never folded into a denominator.
    eprintln!(
        "  unpaired .text functions: {}   tap identities with no .text function: {}",
        c.unpaired.len(),
        c.only_in_tap.len()
    );
    for (f, n) in c.unpaired.iter().take(20) {
        eprintln!("      UNPAIRED {f}  {n}");
    }
    for (f, n) in c.only_in_tap.iter().take(20) {
        eprintln!("      TAP-ONLY {f}  {n}");
    }

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
    // Scoped to the VERIFIED strata. Computed over all non-leaf rows it also
    // swept in the `OrdinalUnverified` quarantine, and then printed "**NOT**
    // an exact function of P" while the hold-rates above said `R = -P` on
    // every verified row — a report contradicting itself out of a population
    // mismatch. The quarantined rows get their own line below.
    let mut map: BTreeMap<u32, BTreeMap<i64, usize>> = BTreeMap::new();
    for r in c
        .rows
        .iter()
        .filter(|r| r.stratum != Stratum::Leaf && r.stratum != Stratum::OrdinalUnverified)
    {
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
        "  => over the VERIFIED strata ({} functions), R is {} an exact function of P",
        map.values().map(|m| m.values().sum::<usize>()).sum::<usize>(),
        if functional { "**EXACTLY**" } else { "**NOT**" }
    );
    let quarantined: Vec<i64> = c
        .rows
        .iter()
        .filter(|r| r.stratum == Stratum::OrdinalUnverified)
        .filter_map(|r| r.residual())
        .collect();
    if !quarantined.is_empty() {
        eprintln!(
            "  (excluded from that: {} OrdinalUnverified residuals {:?} — an unsafe \
             pairing, not a compiler behaviour)",
            quarantined.len(),
            quarantined
        );
    }

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
        // The cap is a print budget, not a measurement bound, so it is a knob.
        // At 120 it silently truncated the alphabetical tail, which is why the
        // base run of this lane could not extract a per-row baseline at all.
        let cap: usize = std::env::var("C2RS_PWORDS_DUMP_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);
        for r in v.iter().take(cap) {
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
    let c = run_corpus(&tc, &pop, Perturb::None, jobs(), Pairing::Identity);
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

    // ---- board #3459 ----
    //
    // Positive by construction: the pairing must have been READ, not assumed.
    // `OrdinalUnverified` now means one thing only — the payload carried no
    // identity — so any row in it is the tap failing to answer, and a run that
    // reported the pairing as verified while grading nothing would be exactly
    // the vacuous green this family of instruments exists not to print.
    let unverified = c.rows.iter().filter(|r| r.stratum == Stratum::OrdinalUnverified).count();
    assert_eq!(
        unverified, 0,
        "{unverified} functions still have NO identity in the funcwalk payload — \
         board #3459 is not closed for them"
    );
    assert!(
        !c.ord_agrees.is_empty(),
        "no fixture reached a `Verified` pairing, so the identity is not being read at all"
    );
    // The fence must have been EXERCISED, not merely present. `w-pwords` §6.1
    // is the priced example of a check that passed while reading nothing.
    assert!(
        c.fence_wrongname >= 5,
        "the wrong-name fence fired on only {} fixtures — too few to call it watched",
        c.fence_wrongname
    );
    assert!(
        c.fence_rotate >= 5,
        "the rotation fence fired on only {} fixtures — too few to call it watched",
        c.fence_rotate
    );
}

/// **Board #3459's own evidence: the SAME binary and the SAME corpus, once
/// under each pairing rule.**
///
/// A base-binary-versus-tip-binary diff cannot attribute a difference to the
/// pairing, because everything else moved too. Running `Pairing::Ordinal` — a
/// faithful replay of the pre-#3459 rule, count-quarantine included — beside
/// `Pairing::Identity` on one process isolates it exactly.
///
/// **What this asserts is a DIRECTION, not a number**: the quarantine must
/// empty, and every row the old rule already graded must survive unchanged.
/// A hold-rate is not pinned here; that would turn the measurement into a gate.
#[test]
fn the_identity_pairing_empties_the_quarantine_and_moves_nothing_else() {
    let Some(tc) = ready() else { return };
    let pop = population();
    let old = run_corpus(&tc, &pop, Perturb::None, jobs(), Pairing::Ordinal);
    let new = run_corpus(&tc, &pop, Perturb::None, jobs(), Pairing::Identity);

    let key = |r: &Row| (r.fixture.clone(), r.func.clone());
    let val = |r: &Row| (r.stratum, r.t, r.w, r.i, r.p, r.pad);
    let om: BTreeMap<_, _> = old.rows.iter().map(|r| (key(r), val(r))).collect();
    let nm: BTreeMap<_, _> = new.rows.iter().map(|r| (key(r), val(r))).collect();

    let old_q: std::collections::BTreeSet<_> = old
        .rows
        .iter()
        .filter(|r| r.stratum == Stratum::OrdinalUnverified)
        .map(key)
        .collect();

    eprintln!(
        "\n===== #3459: ordinal pairing vs identity pairing, one binary, one corpus =====\n\
         rows: ordinal {} / identity {}   quarantined by the OLD rule: {}",
        old.rows.len(),
        new.rows.len(),
        old_q.len()
    );

    // 1. Every row the old rule graded OUTSIDE its quarantine must be
    //    bit-identical under the new one. This is the required-zero.
    let mut moved: Vec<String> = Vec::new();
    for (k, v) in &om {
        if old_q.contains(k) {
            continue;
        }
        match nm.get(k) {
            Some(nv) if nv == v => {}
            Some(nv) => moved.push(format!("{} {}: {v:?} -> {nv:?}", k.0, k.1)),
            None => moved.push(format!("{} {}: DROPPED", k.0, k.1)),
        }
    }
    for m in moved.iter().take(20) {
        eprintln!("  MOVED {m}");
    }
    assert!(
        moved.is_empty(),
        "{} rows outside the old quarantine changed under the identity pairing — the \
         pairing rule was supposed to touch ONLY the rows the old rule could not \
         verify",
        moved.len()
    );

    // 2. The quarantine must actually empty, and its rows must come back.
    assert!(
        !old_q.is_empty(),
        "the OLD rule quarantined nothing on this population, so this test read \
         nothing. #3459's hazard is not exhibited here — widen the population or \
         say so rather than banking a pass"
    );
    let new_q = new.rows.iter().filter(|r| r.stratum == Stratum::OrdinalUnverified).count();
    assert_eq!(new_q, 0, "the identity pairing left {new_q} rows unverified");
    for k in &old_q {
        let Some(nv) = nm.get(k) else { continue };
        let ov = om.get(k).unwrap();
        eprintln!("  RECOVERED {} {}: {ov:?} -> {nv:?}", k.0, k.1);
    }
    let recovered = old_q.iter().filter(|k| nm.contains_key(*k)).count();
    eprintln!(
        "  {recovered} of {} quarantined rows recovered; identity-pairing left {} \
         .text functions unpaired",
        old_q.len(),
        new.unpaired.len()
    );
    eprintln!("=============================================================================\n");
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
    // The SAME population the measurement test uses — deliberately not a
    // smaller slice of it.
    //
    // This test first took `.take(16)` of it, on the reasoning that a
    // sensitivity check does not need the whole corpus. **The gate caught
    // that**: that 16-fixture subset contains no framed function at all, so
    // the control had nothing to perturb and the test died on its own
    // precondition (`EXIT=101`, 1,839 passed / 1 failed). The measurement test
    // asserts `framed > 0` over `population()`, so keying this test to the
    // same population is what makes the precondition hold by construction
    // rather than by luck of where the stride landed.
    let pop: Vec<PathBuf> = population();

    let base = run_corpus(&tc, &pop, Perturb::None, jobs(), Pairing::Identity);
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
    let plus = run_corpus(&tc, &pop, Perturb::PlusOneP, jobs(), Pairing::Identity);
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

    let noflag = run_corpus(&tc, &pop, Perturb::NoInsnFilter, jobs(), Pairing::Identity);
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
