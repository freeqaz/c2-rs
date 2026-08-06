#!/usr/bin/env python3
"""resolve_obj.py — resolve the c2-obj conflict between lanes w-bytes and w-splice.

BOTH SIDES ARE ADDITIVE READERS AND BOTH SURVIVE.

    w-bytes (master)  text_comdat_reloc_sites   (offset, type) per COMDAT
                      text_comdat_call_targets  REL24 targets by name
    w-splice (this)   text_comdat_relocs        (offset, raw type, target) per COMDAT

git interleaved them because `text_comdat_call_targets` and `text_comdat_relocs`
each open with the **same symbol-table walk** — the loop that builds a
`Vec<Option<String>>` indexed by symbol-table slot, skipping aux records. That
walk is now `symbol_names_by_slot`, called by both, which is the resolution this
repo's own rule asks for: one fact, one locator (`docs/GAPS.md` §6). Two copies
of a COFF walk is exactly the shape that produced the four wrong relocation-type
rows `gt_dump.py`'s header records.

Neither reader's SEMANTICS are touched:

  * `text_comdat_call_targets` still fails closed on a target it cannot name
    (`names.get(idx)?.clone()?`) — a REL24 naming an aux slot is a decode
    failure and w-bytes detects it rather than papering over it;
  * `text_comdat_relocs` still reports `None` for a target it cannot name,
    because a `PAIR`'s symbol field is a displacement and the caller compares
    the nameable records only.

The two answers to "what does this word point at" are deliberately different and
both are kept.
"""

import re
import sys

MASTER = "/tmp/obj_master.rs"
MINE = "/tmp/obj_mine.rs"
TARGET = "crates/c2-obj/src/lib.rs"

HELPER = '''    /// **Symbol names by symbol-table SLOT**, aux slots left `None`.
    ///
    /// The one walk two readers need — [`ObjImage::text_comdat_call_targets`]
    /// (lane `w-bytes`) and [`ObjImage::text_comdat_relocs`] (lane `w-splice`)
    /// both resolve a relocation's `SymbolTableIndex` through it, and they
    /// landed in the same release. Factored here rather than written twice:
    /// `docs/GAPS.md` §6's one-fact-one-locator rule, and a COFF walk is the
    /// exact shape that rule exists for — `gt_dump.py`'s own header records four
    /// relocation-type rows that were wrong for the file's whole existence
    /// because a second copy of a table drifted from the first.
    ///
    /// **Aux records occupy indices too**, and they stay `None`: an aux slot is
    /// not a symbol, so a relocation that names one is a decode failure. What
    /// the two readers *do* with that `None` is deliberately different and is
    /// each one's own business — `call_targets` refuses the whole obj,
    /// `relocs` reports the record as unnameable.
    ///
    /// Same fail-closed contract as every other walk: `None` the moment the
    /// table does not decode, never a short list.
    fn symbol_names_by_slot(&self) -> Option<Vec<Option<String>>> {
        let b = &self.0;
        let (_, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];
        let mut names: Vec<Option<String>> = vec![None; nsym];
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            let name = if b[o..o + 4] == [0, 0, 0, 0] {
                let at = u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
                str_at(strtab, at)?
            } else {
                String::from_utf8_lossy(&b[o..o + 8])
                    .trim_end_matches('\\0')
                    .to_owned()
            };
            names[i] = Some(name);
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        Some(names)
    }

'''


def block(text, start_marker, end_marker):
    """The text from `start_marker` up to (not including) `end_marker`."""
    i = text.index(start_marker)
    j = text.index(end_marker, i)
    return text[i:j]


def main():
    master = open(MASTER).read()
    mine = open(MINE).read()

    # ---- master's two readers, verbatim, with the shared walk lifted out ----
    sites = block(master,
                  "    /// [`ObjImage::text_comdat_reloc_counts`] with each relocation's **offset",
                  "    /// **Which symbol each `.text` COMDAT CALLS**")
    calls_orig = block(master,
                       "    /// **Which symbol each `.text` COMDAT CALLS**",
                       "    /// **Every compiler-label symbol")
    calls = calls_orig.replace(
        """        let b = &self.0;
        let (_, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];
        // Name per symbol-table *slot*. Aux slots stay `None`: a relocation that
        // named one would be a decode failure, and this is how it is detected
        // rather than papered over.
        let mut names: Vec<Option<String>> = vec![None; nsym];
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            let name = if b[o..o + 4] == [0, 0, 0, 0] {
                let at = u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
                str_at(strtab, at)?
            } else {
                String::from_utf8_lossy(&b[o..o + 8])
                    .trim_end_matches('\\0')
                    .to_owned()
            };
            names[i] = Some(name);
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        let all = self.relocations()?;""",
        """        // The shared symbol-table walk (`symbol_names_by_slot`). `w-splice`'s
        // `text_comdat_relocs` reads it too, and one copy is the point.
        let names = self.symbol_names_by_slot()?;
        let all = self.relocations()?;""")
    assert "symbol_names_by_slot" in calls, "call_targets' walk did not lift"

    # ---- my reader, verbatim, with the same walk lifted out ----
    relocs = block(mine,
                   "    /// [`ObjImage::text_comdat_reloc_counts`] with each relocation **resolved to",
                   "    /// **Every compiler-label symbol")
    relocs = relocs.replace(
        """        let b = &self.0;
        let (_, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];
        // Symbol names by table index. Aux records occupy indices too, so the
        // vector is built over the whole table and the aux slots are left `None`
        // — indexing past them would name the wrong symbol, which is the exact
        // failure this reader exists to catch elsewhere.
        let mut names: Vec<Option<String>> = vec![None; nsym];
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            names[i] = Some(if b[o..o + 4] == [0, 0, 0, 0] {
                let at = u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
                str_at(strtab, at)?
            } else {
                String::from_utf8_lossy(&b[o..o + 8])
                    .trim_end_matches('\\0')
                    .to_owned()
            });
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        let mut out = Vec::new();""",
        """        let b = &self.0;
        // The shared symbol-table walk (`symbol_names_by_slot`). `w-bytes`'s
        // `text_comdat_call_targets` reads it too, and one copy is the point.
        let names = self.symbol_names_by_slot()?;
        let mut out = Vec::new();""")
    assert "symbol_names_by_slot" in relocs, "relocs' walk did not lift"

    # ---- splice the resolved region into master's file -------------------
    out = master.replace(sites + calls_orig, HELPER + sites + calls + relocs, 1)
    assert out != master, "the resolved region did not splice"
    open(TARGET, "w").write(out)
    print("resolved: kept text_comdat_reloc_sites, text_comdat_call_targets, "
          "text_comdat_relocs; one shared symbol_names_by_slot")


if __name__ == "__main__":
    main()
