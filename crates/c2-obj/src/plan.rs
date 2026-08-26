//! **The OBJECT PLAN** — everything about a COFF `.obj` that is independent of
//! the instruction bytes, read off one image in one walk.
//!
//! # Why this type exists
//!
//! `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §1.2 item 4: the remaining
//! distance to `match` is a **conjunction**. At the tree this landed on,
//! `a-and-b-and-c` is 27 and `match` is 26 — the port has converted 26 of the
//! 27 TUs that satisfy every factor, and the other 844 fail several factors at
//! once. So every single-stage improvement scores **zero** and there is no
//! continuous curve to steer by: a perfect reader converts 2 TUs (#3191), a
//! perfect section emitter converts 0 (#3210), lifting the whole `.gl` walk
//! measured `match +0` (#3093).
//!
//! The object plan is the part of that conjunction that can be graded against
//! real c2 on **all** graded TUs today, including the 844 that produce no IR at
//! all: the emit set, the section table, COMDAT selection and associativity,
//! weak externals, the undefined-external set, and the relocation inventory.
//! None of it needs a body.
//!
//! # THIS IS AN INSTRUMENT AND NEVER A GATE
//!
//! Nothing in this module may appear in an accept path, a refusal predicate, or
//! `scripts/gate.sh`. **`plan-exact` is NECESSARY but NOT SUFFICIENT for
//! `match`** — a TU can be plan-exact and mismatch on every byte of every body.
//! The byte judge (`ObjImage::diff` against real `c2.dll` under wibo) is
//! unchanged, untouched and remains the sole judge. This paragraph is in the
//! module doc and not only in the rung because the hazard is accretion: the
//! figure is convenient and green, and somebody will want to gate on it.
//!
//! # The four invariants, each stated as what it holds and how it is checked
//!
//! 1. **Body-independence.** No field is a function of instruction bytes.
//!    Enforced by construction, not by intent: [`PlanSection::byte_len`] is
//!    `None` for any `.text*` section, the COMDAT aux `CheckSum` is **not a
//!    field at all** (it is a checksum *of* section data), [`PlanSymbol`] has
//!    **no `Value`** (a `$M` label's Value is a body offset), and
//!    [`PlanRelocSet`] entries carry **no `VirtualAddress`** (under `/Gy` a
//!    code relocation's VA *is* its offset in the body). Checked by
//!    [`tests::mutating_text_bytes_does_not_move_the_plan`], which rewrites
//!    every `.text` raw byte of a synthetic image and asserts the plan is
//!    unchanged.
//! 2. **Index-freedom.** Every cross-reference is by **name**, never by section
//!    or symbol index — see [`SectionKey`]. An index shifts when an unrelated
//!    section is added, so two plans differing only in an unmodelled section
//!    would read *"every relocation differs"*: a component at 0 % for a
//!    structural reason nobody can see. Checked by
//!    [`tests::inserting_an_unrelated_section_moves_only_that_entry`].
//! 3. **Fail-closed, whole-object.** [`ObjPlan::observe`] returns `None` the
//!    moment anything does not decode — never a short plan. Same contract as
//!    every other walk in this crate. A short plan reads as *"this obj has less
//!    structure than it does"*, which is absence-as-success, and this project's
//!    most-recorded failure.
//! 4. **Determinism.** `observe(b) == observe(b)`, depending on nothing outside
//!    the byte slice.
//!
//! # What the plan CANNOT see — stated, because #3237
//!
//! An instrument that returns 0 because it did not look is indistinguishable
//! from one that returns 0 because there was nothing to find. So, explicitly:
//!
//! * Every instruction byte, and therefore `.text` section **sizes**, the
//!   COMDAT aux `CheckSum`, symbol `Value` for in-body labels, relocation
//!   `VirtualAddress` inside code, the `.pdata` prolog/epilog fields, and any
//!   `.debug$S` subsection keyed to a code offset.
//! * A `capture-fail` TU. There is no reference obj, so there is no row — never
//!   a false row.
//! * Anything about **correctness**. See the second section of this doc.
//!
//! One thing this module deliberately **retains** and flags rather than
//! dropping: [`PlanSection::byte_len`] for **non**-`.text` sections. `.pdata`'s
//! length is `8 × nfunctions` and `.debug$S`'s length is a function of the
//! `-Fo` path and the compiland record — neither is a function of the
//! *instruction bytes*, so both are inside invariant 1 — but they are
//! body-*correlated* in the weaker sense that a different emit set changes
//! them. They are carried in `sections` and graded in their own component so a
//! consumer can exclude them; they are never folded into the emit-set curve.

use std::collections::BTreeMap;

use crate::reloc::RelocTarget;
use crate::{
    section_name_at, ObjImage, COFF_HEADER_LEN, IMAGE_SCN_LNK_COMDAT, IMAGE_SYM_CLASS_STATIC,
    IMAGE_SYM_CLASS_WEAK_EXTERNAL, SECTION_HEADER_LEN, SYMBOL_LEN, TEXT_SECTION_PREFIX,
};

/// `IMAGE_SYM_CLASS_EXTERNAL`.
/// PROV[S] PE/COFF §5.4.4 storage classes — `IMAGE_SYM_CLASS_EXTERNAL = 2`.
const IMAGE_SYM_CLASS_EXTERNAL: u8 = 2;
/// `IMAGE_SYM_UNDEFINED` — section number 0.
/// PROV[S] PE/COFF §5.4.2 — section number 0 is `IMAGE_SYM_UNDEFINED`.
const IMAGE_SYM_UNDEFINED: i16 = 0;
/// The alignment nibble of `Characteristics`, `IMAGE_SCN_ALIGN_*`.
/// PROV[S] PE/COFF §4.1 — the `IMAGE_SCN_ALIGN_*` values occupy bits 20..23 of `Characteristics`. The MASK is the spec's; WHICH alignment c2 picks per section is not, and is not adopted here.
const IMAGE_SCN_ALIGN_MASK: u32 = 0x00F0_0000;

/// **An index-free handle on one section.**
///
/// A COFF section is identified on disk by its 1-based index, and an index is
/// exactly the wrong key for a structural manifest: insert one unmodelled
/// section and every later index shifts, so a diff of two plans reads *"all
/// relocations differ"* when one section was added. That is invariant 2, and it
/// is the same section-led-not-symbol-led argument
/// [`ObjImage::text_comdat_functions`] already makes, one level up.
///
/// **A name alone is not enough, and under `/Gy` it is badly not enough**: an
/// obj carries one COMDAT `.text` section *per emitted function* and every one
/// of them is named `.text`. So the key is a triple:
///
/// * `name` — the decoded section name (`/NNN` string-table indirections
///   resolved by the crate's one decoder, [`section_name_at`]).
/// * `leader` — the section's **COMDAT leader symbol**, when it has one. This
///   is what actually disambiguates `/Gy` code sections, and it is unique: a
///   leader is a function's mangled name.
/// * `ordinal` — 0-based position among the sections *sharing this name*. It
///   disambiguates the non-COMDAT repeats (`.XBLD$W` occurs twice in this
///   workload's objs) and it does **not** move when a differently-named section
///   is inserted, which is the property invariant 2 asks for.
///
/// **PLAN DEFECT recorded here rather than in prose only.** The lane plan
/// specified COMDAT associativity as `assoc: Option<String> /* by NAME */`.
/// Under `/Gy` that is ambiguous to the point of being useless — a `.pdata`
/// COMDAT associates with *one particular* `.text` section and they are all
/// called `.text`, so an associativity graded by section name would read
/// "equal" for every possible mis-association. Associativity is resolved to the
/// associated section's [`SectionKey`] instead, whose `leader` is the function's
/// mangled name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectionKey {
    pub name: String,
    pub leader: Option<String>,
    pub ordinal: usize,
}

impl SectionKey {
    /// A one-line rendering for a report or a TSV cell. Never parsed back.
    pub fn label(&self) -> String {
        match &self.leader {
            Some(l) => format!("{}({l})", self.name),
            None if self.ordinal == 0 => self.name.clone(),
            None => format!("{}#{}", self.name, self.ordinal),
        }
    }
}

/// A COMDAT section's selection record — the aux of its section-definition
/// symbol, minus the two fields that are functions of the section's data.
///
/// `CheckSum` is **absent by construction** (invariant 1): it is a checksum of
/// the section's raw bytes, so carrying it would make the plan body-dependent
/// in the one place it must not be. `Length`, `NumberOfRelocations` and
/// `NumberOfLinenumbers` are likewise omitted — the first is
/// [`PlanSection::byte_len`]'s business (and `None` for code), and the other
/// two are re-derivable from [`ObjPlan::relocs`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanComdat {
    /// `IMAGE_COMDAT_SELECT_*`. 1 = NODUPLICATES, 2 = ANY, 3 = SAME_SIZE,
    /// 4 = EXACT_MATCH, 5 = ASSOCIATIVE, 6 = LARGEST.
    pub selection: u8,
    /// For `selection == 5` (ASSOCIATIVE), the section it associates with, by
    /// [`SectionKey`] and never by index. `None` for every other selection.
    pub assoc: Option<SectionKey>,
}

/// One section of the plan, in section-table order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanSection {
    pub key: SectionKey,
    /// The raw `Characteristics` word, exactly as it sits on disk. Kept whole
    /// rather than split into flags: a re-derived flag set is a second reader
    /// that can drift from this one, and `docs/GAPS.md` §6's one-fact-one-locator
    /// rule is what this crate's section-name decoder exists for.
    pub characteristics: u32,
    /// `SizeOfRawData`, or **`None` for any `.text*` section** — that number is
    /// the sum of the instruction bytes and is precisely what the plan must not
    /// be able to see (invariant 1).
    pub byte_len: Option<u32>,
    pub comdat: Option<PlanComdat>,
}

impl PlanSection {
    /// The `IMAGE_SCN_ALIGN_*` nibble decoded to a power of two, or `None` when
    /// the section declares no alignment (`0`, which the linker reads as 16).
    ///
    /// A **method and not a stored field**, deliberately, against the lane
    /// plan's `align: u8`: the nibble lives inside [`Self::characteristics`],
    /// and storing both would be one fact in two places — the shape
    /// `docs/GAPS.md` §6 forbids and the reason four relocation-type rows in
    /// `gt_dump.py` were wrong for the file's whole existence.
    pub fn align_pow2(&self) -> Option<u32> {
        let n = (self.characteristics & IMAGE_SCN_ALIGN_MASK) >> 20;
        if n == 0 {
            None
        } else {
            Some(1u32 << (n - 1))
        }
    }
}

/// One symbol of the plan, in symbol-table order. Auxiliary slots are **not**
/// rows (an aux record is not a symbol) but their count is carried, so a plan
/// that lost one is not silently the same as a plan that never had it.
///
/// **No `Value` field.** A `$M<n>` compiler label's `Value` is its offset inside
/// a body; carrying it would make the plan body-dependent (invariant 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanSymbol {
    pub name: String,
    pub storage_class: u8,
    pub section: SymSection,
    pub n_aux: u8,
}

/// What a symbol's `SectionNumber` names, index-free.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymSection {
    /// `IMAGE_SYM_UNDEFINED` (0) — an external this obj references and does not
    /// define, or (with a nonzero `Value`) a common block.
    Undefined,
    /// `IMAGE_SYM_ABSOLUTE` (-1).
    Absolute,
    /// `IMAGE_SYM_DEBUG` (-2).
    Debug,
    /// A real section, by [`SectionKey`].
    In(SectionKey),
}

/// A weak external and the default symbol it aliases.
///
/// c2 realises a `.gl` tag-`0x10` ALIAS as a COFF weak external rather than as
/// a substitution — see [`ObjImage::weak_externals`], which this reproduces and
/// is graded against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanWeak {
    pub weak: String,
    pub default: String,
    pub characteristics: u32,
}

/// One section's relocation inventory: `(packed Type word, target)` in **disk
/// order**, with the `VirtualAddress` dropped.
///
/// The VA is dropped because under `/Gy` a code relocation's `VirtualAddress`
/// *is* its byte offset inside the function body, which is invariant 1's exact
/// prohibition. The **order** is kept: it is a real observable of c2's output
/// and it is not a function of the instruction bytes (rewriting them moves no
/// relocation record), though it is body-*correlated* in the weaker sense that
/// a different instruction selection would emit its relocations in a different
/// order. That is a fact about what the component measures, not a defect, and
/// it is stated so a reader does not have to infer it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRelocSet {
    pub section: SectionKey,
    pub entries: Vec<(u16, RelocTarget)>,
}

/// **The body-byte-independent plan of one object file.**
///
/// See the module doc for the four invariants and for the list of what this
/// cannot see.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjPlan {
    /// Sections in section-table order.
    pub sections: Vec<PlanSection>,
    /// The `.text*` COMDAT leaders, in section order — the **emit set**. Equal
    /// to [`ObjImage::text_comdat_functions`] by the agreement control.
    pub emit_set: Vec<String>,
    /// Symbols in symbol-table order, aux slots elided (their count is on the
    /// preceding [`PlanSymbol`]).
    pub symbols: Vec<PlanSymbol>,
    pub weak: Vec<PlanWeak>,
    /// Undefined externals, in symbol-table order.
    pub undef: Vec<String>,
    /// Relocation inventory per section, in section order.
    pub relocs: Vec<PlanRelocSet>,
    /// The raw contents of `.drectve`, or empty when the obj has none. A
    /// directive string (`-defaultlib:…`), not code.
    pub drectve: Vec<u8>,
}

impl ObjPlan {
    /// The ordered section-name sequence.
    pub fn section_names(&self) -> Vec<&str> {
        self.sections.iter().map(|s| s.key.name.as_str()).collect()
    }

    /// The section-name **multiset**, as a sorted count map. Order-free, so the
    /// nested ladder `names ⊇ order ⊇ attrs` has a bottom rung that is a
    /// strictly weaker claim than the sequence.
    pub fn section_name_multiset(&self) -> BTreeMap<&str, usize> {
        let mut m: BTreeMap<&str, usize> = BTreeMap::new();
        for s in &self.sections {
            *m.entry(s.key.name.as_str()).or_insert(0) += 1;
        }
        m
    }

    /// The ordered `(name, characteristics, comdat selection)` sequence — the
    /// top rung of the section ladder.
    pub fn section_attrs(&self) -> Vec<(&str, u32, Option<u8>)> {
        self.sections
            .iter()
            .map(|s| {
                (
                    s.key.name.as_str(),
                    s.characteristics,
                    s.comdat.as_ref().map(|c| c.selection),
                )
            })
            .collect()
    }
}

/// One section's decoded header, plus the two facts that come from the symbol
/// table. Internal to the walk.
struct RawSection {
    name: String,
    characteristics: u32,
    size: u32,
    data_ptr: u32,
    leader: Option<String>,
    /// `(selection, associated 1-based section number)` from the
    /// section-definition symbol's aux record, when the section is a COMDAT.
    comdat_aux: Option<(u8, u16)>,
}

impl ObjImage {
    /// **Read this image's [`ObjPlan`]** — one walk, fail-closed on the whole
    /// object.
    ///
    /// `None` whenever anything does not decode: a short image, a symbol table
    /// or string table off the end, a section header or raw-data range past
    /// EOF, an aux count that walks past the symbol table, a relocation whose
    /// `SymbolTableIndex` lands on an aux slot, a COMDAT `.text` section with
    /// no leader, or an associativity record naming a section that is not
    /// there. There is no partial answer — see invariant 3.
    pub fn observe(&self) -> Option<ObjPlan> {
        let b = &self.0;
        let (nsec, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];

        // ---- pass 1: section headers ------------------------------------
        let mut raw: Vec<RawSection> = Vec::with_capacity(nsec);
        for i in 0..nsec {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            let name = section_name_at(b, o, strtab)?;
            let size = u32::from_le_bytes([b[o + 16], b[o + 17], b[o + 18], b[o + 19]]);
            let data_ptr = u32::from_le_bytes([b[o + 20], b[o + 21], b[o + 22], b[o + 23]]);
            let characteristics =
                u32::from_le_bytes([b[o + 36], b[o + 37], b[o + 38], b[o + 39]]);
            // A section whose raw data runs off the end is a decode failure,
            // not a short answer — the same rule
            // `text_comdat_functions_with_bytes` applies.
            if data_ptr != 0 {
                let end = (data_ptr as usize).checked_add(size as usize)?;
                if end > b.len() {
                    return None;
                }
            }
            raw.push(RawSection {
                name,
                characteristics,
                size,
                data_ptr,
                leader: None,
                comdat_aux: None,
            });
        }

        // ---- pass 2: the symbol table -----------------------------------
        //
        // One walk producing four things at once — the leaders, the COMDAT aux
        // records, the plan's symbol rows and the undefined-external list —
        // because they are four readings of one table and a second walk is a
        // second chance to disagree with the first.
        let names = self.symbol_names_by_slot()?;
        let mut sym_rows: Vec<(String, u8, i16, u8)> = Vec::new();
        let mut weak: Vec<PlanWeak> = Vec::new();
        let mut undef: Vec<String> = Vec::new();
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            let secnum = i16::from_le_bytes([b[o + 12], b[o + 13]]);
            let sclass = b[o + 16];
            let name = names.get(i)?.clone()?;

            if secnum >= 1 && (secnum as usize) <= nsec {
                let s = secnum as usize - 1;
                let is_section_definition = sclass == IMAGE_SYM_CLASS_STATIC && naux == 1;
                if is_section_definition {
                    if raw[s].characteristics & IMAGE_SCN_LNK_COMDAT != 0 {
                        // The section-definition aux: Length(0..4),
                        // NumberOfRelocations(4..6), NumberOfLinenumbers(6..8),
                        // CheckSum(8..12), Number(12..14), Selection(14).
                        // CheckSum is deliberately NOT read (invariant 1).
                        let a = o + SYMBOL_LEN;
                        if a + SYMBOL_LEN > sym_end {
                            return None;
                        }
                        let assoc = u16::from_le_bytes([b[a + 12], b[a + 13]]);
                        let selection = b[a + 14];
                        // Two section-definition symbols for one section is a
                        // shape this reader has no answer for; refuse rather
                        // than take either.
                        if raw[s].comdat_aux.is_some() {
                            return None;
                        }
                        raw[s].comdat_aux = Some((selection, assoc));
                    }
                } else if raw[s].leader.is_none() {
                    raw[s].leader = Some(name.clone());
                }
            } else if secnum == IMAGE_SYM_UNDEFINED && sclass == IMAGE_SYM_CLASS_EXTERNAL {
                undef.push(name.clone());
            }

            if sclass == IMAGE_SYM_CLASS_WEAK_EXTERNAL {
                if naux == 0 || o + SYMBOL_LEN + 12 > sym_end {
                    return None;
                }
                let a = o + SYMBOL_LEN;
                let tag = u32::from_le_bytes([b[a], b[a + 1], b[a + 2], b[a + 3]]) as usize;
                let ch = u32::from_le_bytes([b[a + 4], b[a + 5], b[a + 6], b[a + 7]]);
                weak.push(PlanWeak {
                    weak: name.clone(),
                    default: names.get(tag)?.clone()?,
                    characteristics: ch,
                });
            }

            sym_rows.push((name, sclass, secnum, b[o + 17]));
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }

        // ---- the section keys -------------------------------------------
        //
        // Built once, after the leaders are known, and every cross-reference
        // below is one of these (invariant 2).
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        let mut keys: Vec<SectionKey> = Vec::with_capacity(nsec);
        for s in &raw {
            let n = seen.entry(s.name.as_str()).or_insert(0);
            keys.push(SectionKey {
                name: s.name.clone(),
                leader: s.leader.clone(),
                ordinal: *n,
            });
            *n += 1;
        }

        // ---- sections ----------------------------------------------------
        let mut sections = Vec::with_capacity(nsec);
        for (i, s) in raw.iter().enumerate() {
            let comdat = match s.comdat_aux {
                None => {
                    // A section flagged COMDAT with no section-definition aux is
                    // malformed, not a non-COMDAT section.
                    if s.characteristics & IMAGE_SCN_LNK_COMDAT != 0 {
                        return None;
                    }
                    None
                }
                Some((selection, assoc)) => {
                    // IMAGE_COMDAT_SELECT_ASSOCIATIVE == 5 is the one selection
                    // whose `Number` field names another section. For every
                    // other selection the field is unused, and reading it would
                    // manufacture an association out of padding.
                    let assoc = if selection == 5 {
                        let a = assoc as usize;
                        if a == 0 || a > nsec {
                            return None; // an association to nothing
                        }
                        Some(keys[a - 1].clone())
                    } else {
                        None
                    };
                    Some(PlanComdat { selection, assoc })
                }
            };
            sections.push(PlanSection {
                key: keys[i].clone(),
                characteristics: s.characteristics,
                byte_len: if s.name.starts_with(TEXT_SECTION_PREFIX) {
                    None
                } else {
                    Some(s.size)
                },
                comdat,
            });
        }

        // ---- the emit set -------------------------------------------------
        //
        // The `.text*` COMDAT leaders in section order. The **same** rule
        // `text_comdat_entries` applies, including its refusal: a COMDAT `.text`
        // section that produced no leader means the symbol walk went wrong, and
        // a short emit set is a denominator that silently inflates every ratio
        // taken against it.
        let mut emit_set = Vec::new();
        for s in &raw {
            if s.name.starts_with(TEXT_SECTION_PREFIX)
                && s.characteristics & IMAGE_SCN_LNK_COMDAT != 0
            {
                emit_set.push(s.leader.clone()?);
            }
        }

        // ---- symbols ------------------------------------------------------
        let symbols = sym_rows
            .into_iter()
            .map(|(name, storage_class, secnum, n_aux)| {
                let section = match secnum {
                    IMAGE_SYM_UNDEFINED => SymSection::Undefined,
                    -1 => SymSection::Absolute,
                    -2 => SymSection::Debug,
                    n if n >= 1 && (n as usize) <= nsec => {
                        SymSection::In(keys[n as usize - 1].clone())
                    }
                    // A section number that is neither a sentinel nor a section
                    // is a decode failure.
                    _ => return None,
                };
                Some(PlanSymbol {
                    name,
                    storage_class,
                    section,
                    n_aux,
                })
            })
            .collect::<Option<Vec<_>>>()?;

        // ---- relocations ---------------------------------------------------
        let all = self.relocations()?;
        let targets = self.symbol_targets()?;
        let mut relocs: Vec<PlanRelocSet> = keys
            .iter()
            .map(|k| PlanRelocSet {
                section: k.clone(),
                entries: Vec::new(),
            })
            .collect();
        for r in &all {
            if r.section >= nsec {
                return None;
            }
            // `PAIR` carries a DISPLACEMENT in the index field (rev 6.0), so it
            // must never be resolved as a symbol. `Reloc::sym_is_an_index` is
            // the single place that decides it — one fact, one locator.
            let target = if r.sym_is_an_index() {
                targets.get(r.sym as usize)?.clone()?
            } else {
                RelocTarget::PairDisplacement(r.sym)
            };
            relocs[r.section].entries.push((r.ty, target));
        }

        // ---- .drectve -------------------------------------------------------
        let mut drectve = Vec::new();
        for s in &raw {
            if s.name == ".drectve" && s.data_ptr != 0 {
                let at = s.data_ptr as usize;
                drectve = b.get(at..at + s.size as usize)?.to_vec();
                break;
            }
        }

        Some(ObjPlan {
            sections,
            emit_set,
            symbols,
            weak,
            undef,
            relocs,
            drectve,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny COFF builder. Enough to exercise the walk's decode and its
    /// refusals with **no toolchain**; the real objs are graded by the
    /// agreement control in `c2-harness`.
    struct Builder {
        sections: Vec<(String, u32, Vec<u8>, Vec<(u32, u32, u16)>)>,
        symbols: Vec<(String, i16, u8, Vec<[u8; 18]>)>,
    }

    impl Builder {
        fn new() -> Self {
            Builder {
                sections: Vec::new(),
                symbols: Vec::new(),
            }
        }
        fn section(mut self, name: &str, chars: u32, data: Vec<u8>) -> Self {
            self.sections.push((name.to_string(), chars, data, Vec::new()));
            self
        }
        fn reloc(mut self, sec: usize, va: u32, sym: u32, ty: u16) -> Self {
            self.sections[sec].3.push((va, sym, ty));
            self
        }
        fn sym(mut self, name: &str, secnum: i16, class: u8) -> Self {
            self.symbols.push((name.to_string(), secnum, class, Vec::new()));
            self
        }
        fn aux(mut self, a: [u8; 18]) -> Self {
            self.symbols.last_mut().unwrap().3.push(a);
            self
        }
        /// A section-definition aux with the given selection and association.
        fn secdef_aux(selection: u8, assoc: u16) -> [u8; 18] {
            let mut a = [0u8; 18];
            a[12..14].copy_from_slice(&assoc.to_le_bytes());
            a[14] = selection;
            a
        }
        fn build(self) -> ObjImage {
            let nsec = self.sections.len();
            let nsym: usize = self.symbols.iter().map(|s| 1 + s.3.len()).sum();
            let mut strtab: Vec<u8> = vec![0, 0, 0, 0];
            let mut long = |s: &str, out: &mut Vec<u8>| {
                let at = out.len();
                out.extend_from_slice(s.as_bytes());
                out.push(0);
                at
            };
            // Layout: header | section headers | data+relocs | symtab | strtab
            let mut cursor = COFF_HEADER_LEN + nsec * SECTION_HEADER_LEN;
            let mut headers: Vec<[u8; SECTION_HEADER_LEN]> = Vec::new();
            let mut blob: Vec<u8> = Vec::new();
            for (name, chars, data, rels) in &self.sections {
                let mut h = [0u8; SECTION_HEADER_LEN];
                let nb = name.as_bytes();
                assert!(nb.len() <= 8, "test builder: short names only");
                h[..nb.len()].copy_from_slice(nb);
                h[16..20].copy_from_slice(&(data.len() as u32).to_le_bytes());
                if data.is_empty() {
                    h[20..24].copy_from_slice(&0u32.to_le_bytes());
                } else {
                    h[20..24].copy_from_slice(&(cursor as u32).to_le_bytes());
                    blob.extend_from_slice(data);
                    cursor += data.len();
                }
                if !rels.is_empty() {
                    h[24..28].copy_from_slice(&(cursor as u32).to_le_bytes());
                    h[32..34].copy_from_slice(&(rels.len() as u16).to_le_bytes());
                    for (va, sym, ty) in rels {
                        blob.extend_from_slice(&va.to_le_bytes());
                        blob.extend_from_slice(&sym.to_le_bytes());
                        blob.extend_from_slice(&ty.to_le_bytes());
                        cursor += 10;
                    }
                }
                h[36..40].copy_from_slice(&chars.to_le_bytes());
                headers.push(h);
            }
            let psym = cursor;
            let mut symtab: Vec<u8> = Vec::new();
            for (name, secnum, class, auxes) in &self.symbols {
                let mut r = [0u8; SYMBOL_LEN];
                if name.len() <= 8 {
                    r[..name.len()].copy_from_slice(name.as_bytes());
                } else {
                    // `str_at` indexes from the START of the string table,
                    // which INCLUDES its own 4-byte size word — so the offset
                    // is `at` and not `at + 4`. Writing `at + 4` produced
                    // `"YAXXZ"` for `"?f@@YAXXZ"`: a plausible mangled-looking
                    // name, four bytes in. Recorded rather than tidied — the
                    // failure was in the TEST BUILDER and `observe` decoded the
                    // wrong offset faithfully, which is exactly how a bad
                    // synthetic fixture certifies a good reader.
                    let at = long(name, &mut strtab);
                    r[4..8].copy_from_slice(&(at as u32).to_le_bytes());
                }
                r[12..14].copy_from_slice(&secnum.to_le_bytes());
                r[16] = *class;
                r[17] = auxes.len() as u8;
                symtab.extend_from_slice(&r);
                for a in auxes {
                    symtab.extend_from_slice(a);
                }
            }
            let mut out = vec![0u8; COFF_HEADER_LEN];
            out[0..2].copy_from_slice(&0x01F2u16.to_le_bytes()); // POWERPCFP
            out[2..4].copy_from_slice(&(nsec as u16).to_le_bytes());
            out[8..12].copy_from_slice(&(psym as u32).to_le_bytes());
            out[12..16].copy_from_slice(&(nsym as u32).to_le_bytes());
            for h in &headers {
                out.extend_from_slice(h);
            }
            out.extend_from_slice(&blob);
            out.extend_from_slice(&symtab);
            let n = strtab.len() as u32;
            strtab[0..4].copy_from_slice(&n.to_le_bytes());
            out.extend_from_slice(&strtab);
            ObjImage::new(out)
        }
    }

    const COMDAT_EXEC: u32 = IMAGE_SCN_LNK_COMDAT | 0x2000_0020 | 0x0050_0000;
    const DATA: u32 = 0x4000_0040 | 0x0030_0000;

    /// Two `/Gy` code COMDATs, a `.pdata` associated with the second, a
    /// `.drectve`, one weak external and one undefined external.
    fn specimen() -> Builder {
        Builder::new()
            .section(".text", COMDAT_EXEC, vec![0x38, 0x60, 0x00, 0x00])
            .section(".text", COMDAT_EXEC, vec![0x4E, 0x80, 0x00, 0x20])
            .section(".pdata", IMAGE_SCN_LNK_COMDAT | DATA, vec![0u8; 8])
            .section(".drectve", 0x0000_0A00, b"-defaultlib:LIBCMT ".to_vec())
            .reloc(2, 0, 0, crate::reloc::IMAGE_REL_PPC_ADDR32NB)
            .sym(".text", 1, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(1, 0))
            .sym("?f@@YAXXZ", 1, IMAGE_SYM_CLASS_EXTERNAL)
            .sym(".text", 2, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(1, 0))
            .sym("?g@@YAXXZ", 2, IMAGE_SYM_CLASS_EXTERNAL)
            .sym(".pdata", 3, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(5, 2))
            .sym("?ext@@YAXXZ", 0, IMAGE_SYM_CLASS_EXTERNAL)
            .sym("?G@@YAXXZ", 0, IMAGE_SYM_CLASS_EXTERNAL)
            .sym("?E@@YAXXZ", 0, IMAGE_SYM_CLASS_WEAK_EXTERNAL)
            .aux({
                let mut a = [0u8; 18];
                // TagIndex -> `?G@@YAXXZ`. Slot NINE, counting the three
                // auxiliary records that occupy slots of their own: an aux is
                // not a symbol but it IS an index, which is the whole reason
                // `symbol_names_by_slot` returns a sparse vector.
                a[0..4].copy_from_slice(&9u32.to_le_bytes());
                a[4..8].copy_from_slice(&2u32.to_le_bytes());
                a
            })
    }

    #[test]
    fn the_specimen_decodes_whole() {
        let p = specimen().build().observe().expect("specimen must decode");
        assert_eq!(p.emit_set, vec!["?f@@YAXXZ", "?g@@YAXXZ"]);
        assert_eq!(
            p.section_names(),
            vec![".text", ".text", ".pdata", ".drectve"]
        );
        assert_eq!(p.drectve, b"-defaultlib:LIBCMT ");
        assert_eq!(p.undef, vec!["?ext@@YAXXZ", "?G@@YAXXZ"]);
        assert_eq!(p.weak.len(), 1);
        assert_eq!(p.weak[0].weak, "?E@@YAXXZ");
        assert_eq!(p.weak[0].default, "?G@@YAXXZ");
        assert_eq!(p.weak[0].characteristics, 2);
        // A `.text*` section carries NO byte_len — invariant 1.
        assert_eq!(p.sections[0].byte_len, None);
        assert_eq!(p.sections[1].byte_len, None);
        assert_eq!(p.sections[2].byte_len, Some(8));
    }

    /// **Invariant 2**, in its sharpest form: the associativity of `.pdata`
    /// names the SECOND `.text`, and both `.text` sections are called `.text`.
    /// A plan keyed by section NAME could not tell a correct association from
    /// any wrong one.
    #[test]
    fn associativity_resolves_to_the_leader_not_to_the_ambiguous_name() {
        let p = specimen().build().observe().unwrap();
        let assoc = p.sections[2]
            .comdat
            .as_ref()
            .unwrap()
            .assoc
            .as_ref()
            .unwrap();
        assert_eq!(assoc.name, ".text");
        assert_eq!(assoc.leader.as_deref(), Some("?g@@YAXXZ"));
        assert_eq!(assoc.ordinal, 1);
        // …and the two same-named sections are distinguishable from each other.
        assert_ne!(p.sections[0].key, p.sections[1].key);
    }

    /// **Invariant 1**, checked and not asserted: rewrite every `.text` raw
    /// byte and the plan must be identical.
    #[test]
    fn mutating_text_bytes_does_not_move_the_plan() {
        let base = specimen().build();
        let before = base.observe().unwrap();
        let mutated = specimen()
            .build()
            .as_bytes()
            .iter()
            .copied()
            .collect::<Vec<u8>>();
        let mut mutated = ObjImage::new(mutated);
        // Both `.text` bodies live at known offsets in the builder's layout;
        // find them by their content rather than by arithmetic.
        let needles: [&[u8]; 2] = [&[0x38, 0x60, 0x00, 0x00], &[0x4E, 0x80, 0x00, 0x20]];
        let mut hits = 0;
        for n in needles {
            let bytes = mutated.0.clone();
            if let Some(at) = bytes.windows(4).position(|w| w == n) {
                mutated.0[at..at + 4].copy_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
                hits += 1;
            }
        }
        assert_eq!(hits, 2, "the test must actually have mutated both bodies");
        assert_ne!(base.as_bytes(), mutated.as_bytes());
        assert_eq!(before, mutated.observe().unwrap());
    }

    /// **Invariant 2** again, the way the module doc states it: inserting an
    /// unrelated section must move exactly that section's entry and nothing
    /// else — not "every relocation differs".
    #[test]
    fn inserting_an_unrelated_section_moves_only_that_entry() {
        let before = specimen().build().observe().unwrap();
        // The same obj with a `.bss` appended, and the symbol table's section
        // numbers untouched (the new section is last, so nothing renumbers) —
        // the interesting half is that the RELOC SET is keyed by name and so
        // compares equal for every pre-existing section.
        let after = specimen()
            .section(".bss", 0xC000_0080u32, Vec::new())
            .build()
            .observe()
            .unwrap();
        assert_eq!(before.emit_set, after.emit_set);
        assert_eq!(before.weak, after.weak);
        assert_eq!(before.undef, after.undef);
        for (a, b) in before.relocs.iter().zip(&after.relocs) {
            assert_eq!(a, b);
        }
        assert_eq!(after.sections.len(), before.sections.len() + 1);
    }

    /// **Invariant 3** — every one of these is a whole-object refusal, never a
    /// short plan.
    #[test]
    fn fail_closed_on_every_malformed_shape() {
        // Short image.
        assert!(ObjImage::new(vec![0u8; 12]).observe().is_none());
        // Symbol table off the end.
        let mut b = specimen().build();
        b.0[8..12].copy_from_slice(&0xFFFF_0000u32.to_le_bytes());
        assert!(b.observe().is_none());
        // A COMDAT `.text` with no leader symbol: drop the leader's row by
        // marking it a section definition too.
        let img = Builder::new()
            .section(".text", COMDAT_EXEC, vec![0u8; 4])
            .sym(".text", 1, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(1, 0))
            .build();
        assert!(img.observe().is_none(), "a leaderless .text COMDAT must refuse");
        // An ASSOCIATIVE selection naming a section that is not there.
        let img = Builder::new()
            .section(".text", COMDAT_EXEC, vec![0u8; 4])
            .section(".pdata", IMAGE_SCN_LNK_COMDAT | DATA, vec![0u8; 8])
            .sym(".text", 1, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(1, 0))
            .sym("?f@@YAXXZ", 1, IMAGE_SYM_CLASS_EXTERNAL)
            .sym(".pdata", 2, IMAGE_SYM_CLASS_STATIC)
            .aux(Builder::secdef_aux(5, 99))
            .build();
        assert!(img.observe().is_none(), "an association to nothing must refuse");
    }

    /// A COMDAT section with no section-definition aux is malformed, not a
    /// plain section.
    #[test]
    fn a_comdat_flag_without_a_selection_record_refuses() {
        let img = Builder::new()
            .section(".text", COMDAT_EXEC, vec![0u8; 4])
            .sym("?f@@YAXXZ", 1, IMAGE_SYM_CLASS_EXTERNAL)
            .build();
        assert!(img.observe().is_none());
    }

    /// `observe` depends on nothing outside the byte slice — invariant 4.
    #[test]
    fn observe_is_deterministic() {
        let img = specimen().build();
        assert_eq!(img.observe(), img.observe());
    }

    /// The alignment nibble is decoded from `Characteristics` and is not a
    /// second stored copy of it.
    #[test]
    fn align_is_derived_from_characteristics() {
        let p = specimen().build().observe().unwrap();
        // 0x00500000 == IMAGE_SCN_ALIGN_16BYTES (nibble 5 -> 1 << 4).
        assert_eq!(p.sections[0].align_pow2(), Some(16));
        // 0x00300000 == IMAGE_SCN_ALIGN_4BYTES (nibble 3 -> 1 << 2).
        assert_eq!(p.sections[2].align_pow2(), Some(4));
        // `.drectve` declares nibble 0 — no alignment stated, which is NOT the
        // same fact as "1-byte aligned" and must not decode to `Some(1)`.
        assert_eq!(p.sections[3].align_pow2(), None);
    }

    /// The ladder's three rungs are genuinely nested: two plans can agree on
    /// the name multiset and disagree on the order.
    #[test]
    fn the_section_ladder_rungs_are_distinguishable() {
        let a = specimen().build().observe().unwrap();
        let mut b = a.clone();
        b.sections.swap(0, 2);
        assert_eq!(a.section_name_multiset(), b.section_name_multiset());
        assert_ne!(a.section_names(), b.section_names());
        assert_ne!(a.section_attrs(), b.section_attrs());
    }
}
