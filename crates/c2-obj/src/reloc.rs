//! **The PowerPC COFF relocation vocabulary, and the reader for a real obj's
//! records.**
//!
//! The port emits five of these types (`crates/c2-core/src/coff/reloc.rs` names
//! them). Everything else here exists to *read* — to say what a reference obj
//! actually contains when a compare fails, instead of printing a bare number.
//!
//! # Where the table comes from
//!
//! Microsoft's live PE/COFF spec publishes 17–18 of the `IMAGE_REL_PPC_*`
//! constants. The complete set of 23, plus the four modifier bits, is in
//! **`gimli-rs/object`, `src/pe.rs`** (Apache-2.0 OR MIT) — the most complete
//! public table there is; four of the values (`TOCREL16`, `TOCREL14`, `IFGLUE`,
//! `IMGLUE`) Microsoft has never documented in any revision, and `SECRELHI
//! 0x14` survives only in **rev 6.0** (Feb 1999) and as a dangling cross
//! reference under `PAIR` in the current text. The values below were
//! transcribed from that table and re-derived against rev 6.0 where rev 6.0 has
//! a row; the *code* is ours (`docs/PRIOR_ART.md` — `object` is a reference to
//! read, never a dependency, because this workspace is std-only).
//!
//! # `Type` is a packed word, not an enum
//!
//! This is the load-bearing fact and the reason this module exists at all. The
//! low byte is the relocation *type*; the high byte carries four independent
//! modifier bits (`NEG`, `BRTAKEN`, `BRNTAKEN`, `TOCDEFN`). A reader that
//! compares the whole 16-bit field against a type constant is correct only for
//! as long as no flag is ever set, and silently wrong the moment one is —
//! `REL24|BRTAKEN` is `0x0206`, which equals no constant in the table and would
//! decode as "unknown type" rather than as a taken-branch hint on a REL24.
//! [`Reloc::base`] masks; [`Reloc::flags`] keeps the rest, so nothing is
//! discarded silently.
//!
//! # Endianness
//!
//! Every field here is **little-endian**, in a big-endian object. That is not a
//! contradiction and it is the single most-repeated trap in this format:
//! `xbox360-binutils` states it in prose as *"PE/COFF with little-endian
//! headers, big-endian data"*, and `object`'s own `is_little_endian()` returns
//! `false` for `POWERPCBE` while every one of its header fields is a hardcoded
//! `U16<LE>`/`U32<LE>`. The first describes the section *payload*; the second
//! describes the *structures*. Read one without the other and you conclude the
//! opposite of the truth.

use crate::{ObjImage, COFF_HEADER_LEN, SECTION_HEADER_LEN};

/// One COFF relocation record on disk: `VirtualAddress` u32, `SymbolTableIndex`
/// u32, `Type` u16 — **packed, not padded to 12**.
pub const RELOC_LEN: usize = 10;

/// `IMAGE_REL_PPC_ABSOLUTE` — no relocation; a placeholder record.
pub const IMAGE_REL_PPC_ABSOLUTE: u16 = 0x0000;
/// `IMAGE_REL_PPC_ADDR64` — 64-bit VA.
pub const IMAGE_REL_PPC_ADDR64: u16 = 0x0001;
/// `IMAGE_REL_PPC_ADDR32` — 32-bit VA. The port emits this for the `.pdata`
/// `BeginAddress` and for the `.CRT$XCU` initializer slot.
pub const IMAGE_REL_PPC_ADDR32: u16 = 0x0002;
/// `IMAGE_REL_PPC_ADDR24` — low 24 bits of a VA, into a `b`/`bl` field.
pub const IMAGE_REL_PPC_ADDR24: u16 = 0x0003;
/// `IMAGE_REL_PPC_ADDR16` — low 16 bits of a VA.
pub const IMAGE_REL_PPC_ADDR16: u16 = 0x0004;
/// `IMAGE_REL_PPC_ADDR14` — low 14 bits of a VA, into a conditional-branch
/// field.
pub const IMAGE_REL_PPC_ADDR14: u16 = 0x0005;
/// `IMAGE_REL_PPC_REL24` — 24-bit PC-relative branch displacement. Every `b`
/// tail call and every `bl` the port emits carries one.
pub const IMAGE_REL_PPC_REL24: u16 = 0x0006;
/// `IMAGE_REL_PPC_REL14` — 14-bit PC-relative conditional-branch displacement.
pub const IMAGE_REL_PPC_REL14: u16 = 0x0007;
/// `IMAGE_REL_PPC_TOCREL16` — 16-bit TOC-relative offset. **Never documented by
/// Microsoft**; from `object`'s table.
pub const IMAGE_REL_PPC_TOCREL16: u16 = 0x0008;
/// `IMAGE_REL_PPC_TOCREL14` — 14-bit TOC-relative offset. **Never documented by
/// Microsoft**; from `object`'s table.
pub const IMAGE_REL_PPC_TOCREL14: u16 = 0x0009;
/// `IMAGE_REL_PPC_ADDR32NB` — 32-bit RVA (VA minus the image base).
pub const IMAGE_REL_PPC_ADDR32NB: u16 = 0x000A;
/// `IMAGE_REL_PPC_SECREL` — 32-bit offset from the start of the target's
/// section. This is how `.debug$S` points at code.
pub const IMAGE_REL_PPC_SECREL: u16 = 0x000B;
/// `IMAGE_REL_PPC_SECTION` — the 16-bit section index of the target.
pub const IMAGE_REL_PPC_SECTION: u16 = 0x000C;
/// `IMAGE_REL_PPC_IFGLUE` — "substitute a `nop` for the instruction". **Never
/// documented by Microsoft**; from `object`'s table.
pub const IMAGE_REL_PPC_IFGLUE: u16 = 0x000D;
/// `IMAGE_REL_PPC_IMGLUE` — the symbol is glue code for a cross-TOC call.
/// **Never documented by Microsoft**; from `object`'s table.
pub const IMAGE_REL_PPC_IMGLUE: u16 = 0x000E;
/// `IMAGE_REL_PPC_SECREL16` — 16-bit section-relative offset.
pub const IMAGE_REL_PPC_SECREL16: u16 = 0x000F;
/// `IMAGE_REL_PPC_REFHI` — high 16 bits of a VA. Takes a trailing `PAIR`.
pub const IMAGE_REL_PPC_REFHI: u16 = 0x0010;
/// `IMAGE_REL_PPC_REFLO` — low 16 bits of a VA. Takes a trailing `PAIR`.
pub const IMAGE_REL_PPC_REFLO: u16 = 0x0011;
/// `IMAGE_REL_PPC_PAIR` — the companion record of a `REFHI`/`REFLO` (and of
/// `SECRELHI`/`SECRELLO`). **Its `SymbolTableIndex` is not an index**: rev 6.0
/// says it holds the *displacement* the other half of the pair needs, which is
/// why every one the port emits carries 0 and why a symbol-index validator must
/// exempt it.
pub const IMAGE_REL_PPC_PAIR: u16 = 0x0012;
/// `IMAGE_REL_PPC_SECRELLO` — low 16 bits of a section-relative offset.
pub const IMAGE_REL_PPC_SECRELLO: u16 = 0x0013;
/// `IMAGE_REL_PPC_SECRELHI` — high 16 bits of a section-relative offset, plus a
/// trailing `PAIR`. Present in **PE/COFF rev 6.0** and dropped from the table in
/// the current text, which still cross-references it under `PAIR` — a live spec
/// bug, not an obsolete constant.
pub const IMAGE_REL_PPC_SECRELHI: u16 = 0x0014;
/// `IMAGE_REL_PPC_GPREL` — 16-bit offset from the GP register.
pub const IMAGE_REL_PPC_GPREL: u16 = 0x0015;
/// `IMAGE_REL_PPC_TOKEN` — a CLR metadata token.
pub const IMAGE_REL_PPC_TOKEN: u16 = 0x0016;

/// Mask selecting the relocation *type* out of the packed `Type` word.
pub const IMAGE_REL_PPC_TYPEMASK: u16 = 0x00FF;

/// Modifier: subtract the reference rather than adding it.
pub const IMAGE_REL_PPC_NEG: u16 = 0x0100;
/// Modifier: the branch is predicted taken.
pub const IMAGE_REL_PPC_BRTAKEN: u16 = 0x0200;
/// Modifier: the branch is predicted not taken.
pub const IMAGE_REL_PPC_BRNTAKEN: u16 = 0x0400;
/// Modifier: the target symbol contains a TOC-table entry definition.
pub const IMAGE_REL_PPC_TOCDEFN: u16 = 0x0800;

/// Every modifier bit, i.e. everything [`IMAGE_REL_PPC_TYPEMASK`] drops that has
/// a name. A `Type` word with a bit outside `TYPEMASK | FLAGMASK` set is
/// malformed, and [`Reloc::unknown_bits`] is how that gets reported instead of
/// ignored.
pub const IMAGE_REL_PPC_FLAGMASK: u16 =
    IMAGE_REL_PPC_NEG | IMAGE_REL_PPC_BRTAKEN | IMAGE_REL_PPC_BRNTAKEN | IMAGE_REL_PPC_TOCDEFN;

/// `IMAGE_SCN_LNK_NRELOC_OVFL` — the section header's 16-bit relocation count
/// overflowed, and the real count is in the first record's `VirtualAddress`.
const IMAGE_SCN_LNK_NRELOC_OVFL: u32 = 0x0100_0000;

/// The spelling of a *base* relocation type — the low byte, after masking.
/// `None` for a value with no row in the table, which is a finding rather than
/// something to render as a fallback string.
pub fn reloc_type_name(base: u16) -> Option<&'static str> {
    Some(match base {
        IMAGE_REL_PPC_ABSOLUTE => "ABSOLUTE",
        IMAGE_REL_PPC_ADDR64 => "ADDR64",
        IMAGE_REL_PPC_ADDR32 => "ADDR32",
        IMAGE_REL_PPC_ADDR24 => "ADDR24",
        IMAGE_REL_PPC_ADDR16 => "ADDR16",
        IMAGE_REL_PPC_ADDR14 => "ADDR14",
        IMAGE_REL_PPC_REL24 => "REL24",
        IMAGE_REL_PPC_REL14 => "REL14",
        IMAGE_REL_PPC_TOCREL16 => "TOCREL16",
        IMAGE_REL_PPC_TOCREL14 => "TOCREL14",
        IMAGE_REL_PPC_ADDR32NB => "ADDR32NB",
        IMAGE_REL_PPC_SECREL => "SECREL",
        IMAGE_REL_PPC_SECTION => "SECTION",
        IMAGE_REL_PPC_IFGLUE => "IFGLUE",
        IMAGE_REL_PPC_IMGLUE => "IMGLUE",
        IMAGE_REL_PPC_SECREL16 => "SECREL16",
        IMAGE_REL_PPC_REFHI => "REFHI",
        IMAGE_REL_PPC_REFLO => "REFLO",
        IMAGE_REL_PPC_PAIR => "PAIR",
        IMAGE_REL_PPC_SECRELLO => "SECRELLO",
        IMAGE_REL_PPC_SECRELHI => "SECRELHI",
        IMAGE_REL_PPC_GPREL => "GPREL",
        IMAGE_REL_PPC_TOKEN => "TOKEN",
        _ => return None,
    })
}

/// The modifier bits set in a packed `Type` word, by name, high bit last.
pub fn reloc_flag_names(ty: u16) -> Vec<&'static str> {
    let mut v = Vec::new();
    for (bit, name) in [
        (IMAGE_REL_PPC_NEG, "NEG"),
        (IMAGE_REL_PPC_BRTAKEN, "BRTAKEN"),
        (IMAGE_REL_PPC_BRNTAKEN, "BRNTAKEN"),
        (IMAGE_REL_PPC_TOCDEFN, "TOCDEFN"),
    ] {
        if ty & bit != 0 {
            v.push(name);
        }
    }
    v
}

/// One decoded relocation record, with the section it belongs to.
///
/// `ty` is kept **raw and packed** on purpose. Everything derived from it goes
/// through an accessor, so no caller can accidentally compare the whole word to
/// a bare type constant — that is the defect this type is shaped to prevent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reloc {
    /// 0-based index of the section whose relocation table this came from.
    pub section: usize,
    /// `VirtualAddress` — the offset within that section's raw data.
    pub va: u32,
    /// `SymbolTableIndex` — **a displacement, not an index, when
    /// [`Reloc::base`] is `PAIR`** (rev 6.0).
    pub sym: u32,
    /// The packed 16-bit `Type` word, exactly as it sits on disk.
    pub ty: u16,
}

impl Reloc {
    /// The relocation type, with the modifier bits masked off.
    pub fn base(&self) -> u16 {
        self.ty & IMAGE_REL_PPC_TYPEMASK
    }

    /// The modifier bits, with the type masked off.
    pub fn flags(&self) -> u16 {
        self.ty & IMAGE_REL_PPC_FLAGMASK
    }

    /// Bits set in `Type` that are neither a type nor a named modifier. Nonzero
    /// means this record is not understood — report it, never round it off.
    pub fn unknown_bits(&self) -> u16 {
        self.ty & !(IMAGE_REL_PPC_TYPEMASK | IMAGE_REL_PPC_FLAGMASK)
    }

    /// `true` when [`Reloc::sym`] is a symbol-table index at all. `PAIR` is the
    /// exception, and it is the only one.
    pub fn sym_is_an_index(&self) -> bool {
        self.base() != IMAGE_REL_PPC_PAIR
    }

    /// A one-line rendering: `REL24`, `REL24|BRTAKEN`, or `?0x37` for a base
    /// type with no row in the table.
    pub fn describe(&self) -> String {
        let mut s = match reloc_type_name(self.base()) {
            Some(n) => n.to_string(),
            None => format!("?0x{:02X}", self.base()),
        };
        for f in reloc_flag_names(self.ty) {
            s.push('|');
            s.push_str(f);
        }
        if self.unknown_bits() != 0 {
            s.push_str(&format!("|?0x{:04X}", self.unknown_bits()));
        }
        s
    }
}

impl ObjImage {
    /// **Every relocation record in the image**, section by section, in section
    /// order and then in table order.
    ///
    /// Same fail-closed contract as the section and COMDAT walks: `None` the
    /// moment anything does not decode — short image, a relocation table
    /// running past EOF, an overflow count that does not fit. A *short*
    /// relocation list is the dangerous answer here, because a histogram built
    /// on one reads as "this obj uses a narrow vocabulary" when what happened is
    /// that the reader gave up, and absence-read-as-success is this project's
    /// most-recorded failure. There is no partial answer.
    ///
    /// Handles `IMAGE_SCN_LNK_NRELOC_OVFL`: when the section header's 16-bit
    /// `NumberOfRelocations` reads `0xFFFF` and that characteristic is set, the
    /// true count lives in the first record's `VirtualAddress` and that first
    /// record is not itself a relocation. No obj in this workload trips it (see
    /// `docs/rungs/2026-08-04-w-reloc.md`), which is exactly why it is written
    /// down rather than assumed away.
    pub fn relocations(&self) -> Option<Vec<Reloc>> {
        let b = &self.0;
        let (nsec, _) = self.coff_layout()?;
        let mut out = Vec::new();
        for i in 0..nsec {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            let ptr = u32::from_le_bytes([b[o + 24], b[o + 25], b[o + 26], b[o + 27]]) as usize;
            let mut n = u16::from_le_bytes([b[o + 32], b[o + 33]]) as usize;
            let chars = u32::from_le_bytes([b[o + 36], b[o + 37], b[o + 38], b[o + 39]]);
            if n == 0 || ptr == 0 {
                // A nonzero count with a null `PointerToRelocations` is a
                // malformed header, not an empty table to step past quietly —
                // stepping past it is how a short list gets returned. The
                // reverse (a stale pointer with a zero count) is harmless and is
                // skipped.
                if n != 0 {
                    return None;
                }
                continue;
            }
            let mut at = ptr;
            if n == 0xFFFF && chars & IMAGE_SCN_LNK_NRELOC_OVFL != 0 {
                let hdr = b.get(at..at.checked_add(RELOC_LEN)?)?;
                n = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
                // The count word occupies a record slot of its own, and it
                // counts itself.
                n = n.checked_sub(1)?;
                at += RELOC_LEN;
            }
            let end = at.checked_add(n.checked_mul(RELOC_LEN)?)?;
            if end > b.len() {
                return None;
            }
            for k in 0..n {
                let r = at + k * RELOC_LEN;
                out.push(Reloc {
                    section: i,
                    va: u32::from_le_bytes([b[r], b[r + 1], b[r + 2], b[r + 3]]),
                    sym: u32::from_le_bytes([b[r + 4], b[r + 5], b[r + 6], b[r + 7]]),
                    ty: u16::from_le_bytes([b[r + 8], b[r + 9]]),
                });
            }
        }
        Some(out)
    }

    /// **Every `.text` COMDAT's relocation records, with the TARGET RESOLVED TO
    /// A NAME** — the reader lane `w-relo` added to close board #884.
    ///
    /// [`ObjImage::text_comdat_reloc_counts`] answers *how many*, which is the
    /// size of a blind spot and not a verdict. This answers *which*: for each
    /// `.text` COMDAT leader, its records **in disk order**, each carrying the
    /// `VirtualAddress`, the **whole packed** `Type` word and a
    /// [`RelocTarget`].
    ///
    /// # Why the target is a name and never an index
    ///
    /// `SymbolTableIndex` is an index into *this obj's* symbol table. Two objs
    /// that relocate against the same function legitimately carry two different
    /// indices, and the consumer this was written for — FUNCTION BYTE MATCH —
    /// has no second obj at all: the port hands it a *name*. So the index is
    /// resolved here, once, against the symbol table it belongs to, and the
    /// comparison downstream is between two names.
    ///
    /// # Fail-closed, exactly like every other walk in this crate
    ///
    /// `None` the moment anything does not decode: the `NRELOC_OVFL` sentinel,
    /// a table running past EOF, a `SymbolTableIndex` outside the symbol table,
    /// **or an index landing on an auxiliary record** (which is a slot, not a
    /// symbol). A short or partly-resolved answer is the dangerous one — a
    /// missing record reads as "this function relocates less than it does",
    /// which is a *credit* in the instrument downstream.
    pub fn text_comdat_relocs(&self) -> Option<Vec<(String, Vec<CodeReloc>)>> {
        let entries = self.text_comdat_entries()?;
        let targets = self.symbol_targets()?;
        let all = self.relocations()?;
        let mut out: Vec<(String, Vec<CodeReloc>)> =
            entries.iter().map(|(n, _)| (n.clone(), Vec::new())).collect();
        // Section index -> position in `out`. A `.text` COMDAT has exactly one
        // leader (`text_comdat_entries` refuses the obj otherwise), so this map
        // is injective by construction.
        let (nsec, _) = self.coff_layout()?;
        let mut slot = vec![usize::MAX; nsec];
        for (k, (_, s)) in entries.iter().enumerate() {
            if *s >= nsec {
                return None;
            }
            slot[*s] = k;
        }
        for r in all {
            if r.section >= slot.len() || slot[r.section] == usize::MAX {
                continue; // not a `.text` COMDAT — `.pdata`, `.debug$S`, …
            }
            // `PAIR` carries a DISPLACEMENT in the index field (rev 6.0), so it
            // must not be resolved as a symbol. This is the one exemption and
            // [`Reloc::sym_is_an_index`] is the single place that decides it.
            let target = if r.sym_is_an_index() {
                targets.get(r.sym as usize)?.clone()?
            } else {
                RelocTarget::PairDisplacement(r.sym)
            };
            out[slot[r.section]].1.push(CodeReloc {
                va: r.va,
                ty: r.ty,
                target,
            });
        }
        Some(out)
    }
}

/// **What one relocation record points AT**, resolved out of the symbol table.
///
/// Three variants and not one string, because a rendered string can collide: a
/// section-definition symbol's "name" *is* a section name, and a mangled symbol
/// could in principle spell the same characters. As separate variants,
/// `Section(".rdata")` can never compare equal to `Symbol(".rdata")` — and the
/// whole point of this type is to be compared.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelocTarget {
    /// An ordinary symbol, by name.
    Symbol(String),
    /// A **section-definition symbol** (`IMAGE_SYM_CLASS_STATIC` with one aux
    /// record). Its name is the *section's* name, which is not unique in a
    /// `/Gy` obj — several `.text` COMDATs all spell `.text`. Never equal to a
    /// [`RelocTarget::Symbol`].
    Section(String),
    /// A `PAIR` record's index field, which is **a displacement and not an
    /// index** (PE/COFF rev 6.0). Compared as a number.
    PairDisplacement(u32),
}

impl RelocTarget {
    /// A one-line rendering, with the two non-symbol classes bracketed so a
    /// reader can never mistake either for a mangled name.
    pub fn describe(&self) -> String {
        match self {
            RelocTarget::Symbol(n) => n.clone(),
            RelocTarget::Section(n) => format!("<section {n}>"),
            RelocTarget::PairDisplacement(d) => format!("<pair +{d}>"),
        }
    }
}

/// One `.text` COMDAT relocation with its target resolved — see
/// [`ObjImage::text_comdat_relocs`].
///
/// `va` is an offset **within that COMDAT's own raw data**, which under `/Gy`
/// is the function's own body: every function starts at offset 0 of its own
/// section, so these offsets are directly comparable with what the port plans.
///
/// `ty` is the **whole packed** `Type` word, flags included. Masking it here
/// would throw away exactly the `BRTAKEN`/`NEG`/`TOCDEFN` bits that make two
/// otherwise-identical records different relocations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeReloc {
    pub va: u32,
    pub ty: u16,
    pub target: RelocTarget,
}

impl CodeReloc {
    /// `REL24@0 -> ?g@@YAXXZ` — the form the instrument prints in a witness.
    pub fn describe(&self) -> String {
        let r = Reloc {
            section: 0,
            va: self.va,
            sym: 0,
            ty: self.ty,
        };
        format!("{}@{} -> {}", r.describe(), self.va, self.target.describe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is a transcription, so the thing worth testing is that no two
    /// rows collide and that the count is the one `object` publishes.
    #[test]
    fn the_table_has_twenty_three_distinct_base_types() {
        let mut seen = Vec::new();
        for t in 0..=0xFFu16 {
            if let Some(n) = reloc_type_name(t) {
                assert!(!seen.iter().any(|&(_, m)| m == n), "duplicate name {n}");
                seen.push((t, n));
            }
        }
        assert_eq!(
            seen.len(),
            23,
            "the complete IMAGE_REL_PPC_* table is 23 rows, 0x00..=0x16 contiguous"
        );
        assert_eq!(seen.first().map(|&(t, _)| t), Some(0x00));
        assert_eq!(seen.last().map(|&(t, _)| t), Some(0x16));
    }

    /// **The packed-word rule.** A `REL24` with `BRTAKEN` set is `0x0206`, and
    /// the whole point is that it decodes as a `REL24` and not as an unknown.
    #[test]
    fn a_flag_bit_does_not_change_the_base_type() {
        let plain = Reloc { section: 0, va: 0, sym: 3, ty: IMAGE_REL_PPC_REL24 };
        let taken = Reloc {
            section: 0,
            va: 0,
            sym: 3,
            ty: IMAGE_REL_PPC_REL24 | IMAGE_REL_PPC_BRTAKEN,
        };
        assert_eq!(taken.ty, 0x0206, "the packed word is type | flag");
        assert_eq!(taken.base(), plain.base());
        assert_eq!(taken.describe(), "REL24|BRTAKEN");
        assert_eq!(plain.describe(), "REL24");
        assert_eq!(plain.flags(), 0);
        // The refuted reading: comparing the whole word to the constant.
        assert_ne!(
            taken.ty, IMAGE_REL_PPC_REL24,
            "a whole-word compare is exactly the defect base() exists to prevent"
        );
    }

    #[test]
    fn all_four_modifiers_can_be_set_at_once() {
        let r = Reloc {
            section: 0,
            va: 0,
            sym: 0,
            ty: IMAGE_REL_PPC_REFHI | IMAGE_REL_PPC_FLAGMASK,
        };
        assert_eq!(r.base(), IMAGE_REL_PPC_REFHI);
        assert_eq!(r.describe(), "REFHI|NEG|BRTAKEN|BRNTAKEN|TOCDEFN");
        assert_eq!(r.unknown_bits(), 0);
    }

    /// A bit outside `TYPEMASK | FLAGMASK` is reported, not masked away.
    #[test]
    fn an_unnamed_high_bit_is_reported() {
        let r = Reloc { section: 0, va: 0, sym: 0, ty: 0x1006 };
        assert_eq!(r.base(), IMAGE_REL_PPC_REL24);
        assert_eq!(r.unknown_bits(), 0x1000);
        assert_eq!(r.describe(), "REL24|?0x1000");
    }

    /// A base type with no row prints its value rather than a plausible name.
    #[test]
    fn an_unknown_base_type_prints_its_number() {
        let r = Reloc { section: 0, va: 0, sym: 0, ty: 0x0037 };
        assert_eq!(reloc_type_name(0x37), None);
        assert_eq!(r.describe(), "?0x37");
    }

    /// The four Microsoft never documented, pinned by value — the reason to port
    /// `object`'s table rather than the live spec's.
    #[test]
    fn the_four_undocumented_constants_are_present() {
        assert_eq!(reloc_type_name(IMAGE_REL_PPC_TOCREL16), Some("TOCREL16"));
        assert_eq!(reloc_type_name(IMAGE_REL_PPC_TOCREL14), Some("TOCREL14"));
        assert_eq!(reloc_type_name(IMAGE_REL_PPC_IFGLUE), Some("IFGLUE"));
        assert_eq!(reloc_type_name(IMAGE_REL_PPC_IMGLUE), Some("IMGLUE"));
        assert_eq!((0x08, 0x09, 0x0D, 0x0E), (
            IMAGE_REL_PPC_TOCREL16,
            IMAGE_REL_PPC_TOCREL14,
            IMAGE_REL_PPC_IFGLUE,
            IMAGE_REL_PPC_IMGLUE
        ));
        // And the one the live spec dropped while still cross-referencing it.
        assert_eq!(IMAGE_REL_PPC_SECRELHI, 0x0014);
    }

    /// `PAIR`'s `SymbolTableIndex` is a displacement. Nothing may validate it as
    /// an index.
    #[test]
    fn pair_is_the_only_type_whose_sym_field_is_not_an_index() {
        for t in 0..=0x16u16 {
            let r = Reloc { section: 0, va: 0, sym: 0, ty: t };
            assert_eq!(
                r.sym_is_an_index(),
                t != IMAGE_REL_PPC_PAIR,
                "0x{t:02X} classified wrongly"
            );
        }
        // …and the flag bits must not change that answer either.
        let flagged = Reloc {
            section: 0,
            va: 0,
            sym: 0,
            ty: IMAGE_REL_PPC_PAIR | IMAGE_REL_PPC_TOCDEFN,
        };
        assert!(!flagged.sym_is_an_index());
    }

    // -----------------------------------------------------------------------
    // `text_comdat_relocs` — the name-resolving reader (lane `w-relo`, #884)
    // -----------------------------------------------------------------------

    /// A minimal but REAL obj: section headers with raw data and relocation
    /// tables laid out the way c2 lays them out (each section's records
    /// immediately after its own raw data), a symbol table with aux records,
    /// and a string table.
    ///
    /// `secs` is `(name, comdat, raw bytes, [(va, sym index, ty)])`;
    /// `syms` is `(name, 1-based section, storage class, naux)`.
    fn obj_with_relocs(
        secs: &[(&str, bool, Vec<u8>, Vec<(u32, u32, u16)>)],
        syms: &[(&str, i16, u8, u8)],
    ) -> Vec<u8> {
        const IMAGE_SCN_LNK_COMDAT: u32 = 0x0000_1000;
        let nsec = secs.len();
        let nsym: usize = syms.iter().map(|s| 1 + s.3 as usize).sum();
        let mut head = vec![0u8; COFF_HEADER_LEN + nsec * SECTION_HEADER_LEN];
        head[0..2].copy_from_slice(&0x01F2u16.to_le_bytes()); // POWERPCBE
        head[2..4].copy_from_slice(&(nsec as u16).to_le_bytes());
        // Raw data + relocations follow the headers, interleaved per section.
        let mut body: Vec<u8> = Vec::new();
        let base = head.len();
        for (i, (name, comdat, raw, rels)) in secs.iter().enumerate() {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            head[o..o + name.len()].copy_from_slice(name.as_bytes());
            head[o + 16..o + 20].copy_from_slice(&(raw.len() as u32).to_le_bytes());
            head[o + 20..o + 24].copy_from_slice(&((base + body.len()) as u32).to_le_bytes());
            body.extend_from_slice(raw);
            if rels.is_empty() {
                head[o + 24..o + 28].copy_from_slice(&0u32.to_le_bytes());
            } else {
                head[o + 24..o + 28].copy_from_slice(&((base + body.len()) as u32).to_le_bytes());
                for (va, sym, ty) in rels {
                    body.extend_from_slice(&va.to_le_bytes());
                    body.extend_from_slice(&sym.to_le_bytes());
                    body.extend_from_slice(&ty.to_le_bytes());
                }
            }
            head[o + 32..o + 34].copy_from_slice(&(rels.len() as u16).to_le_bytes());
            let chars = if *comdat { IMAGE_SCN_LNK_COMDAT } else { 0 };
            head[o + 36..o + 40].copy_from_slice(&chars.to_le_bytes());
        }
        let psym = base + body.len();
        head[8..12].copy_from_slice(&(psym as u32).to_le_bytes());
        head[12..16].copy_from_slice(&(nsym as u32).to_le_bytes());
        let mut strtab: Vec<u8> = vec![0, 0, 0, 0];
        let mut symtab: Vec<u8> = Vec::new();
        for (name, secnum, sclass, naux) in syms {
            let mut rec = [0u8; 18];
            if name.len() <= 8 {
                rec[..name.len()].copy_from_slice(name.as_bytes());
            } else {
                let at = strtab.len() as u32;
                strtab.extend_from_slice(name.as_bytes());
                strtab.push(0);
                rec[4..8].copy_from_slice(&at.to_le_bytes());
            }
            rec[12..14].copy_from_slice(&secnum.to_le_bytes());
            rec[16] = *sclass;
            rec[17] = *naux;
            symtab.extend_from_slice(&rec);
            symtab.extend(std::iter::repeat(0u8).take(*naux as usize * 18));
        }
        let n = strtab.len() as u32;
        strtab[0..4].copy_from_slice(&n.to_le_bytes());
        let mut out = head;
        out.extend_from_slice(&body);
        out.extend_from_slice(&symtab);
        out.extend_from_slice(&strtab);
        out
    }

    /// The `/Gy` shape this reader exists for: two `.text` COMDATs, each one
    /// function, each with one `REL24` naming a **different** callee. The
    /// records must come back per COMDAT, in disk order, with the target
    /// resolved to that callee's NAME.
    #[test]
    fn each_text_comdat_gets_its_own_records_with_the_target_resolved_to_a_name() {
        // Symbol slots: 0 `.text`+aux(1) → 2 `?f` → 3 `.text`+aux(4) → 5 `?g`
        //               6 `?ext1` → 7 `?ext2`
        let obj = ObjImage::new(obj_with_relocs(
            &[
                (
                    ".text",
                    true,
                    vec![0x48, 0, 0, 0],
                    vec![(0, 6, IMAGE_REL_PPC_REL24)],
                ),
                (
                    ".text",
                    true,
                    vec![0x48, 0, 0, 0],
                    vec![(0, 7, IMAGE_REL_PPC_REL24)],
                ),
            ],
            &[
                (".text", 1, 3, 1),
                ("?f@@YAXXZ", 1, 2, 0),
                (".text", 2, 3, 1),
                ("?g@@YAXXZ", 2, 2, 0),
                ("?ext1@@YAXXZ", 0, 2, 0),
                ("?ext2@@YAXXZ", 0, 2, 0),
            ],
        ));
        let got = obj.text_comdat_relocs().expect("the obj decodes");
        assert_eq!(got.len(), 2, "one entry per .text COMDAT");
        assert_eq!(got[0].0, "?f@@YAXXZ");
        assert_eq!(
            got[0].1,
            vec![CodeReloc {
                va: 0,
                ty: IMAGE_REL_PPC_REL24,
                target: RelocTarget::Symbol("?ext1@@YAXXZ".into()),
            }]
        );
        assert_eq!(got[1].0, "?g@@YAXXZ");
        assert_eq!(
            got[1].1[0].target,
            RelocTarget::Symbol("?ext2@@YAXXZ".into()),
            "the SECOND COMDAT's record must resolve to the second callee — \
             the two instruction words are identical and the target is the \
             only thing that distinguishes them"
        );
        assert_eq!(got[0].1[0].describe(), "REL24@0 -> ?ext1@@YAXXZ");
    }

    /// **A COMDAT with no relocations returns an EMPTY list, not an absence.**
    /// "c2 relocated nothing here" is a fact a compare must be able to
    /// contradict; folding it into the unreadable case would let a residue
    /// absorb a measurable answer.
    #[test]
    fn a_comdat_with_no_records_returns_an_empty_list() {
        let obj = ObjImage::new(obj_with_relocs(
            &[(".text", true, vec![0x4e, 0x80, 0x00, 0x20], vec![])],
            &[(".text", 1, 3, 1), ("?leaf@@YAXXZ", 1, 2, 0)],
        ));
        let got = obj.text_comdat_relocs().expect("the obj decodes");
        assert_eq!(got.len(), 1);
        assert!(got[0].1.is_empty(), "empty, and present");
    }

    /// **`PAIR` is not resolved as a symbol.** Its index field is a
    /// displacement (rev 6.0), and a reader that looked it up would report a
    /// plausible, wrong name — here it would name `.text`, symbol 0.
    #[test]
    fn a_pair_records_index_field_is_read_as_a_displacement() {
        let obj = ObjImage::new(obj_with_relocs(
            &[(
                ".text",
                true,
                vec![0x3d, 0x60, 0, 0, 0x38, 0x6b, 0, 0],
                vec![
                    (0, 3, IMAGE_REL_PPC_REFHI),
                    (0, 0, IMAGE_REL_PPC_PAIR),
                    (4, 3, IMAGE_REL_PPC_REFLO),
                    (4, 0, IMAGE_REL_PPC_PAIR),
                ],
            )],
            &[(".text", 1, 3, 1), ("?f@@YAXXZ", 1, 2, 0), ("?gv@@3HA", 0, 2, 0)],
        ));
        let got = obj.text_comdat_relocs().expect("the obj decodes");
        let r = &got[0].1;
        assert_eq!(r.len(), 4, "the REFHI/PAIR/REFLO/PAIR quad, in disk order");
        assert_eq!(r[0].target, RelocTarget::Symbol("?gv@@3HA".into()));
        assert_eq!(
            r[1].target,
            RelocTarget::PairDisplacement(0),
            "symbol index 0 is `.text` here — reading a PAIR as a symbol would \
             name it, which is the defect `sym_is_an_index` exists to prevent"
        );
        assert_eq!(r[3].target, RelocTarget::PairDisplacement(0));
        assert_eq!(r[1].describe(), "PAIR@0 -> <pair +0>");
    }

    /// **A section-definition symbol is a `Section`, never a `Symbol`.** Its
    /// name repeats a section name, which is not unique under `/Gy`, so the
    /// variant is what keeps it from comparing equal to a mangled name that
    /// happens to spell the same characters.
    #[test]
    fn a_section_definition_target_comes_back_as_a_section() {
        let obj = ObjImage::new(obj_with_relocs(
            &[
                (
                    ".text",
                    true,
                    vec![0x3d, 0x60, 0, 0],
                    vec![(0, 2, IMAGE_REL_PPC_REFHI)],
                ),
                (".rdata", false, vec![0, 0, 0, 0], vec![]),
            ],
            &[
                (".text", 1, 3, 1),
                (".rdata", 2, 3, 1),
                ("?f@@YAXXZ", 1, 2, 0),
            ],
        ));
        let got = obj.text_comdat_relocs().expect("the obj decodes");
        assert_eq!(
            got[0].1[0].target,
            RelocTarget::Section(".rdata".into()),
            "IMAGE_SYM_CLASS_STATIC with one aux record is a section definition"
        );
        assert_ne!(
            got[0].1[0].target,
            RelocTarget::Symbol(".rdata".into()),
            "and it must NOT equal a symbol of the same spelling"
        );
        assert_eq!(got[0].1[0].describe(), "REFHI@0 -> <section .rdata>");
    }

    /// **Fail closed: an index landing on an AUXILIARY record is not a
    /// symbol.** Resolving it to the preceding symbol's name would be
    /// plausible and wrong; a short or mis-resolved answer is the dangerous one
    /// here, because a missing record reads as "this function relocates less
    /// than it does", which is a CREDIT in the instrument downstream.
    #[test]
    fn an_index_landing_on_an_aux_record_refuses_the_whole_obj() {
        let obj = ObjImage::new(obj_with_relocs(
            // Slot 1 is `.text`'s aux record, not a symbol.
            &[(
                ".text",
                true,
                vec![0x48, 0, 0, 0],
                vec![(0, 1, IMAGE_REL_PPC_REL24)],
            )],
            &[(".text", 1, 3, 1), ("?f@@YAXXZ", 1, 2, 0)],
        ));
        assert_eq!(
            obj.text_comdat_relocs(),
            None,
            "an aux slot is not a symbol; there is no partial answer"
        );
    }

    /// The same fail-closed contract for an index past the end of the table.
    #[test]
    fn an_out_of_range_symbol_index_refuses_the_whole_obj() {
        let obj = ObjImage::new(obj_with_relocs(
            &[(
                ".text",
                true,
                vec![0x48, 0, 0, 0],
                vec![(0, 99, IMAGE_REL_PPC_REL24)],
            )],
            &[(".text", 1, 3, 1), ("?f@@YAXXZ", 1, 2, 0)],
        ));
        assert_eq!(obj.text_comdat_relocs(), None);
    }

    /// **Records from non-`.text` sections are not attributed to a function.**
    /// `.debug$S` relocates against code constantly; folding those into a
    /// function's set would make every graded function disagree.
    #[test]
    fn a_non_text_sections_records_are_not_attributed_to_a_function() {
        let obj = ObjImage::new(obj_with_relocs(
            &[
                (
                    ".text",
                    true,
                    vec![0x48, 0, 0, 0],
                    vec![(0, 3, IMAGE_REL_PPC_REL24)],
                ),
                (
                    ".pdata",
                    true,
                    vec![0, 0, 0, 0, 0, 0, 0, 0],
                    vec![(0, 2, IMAGE_REL_PPC_ADDR32)],
                ),
            ],
            &[
                (".text", 1, 3, 1),
                ("?f@@YAXXZ", 1, 2, 0),
                ("?ext@@YAXXZ", 0, 2, 0),
            ],
        ));
        let got = obj.text_comdat_relocs().expect("the obj decodes");
        assert_eq!(got.len(), 1, "`.pdata` is a COMDAT but it is not `.text`");
        assert_eq!(got[0].1.len(), 1, "only the function's own REL24");
        assert_eq!(
            got[0].1[0].target,
            RelocTarget::Symbol("?ext@@YAXXZ".into())
        );
    }

    /// The reader and the counter must agree about how many records each COMDAT
    /// has — two walks over one obj, and a disagreement between them is exactly
    /// how a blind spot survives a green count.
    #[test]
    fn the_record_reader_agrees_with_the_record_counter() {
        let obj = ObjImage::new(obj_with_relocs(
            &[
                // Slot 6 is `?ext`: 0 `.text` · 1 aux · 2 `?f` · 3 `.text` ·
                // 4 aux · 5 `?leaf` · 6 `?ext`. Written out because getting it
                // wrong is what the aux-slot refusal above exists to catch, and
                // this test caught it.
                (
                    ".text",
                    true,
                    vec![0x48, 0, 0, 0],
                    vec![(0, 6, IMAGE_REL_PPC_REL24)],
                ),
                (".text", true, vec![0x4e, 0x80, 0x00, 0x20], vec![]),
            ],
            &[
                (".text", 1, 3, 1),
                ("?f@@YAXXZ", 1, 2, 0),
                (".text", 2, 3, 1),
                ("?leaf@@YAXXZ", 2, 2, 0),
                ("?ext@@YAXXZ", 0, 2, 0),
            ],
        ));
        let counts = obj.text_comdat_reloc_counts().expect("counts decode");
        let recs = obj.text_comdat_relocs().expect("records decode");
        assert_eq!(counts.len(), recs.len());
        for ((cn, c), (rn, r)) in counts.iter().zip(&recs) {
            assert_eq!(cn, rn, "same leader, same position");
            assert_eq!(*c, r.len(), "{cn}: counter says {c}, reader says {}", r.len());
        }
    }
}
