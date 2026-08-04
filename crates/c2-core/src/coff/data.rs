//! **The `.data` / `.bss` obj for a TU that defines no functions** (board #174).
//!
//! `docs/OBJ_DATA_BSS_SHAPE.md` is the specification; this is the writer, scoped
//! to what §5.7 measured rather than to what §2–§6 describes.
//!
//! # Why this shape and not a general `.data` path
//!
//! The document's own §8.1 states the bound, and the bound is about **object
//! count per section**, not about size:
//!
//! * a section with **one** object is trivially right, and that is **23,253 of
//!   the workload's 24,055** `.data`/`.bss` sections;
//! * a `.bss` with **exactly two** objects is right on **47 of 48** real
//!   sections;
//! * anything larger is **38 of 62** — refuse, do not guess.
//!
//! The residual above two objects is the **walk order** (board #184) and
//! emphatically not the arithmetic: of the 64 real `.bss` sections whose walk
//! needs no alignment padding anywhere — where every candidate allocator
//! coincides by construction — **10 are still wrong**. So [`emit_data_obj`]
//! gates on the count and returns `None` above it.
//!
//! # The allocator is A3′, and it is NOT [`bss_deferred_layout`]
//!
//! `container.rs` carries `bss_deferred_layout`, which implements §5.4's Rule A3
//! — a bump **with hole reuse**. §5.7 supersedes that clause with **Rule A3′**:
//!
//! > one cursor per section starting at 0, each object placed at the cursor
//! > rounded up to `align(obj) = max(t, 1 if n<2 else 4 if n<64 else 8)`, the
//! > cursor advanced past it, `SizeOfRawData` = the final cursor. **There is no
//! > free list.**
//!
//! A3′ is exact on **110 of 117** real `.bss` sections, **68 of 68** real
//! `.data` and **38 of 38** probe cells. "Hole reuse", "pass-over" and
//! "best-fit" are not three rival allocators — each is a different story about
//! the *order* the objects were visited in, and every one of them emits a layout
//! that is a bump in *some* order. So this file walks and bumps, and does not
//! call `bss_deferred_layout`: that helper has **never had a caller**, has
//! therefore never been graded by the differential, and its unit tests assert
//! the clause A3′ replaced.
//!
//! # The two walks are different, and that is the load-bearing fact
//!
//! | section | walk | source |
//! |---|---|---|
//! | `.bss` | **`.gl` record order**, forwards | Rule A1, §5.2 |
//! | `.data` | **declaration order** | Rule A2, §5.3 |
//!
//! They are different permutations of the same names — measured on six objects
//! whose `.gl` file order is `zulu yankee mike charlie bravo alpha` and whose
//! declaration order is `zulu alpha mike bravo yankee charlie`. Rule A1 beats
//! declaration order by 36 real sections on `.bss` (89 vs 53) and Rule A2 beats
//! `.gl` order by 27 on `.data` (46 vs 19), so reading one order for both is not
//! a simplification, it is 36 or 27 wrong objs.
//!
//! `.gl` order is the input vector's own order; declaration order is
//! `DataObj::decl_index` (the record's operand token, §5.6).

use super::*;

/// `.data` characteristics with the alignment nibble cleared: CNT_INIT_DATA |
/// READ | WRITE. OR in `nibble << 20`. The COMDAT form (`| 0x1000`, Selection 2)
/// is **not** emitted here — see [`emit_data_obj`]'s class check.
pub(crate) const CH_DATA_BASE: u32 = 0xC000_0040;

/// One namespace-scope object, as the writer needs it.
///
/// Deliberately not `c2_il::DataObject`: that one is what the **IL says** and
/// this one is what the **obj gets**. Keeping them apart is what lets the decode
/// bound and the layout bound be enforced in their own crates without either
/// assuming the other ran.
pub struct DataObj<'a> {
    /// The COFF symbol name, already final: undecorated (`s1`) for internal
    /// linkage, decorated (`?d1@@3HA`) for external (§6.1).
    pub symbol: &'a str,
    /// `sizeof` the object.
    pub size: u32,
    /// The object's natural alignment in bytes, from the `.gl` TYPE tag —
    /// **not** derived from the size. `double` is 8-aligned at `n = 8` where
    /// `char[8]` is 4-aligned.
    pub natural_align: u32,
    /// `true` => StorageClass 2 EXTERNAL; `false` => 3 STATIC (§6.1).
    pub external: bool,
    /// `Some(bytes)` => `.data`, with the raw bytes in the obj's order and
    /// `bytes.len() == size`. `None` => `.bss`.
    pub bytes: Option<&'a [u8]>,
    /// The declaration-order key — Rule A2's walk for `.data` (§5.3, §5.6).
    /// Only its **order** is used; the value itself is never emitted.
    pub decl_index: u32,
}

/// The most objects this writer will place in one non-COMDAT section.
///
/// **A measured bound, not a preference** (§8.1). Two is where a `.bss` is right
/// on 47 of 48 real sections; three is where it falls to 38 of 62. Raising this
/// requires board **#184**, not a bigger grid.
pub(crate) const MAX_OBJECTS_PER_SECTION: usize = 2;

/// Lay out one section's objects by **Rule A3′** — a plain bump, no free list.
///
/// `walk` is the visit order as indices into `objs`. Returns the offset for each
/// index of `objs` (parallel to `objs`, not to `walk`) and the section's final
/// size, or `None` for an object whose alignment is outside the modeled set.
fn bump_layout(objs: &[&DataObj<'_>], walk: &[usize]) -> Option<(Vec<u32>, u32)> {
    let mut offsets = vec![0u32; objs.len()];
    let mut cursor: u32 = 0;
    for &i in walk {
        let o = objs[i];
        if o.size == 0 {
            return None; // a zero-length object has no measured placement
        }
        let align = placement_align(o.size, o.natural_align)?;
        let at = cursor.checked_next_multiple_of(align)?;
        cursor = at.checked_add(o.size)?;
        offsets[i] = at;
    }
    Some((offsets, cursor))
}

/// The section's alignment nibble — **Rule B1**: the maximum over the objects it
/// holds of each object's own [`placement_align`], and not of their natural
/// alignments (§3.2).
fn section_nibble(objs: &[&DataObj<'_>]) -> Option<u32> {
    let mut best = 1u32;
    for o in objs {
        best = best.max(placement_align(o.size, o.natural_align)?);
    }
    match best {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
        _ => None,
    }
}

/// Emit the obj for a TU that defines **no functions** and one or more
/// namespace-scope objects, or `None` if the inputs fall outside the class this
/// was measured on — in which case the caller reports `NotImplemented`.
///
/// `objects` must be in **`.gl` record order**, which is Rule A1's `.bss` walk.
///
/// # Section order
///
/// **Rule S1** (§2.2), and two of its clauses are counter-intuitive enough that
/// prereg P3 got them backwards: the *uninitialized* section comes **first**,
/// and the two data sections sit **inside** the watermark shell rather than
/// after it.
///
/// ```text
///   1 .drectve   2 .debug$S   3 .XBLD$W(C2)   [.bss]   .XBLD$W(C1)   [.data]
/// ```
///
/// `.bss` occupies the slot **between** the two `.XBLD$W` watermarks — across
/// all 871 workload objs the only section that ever appears there is a `.bss`,
/// with **zero** exceptions — and `.data` comes **after** the second watermark,
/// in 754 of the 754 objs that have one.
///
/// # Symbol-table order
///
/// The table follows section order, and within a section's group the section
/// symbol + aux comes first. The order of the *defined* symbols inside a group
/// is **linkage-dependent** for `.bss` and is not ascending address (§6.2):
///
/// > **Rule Y1 (eager `.bss`).** Every EXTERNAL symbol first, in **reverse
/// > `.gl`** record order; then every STATIC symbol, in **declaration** order.
///
/// Y1 was fitted on extern-only and static-only cells and confirmed
/// out-of-sample by a mixed-linkage cell it predicts exactly and which no
/// simpler rule matches. `.data`'s group is declaration order, which for that
/// section is also ascending address (§5.3).
///
/// `OBJ_DYNINIT_SHAPE.md` §7.1's *"the `.bss` symbols are listed in strictly
/// descending address order in every same-kind cell"* is true only where the
/// shapes coincide and is **false for eager statics**.
pub fn emit_data_obj(obj_name: &str, objects: &[DataObj<'_>]) -> Option<Vec<u8>> {
    // ---- class check. Every `None` below is a case nothing measured. ----
    if objects.is_empty() {
        return None; // `emit_empty_obj` is the right obj for a TU with nothing
    }
    let bss: Vec<&DataObj> = objects.iter().filter(|o| o.bytes.is_none()).collect();
    let data: Vec<&DataObj> = objects.iter().filter(|o| o.bytes.is_some()).collect();
    // **The measured bound.** Above two objects per non-COMDAT section the walk
    // order is open (board #184) and a guess is a wrong `Value` on every symbol.
    if bss.len() > MAX_OBJECTS_PER_SECTION || data.len() > MAX_OBJECTS_PER_SECTION {
        return None;
    }
    for o in objects {
        if o.size == 0 || o.symbol.is_empty() {
            return None;
        }
        // Refuse an alignment with no nibble encoding rather than rounding to
        // something plausible: a wrong nibble is a wrong Characteristics word.
        placement_align(o.size, o.natural_align)?;
        if let Some(b) = o.bytes {
            if b.len() != o.size as usize {
                return None; // a short or over-long initializer is not an object
            }
        }
    }
    // Two objects that share a symbol name would produce two symbol records the
    // linker cannot tell apart, and a duplicate is a decode fault upstream.
    for (i, a) in objects.iter().enumerate() {
        if objects[..i].iter().any(|b| b.symbol == a.symbol) {
            return None;
        }
    }

    // ---- layout ----
    //
    // `.bss` walks the `.gl` record order it was handed. `.data` walks
    // DECLARATION order, which is `decl_index` ascending — a different
    // permutation of the same names, and the whole content of Rule A2.
    let bss_walk: Vec<usize> = (0..bss.len()).collect();
    let mut data_walk: Vec<usize> = (0..data.len()).collect();
    data_walk.sort_by_key(|&i| data[i].decl_index);

    let (bss_offsets, bss_size) = bump_layout(&bss, &bss_walk)?;
    let (data_offsets, data_size) = bump_layout(&data, &data_walk)?;

    // `.data`'s raw bytes, assembled at the offsets the walk produced. The gaps
    // an alignment round-up leaves are **zero**, and they stay in the CheckSum
    // (§4.2.1's cells f8/f9 refute the variant that omits them).
    let mut data_raw = vec![0u8; data_size as usize];
    for (i, o) in data.iter().enumerate() {
        let at = data_offsets[i] as usize;
        data_raw[at..at + o.size as usize].copy_from_slice(o.bytes?);
    }

    // ---- sections, in Rule S1's order ----
    let mut sections = shell_sections(obj_name);
    // The C1 watermark is `sections[3]`; `.bss` takes the slot before it.
    let sec_bss = if bss.is_empty() {
        None
    } else {
        let nibble = section_nibble(&bss)?;
        sections.insert(
            3,
            Section {
                name: ".bss",
                characteristics: CH_BSS_BASE | (nibble << 20),
                raw: std::borrow::Cow::Borrowed(&[]),
                checksum: 0,
                // Never a COMDAT here: `__declspec(selectany)` and `??_R0` are
                // refused upstream by the `.gl` attribute byte, so every object
                // that reaches this writer is ordinary.
                selection: 0,
                assoc: 0,
                uninit_size: Some(bss_size),
            },
        );
        Some(3)
    };
    let sec_data = if data.is_empty() {
        None
    } else {
        let nibble = section_nibble(&data)?;
        sections.push(Section {
            name: ".data",
            characteristics: CH_DATA_BASE | (nibble << 20),
            // **Rule D1**, and it refutes `OBJ_DYNINIT_SHAPE.md` §2.3's "0 for
            // every non-COMDAT section": a non-COMDAT `.data` carries a REAL
            // CRC-32/`0xEDB88320`/init-0/no-final-XOR over its raw bytes.
            // Verified on 9 probe cells and on 9,087 of 9,139 workload sections;
            // the 52 exceptions all contain floating-point initializers, which
            // this writer refuses upstream.
            //
            // Taken from `data_raw` here and moved into the section on the same
            // line, so the checksummed bytes and the emitted bytes cannot come
            // from two buffers.
            checksum: coff_checksum(&data_raw),
            selection: 0,
            assoc: 0,
            raw: std::borrow::Cow::Owned(data_raw),
            uninit_size: None,
        });
        Some(sections.len() - 1)
    };
    let n_sections = sections.len();
    let n_reloc_of = vec![0u16; n_sections];
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    // ---- symbol count ----
    // 11 shell symbols, then per data section: section symbol + aux, then its
    // objects.
    let n_symbols = N_SHELL_SYMBOLS
        + if bss.is_empty() { 0 } else { 2 + bss.len() as u32 }
        + if data.is_empty() { 0 } else { 2 + data.len() as u32 };

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_none() {
            debug_assert_eq!(b.0.len(), ptrs[i]);
        }
        // `.bss` writes nothing at all, whatever its `SizeOfRawData`.
        debug_assert_eq!(s.file_len(), s.raw.len());
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table ----
    //
    // **`emit_shell_symbols` cannot be used here, and the reason is the whole
    // shape of this obj.** That helper writes the eleven shell records
    // contiguously, because in every other emitter the four shell sections *are*
    // contiguous. Here `.bss` is spliced **between** the two `.XBLD$W`
    // watermarks, and the symbol table follows SECTION order — so the `.bss`
    // group sits between `__C2_11886` and the C1 watermark's own records.
    // MEASURED on `char b1;`:
    //
    // ```text
    //   0 @comp.id   1/2 .drectve   3/4 .debug$S   5/6 .XBLD$W(C2)
    //   7 __C2_11886   8/9 .bss   10 ?b1@@3DA   11/12 .XBLD$W(C1)
    //   13 __C1_11886
    // ```
    //
    // Calling the helper wrote `.bss`'s aux record as the C1 watermark's, which
    // the differential caught at file offset 716 — inside a symbol name, five
    // records downstream of the actual error. So the sequence is spelled out.
    let mut strtab = StringTable::new();
    // slot 0: @comp.id (ABS, STATIC, no aux)
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1); // IMAGE_SYM_ABSOLUTE
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);
    emit_section_symbol(&mut b, &sections[0], 1, 0); // .drectve
    emit_section_symbol(&mut b, &sections[1], 2, 0); // .debug$S
    emit_section_symbol(&mut b, &sections[2], 3, 0); // .XBLD$W C2
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000);

    if let Some(si) = sec_bss {
        emit_section_symbol(&mut b, &sections[si], (si + 1) as i16, 0);
        // **Rule Y1** — externals in reverse `.gl` order, then statics in
        // declaration order. Neither block is in address order, and the two use
        // DIFFERENT keys, which is why this is two sorts and not one.
        let mut order: Vec<usize> = (0..bss.len()).filter(|&i| bss[i].external).collect();
        order.reverse();
        let mut statics: Vec<usize> = (0..bss.len()).filter(|&i| !bss[i].external).collect();
        statics.sort_by_key(|&i| bss[i].decl_index);
        order.extend(statics);
        for i in order {
            let o = bss[i];
            emit_symbol(
                &mut b,
                &mut strtab,
                o.symbol,
                bss_offsets[i],
                (si + 1) as i16,
                0x0000,
                if o.external { 2 } else { 3 },
            );
        }
    }
    // The C1 watermark, whose section index MOVED if a `.bss` was spliced in
    // ahead of it: 4 with one, 3 without. Derived from `sec_bss`, never
    // hard-coded — a stale index here is the bug the block above documents.
    let sec_c1 = if sec_bss.is_some() { 4 } else { 3 };
    emit_section_symbol(&mut b, &sections[sec_c1], (sec_c1 + 1) as i16, 0);
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, (sec_c1 + 1) as i16, 0x0000);

    if let Some(si) = sec_data {
        emit_section_symbol(&mut b, &sections[si], (si + 1) as i16, 0);
        // `.data`'s group is declaration order, which here is also ascending
        // address — `data_walk` is that order already.
        for &i in &data_walk {
            let o = data[i];
            emit_symbol(
                &mut b,
                &mut strtab,
                o.symbol,
                data_offsets[i],
                (si + 1) as i16,
                0x0000,
                if o.external { 2 } else { 3 },
            );
        }
    }

    b.bytes(&strtab.finish());
    Some(b.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj<'a>(
        symbol: &'a str,
        size: u32,
        natural_align: u32,
        external: bool,
        bytes: Option<&'a [u8]>,
        decl_index: u32,
    ) -> DataObj<'a> {
        DataObj { symbol, size, natural_align, external, bytes, decl_index }
    }

    /// Read a section header's `(name, SizeOfRawData, PointerToRawData,
    /// Characteristics)` out of an emitted obj, so the assertions below are
    /// about **bytes** and not about this file's own intermediate values.
    fn sections_of(img: &[u8]) -> Vec<(String, u32, u32, u32)> {
        let n = u16::from_le_bytes([img[2], img[3]]) as usize;
        (0..n)
            .map(|i| {
                let h = &img[20 + i * 40..20 + i * 40 + 40];
                let name = String::from_utf8_lossy(&h[..8]).trim_end_matches('\0').to_string();
                let u = |o: usize| u32::from_le_bytes([h[o], h[o + 1], h[o + 2], h[o + 3]]);
                (name, u(16), u(20), u(36))
            })
            .collect()
    }

    /// `(name, Value, SectionNumber, StorageClass)` for every non-aux symbol.
    fn symbols_of(img: &[u8]) -> Vec<(String, u32, i16, u8)> {
        let ptr = u32::from_le_bytes([img[8], img[9], img[10], img[11]]) as usize;
        let n = u32::from_le_bytes([img[12], img[13], img[14], img[15]]) as usize;
        let strtab = ptr + n * 18;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let r = &img[ptr + i * 18..ptr + i * 18 + 18];
            let name = if r[..4] == [0, 0, 0, 0] {
                let off = u32::from_le_bytes([r[4], r[5], r[6], r[7]]) as usize;
                let s = &img[strtab + off..];
                String::from_utf8_lossy(&s[..s.iter().position(|&c| c == 0).unwrap()]).to_string()
            } else {
                String::from_utf8_lossy(&r[..8]).trim_end_matches('\0').to_string()
            };
            out.push((
                name,
                u32::from_le_bytes([r[8], r[9], r[10], r[11]]),
                i16::from_le_bytes([r[12], r[13]]),
                r[16],
            ));
            i += 1 + r[17] as usize;
        }
        out
    }

    /// **Rule S1's section order, which prereg P3 got backwards on every
    /// clause.** The *uninitialized* section comes FIRST and sits **between**
    /// the two `.XBLD$W` watermarks; the *initialized* one comes after the
    /// second. The natural guess — `.data` then `.bss`, both after the shell —
    /// is what a writer built on the prereg would have emitted.
    #[test]
    fn the_uninitialized_section_sits_between_the_watermarks_and_data_follows_them() {
        let d = [1u8, 0, 0, 0];
        let img = emit_data_obj(
            "Z:\\t\\x.obj",
            &[obj("?b1@@3DA", 1, 1, true, None, 1), obj("?d1@@3HA", 4, 4, true, Some(&d), 2)],
        )
        .expect("one object per section is in class");
        let names: Vec<String> = sections_of(&img).into_iter().map(|s| s.0).collect();
        assert_eq!(
            names,
            vec![".drectve", ".debug$S", ".XBLD$W", ".bss", ".XBLD$W", ".data"],
            "Rule S1: uninitialized BETWEEN the watermarks, initialized AFTER the second"
        );
    }

    /// **`.bss` carries its size in `SizeOfRawData` beside a NULL
    /// `PointerToRawData` and contributes zero file bytes.** The natural guess
    /// is the opposite (`SizeOfRawData = 0`, size in `VirtualSize`), and it is
    /// `OBJ_DYNINIT_SHAPE.md` §1's refuted prediction P8.
    #[test]
    fn bss_has_a_size_a_null_pointer_and_no_file_bytes() {
        let img = emit_data_obj("Z:\\t\\x.obj", &[obj("?b1@@3HA", 4, 4, true, None, 1)]).unwrap();
        let bss = sections_of(&img).into_iter().find(|s| s.0 == ".bss").unwrap();
        assert_eq!(bss.1, 4, "SizeOfRawData carries the size");
        assert_eq!(bss.2, 0, "PointerToRawData is NULL");
        assert_eq!(bss.3, 0xC030_0080, "CNT_UNINIT | READ | WRITE | ALIGN_4");
        // VirtualSize is 0 in every section of every reference obj, `.bss` too.
        assert_eq!(u32::from_le_bytes([img[20 + 3 * 40 + 8], img[20 + 3 * 40 + 9], img[20 + 3 * 40 + 10], img[20 + 3 * 40 + 11]]), 0);
    }

    /// **Rule D1** — a non-COMDAT `.data` carries a REAL aux `CheckSum`, which
    /// refutes `OBJ_DYNINIT_SHAPE.md` §2.3's *"0 for every non-COMDAT
    /// section"*. Two known-answer values, transcribed from real objs in
    /// `OBJ_DATA_BSS_SHAPE.md` §4.2's table.
    #[test]
    fn a_non_comdat_data_carries_a_real_crc() {
        for (raw, want) in [
            (vec![0x01u8], 0x7707_3096u32),
            (vec![0x00, 0x00, 0x00, 0x02], 0xEE0E_612Cu32),
            (vec![0x00, 0x00, 0x00, 0x03], 0x9909_51BAu32),
        ] {
            let img = emit_data_obj(
                "Z:\\t\\x.obj",
                &[obj("?d@@3HA", raw.len() as u32, raw.len().min(4) as u32, true, Some(&raw), 1)],
            )
            .unwrap();
            // The aux record follows the `.data` section symbol; CheckSum is at
            // aux offset 8.
            let ptr = u32::from_le_bytes([img[8], img[9], img[10], img[11]]) as usize;
            let n = u32::from_le_bytes([img[12], img[13], img[14], img[15]]) as usize;
            let mut crc = None;
            let mut i = 0;
            while i < n {
                let r = &img[ptr + i * 18..ptr + i * 18 + 18];
                if &r[..5] == b".data" && r[17] == 1 {
                    let a = &img[ptr + (i + 1) * 18..ptr + (i + 1) * 18 + 18];
                    crc = Some(u32::from_le_bytes([a[8], a[9], a[10], a[11]]));
                }
                i += 1 + r[17] as usize;
            }
            assert_eq!(crc, Some(want), "raw {raw:02x?}");
        }
    }

    /// **Rule Y1** — the eager `.bss` symbol table is every EXTERNAL first in
    /// **reverse `.gl`** order, then every STATIC in **declaration** order. It
    /// is neither ascending nor descending address, and a mixed-linkage section
    /// is the only cell that can show it. This is the case
    /// `OBJ_DYNINIT_SHAPE.md` §7.1's *"strictly descending address order"* is
    /// false for.
    #[test]
    fn the_bss_symbol_order_is_externals_reversed_then_statics_in_declaration_order() {
        // `.gl` order p1, s1 (index order); declaration order s1, p1.
        let img = emit_data_obj(
            "Z:\\t\\x.obj",
            &[obj("?p1@@3HA", 4, 4, true, None, 20), obj("s1", 4, 4, false, None, 10)],
        )
        .unwrap();
        let syms: Vec<String> = symbols_of(&img)
            .into_iter()
            .filter(|s| s.0 == "?p1@@3HA" || s.0 == "s1")
            .map(|s| s.0)
            .collect();
        assert_eq!(syms, vec!["?p1@@3HA", "s1"], "externals first, then statics");

        // …and the addresses are the `.gl` WALK, which puts p1 at 0 and s1 at 4
        // — so the symbol order above is not the address order, which is the
        // whole point of Rule Y1.
        let vals: Vec<(String, u32, u8)> = symbols_of(&img)
            .into_iter()
            .filter(|s| s.0 == "?p1@@3HA" || s.0 == "s1")
            .map(|s| (s.0, s.1, s.3))
            .collect();
        assert_eq!(vals[0], ("?p1@@3HA".to_string(), 0, 2), "EXTERNAL, at the walk's first slot");
        assert_eq!(vals[1], ("s1".to_string(), 4, 3), "STATIC storage class 3");
    }

    /// **Rule A2 — `.data` walks DECLARATION order, not `.gl` order**, and the
    /// two are different permutations of the same names. MEASURED: `int d1=1;
    /// int d2=2;` has `.gl` order `d2 d1` and `.data` addresses `d1@0 d2@4`.
    ///
    /// A writer that used the `.gl` order for both sections would place these
    /// backwards; §5.7 scores that mistake at 19 of 68 real `.data` sections
    /// against Rule A2's 46.
    #[test]
    fn data_walks_declaration_order_where_bss_walks_gl_order() {
        let one = [0u8, 0, 0, 1];
        let two = [0u8, 0, 0, 2];
        // Input is `.gl` order: d2 first. Declaration order is d1 then d2.
        let img = emit_data_obj(
            "Z:\\t\\x.obj",
            &[
                obj("?d2@@3HA", 4, 4, true, Some(&two), 0x9e4),
                obj("?d1@@3HA", 4, 4, true, Some(&one), 0x9e3),
            ],
        )
        .unwrap();
        let syms = symbols_of(&img);
        let at = |n: &str| syms.iter().find(|s| s.0 == n).unwrap().1;
        assert_eq!(at("?d1@@3HA"), 0, "declared first, so placed first");
        assert_eq!(at("?d2@@3HA"), 4);
        // The raw bytes follow the addresses, not the input order.
        let data = sections_of(&img).into_iter().find(|s| s.0 == ".data").unwrap();
        let off = data.2 as usize;
        assert_eq!(&img[off..off + 8], &[0, 0, 0, 1, 0, 0, 0, 2]);

        // The SAME two objects in a `.bss` walk the input (`.gl`) order instead,
        // so d2 lands first. One input, two walks — this is the assertion that
        // says the two rules are genuinely different.
        let img = emit_data_obj(
            "Z:\\t\\x.obj",
            &[obj("?d2@@3HA", 4, 4, true, None, 0x9e4), obj("?d1@@3HA", 4, 4, true, None, 0x9e3)],
        )
        .unwrap();
        let syms = symbols_of(&img);
        let at = |n: &str| syms.iter().find(|s| s.0 == n).unwrap().1;
        assert_eq!(at("?d2@@3HA"), 0, ".bss walks `.gl` order, so d2 is first");
        assert_eq!(at("?d1@@3HA"), 4);
    }

    /// **Rule A3′ — a plain bump with NO free list**, and Rule B1's nibble is
    /// the max over the objects' *size-promoted* alignments.
    ///
    /// `char` (1 B, align 1) then `double` (8 B, align 8) bumps 0 → 1, rounds to
    /// 8, and ends at 16. The refuted §5.4 allocator would reuse the `[1,8)`
    /// hole for a later small object; nothing here does.
    #[test]
    fn the_allocator_is_a_plain_bump_and_the_nibble_is_the_max() {
        let img = emit_data_obj(
            "Z:\\t\\x.obj",
            &[obj("?c@@3DA", 1, 1, true, None, 1), obj("?d@@3NA", 8, 8, true, None, 2)],
        )
        .unwrap();
        let syms = symbols_of(&img);
        let at = |n: &str| syms.iter().find(|s| s.0 == n).unwrap().1;
        assert_eq!(at("?c@@3DA"), 0);
        assert_eq!(at("?d@@3NA"), 8, "rounded up from 1, and the gap is NOT reused");
        let bss = sections_of(&img).into_iter().find(|s| s.0 == ".bss").unwrap();
        assert_eq!(bss.1, 16, "final cursor");
        assert_eq!(bss.3 >> 20 & 0xf, 4, "ALIGN_8 — the max over the objects");
    }

    /// The size-promotion thresholds, on both sides of each step: a `char` is
    /// ALIGN_1 and anything from 2 bytes up is ALIGN_4 until 64, which is
    /// ALIGN_8. These are the steps `align = max(t, 1 if n<2 else 4 if n<64
    /// else 8)` makes, and they are a property of the OBJECT, not the section.
    #[test]
    fn the_promotion_thresholds_are_at_2_and_64() {
        for (n, want_nibble) in [(1u32, 1u32), (2, 3), (63, 3), (64, 4), (200, 4)] {
            let img =
                emit_data_obj("Z:\\t\\x.obj", &[obj("?a@@3DA", n, 1, true, None, 1)]).unwrap();
            let bss = sections_of(&img).into_iter().find(|s| s.0 == ".bss").unwrap();
            assert_eq!(bss.3 >> 20 & 0xf, want_nibble, "n = {n}");
            assert_eq!(bss.1, n);
        }
    }

    /// **The class bound is `MAX_OBJECTS_PER_SECTION`, and it is measured.**
    /// Three objects in one non-COMDAT section is 38-of-62 territory
    /// (`OBJ_DATA_BSS_SHAPE.md` §8.1) and the residual is walk order, board
    /// #184 — so it refuses rather than guessing. Two is admitted, and the
    /// bound is per SECTION, so two `.bss` plus two `.data` is in class.
    #[test]
    fn more_than_two_objects_in_one_section_refuses_but_two_of_each_does_not() {
        let mk = |n: usize, init: bool| -> Vec<DataObj<'static>> {
            const NAMES: [&str; 3] = ["?a@@3HA", "?b@@3HA", "?c@@3HA"];
            const RAW: [u8; 4] = [0, 0, 0, 1];
            (0..n)
                .map(|i| obj(NAMES[i], 4, 4, true, init.then_some(&RAW[..]), i as u32))
                .collect()
        };
        assert!(emit_data_obj("Z:\\t\\x.obj", &mk(2, false)).is_some(), "two `.bss`");
        assert!(emit_data_obj("Z:\\t\\x.obj", &mk(3, false)).is_none(), "three `.bss`");
        assert!(emit_data_obj("Z:\\t\\x.obj", &mk(3, true)).is_none(), "three `.data`");

        // Two of EACH is four objects and is in class — the bound counts per
        // section, and a bound that counted the whole TU would refuse this.
        const RAW: [u8; 4] = [0, 0, 0, 1];
        let both = [
            obj("?a@@3HA", 4, 4, true, None, 0),
            obj("?b@@3HA", 4, 4, true, None, 1),
            obj("?c@@3HA", 4, 4, true, Some(&RAW), 2),
            obj("?d@@3HA", 4, 4, true, Some(&RAW), 3),
        ];
        assert!(emit_data_obj("Z:\\t\\x.obj", &both).is_some(), "two per section, four total");
    }

    /// Out-of-class inputs refuse rather than being approximated: a zero-length
    /// object, an over-aligned one (no nibble encoding above ALIGN_8), an
    /// initializer whose length disagrees with the object's size, a duplicate
    /// symbol, and the empty list.
    #[test]
    fn out_of_class_inputs_refuse() {
        const RAW: [u8; 4] = [0, 0, 0, 1];
        assert!(emit_data_obj("Z:\\t\\x.obj", &[]).is_none(), "empty — emit_empty_obj's job");
        assert!(
            emit_data_obj("Z:\\t\\x.obj", &[obj("?a@@3HA", 0, 1, true, None, 0)]).is_none(),
            "zero-length"
        );
        assert!(
            emit_data_obj("Z:\\t\\x.obj", &[obj("?a@@3HA", 4, 16, true, None, 0)]).is_none(),
            "ALIGN_16 has no nibble here"
        );
        assert!(
            emit_data_obj("Z:\\t\\x.obj", &[obj("?a@@3HA", 8, 4, true, Some(&RAW), 0)]).is_none(),
            "a 4-byte initializer for an 8-byte object"
        );
        assert!(
            emit_data_obj(
                "Z:\\t\\x.obj",
                &[obj("?a@@3HA", 4, 4, true, None, 0), obj("?a@@3HA", 4, 4, true, None, 1)]
            )
            .is_none(),
            "duplicate symbol"
        );
    }

    /// A `.bss`-only TU has five sections and a `.data`-only TU has five too,
    /// but the `.data` one puts its section LAST while the `.bss` one puts it
    /// fourth. The C1 watermark's own section number moves with it, which is
    /// the arithmetic that produced a wrong obj five symbol records downstream
    /// before it was derived instead of hard-coded.
    #[test]
    fn the_c1_watermarks_section_number_moves_with_the_bss() {
        let raw = [0u8, 0, 0, 1];
        for (objs, want_c1) in [
            (vec![obj("?a@@3HA", 4, 4, true, None, 0)], 5i16),
            (vec![obj("?a@@3HA", 4, 4, true, Some(&raw), 0)], 4i16),
        ] {
            let img = emit_data_obj("Z:\\t\\x.obj", &objs).unwrap();
            let syms = symbols_of(&img);
            let c1 = syms.iter().find(|s| s.0 == "__C1_11886").unwrap();
            assert_eq!(c1.2, want_c1, "the C1 watermark's SectionNumber");
        }
    }
}
