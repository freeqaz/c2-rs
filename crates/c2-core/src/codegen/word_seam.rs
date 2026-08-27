//! **The word-composition seam: one rule composes an instruction word, and the
//! exceptions are enumerated rather than assumed.**
//!
//! # Why this file exists
//!
//! Board **#3637**, lane `w-encmap`: eleven live instruction-word productions
//! sat outside [`super::mop::encode_op`], and **eight of them composed a word
//! the port's own READ base-word table already composed, by a second rule**.
//! They agreed to the bit. The gate was green. `mismatch 0` was true.
//!
//! **A byte compare cannot see a *concurring* second producer.** It stays
//! invisible for exactly as long as the two rules agree, and on the day they
//! diverge the defect is indistinguishable from a lowering bug — so this is the
//! one class of defect the project's sole judge (real `c2.dll` under wibo, plus
//! a byte-exact obj compare) is *structurally* unable to report. Nothing in
//! `scripts/gate.sh` can express it, and nothing ever will: the gate grades
//! output, and both producers produce the same output.
//!
//! `docs/ARCHITECTURE_SEAMS.md` §1 class 4 records the earlier instance — two
//! `encode_std`s landed 2,000 lines apart in the old single-file `codegen.rs`
//! and **git flagged nothing**, because a duplicate in two places is not a
//! textual conflict. That one was caught by a human reading. This one was
//! caught by a lane looking for something else. The pattern in both cases is
//! that the detector was an accident, so this file is the detector on purpose.
//!
//! # The rule
//!
//! > In live (non-`#[cfg(test)]`) code under `crates/c2-core/src`, an
//! > instruction word is composed in **one** place — [`super::mop::compose`],
//! > reached through [`super::mop::encode_op`] at run time or
//! > [`super::mop::const_word`] at compile time. Every other site that
//! > produces four big-endian `.text` bytes is named in [`EMISSIONS`], and
//! > every other site that composes a word is named in [`EXCEPTIONS`] with the
//! > reason it cannot go through the table.
//!
//! # The discriminator — legitimate vs duplicate
//!
//! `w-encmap` found eleven sites and called only some of them duplicates, so
//! the whole content of this file is the sentence that separates the two. It is
//! **registered in `work/w-mopfold/PREREG.md` § P5, before the fold**, so it
//! cannot have been back-fitted to the answer:
//!
//! > A live word production outside the `mop` seam is a **DUPLICATE** iff the
//! > word it emits lies in the **image of [`super::mop::encode_op`] over some
//! > [`super::mop::OPCODES`] row at the default `EncodeParams::C2`** — iff
//! > there is a row and an operand assignment that composes the identical 32
//! > bits. It is **LEGITIMATE** iff no row can compose it, which means the port
//! > emits an instruction c2's *transcribed subset* does not carry, and there
//! > is therefore no second rule to disagree with.
//!
//! [`coverable`] decides that mechanically and **over-approximates toward
//! RED**: for a row on form `f` it masks off every bit `mop::plan(f)` places
//! and compares the remainder against the row's base word. That is a *superset*
//! of the true image — a field may not reach every value its mask allows — so
//! the test can only over-report duplicates, never under-report them. **A row
//! claiming LEGITIMATE has to survive the generous test.**
//!
//! ## Which sub-tree each half runs over, and why they differ
//!
//! * The **value** half ([`no_word_is_composed_outside_the_mop_seam`]) runs over
//!   `codegen/` only. It cannot run over `coff/`: measured on this tree,
//!   **111 of 122** live 32-bit literals under `crates/c2-core/src` decode as
//!   plausible instructions, and almost all of them are COFF section
//!   characteristics — `CH_TEXT = 0x6040_0020` reads as an `ori`,
//!   `CH_PDATA = 0x4040_0040` as a `bc`. A value rule there is noise, not a
//!   control, and a control that cries wolf gets deleted. `codegen/` is where
//!   instruction words are composed and it holds **8** live 32-bit literals
//!   total.
//! * The **structural** half ([`every_big_endian_emission_is_inventoried`])
//!   runs over the whole crate, because a *new* emission site is unambiguous
//!   wherever it appears and `coff/ehscope.rs` really does emit `.text`.
//!
//! ## What this cannot see — stated, because a bound nobody states is a lie
//!
//! 1. A word composed with **no literal at all** (`(primary << 26) | …` from a
//!    variable) and emitted through an **existing** inventoried site. Both
//!    halves are blind to it. Nothing in the tree does this today.
//! 2. A word composed inside `coff/` from a literal. The value half does not
//!    run there; the structural half catches it only if it adds an emission
//!    site, which in practice it must.
//! 3. `#[cfg(test)]` code, deliberately and by charter. `encode.rs`'s
//!    `mod incumbent` is a second producer **on purpose**, with its own armed
//!    cross-check (`mod cross_check`), and it is the reason the required-zero
//!    byte delta of lane `w-s1` was provable in the portable lane at all.
//!
//! # Watched failing
//!
//! A control never seen failing is decoration (board **#3336**: this repo
//! shipped a `--check` flag that structurally could not fail, and caught it
//! only by testing the tester). Both halves carry an armed self-test that
//! fabricates the defect and requires the predicate to find it —
//! [`the_value_half_can_fail`] and [`the_structural_half_can_fail`] — and the
//! landing lane additionally planted a real eighth producer in `frame.rs`,
//! watched the suite go red, and reverted it (`docs/rungs/2026-08-26-w-mopfold.md`
//! § "Watched failing").

#[cfg(test)]
mod seam {
    use crate::codegen::mop;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    // ---------------------------------------------------------------------
    // THE COMMITTED INVENTORY
    // ---------------------------------------------------------------------

    /// One live word production that does **not** go through the `mop` seam.
    struct Exception {
        /// Path relative to `crates/c2-core/src`.
        file: &'static str,
        /// The `fn`/`const` that holds it.
        item: &'static str,
        /// c2's own mnemonic for the instruction, from
        /// `docs/whitebox/ref/ENCODE_OPCODES.txt`.
        mnemonic: &'static str,
        /// The word the site actually emits, **re-derived by running the site**
        /// (see [`witness`]) rather than copied — a witness copied by hand is a
        /// third producer of the same fact.
        witness: u32,
        /// Why the table cannot compose it. Not prose for a reader: the test
        /// asserts the claim.
        why: &'static str,
    }

    /// **Every live instruction-word production outside the `mop` seam.**
    ///
    /// Three rows, and all three are the same shape: c2 files the instruction
    /// under an opcode `mop::OPCODES` does not transcribe. **None of them is a
    /// disagreement** — nothing in this port can compose these words, so there
    /// is no second rule to be wrong. The fence is a missing transcription.
    ///
    /// Finishing any of them is an **adoption**, not a derivation: it copies
    /// rows out of `docs/whitebox/ref/ENCODE_OPCODES.txt` and therefore moves
    /// `docs/whitebox/DISCLOSURE.md`'s `W-MOP-2` / `W-MOP-3` counts. Lane
    /// `w-mopfold` was fenced out of that file and declined all three; the
    /// price is in its rung.
    ///
    /// **The refusal is ARMED, which is the point.** The day `bl`, `mfspr` or
    /// `stwux` is transcribed into `OPCODES`,
    /// [`no_inventoried_word_is_one_mop_can_compose`] turns red on that row and
    /// asks for the fold to finish. A refusal that notices when its own reason
    /// expires is worth more than the fold it postponed.
    const EXCEPTIONS: &[Exception] = &[
        Exception {
            file: "codegen/calls.rs",
            item: "encode_call_branch",
            mnemonic: "bl",
            witness: 0x4BFF_FFF5,
            why: "c2 files `bl` as its OWN opcode on its OWN form — ENCODE_OPCODES.txt \
                  0x002b, base 48000001, form 7, arm 10bfa285 — not as `b` with a link \
                  bit. OPCODES transcribes neither the row nor the arm, and form 7 is \
                  outside the 27 arms w-read-r2 read.",
        },
        Exception {
            file: "codegen/frame.rs",
            item: "FRAME_MFLR_R12",
            mnemonic: "mfspr",
            witness: 0x7D88_02A6,
            why: "ENCODE_OPCODES.txt 0x00e6, base 7c0002a6, form 54, arm 10bfa76a. \
                  OPCODES has no `mfspr` row and `plan` has no form-54 arm; `mtspr` \
                  (form 62) is a different form and cannot stand in for it.",
        },
        Exception {
            file: "codegen/frame.rs",
            item: "FRAME_STWUX",
            mnemonic: "stwux",
            witness: 0x7C21_616E,
            why: "ENCODE_OPCODES.txt 0x017f, base 7c00016e, form 61 — and the port \
                  ALREADY has a form-61 plan (it emits `stdx`). So this one needs one \
                  transcribed row and no new arm: the cheapest of the three, and still \
                  an adoption that moves DISCLOSURE.md.",
        },
    ];

    /// Run each exception's site and return the word it really emits.
    ///
    /// **Executed, never transcribed.** The alternative — writing the word in
    /// the row above and trusting it — would make this file a third producer of
    /// the same fact, which is the defect it exists to close.
    fn witness(e: &Exception) -> u32 {
        match (e.file, e.item) {
            ("codegen/calls.rs", "encode_call_branch") => {
                u32::from_be_bytes(crate::codegen::calls::encode_call_branch(0xC))
            }
            ("codegen/frame.rs", "FRAME_MFLR_R12") => crate::codegen::frame::FRAME_MFLR_R12,
            ("codegen/frame.rs", "FRAME_STWUX") => crate::codegen::frame::FRAME_STWUX,
            _ => panic!(
                "EXCEPTIONS has a row `{}::{}` with no witness arm. Every row must be \
                 RUN, not described: add the arm.",
                e.file, e.item
            ),
        }
    }

    /// **Every live site that turns a value into four big-endian bytes**, as
    /// `(file, enclosing item, count)`.
    ///
    /// The seam itself (`codegen/mop.rs::word`) is row one and the rest are
    /// consumers or non-instruction data. A new row anywhere is red: the author
    /// either routes the word through `mop`, or adds the row and says which of
    /// the three kinds it is.
    ///
    /// `count` is per item rather than per line so that inserting a comment
    /// does not turn the suite red, and per item rather than per file so that a
    /// site appearing in a *new* function is caught even if another was deleted.
    const EMISSIONS: &[(&str, &str, usize)] = &[
        // -- the seam --------------------------------------------------------
        ("codegen/mop.rs", "word", 1),
        // -- instruction words, composed elsewhere (see EXCEPTIONS) ----------
        ("codegen/calls.rs", "encode_call_branch", 1),
        // -- instruction words composed by `mop`, emitted through a `const` --
        ("codegen/frame.rs", "prologue_gpr_helper", 1),
        ("codegen/frame.rs", "prologue_gpr_helper_leaf", 1),
        ("codegen/frame.rs", "prologue", 2),
        ("codegen/frame.rs", "epilogue", 3),
        ("coff/ehscope.rs", "plan_text", 6),
        // -- NOT instructions: big-endian .pdata / .rdata payload ------------
        ("coff/ehscope.rs", "pdata_record_eh", 2),
        ("coff/ehscope.rs", "emit_eh_scope_obj", 1),
        ("coff/function.rs", "real_raw_bytes", 2),
        ("coff/pdata.rs", "pdata_record", 2),
    ];

    // ---------------------------------------------------------------------
    // THE DISCRIMINATOR
    // ---------------------------------------------------------------------

    /// The union of the bit positions `mop::plan(form)` places.
    fn placed_mask(form: mop::Form) -> Option<u32> {
        let fp = mop::plan(form)?;
        let mut m = 0u32;
        for f in fp.fields() {
            let w = if f.width >= 32 { u32::MAX } else { (1u32 << f.width) - 1 };
            // `shift` is where the field lands, so a `DispWord` field (`disp >> 2`
            // at shift 2) contributes `0x03FF_FFFC` and leaves the low two bits
            // OUTSIDE the mask — correctly: no operand can reach them, and a `b`
            // word with bit 0 set is a `bl`, which this table does not compose.
            // That single fact is why `encode_call_branch` reads as LEGITIMATE
            // and `encode_tail_branch` read as a duplicate.
            m |= w << f.shift;
        }
        Some(m)
    }

    /// **Which `OPCODES` rows could compose `word`.** Empty = LEGITIMATE.
    ///
    /// Over-approximates toward red; see the module doc.
    fn coverable(word: u32) -> Vec<&'static str> {
        let mut hits = Vec::new();
        for r in mop::OPCODES {
            // Form 68 composes in code rather than through a `FieldPlan`
            // (`encode_op`'s own special case), so it has no mask to take.
            // Treating it as "covers nothing" is the conservative direction for
            // a LEGITIMATE claim only if nothing claims legitimacy on a
            // 64-bit-rotate word — asserted below.
            let Some(mask) = placed_mask(r.form) else { continue };
            let fixed = mop::plan(r.form).map(|p| p.fixed).unwrap_or(0);
            if word & !mask == (r.base | fixed) & !mask {
                hits.push(r.mnemonic);
            }
        }
        hits
    }

    // ---------------------------------------------------------------------
    // THE SOURCE SCANNER
    // ---------------------------------------------------------------------

    /// Every `.rs` file under `crates/c2-core/src`, as `(relative path, text)`.
    fn crate_sources() -> Vec<(String, String)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files: Vec<PathBuf> = Vec::new();
        walk(&root, &mut files);
        files.sort();
        let out: Vec<(String, String)> = files
            .iter()
            .map(|p| {
                let rel = p
                    .strip_prefix(&root)
                    .expect("walk stayed under src/")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(p)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
                (rel, text)
            })
            .collect();
        // A short listing makes every lint here vacuously green while measuring
        // a fraction of the crate — this project's most-repeated defect
        // (absence reading as success) aimed at the instrument built to stop it.
        assert!(
            out.len() >= 45,
            "crate_sources() found only {} .rs file(s) under {}; c2-core is ~50 files \
             and the count only ever grows. A short walk makes this whole module a \
             rubber stamp.",
            out.len(),
            root.display()
        );
        out
    }

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()));
        for e in entries {
            let p = e.expect("cannot read a directory entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                out.push(p);
            }
        }
    }

    /// Blank out comments and string/char literals, **preserving byte offsets
    /// and line structure** so a hit's line number is still the file's.
    ///
    /// Doc comments have to go: `encode.rs` carries dozens of captured words in
    /// prose (`80630000` = `lwz r3,0(r3)`), and a scanner that reads prose about
    /// code instead of code is how board **#3641** moved a census by 9 rows.
    /// Block comments nest, and a `*/` inside a string is not a terminator —
    /// board **#3649** is the same family: a glob inside a `///` line froze the
    /// provenance census's brace depth from that line to EOF and silently
    /// disabled its whole `#[cfg(test)]` exclusion.
    fn code_only(src: &str) -> String {
        let b = src.as_bytes();
        let n = b.len();
        let mut out = vec![b' '; n];
        let mut i = 0usize;
        // Newlines are copied through unconditionally at the end of each blanked
        // run, so line numbers survive.
        macro_rules! blank {
            ($from:expr, $to:expr) => {
                for k in $from..$to {
                    out[k] = if b[k] == b'\n' { b'\n' } else { b' ' };
                }
            };
        }
        while i < n {
            if b[i] == b'/' && i + 1 < n && b[i + 1] == b'/' {
                let mut j = i;
                while j < n && b[j] != b'\n' {
                    j += 1;
                }
                blank!(i, j);
                i = j;
            } else if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                let mut depth = 1usize;
                let mut j = i + 2;
                while j < n && depth > 0 {
                    if j + 1 < n && b[j] == b'/' && b[j + 1] == b'*' {
                        depth += 1;
                        j += 2;
                    } else if j + 1 < n && b[j] == b'*' && b[j + 1] == b'/' {
                        depth -= 1;
                        j += 2;
                    } else {
                        j += 1;
                    }
                }
                blank!(i, j);
                i = j;
            } else if b[i] == b'r'
                && !(i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_'))
                && i + 1 < n
                && (b[i + 1] == b'#' || b[i + 1] == b'"')
                && raw_string_hashes(b, i).is_some()
            {
                // **Raw strings, and they are not a corner case here.** This
                // module's own `the_structural_half_can_fail` holds a planted
                // Rust file in an `r#"…"#` literal — containing `to_be_bytes`,
                // a `#[cfg(test)]`, and braces. Without this arm the blanker
                // left those braces live, `cfg_test_spans` closed the enclosing
                // `#[cfg(test)] mod seam` early, and the scanner reported its
                // own fixture as a new second producer. Observed, not
                // hypothesised: it is what this file did on its first run.
                let h = raw_string_hashes(b, i).expect("checked above");
                let open = i + 1 + h + 1; // r + hashes + "
                let mut j = open;
                let mut end = n;
                while j < n {
                    if b[j] == b'"' {
                        let mut k = 0usize;
                        while k < h && j + 1 + k < n && b[j + 1 + k] == b'#' {
                            k += 1;
                        }
                        if k == h {
                            end = j + 1 + h;
                            break;
                        }
                    }
                    j += 1;
                }
                blank!(i, end.min(n));
                i = end.min(n);
            } else if b[i] == b'"' {
                let mut j = i + 1;
                while j < n {
                    if b[j] == b'\\' {
                        j += 2;
                    } else if b[j] == b'"' {
                        j += 1;
                        break;
                    } else {
                        j += 1;
                    }
                }
                blank!(i, j.min(n));
                i = j.min(n);
            } else {
                out[i] = b[i];
                i += 1;
            }
        }
        String::from_utf8(out).expect("blanking preserved ASCII structure")
    }

    /// `Some(h)` if a raw-string literal with `h` hashes starts at `i`.
    fn raw_string_hashes(b: &[u8], i: usize) -> Option<usize> {
        let mut h = 0usize;
        let mut j = i + 1;
        while j < b.len() && b[j] == b'#' {
            h += 1;
            j += 1;
        }
        if j < b.len() && b[j] == b'"' {
            Some(h)
        } else {
            None
        }
    }

    /// Byte ranges of `#[cfg(test)]` items, over already-blanked source.
    fn cfg_test_spans(src: &str) -> Vec<(usize, usize)> {
        let b = src.as_bytes();
        let mut spans = Vec::new();
        let needle = "#[cfg(test)]";
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(needle) {
            let at = from + rel;
            from = at + needle.len();
            let Some(open) = src[from..].find('{').map(|o| from + o) else { continue };
            let mut depth = 0usize;
            let mut k = open;
            while k < b.len() {
                if b[k] == b'{' {
                    depth += 1;
                } else if b[k] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                k += 1;
            }
            spans.push((at, (k + 1).min(b.len())));
        }
        spans
    }

    /// The nearest preceding `fn` / `const` / `static` declaration.
    ///
    /// Deliberately the *nearest* and not the top-level one: `frame.rs`'s
    /// emitters live in an `impl`, so a column-0 rule would name the `impl`'s
    /// file-level neighbour for all of them. The cost is that a `const`
    /// declared inside a function between the header and the site renames the
    /// item and turns [`every_big_endian_emission_is_inventoried`] red. That is
    /// the safe direction: red here means "re-check the inventory", and the
    /// failure prints the whole observed table to paste back.
    fn enclosing_item(src: &str, pos: usize) -> String {
        let head = &src[..pos];
        let mut best: Option<String> = None;
        for kw in ["fn ", "const ", "static "] {
            let mut from = 0usize;
            while let Some(rel) = head[from..].find(kw) {
                let at = from + rel;
                from = at + kw.len();
                // Must start a token.
                if at > 0 {
                    let p = head.as_bytes()[at - 1];
                    if p.is_ascii_alphanumeric() || p == b'_' {
                        continue;
                    }
                }
                let rest = &head[at + kw.len()..];
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if name.is_empty() {
                    continue;
                }
                match &best {
                    Some(_) if best_pos(&best, head) > at => {}
                    _ => best = Some(format!("{at}\u{0}{name}")),
                }
            }
        }
        best.map(|s| s.split('\u{0}').nth(1).unwrap().to_string())
            .unwrap_or_else(|| "?".to_string())
    }

    fn best_pos(best: &Option<String>, _head: &str) -> usize {
        best.as_ref()
            .and_then(|s| s.split('\u{0}').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Every live `to_be_bytes` site: `(file, item, line)`.
    fn live_emission_sites() -> Vec<(String, String, usize)> {
        let mut out = Vec::new();
        for (name, text) in crate_sources() {
            let src = code_only(&text);
            let spans = cfg_test_spans(&src);
            let mut from = 0usize;
            while let Some(rel) = src[from..].find("to_be_bytes") {
                let at = from + rel;
                from = at + "to_be_bytes".len();
                if spans.iter().any(|(a, b)| *a <= at && at < *b) {
                    continue;
                }
                let line = src[..at].matches('\n').count() + 1;
                out.push((name.clone(), enclosing_item(&src, at), line));
            }
        }
        out
    }

    /// Every live 32-bit-magnitude integer literal under `codegen/`, outside
    /// `mop.rs`'s own table: `(file, item, line, value)`.
    ///
    /// `OPCODE_ROWS` is exempt by byte range, and the range's two markers are
    /// asserted to exist — an exemption whose anchor has silently vanished
    /// exempts nothing or everything, and both look like a pass.
    fn live_word_literals() -> Vec<(String, String, usize, u32)> {
        let mut out = Vec::new();
        for (name, text) in crate_sources() {
            if !name.starts_with("codegen/") {
                continue;
            }
            let src = code_only(&text);
            let mut spans = cfg_test_spans(&src);
            if name == "codegen/mop.rs" {
                let start = src.find("const OPCODE_ROWS").expect(
                    "codegen/mop.rs no longer declares `const OPCODE_ROWS` — this scanner's \
                     one exemption has lost its anchor, and an exemption with no anchor \
                     exempts either nothing or everything.",
                );
                let end = src[start..].find("];").map(|o| start + o + 2).expect(
                    "`const OPCODE_ROWS`'s closing `];` is gone — the exemption range is \
                     open-ended and would swallow the rest of the file.",
                );
                spans.push((start, end));
            }
            let b = src.as_bytes();
            let mut i = 0usize;
            while i + 2 < b.len() {
                if b[i] == b'0' && (b[i + 1] == b'x' || b[i + 1] == b'X') {
                    let start = i;
                    let mut j = i + 2;
                    let mut digits = String::new();
                    while j < b.len() && (b[j].is_ascii_hexdigit() || b[j] == b'_') {
                        if b[j] != b'_' {
                            digits.push(b[j] as char);
                        }
                        j += 1;
                    }
                    i = j;
                    if digits.is_empty() || digits.len() > 8 {
                        continue;
                    }
                    if spans.iter().any(|(a, e)| *a <= start && start < *e) {
                        continue;
                    }
                    let v = u32::from_str_radix(&digits, 16).expect("<= 8 hex digits");
                    if v < 0x0100_0000 {
                        continue;
                    }
                    let line = src[..start].matches('\n').count() + 1;
                    out.push((name.clone(), enclosing_item(&src, start), line, v));
                } else {
                    i += 1;
                }
            }
        }
        out
    }

    // ---------------------------------------------------------------------
    // THE CONTROL, half one: values
    // ---------------------------------------------------------------------

    /// **No live literal under `codegen/` composes a word `mop` already
    /// composes**, unless [`EXCEPTIONS`] names its item.
    ///
    /// This is the half that goes red when a lane writes `0x3D6B_0000` inline.
    #[test]
    fn no_word_is_composed_outside_the_mop_seam() {
        let lits = live_word_literals();
        // Vacuity floor: a scanner that finds nothing proves nothing.
        assert!(
            lits.len() >= 5,
            "the literal scan found only {} live 32-bit literal(s) under codegen/. \
             It found 8 when this control was written; a collapse to near-zero means \
             the scanner broke, not that the tree got clean.",
            lits.len()
        );
        let named: Vec<&str> = EXCEPTIONS.iter().map(|e| e.item).collect();
        let mut bad = Vec::new();
        for (file, item, line, v) in &lits {
            let hits = coverable(*v);
            if hits.is_empty() {
                continue;
            }
            if named.contains(&item.as_str()) {
                continue;
            }
            bad.push(format!(
                "  {file}:{line}  {v:#010x}  in `{item}`  — mop::OPCODES already \
                 composes this word as `{}`",
                hits.join("` / `")
            ));
        }
        assert!(
            bad.is_empty(),
            "A SECOND PRODUCER OF AN INSTRUCTION WORD APPEARED.\n{}\n\n\
             The port's READ base-word table (`mop::OPCODES`, from \
             `docs/whitebox/ref/ENCODE_OPCODES.txt`) can compose the word above, so \
             writing it as a literal creates two rules for one fact. They will agree \
             today — that is the whole problem: a byte compare cannot see a CONCURRING \
             second producer, and `mismatch 0` stays silent until the day they diverge, \
             at which point the defect is indistinguishable from a lowering bug \
             (board #3637).\n\n\
             Compose it instead: `mop::const_word(mop_stw(12, 1, -8))` for a `const`, \
             or `mop_stw(12, 1, -8).word()` at run time.\n\n\
             If it genuinely cannot go through the table, add a row to `EXCEPTIONS` in \
             this file saying WHY — and know that the row is checked, not believed: \
             `no_inventoried_word_is_one_mop_can_compose` re-derives the word every run \
             and fails if the table can in fact compose it.",
            bad.join("\n")
        );
    }

    // ---------------------------------------------------------------------
    // THE CONTROL, half two: structure
    // ---------------------------------------------------------------------

    /// **Every live big-endian 4-byte emission in the crate is inventoried.**
    ///
    /// The value half only sees a word spelled as a literal. This half sees the
    /// *site*, however the word got there — so a composition built entirely
    /// from variables still has to declare itself.
    #[test]
    fn every_big_endian_emission_is_inventoried() {
        let sites = live_emission_sites();
        assert!(
            sites.len() >= 15,
            "only {} live `to_be_bytes` site(s) in the whole crate — the scanner is \
             broken. There were 22 when this control was written.",
            sites.len()
        );
        // The seam itself must be among them, or the scanner is not looking at
        // the emit path at all.
        assert!(
            sites
                .iter()
                .any(|(f, i, _)| f == "codegen/mop.rs" && i == "word"),
            "`codegen/mop.rs::word` — the one composition seam — is not in the scan. \
             Either it moved (update EMISSIONS) or the scanner stopped reading mop.rs, \
             and the second would make every other assertion here meaningless."
        );

        let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
        for (f, i, _) in &sites {
            *seen.entry((f.clone(), i.clone())).or_insert(0) += 1;
        }
        let want: BTreeMap<(String, String), usize> = EMISSIONS
            .iter()
            .map(|(f, i, n)| ((f.to_string(), i.to_string()), *n))
            .collect();
        if seen != want {
            let mut msg = String::from(
                "THE BIG-ENDIAN EMISSION INVENTORY IS STALE.\n\n\
                 A site that turns a value into four big-endian bytes is a site that can \
                 put an instruction into `.text`, and this crate keeps an enumerated list \
                 of them so that a NEW one is a decision somebody made on purpose rather \
                 than a diff nobody read (board #3637; ARCHITECTURE_SEAMS.md §1 class 4, \
                 where two `encode_std`s lived 2,000 lines apart and git flagged \
                 nothing).\n\n",
            );
            for (k, n) in &seen {
                match want.get(k) {
                    Some(w) if w == n => {}
                    Some(w) => msg.push_str(&format!(
                        "  CHANGED  {}::{}  inventoried {w}, found {n}\n",
                        k.0, k.1
                    )),
                    None => msg.push_str(&format!("  NEW      {}::{}  x{n}\n", k.0, k.1)),
                }
            }
            for k in want.keys() {
                if !seen.contains_key(k) {
                    msg.push_str(&format!("  GONE     {}::{}\n", k.0, k.1));
                }
            }
            msg.push_str(
                "\nIf the word is an instruction, compose it through `mop` and it needs \
                 no row. Otherwise add the row to `EMISSIONS` and say which kind it is. \
                 The whole observed table, to paste:\n\n",
            );
            for ((f, i), n) in &seen {
                msg.push_str(&format!("        (\"{f}\", \"{i}\", {n}),\n"));
            }
            panic!("{msg}");
        }
    }

    // ---------------------------------------------------------------------
    // THE CONTROL, half three: the exceptions are checked, not believed
    // ---------------------------------------------------------------------

    /// **Every inventoried exception is still LEGITIMATE.**
    ///
    /// The row says "the table cannot compose this". This runs the site, takes
    /// the word it really emits, and asks the table. A row that becomes false —
    /// because somebody transcribed `bl`'s row into `OPCODES`, say — goes red
    /// and asks for the fold that was postponed.
    #[test]
    fn no_inventoried_word_is_one_mop_can_compose() {
        assert!(!EXCEPTIONS.is_empty(), "an empty exception list checks nothing");
        for e in EXCEPTIONS {
            let got = witness(e);
            assert_eq!(
                got, e.witness,
                "{}::{} now emits {got:#010x}, not the inventoried {:#010x}. The word \
                 moved; re-check the instruction before re-recording it.",
                e.file, e.item, e.witness
            );
            let hits = coverable(got);
            assert!(
                hits.is_empty(),
                "THE REFUSAL ON {}::{} HAS EXPIRED — and that is good news.\n\n\
                 `{}` was inventoried as impossible to compose here: \"{}\"\n\n\
                 `mop::OPCODES` can now compose {got:#010x} as `{}`. So the site is a \
                 DUPLICATE PRODUCER as of this commit: two rules, one word, agreeing \
                 today and invisible to the byte judge for exactly as long as they \
                 agree. Fold it (`mop::const_word` for a `const`, `.word()` at run time) \
                 and delete this row.",
                e.file,
                e.item,
                e.mnemonic,
                e.why,
                hits.join("` / `")
            );
        }
    }

    // ---------------------------------------------------------------------
    // WATCHED FAILING — the armed self-tests
    // ---------------------------------------------------------------------

    /// **The discriminator can say YES.**
    ///
    /// [`coverable`] returning empty is the green condition everywhere above,
    /// and a function that returned empty unconditionally would make all three
    /// controls pass forever. Board **#3336**: this repo shipped a `--check`
    /// flag that structurally could not fail. So: feed it words the table
    /// demonstrably composes and require a hit.
    #[test]
    fn the_value_half_can_fail() {
        // The four words lane `w-mopfold` folded, re-derived from `mop` itself
        // so this control cannot drift from the table it is about.
        let folded = [
            crate::codegen::frame::FRAME_LR_STORE,
            crate::codegen::frame::FRAME_LR_LOAD,
            crate::codegen::frame::FRAME_MTLR_R12,
            u32::from_be_bytes(crate::codegen::calls::encode_tail_branch(8)),
        ];
        for w in folded {
            assert!(
                !coverable(w).is_empty(),
                "coverable({w:#010x}) came back EMPTY for a word `mop` composes itself. \
                 The discriminator cannot say yes, so every green in this module is a \
                 green over nothing."
            );
        }
        // And the planted defect, in the exact shape a future lane would write
        // it: a full `stw r12,-8(r1)` spelled as a literal.
        let planted = 0x9181_FFF8u32;
        let hits = coverable(planted);
        assert!(
            hits.contains(&"stw"),
            "the planted duplicate {planted:#010x} was not identified as `stw`; got {hits:?}"
        );
        // The negative side of the same control: the three inventoried
        // exceptions must NOT be flagged, or the discriminator says yes to
        // everything and is equally useless.
        for e in EXCEPTIONS {
            assert!(
                coverable(e.witness).is_empty(),
                "coverable() flagged the LEGITIMATE {} — it is saying yes to everything",
                e.mnemonic
            );
        }
    }

    /// **The structural half can fail.**
    ///
    /// Runs the real scanner over a fabricated source file that contains one
    /// new emission site, and requires it to be found. The scanner, the
    /// comment-stripper and the `#[cfg(test)]` exclusion are all exercised;
    /// only the file walk is stubbed.
    #[test]
    fn the_structural_half_can_fail() {
        let fake = r#"
//! A doc comment mentioning to_be_bytes — prose, must NOT count.
/* a block /* nested */ comment with to_be_bytes — must NOT count */
pub fn brand_new_producer(off: u32) -> [u8; 4] {
    let word: u32 = 0x9181_FFF8 | off;
    word.to_be_bytes()
}
#[cfg(test)]
mod t {
    #[test]
    fn x() { let _ = 1u32.to_be_bytes(); }
}
const AFTER: &str = "a string with to_be_bytes in it";
"#;
        let src = code_only(fake);
        let spans = cfg_test_spans(&src);
        let mut live = Vec::new();
        let mut from = 0usize;
        while let Some(rel) = src[from..].find("to_be_bytes") {
            let at = from + rel;
            from = at + "to_be_bytes".len();
            if spans.iter().any(|(a, b)| *a <= at && at < *b) {
                continue;
            }
            live.push(enclosing_item(&src, at));
        }
        assert_eq!(
            live,
            vec!["brand_new_producer".to_string()],
            "the scanner did not isolate the one LIVE emission site in a planted file. \
             Got {live:?}. Either it counted prose or a string (over-reporting, which \
             makes the inventory noise) or it missed the real site (under-reporting, \
             which is the failure this whole module exists to prevent)."
        );
        // And the value half must flag the planted literal in the same file.
        let mut flagged = 0;
        let b = src.as_bytes();
        let mut i = 0;
        while i + 2 < b.len() {
            if b[i] == b'0' && b[i + 1] == b'x' {
                let mut j = i + 2;
                let mut d = String::new();
                while j < b.len() && (b[j].is_ascii_hexdigit() || b[j] == b'_') {
                    if b[j] != b'_' {
                        d.push(b[j] as char);
                    }
                    j += 1;
                }
                if !d.is_empty() && d.len() <= 8 {
                    let v = u32::from_str_radix(&d, 16).unwrap();
                    if v >= 0x0100_0000 && !coverable(v).is_empty() {
                        flagged += 1;
                    }
                }
                i = j;
            } else {
                i += 1;
            }
        }
        assert_eq!(
            flagged, 1,
            "the value half found {flagged} coverable literal(s) in the planted file; \
             expected exactly the one `stw` word."
        );
    }

    // ---------------------------------------------------------------------
    // THE SEAM ITSELF — one composition, two entry points
    // ---------------------------------------------------------------------

    /// **`const_word` and `encode_op` are the same function.**
    ///
    /// [`mop::const_word`] exists so a `const` instruction word can be a
    /// `MachineOp`. That would be worthless — a ninth second producer — if it
    /// composed by its own rule. Every row, every slot, both entry points.
    #[test]
    fn const_word_and_encode_op_agree() {
        let mut checked = 0usize;
        for r in mop::OPCODES {
            // Form 68's split immediates compose in code in `encode_op` and are
            // refused by `const_word`; the port emits no 64-bit rotate as a
            // `const`, and the refusal is the honest state.
            if r.form.0 == 68 {
                continue;
            }
            for pat in [
                (0u8, 0u8, 0u8, 0u8, 0u8, 0i32),
                (31, 0, 0, 0, 0, 0),
                (0, 31, 0, 0, 0, 0),
                (12, 1, 8, 0, 0, -8),
                (5, 9, 3, 1, 2, 0x1234),
                (31, 31, 31, 31, 31, -1),
                (1, 2, 3, 4, 5, i16::MIN as i32),
            ] {
                let m = mop::MachineOp::new(r.op)
                    .s(pat.0)
                    .d0(pat.1)
                    .d1(pat.2)
                    .d2(pat.3)
                    .d3(pat.4)
                    .disp(pat.5);
                let run = mop::encode_op(&m, &mop::EncodeParams::C2)
                    .expect("every OPCODES row encodes at the default");
                assert_eq!(
                    mop::const_word(m),
                    run,
                    "const_word and encode_op disagree on `{}` at {pat:?}",
                    r.mnemonic
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 500,
            "only {checked} (row, operand) pairs compared — the sweep is not covering \
             the table"
        );
    }

    /// **The historical literals, pinned.**
    ///
    /// The four words lane `w-mopfold` folded were `const` hex literals from
    /// this project's first framed function until 2026-08-26. This is the
    /// `mod incumbent` pattern one scale down: a deliberate second producer in
    /// `#[cfg(test)]` with an armed cross-check, which is the only kind this
    /// module permits. If a `mop` change moves any of these, the byte judge
    /// would see it — but only on a corpus that reaches a framed non-leaf, and
    /// this sees it in the portable lane with no toolchain at all.
    #[test]
    fn the_folded_words_still_have_their_captured_values() {
        use crate::codegen::calls::encode_tail_branch;
        use crate::codegen::frame::{FRAME_LR_LOAD, FRAME_LR_STORE, FRAME_MTLR_R12};
        assert_eq!(FRAME_LR_STORE, 0x9181_FFF8, "stw r12,-8(r1)");
        assert_eq!(FRAME_LR_LOAD, 0x8181_FFF8, "lwz r12,-8(r1)");
        assert_eq!(FRAME_MTLR_R12, 0x7D88_03A6, "mtlr r12 = mtspr 8,r12");
        // `FRAME_BACKCHAIN` is private to `frame`; its value is pinned there.
        assert_eq!(encode_tail_branch(0), [0x48, 0x00, 0x00, 0x00], "b at offset 0");
        assert_eq!(encode_tail_branch(8), [0x4B, 0xFF, 0xFF, 0xF8], "b at offset 8");
        // The old rule and the new one are the same FUNCTION, not merely equal
        // on the captured pair — `((d >> 2) & 0xFFFFFF) << 2 == d & 0x03FF_FFFC`
        // for all `d`, because the mask's low two bits are zero and its width
        // discards the sign extension. Checked over a range that includes
        // misaligned and negative displacements, which is where a `>>` and a
        // `&` would part company if they ever did.
        for off in (0u32..0x4000).step_by(1).chain([0x00FF_FFFCu32, 0x0100_0000]) {
            let old = 0x4800_0000u32 | ((-(off as i32)) as u32 & 0x03FF_FFFC);
            assert_eq!(
                u32::from_be_bytes(encode_tail_branch(off)),
                old,
                "the folded `b` and the retired mask disagree at offset {off:#x}"
            );
        }
    }
}
