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
/// The nibble is `log2(align) + 1` — 1→1, 2→2, 4→3, 8→4. Returns `None` for an
/// alignment that is not a power of two in 1..=8, rather than emitting a nibble
/// for a case nothing measured.
pub(crate) fn align_nibble(n: u32, natural: u32) -> Option<u32> {
    match placement_align(n, natural)? {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
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
/// 14/18 (§5.5). Keeping [`align_nibble`] and [`bss_deferred_layout`] on one body
/// means a later correction to the thresholds cannot land in one and not the
/// other.
///
/// `None` outside `{1,2,4,8}`: the section nibble has no encoding above ALIGN_8
/// here, so a `__declspec(align(16))` object is refused rather than rounded to
/// something plausible.
pub(crate) fn placement_align(n: u32, natural: u32) -> Option<u32> {
    let implied: u32 = if n < 2 {
        1
    } else if n < 64 {
        4
    } else {
        8
    };
    match natural.max(implied) {
        a @ (1 | 2 | 4 | 8) => Some(a),
        _ => None,
    }
}

/// The `.bss` address walk for a run of **deferred** (dynamic-initializer)
/// objects: one offset per object plus the section's total size, or `None` for a
/// case this was not measured on.
///
/// `objects` is `(size, natural_align)` **in `.gl` record order**, and the
/// returned offsets are parallel to it — `.gl` order, not walk order, because
/// that is the order the symbol table wants (Rule Y2, §6.2).
///
/// Two independent rules from `docs/OBJ_DATA_BSS_SHAPE.md`, applied in order:
///
/// * **Rule A1 (§5.2)** — walk the deferred objects in the **reverse** of their
///   `.gl` record order, so this loop runs back to front. (Eager objects walk
///   forwards; this port has no eager-object path and must not grow one here by
///   accident.)
/// * **Rule A3 (§5.4)** — one cursor from 0. Round it up to the object's
///   [`placement_align`]; the skipped bytes become a **hole**. Before taking from
///   the cursor, try the **lowest-addressed hole that fits at the object's
///   alignment**, splitting the hole around it. `SizeOfRawData` is the final
///   cursor.
///
/// The hole reuse is not decoration: §5.4's worked example places an object at
/// `0x0c` while the cursor stands at `0x18`, and §5.5 scores a no-reuse
/// allocator at 12/18 against this one's 14/18.
///
/// **MEASURED, this lane**, on a three-object mixed-size probe (`sizeof` 1 / 8 /
/// 4, `.gl` order `sB sA sC`): the reversed walk `sC sA sB` gives `sC@0`, `sA@4`,
/// a hole `[5,8)` from `sB`'s round-up, `sB@8`, final cursor `0x10` — every
/// offset and the section size exactly what the real obj carries. Mixed sizes are
/// the class §5.5 flags as *not* exact (14/18), so this is graded by the
/// differential on every run rather than assumed.
pub(crate) fn bss_deferred_layout(objects: &[(u32, u32)]) -> Option<(Vec<u32>, u32)> {
    let mut offsets = vec![0u32; objects.len()];
    let mut cursor: u32 = 0;
    // Half-open `[lo, hi)` ranges skipped by an earlier round-up, kept sorted by
    // `lo`, so "the lowest-addressed hole that fits" is the first one that does.
    let mut holes: Vec<(u32, u32)> = Vec::new();
    for idx in (0..objects.len()).rev() {
        let (size, natural) = objects[idx];
        if size == 0 {
            return None; // a zero-length object has no measured placement
        }
        let align = placement_align(size, natural)?;
        let mut placed = None;
        for h in 0..holes.len() {
            let (lo, hi) = holes[h];
            let at = lo.checked_next_multiple_of(align)?;
            if at.checked_add(size)? <= hi {
                holes.remove(h);
                // Split around the object. A zero-width remnant is NOT a hole:
                // keeping `[x, x)` would leave a range that "fits" every future
                // request of size 0 and reorders nothing visibly until it does.
                let mut at_h = h;
                if at > lo {
                    holes.insert(at_h, (lo, at));
                    at_h += 1;
                }
                if at + size < hi {
                    holes.insert(at_h, (at + size, hi));
                }
                placed = Some(at);
                break;
            }
        }
        let at = match placed {
            Some(at) => at,
            None => {
                let at = cursor.checked_next_multiple_of(align)?;
                if at > cursor {
                    holes.push((cursor, at));
                    holes.sort_unstable();
                }
                cursor = at.checked_add(size)?;
                at
            }
        };
        offsets[idx] = at;
    }
    Some((offsets, cursor))
}

#[cfg(test)]
mod alloc_tests {
    use super::*;

    /// **The mixed-size cell, measured against a real obj this lane compiled.**
    ///
    /// `psize.cpp` — three dynamic-initializer objects of `sizeof` 1, 8 and 4 —
    /// captures `.gl` order `sB sA sC`, and the real obj carries
    /// `sB@8 sA@4 sC@0` with `SizeOfRawData = 0x10`.
    ///
    /// Rule A1 walks deferred objects in the REVERSE of `.gl` order, so the walk
    /// is `sC sA sB`: `sC` (4 B, align 4) at 0, `sA` (1 B, align 1) at 4, then
    /// `sB` (8 B, align 8) rounds the cursor 5 → 8 and leaves the hole `[5,8)`.
    ///
    /// This is one of the mixed-size cells `docs/OBJ_DATA_BSS_SHAPE.md` §5.5
    /// scores the model at only 14/18 on, which is exactly why it is pinned here
    /// rather than assumed.
    #[test]
    fn the_measured_mixed_size_cell() {
        // (size, natural_align), in `.gl` record order.
        let objs = [(1u32, 1u32), (8, 8), (4, 4)];
        let (offsets, size) = bss_deferred_layout(&objs).expect("all three are in class");
        assert_eq!(offsets, vec![4, 8, 0], "sA@4 sB@8 sC@0, parallel to `.gl` order");
        assert_eq!(size, 0x10);
    }

    /// The uniform case, where the walk is the only thing that shows: six 1-byte
    /// objects land at consecutive addresses in reverse `.gl` order. Measured on
    /// `p6.cpp`, whose `.gl` spells `s2 s1 s5 s3 s4 s6` and whose obj carries
    /// `s6@0 s4@1 s3@2 s5@3 s1@4 s2@5` — `docs/OBJ_DATA_BSS_SHAPE.md` §7.1's
    /// family-A row for N = 6, reproduced.
    #[test]
    fn six_one_byte_objects_run_backwards() {
        let objs = [(1u32, 1u32); 6];
        let (offsets, size) = bss_deferred_layout(&objs).unwrap();
        assert_eq!(offsets, vec![5, 4, 3, 2, 1, 0]);
        assert_eq!(size, 6);
    }

    /// **The hole is reused, and this is the assertion that says so.** Without
    /// reuse the last object would take the cursor and the section would be
    /// larger; §5.5 scores no-reuse at 12/18 against 14/18.
    #[test]
    fn a_later_object_lands_in_an_earlier_holes_gap() {
        // `.gl` order chosen so the reversed walk is (8 B align 8), (1 B), (1 B):
        // cursor 0 → 8 B at 0, 1 B at 8, 1 B at 9. No hole. Now front-load a
        // 1-byte object so the 8-aligned one must skip: walk = 1 B, 8 B, 1 B.
        let objs = [(1u32, 1u32), (8, 8), (1, 1)];
        let (offsets, size) = bss_deferred_layout(&objs).unwrap();
        // walk: [2]=1 B @0; [1]=8 B rounds 1 → 8, hole [1,8); [0]=1 B fills it @1.
        assert_eq!(offsets, vec![1, 8, 0]);
        assert_eq!(size, 0x10);
    }

    /// A zero-length object and an over-aligned one are **refused**, not
    /// approximated — the section nibble has no encoding above ALIGN_8.
    #[test]
    fn out_of_class_inputs_refuse() {
        assert!(bss_deferred_layout(&[(0, 1)]).is_none(), "zero-length");
        assert!(bss_deferred_layout(&[(4, 16)]).is_none(), "ALIGN_16");
    }

    /// [`align_nibble`] and [`bss_deferred_layout`] share one promotion table, so
    /// the size thresholds cannot drift apart. Pin both sides of each boundary.
    #[test]
    fn the_promotion_table_is_shared() {
        for (n, want) in [(1u32, 1u32), (2, 4), (63, 4), (64, 8), (256, 8)] {
            assert_eq!(placement_align(n, 1), Some(want), "n = {n}");
        }
        assert_eq!(placement_align(8, 8), Some(8), "natural beats implied");
        assert_eq!(placement_align(1, 1), Some(1));
        assert_eq!(align_nibble(64, 1), Some(4), "ALIGN_8 nibble");
        assert_eq!(align_nibble(2, 1), Some(3), "ALIGN_4 nibble");
    }
}
