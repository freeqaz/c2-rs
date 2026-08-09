//! **W-WORDWRAP — the file-scope-global store leaf.** `void f(T x) { g = x; }`,
//! `src/system/rndobj/wordwrap.cpp`'s `?WordWrap_SetOption@@YAXI@Z`, twelve
//! bytes, and the smallest unconverted body on the whole frontier
//! (board **#2625**).
//!
//! ```text
//!   0000  3d600000  lis      r11,0          REFHI <g> + PAIR
//!   0004  906b0000  st{b,h,w,d} r3,0(r11)   REFLO <g> + PAIR
//!   0008  4e800020  blr
//! ```
//!
//! **Three words with ONE free field**, the store's width, and every other
//! choice in them is a measurement rather than a default:
//!
//! * **r11 is the address scratch, and only because the body needs exactly one
//!   register.** GRID G's `G_lit` (`g = 7u`), `G_widen` and `G_narrow` each need
//!   a second, and in all three c2 moves the address to **r10**. `G_narrow` is
//!   the sharp one: it is still **twelve bytes** and two of its three words
//!   differ, so a length check cannot separate it from the accepted cell and the
//!   recognizer's no-conversion clause is what does.
//! * **r3 is the value, and only because the class is fenced at one formal.**
//!   `G_second` stores r4.
//! * **The displacement is 0**, and the REFLO relocation is what supplies the
//!   object's low half. `G_arr2` (`g_arr[2] = x`) is the cell that shows a real
//!   displacement arriving instead as `addi r11,r11,<lo>` + `stw r3,8(r11)` —
//!   four words, and a different relocation site.
//!
//! # This body DEFINES the object it writes, and that is a `.bss`
//!
//! It travels with [`c2_il::IlFunction::data_def`] set, exactly as
//! `static_scan_loop` does, and with
//! [`c2_il::IlDataDef::uninitialized`] — the object has no `.in` record at all.
//! The relocation plan is identical either way (`comdat::text_reloc_plan`
//! compares targets by NAME), so this body is byte-and-relocation gradable
//! today; the **obj** it belongs to is not, and `coff::writer::emit_obj_multi`
//! refuses it by name. See that field for why.
//!
//! # The mode gate
//!
//! `/O1`, `/O1 /Oi`, `/O2`, `/Ox` and `/Ox /Gy` all emit these identical three
//! words — measured, `work/w-wordwrap/probe/gstore.cpp`. `/Od` emits five and is
//! already refused by `opt_word_mode`. So unlike every `w-xtea3` class this one
//! is **not** `/O1`-only, and the emitter asks for no mode at all rather than
//! restating a gate it does not have.

use crate::codegen::encode::{encode_addis, encode_blr, encode_stb, encode_std, encode_sth, encode_stw};
use crate::codegen::select::out_of_class;
use crate::BackendError;
use c2_il::GlobalStoreLeaf;

/// The address scratch. See the module header: it is r11 **because** the body
/// needs exactly one register, and r10 in every cell that needs two.
const R_ADDR: u8 = 11;
/// The value's argument register — the class's single formal.
const R_VALUE: u8 = 3;

/// The whole body, `blr` included. Nothing is left for the caller: this class
/// has no branch word that encodes its own `.text` offset.
///
/// The two relocation SITES are `.text+0` (the `lis`) and `.text+4` (the store),
/// and they are located by `crate::data_defs_of` off these very bytes rather
/// than being asserted here — one fact, one locator.
pub fn global_store_leaf_text(g: &GlobalStoreLeaf) -> Result<Vec<u8>, BackendError> {
    let store = match g.width {
        1 => encode_stb(R_VALUE, R_ADDR, 0),
        2 => encode_sth(R_VALUE, R_ADDR, 0),
        4 => encode_stw(R_VALUE, R_ADDR, 0),
        8 => encode_std(R_VALUE, R_ADDR, 0),
        // Unreachable from the parser — `global_store_leaf`'s `STORE_WIDTHS`
        // table yields only these four — and stated as a refusal rather than an
        // `unreachable!` because the CLI must degrade cleanly.
        _ => {
            return Err(out_of_class(
                "a file-scope-global store whose width is not 1, 2, 4 or 8: the \
                 store opcode comes from GRID T's enumerated table and nothing \
                 else spells one",
            ))
        }
    };
    let mut t = Vec::with_capacity(12);
    t.extend_from_slice(&encode_addis(R_ADDR, 0, 0));
    t.extend_from_slice(&store);
    t.extend_from_slice(&encode_blr());
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?WordWrap_SetOption@@YAXI@Z`'s own twelve bytes, word for word off
    /// `work/w-wordwrap/ref/wordwrap.dump`.
    #[test]
    fn the_target_body_is_twelve_bytes() {
        let t = global_store_leaf_text(&GlobalStoreLeaf { width: 4 }).unwrap();
        assert_eq!(
            t,
            vec![
                0x3d, 0x60, 0x00, 0x00, // lis 11,0
                0x90, 0x6b, 0x00, 0x00, // stw 3,0(11)
                0x4e, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    /// GRID T's other three store opcodes, each off its own compiled cell —
    /// `T_uc`, `T_us` and `T_ull` in `work/w-wordwrap/probe/gtype.cpp`.
    #[test]
    fn the_three_other_widths_are_grid_t_cells() {
        let w = |n| global_store_leaf_text(&GlobalStoreLeaf { width: n }).unwrap()[4..8].to_vec();
        assert_eq!(w(1), vec![0x98, 0x6b, 0x00, 0x00], "stb 3,0(11)  — T_uc");
        assert_eq!(w(2), vec![0xb0, 0x6b, 0x00, 0x00], "sth 3,0(11)  — T_us");
        assert_eq!(w(8), vec![0xf8, 0x6b, 0x00, 0x00], "std 3,0(11)  — T_ull");
    }

    /// A width no cell produced refuses rather than emitting a default store.
    #[test]
    fn an_unmeasured_width_refuses() {
        assert!(global_store_leaf_text(&GlobalStoreLeaf { width: 3 }).is_err());
    }
}
