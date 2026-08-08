//! Tests for the COFF writers.
//!
//! Kept as one module across the `coff` split so the split stayed a pure move
//! with nothing re-scoped; new tests belong in the module they exercise.

use super::*;

#[cfg(test)]
mod comdat_tests {
    use super::*;

    /// An undefined external callee is emitted once per distinct *name*, not once
    /// per call site, and every later site relocates against that first index.
    ///
    /// Invisible until a TU has two functions calling the same callee, which no
    /// fixture did before `il_call_perm.cpp` — there the five functions after
    /// `pass3` all call `g3` and the reference has exactly one `?g3@@YAHHHH@Z`.
    /// Emitting per call site inflates `NumberOfSymbols` and shifts every symbol
    /// index after the duplicate, so it is a whole-obj mismatch, not a local one.
    #[test]
    fn callee_symbols_are_emitted_once_per_distinct_name() {
        let text = vec![0u8; 12];
        let mk = |name: &'static str, off: u32, callee: &'static str| Function {
            name,
            text_offset: off,
            calls: vec![Call { reloc_offset: off, callee }],
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            data_defs: Vec::new(),
            frame: None,
            label_lead: 0,
        };
        // Three functions, two of them calling the same callee.
        let funcs = [mk("?a@@YAHXZ", 0, "?g@@YAHXZ"), mk("?b@@YAHXZ", 4, "?h@@YAHXZ"), mk("?c@@YAHXZ", 8, "?g@@YAHXZ")];
        let obj = emit_obj("Z:\\t.obj", &funcs, &text, 0);
        let n_symbols = u32::from_le_bytes(obj[12..16].try_into().unwrap());
        // 13 fixed + 3 defined + 2 distinct callees, NOT 3.
        assert_eq!(n_symbols, 18, "expected one symbol per distinct callee");

        // All three relocations are present, and the first and third share a
        // symbol index while the second differs.
        let n_reloc = u16::from_le_bytes(
            obj[COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 32..][..2].try_into().unwrap(),
        );
        assert_eq!(n_reloc, 3);
        let prel = u32::from_le_bytes(
            obj[COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 24..][..4].try_into().unwrap(),
        ) as usize;
        let sym_of = |i: usize| {
            u32::from_le_bytes(obj[prel + i * RELOC_LEN + 4..][..4].try_into().unwrap())
        };
        assert_eq!(sym_of(0), sym_of(2), "both `?g` call sites relocate to one symbol");
        assert_ne!(sym_of(0), sym_of(1));
    }

    /// The COMDAT emitter's two layout bugs, both found only when
    /// `scripts/mode_lane.sh` first compiled the call fixtures with `/Gy`:
    /// a callee symbol per *call site* rather than per distinct name, and all
    /// relocations batched after all raw data rather than each following its own
    /// section's.
    #[test]
    fn comdat_dedups_callees_and_places_relocs_with_their_section() {
        let blr = crate::codegen::encode_blr().to_vec();
        let mk = |name: &'static str, callee: &'static str| Function {
            name,
            text_offset: 0,
            calls: vec![Call { reloc_offset: 0, callee }],
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            data_defs: Vec::new(),
            frame: None,
            label_lead: 0,
        };
        // Three functions, two calling the same callee — the shape `il_call_perm.cpp`
        // has six of, where the port came out five symbols long.
        let funcs = [
            mk("?a@@YAHXZ", "?g@@YAHXZ"),
            mk("?b@@YAHXZ", "?h@@YAHXZ"),
            mk("?c@@YAHXZ", "?g@@YAHXZ"),
        ];
        let texts = vec![blr.clone(), blr.clone(), blr];
        let obj = emit_comdat_obj("Z:\\t.obj", &funcs, &texts, 0).expect("no defined data");

        // 11 fixed + per function (section symbol + aux + defined symbol) = 9,
        // + 2 distinct callees, NOT 3.
        let n_symbols = u32::from_le_bytes(obj[12..16].try_into().unwrap());
        assert_eq!(n_symbols, 22, "expected one symbol per distinct callee");

        // Each `.text` section's relocation sits immediately after its own raw
            // 4 fixed sections precede the per-function `.text` run.
        // data, so `PointerToRelocations` == `PointerToRawData` + raw length.
        for i in 0..funcs.len() {
            let h = COFF_HEADER_LEN + (4 + i) * SECTION_HEADER_LEN;
            let size = u32::from_le_bytes(obj[h + 16..][..4].try_into().unwrap()) as usize;
            let raw = u32::from_le_bytes(obj[h + 20..][..4].try_into().unwrap()) as usize;
            let prel = u32::from_le_bytes(obj[h + 24..][..4].try_into().unwrap()) as usize;
            assert_eq!(
                prel,
                raw + size,
                "section {i}: relocations must follow their own raw data"
            );
        }
        // And the two `?g` sites share one symbol index while `?h` differs.
        let sym_at = |i: usize| {
            let h = COFF_HEADER_LEN + (4 + i) * SECTION_HEADER_LEN;
            let prel = u32::from_le_bytes(obj[h + 24..][..4].try_into().unwrap()) as usize;
            u32::from_le_bytes(obj[prel + 4..][..4].try_into().unwrap())
        };
        assert_eq!(sym_at(0), sym_at(2), "both `?g` call sites relocate to one symbol");
        assert_ne!(sym_at(0), sym_at(1));
    }

    /// The COMDAT shape, pinned against `system/utl/Spew.cpp` compiled with the
    /// dc3 workload's real flags (two empty functions, so two 4-byte `.text`
    /// sections each holding a single `blr`).
    #[test]
    fn comdat_obj_has_one_text_section_per_function() {
        let blr = crate::codegen::encode_blr().to_vec();
        let funcs = [
            Function::plain("?SpewInit@@YAXXZ", 0),
            Function::plain("?SpewTerminate@@YAXXZ", 0),
        ];
        let obj = emit_comdat_obj("Z:\\x.obj", &funcs, &[blr.clone(), blr], 0).expect("no defined data");

        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| {
            u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]])
        };
        // 4 fixed sections + one per function; 11 fixed symbols + 3 per function.
        assert_eq!(u16at(2), 6, "section count");
        assert_eq!(u32at(12), 17, "symbol count");

        // Both `.text` sections are 4 bytes and carry the COMDAT bit.
        for i in 4..6 {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            assert_eq!(&obj[o..o + 5], b".text");
            assert_eq!(u32at(o + 16), 4, "section {i} size");
            assert_eq!(u32at(o + 36), CH_TEXT_COMDAT, "section {i} characteristics");
        }
        // Contiguous raw data — no inter-function padding, unlike the packed
        // layout's 8-byte function alignment.
        let raw0 = u32at(COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 20);
        let raw1 = u32at(COFF_HEADER_LEN + 5 * SECTION_HEADER_LEN + 20);
        assert_eq!(raw1, raw0 + 4, "second .text follows the first with no padding");

        // Each function symbol sits at Value 0 in its OWN section, and each
        // section symbol's aux selects NODUPLICATES.
        let symtab = u32at(8) as usize;
        for (k, sec_num) in [(0usize, 5i16), (1, 6)] {
            let secsym = symtab + (11 + k * 3) * 18;
            assert_eq!(obj[secsym + 17], 1, "section symbol has one aux");
            let aux = secsym + 18;
            assert_eq!(obj[aux + 14], COMDAT_SELECT_NODUPLICATES, "aux selection");
            let fnsym = secsym + 36;
            assert_eq!(u32at(fnsym + 8), 0, "function Value is 0 in its own section");
            assert_eq!(
                i16::from_le_bytes([obj[fnsym + 12], obj[fnsym + 13]]),
                sec_num
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /// **[`PORT_WRITER_SECTIONS`] is a transcription, so a test reads the source
    /// and checks it.**
    ///
    /// The constant is the whole of what the scan's **factor C** admits, and its
    /// failure mode is silent in both directions: too small and a byte-exact TU
    /// falls outside C, too large and C is inflated. It went stale exactly once
    /// already — it listed six names while `emit_dyninit_obj` sat uncalled, and
    /// wiring that emitter to a caller made three of the port's real section
    /// names missing from it.
    ///
    /// Scanning the emitters' own `Section { name: "…" }` literals is what makes
    /// the constant checkable rather than merely asserted. Portable: no
    /// toolchain, no obj, just the source text.
    ///
    /// **Every module that can build a `Section` must be listed below.** When
    /// `coff.rs` was one file this read that single file; the split makes the
    /// source set explicit, and a new emitter module that is not added here is a
    /// module whose section names stop being checked. That is the same silent
    /// staleness the constant already suffered once, so the list is asserted
    /// against the directory rather than trusted: `SECTION_SOURCES` must name
    /// every `coff/*.rs` that contains the literal `Section {`.
    #[test]
    fn the_writer_vocabulary_is_every_section_name_this_file_emits() {
        const SECTION_SOURCES: [(&str, &str); 5] = [
            ("data.rs", include_str!("data.rs")),
            ("shell.rs", include_str!("shell.rs")),
            ("writer.rs", include_str!("writer.rs")),
            ("dyninit.rs", include_str!("dyninit.rs")),
            ("function.rs", include_str!("function.rs")),
        ];
        // The list above is a transcription too, so it gets the same treatment as
        // the constant it feeds: every `coff/*.rs` that can construct a `Section`
        // must appear in it. A new emitter module is otherwise invisible here and
        // its section names silently stop being checked.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/coff");
        let mut builders: Vec<String> = std::fs::read_dir(&dir)
            .expect("the coff module directory is readable from the test cwd")
            .filter_map(|e| {
                let p = e.ok()?.path();
                if p.extension()? != "rs" {
                    return None;
                }
                let name = p.file_name()?.to_str()?.to_string();
                if name == "tests.rs" {
                    return None; // fixtures, not emitters
                }
                std::fs::read_to_string(&p)
                    .ok()?
                    .contains("Section {")
                    .then_some(name)
            })
            .collect();
        builders.sort();
        let mut listed: Vec<String> = SECTION_SOURCES.iter().map(|(n, _)| n.to_string()).collect();
        listed.sort();
        assert_eq!(
            listed, builders,
            "SECTION_SOURCES must name every coff/*.rs that constructs a `Section`"
        );

        let mut found: Vec<&str> = Vec::new();
        for (_, src) in SECTION_SOURCES {
            for (at, pat) in src.match_indices("name: \"") {
                let s = &src[at + pat.len()..];
                let Some(end) = s.find('"') else { continue };
                let name = &s[..end];
                if name.starts_with('.') && !found.contains(&name) {
                    found.push(name);
                }
            }
        }
        found.sort_unstable();
        let mut declared: Vec<&str> = PORT_WRITER_SECTIONS.to_vec();
        declared.sort_unstable();
        assert_eq!(
            declared, found,
            "PORT_WRITER_SECTIONS must be exactly the section names this file's \
             `Section {{ name: … }}` tables can emit — it is a vocabulary, so \
             `.XBLD$W` appears once even though it is emitted twice"
        );
    }

    /// **The OTHER half of the [`PORT_WRITER_SECTIONS`] guard — board #301,
    /// closed here by lane `w-rtti`.**
    ///
    /// The test above reconciles the constant against the `Section { name: … }`
    /// literals. It cannot see one level up: **a literal inside an emitter that
    /// nothing calls satisfies it and still inflates factor C.** That is not a
    /// hypothetical — `container::bss_deferred_layout` was a `.bss` layout the
    /// differential had never graded one byte of, it disagreed with reality on
    /// the walk *and* on the free list the moment a real caller was written, and
    /// board **#278** deleted it. `w-rdata` §10 filed the residual hole and
    /// priced the closure; this is that closure.
    ///
    /// The rule: **every `pub fn emit_*` that exists in a non-test build must be
    /// named by `lib.rs`.**
    ///
    /// It read `pub fn emit_*_obj` for one commit, and `work/w-rtti/counterfactual.sh`
    /// refuted that in its first run: the breaker's uncalled emitter was named
    /// `emit_rtti_obj_counterfactual`, which does not END in `_obj`, so the
    /// population excluded it and BREAK 2 passed both tests. A suffix
    /// convention is an allow-list wearing a different hat. `lib.rs` is where `PortC2::build` dispatches,
    /// so being named there is the cheapest checkable proxy for *"the
    /// differential can reach this"* — a proxy, not a proof: it cannot tell a
    /// live call from a doc link, and a caller behind an unsatisfiable condition
    /// still counts. It catches the shape that has actually cost this project
    /// something twice, which is an emitter added with no caller at all.
    ///
    /// `emit_mvp_obj` is `#[cfg(test)]` for exactly this reason and is therefore
    /// **absent from the population below** rather than exempted from it — an
    /// allow-list is the thing that goes stale.
    ///
    /// Portable: source text only, no toolchain.
    #[test]
    fn every_production_emitter_has_a_lib_rs_caller() {
        const EMITTER_SOURCES: [(&str, &str); 5] = [
            ("data.rs", include_str!("data.rs")),
            ("shell.rs", include_str!("shell.rs")),
            ("writer.rs", include_str!("writer.rs")),
            ("dyninit.rs", include_str!("dyninit.rs")),
            ("function.rs", include_str!("function.rs")),
        ];
        // Same discipline as SECTION_SOURCES above: the list is a transcription,
        // so it is asserted against the directory rather than trusted.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/coff");
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .expect("the coff module directory is readable from the test cwd")
            .filter_map(|e| {
                let p = e.ok()?.path();
                if p.extension()? != "rs" {
                    return None;
                }
                let name = p.file_name()?.to_str()?.to_string();
                if name == "tests.rs" || name == "mod.rs" {
                    return None;
                }
                std::fs::read_to_string(&p)
                    .ok()?
                    .contains("pub fn emit_")
                    .then_some(name)
            })
            .collect();
        on_disk.sort();
        let mut listed: Vec<String> = EMITTER_SOURCES.iter().map(|(n, _)| n.to_string()).collect();
        listed.sort();
        assert_eq!(
            listed, on_disk,
            "EMITTER_SOURCES must name every coff/*.rs that declares a `pub fn emit_`"
        );

        // The population: EVERY `pub fn emit_*` NOT immediately preceded by a
        // `#[cfg(test)]` attribute. Reading the attribute off the line above the
        // signature is crude and it is also exactly what `#[cfg(test)]` looks
        // like in this file set — checked against `emit_mvp_obj`, which is the
        // only member today and must NOT appear in `production`.
        let mut production: Vec<String> = Vec::new();
        let mut test_only: Vec<String> = Vec::new();
        for (file, src) in EMITTER_SOURCES {
            let lines: Vec<&str> = src.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let t = line.trim_start();
                let Some(rest) = t.strip_prefix("pub fn emit_") else {
                    continue;
                };
                let Some(end) = rest.find(['(', '<']) else { continue };
                let name = format!("emit_{}", &rest[..end]);
                let gated = i > 0 && lines[i - 1].trim() == "#[cfg(test)]";
                if gated {
                    test_only.push(name);
                } else {
                    production.push(format!("{name} ({file})"));
                }
            }
        }
        production.sort();
        test_only.sort();

        assert_eq!(
            test_only,
            vec!["emit_mvp_obj".to_string()],
            "the test-only emitter set changed; if an emitter was gated to stop \
             it inflating factor C, say so in its doc and update this expectation"
        );
        assert!(
            !production.is_empty(),
            "no production emitter was found at all — the scan below would pass \
             vacuously, which is `docs/STATUS.md` trap 5"
        );

        let lib_rs = include_str!("../lib.rs");
        let orphans: Vec<&String> = production
            .iter()
            .filter(|p| {
                let name = p.split(' ').next().unwrap();
                !lib_rs.contains(name)
            })
            .collect();
        assert!(
            orphans.is_empty(),
            "every `pub fn emit_*` must be named by lib.rs — an emitter with \
             no caller satisfies the vocabulary test above and still inflates \
             factor C (board #278, board #301). Orphans: {orphans:?}. Checked \
             {} production emitters.",
            production.len()
        );
        // A count, never a status (trap 5): a run that checked nothing must not
        // read the same as a run that checked everything.
        assert_eq!(
            production.len(),
            5,
            "production emitters: {production:?} — update this count deliberately"
        );
    }

    /// Five representative objs from the three pre-existing emitters, reduced to
    /// `(length, CRC)` — a byte-level pin taken **before** the shared-primitive
    /// refactor that `emit_dyninit_obj` needed, so that refactor could be proved
    /// output-preserving rather than asserted to be.
    ///
    /// `emit_obj`, `emit_comdat_obj` and `emit_empty_obj` had their section
    /// layout, section-header writing and 11-symbol shell open-coded three times
    /// over; the dynamic-initializer obj needs a `.bss` section whose
    /// `SizeOfRawData` is non-zero while it contributes **no** file bytes, which
    /// touches all three of those. A fourth open-coded copy is this file's
    /// recorded defect shape (see the `emit_framed_obj` note above), so the
    /// copies were merged instead — and this test is what said the merge changed
    /// nothing. It is not a spec: if a *deliberate* change to one of those three
    /// emitters lands, re-derive these numbers from the reference obj, never from
    /// the port.
    fn obj_fingerprints() -> Vec<(&'static str, usize, u32)> {
        let mk_call = || Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(Frame { prolog_len: 0x0C, func_len: 0x24 }),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let mk_data = || Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA", is_function: false }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let mk_fp = || Function {
            is_float: true,
            fp_refs: vec![crate::codegen::FpConstRef {
                hi_off: 0,
                bits: 0x3FF0_0000_0000_0000,
                double: false,
            }],
            ..Function::plain("?fc@@YAMXZ", 0)
        };
        let blr = crate::codegen::encode_blr().to_vec();
        let objs: Vec<(&'static str, Vec<u8>)> = vec![
            ("empty", emit_empty_obj(r"Z:\tmp\anat\mvp.obj")),
            ("mvp", emit_mvp_obj(r"Z:\tmp\anat\mvp.obj", "?add3@@YAHHHH@Z", &[0u8; 12])),
            ("framed", emit_obj(r"Z:\t\f.obj", &[mk_call()], &[0u8; 0x24], 2536)),
            ("dataref", emit_obj(r"Z:\t\a7.obj", &[mk_data()], &[0u8; 16], 2536)),
            ("fppool", emit_obj(r"Z:\t\fc.obj", &[mk_fp()], &[0u8; 12], 2536)),
            (
                "comdat",
                emit_comdat_obj(
                    r"Z:\t\s.obj",
                    &[mk_call(), mk_data()],
                    &[vec![0u8; 0x24], vec![0u8; 16]],
                    2536,
                )
                .expect("no defined data"),
            ),
            (
                "comdat_plain",
                emit_comdat_obj(
                    r"Z:\x.obj",
                    &[
                        Function::plain("?SpewInit@@YAXXZ", 0),
                        Function::plain("?SpewTerminate@@YAXXZ", 0),
                    ],
                    &[blr.clone(), blr],
                    0,
                )
                .expect("no defined data"),
            ),
        ];
        objs.into_iter().map(|(k, o)| (k, o.len(), coff_checksum(&o))).collect()
    }

    #[test]
    fn the_three_pre_existing_emitters_are_byte_stable() {
        assert_eq!(
            obj_fingerprints(),
            vec![
                ("empty", 668, 0x7E17_6256u32),
                ("mvp", 790, 0x8036_8217),
                ("framed", 984, 0x529B_5631),
                ("dataref", 883, 0x187A_138D),
                ("fppool", 949, 0x1EEB_0597),
                ("comdat", 1207, 0xB4F7_683C),
                ("comdat_plain", 891, 0xC7A4_226B),
            ]
        );
    }

    #[test]
    fn drectve_is_132_bytes() {
        assert_eq!(DRECTVE.len(), 132, "drectve must be exactly 132 bytes");
    }

    #[test]
    fn s_compile2_is_57_bytes() {
        assert_eq!(S_COMPILE2.len(), 57);
    }

    /// The two lengths of the `+k` frame class, as a `Frame`.
    fn frame(func_len: u32) -> Frame {
        Frame { prolog_len: 0x0C, func_len }
    }

    #[test]
    fn pdata_unwind_word_encodes_function_and_prologue_lengths() {
        // 0x24 body (9 words, +k class) → BeginAddress 0 + big-endian
        // 0x40000903. 0x28 body (10 words, *5) → 0x40000A03 (length +1 = +0x100).
        assert_eq!(pdata_record(0, &frame(0x24)), [0, 0, 0, 0, 0x40, 0x00, 0x09, 0x03]);
        assert_eq!(pdata_record(0, &frame(0x28)), [0, 0, 0, 0, 0x40, 0x00, 0x0A, 0x03]);
        // The prologue field is the low byte and moves independently: the
        // two-call `r30`/`r31` body is 18 words with a 5-word prologue, and the
        // 100 KB-frame body 22 words with a 7-word one. Both read straight out
        // of reference objs; `build_pdata` hardcoded 3 until this landed.
        assert_eq!(
            pdata_record(0, &Frame { prolog_len: 0x14, func_len: 0x48 }),
            [0, 0, 0, 0, 0x40, 0x00, 0x12, 0x05]
        );
        assert_eq!(
            pdata_record(0, &Frame { prolog_len: 0x1C, func_len: 0x58 }),
            [0, 0, 0, 0, 0x40, 0x00, 0x16, 0x07]
        );
    }

    #[test]
    fn pdata_checksum_matches_reference_aux() {
        // The `.pdata` aux CheckSum in the reference obj (0xd3dfb2ce for the +k
        // frame) is the reflected CRC-32 of the 8 raw bytes.
        assert_eq!(coff_checksum(&build_pdata(&[&frame(0x24)])), 0xD3DF_B2CE);
        assert_eq!(coff_checksum(&build_pdata(&[&frame(0x28)])), 0xF8F2_E10D);
    }

    #[test]
    fn label_plan_matches_the_captured_counters() {
        let leaf = Function::plain("?L@@YAHH@Z", 0);
        let framed = |name| Function {
            frame: Some(frame(0x24)),
            ..Function::plain(name, 0)
        };
        // mvp_framed: one framed function, `.gl` counter 2536 → $M2545/6, $T2547.
        assert_eq!(
            plan_labels(2536, &[framed("?f@@YAHH@Z")], false),
            vec![Some([2545, 2546, 2547])]
        );
        // Under `/Gy` the same TU pays a flat 3-per-function surcharge first.
        assert_eq!(
            plan_labels(2536, &[framed("?f@@YAHH@Z")], true),
            vec![Some([2548, 2549, 2550])]
        );
        // A leading leaf consumes exactly one slot (`n1`: counter 2539 → 2549).
        assert_eq!(
            plan_labels(2539, &[leaf, framed("?F@@YAHH@Z")], false),
            vec![None, Some([2549, 2550, 2551])]
        );
        // Framed stride: 4 packed, 5 under `/Gy` (`m2`, counter 2539).
        let two = [framed("?F1@@YAHH@Z"), framed("?F2@@YAHH@Z")];
        assert_eq!(
            plan_labels(2539, &two, false),
            vec![Some([2548, 2549, 2550]), Some([2552, 2553, 2554])]
        );
        assert_eq!(
            plan_labels(2539, &two, true),
            vec![Some([2554, 2555, 2556]), Some([2559, 2560, 2561])]
        );
    }

    /// **The leading surcharge moves the function's OWN triple, not just the
    /// next one's** — which is the whole reason it is a separate field rather
    /// than a bigger stride, and the direction a "stride 7" model gets wrong.
    /// Allocating the same two slots after the triple instead of before it is
    /// **119 mismatches** in `scripts/sweep.d/98-cmp-order.py`, the same number
    /// as dropping them entirely: the total and the placement are two claims and
    /// this test is the one that pins the second.
    ///
    /// Measured: a signed `>`/`<` two-call comparator is stride 7 / lead 2 under
    /// `/Gy`, 6 / 2 packed (`scripts/gt_cmp_rr.py --stride`).
    #[test]
    fn a_leading_label_surcharge_moves_its_own_triple_and_every_later_one() {
        let cmp = |name| Function {
            frame: Some(frame(0x24)),
            label_lead: 2,
            ..Function::plain(name, 0)
        };
        let plain = |name| Function {
            frame: Some(frame(0x24)),
            ..Function::plain(name, 0)
        };
        // Packed: base 2545 (see the row above), + 2 for the lead, then the
        // following function starts 6 later rather than 4.
        assert_eq!(
            plan_labels(2536, &[cmp("?c@@YA_NPBU@Z"), plain("?f@@YAHH@Z")], false),
            vec![Some([2547, 2548, 2549]), Some([2551, 2552, 2553])]
        );
        // `/Gy`: the flat 3-per-function pre-pass, then the same +2 / stride 7.
        assert_eq!(
            plan_labels(2536, &[cmp("?c@@YA_NPBU@Z"), plain("?f@@YAHH@Z")], true),
            vec![Some([2553, 2554, 2555]), Some([2558, 2559, 2560])]
        );
        // A lead of 0 is the shipped behaviour, unchanged.
        assert_eq!(
            plan_labels(2536, &[plain("?f@@YAHH@Z")], false),
            vec![Some([2545, 2546, 2547])]
        );
    }

    /// **w-tu2's INTER-FUNCTION LABEL STRIDE, pinned against the allocator that
    /// already implements it** (board **#481**, this lane's **#503**).
    ///
    /// Lane `w-tu2` measured, through **real `c2`** at the workload's own
    /// `/O1 /Oi /EHsc /GR`, over a **36-cell** cross product with **six shapes
    /// held out before any fit** and **no free parameter**:
    ///
    /// > inter-function stride = `5 + 1·(leaf/tail fns between) + 5·(framed fns
    /// > between)`, and **the probe's own interior control flow does not enter
    /// > it at all**.
    ///
    /// It predicted `mmio.cpp`'s two label gaps exactly — 5 and 10 predicted,
    /// 5 and 10 observed, out of sample. It was registered as a prediction that
    /// **no** out-of-sample rule would hold, and it **MISSED**.
    ///
    /// # The rule needed no new home — it was already here, and this test says so
    ///
    /// [`plan_labels`] charges `5` per framed function under `/Gy` and `1` per
    /// leaf, in `.text` order. w-tu2's rule **is** that loop, measured from
    /// outside by somebody who did not read it. This test is the machine-checked
    /// pin, so the 36 real-`c2` cells can never silently disagree with the
    /// allocator again.
    ///
    /// # And it supplies the MECHANISM w-tu2 could only observe
    ///
    /// The reason interior control flow does not enter the stride is visible in
    /// three lines of [`plan_labels`]: `cur += f.label_lead` runs **before** the
    /// function's own triple is taken, so the *source* function's surcharge
    /// moves its own `$M` and everything after it **by the same amount** and
    /// **cancels out of any difference between two of its own successors**. That
    /// is asserted below as a cross product over the surcharges `w-label`
    /// actually measured, not as prose.
    ///
    /// # What is NOT claimed — board #482 / #286 stay open
    ///
    /// The cancellation is a property of the **source** function only. An
    /// **intervening** function's `label_lead` does **not** cancel, and the last
    /// assertion here proves it. So:
    ///
    /// * the stride from one framed function to a later one is derivable — the
    ///   part `mmio` needed, and it is in `crates/`;
    /// * **where a control-flow-bearing function's OWN labels start is still
    ///   not** — `label_lead` remains an input this file receives and cannot
    ///   derive, and `ifelse` at +3 with one `if` against `if3_ret` at +3 with
    ///   three is the cell that kills the obvious rule (#286).
    ///
    /// **Do not read this test as closing #286.** It closes the half w-tu2
    /// measured and machine-checks the boundary of the other half.
    #[test]
    fn the_inter_function_label_stride_is_a_constant_and_the_source_lead_cancels() {
        let framed = |name| Function {
            frame: Some(frame(0x24)),
            ..Function::plain(name, 0)
        };
        let leaf = |name| Function::plain(name, 0);
        // w-tu2's own cell shape: a probe framed function, then N intervening
        // functions, then the FIXED framed function `Z` whose `$M` is measured.
        // `first($M of Z) − first($M of probe)` is the quantity, self-normalizing
        // so the TU-level `.gl` seed and the `/Gy` pre-pass both drop out.
        let stride = |between: &[bool], lead: u32| -> u32 {
            let mut fns = vec![Function {
                label_lead: lead,
                ..framed("?probe@@YAHH@Z")
            }];
            for (i, is_framed) in between.iter().enumerate() {
                let n: &'static str = ["?b0@@YAHH@Z", "?b1@@YAHH@Z", "?b2@@YAHH@Z",
                                       "?b3@@YAHH@Z", "?b4@@YAHH@Z"][i];
                fns.push(if *is_framed { framed(n) } else { leaf(n) });
            }
            fns.push(framed("?Z@@YAHH@Z"));
            let p = plan_labels(2536, &fns, true);
            p.last().unwrap().unwrap()[0] - p[0].unwrap()[0]
        };
        // **The grid, not a cell.** Every arrangement of up to three intervening
        // functions, crossed with every intra-function surcharge `w-label`
        // measured on the source (`straight` 0, `if1_ret`/`and` 1,
        // `if2_ret`/`or`/`ternary` 2, `if3_ret`/`ifelse` 3, `dowhile` 4,
        // `while` 5). 15 arrangements × 6 surcharges = **90 cells**, and the
        // predicted value never mentions the surcharge.
        let mut cells = 0;
        for n in 0..=3usize {
            for bits in 0..(1u32 << n) {
                let between: Vec<bool> = (0..n).map(|i| bits >> i & 1 == 1).collect();
                let framed_between = between.iter().filter(|b| **b).count() as u32;
                let leaf_between = n as u32 - framed_between;
                let predicted = 5 + leaf_between + 5 * framed_between;
                for lead in [0, 1, 2, 3, 4, 5] {
                    assert_eq!(
                        stride(&between, lead),
                        predicted,
                        "w-tu2's rule, 36 real-c2 cells: stride = 5 + 1*{leaf_between} \
                         + 5*{framed_between}. The source function's own surcharge \
                         ({lead}) must CANCEL — it is charged before its own triple, \
                         so it moves that triple and every later one equally"
                    );
                    cells += 1;
                }
            }
        }
        // A printed count, not a status: a loop that ranged over nothing would
        // otherwise pass silently, which is this project's most-repeated defect.
        assert_eq!(cells, 90, "the grid must actually have been walked");

        // **`mmio.cpp`'s own two gaps, the out-of-sample prediction that made
        // w-tu2's prereg MISS.** Its six labels are $M3381,$M3382,$T3383 ·
        // $M3386,$M3387,$T3388 · $M3396,$M3397,$T3398 — gaps of 5 and 10, with
        // nothing between the first pair and 5 leaf stubs between the second.
        assert_eq!(stride(&[], 0), 5, "mmioGetInfo -> mmioSetInfo: 5 predicted, 5 observed");
        assert_eq!(
            stride(&[false, false, false, false, false], 0),
            10,
            "mmioSetInfo -> mmioClose across 5 leaf stubs: 10 predicted, 10 observed"
        );

        // **THE BOUNDARY, asserted rather than described.** An INTERVENING
        // function's surcharge does NOT cancel. The rule is blind to the source
        // function's interior control flow and is NOT blind to control flow in
        // general — #482/#286's open half, machine-checked so no later reader
        // can widen the claim by rereading the doc comment.
        let with_mid_lead = {
            let fns = vec![
                framed("?probe@@YAHH@Z"),
                Function { label_lead: 3, ..framed("?mid@@YAHH@Z") },
                framed("?Z@@YAHH@Z"),
            ];
            let p = plan_labels(2536, &fns, true);
            p[2].unwrap()[0] - p[0].unwrap()[0]
        };
        assert_eq!(
            with_mid_lead,
            5 + 5 + 3,
            "an INTERVENING function's intra-function surcharge is inside the \
             difference and does not cancel: the constant-stride rule needs \
             every function between (and the destination) to have lead 0, which \
             is exactly the condition w-tu2's 36 cells satisfied and did not \
             have to state"
        );
        assert_ne!(
            with_mid_lead, 10,
            "and that is why #286 is NARROWED, not CLOSED — deriving this cell \
             still needs the intra-function charge nothing in the port can \
             compute (`ifelse` +3 with one `if` vs `if3_ret` +3 with three)"
        );
    }

    #[test]
    fn framed_obj_has_six_sections_and_twenty_symbols() {
        // A framed obj built with the verified 0x24 text: 6 sections, 20 symbols.
        let text = vec![0u8; 0x24];
        let f = Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(frame(0x24)),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let obj = emit_obj(r"Z:\t\f.obj", &[f], &text, 2536);
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 6); // NumberOfSections
        assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 20); // NumberOfSymbols
    }

    #[test]
    fn debug_s_size_for_mvp_path() {
        // "Z:\tmp\anat\mvp.obj" = 19 chars → raw 97 → padded 100.
        let d = build_debug_s(r"Z:\tmp\anat\mvp.obj");
        assert_eq!(d.len(), 100);
    }

    // -----------------------------------------------------------------------
    // #137 — the PORTABLE pins for WR1's two ordering rules.
    //
    // WR1 landed 150 lines in this file and moved the workspace test-block total
    // (the attribute is spelled out in prose on purpose: `git grep -c` for it is
    // how §9.10 counts, and a literal in a comment inflates that count by one —
    // this lane's own first tally read 580 blocks against 579 running tests)
    // by **zero** (`docs/ROADMAP.md` §9.10). Its two ordering rules were pinned
    // only by `fixtures/cpp/wr1_sym_addr.cpp`, and the mutation table in §9.12
    // shows what that was worth: with the address rule inverted, or with the
    // REFLO offset forced back to `hi_off + 4`, `cargo test --workspace` is
    // **571 passed / 0 failed in BOTH lanes** — the portable one *and* the one
    // with the toolchain resolving, because `differential.rs` names three
    // fixtures and `wr1_sym_addr.cpp` is not among them. Only `scripts/gate.sh`
    // went red (10 of 12 lanes). These tests move that pin into `cargo test`.
    // -----------------------------------------------------------------------

    /// A COFF section header field reader, used only by the tests below.
    /// Deliberately a *separate* walk of the container from the emitter's — the
    /// point of a pin is that it fails when the emitter changes, so it must not
    /// share the emitter's arithmetic.
    fn text_relocations(obj: &[u8]) -> Vec<(u32, u32, u16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let n_sections = u16at(2) as usize;
        let mut out = Vec::new();
        for s in 0..n_sections {
            let h = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
            if &obj[h..h + 5] != b".text" {
                continue;
            }
            let ptr = u32at(h + 24) as usize;
            let n = u16at(h + 32) as usize;
            for r in 0..n {
                let o = ptr + r * 10;
                out.push((u32at(o), u32at(o + 4), u16at(o + 8)));
            }
        }
        out
    }

    /// **#137 rule 2 — the REFHI/REFLO quad's halves are NOT adjacent.**
    ///
    /// The `lis rS,sym@ha` is hoisted to the top of the body while the
    /// `addi rD,rS,sym@l` is emitted after the rest of the argument setup, so a
    /// literal slot lands *between* them and REFLO is **not** at `hi_off + 4`.
    /// MEASURED, `work/wr1/probes/p4.cpp`: `void a7(){ gsp(&gI, 7); }` is
    /// `lis r11 · li r4,7 · addi r3,r11,0 · b`, REFLO **eight** bytes past
    /// REFHI. Emitting the quad as the adjacent pair a pooled FP constant uses
    /// was a live wrong-bytes emit on exactly that body.
    ///
    /// The input here is that body's shape and nothing else: `hi_off` 0 and
    /// `lo_off` 8, four words of `.text`. Every assertion carries its own
    /// message and the two quantities the later ones rest on — how many
    /// relocation records the section has, and that `hi_off + 4` is a real
    /// offset inside the body rather than past its end — are pinned first, so a
    /// broken reader goes red on its own line instead of making the offset
    /// assertions unreachable.
    #[test]
    fn the_data_address_quad_puts_reflo_at_its_own_offset_not_beside_refhi() {
        let text = vec![0u8; 16]; // lis · li · addi · b
        let f = Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA", is_function: false }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let obj = emit_obj(r"Z:\t\a7.obj", &[f], &text, 2536);
        let recs = text_relocations(&obj);

        // (a) The fixture property, pinned over the INPUT and not over the rule
        // under test: the two halves are 8 bytes apart, so `hi_off + 4` is a
        // different word of a body that actually has one there. Without this the
        // test could be satisfied by a body too short to tell the two apart.
        assert_eq!(
            (0u32, 8u32, text.len()),
            (0, 8, 16),
            "(a) the discriminating body is `lis · li · addi · b` with the halves \
             8 bytes apart and a real word at +4"
        );
        // (b) One REL24 for the branch plus the quad — and nothing else. Pinned
        // before any record is inspected by index.
        assert_eq!(
            recs.len(),
            5,
            "(b) expected 5 .text relocation records (1 REL24 + a REFHI/PAIR/\
             REFLO/PAIR quad), got {}",
            recs.len()
        );
        // (c) REFHI sits at the hoisted `lis`, offset 0.
        let refhi: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFHI).map(|r| r.0).collect();
        assert_eq!(refhi, vec![0], "(c) REFHI is not at the hoisted `lis` (offset 0): {refhi:?}");
        // (d) **The rule.** REFLO is at the `addi`'s own offset, 8 — NOT at
        // `hi_off + 4` = 4, which is where the literal's `li` is.
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![8],
            "(d) REFLO must be at the `addi`'s own offset 8, not at hi_off+4 = 4 \
             — the two halves of the quad are NOT adjacent: {reflo:?}"
        );
        // (e) Both PAIRs shadow their own half, and against symbol index 0.
        let pairs: Vec<(u32, u32)> =
            recs.iter().filter(|r| r.2 == REL_PPC_PAIR).map(|r| (r.0, r.1)).collect();
        assert_eq!(
            pairs,
            vec![(0, 0), (8, 0)],
            "(e) each PAIR shadows its own half's offset against symbol 0: {pairs:?}"
        );
        // (f) Records are ascending by VirtualAddress and REFHI precedes its
        // PAIR at the equal VA — the order c2 writes them in.
        let order: Vec<(u32, u16)> = recs.iter().map(|r| (r.0, r.2)).collect();
        assert_eq!(
            order,
            vec![
                (0, REL_PPC_REFHI),
                (0, REL_PPC_PAIR),
                (8, REL_PPC_REFLO),
                (8, REL_PPC_PAIR),
                (12, REL_PPC_REL24),
            ],
            "(f) the .text relocation records are not in ascending-VA order with \
             REFHI ahead of its PAIR: {order:?}"
        );
    }

    /// The same rule in the **`/Gy` COMDAT** emitter, which is a second copy of
    /// the quad code — and a second copy of one fact is this file's recorded
    /// defect shape (see the `emit_framed_obj` note above). One emitter fixed
    /// and one not is exactly how the `.pdata`-ordering bug survived.
    #[test]
    fn the_comdat_emitter_places_reflo_at_its_own_offset_too() {
        let text = vec![0u8; 16];
        let f = Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA", is_function: false }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let obj = emit_comdat_obj(r"Z:\t\a7.obj", &[f], &[text], 2536).expect("no defined data");
        let recs = text_relocations(&obj);
        assert_eq!(
            recs.len(),
            5,
            "(g) the COMDAT emitter wrote {} .text relocation records, expected 5",
            recs.len()
        );
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![8],
            "(h) COMDAT emitter: REFLO must be at the `addi`'s own offset 8, not \
             at hi_off+4 = 4: {reflo:?}"
        );
    }

    /// Every COFF symbol record's `(name, Value, SectionNumber)`, in table
    /// order. A second walk of the container, like [`text_relocations`].
    fn symbols(obj: &[u8]) -> Vec<(String, u32, i16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let ptr = u32at(8) as usize;
        let n = u32at(12) as usize;
        let strtab = ptr + n * 18;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let r = ptr + i * 18;
            let name = if u32at(r) == 0 {
                let off = strtab + u32at(r + 4) as usize;
                let end = obj[off..].iter().position(|&c| c == 0).unwrap_or(0) + off;
                String::from_utf8_lossy(&obj[off..end]).into_owned()
            } else {
                let raw = &obj[r..r + 8];
                let end = raw.iter().position(|&c| c == 0).unwrap_or(8);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            };
            out.push((name, u32at(r + 8), u16at(r + 12) as i16));
            i += 1 + obj[r + 17] as usize;
        }
        out
    }

    /// **#135/#137 — the compiler-label triple's three slots are not
    /// interchangeable, and the symbol table emits the two `$M` out of numeric
    /// order.** Asserted in BOTH emitters.
    ///
    /// `plan_labels` hands back `[n, n+1, n+2]` and the emitter binds them:
    /// `$M(n)` carries the **prologue** length, `$M(n+1)` the **function**
    /// length, `$T(n+2)` the `.pdata` record — and the two `$M` records are
    /// written `$M(n+1)` **first**, `$M(n)` second, with the callee external
    /// between them. Nothing pinned either fact portably; swapping the two
    /// `Value`s is six wrong bytes in an obj that still links, which is this
    /// file's recorded defect class (#5).
    ///
    /// **Both emitters, because there are two copies of this binding** — and
    /// the first draft of this test called only [`emit_comdat_obj`], under which
    /// swapping the two `$M` in [`emit_obj`] left `cargo test` **85 passed / 0
    /// failed**. One rule in two emitters, pinned in one, is how the `.pdata`
    /// ordering bug survived (see the `emit_framed_obj` note above).
    ///
    /// The number→meaning half is **independently confirmed by `.cod`**
    /// (`scripts/gt_label_cod.py`, `docs/ROADMAP.md` §9.12): on 56 of 56 graded
    /// bodies across 20 shapes and 4 flag sets the listing prints `$M(n)` at a
    /// **lower** text offset than `$M(n+1)` in the same body — the prologue end
    /// really is the lower number. Measured on both sides of the seam.
    #[test]
    fn the_label_triple_binds_prolog_to_n_and_function_length_to_n_plus_one() {
        let mk = || Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(Frame { prolog_len: 0x0C, func_len: 0x24 }),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let text = vec![0u8; 0x24];

        // (l) The triples this obj is supposed to carry, pinned against
        // `plan_labels` itself so the assertions below name real symbols. If
        // the planner moves, this line goes red rather than the later ones
        // silently comparing `None` to `None`. Packed is 4 lower than `/Gy`:
        // 2536 + LABEL_SEED_GAP = 2545, plus the flat 3-per-function pre-pass.
        let planned = |comdat| {
            plan_labels(2536, &[mk()], comdat)[0].expect("a framed function gets a triple")
        };
        assert_eq!(
            (planned(false), planned(true)),
            ([2545, 2546, 2547], [2548, 2549, 2550]),
            "(l) the planned triple moved: packed {:?}, /Gy {:?}",
            planned(false),
            planned(true)
        );

        for (tag, obj, m) in [
            ("packed", emit_obj(r"Z:\t\f.obj", &[mk()], &text, 2536), planned(false)),
            (
                "/Gy",
                emit_comdat_obj(r"Z:\t\f.obj", &[mk()], &[text.clone()], 2536)
                    .expect("no defined data"),
                planned(true),
            ),
        ] {
            let syms = symbols(&obj);
            let n0 = label_name('M', m[0]);
            let n1 = label_name('M', m[1]);
            let n2 = label_name('T', m[2]);
            let ix = |n: &str| syms.iter().position(|s| s.0 == n);
            let val = |n: &str| syms.iter().find(|s| s.0 == n).map(|s| s.1);

            // (m) All three symbols are present, under `label_name`'s spelling.
            for n in [&n0, &n1, &n2] {
                assert!(ix(n).is_some(), "(m) {tag}: the obj has no symbol named {n}");
            }

            // (n) **The binding.** `$M(n)` is the PROLOGUE length and `$M(n+1)`
            // the FUNCTION length — not the other way round.
            assert_eq!(
                (val(&n0), val(&n1)),
                (Some(0x0C), Some(0x24)),
                "(n) {tag}: $M(n)={n0} must carry the prologue length 0x0C and \
                 $M(n+1)={n1} the function length 0x24 — swapping them is six \
                 wrong bytes in an obj that still links"
            );

            // (o) **The emission order**, the opposite of the numeric order:
            // `$M(n+1)` is written BEFORE `$M(n)`, and `$T(n+2)` after both.
            let (a, b, c) = (ix(&n1).unwrap(), ix(&n0).unwrap(), ix(&n2).unwrap());
            assert!(
                a < b && b < c,
                "(o) {tag}: the symbol table must carry $M(n+1) before $M(n) \
                 before $T(n+2); got {n1} at {a}, {n0} at {b}, {n2} at {c}"
            );

            // (o2) …with the callee external BETWEEN the two `$M`.
            let callee = ix("?g@@YAHH@Z")
                .unwrap_or_else(|| panic!("(o2) {tag}: the callee symbol is missing"));
            assert!(
                a < callee && callee < b,
                "(o2) {tag}: the callee external sits between $M(n+1) and $M(n): \
                 {n1} at {a}, callee at {callee}, {n0} at {b}"
            );

            // (p) `$T(n+2)` is the `.pdata` record's own label and is the only
            // member of the triple that leaves the code section.
            let t_sec = syms.iter().find(|s| s.0 == n2).map(|s| s.2).unwrap();
            let m_sec = syms.iter().find(|s| s.0 == n0).map(|s| s.2).unwrap();
            assert_ne!(
                t_sec, m_sec,
                "(p) {tag}: $T(n+2) must live in `.pdata`, not beside the two $M \
                 in `.text` (both read section {t_sec})"
            );
        }
    }

    // =======================================================================
    // #158 — the dynamic-initializer obj.
    //
    // PORTABLE pins (prereg D2). `cargo test` has twice missed an ordering bug
    // in this file that only `scripts/gate.sh` caught — the callee-per-call-site
    // inflation and the batched-relocations layout — because the shapes that
    // discriminate them were reachable only through a fixture. Everything below
    // runs with **no toolchain**: `emit_dyninit_obj` plus a parser written here,
    // deliberately a separate walk of the container from the emitter's.
    // =======================================================================

    /// Every section header, as `(name, SizeOfRawData, PointerToRawData,
    /// PointerToRelocations, NumberOfRelocations, VirtualSize, Characteristics)`.
    fn sections_of(obj: &[u8]) -> Vec<(String, u32, u32, u32, u16, u32, u32)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        (0..u16at(2) as usize)
            .map(|s| {
                let h = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
                let end = obj[h..h + 8].iter().position(|&c| c == 0).unwrap_or(8);
                (
                    String::from_utf8_lossy(&obj[h..h + end]).into_owned(),
                    u32at(h + 16),
                    u32at(h + 20),
                    u32at(h + 24),
                    u16at(h + 32),
                    u32at(h + 8),
                    u32at(h + 36),
                )
            })
            .collect()
    }

    /// Every relocation record of the named section, in file order.
    fn relocations_of(obj: &[u8], want: &str) -> Vec<(u32, u32, u16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let mut out = Vec::new();
        for s in sections_of(obj).iter() {
            if s.0 != want {
                continue;
            }
            for r in 0..s.4 as usize {
                let o = s.3 as usize + r * RELOC_LEN;
                out.push((u32at(o), u32at(o + 4), u16at(o + 8)));
            }
        }
        out
    }

    /// Every symbol record as `(name, Value, SectionNumber, Type, StorageClass,
    /// nAux)`, aux records skipped — plus, for a symbol that has one, its aux
    /// decoded as `(Length, nReloc, CheckSum, Number, Selection)`.
    #[allow(clippy::type_complexity)]
    fn symbols_full(
        obj: &[u8],
    ) -> Vec<((String, u32, i16, u16, u8, u8), Option<(u32, u16, u32, u16, u8)>)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let ptr = u32at(8) as usize;
        let n = u32at(12) as usize;
        let strtab = ptr + n * SYMBOL_LEN;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let r = ptr + i * SYMBOL_LEN;
            let name = if u32at(r) == 0 {
                let off = strtab + u32at(r + 4) as usize;
                let end = obj[off..].iter().position(|&c| c == 0).unwrap_or(0) + off;
                String::from_utf8_lossy(&obj[off..end]).into_owned()
            } else {
                let raw = &obj[r..r + 8];
                let end = raw.iter().position(|&c| c == 0).unwrap_or(8);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            };
            let naux = obj[r + 17];
            let aux = if naux == 1 {
                let a = r + SYMBOL_LEN;
                Some((u32at(a), u16at(a + 4), u32at(a + 8), u16at(a + 12), obj[a + 14]))
            } else {
                None
            };
            out.push((
                (name, u32at(r + 8), u16at(r + 12) as i16, u16at(r + 14), obj[r + 16], naux),
                aux,
            ));
            i += 1 + naux as usize;
        }
        out
    }

    /// The `.text$yc` payload shared byte-for-byte by the fixture and both
    /// workload TUs (`docs/OBJ_DYNINIT_SHAPE.md` §3.3):
    /// `lis r11 · lis r10 · addi r4,r11 · addi r3,r10 · li r5,0 · b -0x14`.
    const DYNINIT_TEXT: [u8; 0x18] = [
        0x3d, 0x60, 0x00, 0x00, 0x3d, 0x40, 0x00, 0x00, 0x38, 0x8b, 0x00, 0x00, 0x38, 0x6a, 0x00,
        0x00, 0x38, 0xa0, 0x00, 0x00, 0x4b, 0xff, 0xff, 0xec,
    ];

    /// The reference cell: `fixtures/cpp/il_dyninit_static.cpp`,
    /// `struct L { L(const char*, int); }; static L sL("abc", 0);` at
    /// `/O1 /Oi /EHsc /GS- /c`.
    fn fixture_obj() -> Vec<u8> {
        let lit = StringLiteral { bytes: b"abc\0" };
        let name = string_comdat_name(lit.bytes).expect("the fixture literal is representable");
        let thunk = DynInitThunk {
            name: "??__EsL@@YAXXZ",
            text: &DYNINIT_TEXT,
            calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
            data_refs: vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: &name, is_function: false },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL", is_function: false },
            ],
        };
        let object = BssObject {
            symbol: "sL",
            size: 1,
            natural_align: 1,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        emit_dyninit_obj(r"Z:\tmp\anat\mvp.obj", &thunk, Some(&lit), &object)
            .expect("the reference cell is in class")
    }

    /// **The eight verified literals**, name for name. The hash column is
    /// `docs/OBJ_DYNINIT_SHAPE.md` §5; the full names are the ones the reference
    /// objs' symbol tables carry.
    #[test]
    fn the_string_comdat_name_matches_every_measured_literal() {
        // The 101-byte held-out cell, built rather than typed: a 7-digit hash
        // (the leading `A` suppressed) and an escaped text cut at 32 source
        // bytes. Miscounting the `q`s by one silently grades a different cell.
        let q100 = {
            let mut v = vec![b'q'; 100];
            v.push(0);
            v
        };
        assert_eq!(q100.len(), 101);
        assert_eq!(
            string_comdat_name(&q100).as_deref(),
            Some("??_C@_0GF@LHLJLME@qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq@"),
            "the 101-byte cell: 7 hash digits, and the text cut at 32 source bytes"
        );

        let cases: [(&[u8], &str); 7] = [
            (b"abc\0", "??_C@_03FIKCJHKP@abc?$AA@"),
            (b"defg\0", "??_C@_04DPHBJEKM@defg?$AA@"),
            (b"\0", "??_C@_00CNPNBAHC@?$AA@"),
            (b"Hello, world!\0", "??_C@_0O@GEHPLBPJ@Hello?0?5world?$CB?$AA@"),
            (b"xyzzy\0", "??_C@_05POJHDMIP@xyzzy?$AA@"),
            (
                b"system/src/synth/tomcrypt\0",
                "??_C@_0BK@PELMDOBM@system?1src?1synth?1tomcrypt?$AA@",
            ),
            (b"system/src/zlib\0", "??_C@_0BA@FFMAKHEN@system?1src?1zlib?$AA@"),
        ];
        for (bytes, want) in cases {
            assert_eq!(
                string_comdat_name(bytes).as_deref(),
                Some(want),
                "literal {:?}",
                String::from_utf8_lossy(&bytes[..bytes.len() - 1])
            );
        }
    }

    /// **The swapped-init trap, made a test.** §2.3 closes by naming it: the
    /// same polynomial appears twice with different initial values — section
    /// aux CheckSum init `0`, string-name hash init `0xFFFFFFFF` — and getting
    /// them the wrong way round is the obvious way to implement this wrong.
    /// Both values are 32 bits of noise, so nothing else in the port notices.
    #[test]
    fn the_two_crc_initial_values_are_not_interchangeable() {
        for lit in [&b"abc\0"[..], b"defg\0", b"xyzzy\0", b"system/src/zlib\0"] {
            assert_ne!(
                coff_checksum(lit),
                jamcrc(lit),
                "the aux checksum and the name hash must not coincide on {lit:?}"
            );
        }
        // The measured pairs, both directions pinned on one literal.
        assert_eq!(jamcrc(b"abc\0"), 0x58A2_97AF, "JamCRC uses init 0xFFFFFFFF");
        assert_eq!(coff_checksum(b"abc\0"), 0x8619_B74C, "the aux CheckSum uses init 0");
        assert_eq!(jamcrc(b"defg\0"), 0x3F71_94AC);
        assert_eq!(coff_checksum(b"defg\0"), 0x06AC_9C4E);
        assert_eq!(jamcrc(b"xyzzy\0"), 0xFE97_3C8F);
        assert_eq!(coff_checksum(b"xyzzy\0"), 0xB0AA_62D3);
        // …and the two `.XBLD$W` constants, which predate this lane, are init-0.
        assert_eq!(coff_checksum(&XBLD_C2), XBLD_C2_CHECKSUM);
        assert_eq!(coff_checksum(&XBLD_C1), XBLD_C1_CHECKSUM);
    }

    /// **The refusal, which is the deliberate part.** `?2`, `?6`, `?7` and `?8`
    /// are single-`?` escape slots this lane never observed a character in. A
    /// byte that takes one of them in real c2 would be rendered here as a
    /// two-digit `?$XX`, and the COMDAT name, the length field and the obj's
    /// whole string table would be wrong with nothing to flag it — so any byte
    /// outside the measured set refuses the name, and the caller refuses the obj.
    #[test]
    fn an_unmeasured_escape_byte_refuses_the_name_rather_than_guessing() {
        // Backslash, newline, tab, apostrophe, `<`, `%`, `#`, and a high byte:
        // all plausible occupants of ?2/?6/?7/?8 or of an unverified `?$XX`.
        for b in [b'\\', b'\n', b'\t', b'\'', b'<', b'%', b'#', 0xE9] {
            let lit = [b'a', b, 0];
            assert_eq!(
                string_comdat_name(&lit),
                None,
                "byte {b:#04x} has no measured escape and must refuse"
            );
        }
        // A missing NUL refuses too — it is part of the length, the hash and the
        // text, so a caller that dropped it gets a name wrong in three places.
        assert_eq!(string_comdat_name(b"abc"), None);
        assert_eq!(string_comdat_name(b""), None);
        // …and the whole obj declines with it.
        let lit = StringLiteral { bytes: b"a\\b\0" };
        let thunk = DynInitThunk {
            name: "??__EsL@@YAXXZ",
            text: &DYNINIT_TEXT,
            calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
            data_refs: vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: "unused", is_function: false },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL", is_function: false },
            ],
        };
        let object = BssObject {
            symbol: "sL",
            size: 1,
            natural_align: 1,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        assert_eq!(
            emit_dyninit_obj(r"Z:\t\x.obj", &thunk, Some(&lit), &object).map(|o| o.len()),
            None,
            "an unrepresentable literal must decline the whole obj"
        );
    }

    /// **CORRECTION to §5's "truncated at 32 characters".** The limit is on the
    /// *source* bytes of `literal + NUL`, not on the escaped output. Three
    /// discriminating cells, none of which the doc's reading gets right.
    #[test]
    fn the_escaped_text_is_cut_at_thirty_two_source_bytes_not_output_characters() {
        // 31 source characters = 32 bytes with the NUL → the `?$AA` IS rendered.
        let n31 = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0";
        assert_eq!(n31.len(), 32);
        let s31 = string_comdat_name(n31).unwrap();
        assert!(s31.ends_with("?$AA@"), "31 chars + NUL keeps the NUL escape: {s31}");
        // 32 source characters = 33 bytes → the NUL is DROPPED.
        let n32 = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0";
        assert_eq!(n32.len(), 33);
        let s32 = string_comdat_name(n32).unwrap();
        assert!(!s32.ends_with("?$AA@"), "32 chars + NUL drops the NUL escape: {s32}");
        assert!(s32.ends_with("aaaa@"));
        // A 30-character all-`/` literal escapes to 2 characters each — 62
        // escaped characters from 31 source bytes, and NOTHING is cut. Reading
        // the limit as an output-character budget truncates this one mid-name.
        let slashes = b"//////////////////////////////\0";
        assert_eq!(slashes.len(), 31);
        let s = string_comdat_name(slashes).unwrap();
        assert_eq!(s.matches("?1").count(), 30, "all 30 slashes must survive: {s}");
        assert!(s.ends_with("?$AA@"), "and so must the NUL: {s}");
    }

    /// **(a) Section order**, with `.bss` then `.CRT$XCU` always last (§4.1).
    /// At `/O1` `.text$yc` precedes `.rdata`; at `/Ox` it is the other way round
    /// and the obj is a different shape entirely, which is why the grading flags
    /// matter (§7.3 caveat 1). Prereg P1 got the code section's *name* wrong —
    /// it is `.text$yc`, not `.text`.
    #[test]
    fn dyninit_section_order_puts_bss_and_crt_xcu_last() {
        let obj = fixture_obj();
        let names: Vec<String> = sections_of(&obj).into_iter().map(|s| s.0).collect();
        assert_eq!(
            names,
            vec![
                ".drectve", ".debug$S", ".XBLD$W", ".XBLD$W", ".text$yc", ".rdata", ".bss",
                ".CRT$XCU"
            ],
            "(a) the eight sections, in order"
        );
        let ix = |n: &str| names.iter().rposition(|s| s == n).unwrap();
        assert!(
            ix(".text$yc") < ix(".rdata") && ix(".rdata") < ix(".bss") && ix(".bss") < ix(".CRT$XCU"),
            "(a) .text$yc < .rdata < .bss < .CRT$XCU"
        );
        assert_eq!(ix(".CRT$XCU"), names.len() - 1, "(a) .CRT$XCU is last");
        assert_eq!(ix(".bss"), names.len() - 2, "(a) .bss is second to last");
        // Characteristics, per §2.1/§4.2: ALIGN_4 `.rdata` (n=4, t=1) and
        // ALIGN_1 `.bss` (n=1, t=1).
        let ch: Vec<u32> = sections_of(&obj).into_iter().map(|s| s.6).collect();
        assert_eq!(ch[4], 0x6040_1020, "(a) .text$yc characteristics");
        assert_eq!(ch[5], 0x4030_1040, "(a) .rdata characteristics, ALIGN_4");
        assert_eq!(ch[6], 0xC010_0080, "(a) .bss characteristics, ALIGN_1");
        assert_eq!(ch[7], 0xC030_0040, "(a) .CRT$XCU characteristics");
    }

    /// **(b) The undefined external constructor sits at index 14** — inside the
    /// `.text$yc` group and *before* the `.rdata` section symbol at 15.
    ///
    /// This is the ordering rule of §3.1 (the symbol table follows section
    /// order; per section, the section symbol + aux, then what it defines, then
    /// any undefined external it is the first to reference), and it is **not**
    /// where either pre-existing emitter puts an undefined external — both put
    /// callees after the defining function with no interleaved section group to
    /// get wrong. Placing the constructor after the `.rdata` group instead
    /// shifts three symbol indices, which every relocation would still resolve
    /// against: a wrong obj no linker complains about, this file's recorded
    /// defect class.
    #[test]
    fn the_undefined_constructor_sits_inside_the_text_yc_group_at_index_fourteen() {
        let obj = fixture_obj();
        let syms = symbols_full(&obj);
        // Flatten to raw record indices so "index 14" means the COFF index.
        let mut at: Vec<String> = Vec::new();
        for (s, aux) in &syms {
            at.push(s.0.clone());
            if aux.is_some() {
                at.push(format!("<aux of {}>", s.0));
            }
        }
        assert_eq!(at.len(), 24, "(b) 24 symbol records");
        assert_eq!(at[11], ".text$yc", "(b) the .text$yc section symbol is at 11");
        assert_eq!(at[13], "??__EsL@@YAXXZ", "(b) the thunk is at 13");
        assert_eq!(
            at[14], "??0L@@QAA@PBDH@Z",
            "(b) the undefined external constructor is at 14, inside the \
             .text$yc group"
        );
        assert_eq!(at[15], ".rdata", "(b) and BEFORE the .rdata section symbol at 15");
        // The constructor really is undefined and really is a function.
        let ctor = syms.iter().find(|(s, _)| s.0 == "??0L@@QAA@PBDH@Z").unwrap();
        assert_eq!(
            (ctor.0 .2, ctor.0 .3, ctor.0 .4),
            (0, 0x0020, 2),
            "(b) the constructor is SectionNumber 0, Type 0x0020, EXTERNAL"
        );
    }

    /// **(c) The relocation record order on `.text$yc`.**
    ///
    /// Nine records: the HI block (VA 0, 4) entirely before the LO block
    /// (VA 8, 12) — the halves are **not** adjacent — a PAIR after every REFHI
    /// *and* every REFLO with `SymbolTableIndex` 0, and **no** PAIR after the
    /// REL24. Prereg P5 predicted 5, and its registered alternative "7, a PAIR
    /// after each REFHI" was wrong too.
    ///
    /// The block separation is asserted as a property, not as fixed positions:
    /// §3.2's `L(float)` row is a cell where the HI and LO blocks name their
    /// symbols in *different* orders, so the emitter derives this by sorting on
    /// offset and the test checks the sorted consequence.
    #[test]
    fn the_dyninit_relocations_pair_both_halves_and_leave_rel24_bare() {
        let obj = fixture_obj();
        let recs = relocations_of(&obj, ".text$yc");
        assert_eq!(recs.len(), 9, "(c) nine .text$yc relocation records");
        assert_eq!(
            recs,
            vec![
                (0x00, 17, REL_PPC_REFHI),
                (0x00, 0, REL_PPC_PAIR),
                (0x04, 20, REL_PPC_REFHI),
                (0x04, 0, REL_PPC_PAIR),
                (0x08, 17, REL_PPC_REFLO),
                (0x08, 0, REL_PPC_PAIR),
                (0x0c, 20, REL_PPC_REFLO),
                (0x0c, 0, REL_PPC_PAIR),
                (0x14, 14, REL_PPC_REL24),
            ],
            "(c) the nine records, transcribed from the reference obj"
        );
        // The same facts as properties, so a future cell with a different
        // symbol order inside a block still grades.
        let hi: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFHI).map(|r| r.0).collect();
        let lo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert!(
            hi.iter().max() < lo.iter().min(),
            "(c) the whole HI block precedes the whole LO block: {hi:?} then {lo:?}"
        );
        for (i, r) in recs.iter().enumerate() {
            if r.2 == REL_PPC_REFHI || r.2 == REL_PPC_REFLO {
                let p = recs[i + 1];
                assert_eq!(
                    (p.0, p.1, p.2),
                    (r.0, 0, REL_PPC_PAIR),
                    "(c) record {i} must be followed by a PAIR at its own VA against symbol 0"
                );
            }
            if r.2 == REL_PPC_REL24 {
                assert!(
                    recs.get(i + 1).map(|n| n.2) != Some(REL_PPC_PAIR),
                    "(c) REL24 takes no PAIR"
                );
            }
        }
        // `.CRT$XCU`: one ADDR32 at offset 0 against the thunk at 13 (§3.4).
        assert_eq!(
            relocations_of(&obj, ".CRT$XCU"),
            vec![(0, 13, REL_PPC_ADDR32)],
            "(c) .CRT$XCU carries one ADDR32 -> the thunk — the pre-existing \
             emitters assume only .text carries relocations, and here that is false"
        );
    }

    /// **(d) The `.bss` inversion** — prereg P8, refuted, and the single most
    /// likely wrong-bytes trap in this shape.
    ///
    /// `SizeOfRawData` carries the object's size, `VirtualSize` is 0,
    /// `PointerToRawData` is 0, the aux `Length` is the size and the aux
    /// `Selection` is 0 (never a COMDAT) — **and the section contributes zero
    /// bytes to the file**, so `.rdata` and `.CRT$XCU` are contiguous across it.
    /// The natural implementation puts the size in `VirtualSize`, and every
    /// other emitter in this file equates "the section's length" with
    /// `raw.len()` in four separate places.
    #[test]
    fn the_bss_section_declares_its_size_but_occupies_no_file_bytes() {
        let obj = fixture_obj();
        let secs = sections_of(&obj);
        let (name, size, ptr_raw, ptr_rel, n_rel, vsize, _ch) = secs[6].clone();
        assert_eq!(name, ".bss");
        assert_eq!(size, 1, "(d) SizeOfRawData carries `sizeof`");
        assert_eq!(vsize, 0, "(d) VirtualSize is 0 — the P8 inversion");
        assert_eq!(ptr_raw, 0, "(d) PointerToRawData is 0");
        assert_eq!((ptr_rel, n_rel), (0, 0), "(d) .bss has no relocations");
        // The aux record.
        let bss_aux = symbols_full(&obj)
            .into_iter()
            .find(|(s, a)| s.0 == ".bss" && a.is_some())
            .and_then(|(_, a)| a)
            .expect("(d) .bss has a section symbol with one aux");
        assert_eq!(bss_aux.0, 1, "(d) aux Length is the object size");
        assert_eq!(bss_aux.1, 0, "(d) aux nReloc");
        assert_eq!(bss_aux.2, 0, "(d) aux CheckSum is 0 for .bss");
        assert_eq!(bss_aux.4, 0, "(d) aux Selection 0 — .bss is NEVER a COMDAT");
        // **Zero file bytes.** `.CRT$XCU` starts exactly where `.rdata`'s own
        // relocations would end — here `.rdata` has none, so immediately after
        // `.rdata`'s raw data, with no gap for `.bss`.
        let text = &secs[4];
        let rdata = &secs[5];
        let crt = &secs[7];
        assert_eq!(
            text.3,
            text.2 + text.1,
            "(d) .text$yc relocations follow its own raw data"
        );
        assert_eq!(
            rdata.2,
            text.3 + 9 * RELOC_LEN as u32,
            "(d) .rdata follows .text$yc's nine relocation records"
        );
        assert_eq!(
            crt.2,
            rdata.2 + rdata.1,
            "(d) .CRT$XCU follows .rdata with NO gap — .bss contributed nothing"
        );
        // A larger object moves only the declared size, never the file layout.
        let big = {
            let lit = StringLiteral { bytes: b"abc\0" };
            let name = string_comdat_name(lit.bytes).unwrap();
            let thunk = DynInitThunk {
                name: "??__EsL@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
                data_refs: vec![
                    DataRef { hi_off: 0x00, lo_off: 0x08, name: &name, is_function: false },
                    DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL", is_function: false },
                ],
            };
            let object = BssObject {
                symbol: "sL",
                size: 0x1000,
                natural_align: 4,
                external: false,
                initializer_symbol: "sL$initializer$",
            };
            emit_dyninit_obj(r"Z:\tmp\anat\mvp.obj", &thunk, Some(&lit), &object).unwrap()
        };
        assert_eq!(
            big.len(),
            obj.len(),
            "(d) a 0x1000-byte object must not add a single byte to the file"
        );
        let bs = sections_of(&big);
        assert_eq!(bs[6].1, 0x1000, "(d) …only SizeOfRawData moves");
        assert_eq!(bs[6].6, 0xC040_0080, "(d) …and the alignment nibble, to ALIGN_8");
        assert_eq!(bs[7].2, crt.2, "(d) .CRT$XCU stays at the same file offset");
    }

    /// The whole reference cell, all 24 symbol records and both aux fields that
    /// vary, against `docs/OBJ_DYNINIT_SHAPE.md` §3.1's table.
    ///
    /// Storage classes are the part that is easy to get backwards and the part
    /// the workload TUs discriminate: the thunk is **STATIC** with `Type`
    /// 0x0020 even though an ordinary function is EXTERNAL; the string COMDAT's
    /// defining symbol is **EXTERNAL** with `Type` 0 so the linker can fold it;
    /// a `static` object's `.bss` symbol is STATIC and undecorated while a
    /// non-`static` one is EXTERNAL and decorated; `<name>$initializer$` is
    /// STATIC and undecorated either way.
    #[test]
    fn the_dyninit_symbol_table_is_the_reference_cells_twenty_four_records() {
        let obj = fixture_obj();
        // Header.
        assert_eq!(u16::from_le_bytes([obj[0], obj[1]]), MACHINE_POWERPCBE);
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 8, "8 sections");
        assert_eq!(u32::from_le_bytes([obj[4], obj[5], obj[6], obj[7]]), 0, "TimeDateStamp 0");
        assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 24, "24 symbols");
        assert_eq!(u16::from_le_bytes([obj[16], obj[17]]), 0, "SizeOfOptionalHeader");
        assert_eq!(u16::from_le_bytes([obj[18], obj[19]]), CHARACTERISTICS);

        let str_name = string_comdat_name(b"abc\0").unwrap();
        // (name, Value, Sec, Type, StorageClass, nAux)
        let want: Vec<(&str, u32, i16, u16, u8, u8)> = vec![
            ("@comp.id", COMP_ID_VALUE, -1, 0, 3, 0),
            (".drectve", 0, 1, 0, 3, 1),
            (".debug$S", 0, 2, 0, 3, 1),
            (".XBLD$W", 0, 3, 0, 3, 1),
            ("__C2_11886", 0, 3, 0, 2, 0),
            (".XBLD$W", 0, 4, 0, 3, 1),
            ("__C1_11886", 0, 4, 0, 2, 0),
            (".text$yc", 0, 5, 0, 3, 1),
            ("??__EsL@@YAXXZ", 0, 5, 0x0020, 3, 0),
            ("??0L@@QAA@PBDH@Z", 0, 0, 0x0020, 2, 0),
            (".rdata", 0, 6, 0, 3, 1),
            (&str_name, 0, 6, 0x0000, 2, 0),
            (".bss", 0, 7, 0, 3, 1),
            ("sL", 0, 7, 0x0000, 3, 0),
            (".CRT$XCU", 0, 8, 0, 3, 1),
            ("sL$initializer$", 0, 8, 0x0000, 3, 0),
        ];
        let got = symbols_full(&obj);
        let got_hdr: Vec<(&str, u32, i16, u16, u8, u8)> =
            got.iter().map(|(s, _)| (s.0.as_str(), s.1, s.2, s.3, s.4, s.5)).collect();
        assert_eq!(got_hdr, want, "the 16 non-aux symbol records");

        // The aux records that carry something other than zeros:
        // (Length, nReloc, CheckSum, Number, Selection).
        let aux = |n: &str, k: usize| {
            got.iter().filter(|(s, _)| s.0 == n).nth(k).and_then(|(_, a)| *a).unwrap()
        };
        assert_eq!(aux(".drectve", 0), (132, 0, 0, 0, 0));
        assert_eq!(aux(".XBLD$W", 0), (16, 0, XBLD_C2_CHECKSUM, 0, 2));
        assert_eq!(aux(".XBLD$W", 1), (16, 0, XBLD_C1_CHECKSUM, 0, 2));
        assert_eq!(
            aux(".text$yc", 0),
            (0x18, 9, 0, 0, 2),
            ".text$yc: 9 relocations, CheckSum 0, Selection 2 ANY (not 1 \
             NODUPLICATES — that is an ORDINARY function's .text)"
        );
        assert_eq!(
            aux(".rdata", 0),
            (4, 0, 0x8619_B74C, 0, 2),
            ".rdata: a STRING literal COMDAT carries the real CRC — an \
             FP-constant one carries 0"
        );
        assert_eq!(aux(".bss", 0), (1, 0, 0, 0, 0));
        assert_eq!(aux(".CRT$XCU", 0), (4, 1, 0, 0, 0));

        // The string table: six long names, in first-use order, 100 bytes.
        let symtab = u32::from_le_bytes([obj[8], obj[9], obj[10], obj[11]]) as usize;
        let st = symtab + 24 * SYMBOL_LEN;
        let st_size = u32::from_le_bytes([obj[st], obj[st + 1], obj[st + 2], obj[st + 3]]);
        assert_eq!(st_size as usize, obj.len() - st);
        assert_eq!(
            st_size, 100,
            "the reference cell's string table is 100 bytes: __C2_11886, \
             __C1_11886, ??__EsL@@YAXXZ, ??0L@@QAA@PBDH@Z, {str_name}, \
             sL$initializer$ — `sL` and `.text$yc` are <= 8 chars and go inline"
        );

        // The total obj size is **`-Fo`-path dependent** and must not be
        // hardcoded: `.debug$S` embeds the output path in its S_OBJNAME record
        // and measured 0x94 in the probes against 0xac in the workload TUs. So
        // the pin is the path-independent remainder, and the doc's 1,316-byte
        // reference cell is then a consequence of its 148-byte `.debug$S`.
        let debug_s_len = build_debug_s(r"Z:\tmp\anat\mvp.obj").len();
        assert_eq!(
            obj.len(),
            1168 + debug_s_len,
            "everything but `.debug$S` is 1,168 bytes for this cell"
        );
        assert_eq!(1168 + 148, 1316, "…so the reference cell's 0x94 `.debug$S` gives 1,316 B");
    }

    /// The two real workload TUs, `TomCryptLicense.cpp` and `ZlibLicense.cpp`
    /// (§7.2) — the only structural difference between them is the object
    /// symbol's linkage, and the string table size is a whole-obj consequence of
    /// the COMDAT name rule that nothing here was fitted to.
    #[test]
    fn the_two_workload_tus_differ_only_in_the_objects_linkage() {
        let cell = |lit: &'static [u8], sym: &'static str, ctor: &'static str, external: bool| {
            let name = string_comdat_name(lit).unwrap();
            let thunk = DynInitThunk {
                name: "??__EsLicense@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls: vec![Call { reloc_offset: 0x14, callee: ctor }],
                data_refs: vec![
                    DataRef { hi_off: 0x00, lo_off: 0x08, name: &name, is_function: false },
                    DataRef { hi_off: 0x04, lo_off: 0x0c, name: sym, is_function: false },
                ],
            };
            let object = BssObject {
                symbol: sym,
                size: 0xc,
                natural_align: 4,
                external,
                initializer_symbol: "sLicense$initializer$",
            };
            emit_dyninit_obj(
                r"Z:\t\x.obj",
                &thunk,
                Some(&StringLiteral { bytes: lit }),
                &object,
            )
            .expect("both workload TUs are in class")
        };
        let ctor = "??0Licenses@@QAA@PBDW4Requirement@0@@Z";
        let tomcrypt = cell(b"system/src/synth/tomcrypt\0", "sLicense", ctor, false);
        let zlib = cell(b"system/src/zlib\0", "?sLicense@@3VLicenses@@A", ctor, true);

        for (tag, obj, rdata_size, class, obj_sym) in [
            ("tomcrypt", &tomcrypt, 0x1au32, 3u8, "sLicense"),
            ("zlib", &zlib, 0x10, 2, "?sLicense@@3VLicenses@@A"),
        ] {
            let secs = sections_of(obj);
            assert_eq!(secs.len(), 8, "{tag}: 8 sections");
            assert_eq!(secs[5].1, rdata_size, "{tag}: .rdata size");
            assert_eq!(secs[5].6, 0x4030_1040, "{tag}: .rdata ALIGN_4");
            assert_eq!(secs[6].1, 0xc, "{tag}: .bss size");
            assert_eq!(secs[6].6, 0xC030_0080, "{tag}: .bss ALIGN_4");
            assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 24);
            let syms = symbols_full(obj);
            // By exact name: `??__EsLicense@@YAXXZ` also *contains* the object's
            // spelling and sits earlier in the table, so a substring match here
            // grades the thunk instead and reads STATIC in both cells — a test
            // that passes for the wrong reason on the row that matters.
            let (o, _) = syms.iter().find(|(s, _)| s.0 == obj_sym).unwrap();
            assert_eq!(o.4, class, "{tag}: the object symbol's storage class");
            assert_eq!(o.2, 7, "{tag}: the object lives in .bss");
            // The thunk stays STATIC in BOTH — the object's linkage does not
            // move it (§4.3). ZlibLicense.cpp confirms both halves at once.
            let (t, _) = syms.iter().find(|(s, _)| s.0 == "??__EsLicense@@YAXXZ").unwrap();
            assert_eq!((t.3, t.4), (0x0020, 3), "{tag}: the thunk is STATIC of FUNCTION type");
            let (init, _) =
                syms.iter().find(|(s, _)| s.0 == "sLicense$initializer$").unwrap();
            assert_eq!((init.2, init.3, init.4), (8, 0, 3), "{tag}: $initializer$ is STATIC in .CRT$XCU");
        }
        // The string tables, whose sizes are a byte-level consequence of the
        // COMDAT-name rule and were transcribed from the reference objs.
        let st_size = |obj: &[u8]| {
            let symtab = u32::from_le_bytes([obj[8], obj[9], obj[10], obj[11]]) as usize;
            let st = symtab + 24 * SYMBOL_LEN;
            u32::from_le_bytes([obj[st], obj[st + 1], obj[st + 2], obj[st + 3]])
        };
        assert_eq!(
            st_size(&tomcrypt),
            161,
            "TomCrypt: 6 entries — `sLicense` is exactly 8 chars and goes INLINE"
        );
        assert_eq!(
            st_size(&zlib),
            175,
            "Zlib: 7 entries — the decorated ?sLicense@@3VLicenses@@A is interned \
             before sLicense$initializer$"
        );
    }

    /// The class boundary, stated as refusals rather than as a comment. Each of
    /// these is a shape `docs/OBJ_DYNINIT_SHAPE.md` measured to be *different*
    /// or never measured at all, and an honest `None` is the required answer.
    #[test]
    fn emit_dyninit_obj_declines_everything_outside_the_measured_class() {
        let lit = StringLiteral { bytes: b"abc\0" };
        let name = string_comdat_name(lit.bytes).unwrap();
        let ok_refs = || {
            vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: &name, is_function: false },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL", is_function: false },
            ]
        };
        let object = |size: u32, align: u32| BssObject {
            symbol: "sL",
            size,
            natural_align: align,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        let go = |t: DynInitThunk, l: Option<&StringLiteral>, o: BssObject| {
            emit_dyninit_obj(r"Z:\t\x.obj", &t, l, &o).is_some()
        };
        fn base<'a>(calls: Vec<Call<'a>>, refs: Vec<DataRef<'a>>) -> DynInitThunk<'a> {
            DynInitThunk {
                name: "??__EsL@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls,
                data_refs: refs,
            }
        }
        let one_call = || vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }];

        assert!(go(base(one_call(), ok_refs()), Some(&lit), object(1, 1)), "the reference cell is in class");
        // No call, or two: a different body — the destructor shape is framed
        // with 14 relocations and a `bl atexit` (§4.4).
        assert!(!go(base(vec![], ok_refs()), Some(&lit), object(1, 1)), "zero calls");
        assert!(
            !go(
                base(
                    vec![
                        Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" },
                        Call { reloc_offset: 0x18, callee: "atexit" },
                    ],
                    ok_refs()
                ),
                Some(&lit),
                object(1, 1)
            ),
            "two calls — that is the destructor shape"
        );
        // A quad against a symbol that is neither the literal nor the object.
        assert!(
            !go(
                base(
                    one_call(),
                    vec![
                        DataRef { hi_off: 0, lo_off: 8, name: &name, is_function: false },
                        DataRef { hi_off: 4, lo_off: 12, name: "?other@@3HA", is_function: false },
                    ]
                ),
                Some(&lit),
                object(1, 1)
            ),
            "an unrelated data symbol"
        );
        // A literal present but never referenced, or referenced twice.
        assert!(
            !go(
                base(one_call(), vec![DataRef { hi_off: 4, lo_off: 12, name: "sL", is_function: false }]),
                Some(&lit),
                object(1, 1)
            ),
            "a literal with no reference to it"
        );
        // A zero-sized object, and an alignment that is not 1/2/4/8.
        assert!(!go(base(one_call(), ok_refs()), Some(&lit), object(0, 1)), "sizeof 0");
        assert!(!go(base(one_call(), ok_refs()), Some(&lit), object(1, 3)), "align 3");
        // A `.text` that is not a whole number of instructions.
        assert!(
            emit_dyninit_obj(
                r"Z:\t\x.obj",
                &DynInitThunk {
                    name: "??__EsL@@YAXXZ",
                    text: &[0, 1, 2],
                    calls: one_call(),
                    data_refs: ok_refs(),
                },
                Some(&lit),
                &object(1, 1)
            )
            .is_none(),
            "a 3-byte .text"
        );
        // The literal-free cell IS in class (§3.2's `L(int)` row: one address
        // operand, five relocations) — and it is a 7-section, 21-symbol obj, so
        // nothing here may assume 8 and 24.
        let no_lit = emit_dyninit_obj(
            r"Z:\t\x.obj",
            &base(one_call(), vec![DataRef { hi_off: 0, lo_off: 4, name: "sL", is_function: false }]),
            None,
            &object(1, 1),
        )
        .expect("a constructor with no string argument is in class");
        assert_eq!(u16::from_le_bytes([no_lit[2], no_lit[3]]), 7, "no .rdata section");
        assert_eq!(
            u32::from_le_bytes([no_lit[12], no_lit[13], no_lit[14], no_lit[15]]),
            21,
            "24 minus the .rdata section symbol, its aux and the literal"
        );
        assert_eq!(
            relocations_of(&no_lit, ".text$yc").len(),
            5,
            "one quad plus the REL24"
        );
    }

    /// The alignment rule (§4.2), both sides, at every measured threshold.
    #[test]
    fn the_alignment_nibble_rule_matches_both_measured_columns() {
        // n = 1 -> ALIGN_1; 2..63 -> ALIGN_4; >= 64 -> ALIGN_8, then `max` with
        // the natural alignment.
        for (n, t, want) in [
            (1u32, 1u32, 1u32),
            (2, 1, 3),
            (3, 1, 3),
            (63, 1, 3),
            (64, 1, 4),
            (256, 1, 4),
            // `t` moves independently: a `double` member is ALIGN_8 at n = 8
            // where a `char[8]` is ALIGN_4.
            (8, 8, 4),
            (8, 1, 3),
            (1, 2, 2),
            (4, 8, 4),
        ] {
            assert_eq!(align_nibble(n, t), Some(want), "n={n}, t={t}");
        }
        assert_eq!(align_nibble(1, 3), None, "a non-power-of-two alignment is refused");
        // **Board #1120, lane `w-align16`.** ALIGN_16 is now measured — on a
        // nine-way structural grid whose alignments were read off c2's own obj
        // (`work/w-align16/`), and it converts byte-exact through both
        // consumers. `n = 1, t = 16` is cell `A01`'s exact shape scaled down:
        // a 4-byte `__declspec(align(16)) int`, where the object is SMALLER
        // than its own alignment.
        assert_eq!(align_nibble(1, 16), Some(5), "ALIGN_16 — A01/A02, nibble 5");
        assert_eq!(align_nibble(4, 16), Some(5), "A01 scalar: size 4, align 16");
        assert_eq!(align_nibble(64, 16), Some(5), "A04 array: size 64, align 16");
        // **The `implied` ceiling is 8 and it does NOT keep climbing.** Cell
        // `A07` is `char g[4096]` and real c2 gives it nibble 4, not 5. This is
        // the row that says everything above 8 arrives through `natural`.
        assert_eq!(align_nibble(4096, 1), Some(4), "A07 char[4096] is ALIGN_8, not 16");
        // **32 and 64 exist, are measured, and stay refused** (`A09`/`A10` get
        // nibbles 6 and 7 from c2). The grid varies structure at 16 and varies
        // nothing at 32/64, so the table stops where the cells stop.
        assert_eq!(align_nibble(1, 32), None, "A09 align(32) — measured, refused");
        assert_eq!(align_nibble(1, 64), None, "A10 align(64) — measured, refused");
    }

    /// The **negative half of the same rule**: a pooled FP constant's halves
    /// *are* adjacent (`addis` then `lfs`, four bytes apart), and that is why
    /// `hi_off + 4` looked right. Pinning it here is what stops a future
    /// "unify the two quad emitters" refactor from fixing one by breaking the
    /// other — the two quads are genuinely different and this says so portably.
    ///
    /// Packed, not `/Gy`: [`emit_comdat_obj`] carries no constant-pool code at
    /// all, because `PortC2::build` refuses a pooled constant under `/Gy`
    /// (`docs/OBJ_GY_SHAPES.md` §2, the reverse-append ordering) and hardcodes
    /// `fp_refs: Vec::new()` on that path. Writing this test against the COMDAT
    /// emitter read **0 relocation records** and would have been the vacuous
    /// shape — a control run where the effect cannot appear.
    #[test]
    fn the_pooled_fp_constant_quad_is_adjacent_which_is_why_the_data_one_looked_it() {
        let text = vec![0u8; 12];
        let f = Function {
            is_float: true,
            fp_refs: vec![crate::codegen::FpConstRef {
                hi_off: 0,
                bits: 0x3FF0_0000_0000_0000,
                double: false,
            }],
            ..Function::plain("?fc@@YAMXZ", 0)
        };
        let obj = emit_obj(r"Z:\t\fc.obj", &[f], &text, 2536);
        let recs = text_relocations(&obj);
        assert_eq!(
            recs.len(),
            4,
            "(i) a single pooled FP constant is one quad = 4 records, got {}",
            recs.len()
        );
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![4],
            "(j) the FP quad's halves ARE adjacent — REFLO belongs at hi_off+4 = \
             4 here, and the data-symbol quad's does NOT: {reflo:?}"
        );
    }
}
