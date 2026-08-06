//! COFF object handling for the differential compare.
//!
//! The oracle criterion is `port(IL) == c2(IL)` byte-exact — **except** the
//! 4-byte COFF `TimeDateStamp` at file offset 4..8, which is the only field
//! that varies between otherwise-identical rebuilds of the same source. So the
//! comparison always runs on *normalized* bytes with those four zeroed.
//!
//! (Reference: `msvc-src/docs/IL_CHANNEL_PROBE.md` — COFF TimeDateStamp
//! determinism note, offset 4-7. Verified empirically: identical source →
//! byte-identical `.obj` apart from those four bytes.)

mod reloc;
pub use reloc::*;

/// A COFF `.obj` image: just its raw bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjImage(pub Vec<u8>);

/// Result of comparing two [`ObjImage`]s on their normalized bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjDiff {
    /// Normalized bytes are identical.
    Identical,
    /// Normalized bytes differ. `first_offset` is the first differing byte
    /// index (or `min(a_len, b_len)` when one is a prefix of the other).
    Differs {
        first_offset: usize,
        a_len: usize,
        b_len: usize,
    },
}

/// Byte offset of the COFF `TimeDateStamp` field.
const TIMESTAMP_OFFSET: usize = 4;
const TIMESTAMP_END: usize = 8;

/// COFF file header width, section-header width, symbol-record width.
const COFF_HEADER_LEN: usize = 20;
const SECTION_HEADER_LEN: usize = 40;
const SYMBOL_LEN: usize = 18;
/// `IMAGE_SCN_LNK_COMDAT`.
const IMAGE_SCN_LNK_COMDAT: u32 = 0x0000_1000;
/// `IMAGE_SYM_CLASS_STATIC`.
const IMAGE_SYM_CLASS_STATIC: u8 = 3;
/// Emitted code lands in `.text` and its `$`-suffixed variants (`.text$yd`
/// carries the dynamic-initializer thunks).
const TEXT_SECTION_PREFIX: &str = ".text";

/// Is this symbol name one of c2's **compiler labels** — `$M<digits>` or
/// `$T<digits>`?
///
/// The digit check is not decoration. `$M` and `$T` are also legal leading
/// characters of a mangled name, and a rule that matched the two-character
/// prefix alone would count a user symbol as a compiler label and report a
/// counter where there is none. Anything else beginning `$` (there is none in
/// this workload) is **not** claimed: a reader that guessed here would be worse
/// than one that did not read at all.
fn is_compiler_label(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("$M").or_else(|| name.strip_prefix("$T")) else {
        return false;
    };
    !rest.is_empty() && rest.bytes().all(|c| c.is_ascii_digit())
}

/// A NUL-terminated string at byte offset `i` of the COFF string table.
fn str_at(strtab: &[u8], i: usize) -> Option<String> {
    let s = strtab.get(i..)?;
    let e = s.iter().position(|&c| c == 0)?;
    Some(String::from_utf8_lossy(&s[..e]).into_owned())
}

/// **The one section-name decoder.** `o` is the section header's offset.
///
/// Two forms, and the second is the one a re-implementation forgets: an 8-byte
/// field holding the name directly (NUL-padded, and *not* NUL-terminated when it
/// fills all eight), or `/<decimal offset>` pointing into the string table for a
/// name longer than eight bytes. `.debug$S`, `.text$yd` and `.CRT$XCU` are all
/// exactly eight or fewer bytes, so the workload happens not to exercise the
/// second form — which is precisely why a second reader that dropped it would
/// look right (`ROADMAP.md` §10.14).
fn section_name_at(b: &[u8], o: usize, strtab: &[u8]) -> Option<String> {
    let raw = b.get(o..o + 8)?;
    if raw[0] == b'/' {
        let digits = String::from_utf8_lossy(&raw[1..]);
        str_at(strtab, digits.trim_end_matches('\0').trim().parse::<usize>().ok()?)
    } else {
        Some(String::from_utf8_lossy(raw).trim_end_matches('\0').to_owned())
    }
}

impl ObjImage {
    pub fn new(bytes: Vec<u8>) -> Self {
        ObjImage(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The raw COFF `TimeDateStamp` (LE u32 at offset 4), or `None` if the
    /// image is too short to contain it.
    pub fn timestamp(&self) -> Option<u32> {
        if self.0.len() >= TIMESTAMP_END {
            Some(u32::from_le_bytes([
                self.0[4], self.0[5], self.0[6], self.0[7],
            ]))
        } else {
            None
        }
    }

    /// A clone of the bytes with the 4-byte `TimeDateStamp` zeroed. Guards the
    /// length: images shorter than 8 bytes are returned unchanged.
    pub fn normalized(&self) -> Vec<u8> {
        let mut v = self.0.clone();
        if v.len() >= TIMESTAMP_END {
            for b in &mut v[TIMESTAMP_OFFSET..TIMESTAMP_END] {
                *b = 0;
            }
        }
        v
    }

    /// **The emitted-function set**: the leader symbol of every `.text*` COMDAT
    /// section, in section order.
    ///
    /// This is the denominator of `docs/GAPS.md` §8 — *what c2 actually
    /// compiled*, as opposed to what its input IL contained. Under `/Gy` (which
    /// `/O1` implies, and the workload compiles with) c2 puts each emitted
    /// function in its own COMDAT `.text` section, so the count of those
    /// sections is the count of emitted functions and each one's leader symbol
    /// is that function's mangled name.
    ///
    /// **Section-led, not symbol-led, and that is the whole reason it is
    /// correct.** A COMDAT `.text` section carries more than one symbol: on
    /// `src/App.cpp` its 158 sections hold 372 symbols, the surplus being
    /// `__unwind$NNNNNN` labels that are `IMAGE_SYM_CLASS_EXTERNAL` and
    /// `DT_FUNCTION` exactly like the real ones. Counting symbols there reads
    /// **372 emitted functions** where c2 emitted 158 — a 2.35× over-count that
    /// no invariant downstream would have caught, because both numbers are
    /// plausible. Walking sections and taking each one's *first non-definition*
    /// symbol cannot make that mistake: a section has exactly one leader.
    ///
    /// The section-definition symbol itself (`IMAGE_SYM_CLASS_STATIC` carrying
    /// one aux record — the COMDAT selection record) is skipped; it repeats the
    /// section name, not the function's.
    ///
    /// **Fail-closed.** `None` whenever the headers do not decode: a short
    /// image, a symbol table or string table off the end, or a section header
    /// running past EOF. A partially-read symbol table would return a *shorter*
    /// emitted set, and a short denominator inflates every ratio computed
    /// against it — so there is no partial answer here.
    pub fn text_comdat_functions(&self) -> Option<Vec<String>> {
        Some(
            self.text_comdat_entries()?
                .into_iter()
                .map(|(n, _)| n)
                .collect(),
        )
    }

    /// [`ObjImage::text_comdat_functions`] with each function's **emitted
    /// bytes** — the section's raw data, in section order.
    ///
    /// Added for the listing seam (board #132): comparing c2's `.cod` rows
    /// against what it actually wrote needs the bytes *per COMDAT*, because the
    /// listing's offsets restart at `00000` for every function and are not
    /// `.text`-wide. Same fail-closed contract as the names-only form.
    pub fn text_comdat_functions_with_bytes(&self) -> Option<Vec<(String, Vec<u8>)>> {
        let b = &self.0;
        let mut out = Vec::new();
        for (name, s) in self.text_comdat_entries()? {
            let o = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
            let size = u32::from_le_bytes([b[o + 16], b[o + 17], b[o + 18], b[o + 19]]) as usize;
            let ptr = u32::from_le_bytes([b[o + 20], b[o + 21], b[o + 22], b[o + 23]]) as usize;
            // A section whose raw data runs off the end is a decode failure, not
            // a short answer — the same reason the names walk is fail-closed.
            let end = ptr.checked_add(size)?;
            if end > b.len() {
                return None;
            }
            out.push((name, b[ptr..end].to_vec()));
        }
        Some(out)
    }

    /// [`ObjImage::text_comdat_functions`] with each function's **relocation
    /// count**, in section order.
    ///
    /// Added for FUNCTION BYTE MATCH (board #320, `docs/FUNCTION_BYTE_MATCH.md`).
    /// FBM compares a COMDAT's *raw bytes*, and a `.text` section's raw bytes do
    /// not contain its relocations: two functions that load the address of two
    /// different globals have **byte-identical** text and different relocation
    /// records. So "the bytes match" is strictly weaker than "the function
    /// matches" on any body that relocates, and the number of credited functions
    /// that relocate is the size of that gap. It is measured rather than
    /// argued.
    ///
    /// Same fail-closed contract as the other walks — `None` whenever the
    /// headers do not decode. `IMAGE_SCN_LNK_NRELOC_OVFL` is **not** unpacked
    /// here: a section carrying it returns `None` rather than the literal
    /// `0xFFFF`, because a count that is really a sentinel is exactly the shape
    /// that reads as a plausible number. [`ObjImage::relocations`] owns the
    /// overflow decode; no obj in this workload trips it.
    pub fn text_comdat_reloc_counts(&self) -> Option<Vec<(String, usize)>> {
        let b = &self.0;
        let mut out = Vec::new();
        for (name, s) in self.text_comdat_entries()? {
            let o = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
            let n = u16::from_le_bytes([b[o + 32], b[o + 33]]) as usize;
            let ptr = u32::from_le_bytes([b[o + 24], b[o + 25], b[o + 26], b[o + 27]]) as usize;
            if n == 0xFFFF {
                return None; // the overflow sentinel, not a count
            }
            if n != 0 && ptr == 0 {
                return None; // a count with no table is malformed, not empty
            }
            out.push((name, n));
        }
        Some(out)
    }

    /// [`ObjImage::text_comdat_reloc_counts`] with each relocation's **offset
    /// inside the COMDAT** and its packed type word, in section order and then
    /// in table order.
    ///
    /// Added for the diff-signature census (lane `w-bytes`, board #976). A
    /// mismatched instruction word that sits **under a relocation** is a
    /// different kind of finding from one that does not: the linker owns part of
    /// that word, so its immediate or displacement field is a filler c2 chose
    /// and the port chose differently, and the two can disagree in the obj while
    /// naming the same symbol. Counting the relocations, which
    /// [`ObjImage::text_comdat_reloc_counts`] already does, cannot say *which
    /// word* — and "which word" is the whole question when the unit is a 4-byte
    /// instruction.
    ///
    /// Under `/Gy` every function starts at offset 0 of its own section, so a
    /// record's `VirtualAddress` **is** its offset within the body's bytes and
    /// no section-base subtraction is needed. That is a property of the COMDAT
    /// population this walk is defined over, not a general COFF fact.
    ///
    /// Same fail-closed contract as every other walk here: `None` the moment
    /// anything does not decode, never a short list — a short relocation list
    /// would read as "this body relocates less than it does", which is
    /// absence-as-success in its most direct form.
    pub fn text_comdat_reloc_sites(&self) -> Option<Vec<(String, Vec<(u32, u16)>)>> {
        // Reuses the whole-image walk rather than re-reading the tables: the
        // overflow sentinel, the malformed-header cases and the record layout
        // are decided in exactly one place.
        let all = self.relocations()?;
        let mut out = Vec::new();
        for (name, s) in self.text_comdat_entries()? {
            let mut v: Vec<(u32, u16)> = all
                .iter()
                .filter(|r| r.section == s)
                .map(|r| (r.va, r.ty))
                .collect();
            v.sort_unstable();
            out.push((name, v));
        }
        Some(out)
    }

    /// **Which symbol each `.text` COMDAT CALLS**, by name, in offset order —
    /// the `REL24` targets and nothing else.
    ///
    /// [`ObjImage::text_comdat_reloc_sites`] answers *"is this word one the
    /// linker owns"*; this answers *"and what does it point at"*. The two are
    /// different questions and only the second can tell two byte-identical
    /// branch words apart.
    ///
    /// # Why a byte compare cannot answer this
    ///
    /// Under `/Gy` a call to a symbol outside the COMDAT is emitted with a
    /// placeholder displacement of **`-(offset of the branch word)`**, i.e. it
    /// points at offset 0 of the section it sits in, for *every* callee. So two
    /// bodies that call two entirely different functions from the same word
    /// index carry the **same four bytes**. Lane `w-drop3` measured the
    /// consequence on the workload: 140 bodies where the port emits
    /// `bl ??$Obj@…@DataNode@@…` and c2 emits `bl ?GetObj@DataNode@@…` at the
    /// same offset, both `4bffffe5`, and the byte instrument scored the word
    /// **equal**. Board **#882**'s "4,664 credited functions carry a relocation
    /// FBM does not check" is that gap, and this reader is what lets it be
    /// counted rather than restated.
    ///
    /// `PAIR` records are excluded by construction — [`Reloc::sym`] is a
    /// displacement rather than a symbol index on those (rev 6.0), so naming one
    /// would be reading a number as an index. Every other base type is excluded
    /// too: the question here is *calls*, and widening it to data references
    /// would mix a call graph with an address-taken set.
    ///
    /// Same fail-closed contract as every other walk: `None` the moment
    /// anything does not decode — a symbol index past the table, an aux slot, a
    /// long name whose string-table offset does not resolve. A **short** target
    /// list is the dangerous answer, because it reads as "this body calls
    /// fewer things than it does", which is absence-as-success.
    pub fn text_comdat_call_targets(&self) -> Option<Vec<(String, Vec<(u32, String)>)>> {
        let b = &self.0;
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
                    .trim_end_matches('\0')
                    .to_owned()
            };
            names[i] = Some(name);
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        let all = self.relocations()?;
        let mut out = Vec::new();
        for (name, s) in self.text_comdat_entries()? {
            let mut v: Vec<(u32, String)> = Vec::new();
            for r in all.iter().filter(|r| r.section == s) {
                if r.base() != crate::reloc::IMAGE_REL_PPC_REL24 {
                    continue;
                }
                let idx = usize::try_from(r.sym).ok()?;
                let target = names.get(idx)?.clone()?;
                v.push((r.va, target));
            }
            v.sort_unstable();
            out.push((name, v));
        }
        Some(out)
    }

    /// **Every compiler-label symbol (`$M<n>` / `$T<n>`) in the obj**, in
    /// symbol-table order — the *only* channel through which the value of c2's
    /// compiler-label counter reaches an object file.
    ///
    /// # Why this is worth a reader of its own
    ///
    /// `coff::plan_labels` mints a `$M`/`$M`/`$T` triple for a function with a
    /// frame and **nothing at all** for a leaf. Lane `w-loop` measured c2
    /// agreeing, over 34 leaf-only TUs across 17 control-flow shapes: an obj
    /// whose every function is a leaf carries **zero** of these symbols, and 28
    /// of those 34 contain a backward intra-section branch. The control — the
    /// same 17 bodies each followed by one framed function — minted a triple
    /// **17 of 17** (`work/w-loop/loopcost.py`, `--q2`).
    ///
    /// That matters because the port's standing refusal of a **backward**
    /// branch (`c2-core`'s `codegen::labels`, invariant 4) is justified entirely
    /// by *"the obj would carry a wrong `$M`"* — a leaf loop charges the counter
    /// **+1 to +4** and `plan_labels` charges 0. The refusal is right, and its
    /// stated consequence is **conditional on this list being non-empty**. So
    /// the list is a per-TU, oracle-side fact worth printing beside the
    /// CFG-reachability screen: it says which loop-blocked TUs could ever be
    /// hurt by the counter and which could not.
    ///
    /// **A fact about the reference obj, never a licence.** It is read off
    /// c2's own output, it moves no numerator, and it appears in no accept path
    /// — an emitter that consulted it would be grading itself on the answer.
    ///
    /// Names only, deliberately: the *numbers* are a TU-level running counter
    /// whose seed is not in the obj, so a caller that compared them across two
    /// TUs would be comparing two coordinate systems. Same fail-closed contract
    /// as the other walks — `None` whenever the headers do not decode, never a
    /// short list.
    pub fn compiler_label_symbols(&self) -> Option<Vec<String>> {
        let b = &self.0;
        let (_, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            let name = if b[o..o + 4] == [0, 0, 0, 0] {
                // A `$M2545` is six bytes and always fits the inline field, so
                // a long-name indirection here is not one of ours. It is still
                // decoded rather than skipped: a reader that silently dropped a
                // symbol it could not name would under-report, and under-report
                // is the direction that reads as "no labels".
                let at = u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
                str_at(strtab, at)?
            } else {
                String::from_utf8_lossy(&b[o..o + 8])
                    .trim_end_matches('\0')
                    .to_owned()
            };
            if is_compiler_label(&name) {
                out.push(name);
            }
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        Some(out)
    }

    /// **The obj's section-name list**, in section order, decoded by the same
    /// code path [`ObjImage::text_comdat_functions`] uses.
    ///
    /// This is **factor C**'s input (`docs/ROADMAP.md` §10.19): the port's COFF
    /// writer can emit six section names, so a TU whose obj carries a seventh is
    /// out of reach of the writer however good the codegen becomes. `gap.rs`
    /// turns this list into the `emit-sec-*` keys.
    ///
    /// **It lives here rather than in the harness because the name decode is
    /// already here.** `/NNN` is a string-table indirection, `$`-suffixed names
    /// are ordinary names, and a section name is *not* NUL-terminated when it
    /// fills all 8 bytes — three chances for a second reader to disagree with
    /// this one, and `ROADMAP.md` §10.14 is the record of exactly that costing a
    /// session. [`section_name_at`] is the single decoder; both walks call it.
    ///
    /// Duplicates are kept: an obj carries one `.text` COMDAT per emitted
    /// function under `/Gy`, and the caller wanting a *set* is the caller that
    /// says so. Same fail-closed contract as the names-only COMDAT walk — `None`
    /// whenever the headers do not decode, never a short list.
    pub fn section_names(&self) -> Option<Vec<String>> {
        let b = &self.0;
        let (nsec, sym_end) = self.coff_layout()?;
        let strtab = &b[sym_end..];
        (0..nsec)
            .map(|i| section_name_at(b, COFF_HEADER_LEN + i * SECTION_HEADER_LEN, strtab))
            .collect()
    }

    /// The header-bounds check both walks need: `(section count, symbol-table
    /// end == string-table start)`, or `None` when anything is off the end.
    fn coff_layout(&self) -> Option<(usize, usize)> {
        let b = &self.0;
        if b.len() < COFF_HEADER_LEN {
            return None;
        }
        let nsec = u16::from_le_bytes([b[2], b[3]]) as usize;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        // Section headers, the symbol table and the string-table size word must
        // all be inside the image before anything is decoded.
        let sec_end = COFF_HEADER_LEN.checked_add(nsec.checked_mul(SECTION_HEADER_LEN)?)?;
        let sym_end = psym.checked_add(nsym.checked_mul(SYMBOL_LEN)?)?;
        if sec_end > b.len() || psym < sec_end || sym_end.checked_add(4)? > b.len() {
            return None;
        }
        Some((nsec, sym_end))
    }

    /// The shared walk: `(leader symbol, section index)` for every COMDAT
    /// `.text*` section, in section order.
    fn text_comdat_entries(&self) -> Option<Vec<(String, usize)>> {
        let b = &self.0;
        let (nsec, sym_end) = self.coff_layout()?;
        let psym = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
        let nsym = u32::from_le_bytes([b[12], b[13], b[14], b[15]]) as usize;
        let strtab = &b[sym_end..];
        let str_at = |i: usize| -> Option<String> { str_at(strtab, i) };
        // Which sections are COMDAT `.text`?
        let mut is_text = vec![false; nsec];
        for (i, flag) in is_text.iter_mut().enumerate() {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            let name = section_name_at(b, o, strtab)?;
            let chars = u32::from_le_bytes([b[o + 36], b[o + 37], b[o + 38], b[o + 39]]);
            *flag = name.starts_with(TEXT_SECTION_PREFIX) && chars & IMAGE_SCN_LNK_COMDAT != 0;
        }
        let mut claimed = vec![false; nsec];
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < nsym {
            let o = psym + i * SYMBOL_LEN;
            let naux = b[o + 17] as usize;
            let secnum = i16::from_le_bytes([b[o + 12], b[o + 13]]);
            let sclass = b[o + 16];
            if secnum >= 1 && (secnum as usize) <= nsec {
                let s = secnum as usize - 1;
                let is_section_definition = sclass == IMAGE_SYM_CLASS_STATIC && naux == 1;
                if is_text[s] && !claimed[s] && !is_section_definition {
                    let name = if b[o..o + 4] == [0, 0, 0, 0] {
                        let at =
                            u32::from_le_bytes([b[o + 4], b[o + 5], b[o + 6], b[o + 7]]) as usize;
                        str_at(at)?
                    } else {
                        String::from_utf8_lossy(&b[o..o + 8])
                            .trim_end_matches('\0')
                            .to_owned()
                    };
                    claimed[s] = true;
                    out.push((name, s));
                }
            }
            // An aux record count that walks past the table is a decode failure,
            // not something to clamp: clamping would silently shorten the answer.
            i = i.checked_add(1)?.checked_add(naux)?;
            if i > nsym {
                return None;
            }
        }
        // **One leader per COMDAT `.text` section, or no answer.** Under `/Gy`
        // every emitted function gets its own section and every such section
        // gets a leader, so a section that produced none means the symbol walk
        // went wrong — and a short emitted set is worse than none, because it is
        // a denominator that silently inflates every ratio computed against it.
        // Measured across the 871 capturable workload objs: 0 refusals.
        if claimed.iter().zip(&is_text).any(|(&c, &t)| t && !c) {
            return None;
        }
        Some(out)
    }

    /// Compare two images on their normalized (timestamp-zeroed) bytes.
    pub fn diff(a: &ObjImage, b: &ObjImage) -> ObjDiff {
        let na = a.normalized();
        let nb = b.normalized();
        let common = na.len().min(nb.len());
        for i in 0..common {
            if na[i] != nb[i] {
                return ObjDiff::Differs {
                    first_offset: i,
                    a_len: na.len(),
                    b_len: nb.len(),
                };
            }
        }
        if na.len() != nb.len() {
            return ObjDiff::Differs {
                first_offset: common,
                a_len: na.len(),
                b_len: nb.len(),
            };
        }
        ObjDiff::Identical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_obj() -> Vec<u8> {
        // A plausible-ish COFF header prefix: machine word (POWERPCBE 0x01F2),
        // section count, then a timestamp, then arbitrary payload.
        let mut v = vec![0xF2, 0x01, 0x03, 0x00]; // machine + nsections
        v.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // TimeDateStamp
        v.extend_from_slice(&[0x10, 0x20, 0x30, 0x40, 0x50, 0x60]); // payload
        v
    }

    #[test]
    fn timestamp_reads_offset_4_le() {
        let obj = ObjImage::new(base_obj());
        assert_eq!(obj.timestamp(), Some(0xDDCCBBAA));
        assert_eq!(ObjImage::new(vec![0, 1, 2]).timestamp(), None);
    }

    #[test]
    fn normalized_zeroes_timestamp_only() {
        let obj = ObjImage::new(base_obj());
        let n = obj.normalized();
        assert_eq!(&n[0..4], &[0xF2, 0x01, 0x03, 0x00]);
        assert_eq!(&n[4..8], &[0, 0, 0, 0]);
        assert_eq!(&n[8..], &[0x10, 0x20, 0x30, 0x40, 0x50, 0x60]);
        // Original untouched.
        assert_eq!(&obj.as_bytes()[4..8], &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn differ_only_in_timestamp_compares_identical() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb[4] = 0x11;
        bb[5] = 0x22;
        bb[6] = 0x33;
        bb[7] = 0x44;
        let b = ObjImage::new(bb);
        assert_eq!(ObjImage::diff(&a, &b), ObjDiff::Identical);
    }

    #[test]
    fn difference_elsewhere_reports_offset() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb[9] = 0x99; // offset 9 is in the payload
        let b = ObjImage::new(bb);
        match ObjImage::diff(&a, &b) {
            ObjDiff::Differs { first_offset, .. } => assert_eq!(first_offset, 9),
            ObjDiff::Identical => panic!("expected a difference at offset 9"),
        }
    }

    /// A synthetic COFF with `sections = [(name, comdat)]` and
    /// `symbols = [(name, section-1-based, class, naux)]`, names longer than 8
    /// bytes going to the string table exactly as a real obj does.
    fn coff(sections: &[(&str, bool)], symbols: &[(&str, i16, u8, u8)]) -> Vec<u8> {
        let nsec = sections.len();
        let nsym: usize = symbols.iter().map(|s| 1 + s.3 as usize).sum();
        let psym = COFF_HEADER_LEN + nsec * SECTION_HEADER_LEN;
        let mut head = vec![0u8; psym];
        head[0..2].copy_from_slice(&0x01F2u16.to_le_bytes()); // POWERPCBE
        head[2..4].copy_from_slice(&(nsec as u16).to_le_bytes());
        head[8..12].copy_from_slice(&(psym as u32).to_le_bytes());
        head[12..16].copy_from_slice(&(nsym as u32).to_le_bytes());
        // The string table starts with its own 4-byte size, so offset 4 is the
        // first name slot.
        let mut strtab: Vec<u8> = vec![0, 0, 0, 0];
        let intern = |s: &str, strtab: &mut Vec<u8>| -> u32 {
            let at = strtab.len() as u32;
            strtab.extend_from_slice(s.as_bytes());
            strtab.push(0);
            at
        };
        for (i, (name, comdat)) in sections.iter().enumerate() {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            if name.len() <= 8 {
                head[o..o + name.len()].copy_from_slice(name.as_bytes());
            } else {
                let at = intern(name, &mut strtab);
                let s = format!("/{at}");
                head[o..o + s.len()].copy_from_slice(s.as_bytes());
            }
            let chars = if *comdat { IMAGE_SCN_LNK_COMDAT } else { 0 };
            head[o + 36..o + 40].copy_from_slice(&chars.to_le_bytes());
        }
        let mut syms = Vec::new();
        for (name, secnum, sclass, naux) in symbols {
            let mut rec = [0u8; SYMBOL_LEN];
            if name.len() <= 8 {
                rec[..name.len()].copy_from_slice(name.as_bytes());
            } else {
                let at = intern(name, &mut strtab);
                rec[4..8].copy_from_slice(&at.to_le_bytes());
            }
            rec[12..14].copy_from_slice(&secnum.to_le_bytes());
            rec[16] = *sclass;
            rec[17] = *naux;
            syms.extend_from_slice(&rec);
            syms.extend(std::iter::repeat(0u8).take(*naux as usize * SYMBOL_LEN));
        }
        let n = strtab.len() as u32;
        strtab[0..4].copy_from_slice(&n.to_le_bytes());
        let mut out = head;
        out.extend_from_slice(&syms);
        out.extend_from_slice(&strtab);
        out
    }

    /// The realistic shape: two COMDAT `.text` sections, each carrying its
    /// section-definition symbol, its function leader, and an `__unwind$` label
    /// that looks exactly like a function to a symbol-led count.
    fn workload_shaped_obj() -> Vec<u8> {
        coff(
            &[(".text", true), (".text$yd", true), (".data", false)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
                ("__unwind$1", 1, 2, 0),
                (".text$yd", 2, IMAGE_SYM_CLASS_STATIC, 1),
                ("??__Egs@@YAXXZ", 2, 2, 0),
                (".data", 3, IMAGE_SYM_CLASS_STATIC, 1),
                ("?gv@@3HA", 3, 2, 0),
            ],
        )
    }

    /// [`coff`] plus a relocation table for one section: `(section index
    /// 0-based, records)` where each record is `(va, symbol slot, packed type)`.
    /// The records are appended past the string table and the section header's
    /// `PointerToRelocations` / `NumberOfRelocations` are patched to point at
    /// them, which is where a real obj puts them relative to the parts [`coff`]
    /// already writes.
    fn coff_with_relocs(
        sections: &[(&str, bool)],
        symbols: &[(&str, i16, u8, u8)],
        sec: usize,
        records: &[(u32, u32, u16)],
    ) -> Vec<u8> {
        let mut out = coff(sections, symbols);
        let ptr = out.len() as u32;
        for (va, sym, ty) in records {
            out.extend_from_slice(&va.to_le_bytes());
            out.extend_from_slice(&sym.to_le_bytes());
            out.extend_from_slice(&ty.to_le_bytes());
        }
        let o = COFF_HEADER_LEN + sec * SECTION_HEADER_LEN;
        out[o + 24..o + 28].copy_from_slice(&ptr.to_le_bytes());
        out[o + 32..o + 34].copy_from_slice(&(records.len() as u16).to_le_bytes());
        out
    }

    /// Symbols for the call-target tests. A relocation's `SymbolTableIndex`
    /// names a **slot**, and an aux record occupies one, so the indices are
    /// written out here rather than counted at each call site:
    /// `.text` 0 (+1 aux) · `?caller` 2 · `?callee` 3 · `?other` 4 ·
    /// `?g` 5 (+1 aux, so slot 6 is the aux) · `.data` 7.
    const CALL_SYMS: &[(&str, i16, u8, u8)] = &[
        (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
        ("?caller@@YAXXZ", 1, 2, 0),
        ("?callee@@YAXXZ", 0, 2, 0),
        ("?other@@YAXXZ", 0, 2, 0),
        ("?g@@YAXXZ", 0, 2, 1),
        (".data", 2, IMAGE_SYM_CLASS_STATIC, 1),
    ];

    /// **The whole reason this reader exists** (board #984): under `/Gy` a call
    /// out of a COMDAT is emitted with the placeholder displacement
    /// `-(offset of the word)` whatever the callee is, so two branch words that
    /// call two different functions are byte-identical and only the relocation
    /// table can tell them apart.
    #[test]
    fn a_call_target_is_named_by_its_relocation_and_not_by_its_bytes() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 3, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![(
                "?caller@@YAXXZ".to_string(),
                vec![(0x0, "?callee@@YAXXZ".to_string())]
            )]),
            "the REL24 names slot 3, which is ?callee"
        );
    }

    /// Only `REL24`. A `REFHI`/`REFLO` pair is a *data* reference, and a `PAIR`
    /// record's `sym` field is a displacement rather than an index (rev 6.0), so
    /// naming one would be reading a number as a symbol. Both `PAIR`s here carry
    /// a displacement that is also a valid slot, which is the case a
    /// base-type-blind reader gets wrong.
    #[test]
    fn only_rel24_records_are_call_targets() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[
                (0x0, 4, crate::reloc::IMAGE_REL_PPC_REFHI),
                (0x0, 2, crate::reloc::IMAGE_REL_PPC_PAIR),
                (0x8, 4, crate::reloc::IMAGE_REL_PPC_REFLO),
                (0x8, 2, crate::reloc::IMAGE_REL_PPC_PAIR),
                (0xc, 5, crate::reloc::IMAGE_REL_PPC_REL24),
            ],
        ));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![(
                "?caller@@YAXXZ".to_string(),
                vec![(0xc, "?g@@YAXXZ".to_string())]
            )]),
            "four data-reference records must contribute no call target"
        );
    }

    /// A flag bit on the packed type word must not hide the record: `REL24 |
    /// BRTAKEN` is `0x0206` and is still a call.
    #[test]
    fn a_modifier_bit_does_not_hide_a_call_target() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(
                0x4,
                3,
                crate::reloc::IMAGE_REL_PPC_REL24 | crate::reloc::IMAGE_REL_PPC_BRTAKEN,
            )],
        ));
        assert_eq!(
            obj.text_comdat_call_targets().map(|v| v[0].1.len()),
            Some(1),
            "a whole-word type compare is the defect Reloc::base() exists to prevent"
        );
    }

    /// Fail-closed: a `SymbolTableIndex` past the table is `None` for the whole
    /// walk, never a short list. A short list reads as "this body calls fewer
    /// things than it does", which is absence-as-success.
    #[test]
    fn a_symbol_index_past_the_table_returns_no_answer_at_all() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 9999, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(obj.text_comdat_call_targets(), None);
    }

    /// …and so is a relocation naming an AUX slot, which carries no name.
    #[test]
    fn a_relocation_naming_an_aux_slot_returns_no_answer_at_all() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 6, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(obj.text_comdat_call_targets(), None);
    }

    /// A COMDAT with no relocations at all is an EMPTY target list, not an
    /// absent row: "calls nothing" and "did not decode" are different answers,
    /// and the caller distinguishes them.
    #[test]
    fn a_comdat_with_no_relocations_is_an_empty_list_and_not_an_absent_row() {
        let obj = ObjImage::new(coff(&[(".text", true), (".data", false)], CALL_SYMS));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![("?caller@@YAXXZ".to_string(), Vec::new())])
        );
    }

    #[test]
    fn the_emitted_set_is_one_leader_per_text_comdat_section() {
        let obj = ObjImage::new(workload_shaped_obj());
        assert_eq!(
            obj.text_comdat_functions(),
            Some(vec!["?f@@YAHH@Z".to_string(), "??__Egs@@YAXXZ".to_string()]),
            "expected exactly the two COMDAT .text leaders"
        );
    }

    /// The over-count this reader exists to avoid: `__unwind$1` is external and
    /// sits in a COMDAT `.text`, so a symbol-led count would report three.
    #[test]
    fn an_unwind_label_in_a_text_comdat_is_not_an_emitted_function() {
        let obj = ObjImage::new(workload_shaped_obj());
        let got = obj.text_comdat_functions().expect("headers decode");
        assert!(
            !got.iter().any(|n| n.starts_with("__unwind$")),
            "an __unwind$ label was counted as an emitted function: {got:?}"
        );
    }

    /// **Factor C's input** (`ROADMAP.md` §10.19): the section-name list, in
    /// order, with duplicates kept and non-COMDAT sections included. It is a
    /// *different* question from the emitted set — that walk takes COMDAT
    /// `.text` only, and a TU is out of the port writer's reach because of its
    /// `.data` or `.bss`, which the emitted set cannot see at all.
    #[test]
    fn the_section_name_list_is_every_section_in_order() {
        let obj = ObjImage::new(workload_shaped_obj());
        assert_eq!(
            obj.section_names(),
            Some(vec![".text".to_string(), ".text$yd".to_string(), ".data".to_string()]),
            "every section, in section order — including the non-COMDAT .data that \
             the emitted-set walk deliberately drops"
        );
    }

    /// **The compiler-label channel** (lane `w-loop`, board **#742**): the
    /// `$M`/`$T` short names, in symbol-table order, and nothing else.
    #[test]
    fn the_compiler_label_list_is_the_dollar_m_and_dollar_t_symbols_in_order() {
        let obj = ObjImage::new(coff(
            &[(".text", true), (".pdata", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
                ("$M2545", 1, 6, 0),
                ("$M2546", 1, 6, 0),
                (".pdata", 2, IMAGE_SYM_CLASS_STATIC, 1),
                ("$T2547", 2, IMAGE_SYM_CLASS_STATIC, 0),
            ],
        ));
        assert_eq!(
            obj.compiler_label_symbols(),
            Some(vec!["$M2545".into(), "$M2546".into(), "$T2547".into()]),
            "the triple a framed function mints, in symbol-table order"
        );
    }

    /// **The reading the whole instrument turns on**: a leaf-only obj is
    /// `label-free`, and it stays `label-free` when the leaf branches. c2 agrees
    /// over 34 leaf-only probe TUs across 17 control-flow shapes, 28 of them
    /// carrying a backward branch (`work/w-loop/loopcost.py --q2`); this pins
    /// the *reader's* half so a future change cannot make an obj look
    /// label-free by failing to walk it.
    #[test]
    fn a_leaf_only_obj_is_label_free() {
        let obj = ObjImage::new(coff(
            &[(".text", true), (".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?loop@@YAHH@Z", 1, 2, 0),
                (".text", 2, IMAGE_SYM_CLASS_STATIC, 1),
                ("?leaf@@YAHH@Z", 2, 2, 0),
            ],
        ));
        assert_eq!(obj.compiler_label_symbols(), Some(vec![]));
    }

    /// A user symbol whose mangled name starts `$M`/`$T` is **not** a compiler
    /// label. Matching the two-character prefix alone would report a counter
    /// where there is none, and the digit check is the whole difference — so it
    /// is tested rather than assumed.
    #[test]
    fn a_user_symbol_beginning_dollar_m_is_not_a_compiler_label() {
        assert!(is_compiler_label("$M2545"));
        assert!(is_compiler_label("$T2547"));
        assert!(!is_compiler_label("$M"), "no digits is not a label");
        assert!(!is_compiler_label("$Mangled"), "letters are not digits");
        assert!(!is_compiler_label("$T12a"), "one non-digit is enough");
        assert!(!is_compiler_label("$L2545"), "only $M and $T are claimed");
        assert!(!is_compiler_label("?f@@YAHH@Z"));
    }

    /// Fail-closed: an obj whose symbol table walks off the end returns `None`,
    /// never an empty list. `None` and `Some(vec![])` are the difference between
    /// *"could not read"* and *"label-free"*, and the scan prints them as two
    /// different rows for exactly that reason.
    #[test]
    fn an_undecodable_obj_is_none_and_not_an_empty_label_list() {
        let mut bytes = workload_shaped_obj();
        // Claim a symbol count far past the end of the image.
        bytes[12..16].copy_from_slice(&9_999_999u32.to_le_bytes());
        assert_eq!(ObjImage::new(bytes).compiler_label_symbols(), None);
    }

    /// The two decoders must agree about what a section is *called*, or factor C
    /// and the emitted set are computed over different objs. Held to a `/NNN`
    /// long name, which is the form a re-implementation forgets (`§10.14`).
    #[test]
    fn both_walks_share_one_section_name_decoder() {
        let long = ".text$averyverylongsectionname";
        let obj = ObjImage::new(coff(
            &[(long, true), (".rdata$r", false)],
            &[
                ("x", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        ));
        assert_eq!(
            obj.section_names(),
            Some(vec![long.to_string(), ".rdata$r".to_string()]),
            "a `/NNN` name must be looked up by the name walk too, not returned as `/4`"
        );
        assert_eq!(
            obj.text_comdat_functions(),
            Some(vec!["?f@@YAHH@Z".to_string()]),
            "control: the emitted-set walk resolves the same name to the same `.text*` \
             prefix — if these two ever disagree, C is measured on a different obj \
             than the emitted census"
        );
    }

    /// NEGATIVE CONTROL — an obj whose headers do not decode gives **no** section
    /// list, not an empty one. An empty list would read as "carries no section
    /// outside the writer's set", i.e. as *inside* factor C: absence reading as
    /// success, on the flattering side.
    #[test]
    fn a_short_image_has_no_section_list_rather_than_an_empty_one() {
        assert_eq!(ObjImage::new(vec![0u8; 12]).section_names(), None);
        let full = coff(
            &[(".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        );
        assert_eq!(
            ObjImage::new(full.clone()).section_names(),
            Some(vec![".text".to_string()]),
            "control: the intact image lists its one section — the truncation below \
             must be what changes the answer, not the obj's shape"
        );
        let truncated = ObjImage::new(full[..full.len() - 12].to_vec());
        assert_eq!(
            truncated.section_names(),
            None,
            "a string table running off the end must refuse, exactly as the emitted \
             walk does — same layout check, one implementation"
        );
    }

    #[test]
    fn a_non_comdat_text_section_is_not_counted() {
        let obj = ObjImage::new(coff(
            &[(".text", false)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        ));
        assert_eq!(
            obj.text_comdat_functions(),
            Some(vec![]),
            "a non-COMDAT .text has no per-function leader to take"
        );
    }

    #[test]
    fn a_long_section_name_resolves_through_the_string_table() {
        let obj = ObjImage::new(coff(
            &[(".text$averylongsectionname", true)],
            &[
                ("x", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        ));
        assert_eq!(
            obj.text_comdat_functions(),
            Some(vec!["?f@@YAHH@Z".to_string()]),
            "a `/NNN` section name must be looked up, not compared literally"
        );
    }

    /// NEGATIVE CONTROL — a truncated image has no partial answer. The guard's
    /// quantity (one COMDAT `.text` with one leader) is held fixed; only the
    /// image length moves.
    #[test]
    fn a_truncated_symbol_table_refuses_rather_than_shortening_the_set() {
        let full = coff(
            &[(".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        );
        assert_eq!(
            ObjImage::new(full.clone()).text_comdat_functions(),
            Some(vec!["?f@@YAHH@Z".to_string()]),
            "control: the intact image must bind its one leader"
        );
        let cut = full[..full.len() - 12].to_vec();
        assert_eq!(
            ObjImage::new(cut).text_comdat_functions(),
            None,
            "a truncated string table must refuse, not return a short emitted set"
        );
    }

    /// NEGATIVE CONTROL — an aux count that runs off the end of the symbol table
    /// is a decode failure, not something to clamp.
    #[test]
    fn an_aux_count_past_the_table_end_refuses() {
        let mut obj = coff(
            &[(".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
            ],
        );
        let psym = u32::from_le_bytes([obj[8], obj[9], obj[10], obj[11]]) as usize;
        // Second symbol claims 200 aux records; the leader is still there.
        obj[psym + 2 * SYMBOL_LEN + 17] = 200;
        assert_eq!(
            ObjImage::new(obj).text_comdat_functions(),
            None,
            "an aux run past the table end must refuse"
        );
    }

    /// NEGATIVE CONTROL — a COMDAT `.text` section with no leader symbol. The
    /// guard's quantity (two COMDAT `.text` sections) is held FIXED; only the
    /// second section's leader is removed, so the assertion under test is the
    /// one-leader-per-section rule and not the section count.
    #[test]
    fn a_text_comdat_section_with_no_leader_refuses_the_whole_obj() {
        let full = coff(
            &[(".text", true), (".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
                (".text", 2, IMAGE_SYM_CLASS_STATIC, 1),
                ("?g@@YAHH@Z", 2, 2, 0),
            ],
        );
        assert_eq!(
            ObjImage::new(full).text_comdat_functions(),
            Some(vec!["?f@@YAHH@Z".to_string(), "?g@@YAHH@Z".to_string()]),
            "control: two sections, two leaders"
        );
        let leaderless = coff(
            &[(".text", true), (".text", true)],
            &[
                (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
                ("?f@@YAHH@Z", 1, 2, 0),
                (".text", 2, IMAGE_SYM_CLASS_STATIC, 1),
            ],
        );
        assert_eq!(
            ObjImage::new(leaderless).text_comdat_functions(),
            None,
            "a COMDAT .text with no leader must refuse, not return a set of one — \
             a short denominator inflates every ratio computed against it"
        );
    }

    #[test]
    fn a_short_image_refuses() {
        assert_eq!(
            ObjImage::new(vec![0xF2, 0x01, 0x00]).text_comdat_functions(),
            None,
            "an image shorter than a COFF header has no emitted set"
        );
    }

    #[test]
    fn length_mismatch_reports_at_common_len() {
        let a = ObjImage::new(base_obj());
        let mut bb = base_obj();
        bb.push(0x77);
        let b = ObjImage::new(bb);
        match ObjImage::diff(&a, &b) {
            ObjDiff::Differs {
                first_offset,
                a_len,
                b_len,
            } => {
                assert_eq!(first_offset, a.len());
                assert_eq!(a_len, a.len());
                assert_eq!(b_len, a.len() + 1);
            }
            ObjDiff::Identical => panic!("expected a length difference"),
        }
    }
}
