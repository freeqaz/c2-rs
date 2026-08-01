//! `c2-reference::cod` — a reader for c2's own **assembly listing** (`.cod`),
//! the artifact `-FAasc -Fa <file>` makes it write (roadmap §9, board #132).
//!
//! # What this is, and the one thing it is NOT
//!
//! The listing is an **output of the black box**, in the same category as the
//! obj: c2 wrote it, we did not decompile anything to get it. It carries, per
//! function, the emitted bytes with source-line correlation, the COMDAT/section
//! emission order, every relocation **target by name**, the frame slot
//! assignments, the EH record layout, and c2's internal label counter in
//! allocation order.
//!
//! It is **never a gate**. The obj byte-compare remains the sole judge of the
//! port (CLAUDE.md); the listing is a decode aid.
//!
//! # The trap this module exists to document: `.cod` is NOT raw-byte truth
//!
//! Row-for-row the listing agrees with the obj — *except at relocated branches*.
//! There the listing prints the **canonical unrelocated word** and names the
//! target symbol:
//!
//! ```text
//!   .cod:  00014  48000001   bl   ?void_func@@YAXH@Z
//!   .obj:  00014  4bffffed                              <- real displacement
//! ```
//!
//! `b` → `48000000`, `bl` → `48000001`. Measured over the whole 204-fixture
//! corpus at `/O1 /Oi /EHsc /GS-`: **9,430 rows identical, 1,024 differing, and
//! every one of the 1,024 is a `b` or a `bl`.**
//!
//! Two things about that measurement are worth keeping, because both are places
//! a lane has already gone wrong here:
//!
//! 1. **`add3` cannot detect this.** Its bodies are `mullw`/`add`/`blr` — a
//!    control with no relocated branch in it, run against a claim that only
//!    relocated branches violate. That was recorded as the twelfth instance of
//!    absence-read-as-success in this project (§9.1). [`CodListing`]'s standing
//!    test therefore asserts, with its own distinct failure message, that the
//!    fixture it runs on **contains** at least one relocated branch.
//! 2. **Non-branch relocations do *not* differ**, and that is the interesting
//!    half. A data-address row (`lwz r31,?g_i@@3HA(r11)` = `83eb0000`) carries a
//!    relocation too, but c2 leaves the displacement field **0 in both** the obj
//!    and the listing — the linker fills it in. So the differing class is
//!    exactly `{b, bl}`, not "anything relocated", and a reader may trust every
//!    other row's bytes.
//!
//! # Format notes (all load-bearing, all bitten by at least once)
//!
//! * Lines are **CRLF**. Rust's `str::lines` strips the `\r`; a hand-rolled
//!   `split('\n')` would not.
//! * Offsets are **per COMDAT**, restarting at `00000` for every function — not
//!   file offsets and not `.text`-wide.
//! * `PROC NEAR` is separated from the symbol by a **space or a tab**, chosen
//!   for column alignment. A space-only pattern silently drops 5 of 7 functions
//!   on `il_call_perm.cpp` and then reports "all differing rows are branches"
//!   over the rows it did not read — the same failure shape as (1) above.

/// One decoded instruction row of a listing: `  00014\t48000001\t bl   ?f@@…`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodRow {
    /// Offset **within the function's COMDAT**, not within `.text`.
    pub offset: u32,
    /// The 32-bit word as the listing prints it — canonical/unrelocated at a
    /// `b`/`bl`, identical to the obj everywhere else.
    pub word: u32,
    /// Everything after the word: mnemonic, operands and any trailing comment.
    pub asm: String,
}

impl CodRow {
    /// The mnemonic (first whitespace-delimited token of [`CodRow::asm`]).
    pub fn mnemonic(&self) -> &str {
        self.asm.split_whitespace().next().unwrap_or("")
    }

    /// True iff this row is one of the two shapes whose listing word is
    /// **canonical rather than emitted** — see the module docs.
    pub fn is_relocated_branch(&self) -> bool {
        matches!(self.mnemonic(), "b" | "bl")
            && matches!(self.word, 0x4800_0000 | 0x4800_0001)
            && self.operands().starts_with(|c: char| c == '?' || c == '_' || c == '|')
    }

    /// The operand text: [`CodRow::asm`] minus the mnemonic and minus any
    /// trailing `;` comment, trimmed.
    pub fn operands(&self) -> &str {
        let rest = match self.asm.split_once(char::is_whitespace) {
            Some((_, r)) => r,
            None => "",
        };
        let rest = match rest.split_once(';') {
            Some((l, _)) => l,
            None => rest,
        };
        rest.trim()
    }
}

/// One `PROC NEAR` … `ENDP` block.
#[derive(Clone, Debug)]
pub struct CodFunction {
    /// The decorated (mangled) symbol — the same spelling the obj's `.text`
    /// COMDAT leader carries. This is what makes the listing a **second,
    /// name-carrying source** for the emitted census (board #136).
    pub name: String,
    /// Instruction rows in listing (= emission) order.
    pub rows: Vec<CodRow>,
    /// Raw listing lines of this block, kept so annotation readers (`/QXSTALLS`,
    /// board #134) do not need a second pass over the file.
    pub lines: Vec<String>,
}

/// A parsed `.cod`.
#[derive(Clone, Debug, Default)]
pub struct CodListing {
    /// `PROC NEAR` blocks in listing order.
    pub functions: Vec<CodFunction>,
    /// Every `PUBLIC <name>` declaration, in order, **including** the two
    /// `__C1_11886` / `__C2_11886` build-stamp symbols — filtering is the
    /// caller's decision, not the reader's.
    pub publics: Vec<String>,
}

/// True for the `.XBLD$W` build-stamp publics c2 emits in every TU
/// (`__C1_11886`, `__C2_11886`). They are `PUBLIC` but never `PROC`, and they
/// are not `.text` COMDATs, so a census reconciliation must exclude them.
pub fn is_build_stamp(name: &str) -> bool {
    name.starts_with("__C1_") || name.starts_with("__C2_")
}

impl CodListing {
    /// Parse a listing. Never fails: anything unrecognized is ignored, because
    /// this is a decode aid and a parse error must not be able to fail a gate.
    /// Under-reading *is* a hazard, though, so [`CodListing::functions`] is
    /// reconciled against the obj by the caller (board #136) rather than
    /// trusted.
    pub fn parse(text: &str) -> CodListing {
        let mut out = CodListing::default();
        let mut cur: Option<CodFunction> = None;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("PUBLIC\t") {
                let name = rest.split(['\t', ';']).next().unwrap_or("").trim();
                if !name.is_empty() {
                    out.publics.push(name.to_string());
                }
            }
            if let Some(name) = parse_proc(line) {
                if let Some(f) = cur.take() {
                    out.functions.push(f);
                }
                cur = Some(CodFunction {
                    name,
                    rows: Vec::new(),
                    lines: Vec::new(),
                });
                continue;
            }
            if let Some(name) = parse_endp(line) {
                if let Some(f) = cur.take() {
                    debug_assert_eq!(f.name, name);
                    out.functions.push(f);
                }
                continue;
            }
            if let Some(f) = cur.as_mut() {
                f.lines.push(line.to_string());
                if let Some(row) = parse_row(line) {
                    f.rows.push(row);
                }
            }
        }
        if let Some(f) = cur.take() {
            out.functions.push(f);
        }
        out
    }

    /// The `PROC` names, in listing order. The census-relevant set.
    pub fn proc_names(&self) -> Vec<String> {
        self.functions.iter().map(|f| f.name.clone()).collect()
    }
}

/// `<name>` from `<name>[ \t]+PROC NEAR…`. Both separators occur — c2 picks
/// whichever lands the `PROC` on its column.
fn parse_proc(line: &str) -> Option<String> {
    let idx = line.find("PROC NEAR")?;
    let head = &line[..idx];
    let name = head.trim_end();
    if name.is_empty() || name.len() == head.len() {
        return None; // no separator: this is `EXTRN x:PROC NEAR`-ish, not a def
    }
    if name.contains(char::is_whitespace) || name.starts_with(';') {
        return None;
    }
    Some(name.to_string())
}

/// `<name>` from `<name> ENDP`.
fn parse_endp(line: &str) -> Option<String> {
    let head = line.strip_suffix(" ENDP")?;
    if head.is_empty() || head.contains(char::is_whitespace) {
        return None;
    }
    Some(head.to_string())
}

/// `  00014\t48000001\t bl   ?f@@…` → row. Two leading spaces, tab-separated.
fn parse_row(line: &str) -> Option<CodRow> {
    let rest = line.strip_prefix("  ")?;
    let (off_s, rest) = rest.split_once('\t')?;
    let (word_s, asm) = rest.split_once('\t')?;
    if off_s.len() != 5 || word_s.len() != 8 {
        return None;
    }
    let offset = u32::from_str_radix(off_s, 16).ok()?;
    let word = u32::from_str_radix(word_s, 16).ok()?;
    Some(CodRow {
        offset,
        word,
        asm: asm.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact bytes c2 writes (CRLF, tab-aligned, both `PROC` separators).
    const SAMPLE: &str = concat!(
        "; Listing generated by Microsoft (R) Optimizing Compiler\r\n",
        "PUBLIC\t__C2_11886\r\n",
        "PUBLIC\t?pass2@@YAHHH@Z\t\t\t\t\t; pass2\r\n",
        "EXTRN\t?g2@@YAHHH@Z:PROC\t\t\t\t; g2\r\n",
        "?pass2@@YAHHH@Z\tPROC NEAR\t\t\t\t; pass2, COMDAT\r\n",
        "  00000\t48000000\t b            ?g2@@YAHHH@Z\r\n",
        "?pass2@@YAHHH@Z ENDP\r\n",
        "PUBLIC\t?chain@@YAHH@Z\t\t\t\t; chain\r\n",
        "?chain@@YAHH@Z PROC NEAR\t\t\t\t; chain, COMDAT\r\n",
        "  00000\t7d8802a6\t mflr         r12\r\n",
        "  00004\t48000001\t bl           ?void_func@@YAXH@Z\r\n",
        "  00008\t83eb0000\t lwz          r31,?g_i@@3HA(r11)\r\n",
        "?chain@@YAHH@Z ENDP\r\n",
        "END\r\n",
    );

    #[test]
    fn both_proc_separators_are_read() {
        let l = CodListing::parse(SAMPLE);
        assert_eq!(
            l.proc_names(),
            vec!["?pass2@@YAHHH@Z".to_string(), "?chain@@YAHH@Z".to_string()],
            "a space-only `PROC NEAR` pattern drops the tab-aligned definitions"
        );
    }

    #[test]
    fn rows_decode_offset_word_and_mnemonic() {
        let l = CodListing::parse(SAMPLE);
        let f = &l.functions[1];
        assert_eq!(f.rows.len(), 3);
        assert_eq!(f.rows[0].offset, 0);
        assert_eq!(f.rows[0].word, 0x7d88_02a6);
        assert_eq!(f.rows[0].mnemonic(), "mflr");
        assert_eq!(f.rows[1].operands(), "?void_func@@YAXH@Z");
    }

    /// The whole point of the module: `bl` is canonical, the data-address row
    /// is not — and treating "carries a relocation" as the class would wrongly
    /// distrust the `lwz`.
    #[test]
    fn only_branches_are_canonical_not_every_relocated_row() {
        let l = CodListing::parse(SAMPLE);
        let f = &l.functions[1];
        assert!(f.rows[1].is_relocated_branch(), "bl must be canonical");
        assert!(
            !f.rows[2].is_relocated_branch(),
            "a data-address row carries a relocation but its bytes are real"
        );
        assert!(l.functions[0].rows[0].is_relocated_branch(), "b is canonical");
    }

    #[test]
    fn publics_include_the_build_stamps_and_the_caller_filters_them() {
        let l = CodListing::parse(SAMPLE);
        assert_eq!(l.publics.len(), 3);
        assert_eq!(l.publics.iter().filter(|n| is_build_stamp(n)).count(), 1);
    }

    /// An `EXTRN …:PROC` declaration is not a definition. If it were read as
    /// one, every TU's `PROC` set would be inflated by its callees and #136's
    /// totality residue would be nonsense.
    #[test]
    fn extrn_declarations_are_not_definitions() {
        let l = CodListing::parse(SAMPLE);
        assert!(
            !l.proc_names().iter().any(|n| n.contains("?g2@@")),
            "EXTRN ?g2@@YAHHH@Z:PROC was read as a definition"
        );
    }
}
