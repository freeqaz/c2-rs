//! The two ordinary-function obj writers: packed (`emit_obj`) and per-function
//! COMDAT (`emit_comdat_obj`).

use super::*;

/// `.text` COMDAT selection: `IMAGE_COMDAT_SELECT_NODUPLICATES`.
/// PROV[S] PE/COFF spec — `IMAGE_COMDAT_SELECT_NODUPLICATES` is 1. Which selection c2 chooses for a `/Gy` `.text` is `[O]`; the value 1 is not from c2.
pub(crate) const COMDAT_SELECT_NODUPLICATES: u8 = 1;

/// `.text` characteristics under **function-level linking** (`/Gy`): the packed
/// [`CH_TEXT`] plus `IMAGE_SCN_LNK_COMDAT` (0x1000).
/// PROV[O] transcribed from real `/Gy` objs.
pub(crate) const CH_TEXT_COMDAT: u32 = 0x6040_1020;

/// How many relocation records one function's `.text` carries.
///
/// **One locator.** The count appears in three places that must agree — the
/// section header's `NumberOfRelocations`, the section symbol's aux record, and
/// the `debug_assert` against [`crate::comdat::text_reloc_plan`]'s own length —
/// and `writer.rs`'s own history has the aux reading 0 while the header read 1,
/// on an obj that was otherwise byte-identical.
fn text_reloc_count(f: &Function) -> u16 {
    let fanout: usize = f.data_defs.iter().map(|d| 2 + 2 * d.lo_offs.len()).sum();
    // **W-BIQUAD** — a pooled FP constant reference is the same REFHI/PAIR +
    // REFLO/PAIR quad a `DataRef` is.
    (f.calls.len() + 4 * f.data_refs.len() + 4 * f.fp_refs.len() + fanout) as u16
}

/// Build the complete `.obj` image with **one COMDAT `.text` section per
/// function** — the shape c2 emits under function-level linking (`/Gy`, which
/// `/O1` and `/O2` imply).
///
/// This is not a variant spelling of [`emit_obj`]; it is a different obj:
///
/// | | packed (`/Ox`) | COMDAT (`/Gy`) |
/// |---|---|---|
/// | `.text` sections | 1 | one per function |
/// | characteristics | `0x60400020` | `0x60401020` |
/// | aux `Selection` | 0 | 1 (NODUPLICATES) |
/// | function `Value` | its offset in `.text` | always 0 |
/// | inter-function padding | 8-byte aligned | none — each has its own section |
/// | symbol count | 13 + 1/fn (+callees) | 11 + 3/fn (+callees) |
///
/// So the same IL yields two legitimately different objs depending on an argv
/// flag the bundle does not record. Verified against `system/utl/Spew.cpp`
/// compiled with the dc3 workload's real flags: 6 sections, 17 symbols, two
/// 4-byte `.text` sections each holding a single `blr`, laid out contiguously
/// with no padding between them.
///
/// `texts[i]` is function `i`'s own `.text` bytes; each function's
/// `text_offset` is ignored (it is 0 within its own section) and any
/// `call.reloc_offset` is relative to that function's section.
///
/// A **framed** function additionally gets its own `.pdata` COMDAT, emitted
/// immediately after its `.text` COMDAT and tied to it by
/// `IMAGE_COMDAT_SELECT_ASSOCIATIVE` with the aux `Number` field naming that
/// `.text`'s section number — so the linker drops a function's unwind record
/// with the function. `label_counter` is the `.gl` seed
/// ([`c2_il::label_counter`]); it is unused when no function is framed, and a
/// caller with a framed function and no counter must refuse rather than guess.
/// **W-WORDWRAP2 — `bss` is the TU's shared non-COMDAT `.bss`**, board #2727's
/// shell slot, in `c2_il::IlBundle::bss_shell_objects`' `.gl` record order
/// (Rule A1's walk). Empty for every obj this writer emitted before that field
/// existed; when it is non-empty every `DataDef` marked `uninitialized` on any
/// function must name one of these objects, and each such def contributes only
/// its REFHI/PAIR/REFLO/PAIR quad — the section and the symbol belong to the
/// TU, not to the referring function.
pub fn emit_comdat_obj(
    obj_name: &str,
    funcs: &[Function],
    texts: &[Vec<u8>],
    label_counter: u32,
    bss: &[crate::coff::DataObj<'_>],
) -> Option<Vec<u8>> {
    assert_eq!(funcs.len(), texts.len(), "one text per function");
    // **W-DATA — the defined-data class check.**
    //
    // # The fence that was here, and the measurement that replaced it
    //
    // This shipped as *"at most ONE defined object over the whole obj"*, on the
    // honest ground that every rule about a COMDAT `.data` — its slot, its
    // alignment nibble, its aux CheckSum, the position of its symbol group and
    // the index its relocations resolve against — had been read off **one** obj,
    // `src/system/math/Primes.cpp`'s. At n = 1 *"grouped after the code"* and
    // *"interleaved with the code"* are the same obj, so nothing separated them.
    //
    // GRID C separated them, and **the writer was wrong**. Its two rivals were
    // frozen in `work/w-data/GRID.md` before the cell compiled, and the
    // three-function cell says **INTERLEAVED**:
    //
    // ```text
    //   .drectve .debug$S .XBLD$W .XBLD$W
    //   .text(p0) .data(p0)  .text(p1) .data(p1)  .text(p2) .data(p2)
    //
    //   … 11/12 .text 13 ?p0 14/15 .data 16 ?a
    //     17/18 .text 19 ?p1 20/21 .data 22 ?b
    //     23/24 .text 25 ?p2 26/27 .data 28 ?table
    // ```
    //
    // The symbol table follows section order and therefore interleaves too, so
    // the grouped reading is wrong about the section table, the section
    // indices, the symbol indices and every relocation's `SymbolTableIndex` at
    // once. The count (29 symbols) was right either way, which is why the cell
    // had to be read by ORDER and not by size. The refusal is what kept the
    // grouped reading from ever reaching an obj.
    //
    // # What still refuses, and why each one has no cell
    //
    // * **more than one object per FUNCTION.** `undname.cpp` and `osfinfo.cpp`
    //   need two data symbols in one *body*, which is a different question from
    //   two objects in one obj and has no cell here;
    // * **a defined object on a FRAMED function.** Where its `.data` sits
    //   relative to that function's `.pdata` COMDAT — which is associative and
    //   tied back to the `.text` — is unmeasured, and so is where its symbol
    //   group sits among the `$M`/`$M`/`$T` triple;
    // * **a defined object on a FLOAT function**, for the same reason one door
    //   along: `_fltused` goes after the first float function's *complete*
    //   group, and whether the data group is inside that "complete" is a cell
    //   nobody has cut.
    //
    // # W-WORDWRAP2 — the SHARED `.bss` is a different question and gets its own
    //
    // Everything above is about a COMDAT `.data` a single function OWNS. A
    // non-COMDAT `.bss` is owned by the TU: `wordwrap.cpp`'s 588-byte one holds
    // two objects shared by three functions. So its class check is here, over
    // the whole obj rather than per function, and the per-function loop below
    // only has to know that an `uninitialized` def emits relocations and no
    // section of its own.
    //
    // The LAYOUT bound is board #184's, quoted from the sibling writer rather
    // than restated: above [`super::data::MAX_OBJECTS_PER_SECTION`] objects the
    // walk order is open and a guess is a wrong `Value` on every symbol.
    if bss.len() > super::data::MAX_OBJECTS_PER_SECTION {
        return None;
    }
    for o in bss {
        // `.bss` means no bytes at all; an `external` one is Rule S1′'s slot `B`
        // and an internal one is slot `C`, which nothing places (cell `p5`).
        // Both are the reader's clauses, re-asserted here because neither crate
        // may assume the other ran.
        if o.bytes.is_some() || !o.external || o.size == 0 || !o.relocs.is_empty() {
            return None;
        }
    }
    // **Every `uninitialized` def must name one of them, and this test is
    // UNCONDITIONAL.** It read `if !bss.is_empty() { … }` for one commit and
    // that is the same hole `IlBundle::bss_shell_objects`' own doc records: an
    // EMPTY `bss` beside a function that carries an uninitialized def is
    // precisely the dangling case, and gating the check on `bss` being
    // non-empty is asking the broken input to report itself. It panicked —
    // `every relocation target got a symbol`, in the `/O1` fixture lane.
    //
    // Two crates, two independent statements of one rule, on purpose: this one
    // fires whatever the reader did.
    for f in funcs {
        for d in &f.data_defs {
            if d.uninitialized && !bss.iter().any(|o| o.symbol == d.symbol) {
                return None;
            }
        }
    }
    for f in funcs {
        // The COMDAT-`.data` clauses below count only the defs that ask for a
        // `.data` section. A shared-`.bss` reference asks for none.
        let owned = f.data_defs.iter().filter(|d| !d.uninitialized).count();
        if owned > 1 {
            return None;
        }
        if owned > 0 && (f.frame.is_some() || f.is_float) {
            return None;
        }
        // **The FRAMED and FLOAT refusals stay live for a shared `.bss` too**,
        // and that is a measurement this lane did NOT make rather than a
        // conservatism it chose. Cell `p4` shows real c2 emitting
        // `.bss · .XBLD$W · .text · .pdata` for a framed function that stores to
        // one, so the SECTION order is known; what is not known is where the
        // object's symbol group sits among that function's `$M`/`$M`/`$T`
        // triple, because `p4`'s `.bss` group is in the shell and its label
        // group is in the code region and no cell puts a second data object
        // between them. `?WordWrap_CanBreakLineAt` is exactly this shape, so
        // this clause is one of the two that keep `wordwrap.cpp` refused after
        // the shell placement is paid (the other is `lib.rs`'s two-`lis` fence).
        if !f.data_defs.is_empty() && (f.frame.is_some() || f.is_float) {
            return None;
        }
        // **W-BIQUAD — a FRAMED function that also introduces a pooled
        // constant.** Where its `.rdata` COMDATs sit relative to its own
        // `.pdata` COMDAT — which is associative and tied back to the `.text` —
        // and where their symbol pairs sit among the `$M`/`$M`/`$T` triple are
        // both unmeasured: `docs/OBJ_GY_SHAPES.md` §2 captures pools only on
        // leaf functions and §3 captures the `.pdata` group only without them.
        // `Biquad.cpp` is the leaf-pools-plus-framed-caller shape, which needs
        // neither answer. Refused rather than ordered on a guess, which is a
        // wrong section count at file offset 2.
        if !f.fp_refs.is_empty() && f.frame.is_some() {
            return None;
        }
        for d in &f.data_defs {
            // **W-WORDWRAP2 — an UNINITIALIZED object is now PLACED, in the
            // shell, and its class check ran above over the whole obj.**
            //
            // This arm read `return None` from `w-wordwrap` (board #2722) until
            // this lane, on the honest ground that no cell had graded a
            // non-COMDAT `.bss` on a function-bearing TU. GRID B graded nine,
            // and the finding is that every LAYOUT rule the slot needs was
            // already shipped in `coff::data` by three lanes serving the
            // FUNCTIONLESS TU — S1′, A1, A3′, B1 and Y1's external clause. What
            // was missing was the composition, which is what this file now does.
            //
            // The one clause that still fires is `lo_offs`: a def with a high
            // half and no low half has no relocation quad to emit.
            if d.uninitialized {
                if d.lo_offs.is_empty() {
                    return None;
                }
                continue;
            }
            if d.bytes.len() as usize != d.size as usize || d.size == 0 || d.lo_offs.is_empty() {
                return None;
            }
            // A nibble this container cannot spell is a wrong Characteristics
            // word, which is the refusal `placement_align` exists for.
            align_nibble(d.size, d.natural_align)?;
        }
    }
    let labels = plan_labels(label_counter, funcs, true);
    // Per-function `.pdata` raw, built up front so the sections can borrow it.
    let pdata_raw: Vec<Option<[u8; 8]>> =
        funcs.iter().map(|f| f.frame.as_ref().map(|fr| pdata_record(0, fr))).collect();

    let mut sections: Vec<Section> = shell_sections(obj_name);
    // **W-WORDWRAP2 — the shared `.bss`, spliced into the shell at Rule S1′'s
    // slot `B`: index 3, BETWEEN the two `.XBLD$W` watermarks and before every
    // code group.** Eight of GRID B's nine cells put it there, including the
    // framed one (`p4`) and the one that also has a COMDAT-free `.data` (`p6`);
    // the ninth (`p5`) is the internal-linkage object that takes slot `C`, and
    // the reader refuses it.
    //
    // The walk and the allocator are `coff::data`'s own, CALLED and not copied
    // (board #1120's rule: `section_nibble` was already a second copy of
    // `align_nibble`'s table once, and a lane edited one of the two).
    let bss_refs: Vec<&DataObj> = bss.iter().collect();
    let bss_walk: Vec<usize> = (0..bss.len()).collect();
    let (bss_offsets, bss_size) = if bss.is_empty() {
        (Vec::new(), 0)
    } else {
        super::data::bump_layout(&bss_refs, &bss_walk)?
    };
    let sec_bss: Option<usize> = if bss.is_empty() {
        None
    } else {
        let nibble = super::data::section_nibble(&bss_refs)?;
        sections.insert(
            3,
            Section {
                name: ".bss",
                characteristics: CH_BSS_BASE | (nibble << 20),
                raw: std::borrow::Cow::Borrowed(&[]),
                checksum: 0,
                selection: 0,
                assoc: 0,
                uninit_size: Some(bss_size),
            },
        );
        Some(3)
    };
    // Rule Y1's EXTERNAL clause — the symbol group is the REVERSE of the `.gl`
    // record order the storage walk above uses. Cells `p2` (`g2 g1`), `p7`
    // (`g2 g1 g3`), `p8` and `p9` each separate it from ascending address, from
    // descending address and from declaration order, and `wordwrap.obj` itself
    // is `p9`'s permutation.
    let bss_symbol_order: Vec<usize> = (0..bss.len()).rev().collect();
    // Index of each shared object's symbol, derived from
    // `emit_shell_symbols_bss_slot_b`'s own sequence and asserted where the
    // records go out.
    let mut bss_sym: Vec<(&str, u32)> = Vec::with_capacity(bss.len());
    for (slot, &i) in bss_symbol_order.iter().enumerate() {
        bss_sym.push((bss[i].symbol, FIRST_BSS_SYMBOL_SLOT_B + slot as u32));
    }
    // Per function: its `.text` COMDAT, then — if it is framed — its `.pdata`
    // COMDAT immediately after, tied back with SELECT_ASSOCIATIVE. `sec_text[i]`
    // / `sec_pdata[i]` are 0-based indices into `sections`.
    let mut sec_text: Vec<usize> = Vec::with_capacity(funcs.len());
    let mut sec_pdata: Vec<Option<usize>> = Vec::with_capacity(funcs.len());
    // **W-DATA** — function `i`'s own COMDAT `.data`, immediately after its
    // `.text`. `None` for a function that defines no object, which is every
    // function the port emitted before this field existed.
    let mut sec_data: Vec<Option<usize>> = vec![None; funcs.len()];
    // **W-BIQUAD — the TU-wide constant pool, and the order it is built in.**
    //
    // `docs/OBJ_GY_SHAPES.md` §2.4, three rules, and the third is the one a
    // straight port gets wrong:
    //
    //  1. INTERLEAVED — a constant's `.rdata` COMDAT (and its symbol pair) is
    //     emitted immediately after the `.text` COMDAT of the function that
    //     FIRST references it, not grouped at the end;
    //  2. one section per distinct `(bit pattern, width)` TU-wide, with later
    //     functions relocating against the existing symbol index;
    //  3. within ONE introducing function, several new constants are appended
    //     in **reverse first-reference order** (LIFO). §2.3's three-constant
    //     cell separates that from descending bit-pattern order, which the
    //     two-constant cells alone do not; this lane re-confirmed it on
    //     `work/w-biquad/probe/pool{1,2}.cpp`, whose two constants are the same
    //     pair in both use orders, and on `Biquad.cpp` itself.
    //
    // `pool_of[i]` is the list of pool indices function `i` introduces, already
    // reversed, so the section loop and the symbol loop below walk one order.
    let mut pool: Vec<(u64, bool)> = Vec::new();
    let mut pool_of: Vec<Vec<usize>> = Vec::with_capacity(funcs.len());
    for f in funcs {
        let mut here: Vec<usize> = Vec::new();
        for r in &f.fp_refs {
            let key = (r.bits, r.double);
            // Rule 2's dedup is TU-wide and therefore also covers a constant
            // this same function references twice.
            if pool.contains(&key) {
                continue;
            }
            pool.push(key);
            here.push(pool.len() - 1);
        }
        // Rule 3 — LIFO within the introducing function. Reversing the INDEX
        // list rather than the pool itself keeps every already-assigned index
        // stable, which is what rule 2's cross-function dedup relies on.
        here.reverse();
        pool_of.push(here);
    }
    let pool_ix = |bits: u64, double: bool| -> Option<usize> {
        pool.iter().position(|&k| k == (bits, double))
    };
    // `.rdata` section index per pool entry, filled by the section loop.
    let mut sec_pool: Vec<Option<usize>> = vec![None; pool.len()];
    // The inverse map, so the layout and relocation passes below index rather
    // than search: section -> the function it belongs to, and which of its two
    // sections it is. `SectionOwner::None` for the fixed prefix.
    let mut owner: Vec<SectionOwner> = vec![SectionOwner::Fixed; sections.len()];
    for (i, t) in texts.iter().enumerate() {
        sec_text.push(sections.len());
        owner.push(SectionOwner::Text(i));
        sections.push(Section {
            name: ".text",
            characteristics: CH_TEXT_COMDAT,
            raw: std::borrow::Cow::Borrowed(t.as_slice()),
            checksum: 0,
            selection: COMDAT_SELECT_NODUPLICATES,
            assoc: 0,
            uninit_size: None,
        });
        match &pdata_raw[i] {
            None => sec_pdata.push(None),
            Some(rec) => {
                let text_sec_num = (sec_text[i] + 1) as u16;
                sec_pdata.push(Some(sections.len()));
                owner.push(SectionOwner::Pdata(i));
                sections.push(Section {
                    name: ".pdata",
                    characteristics: CH_PDATA_COMDAT,
                    raw: std::borrow::Cow::Borrowed(&rec[..]),
                    // `.pdata` is the one COMDAT c2 gives a real CheckSum —
                    // `.text` and the `.rdata` constant pools carry 0.
                    checksum: coff_checksum(&rec[..]),
                    selection: COMDAT_SELECT_ASSOCIATIVE,
                    assoc: text_sec_num,
                    uninit_size: None,
                });
            }
        }
        // **W-DATA — this function's COMDAT `.data`, immediately after its own
        // `.text`.** INTERLEAVED, not grouped at the end: GRID C's
        // three-function cell reads `.text .data .text .data .text .data` in the
        // section table and the same interleave in the symbol table. The
        // grouped reading was this writer's first draft and the class check
        // above is where it was caught.
        //
        // A framed function is refused upstream, so this never has to decide
        // whether it goes before or after a `.pdata`.
        for d in &funcs[i].data_defs {
            // A shared-`.bss` reference has no section of its own — the TU's is
            // already spliced into the shell above.
            if d.uninitialized {
                continue;
            }
            let nibble = align_nibble(d.size, d.natural_align)?;
            sec_data[i] = Some(sections.len());
            owner.push(SectionOwner::Data(i));
            sections.push(Section {
                name: ".data",
                // `CH_DATA_BASE | LNK_COMDAT | nibble<<20`. Read off c2's own
                // objs at three sizes: `0xC0401040` for `Primes.cpp`'s 248-byte
                // `int[62]` and for GRID C's `p1` (256 B) — both ALIGN_8,
                // because `placement_align` promotes anything ≥ 64 bytes — and
                // `0xC0301040` for `p0`/`p2`'s 32-byte arrays (ALIGN_4). One
                // promotion table, already shared with the `.bss` allocator, and
                // `p1` is the cell that crosses the boundary.
                characteristics: CH_DATA_BASE | 0x1000 | (nibble << 20),
                // **A COMDAT `.data` carries a REAL aux CheckSum**, unlike the
                // `.text` and `.rdata` COMDATs beside it, which carry 0.
                // Verified against c2's own objs on four distinct payloads
                // (`0x25B5A181`, `0xFC84F8C5`, `0x52892C86`, `0x2AFF742F`).
                // This was the lane's least-confident prediction (PREREG P8)
                // precisely because "COMDAT ⇒ CheckSum 0" holds for every other
                // COMDAT this port emits.
                checksum: coff_checksum(d.bytes),
                // SELECT_ANY, which is what a function-local `static`'s section
                // carries in all six of lane w-cfg2's GRID A cells and all three
                // of GRID C's. **Not** ASSOCIATIVE — nothing ties it to the
                // `.text` the way a `.pdata` is tied, which is why its placement
                // had to be measured rather than inherited.
                selection: 2,
                assoc: 0,
                raw: std::borrow::Cow::Borrowed(d.bytes),
                uninit_size: None,
            });
        }
        // **W-BIQUAD — the `.rdata` pools this function introduces**, in the
        // LIFO order `pool_of` already fixed, immediately after its `.text`
        // (rule 1). A framed function that introduces one is refused by the
        // class check above, so this never has to decide whether it goes before
        // or after a `.pdata` — the same sentence the `.data` loop above carries
        // and for the same reason.
        for &k in &pool_of[i] {
            let (bits, double) = pool[k];
            sec_pool[k] = Some(sections.len());
            owner.push(SectionOwner::Rdata(k));
            sections.push(Section {
                name: ".rdata",
                characteristics: if double { CH_RDATA_F64 } else { CH_RDATA_F32 },
                raw: std::borrow::Cow::Owned(real_raw_bytes(bits, double)),
                // 0, not a real one: a `.rdata` constant pool is the COMDAT c2
                // leaves the aux CheckSum at zero, unlike the `.data` above and
                // the `.pdata` beside it (`docs/OBJ_GY_SHAPES.md` §2.4 rule 4).
                checksum: 0,
                selection: 2,
                assoc: 0,
                uninit_size: None,
            });
        }
    }
    let n_sections = sections.len();

    // Raw data is packed contiguously after the section headers — including
    // between the per-function `.text` sections, which carry no padding —
    // **except** that a section's relocations immediately follow *its own* raw
    // data, before the next section's:
    //
    //   .text[0] raw @696 ; .text[0] reloc @700
    //   .text[1] raw @710 ; .text[1] reloc @714 ; …
    //
    // Not all raw data followed by all relocations. This emitter did the latter,
    // which is only invisible when at most one section has relocations — and under
    // `/Gy` every calling function's COMDAT `.text` has one, so the port's whole
    // section table carried wrong `PointerToRelocations` values from the fifth
    // header on (`il_call_value.cpp`, divergence at obj offset 204).
    //
    // Precisely the bug already fixed in [`emit_obj`] for the packed layout, where
    // `.text` being last hid it. Two emitters, one wrong assumption, and the second
    // one stayed wrong because no lane compiled a multi-call fixture with `/Gy`
    // until `scripts/mode_lane.sh`.
    //
    // A framed function's `.pdata` has exactly one relocation of its own (the
    // ADDR32 on `BeginAddress`), so it follows the same rule.
    let n_reloc_of: Vec<u16> = owner
        .iter()
        .map(|o| match o {
            // WR1: each data-symbol reference adds a REFHI/PAIR/REFLO/PAIR quad.
            SectionOwner::Text(k) => text_reloc_count(&funcs[*k]),
            SectionOwner::Pdata(_) => 1,
            // The relocations that name a defined object live in the referring
            // function's `.text`; the object's own section is pure data.
            SectionOwner::Data(_) | SectionOwner::Rdata(_) | SectionOwner::Fixed => 0,
        })
        .collect();
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    // Symbols: the fixed 11-slot prefix, then per function a `.text` section
    // symbol (+aux), the defined FUNCTION symbol, and its callee if any.
    // An undefined external callee is emitted **once per distinct name**, after the
    // symbol of the function that first calls it; every later call site relocates
    // against that same index. This path emitted one per *call site* instead, which
    // is invisible until a TU has two functions calling the same callee under `/Gy`
    // — `il_call_perm.cpp` has six calling `?g3`, and the port's symbol table came
    // out five symbols long (obj offset 12, `NumberOfSymbols`). The packed emitter
    // had already been fixed for exactly this; `emit_comdat_obj` had not, and no lane
    // compiled the call fixtures with `/Gy` until `scripts/mode_lane.sh`.
    //
    // A framed function's group is longer, and the order inside it is the
    // reference's, not an obvious one — the END label comes before the callee and
    // the PROLOGUE label after it:
    //
    //   [.text sym + aux] [fn] [$M(n+1) @ function end] [callee, if new]
    //   [$M(n) @ prologue end] [.pdata sym + aux] [$T(n+2) @ 0]
    //
    // `_fltused` goes immediately after the **first** float function's complete
    // group — its section symbol + aux, its function symbol, and any callee external
    // it introduced — and before the next function's section symbol. That is the
    // same rule as the packed layout; `/Gy` does not move it (`docs/OBJ_GY_SHAPES.md`
    // §1, six orderings captured: float-first, int-first, float-int-float,
    // int-int-float, and a float function whose callee external precedes the marker).
    // Omitting it entirely is what left `mvp_fmul3.cpp` one symbol short of the
    // reference under `/Gy`.
    let fltused_after = funcs.iter().position(|f| f.is_float);
    // **W-WORDWRAP2** — with a shared `.bss` in the shell the prefix is longer
    // by its section symbol, its aux and one record per object, and every index
    // below shifts with it. Derived from `emit_shell_symbols_bss_slot_b`'s own
    // sequence, never counted twice.
    let mut next_idx: u32 = if bss.is_empty() {
        N_SHELL_SYMBOLS
    } else {
        n_shell_symbols_bss(bss.len())
    };
    // The callee symbols this function emits, in emission order (reverse
    // first-reference), each with the index it lands at.
    // Per function, the undefined externals it introduces, in EMISSION order —
    // one merged list, `(name, index, is a FUNCTION record)`.
    let mut introduced: Vec<Vec<(&str, u32, bool)>> = Vec::with_capacity(funcs.len());
    let mut fn_idx: Vec<u32> = Vec::with_capacity(funcs.len());
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
    let mut data_syms: Vec<(&str, u32)> = Vec::new();
    // **W-DATA** — the index of function `i`'s defined object's symbol, or
    // `None`. Derived here because the `.text` relocation records need it before
    // the symbol table is written, and asserted again where the record goes out.
    let mut def_sym: Vec<Option<u32>> = Vec::with_capacity(funcs.len());
    // **W-XLR** — per function, the frame-helper externals it INTRODUCES, with
    // the indices they land at. Sized up front because the index pass fills it
    // by position.
    let mut helper_idx: Vec<Vec<(&str, u32)>> = vec![Vec::new(); funcs.len()];
    // **W-BIQUAD** — the `__real@…` external's symbol index per pool entry.
    // Filled by this pass because the `.text` relocation records need it before
    // the symbol table is written.
    let mut real_idx: Vec<Option<u32>> = vec![None; pool.len()];
    for (i, f) in funcs.iter().enumerate() {
        next_idx += 2; // section symbol + aux
        fn_idx.push(next_idx);
        next_idx += 1; // the function symbol
        if labels[i].is_some() {
            next_idx += 1; // $M(n+1), the function-end label
        }
        // **W-UNDNAME / board #1720 — ONE list, reverse first-reference order
        // over callees ∪ data names, kind ignored**
        // ([`Function::introduced_externals`]). This was two loops, callees then
        // data symbols, and GRID A refutes that on three of five cells.
        //
        // The two lookup tables stay two, and that is deliberate: a call site
        // resolves in `callee_syms` and a data reference in `data_syms`, so one
        // list searched for both could silently resolve a data symbol against a
        // callee of the same spelling. What is shared now is the INDEX
        // assignment, which is the thing GRID A measured.
        let mut here: Vec<(&str, u32, bool)> = Vec::new();
        for (name, is_fn) in f.introduced_externals() {
            let known = callee_syms.iter().chain(data_syms.iter()).any(|(n, _)| *n == name);
            if known {
                continue;
            }
            // **W-FENCE2 — a callee THIS OBJ DEFINES is not an undefined
            // external, and minting one for it is an extra 18-byte symbol
            // record.** It was unreachable until this lane: `IlBundle::functions`
            // refused every TU that defined one of its own callees, so no obj had
            // ever carried an intra-TU `REL24`. `vsnprnc.cpp` is the first, and
            // the omission read as `Port=Mismatch @ offset 12`
            // (`NumberOfSymbols`), 1,501 bytes against the reference's 1,483.
            //
            // The resolution side is below, in the same `PlanTarget::Symbol`
            // arm the DEFINED-object table (W-DATA) already lives in — searched
            // FIRST there, for the same reason: a name this obj defines is never
            // also one of its undefined externals.
            if funcs.iter().any(|g| g.name == name) {
                continue;
            }
            if f.calls.iter().any(|c| c.callee == name) {
                callee_syms.push((name, next_idx));
            }
            if f.data_refs.iter().any(|r| r.name == name) {
                data_syms.push((name, next_idx));
            }
            here.push((name, next_idx, is_fn));
            next_idx += 1;
        }
        introduced.push(here);
        if labels[i].is_some() {
            next_idx += 1; // $M(n), the prologue-end label
            next_idx += 2; // .pdata section symbol + aux
            next_idx += 1; // $T(n+2)
        }
        // **W-XLR — the frame helpers, AFTER the `$T` label.**
        //
        // `docs/CODEGEN_FRAMED_CALLS.md` §2.3a's witnessed group ends
        // `… .pdata+aux · $T · __restgprlr_29 · __savegprlr_29`, so these two
        // are not in the callee region above and their indices are allocated
        // here instead. They go into `callee_syms` all the same, because their
        // relocations are ordinary REL24s and that is the table a REL24
        // resolves in.
        //
        // The `known` test is the same one the callee region uses and it is
        // load-bearing for a different reason here: `docs/LABEL_COUNTER.md`
        // §1.1's `gpr3-dup` row measures that a SECOND function reusing a width
        // an earlier one introduced pays **no** label surcharge and emits **no**
        // second symbol.
        for name in &funcs[i].helper_externals {
            if callee_syms.iter().chain(data_syms.iter()).any(|(n, _)| n == name) {
                continue;
            }
            callee_syms.push((name, next_idx));
            helper_idx[i].push((*name, next_idx));
            next_idx += 1;
        }
        // **W-DATA — this function's object group, immediately after its own.**
        //
        // The symbol table follows SECTION order and the `.data` COMDAT is
        // interleaved, so the group is too. MEASURED on `Primes.cpp`'s obj and
        // on GRID C's three-function cell:
        //
        // ```text
        //   … 11/12 .text 13 ?p0 14/15 .data 16 ?a
        //     17/18 .text 19 ?p1 20/21 .data 22 ?b
        //     23/24 .text 25 ?p2 26/27 .data 28 ?table
        // ```
        //
        // Its position relative to `_fltused` is not decided here and does not
        // have to be: a float function carrying an object is refused by the
        // class check above, because no cell says whether the marker goes inside
        // this group or after it.
        def_sym.push(if funcs[i].data_defs.iter().all(|d| d.uninitialized) {
            // Either no object at all, or only references into the TU's shared
            // `.bss`, whose group is in the shell and was counted there.
            None
        } else {
            next_idx += 2; // the `.data` section symbol + its aux record
            next_idx += 1; // the object's own defined STATIC symbol
            Some(next_idx - 1)
        });
        // **W-BIQUAD — this function's pool groups**, interleaved exactly as
        // their sections are and in the same LIFO order. Each is a `.rdata`
        // section symbol + aux, then the `__real@…` EXTERNAL.
        //
        // **Before `_fltused`, and that is measured rather than convenient**:
        // `docs/OBJ_GY_SHAPES.md` §1.2 states the marker goes after the first
        // float function's COMPLETE group — *"its `.text` section symbol + aux,
        // its function symbol, any callee externals it introduced, and any
        // `.rdata`/`__real@` pairs it introduced"*. `Biquad.cpp`'s obj is that
        // sentence: `?SetCoefficients` at 13, the two pool groups at 14–19,
        // `_fltused` at 20.
        for &k in &pool_of[i] {
            next_idx += 2; // the `.rdata` section symbol + its aux record
            real_idx[k] = Some(next_idx);
            next_idx += 1; // the `__real@…` external
        }
        if fltused_after == Some(i) {
            next_idx += 1;
        }
    }
    let n_symbols = next_idx;

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);

    // Interleaved to match the layout computed above: each section's raw data,
    // then its own relocations.
    //
    // **The cursor check EXCEPTS an uninitialized section, and that exception is
    // the whole content of board #3074.** `layout_sections` gives a `.bss`
    // `PointerToRawData = 0` — measured against real c2, `OBJ_DYNINIT_SHAPE.md`
    // §1 — so `ptrs[i]` is 0 there and is not a file offset at all. Written
    // 2026-07-29 (`cebfb88d`, W13b) when no section in this emitter could be
    // uninitialized, the bare form became FALSE on 2026-08-10 when `w-wordwrap2`
    // spliced the shared `.bss` into slot `B`, and stayed false for four days
    // because every standing instrument runs `--release`. The two siblings that
    // met `.bss` first — [`super::dyninit`] and [`super::data`] — already carry
    // exactly this guard; this file is the copy that did not get it.
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_none() {
            debug_assert_eq!(b.0.len(), ptrs[i]);
        }
        // The check that actually has teeth here, and the one the bare cursor
        // assertion could never make: an uninitialized section must carry NO raw
        // bytes, or this write and the layout cursor disagree by exactly
        // `raw.len()` and every later section's offset is wrong.
        debug_assert_eq!(s.file_len(), s.raw.len());
        b.bytes(&s.raw);
        match owner[i] {
            SectionOwner::Text(k) => {
                debug_assert!(
                    n_reloc_of[i] == 0 || b.0.len() == reloc_ptr[i].unwrap()
                );
                // One REL24 per call site (several sites may share one symbol
                // index — the same callee called twice) and, WR1, one
                // REFHI/PAIR/REFLO/PAIR quad per data-symbol address. Emitted
                // **ascending by VirtualAddress**, which is what the records in a
                // section are ordered by: the `lis` is at offset 0 and the tail
                // branch is last. The sort is stable, so each quad keeps its
                // REFHI-before-PAIR order at equal VA.
                //
                // **The list itself comes from `comdat::text_reloc_plan`** —
                // the same call FUNCTION BYTE MATCH makes. Only the NAME → this
                // obj's symbol index resolution happens here, because only this
                // writer has a symbol table. One locator (board #880's rule):
                // an instrument that rebuilt the plan could drift from what the
                // writer actually emits.
                let recs = crate::comdat::text_reloc_plan(
                    &funcs[k].calls,
                    &funcs[k].data_refs,
                    &funcs[k].data_defs,
                    &funcs[k].fp_refs,
                );
                debug_assert_eq!(recs.len(), n_reloc_of[i] as usize);
                for r in recs {
                    let sym = match r.target {
                        crate::comdat::PlanTarget::PairDisplacement(d) => d,
                        // **W-BIQUAD — a FIFTH table**, and the only one keyed
                        // by a value rather than a name: a pooled constant's
                        // symbol is identified by its `(bit pattern, width)`,
                        // which is exactly the key the pool is deduped on. A
                        // name lookup here would have to render the symbol and
                        // then parse it back, which is two spellings of one
                        // fact.
                        crate::comdat::PlanTarget::FpPool { bits, double } => real_idx
                            [pool_ix(bits, double).expect("every FP reference is pooled")]
                        .expect("every pooled constant got a symbol"),
                        // A call site resolves in the CALLEE table and a data
                        // reference in the DATA table — never one list searched
                        // for both, which would silently resolve a data symbol
                        // against a callee of the same spelling.
                        //
                        // **W-DATA adds a THIRD table**, for the same reason
                        // rather than for a new one: a DEFINED object's symbol
                        // and an undefined external's are two different records
                        // and nothing forbids a TU from spelling both. Searched
                        // first because a name that is defined here is never
                        // also an undefined external, so a hit is unambiguous.
                        crate::comdat::PlanTarget::Symbol(n) => {
                            // **W-FENCE2 — a FOURTH table, and it is searched
                            // before all of them**: a `REL24` whose target is a
                            // function THIS OBJ DEFINES resolves to that
                            // function's own defined symbol, not to an undefined
                            // external. Same argument as the DEFINED-object table
                            // below — a name this obj defines is never also one
                            // of its undefined externals, so a hit is
                            // unambiguous — and the index-assignment pass above
                            // therefore mints no symbol for it.
                            // **W-WORDWRAP2 — a SIXTH table, searched with the
                            // fourth and for its argument**: the TU's shared
                            // `.bss` objects. Their symbols live in the shell,
                            // not in any function's group, so `def_sym` cannot
                            // hold them; and a name this obj defines in `.bss`
                            // is never also one of its undefined externals, so
                            // a hit is unambiguous.
                            if let Some(ix) =
                                bss_sym.iter().find_map(|(nm, ix)| (*nm == n).then_some(*ix))
                            {
                                ix
                            } else if let Some(ix) = funcs
                                .iter()
                                .zip(&fn_idx)
                                .find_map(|(g, ix)| (g.name == n).then_some(*ix))
                            {
                                ix
                            } else if let Some(ix) = funcs
                                .iter()
                                .zip(&def_sym)
                                .find_map(|(g, ix)| {
                                    g.data_defs
                                        .iter()
                                        .any(|d| d.symbol == n)
                                        .then_some(*ix)
                                        .flatten()
                                })
                            {
                                ix
                            } else {
                                let table = if r.ty == REL_PPC_REL24 {
                                    &callee_syms
                                } else {
                                    &data_syms
                                };
                                table
                                    .iter()
                                    .find(|(m, _)| *m == n)
                                    .map(|(_, ix)| *ix)
                                    .expect("every relocation target got a symbol")
                            }
                        }
                    };
                    b.u32(r.va);
                    b.u32(sym);
                    b.u16(r.ty);
                }
            }
            SectionOwner::Pdata(k) => {
                // `BeginAddress` at `.pdata` offset 0, ADDR32 against the framed
                // function's own symbol (the record's raw addend is 0).
                debug_assert_eq!(b.0.len(), reloc_ptr[i].unwrap());
                b.u32(0);
                b.u32(fn_idx[k]);
                b.u16(REL_PPC_ADDR32);
            }
            SectionOwner::Data(_) | SectionOwner::Rdata(_) | SectionOwner::Fixed => {}
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    let mut strtab = StringTable::new();
    if sec_bss.is_some() {
        // **W-WORDWRAP2** — the shell with the shared `.bss` group spliced in.
        // `bss_symbol_order` is Rule Y1's reverse-`.gl` permutation and
        // `bss_offsets` is Rule A1/A3′'s forward bump, so the two are read from
        // different vectors on purpose: a writer that used one order for both
        // reproduces `p2` and is wrong on `p9`, which is `wordwrap.obj`.
        let syms: Vec<(&str, u32, bool)> = bss_symbol_order
            .iter()
            .map(|&i| (bss[i].symbol, bss_offsets[i], bss[i].external))
            .collect();
        debug_assert_eq!(
            syms.iter().map(|s| s.0).collect::<Vec<_>>(),
            bss_sym.iter().map(|s| s.0).collect::<Vec<_>>(),
            "the order the relocation records were written with"
        );
        emit_shell_symbols_bss_slot_b(&mut b, &mut strtab, &sections, &syms);
    } else {
        emit_shell_symbols(&mut b, &mut strtab, &sections);
    }

    for (i, f) in funcs.iter().enumerate() {
        let sec_num = (sec_text[i] + 1) as i16;
        emit_section_symbol(&mut b, &sections[sec_text[i]], sec_num, text_reloc_count(f));
        // The function is at offset 0 of its own section.
        emit_function_symbol(&mut b, &mut strtab, f.name, sec_num, 0);
        if let (Some(m), Some(frame)) = (labels[i], f.frame.as_ref()) {
            emit_label_symbol(&mut b, &label_name('M', m[1]), frame.func_len, sec_num);
        }
        // **W-UNDNAME / board #1720 — ONE list, in the order the index
        // assignment above fixed**: reverse first-reference order over callees
        // and data names alike. Only the function that *introduces* a name emits
        // its symbol.
        //
        // **W-EXTDATA — the `Type` is not always 0x0020 and not always 0x0000.**
        // A callee is `0x0020`; a REFHI/REFLO against a data name is `0x0000` and
        // against a FUNCTION is `0x0020` (`coff::DataRef::is_function`, measured
        // side by side in one workload obj). Emitting the wrong one is a single
        // wrong byte in a symbol record that every relocation resolves through
        // regardless — `docs/GAPS.md` §6's silent shape.
        for (name, _, is_fn) in &introduced[i] {
            emit_external_symbol(&mut b, &mut strtab, name, 0, if *is_fn { 0x0020 } else { 0x0000 });
        }
        if let (Some(m), Some(frame), Some(ps)) = (labels[i], f.frame.as_ref(), sec_pdata[i]) {
            emit_label_symbol(&mut b, &label_name('M', m[0]), frame.prolog_len, sec_num);
            emit_section_symbol(&mut b, &sections[ps], (ps + 1) as i16, 1);
            emit_pdata_label_symbol(&mut b, &label_name('T', m[2]), 0, (ps + 1) as i16);
        }
        // **W-XLR — the frame helpers, in the slot the index pass reserved.**
        // Both are FUNCTION records (`Type 0x0020`), like any callee.
        for (name, _) in &helper_idx[i] {
            emit_external_symbol(&mut b, &mut strtab, name, 0, 0x0020);
        }
        // **W-DATA — this function's defined object's group**, interleaved
        // exactly as its section is.
        if let (Some(si), Some(d)) =
            (sec_data[i], f.data_defs.iter().find(|d| !d.uninitialized))
        {
            let dsec = (si + 1) as i16;
            // `nrel` 0: the section is pure data (see `n_reloc_of` above), and
            // the aux record is the SECOND place that count lives — this
            // writer's own history has it reading 0 while the header read 1.
            emit_section_symbol(&mut b, &sections[si], dsec, 0);
            debug_assert_eq!(
                ((b.0.len() - ptr_symtab) / SYMBOL_LEN) as u32,
                def_sym[i].expect("a function with a `.data` has a symbol slot"),
                "the index the `.text` relocation records were written with"
            );
            // **Defined, STATIC, DATA type, `Value` = its offset in its own
            // section — which is 0, because the object owns the whole COMDAT.**
            // StorageClass 3 and a real section number are the whole difference
            // from the WR1 undefined external emitted a few lines above, which
            // is class 2 in section 0.
            emit_symbol(&mut b, &mut strtab, d.symbol, 0, dsec, 0x0000, 3);
        }
        // **W-BIQUAD — this function's pool groups**, in the slots the index
        // pass reserved. A `.rdata` section symbol carries `nrel` 0 (the section
        // is pure data) and the `__real@…` record is an undefined EXTERNAL of
        // DATA type `0x0000` — not `0x0020`, which is what a callee carries.
        for &k in &pool_of[i] {
            let si = sec_pool[k].expect("every introduced pool got a section");
            emit_section_symbol(&mut b, &sections[si], (si + 1) as i16, 0);
            debug_assert_eq!(
                ((b.0.len() - ptr_symtab) / SYMBOL_LEN) as u32,
                real_idx[k].expect("every introduced pool got a symbol"),
                "the index the `.text` relocation records were written with"
            );
            let (bits, double) = pool[k];
            // **In its OWN section, not section 0.** The record is storage class
            // EXTERNAL and DATA type `0x0000`, but it is a DEFINED external —
            // the constant lives in the `.rdata` COMDAT emitted three lines up.
            // Emitting section 0 makes it undefined, which links (the linker
            // finds another TU's copy) and is one wrong `i16` in the middle of
            // the symbol table.
            emit_external_symbol(
                &mut b,
                &mut strtab,
                &real_symbol_name(bits, double),
                (si + 1) as i16,
                0x0000,
            );
        }
        // The CRT float-support marker, once, after the first FP function's group.
        if fltused_after == Some(i) {
            emit_function_symbol(&mut b, &mut strtab, NAME_FLTUSED, 0, 0);
        }
    }

    b.bytes(&strtab.finish());
    Some(b.0)
}

/// Build the complete `.obj` image for one or more straight-line functions
/// sharing a single `.text`. Generalizes [`emit_mvp_obj`]: functions are packed
/// contiguously in `.text` (no inter-function padding — c2's real layout), each
/// gets an EXTERNAL FUNCTION symbol whose `Value` is its `.text` byte offset,
/// and `NumberOfSymbols` = 13 fixed slots + one per function.
///
/// * `obj_name` — the `-Fo` path (embedded in `.debug$S` S_OBJNAME).
/// * `funcs` — functions in emit order (matches `.gl`/`.ex` order); each
///   `text_offset` is its start within `text`.
/// * `text` — the full concatenated `.text` bytes from codegen.
pub fn emit_obj(obj_name: &str, funcs: &[Function], text: &[u8], label_counter: u32) -> Vec<u8> {
    // **W-DATA — the packed layout has no measured place for a COMDAT `.data`.**
    //
    // A `debug_assert` and not a refusal, because the only caller is
    // `PortC2::build`'s `/Ox` arm and that arm refuses a function carrying a
    // `data_def` **upstream**, with a message. Emitting the section here
    // anyway would put it after `.pdata` and the interleave rule this
    // function's own `pdata_idx` comment measures (six distinct orders over
    // 240 objs) says section order in the packed layout is `.text`-order and
    // not append. Nothing has ever captured a packed obj with one.
    debug_assert!(
        funcs.iter().all(|f| f.data_defs.is_empty()),
        "the packed writer has no measured slot for a COMDAT `.data`; \
         `PortC2::build` refuses upstream"
    );
    let labels = plan_labels(label_counter, funcs, false);
    // One `.pdata` section for the whole TU, records in `.text` order — packed,
    // unlike `/Gy`, which gives each framed function its own COMDAT.
    let framed: Vec<&Frame> = funcs.iter().filter_map(|f| f.frame.as_ref()).collect();
    let pdata = build_pdata(&framed);

    // W13b: pool the floating-point constants, TU-wide, by bit pattern **and**
    // width (a `float` 1.0 and a `double` 1.0 are different symbols with
    // different section sizes). First-reference order fixes both the `.rdata`
    // section order and the symbol order.
    let mut pool: Vec<(u64, bool)> = Vec::new();
    for f in funcs {
        for r in &f.fp_refs {
            if !pool.contains(&(r.bits, r.double)) {
                pool.push((r.bits, r.double));
            }
        }
    }
    let pool_ix = |bits: u64, double: bool| -> usize {
        pool.iter().position(|&k| k == (bits, double)).expect("pooled")
    };

    // Section table, in the fixed emit order.
    let mut sections = shell_sections(obj_name);
    sections.push(Section {
        name: ".text",
        characteristics: CH_TEXT,
        raw: std::borrow::Cow::Borrowed(text),
        checksum: 0,
        selection: 0,
        assoc: 0,
        uninit_size: None,
    });
    let text_idx = sections.len() - 1;
    for &(bits, double) in &pool {
        sections.push(Section {
            name: ".rdata",
            characteristics: if double { CH_RDATA_F64 } else { CH_RDATA_F32 },
            raw: std::borrow::Cow::Owned(real_raw_bytes(bits, double)),
            checksum: 0,
            selection: 2,
            assoc: 0,
            uninit_size: None,
        });
    }
    // `.pdata` last — which is right only because the combination that would test
    // it is refused upstream, and **the rule it would need is now measured**.
    //
    // The comment here used to read "a TU with BOTH a constant pool and a framed
    // function would settle the `.rdata`/`.pdata` order, and none has been
    // captured". 240 such TUs were then captured (`/Ox /GS- /c`, every order of
    // one or two constant-pooling FP leaves against one or two framed functions),
    // and the answer is **not a fixed order at all**:
    //
    // > The packed section table lists `.rdata` and `.pdata` **interleaved, in
    // > `.text` order** — each section at the position of the FIRST function that
    // > needs it. `.pdata` stays a single section for the whole TU and sits where
    // > the first framed function does.
    //
    // Six distinct orders occur in those 240 objs — `(.pdata,.rdata)` 78,
    // `(.rdata,.pdata)` 64, `(.pdata,.rdata,.rdata)` 30, `(.pdata,)` 22,
    // `(.rdata,.rdata,.pdata)` 20, `(.rdata,.pdata,.rdata)` 20 — and this
    // function can express exactly one of those shapes. `L1(2.5f); seq2();
    // L2(3.5f);` is `.rdata .pdata .rdata`, which no amount of reordering the two
    // groups below produces.
    //
    // **One capture would have said the opposite.** A single leaf-then-framed TU
    // reads `.rdata .pdata`, i.e. exactly what this code already emits, and would
    // have licensed deleting the refusal. Widening here needs the interleave, not
    // a second constant in a list.
    let pdata_idx = if framed.is_empty() {
        None
    } else {
        debug_assert!(pool.is_empty(), "framed + pooled FP constant is refused upstream");
        sections.push(Section {
            name: ".pdata",
            characteristics: CH_PDATA,
            raw: std::borrow::Cow::Borrowed(&pdata),
            // The one non-COMDAT section c2 gives a real CheckSum.
            checksum: coff_checksum(&pdata),
            selection: 0,
            assoc: 0,
            uninit_size: None,
        });
        Some(sections.len() - 1)
    };
    let n_sections = sections.len();

    // Symbol layout: 13 fixed slots (indices 0..13), then per function a defined
    // FUNCTION symbol, each immediately followed by its callee's undefined
    // external symbol (if any), then — for each pooled constant this function is
    // the *first* to reference — that constant's `.rdata` section symbol (+ aux)
    // and its `__real@…` external. `_fltused` is emitted once, immediately after
    // the FIRST float function's symbol group.
    //
    // This runs before the relocations are written because each REFHI/REFLO
    // record needs its `__real@…` symbol index.
    let fltused_after = funcs.iter().position(|f| f.is_float);
    let mut next_idx: u32 = 13;
    // (function index, its defined symbol, the callee symbols it introduces —
    // reverse first-reference order, with their indices — constants introduced)
    let mut plan: Vec<(usize, u32, Vec<(&str, u32, bool)>, Vec<usize>)> =
        Vec::with_capacity(funcs.len());
    let mut real_idx: Vec<Option<u32>> = vec![None; pool.len()];
    // An undefined external callee is emitted **once per distinct name**, after the
    // symbol of the function that first calls it — every later call site relocates
    // against that same index. Emitting one per call site instead is invisible
    // until two functions in a TU call the same callee, which no fixture did before
    // `il_call_perm.cpp`; the reference puts `?g3` after `pass3` and nothing after
    // the four later functions that also call it.
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
    // **WR1** — the same rule for a named data symbol: one undefined external per
    // distinct name, emitted in the group of the function that first references
    // it, with every later site relocating against that index. MEASURED
    // (`work/wr1/probes/p1.cpp`): `?gI@@3HA` is referenced by three functions and
    // appears once, at index 21, which all three relocations name.
    let mut data_syms: Vec<(&str, u32)> = Vec::new();
    // Packed, the whole TU shares ONE `.pdata`, so its section symbol + aux are
    // emitted once — inside the group of the FIRST framed function, after that
    // function's prologue label and before its `$T`. Every later framed function
    // contributes only `$M`, `$M` and `$T`.
    let first_framed = funcs.iter().position(|f| f.frame.is_some());
    for (i, f) in funcs.iter().enumerate() {
        let def_idx = next_idx;
        next_idx += 1;
        if labels[i].is_some() {
            next_idx += 1; // $M(n+1), the function-end label
        }
        // **W-UNDNAME / board #1720 — ONE list, reverse first-reference order
        // over callees ∪ data names, kind ignored**
        // ([`Function::introduced_externals`]). Two loops here too, and GRID A
        // refutes them here too; the same rule and the same locator, so the two
        // writers cannot disagree about the symbol order the way they could
        // about the `Type`.
        let mut new_ext: Vec<(&str, u32, bool)> = Vec::new();
        for (name, is_fn) in f.introduced_externals() {
            if callee_syms.iter().chain(data_syms.iter()).any(|(n, _)| *n == name) {
                continue;
            }
            // The two lookup tables stay two — a REL24 resolves in one and a
            // REFHI/REFLO in the other, so a data symbol can never be resolved
            // against a callee of the same spelling. Only the INDEX is shared.
            if f.calls.iter().any(|c| c.callee == name) {
                callee_syms.push((name, next_idx));
            }
            if f.data_refs.iter().any(|r| r.name == name) {
                data_syms.push((name, next_idx));
            }
            new_ext.push((name, next_idx, is_fn));
            next_idx += 1;
        }
        if labels[i].is_some() {
            next_idx += 1; // $M(n), the prologue-end label
            if first_framed == Some(i) {
                next_idx += 2; // the shared .pdata section symbol + aux
            }
            next_idx += 1; // $T(n+2)
        }
        // Constants this function introduces, in first-reference order.
        let mut introduced: Vec<usize> = Vec::new();
        for r in &f.fp_refs {
            let k = pool_ix(r.bits, r.double);
            if real_idx[k].is_none() {
                next_idx += 2; // .rdata section symbol + its aux record
                real_idx[k] = Some(next_idx);
                next_idx += 1; // the __real@… external
                introduced.push(k);
            }
        }
        plan.push((i, def_idx, new_ext, introduced));
        if fltused_after == Some(i) {
            next_idx += 1;
        }
    }
    let n_symbols: u32 = next_idx;

    // The `.pdata` relocations: one ADDR32 per record, at the record's own
    // offset, against the framed function's defined symbol. In `.text` order,
    // which is also ascending VirtualAddress.
    let mut pdata_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for (i, def, _new, _intro) in &plan {
        if funcs[*i].frame.is_some() {
            pdata_relocs.push((pdata_relocs.len() as u32 * 8, *def, REL_PPC_ADDR32));
        }
    }

    // Relocations (`.text` only in this class) sit between the raw data and the
    // symbol table, **ascending by VirtualAddress**. A tail call contributes one
    // REL24; each FP constant reference contributes a REFHI/PAIR on the `addis`
    // and a REFLO/PAIR on the `lfs`/`lfd` four bytes later. The PAIR records
    // carry the partner half's displacement in the symbol-index field, which is
    // always 0 because every constant owns its whole COMDAT section.
    let mut text_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for (i, _def, _new, _intro) in &plan {
        let f = &funcs[*i];
        // One REL24 per call site; several sites may share one symbol index.
        for call in &f.calls {
            let cidx = callee_syms
                .iter()
                .find(|(n, _)| *n == call.callee)
                .map(|(_, ix)| *ix)
                .expect("every callee got a symbol");
            text_relocs.push((call.reloc_offset, cidx, REL_PPC_REL24));
        }
        for r in &f.fp_refs {
            let sym = real_idx[pool_ix(r.bits, r.double)].expect("pooled symbol");
            text_relocs.push((r.hi_off, sym, REL_PPC_REFHI));
            text_relocs.push((r.hi_off, 0, REL_PPC_PAIR));
            text_relocs.push((r.lo_off, sym, REL_PPC_REFLO));
            text_relocs.push((r.lo_off, 0, REL_PPC_PAIR));
        }
        // WR1: byte-for-byte the same quad, against an undefined external instead
        // of a pooled constant's `.rdata` symbol.
        for r in &f.data_refs {
            let sym = data_syms
                .iter()
                .find(|(n, _)| *n == r.name)
                .map(|(_, ix)| *ix)
                .expect("every data symbol got a slot");
            text_relocs.push((r.hi_off, sym, REL_PPC_REFHI));
            text_relocs.push((r.hi_off, 0, REL_PPC_PAIR));
            text_relocs.push((r.lo_off, sym, REL_PPC_REFLO));
            text_relocs.push((r.lo_off, 0, REL_PPC_PAIR));
        }
    }
    text_relocs.sort_by_key(|&(va, _, _)| va);
    let n_text_reloc = text_relocs.len();

    // Raw data is packed right after the section headers, and a section's
    // relocation records sit immediately after **that section's own** raw data —
    // not after every section's. With `.text` last (no constant pool) the two
    // layouts coincide, which is why this only surfaced once `.rdata` followed
    // `.text`: c2 put the four REFHI/REFLO records between `.text` and the
    // constant pool, the port put them after both.
    // Only `.text` and (when present) `.pdata` carry relocations in this class —
    // the `.rdata` constant pools are pure data.
    let mut n_reloc_of = vec![0u16; n_sections];
    n_reloc_of[text_idx] = n_text_reloc as u16;
    if let Some(pi) = pdata_idx {
        n_reloc_of[pi] = pdata_relocs.len() as u16;
    }
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);
    let ptr_text_reloc = reloc_ptr[text_idx].unwrap_or(0);
    let ptr_pdata_reloc = pdata_idx.and_then(|pi| reloc_ptr[pi]).unwrap_or(0);

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);

    // ---- raw section data, each section followed by its own relocations ----
    // (10 bytes each: VA u32, SymIdx u32, Type u16)
    //
    // Same guard as `emit_comdat_obj` above, and here it is LATENT rather than
    // live: no `.bss` reaches the packed emitter today, so this site never
    // fired. It is corrected anyway because two copies of one invariant, one
    // right and one known-wrong, is the shape board #880 forbids — the next
    // section this emitter learns to place uninitialized would trip the same
    // false assertion, invisibly, for exactly as long.
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_none() {
            debug_assert_eq!(b.0.len(), ptrs[i]);
        }
        debug_assert_eq!(s.file_len(), s.raw.len());
        b.bytes(&s.raw);
        if i == text_idx {
            debug_assert!(n_text_reloc == 0 || b.0.len() == ptr_text_reloc);
            for &(va, sym, typ) in &text_relocs {
                b.u32(va);
                b.u32(sym);
                b.u16(typ);
            }
        }
        if Some(i) == pdata_idx {
            debug_assert_eq!(b.0.len(), ptr_pdata_reloc);
            for &(va, sym, typ) in &pdata_relocs {
                b.u32(va);
                b.u32(sym);
                b.u16(typ);
            }
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table + string table ----
    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections); // slots 0..=10
    // Section STATIC symbols each carry one aux section-def record. `.text`
    // (sec 5) carries the relocation count in its aux.
    emit_section_symbol(&mut b, &sections[4], 5, n_text_reloc as u16); // slot 11/12 .text

    // Per function: the defined FUNCTION symbol, then (if a tail call) the
    // undefined external callee symbol, then the constant pools this function
    // introduces (`.rdata` section symbol + aux, then the `__real@…` external).
    for (i, _def, new_ext, introduced) in &plan {
        let f = &funcs[*i];
        emit_function_symbol(&mut b, &mut strtab, f.name, 5, f.text_offset);
        // A framed function's `$M` labels are its prologue end and its function
        // end **relative to its own start**, so packed they are rebased onto the
        // shared `.text`; under `/Gy` the function starts at 0 of its own COMDAT
        // and the two coincide.
        if let (Some(m), Some(frame)) = (labels[*i], f.frame.as_ref()) {
            emit_label_symbol(&mut b, &label_name('M', m[1]), f.text_offset + frame.func_len, 5);
        }
        // Undefined externals: section 0 (UNDEF), in the order the merged index
        // assignment fixed. Only the function that FIRST names one emits its
        // symbol.
        //
        // The `Type` is the whole difference between a callee and a data name —
        // `0x0020` against `0x0000` — and it is the difference between "a
        // function pointer" and "a data address" in the linker's eyes.
        // **W-EXTDATA**: a REFHI/REFLO against a FUNCTION takes `0x0020` too, so
        // the bit is `DataRef::is_function` and not "did this come from `calls`".
        // One fact, two writers, asked through `introduced_externals` in both.
        for (name, _, is_fn) in new_ext {
            emit_external_symbol(&mut b, &mut strtab, name, 0, if *is_fn { 0x0020 } else { 0x0000 });
        }
        if let (Some(m), Some(frame), Some(pi)) = (labels[*i], f.frame.as_ref(), pdata_idx) {
            emit_label_symbol(&mut b, &label_name('M', m[0]), f.text_offset + frame.prolog_len, 5);
            if first_framed == Some(*i) {
                emit_section_symbol(
                    &mut b,
                    &sections[pi],
                    (pi + 1) as i16,
                    pdata_relocs.len() as u16,
                );
            }
            // `$T` value is this record's byte offset inside the shared `.pdata`.
            let rec = funcs[..*i].iter().filter(|g| g.frame.is_some()).count() as u32 * 8;
            emit_pdata_label_symbol(&mut b, &label_name('T', m[2]), rec, (pi + 1) as i16);
        }
        for &k in introduced {
            let sec_num = (text_idx + 1 + k + 1) as i16;
            emit_section_symbol(&mut b, &sections[text_idx + 1 + k], sec_num, 0);
            let (bits, double) = pool[k];
            // A pooled constant is DATA, not a function: type 0x0000.
            emit_external_symbol(
                &mut b,
                &mut strtab,
                &real_symbol_name(bits, double),
                sec_num,
                0x0000,
            );
        }
        // The CRT float-support marker, once, after the first FP function.
        if fltused_after == Some(*i) {
            emit_function_symbol(&mut b, &mut strtab, NAME_FLTUSED, 0, 0);
        }
    }

    // ---- string table ----
    b.bytes(&strtab.finish());

    b.0
}

// ===========================================================================
// #158 — the `??__E` dynamic-initializer obj.
//
// A TU whose only emitted function is one `??__E<name>@@YAXXZ` thunk running
// one namespace-scope object's constructor. Eight sections, 24 symbol records,
// 9 + 1 relocations. Every byte below is transcribed from an obj produced by the
// real cl 16.00.11886.00 / c2.dll under wibo; `docs/OBJ_DYNINIT_SHAPE.md` is the
// characterization and names the cell each rule was fitted on and tested
// against. Where that doc and the bytes disagree, the bytes win, and the three
// places they do are marked CORRECTION below.
//
// **Grade at `/O1`, not `/Ox`** (§7.3 caveat 1): `/Ox` does not imply `/GF`, and
// without `/GF` the literal is a non-COMDAT `$SG<n>` `.rdata` placed *before*
// `.text`, with no `??_C@…` symbol at all. That is a different obj.
// ===========================================================================
