//! Symbol-record writers — one shape per (Type, StorageClass, Value)
//! combination this port emits, as arguments rather than near-copies.

use super::*;

/// One COFF symbol-table record (also the aux-record stride).
pub(crate) const SYMBOL_LEN: usize = 18;


/// Emit a section STATIC symbol + its aux section-def record. `n_reloc` is the
/// section's relocation count (0 for all sections except `.text` when calls
/// are present) — it appears in the aux record and must match the section
/// header's `NumberOfRelocations`.
pub(crate) fn emit_section_symbol(b: &mut Buf, s: &Section, sec_num: i16, n_reloc: u16) {
    b.name8(s.name);
    b.u32(0); // Value
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(3); // STATIC
    b.u8(1); // one aux record

    // Aux section-def: Length | nReloc(u16) | nLineno(u16) | CheckSum(u32) |
    //                  Number(u16) | Selection(u8) | Unused[3].
    // `Length` is the section's declared size, which for `.bss` is NOT the raw
    // byte count (there is none) — see [`Section::uninit_size`].
    b.u32(s.size());
    b.u16(n_reloc); // NumberOfRelocations
    b.u16(0); // NumberOfLinenumbers
    b.u32(s.checksum);
    b.u16(s.assoc); // Number — 0 unless Selection=5 (ASSOCIATIVE)
    b.u8(s.selection);
    b.bytes(&[0, 0, 0]); // Unused
}

/// Emit one aux-less symbol record with an inline (≤ 8 byte) or string-table
/// name. The three axes that actually vary between the callers — `Type`
/// (0x0020 FUNCTION vs 0x0000 DATA), `StorageClass` (2 EXTERNAL vs 3 STATIC)
/// and `Value` — are arguments rather than three near-copies of this body,
/// because the dynamic-initializer obj needs the one combination none of the
/// older helpers spelled: a **STATIC symbol of FUNCTION type** (the
/// `??__E<name>@@YAXXZ` thunk, `docs/OBJ_DYNINIT_SHAPE.md` §3.1).
pub(crate) fn emit_symbol(
    b: &mut Buf,
    strtab: &mut StringTable,
    name: &str,
    value: u32,
    sec_num: i16,
    typ: u16,
    storage_class: u8,
) {
    if name.len() <= 8 {
        b.name8(name);
    } else {
        let off = strtab.intern(name);
        b.u32(0); // long-name marker
        b.u32(off);
    }
    b.u32(value);
    b.i16(sec_num);
    b.u16(typ);
    b.u8(storage_class);
    b.u8(0); // no aux
}

/// Emit an EXTERNAL symbol whose (long) name lives in the string table.
pub(crate) fn emit_external_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, typ: u16) {
    emit_symbol(b, strtab, name, 0, sec_num, typ, 2);
}

/// Emit an EXTERNAL FUNCTION symbol (type 0x20) whose (long) name lives in the
/// string table, with `Value` = its byte offset within `.text`.
pub(crate) fn emit_function_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, value: u32) {
    emit_symbol(b, strtab, name, value, sec_num, 0x0020, 2);
}
