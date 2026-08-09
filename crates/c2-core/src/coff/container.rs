//! The COFF container itself: header, section table, section sizing, and the
//! alignment/allocation rules that decide where bytes land.
//!
//! Nothing here knows what a function or a dynamic initializer is.

use super::*;

// COFF machine + characteristics.
pub(crate) const MACHINE_POWERPCBE: u16 = 0x01F2;
pub(crate) const CHARACTERISTICS: u16 = 0x0180;


pub(crate) const CH_TEXT: u32 = 0x6040_0020;

pub(crate) const SECTION_HEADER_LEN: usize = 40;
pub(crate) const COFF_HEADER_LEN: usize = 20;

/// Which function a `/Gy` section belongs to. The COMDAT layout interleaves
/// `.text` and `.pdata` per function, so "section index minus the fixed prefix"
/// is **not** the function index once any function is framed — that arithmetic
/// is what this replaces.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SectionOwner {
    /// One of the four fixed sections every obj carries.
    Fixed,
    /// Function `i`'s own `.text` COMDAT.
    Text(usize),
    /// Function `i`'s `.pdata` COMDAT.
    Pdata(usize),
    /// **W-DATA** — the COMDAT `.data` of the `j`-th defined data object, placed
    /// **after** every code group. It carries no relocations of its own; the
    /// relocations that name it live in the referring function's `.text`.
    Data(usize),
    /// **W-BIQUAD** — the COMDAT `.rdata` of pool entry `k`, placed immediately
    /// after the `.text` of the function that first references it. Like
    /// [`SectionOwner::Data`] it carries no relocations of its own; the
    /// REFHI/REFLO quads that name it live in the referring function's `.text`.
    Rdata(usize),
}

/// A section, resolved to its raw data + header metadata.
pub(crate) struct Section<'a> {
    pub(crate) name: &'static str,
    pub(crate) characteristics: u32,
    /// Raw section data. Borrowed for the fixed blobs (`.drectve`, the XBLD
    /// watermarks) and the caller's `.text`; owned only where it is actually
    /// built per obj (`.debug$S`, `.pdata`, the `.rdata` pools). The emitted
    /// bytes are identical either way — this only removes per-obj copies.
    pub(crate) raw: std::borrow::Cow<'a, [u8]>,
    /// Aux section-def CheckSum (0 for non-COMDAT).
    pub(crate) checksum: u32,
    /// COMDAT selection (0 = not COMDAT; 2 = SELECT_ANY; 1 = NODUPLICATES;
    /// 5 = ASSOCIATIVE).
    pub(crate) selection: u8,
    /// Aux section-def `Number`. Zero everywhere except a Selection=5
    /// (ASSOCIATIVE) COMDAT, where it is the **1-based section number of the
    /// section this one is tied to** — the mechanism `/Gy` uses to attach a
    /// function's `.pdata` COMDAT to its `.text` COMDAT so the linker discards
    /// both together.
    pub(crate) assoc: u16,
    /// `Some(n)` for an **uninitialized** section (`.bss`): the section is `n`
    /// bytes long as far as `SizeOfRawData` and the aux `Length` are concerned,
    /// but it contributes **zero bytes to the file** and its
    /// `PointerToRawData` is 0.
    ///
    /// This inversion is the doc's refuted prediction P8
    /// (`docs/OBJ_DYNINIT_SHAPE.md` §1) — the natural guess is
    /// `SizeOfRawData = 0` with the size in `VirtualSize`, and c2 does the exact
    /// opposite: `VirtualSize` is 0 in **every** section including `.bss`.
    /// Everything else in this file conflates "how many bytes the section is"
    /// with `raw.len()`, in four places (the header, the layout cursor, the raw
    /// write and the aux `Length`), which is why this is a field on `Section`
    /// and not a special case at the one call site — three of those four would
    /// still have been wrong.
    pub(crate) uninit_size: Option<u32>,
}

impl Section<'_> {
    /// The section's length as the container reports it — `SizeOfRawData` and
    /// the aux section-def `Length`. Equal to `raw.len()` except for `.bss`.
    pub(crate) fn size(&self) -> u32 {
        self.uninit_size.unwrap_or(self.raw.len() as u32)
    }
    /// How many bytes this section contributes to the obj **file**. Zero for an
    /// uninitialized section, whatever its [`Section::size`].
    pub(crate) fn file_len(&self) -> usize {
        if self.uninit_size.is_some() { 0 } else { self.raw.len() }
    }
}


/// Lay the obj out after the section headers and return
/// `(PointerToRawData per section, PointerToRelocations per section, the
/// symbol-table offset)`.
///
/// Two rules, both of which this file has already been wrong about once:
///
/// * **a section's relocations immediately follow its OWN raw data**, before the
///   next section's — not all raw data then all relocations. That was invisible
///   while at most one section had relocations, and wrong from the fifth section
///   header on once two did (`docs/ROADMAP.md`; the note on
///   [`emit_comdat_obj`]).
/// * **an uninitialized section advances the cursor by nothing and gets
///   `PointerToRawData = 0`**, however large its [`Section::size`] — `.bss` sits
///   between `.rdata` and `.CRT$XCU` in the dynamic-initializer obj and those
///   two are contiguous in the file.
pub(crate) fn layout_sections(
    sections: &[Section],
    n_reloc_of: &[u16],
) -> (Vec<usize>, Vec<Option<usize>>, usize) {
    debug_assert_eq!(sections.len(), n_reloc_of.len());
    let mut ptrs = Vec::with_capacity(sections.len());
    let mut reloc_ptr: Vec<Option<usize>> = vec![None; sections.len()];
    let mut cursor = COFF_HEADER_LEN + sections.len() * SECTION_HEADER_LEN;
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_some() {
            ptrs.push(0);
        } else {
            ptrs.push(cursor);
            cursor += s.raw.len();
        }
        if n_reloc_of[i] > 0 {
            reloc_ptr[i] = Some(cursor);
            cursor += n_reloc_of[i] as usize * RELOC_LEN;
        }
    }
    (ptrs, reloc_ptr, cursor)
}

/// The 20-byte COFF header. `TimeDateStamp` is 0 — the differential normalizes
/// it away; every other byte must genuinely match.
pub(crate) fn write_coff_header(b: &mut Buf, n_sections: usize, ptr_symtab: usize, n_symbols: u32) {
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);
}

/// The section headers, 40 bytes each. `VirtualSize`, `VirtualAddress`,
/// `PointerToLinenumbers` and `NumberOfLinenumbers` are 0 in every section of
/// every reference obj measured — **including `.bss`**, whose size lives in
/// `SizeOfRawData` beside a null `PointerToRawData`.
pub(crate) fn write_section_headers(
    b: &mut Buf,
    sections: &[Section],
    ptrs: &[usize],
    reloc_ptr: &[Option<usize>],
    n_reloc_of: &[u16],
) {
    for (i, s) in sections.iter().enumerate() {
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.size()); // SizeOfRawData
        b.u32(ptrs[i] as u32); // PointerToRawData
        b.u32(reloc_ptr[i].unwrap_or(0) as u32);
        b.u32(0); // PointerToLinenumbers
        b.u16(n_reloc_of[i]);
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }
}


/// The alignment nibble (bits 23:20 of `Characteristics`) for a blob of `n`
/// bytes whose natural alignment is `t`:
///
/// > `align = max(t, 1 if n < 2 else 4 if n < 64 else 8)`
///
/// One rule for both `.bss` and the string `.rdata`, measured on both sides
/// across `n = 1, 2, 3..63, 64, 65..256` (`docs/OBJ_DYNINIT_SHAPE.md` §4.2).
/// `t` moves independently: a `double` member gives ALIGN_8 at `n = 8` where a
/// `char[8]` gives ALIGN_4.
///
/// The nibble is `log2(align) + 1` — 1→1, 2→2, 4→3, 8→4, **16→5** (board
/// #1120). Returns `None` for an alignment that is not a power of two in
/// 1..=16, rather than emitting a nibble for a case nothing measured.
///
/// **32 and 64 are real and are still refused.** `__declspec(align(32))` and
/// `align(64)` make c2 emit nibbles **6** and **7** — measured on cells
/// `A09`/`A10`/`A18` of `work/w-align16/` — so the `log2 + 1` law itself is
/// confirmed to 64. What is *not* confirmed above 16 is the rest of the model
/// that consumes this number: [`super::data::bump_layout`]'s cursor and
/// [`super::data::section_nibble`]'s max were graded on a structural grid at 16
/// and on nothing at 32/64. Extending by `log2` past the cells that constrain
/// it is exactly the "mostly right" table a refusal beats.
pub(crate) fn align_nibble(n: u32, natural: u32) -> Option<u32> {
    match placement_align(n, natural)? {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
        16 => Some(5),
        _ => None,
    }
}

/// The **size-promoted** alignment of a blob of `n` bytes with natural alignment
/// `t`, in bytes:
///
/// > `align = max(t, 1 if n < 2 else 4 if n < 64 else 8)`
///
/// This is the *same* promotion in two places, which is why it is one function.
/// `docs/OBJ_DYNINIT_SHAPE.md` §4.2 measures it as the rule for a section's
/// alignment **nibble**; `docs/OBJ_DATA_BSS_SHAPE.md` §5.4 (Rule A3) measures it
/// as the alignment the `.bss` **allocator** rounds each object up to. Prereg P9
/// of that lane predicted the allocator used the plain *natural* alignment and
/// was refuted — a natural-alignment allocator scores 7/18 against this one's
/// 14/18 (§5.5). Keeping [`align_nibble`] and the allocator
/// (`super::data::bump_layout`, Rule A3′) on one body means a later correction to
/// the thresholds cannot land in one and not the other. The §5.4 figures above
/// are the *promotion*'s provenance and survive §5.7's revision of the allocator
/// they were first measured through.
///
/// # The `implied` ceiling is 8, re-measured at n = 4096 (board #1120)
///
/// `implied` was fitted over `n = 1 … 256` and it would have been easy to read
/// its top clause as "8 and rising". It is not: `char g[4096]` — natural
/// alignment 1, size 4096 — gets `Characteristics` nibble **4**, i.e. ALIGN_8,
/// from real c2 (cell `A07`, and `A08` at 256). **Nothing is promoted past 8 by
/// SIZE.** Everything above 8 in this table therefore arrives through `natural`,
/// which is read off the `.gl` type tag and never inferred from the type
/// (`c2_il::func::gl::align_of_type_tag`, and see `w-align`'s `T16` — a natural
/// reading there was a live wrong emit, not a refusal).
///
/// `None` outside `{1,2,4,8,16}`. **16 is in and 32/64 are out, and both halves
/// of that are measurements** (`work/w-align16/`): `__declspec(align(16))` is
/// nibble 5 and is graded byte-exact through both consumers on a grid that
/// varies structure nine ways; `align(32)`/`align(64)` are nibbles 6 and 7 and
/// are refused, because the grid varies *nothing* at those values. See
/// [`align_nibble`].
pub(crate) fn placement_align(n: u32, natural: u32) -> Option<u32> {
    let implied: u32 = if n < 2 {
        1
    } else if n < 64 {
        4
    } else {
        8
    };
    match natural.max(implied) {
        a @ (1 | 2 | 4 | 8 | 16) => Some(a),
        _ => None,
    }
}

#[cfg(test)]
mod alloc_tests {
    use super::*;

    /// [`align_nibble`] and the `.data`/`.bss` allocator (`super::data::bump_layout`,
    /// Rule A3′) share one promotion table, so the size thresholds cannot drift
    /// apart. Pin both sides of each boundary.
    #[test]
    fn the_promotion_table_is_shared() {
        for (n, want) in [(1u32, 1u32), (2, 4), (63, 4), (64, 8), (256, 8)] {
            assert_eq!(placement_align(n, 1), Some(want), "n = {n}");
        }
        assert_eq!(placement_align(8, 8), Some(8), "natural beats implied");
        assert_eq!(placement_align(1, 1), Some(1));
        assert_eq!(align_nibble(64, 1), Some(4), "ALIGN_8 nibble");
        assert_eq!(align_nibble(2, 1), Some(3), "ALIGN_4 nibble");
        // **Board #1120.** The 16 arm has to be pinned in the SHARED test, not
        // only in `align_nibble`'s own, because the thing that can drift is the
        // allocator: `super::data::bump_layout` rounds its `.bss` cursor with
        // this function and cell `A13` puts a 16-aligned object at offset 16
        // behind a one-byte `char`.
        assert_eq!(placement_align(16, 16), Some(16), "A02 declspec(align(16))");
        assert_eq!(placement_align(4, 16), Some(16), "A01 scalar: n < natural");
        assert_eq!(align_nibble(16, 16), Some(5), "ALIGN_16 nibble");
        // The `implied` ceiling does not climb past 8 — `A07` is `char[4096]`
        // and c2 gives it ALIGN_8.
        assert_eq!(placement_align(4096, 1), Some(8), "A07: size never implies 16");
        assert_eq!(placement_align(65536, 1), Some(8), "and it does not climb later");
        // 32 and 64 are measured (nibbles 6 and 7) and deliberately refused.
        assert_eq!(placement_align(32, 32), None, "A09 align(32) — measured, refused");
        assert_eq!(placement_align(64, 64), None, "A10 align(64) — measured, refused");
    }
}
