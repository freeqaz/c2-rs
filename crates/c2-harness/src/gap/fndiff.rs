//! **THE DIFF SIGNATURE** — the structure *inside* a `fnbyte-differs` body.
//!
//! # What this answers that `fnbyte-differs` cannot
//!
//! [`super::fnbytes`] grades a body and prints one forensic triple:
//! `w<port>/<ref>/eq<equal>` plus the first positionally-disagreeing word. That
//! is enough to reproduce a defect and not enough to *group* 3,195 of them. It
//! cannot say whether the port emitted the same instructions in a different
//! order, whether a word was inserted or substituted, or — when substituted —
//! whether the two words differ in their **opcode**, a **register field**, an
//! **immediate**, a **displacement** or a **branch target**. Those are different
//! defects with different fixes, and until they are separated the population is
//! one undifferentiated number.
//!
//! This module produces, per differing symbol, a **signature**: common
//! prefix/suffix, a word-granular alignment (LCS), per-substitution field
//! classification, a same-multiset bit, and whether the disagreement sits under
//! a relocation. Clustering the signatures turns "3,195 bodies are wrong" into
//! "N bodies are wrong in exactly one way", which is the smaller target.
//!
//! # The method is objdiff's, and the change of unit is what makes it cheap
//!
//! `../objdiff` aligns two functions at *instruction* granularity, diffs
//! relocation-aware, and renders per-field. On x86 that needs a full length
//! decoder; on PPC an instruction **is** a 4-byte big-endian word, so the
//! alignment is a plain LCS over `u32`s and the field decode is a bit-field
//! partition. What is deliberately **not** taken from objdiff is its scoring:
//! `docs/FUNCTION_BYTE_MATCH.md` and [`super::fnbytes`]'s module docs record why
//! partial credit inverts the correctness rule here. **Nothing in this module
//! reaches a numerator.** It is forensics attached to bodies the judge has
//! already called wrong.
//!
//! # The decode discipline: re-encode or say `undecoded`
//!
//! `docs/CODEGEN_W6_COMPARE.md` established the rule this file obeys — every
//! word decoded there was **re-encoded from its fields and compared against the
//! observed word**, all 29 of them bit-exactly. Here that rule is structural
//! rather than manual: a [`Decoded`] word is a list of [`Field`]s with explicit
//! bit ranges, and [`Decoded::reencode`] ORs them back together. A form whose
//! field partition does not cover all 32 bits, or covers one twice, cannot
//! round-trip, and [`decode`] returns `None` for it — the word is `undecoded`
//! and is **never** classified by guess. Primary opcodes this file does not
//! model (VMX/VMX128's `04`, and anything unlisted) are `None` by construction:
//! the Xbox 360's vector encodings borrow bits in ways a generic form table gets
//! wrong, and a plausible-looking wrong field is worse than an honest refusal.
//!
//! Two decode simplifications are recorded rather than hidden, because both are
//! visible in the output as an `opcode` classification:
//!
//! * **XO-form's `OE` bit is inside the extended opcode field** (bits 21–30
//!   rather than 22–30), so `addo` reads as a different opcode from `add`
//!   instead of as a flag difference. No `OE=1` word occurs in this workload's
//!   differing bodies — checked, not assumed (`fndiff-oe-set`).
//! * **Primary 63 is A-form iff bits 26–30 name one**, else X-form. That is
//!   sound rather than heuristic: the low five bits of every X-form `63` extended
//!   opcode (`fcmpu` 0, `frsp` 12, `fctiw` 14, `fctiwz` 15, `fcmpo` 32, `mtfsb1`
//!   38, `fneg` 40, `mcrfs` 64, `mtfsb0` 70, `fmr` 72, `mtfsfi` 134, `fnabs`
//!   136, `fabs` 264, `mffs` 583, `mtfsf` 711, `fctid` 814, `fctidz` 815,
//!   `fcfid` 846) lands outside the A-form set, so the two never collide.
//!
//! # It is an INSTRUMENT and never a gate
//!
//! It licenses no emit, appears in no accept/refuse path, changes no existing
//! count, and runs only on bodies already bucketed `fnbyte-differs`. Its own
//! correctness is checked by a **positive per-row identity** — `equal + sub +
//! del == ref_words` and `equal + sub + ins == port_words` — counted as
//! `fndiff-accounting-broken` (known answer 0) rather than left to be inferred
//! from an absence.

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// The field model
// ---------------------------------------------------------------------------

/// What a field *is*, which is what a difference in it **means**.
///
/// The classification of a substituted instruction pair is a function of the
/// kinds of the fields that moved and nothing else — so this enum is the whole
/// vocabulary of the cluster table's `class` column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// Primary or extended opcode. A difference here means *a different
    /// instruction*, not a different operand.
    Op,
    /// A GPR/FPR number.
    Reg,
    /// An immediate operand (`addi`'s `SI`, `ori`'s `UI`, …).
    Imm,
    /// A memory displacement (`lwz`/`stw`'s `D`, `std`'s `DS`).
    Disp,
    /// A branch displacement (`b`'s `LI`, `bc`'s `BD`).
    Target,
    /// A shift amount.
    Shift,
    /// A mask field (`MB`/`ME`/`FXM`).
    Mask,
    /// A condition-register field or bit selector.
    Cr,
    /// A special-purpose register number (`mfspr`/`mtspr`'s split `SPR`).
    Spr,
    /// `Rc`/`OE`/`AA`/`LK`, a reserved bit, or an operand-less flag field.
    Flag,
}

impl Kind {
    fn tag(self) -> &'static str {
        match self {
            Kind::Op => "opcode",
            Kind::Reg => "reg",
            Kind::Imm => "imm",
            Kind::Disp => "disp",
            Kind::Target => "branch-target",
            Kind::Shift => "shift",
            Kind::Mask => "mask",
            Kind::Cr => "cr-field",
            Kind::Spr => "spr",
            Kind::Flag => "flag",
        }
    }
}

/// One bit-field of one instruction word.
///
/// `hi`/`lo` are **PPC bit numbers**: bit 0 is the most significant bit of the
/// 32-bit word and `hi <= lo`. They are stored rather than derived so
/// [`Decoded::reencode`] can put the value back exactly where it came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub kind: Kind,
    pub hi: u8,
    pub lo: u8,
    pub val: u32,
}

impl Field {
    fn width(&self) -> u32 {
        (self.lo - self.hi + 1) as u32
    }
    fn shift(&self) -> u32 {
        31 - self.lo as u32
    }
}

/// A word, decoded into a field partition that provably covers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    /// The form's name — `D`, `DS`, `I`, `B`, `XL`, `X`, `XO-31`, `M`, `MD`,
    /// `MDS`, `A`. Two words of **different forms** are always an `opcode`
    /// difference, never a field one.
    pub form: &'static str,
    pub fields: Vec<Field>,
}

impl Decoded {
    /// Reassemble the word from the fields alone. The round-trip check.
    pub fn reencode(&self) -> u32 {
        let mut w = 0u32;
        for f in &self.fields {
            w |= (f.val & mask(f.width())) << f.shift();
        }
        w
    }

    fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }
}

fn mask(width: u32) -> u32 {
    if width >= 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    }
}

fn bits(w: u32, hi: u8, lo: u8) -> u32 {
    (w >> (31 - lo as u32)) & mask((lo - hi + 1) as u32)
}

fn f(name: &'static str, kind: Kind, hi: u8, lo: u8, w: u32) -> Field {
    Field {
        name,
        kind,
        hi,
        lo,
        val: bits(w, hi, lo),
    }
}

/// The A-form extended opcodes of primary 59/63 — see the module docs for why
/// membership in this set is a *sound* discriminator against X-form and not a
/// heuristic.
const A_FORM_XO: [u32; 12] = [18, 20, 21, 22, 23, 24, 25, 26, 28, 29, 30, 31];

/// **Decode one big-endian instruction word**, or `None` if this file does not
/// model its form.
///
/// The returned partition is verified here: a form whose fields do not
/// reassemble into the original word is reported as **undecoded** rather than
/// returned, so no caller can be handed a field list that does not describe the
/// bytes it came from.
pub fn decode(w: u32) -> Option<Decoded> {
    let p = bits(w, 0, 5);
    let opcd = f("OPCD", Kind::Op, 0, 5, w);
    let (form, mut fields): (&'static str, Vec<Field>) = match p {
        // twi — `TO` is a trap condition selector, not a register.
        3 => (
            "D",
            vec![
                f("TO", Kind::Flag, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("SI", Kind::Imm, 16, 31, w),
            ],
        ),
        // mulli, subfic, addic, addic., addi, addis
        7 | 8 | 12 | 13 | 14 | 15 => (
            "D",
            vec![
                f("RT", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("SI", Kind::Imm, 16, 31, w),
            ],
        ),
        // ori/oris/xori/xoris/andi./andis. — destination is RA, source RS.
        24 | 25 | 26 | 27 | 28 | 29 => (
            "D",
            vec![
                f("RS", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("UI", Kind::Imm, 16, 31, w),
            ],
        ),
        // cmpli / cmpi — bits 6..10 are BF/./L, NOT a register. Modelled apart
        // so a CR-field move is not reported as a register move.
        10 | 11 => (
            "D",
            vec![
                f("BF", Kind::Cr, 6, 8, w),
                f("rsv9", Kind::Flag, 9, 9, w),
                f("L", Kind::Flag, 10, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("IMM", Kind::Imm, 16, 31, w),
            ],
        ),
        // The D-form load/store block, integer (32..47) and float (48..55).
        32..=55 => (
            "D",
            vec![
                f("RST", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("D", Kind::Disp, 16, 31, w),
            ],
        ),
        // DS-form: ld/ldu/lwa (58), std/stdu (62).
        58 | 62 => (
            "DS",
            vec![
                f("RST", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("DS", Kind::Disp, 16, 29, w),
                f("XO", Kind::Op, 30, 31, w),
            ],
        ),
        // bc — the conditional branch.
        16 => (
            "B",
            vec![
                f("BO", Kind::Flag, 6, 10, w),
                f("BI", Kind::Cr, 11, 15, w),
                f("BD", Kind::Target, 16, 29, w),
                f("AA", Kind::Flag, 30, 30, w),
                f("LK", Kind::Flag, 31, 31, w),
            ],
        ),
        // b / bl / ba / bla.
        18 => (
            "I",
            vec![
                f("LI", Kind::Target, 6, 29, w),
                f("AA", Kind::Flag, 30, 30, w),
                f("LK", Kind::Flag, 31, 31, w),
            ],
        ),
        // bclr/bcctr and the CR-logical block. The three operand fields are a
        // BO/BI/BH triple or a BT/BA/BB triple depending on XO; both are
        // condition-register selectors, so the kinds are right either way.
        19 => (
            "XL",
            vec![
                f("BO", Kind::Flag, 6, 10, w),
                f("BI", Kind::Cr, 11, 15, w),
                f("BH", Kind::Cr, 16, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("LK", Kind::Flag, 31, 31, w),
            ],
        ),
        // M-form. `rlwnm` (23) takes its shift from a register; `rlwimi` (20)
        // and `rlwinm` (21) take an immediate one.
        20 | 21 | 23 => (
            "M",
            vec![
                f("RS", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                if p == 23 {
                    f("RB", Kind::Reg, 16, 20, w)
                } else {
                    f("SH", Kind::Shift, 16, 20, w)
                },
                f("MB", Kind::Mask, 21, 25, w),
                f("ME", Kind::Mask, 26, 30, w),
                f("Rc", Kind::Flag, 31, 31, w),
            ],
        ),
        // MD / MDS — the 64-bit rotates. MDS is XO 8/9 in bits 27..30; every
        // other value there is MD, whose XO is bits 27..29 with the shift's
        // high bit at 30.
        30 => {
            let x4 = bits(w, 27, 30);
            if x4 == 8 || x4 == 9 {
                (
                    "MDS",
                    vec![
                        f("RS", Kind::Reg, 6, 10, w),
                        f("RA", Kind::Reg, 11, 15, w),
                        f("RB", Kind::Reg, 16, 20, w),
                        f("mb", Kind::Mask, 21, 26, w),
                        f("XO", Kind::Op, 27, 30, w),
                        f("Rc", Kind::Flag, 31, 31, w),
                    ],
                )
            } else {
                (
                    "MD",
                    vec![
                        f("RS", Kind::Reg, 6, 10, w),
                        f("RA", Kind::Reg, 11, 15, w),
                        f("sh", Kind::Shift, 16, 20, w),
                        f("mb", Kind::Mask, 21, 26, w),
                        f("XO", Kind::Op, 27, 29, w),
                        f("sh2", Kind::Shift, 30, 30, w),
                        f("Rc", Kind::Flag, 31, 31, w),
                    ],
                )
            }
        }
        31 => decode_31(w),
        // Primary 59 is A-form throughout (fdivs/fsubs/fadds/fmuls/fmadds/…).
        59 => ("A", a_form(w)),
        63 => {
            if A_FORM_XO.contains(&bits(w, 26, 30)) {
                ("A", a_form(w))
            } else {
                (
                    "X",
                    vec![
                        f("FRT", Kind::Reg, 6, 10, w),
                        f("FRA", Kind::Reg, 11, 15, w),
                        f("FRB", Kind::Reg, 16, 20, w),
                        f("XO", Kind::Op, 21, 30, w),
                        f("Rc", Kind::Flag, 31, 31, w),
                    ],
                )
            }
        }
        _ => return None,
    };
    fields.insert(0, opcd);
    let d = Decoded { form, fields };
    // **The round-trip.** A partition that does not reassemble the word is not
    // a decode of it. `docs/CODEGEN_W6_COMPARE.md`'s rule, enforced structurally
    // rather than by hand.
    if d.reencode() != w {
        return None;
    }
    Some(d)
}

fn a_form(w: u32) -> Vec<Field> {
    vec![
        f("FRT", Kind::Reg, 6, 10, w),
        f("FRA", Kind::Reg, 11, 15, w),
        f("FRB", Kind::Reg, 16, 20, w),
        f("FRC", Kind::Reg, 21, 25, w),
        f("XO", Kind::Op, 26, 30, w),
        f("Rc", Kind::Flag, 31, 31, w),
    ]
}

/// Primary 31 — the integer X/XO block, with the four shapes whose operand
/// fields are **not** three registers modelled apart.
///
/// Each special case exists because the generic partition would report a real
/// difference under the wrong `Kind`: a `cmp`'s CR-field move would read as a
/// register move, an `mfspr`'s SPR number would read as two register moves, and
/// an `srawi`'s shift amount would read as a register.
fn decode_31(w: u32) -> (&'static str, Vec<Field>) {
    let xo = bits(w, 21, 30);
    match xo {
        // cmp (0) / cmpl (32)
        0 | 32 => (
            "X",
            vec![
                f("BF", Kind::Cr, 6, 8, w),
                f("rsv9", Kind::Flag, 9, 9, w),
                f("L", Kind::Flag, 10, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("RB", Kind::Reg, 16, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("rsv31", Kind::Flag, 31, 31, w),
            ],
        ),
        // mfspr (339) / mtspr (467) — the SPR number is one 10-bit field split
        // across two halves on the wire, and it selects LR/CTR/XER.
        339 | 467 => (
            "XFX",
            vec![
                f("RST", Kind::Reg, 6, 10, w),
                f("SPR", Kind::Spr, 11, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("Rc", Kind::Flag, 31, 31, w),
            ],
        ),
        // mtcrf (144) — an 8-bit field mask, not a register.
        144 => (
            "XFX",
            vec![
                f("RS", Kind::Reg, 6, 10, w),
                f("rsv11", Kind::Flag, 11, 11, w),
                f("FXM", Kind::Mask, 12, 19, w),
                f("rsv20", Kind::Flag, 20, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("Rc", Kind::Flag, 31, 31, w),
            ],
        ),
        // srawi (824) — an immediate shift amount in the RB position.
        824 => (
            "X",
            vec![
                f("RS", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("SH", Kind::Shift, 16, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("Rc", Kind::Flag, 31, 31, w),
            ],
        ),
        _ => (
            "X",
            vec![
                f("RT", Kind::Reg, 6, 10, w),
                f("RA", Kind::Reg, 11, 15, w),
                f("RB", Kind::Reg, 16, 20, w),
                f("XO", Kind::Op, 21, 30, w),
                f("Rc", Kind::Flag, 31, 31, w),
            ],
        ),
    }
}

/// `true` when a primary-31 word sets `OE` — the one bit the XO-form
/// simplification in the module docs folds into the extended opcode. Counted on
/// every scan so the simplification is a measured non-issue rather than an
/// assumed one.
pub fn oe_set(w: u32) -> bool {
    bits(w, 0, 5) == 31 && bits(w, 21, 21) == 1
}

// ---------------------------------------------------------------------------
// Pair classification
// ---------------------------------------------------------------------------

/// How two aligned, unequal instruction words differ.
///
/// `fields` names the fields that moved, so a `reg` cluster can be split
/// further (`RT` alone is a destination-register choice; `RA`/`RB` alone is an
/// operand choice) without re-running the scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairClass {
    pub class: String,
    pub fields: Vec<&'static str>,
}

/// Classify one substituted pair. `undecoded` whenever either word is a form
/// this file does not model — never a guess.
pub fn classify_pair(port: u32, refw: u32) -> PairClass {
    let (Some(a), Some(b)) = (decode(port), decode(refw)) else {
        return PairClass {
            class: "undecoded".to_string(),
            fields: Vec::new(),
        };
    };
    // **Different instruction ⇒ `opcode`, decided before any field is compared.**
    //
    // This ordering is not cosmetic. The first version of this function fell
    // straight to the per-field loop and returned `undecoded` whenever a field
    // name was missing on the other side — and `addi` (primary 14, fields
    // `RT`/`RA`/`SI`) and `lwz` (primary 32, fields `RST`/`RA`/`D`) share the
    // form name `D` and share **no** field names, so every `addi`-vs-`lwz`
    // substitution was reported as a word this decoder could not read. All
    // **470** `undecoded` words in the first dc3 census were that: two words
    // that each decode perfectly and are simply different instructions. The
    // measurement said "9.1 % of the mismatched words are unreadable" when the
    // true figure was **0**. `undecoded` now means exactly one thing — a word
    // whose form this file does not model — which is what makes it usable as
    // the answer to "do we understand the layout".
    if a.field("OPCD").map(|f| f.val) != b.field("OPCD").map(|f| f.val) {
        return PairClass {
            class: "opcode".to_string(),
            fields: vec!["OPCD"],
        };
    }
    if a.form != b.form {
        return PairClass {
            class: "opcode".to_string(),
            fields: vec!["FORM"],
        };
    }
    let mut kinds: Vec<Kind> = Vec::new();
    let mut names: Vec<&'static str> = Vec::new();
    for fa in &a.fields {
        // Same primary and same form, different field layout: one of the
        // extended-opcode special cases in [`decode_31`] (a `cmp` against an
        // `srawi`, say). A different layout under one primary is always a
        // different extended opcode, so this is an `opcode` difference — never
        // an unreadable word.
        let Some(fb) = b.field(fa.name) else {
            return PairClass {
                class: "opcode".to_string(),
                fields: vec!["LAYOUT"],
            };
        };
        if fa.val != fb.val {
            if !kinds.contains(&fa.kind) {
                kinds.push(fa.kind);
            }
            names.push(fa.name);
        }
    }
    kinds.sort_unstable();
    let class = if kinds.contains(&Kind::Op) {
        "opcode".to_string()
    } else {
        match kinds.as_slice() {
            [] => "equal".to_string(),
            [k] => k.tag().to_string(),
            many => {
                let mut s = String::from("mixed:");
                for (i, k) in many.iter().enumerate() {
                    if i > 0 {
                        s.push('+');
                    }
                    s.push_str(k.tag());
                }
                s
            }
        }
    };
    PairClass {
        class,
        fields: names,
    }
}

// ---------------------------------------------------------------------------
// Alignment
// ---------------------------------------------------------------------------

/// One aligned edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Both bodies have this word at aligned positions.
    Equal(usize, usize),
    /// Aligned positions hold different words: `(port index, ref index)`.
    Sub(usize, usize),
    /// The port has a word c2 does not.
    Insert(usize),
    /// c2 has a word the port does not.
    Delete(usize),
}

/// The LCS cell budget. Bodies here are tens of words; the cap exists so one
/// pathological pair cannot dominate a scan, and it is **counted**
/// (`fndiff-align-capped`) rather than silently degrading the table.
const LCS_CELL_CAP: usize = 400_000;

/// Align two word sequences at instruction granularity.
///
/// Common prefix and suffix are stripped first — that is both a large speedup
/// and the two numbers the report wants anyway — and the interior is aligned by
/// LCS. Adjacent delete/insert runs are then **paired into substitutions**,
/// which is what makes the per-word field classification possible at all: an
/// unpaired insert has no counterpart to compare fields against.
///
/// Returns `(edits, capped)`. When `capped`, the interior is aligned
/// positionally instead of by LCS and the flag is propagated into the row.
pub fn align(port: &[u32], refw: &[u32], cap: usize) -> (Vec<Edit>, bool) {
    let n = port.len();
    let m = refw.len();
    let mut pre = 0usize;
    while pre < n && pre < m && port[pre] == refw[pre] {
        pre += 1;
    }
    let mut suf = 0usize;
    while suf < n - pre && suf < m - pre && port[n - 1 - suf] == refw[m - 1 - suf] {
        suf += 1;
    }
    let a = &port[pre..n - suf];
    let b = &refw[pre..m - suf];
    let mut edits: Vec<Edit> = (0..pre).map(|i| Edit::Equal(i, i)).collect();
    let capped = a.len().saturating_mul(b.len()) > cap;
    let mut interior: Vec<Edit> = Vec::new();
    if capped {
        // Positional fallback: the honest degraded reading, and it is flagged.
        let k = a.len().min(b.len());
        for i in 0..k {
            interior.push(Edit::Sub(pre + i, pre + i));
        }
        for i in k..a.len() {
            interior.push(Edit::Insert(pre + i));
        }
        for j in k..b.len() {
            interior.push(Edit::Delete(pre + j));
        }
    } else {
        // Classic LCS DP, then a backtrack that emits deletes before inserts so
        // the pairing pass below sees them adjacent.
        let (la, lb) = (a.len(), b.len());
        let mut dp = vec![0u32; (la + 1) * (lb + 1)];
        for i in (0..la).rev() {
            for j in (0..lb).rev() {
                dp[i * (lb + 1) + j] = if a[i] == b[j] {
                    dp[(i + 1) * (lb + 1) + j + 1] + 1
                } else {
                    dp[(i + 1) * (lb + 1) + j].max(dp[i * (lb + 1) + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0usize, 0usize);
        while i < la && j < lb {
            if a[i] == b[j] {
                interior.push(Edit::Equal(pre + i, pre + j));
                i += 1;
                j += 1;
            } else if dp[(i + 1) * (lb + 1) + j] >= dp[i * (lb + 1) + j + 1] {
                interior.push(Edit::Insert(pre + i));
                i += 1;
            } else {
                interior.push(Edit::Delete(pre + j));
                j += 1;
            }
        }
        while i < la {
            interior.push(Edit::Insert(pre + i));
            i += 1;
        }
        while j < lb {
            interior.push(Edit::Delete(pre + j));
            j += 1;
        }
        interior = pair_runs(interior);
    }
    edits.extend(interior);
    for k in 0..suf {
        edits.push(Edit::Equal(n - suf + k, m - suf + k));
    }
    (edits, capped)
}

/// Pair each maximal run of `Insert`s against an adjacent run of `Delete`s,
/// one for one, turning the overlap into `Sub`s.
///
/// LCS produces only insert/delete. A one-word register change would therefore
/// read as "one insert and one delete" — two edits and no field comparison —
/// when it is plainly one substitution, which is the reading the whole cluster
/// table depends on.
fn pair_runs(edits: Vec<Edit>) -> Vec<Edit> {
    let mut out: Vec<Edit> = Vec::with_capacity(edits.len());
    let mut k = 0usize;
    while k < edits.len() {
        // Collect a maximal delete-run followed by a maximal insert-run, or an
        // insert-run followed by a delete-run — both orders occur.
        let mut dels: Vec<usize> = Vec::new();
        let mut inss: Vec<usize> = Vec::new();
        let start = k;
        while k < edits.len() {
            match edits[k] {
                Edit::Delete(j) => dels.push(j),
                Edit::Insert(i) => inss.push(i),
                Edit::Equal(..) | Edit::Sub(..) => break,
            }
            k += 1;
        }
        if k == start {
            out.push(edits[k]);
            k += 1;
            continue;
        }
        let paired = dels.len().min(inss.len());
        for t in 0..paired {
            out.push(Edit::Sub(inss[t], dels[t]));
        }
        for &i in inss.iter().skip(paired) {
            out.push(Edit::Insert(i));
        }
        for &j in dels.iter().skip(paired) {
            out.push(Edit::Delete(j));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The signature
// ---------------------------------------------------------------------------

/// The per-symbol diff signature. One of these per `fnbyte-differs` row.
#[derive(Clone, Debug)]
pub struct DiffSig {
    pub tu: String,
    pub sym: String,
    /// The `Selected` variant the port chose — `tail`/`seq`/`framed`/…
    pub shape: &'static str,
    pub port_words: usize,
    pub ref_words: usize,
    /// Words equal at the head before anything disagrees.
    pub prefix: usize,
    /// Words equal at the tail after everything disagrees.
    pub suffix: usize,
    /// Index of the first aligned disagreement, in **reference** words.
    pub first: usize,
    pub equal: usize,
    pub sub: usize,
    pub ins: usize,
    pub del: usize,
    /// The port's instruction multiset equals c2's — a pure reordering.
    pub same_multiset: bool,
    pub capped: bool,
    /// Per-substitution field class counts.
    pub classes: BTreeMap<String, usize>,
    /// Substitutions whose reference word sits under a relocation.
    pub sub_at_reloc: usize,
    /// Deletions (words only c2 emitted) under a relocation.
    pub del_at_reloc: usize,
    /// Relocation records whose `VirtualAddress` is not word-aligned.
    pub reloc_unaligned: usize,
    pub reloc_count: usize,
    /// A primary-31 word with `OE` set on either side — the one decode
    /// simplification, measured.
    pub oe_seen: bool,
    /// Up to [`SUB_SAMPLE`] worked substitutions, for the report's side-by-side.
    pub samples: Vec<(usize, usize, u32, u32, String, Vec<&'static str>)>,
    pub samples_truncated: bool,
    /// Words the port emitted and c2 did not (first few), and vice versa.
    pub ins_words: Vec<u32>,
    pub del_words: Vec<u32>,
    /// **The positive accounting identity.** `false` means this row's alignment
    /// does not add up and the row must not be believed.
    pub accounting_ok: bool,
}

/// How many substitutions a row carries verbatim. Enough to decode a cluster's
/// worked example by hand; short enough that the JSONL stays a text file.
pub const SUB_SAMPLE: usize = 8;

fn words(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4)
        .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Compute one signature. `relocs` are the **reference** COMDAT's relocation
/// `(VirtualAddress, type)` pairs — under `/Gy` the address is the offset inside
/// this body (see [`c2_obj::ObjImage::text_comdat_reloc_sites`]).
pub fn signature(
    tu: &str,
    sym: &str,
    shape: &'static str,
    port: &[u8],
    reference: &[u8],
    relocs: &[(u32, u16)],
) -> DiffSig {
    let p = words(port);
    let r = words(reference);
    let (edits, capped) = align(&p, &r, LCS_CELL_CAP);

    let mut reloc_words: Vec<usize> = Vec::new();
    let mut reloc_unaligned = 0usize;
    for (va, _) in relocs {
        if va % 4 != 0 {
            reloc_unaligned += 1;
        }
        reloc_words.push((*va / 4) as usize);
    }

    let mut sig = DiffSig {
        tu: tu.to_string(),
        sym: sym.to_string(),
        shape,
        port_words: p.len(),
        ref_words: r.len(),
        prefix: 0,
        suffix: 0,
        first: usize::MAX,
        equal: 0,
        sub: 0,
        ins: 0,
        del: 0,
        same_multiset: {
            let (mut a, mut b) = (p.clone(), r.clone());
            a.sort_unstable();
            b.sort_unstable();
            a == b
        },
        capped,
        classes: BTreeMap::new(),
        sub_at_reloc: 0,
        del_at_reloc: 0,
        reloc_unaligned,
        reloc_count: relocs.len(),
        oe_seen: false,
        samples: Vec::new(),
        samples_truncated: false,
        ins_words: Vec::new(),
        del_words: Vec::new(),
        accounting_ok: false,
    };

    let mut seen_diff = false;
    for e in &edits {
        match *e {
            Edit::Equal(..) => {
                sig.equal += 1;
                if !seen_diff {
                    sig.prefix += 1;
                }
            }
            Edit::Sub(i, j) => {
                seen_diff = true;
                sig.sub += 1;
                if sig.first == usize::MAX {
                    sig.first = j;
                }
                let (pw, rw) = (p[i], r[j]);
                if oe_set(pw) || oe_set(rw) {
                    sig.oe_seen = true;
                }
                let pc = classify_pair(pw, rw);
                *sig.classes.entry(pc.class.clone()).or_insert(0) += 1;
                if reloc_words.contains(&j) {
                    sig.sub_at_reloc += 1;
                }
                if sig.samples.len() < SUB_SAMPLE {
                    sig.samples.push((i, j, pw, rw, pc.class, pc.fields));
                } else {
                    sig.samples_truncated = true;
                }
            }
            Edit::Insert(i) => {
                seen_diff = true;
                sig.ins += 1;
                if sig.ins_words.len() < SUB_SAMPLE {
                    sig.ins_words.push(p[i]);
                }
            }
            Edit::Delete(j) => {
                seen_diff = true;
                sig.del += 1;
                if sig.first == usize::MAX {
                    sig.first = j;
                }
                if reloc_words.contains(&j) {
                    sig.del_at_reloc += 1;
                }
                if sig.del_words.len() < SUB_SAMPLE {
                    sig.del_words.push(r[j]);
                }
            }
        }
    }
    // Trailing equal run.
    let mut suf = 0usize;
    for e in edits.iter().rev() {
        match e {
            Edit::Equal(..) => suf += 1,
            _ => break,
        }
    }
    sig.suffix = suf;
    if sig.first == usize::MAX {
        sig.first = 0;
    }
    // **The identity, positively.** `equal + sub + del == ref_words` and
    // `equal + sub + ins == port_words`. A broken alignment would still produce
    // a tidy-looking cluster table, so this is checked per row and counted.
    sig.accounting_ok = sig.equal + sig.sub + sig.del == sig.ref_words
        && sig.equal + sig.sub + sig.ins == sig.port_words;
    sig
}

impl DiffSig {
    /// The **coarse cluster key** — the column the report's table is grouped by.
    ///
    /// Deliberately coarser than the row: shape, the length relation, the edit
    /// shape, and the *set* of field classes present. Counts are excluded so
    /// that "one register field is wrong" and "three register fields are wrong"
    /// land in the same cluster, which is the grouping a fix lane wants.
    pub fn csig(&self) -> String {
        let lenrel = match self.port_words.cmp(&self.ref_words) {
            std::cmp::Ordering::Equal => "same-len",
            std::cmp::Ordering::Greater => "port-longer",
            std::cmp::Ordering::Less => "ref-longer",
        };
        let editshape = match (self.sub > 0, self.ins > 0, self.del > 0) {
            (true, false, false) => "sub-only",
            (false, true, false) => "ins-only",
            (false, false, true) => "del-only",
            (true, true, false) => "sub+ins",
            (true, false, true) => "sub+del",
            (false, true, true) => "ins+del",
            (true, true, true) => "sub+ins+del",
            (false, false, false) => "none",
        };
        let classes = if self.classes.is_empty() {
            "-".to_string()
        } else {
            self.classes.keys().cloned().collect::<Vec<_>>().join("+")
        };
        format!(
            "{}|{lenrel}|{editshape}|{classes}{}",
            self.shape,
            if self.same_multiset { "|reorder" } else { "" }
        )
    }

    /// The **fine signature** — the coarse key with the edit counts and the
    /// first-divergence index, for the long tail.
    pub fn sig(&self) -> String {
        format!(
            "{}|first@{}|{}s{}i{}d",
            self.csig(),
            self.first,
            self.sub,
            self.ins,
            self.del
        )
    }

    /// The first-divergence bucket, so the histogram has a bounded row count.
    pub fn first_bucket(&self) -> String {
        match self.first {
            0..=7 => self.first.to_string(),
            8..=15 => "8-15".to_string(),
            16..=31 => "16-31".to_string(),
            _ => "32+".to_string(),
        }
    }

    /// One JSONL row. Hand-rolled, like every other JSON in this workspace.
    pub fn to_json(&self) -> String {
        let hex = |w: u32| format!("{w:08x}");
        let classes = self
            .classes
            .iter()
            .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
            .collect::<Vec<_>>()
            .join(",");
        let samples = self
            .samples
            .iter()
            .map(|(i, j, pw, rw, cls, flds)| {
                format!(
                    "{{\"pi\":{i},\"ri\":{j},\"port\":\"{}\",\"ref\":\"{}\",\"class\":{},\"fields\":[{}]}}",
                    hex(*pw),
                    hex(*rw),
                    crate::jstr(cls),
                    flds.iter()
                        .map(|s| crate::jstr(s))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let wlist = |v: &[u32]| {
            v.iter()
                .map(|w| format!("\"{}\"", hex(*w)))
                .collect::<Vec<_>>()
                .join(",")
        };
        format!(
            "{{\"tu\":{},\"sym\":{},\"shape\":{},\"port_words\":{},\"ref_words\":{},\
             \"prefix\":{},\"suffix\":{},\"first\":{},\"equal\":{},\"sub\":{},\"ins\":{},\
             \"del\":{},\"same_multiset\":{},\"capped\":{},\"classes\":{{{}}},\
             \"sub_at_reloc\":{},\"del_at_reloc\":{},\"reloc_count\":{},\
             \"reloc_unaligned\":{},\"oe\":{},\"accounting_ok\":{},\
             \"samples\":[{}],\"samples_truncated\":{},\"ins_words\":[{}],\
             \"del_words\":[{}],\"csig\":{},\"sig\":{}}}",
            crate::jstr(&self.tu),
            crate::jstr(&self.sym),
            crate::jstr(self.shape),
            self.port_words,
            self.ref_words,
            self.prefix,
            self.suffix,
            self.first,
            self.equal,
            self.sub,
            self.ins,
            self.del,
            self.same_multiset,
            self.capped,
            classes,
            self.sub_at_reloc,
            self.del_at_reloc,
            self.reloc_count,
            self.reloc_unaligned,
            self.oe_seen,
            self.accounting_ok,
            samples,
            self.samples_truncated,
            wlist(&self.ins_words),
            wlist(&self.del_words),
            crate::jstr(&self.csig()),
            crate::jstr(&self.sig()),
        )
    }

    /// The scan counters this row contributes. Every key is `fndiff-`-prefixed
    /// and additive; nothing here reads or writes an existing count.
    pub fn keys(&self) -> Vec<(String, usize)> {
        let mut v = vec![
            ("fndiff-rows".to_string(), 1),
            (format!("fndiff-csig|{}", self.csig()), 1),
            (format!("fndiff-shape|{}", self.shape), 1),
            (format!("fndiff-first|{}", self.first_bucket()), 1),
            ("fndiff-sub".to_string(), self.sub),
            ("fndiff-ins".to_string(), self.ins),
            ("fndiff-del".to_string(), self.del),
            ("fndiff-equal".to_string(), self.equal),
            ("fndiff-prefix".to_string(), self.prefix),
            ("fndiff-suffix".to_string(), self.suffix),
            ("fndiff-sub-at-reloc".to_string(), self.sub_at_reloc),
            ("fndiff-del-at-reloc".to_string(), self.del_at_reloc),
            ("fndiff-reloc-unaligned".to_string(), self.reloc_unaligned),
        ];
        for (k, n) in &self.classes {
            v.push((format!("fndiff-class|{k}"), *n));
        }
        if self.same_multiset {
            v.push(("fndiff-same-multiset".to_string(), 1));
        }
        if self.capped {
            v.push(("fndiff-align-capped".to_string(), 1));
        }
        if self.oe_seen {
            v.push(("fndiff-oe-set".to_string(), 1));
        }
        if !self.accounting_ok {
            v.push(("fndiff-accounting-broken".to_string(), 1));
        }
        if self.prefix == 0 {
            v.push(("fndiff-first-word".to_string(), 1));
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_core::codegen::encode::*;

    /// **The round-trip, over the port's own encoders.** Every word the port can
    /// emit is decoded here and reassembled from its fields; a form whose
    /// partition is wrong cannot survive this. This is the mechanized form of
    /// `docs/CODEGEN_W6_COMPARE.md`'s hand method.
    #[test]
    fn every_port_encoder_word_round_trips() {
        let mut words: Vec<[u8; 4]> = vec![
            encode_add(3, 4, 5),
            encode_mullw(3, 4, 5),
            encode_subf(3, 4, 5),
            encode_and(3, 11, 5),
            encode_or(3, 11, 10),
            encode_xor(3, 11, 5),
            encode_slw(3, 11, 5),
            encode_srw(3, 11, 5),
            encode_sraw(11, 3, 4),
            encode_addi(3, 0, -7),
            encode_addis(3, 0, 0x1234u16 as i16),
            encode_ori(3, 3, 0xbeef),
            encode_blr(),
            encode_bclr(4, 2),
            encode_lwz(3, 1, 80),
            encode_lbz(3, 1, 80),
            encode_lhz(3, 1, 80),
            encode_ld(3, 1, 80),
            encode_extsb(3, 4),
            encode_extsb_record(3, 4),
            encode_extsh(3, 4),
            encode_stw(3, 1, 80),
            encode_stb(3, 1, 80),
            encode_sth(3, 1, 80),
            encode_addic(3, 4, -1),
            encode_subfic(3, 4, 0),
            encode_subfc(3, 4, 5),
            encode_subfe(3, 4, 5),
            encode_addze(3, 4),
            encode_adde(3, 4, 5),
            encode_subfze(3, 4),
            encode_srawi(3, 4, 31),
            encode_neg(3, 4),
            encode_andc(3, 4, 5),
            encode_orc(3, 4, 5),
            encode_eqv(3, 4, 5),
            encode_cntlzw(3, 4),
            encode_xori(3, 4, 0x1111),
            encode_rlwinm(3, 4, 5, 6, 31),
            encode_rlwimi(3, 4, 5, 6, 31),
            encode_srwi31(3, 4),
            encode_clrlwi31(3, 4),
            encode_fadd(true, 1, 2, 3),
            encode_fsub(true, 1, 2, 3),
            encode_fmul(true, 1, 2, 3),
            encode_fdiv(true, 1, 2, 3),
            encode_fadd(false, 1, 2, 3),
            encode_fmul(false, 1, 2, 3),
            encode_stfs(true, 1, 1, 16),
            encode_fmr(1, 2),
            encode_frsp(1, 2),
            encode_lfs(true, 1, 1, 16),
            encode_std(3, 1, 80),
            encode_stfd(1, 1, 16),
            encode_lfd(1, 1, 16),
            encode_stwu(1, 1, -80),
            encode_mr(4, 3),
            encode_mr_record(4, 3),
            encode_mulli(3, 4, 10),
            encode_lbzu(3, 4, 1),
            encode_divw(3, 4, 5),
            encode_divwu(3, 4, 5),
            encode_twi(6, 4, 0),
            encode_cmpwi(0, 3, 7),
            encode_cmplwi(0, 3, 7),
        ];
        words.push(encode_bc(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_EQ), 8).unwrap());
        words.push(encode_b_intra(-16).unwrap());
        let mut undecoded = Vec::new();
        for w in &words {
            let word = u32::from_be_bytes(*w);
            match decode(word) {
                Some(d) => assert_eq!(d.reencode(), word, "field partition lost bits in {word:08x}"),
                None => undecoded.push(format!("{word:08x}")),
            }
        }
        assert!(
            undecoded.is_empty(),
            "the port emits words this decoder does not model: {undecoded:?}"
        );
    }

    /// The prologue/epilogue words the port's frame emitter writes, spelled as
    /// literals because they come from captured objs rather than an encoder:
    /// `mflr r12`, `mtlr r12`, `blr`, `stwu`, `b`.
    #[test]
    fn captured_frame_words_decode_and_name_their_fields() {
        // mflr r12 = mfspr r12,8 — the SPR number is one field, not two regs.
        let mflr = 0x7d88_02a6u32;
        let d = decode(mflr).expect("mflr decodes");
        assert_eq!(d.form, "XFX");
        assert_eq!(d.reencode(), mflr);
        assert_eq!(d.field("SPR").unwrap().val, 0x100); // SPR 8, halves swapped
        assert_eq!(d.field("SPR").unwrap().kind, Kind::Spr);
        // mtlr r12
        let mtlr = 0x7d88_03a6u32;
        assert_eq!(decode(mtlr).unwrap().reencode(), mtlr);
        // blr
        assert_eq!(decode(0x4e80_0020).unwrap().form, "XL");
        // cmpw cr6,r3,r4 — a CR field, never a register.
        let cmpw = 0x7f83_2000u32;
        let d = decode(cmpw).unwrap();
        assert_eq!(d.field("BF").unwrap().kind, Kind::Cr);
        assert_eq!(d.reencode(), cmpw);
    }

    #[test]
    fn a_register_only_difference_is_classified_as_one() {
        let a = u32::from_be_bytes(encode_add(3, 4, 5));
        let b = u32::from_be_bytes(encode_add(3, 4, 6));
        let c = classify_pair(a, b);
        assert_eq!(c.class, "reg");
        assert_eq!(c.fields, vec!["RB"]);
    }

    #[test]
    fn an_immediate_only_difference_is_classified_as_one() {
        let a = u32::from_be_bytes(encode_addi(3, 0, 1));
        let b = u32::from_be_bytes(encode_addi(3, 0, 2));
        assert_eq!(classify_pair(a, b).class, "imm");
    }

    #[test]
    fn a_displacement_only_difference_is_classified_as_one() {
        let a = u32::from_be_bytes(encode_lwz(3, 1, 80));
        let b = u32::from_be_bytes(encode_lwz(3, 1, 84));
        assert_eq!(classify_pair(a, b).class, "disp");
    }

    #[test]
    fn a_branch_target_only_difference_is_classified_as_one() {
        let a = u32::from_be_bytes(encode_b_intra(-16).unwrap());
        let b = u32::from_be_bytes(encode_b_intra(-32).unwrap());
        assert_eq!(classify_pair(a, b).class, "branch-target");
    }

    #[test]
    fn two_different_instructions_are_an_opcode_difference() {
        let a = u32::from_be_bytes(encode_add(3, 4, 5));
        let b = u32::from_be_bytes(encode_subf(3, 4, 5));
        assert_eq!(classify_pair(a, b).class, "opcode");
        // Different form entirely.
        let c = u32::from_be_bytes(encode_lwz(3, 1, 0));
        assert_eq!(classify_pair(a, c).class, "opcode");
    }

    /// **The regression that the dc3 census itself found.** `addi` and `lwz` are
    /// both D-form and share no field names; the classifier used to report the
    /// pair as `undecoded`, which is how 470 perfectly readable words came to be
    /// counted as unreadable. Both decode; they are a different instruction.
    #[test]
    fn two_d_form_instructions_with_different_field_names_are_an_opcode_difference() {
        let addi = 0x3880_0000u32; // li r4,0
        let lwz = 0x8144_0004u32; // lwz r10,4(r4)
        assert!(decode(addi).is_some() && decode(lwz).is_some());
        let c = classify_pair(addi, lwz);
        assert_eq!(c.class, "opcode");
        assert_eq!(c.fields, vec!["OPCD"]);
        // …and the other real pair from the same census: stw vs addi.
        assert_eq!(classify_pair(0x9181_fff8, 0x386b_ffec).class, "opcode");
    }

    /// Same primary, same form name, different field layout — `cmp` against
    /// `srawi`, both primary 31 and both form `X`. A different layout under one
    /// primary is a different extended opcode, never an unreadable word.
    #[test]
    fn one_primary_with_two_layouts_is_an_opcode_difference() {
        let cmp = 0x7f83_2000u32;
        let srawi = u32::from_be_bytes(encode_srawi(3, 4, 31));
        assert_eq!(decode(cmp).unwrap().form, decode(srawi).unwrap().form);
        let c = classify_pair(cmp, srawi);
        assert_eq!(c.class, "opcode");
    }

    #[test]
    fn an_unmodelled_form_is_undecoded_and_never_guessed() {
        // Primary 4 is VMX/VMX128 — deliberately not modelled.
        let vmx = 0x1000_0000u32 | (4 << 26);
        assert!(decode(vmx).is_none());
        assert_eq!(classify_pair(vmx, vmx ^ 1).class, "undecoded");
    }

    #[test]
    fn alignment_finds_an_insertion_rather_than_a_shift() {
        // port = [A, X, B, C], ref = [A, B, C] — one inserted word, not three
        // substitutions.
        let (a, b, c, x) = (1u32, 2, 3, 9);
        let (edits, capped) = align(&[a, x, b, c], &[a, b, c], LCS_CELL_CAP);
        assert!(!capped);
        let ins = edits.iter().filter(|e| matches!(e, Edit::Insert(_))).count();
        let sub = edits.iter().filter(|e| matches!(e, Edit::Sub(..))).count();
        assert_eq!((ins, sub), (1, 0), "{edits:?}");
    }

    #[test]
    fn adjacent_insert_and_delete_runs_pair_into_substitutions() {
        let (edits, _) = align(&[1, 9, 3], &[1, 2, 3], LCS_CELL_CAP);
        let sub: Vec<_> = edits
            .iter()
            .filter(|e| matches!(e, Edit::Sub(..)))
            .collect();
        assert_eq!(sub.len(), 1, "{edits:?}");
        assert!(matches!(sub[0], Edit::Sub(1, 1)));
    }

    #[test]
    fn the_accounting_identity_holds_on_every_edit_shape() {
        let cases: [(&[u32], &[u32]); 5] = [
            (&[1, 2, 3], &[1, 9, 3]),
            (&[1, 2, 3], &[1, 3]),
            (&[1, 3], &[1, 2, 3]),
            (&[3, 2, 1], &[1, 2, 3]),
            (&[], &[1, 2]),
        ];
        for (p, r) in cases {
            let pb: Vec<u8> = p.iter().flat_map(|w| w.to_be_bytes()).collect();
            let rb: Vec<u8> = r.iter().flat_map(|w| w.to_be_bytes()).collect();
            let s = signature("t.cpp", "?f", "seq", &pb, &rb, &[]);
            assert!(s.accounting_ok, "identity broke on {p:?} vs {r:?}: {s:?}");
        }
    }

    #[test]
    fn a_pure_reordering_reads_as_same_multiset() {
        let p: Vec<u8> = [1u32, 2, 3].iter().flat_map(|w| w.to_be_bytes()).collect();
        let r: Vec<u8> = [3u32, 2, 1].iter().flat_map(|w| w.to_be_bytes()).collect();
        let s = signature("t.cpp", "?f", "seq", &p, &r, &[]);
        assert!(s.same_multiset);
        assert!(s.csig().ends_with("|reorder"));
    }

    #[test]
    fn a_relocated_word_is_marked_as_one() {
        let bl_a = 0x4800_0001u32; // bl +0
        let bl_b = 0x4800_0005u32; // bl +4, same opcode different target
        let p: Vec<u8> = [0x3860_0000u32, bl_a].iter().flat_map(|w| w.to_be_bytes()).collect();
        let r: Vec<u8> = [0x3860_0000u32, bl_b].iter().flat_map(|w| w.to_be_bytes()).collect();
        let s = signature("t.cpp", "?f", "tail", &p, &r, &[(4, 6)]);
        assert_eq!(s.sub, 1);
        assert_eq!(s.sub_at_reloc, 1);
        assert_eq!(s.classes.get("branch-target").copied(), Some(1));
    }

    #[test]
    fn the_json_row_is_parseable_and_carries_the_keys() {
        let p: Vec<u8> = [1u32, 2].iter().flat_map(|w| w.to_be_bytes()).collect();
        let r: Vec<u8> = [1u32, 3].iter().flat_map(|w| w.to_be_bytes()).collect();
        let s = signature("a/b.cpp", "?f@@YAXXZ", "seq", &p, &r, &[]);
        let j = s.to_json();
        assert!(j.starts_with('{') && j.ends_with('}'));
        assert!(j.contains("\"tu\":\"a/b.cpp\""));
        assert!(j.contains("\"csig\":"));
        assert!(j.contains("\"accounting_ok\":true"));
        assert!(s.keys().iter().any(|(k, _)| k == "fndiff-rows"));
    }

    #[test]
    fn the_cap_degrades_positionally_and_says_so() {
        let p: Vec<u32> = (0..40u32).collect();
        let r: Vec<u32> = (100..140u32).collect();
        let (edits, capped) = align(&p, &r, 100);
        assert!(capped);
        assert_eq!(edits.len(), 40);
        assert!(edits.iter().all(|e| matches!(e, Edit::Sub(..))));
    }
}
