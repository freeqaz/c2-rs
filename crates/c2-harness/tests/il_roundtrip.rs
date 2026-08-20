//! K1 codec round-trip gate over the **full captured-fixture spread**.
//!
//! Toolchain-guarded like `differential.rs`: skips cleanly (never fails) when
//! `Toolchain::locate()` is `None`. For every `fixtures/cpp/*.cpp`, capture the
//! real IL bundle through the harness and assert
//! `IlModel::encode(IlModel::parse(bundle)) == bundle` byte-for-byte — the K1
//! invariant, exercised against live-toolchain output rather than committed
//! bytes (the captured `.gl` embeds the host source path, so the bundles are not
//! committable; the always-on portable round-trip lane lives in
//! `c2-il::codec`'s unit tests over hand-built bundles).
//!
//! The spread covers every shape the codec must handle: single-function
//! arithmetic (`mvp_add3`), multi-function TUs (`mvp_two`, `mvp_lit`, `mvp_sub`),
//! wide literals (`mvp_wide`), the void tail call (`mvp_call`), the framed call
//! (`mvp_framed`), and the nine out-of-class call shapes — none of which the
//! codec needs to *decode*, only losslessly re-encode.

use std::path::PathBuf;

use c2_harness::all_fixtures;
use c2_il::{ExToken, IlModel};
use c2_reference::Toolchain;

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::unique_scratch_dir("ilrt", tag)
}

#[test]
fn roundtrip_all_fixtures_byte_identical() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let mut checked = 0usize;
    // **W-VEC (#2507)** — the fixtures whose `.gl` frames fewer body starts than
    // `.ex` has bodies, named by the gate rather than by this file. Printed, not
    // asserted against a count: a pinned count is the skip list this exemption
    // exists to avoid being.
    let mut bind_count_exempt: Vec<String> = Vec::new();
    for cpp in &fixtures {
        let name = cpp
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let w = work("cap");
        let bundle = match tc.capture_il(cpp, &w) {
            Ok(b) => b,
            Err(e) => {
                std::fs::remove_dir_all(&w).ok();
                panic!("capture_il failed for {name}: {e}");
            }
        };

        // Parse must succeed (fail-closed: parse itself verifies re-encode).
        let model = IlModel::parse(&bundle)
            .unwrap_or_else(|e| panic!("codec parse refused {name}: {e}"));

        // And the encoded bundle is byte-identical, file for file.
        let back = model.encode();
        assert_eq!(
            back.base_name, bundle.base_name,
            "base name changed for {name}"
        );
        assert_eq!(
            back.files.keys().collect::<Vec<_>>(),
            bundle.files.keys().collect::<Vec<_>>(),
            "file set changed for {name}"
        );
        for (suffix, orig) in &bundle.files {
            let got = back.get(suffix).unwrap_or(&[]);
            assert!(
                got == orig.as_slice(),
                "{name}: .{suffix} not byte-identical (orig {} B, re-encoded {} B)",
                orig.len(),
                got.len(),
            );
        }

        // Sanity: the .gl carried EXACTLY one structurally-identified body-start
        // offset per function — the `== function_count` invariant K3 relies on
        // (K2a strengthened this from the earlier `>= 1`, now that offsets are
        // located by record framing and gated 1:1/in-order against the `.ex`
        // `4F 1F` markers).
        //
        // **This read *"This 1:1 claim holds for every fixture"* until lane
        // `w-vec` (board #2507), and the claim was universal only because the
        // corpus contained no counterexample.** It is FALSE on **811 of the
        // 878 workload TUs** — the single most common shape there — and the
        // fixture spread had zero instances of it. `docs/STATUS.md` trap 5,
        // absence reading as success, in a standing gate.
        //
        // Two claims replace the one, and together they are STRICTLY MORE than
        // it asserted:
        //
        //  * **the DIRECTION is universal and stays a hard failure.** `.gl` may
        //    frame fewer body starts than `.ex` has bodies — that is the
        //    workload's normal state, c2 emitting a fraction of what c1xx hands
        //    it. A *surplus* is a codec defect and has no legitimate cause;
        //  * **the EQUALITY holds unless the gate itself says the record count
        //    differs.** The exemption is not a name list and cannot be turned
        //    into one: it is read from `IlBundle::decode_causes().first`, the
        //    same reader, and it is `BIND_COUNT` — literally *"the records
        //    bound, but their count is not the `.ex` segment count"*. A fixture
        //    can only be exempt by making the gate say so, and when it does the
        //    inequality must be STRICT, so an exempt cell cannot quietly become
        //    an equal one and keep its exemption.
        let noff = model.gl_body_start_offsets().len();
        let nfns = model.ex_function_count();
        assert!(
            noff <= nfns,
            "{name}: typed .gl offsets ({noff}) EXCEED .ex functions ({nfns}) — \
             a surplus is a codec defect, not a workload shape"
        );
        if bundle.decode_causes().first == Some(c2_il::func::cause::BIND_COUNT) {
            assert!(
                noff < nfns,
                "{name}: the gate reports `{}` and yet the offsets are 1:1 \
                 ({noff} == {nfns}) — the exemption is claimed by a cell that \
                 does not need it",
                c2_il::func::cause::BIND_COUNT
            );
            bind_count_exempt.push(name.clone());
        } else {
            assert_eq!(
                noff, nfns,
                "{name}: typed .gl offsets ({noff}) must be 1:1 with .ex functions ({nfns})"
            );
        }

        // The body-token claim is conditional on the TU actually having bodies.
        // `mvp_empty.cpp` (R1) defines no functions: the front end still emits a
        // full five-file bundle, but its `.ex` carries no function bodies at
        // all, so "decoded body tokens" is legitimately zero. The round-trip
        // itself is still gated byte-for-byte above — this only scopes the
        // structural expectation, it does not relax the codec's invariant.
        if c2_il::is_empty_module(bundle.ex().unwrap_or(&[])) {
            assert_eq!(nfns, 0, "{name}: empty module must decode to 0 functions");
            assert!(
                model.ex_tokens().is_empty(),
                "{name}: empty module must decode to 0 body tokens"
            );
        } else if has_lo_anchored_body(bundle.ex().unwrap_or(&[])) {
            let toks = model.ex_tokens();
            assert!(!toks.is_empty(), "{name}: expected decoded .ex tokens");
            assert!(nfns >= 1, "{name}: expected >=1 function, got {nfns}");
            // The composed form, decoded as the two tokens it is: the one-byte
            // `4C` body start and the optional `4F 11` record beside it. A file
            // that carries `4C 4F 11` must decode BOTH, or the split is not
            // sitting where the marker is.
            assert!(
                toks.contains(&ExToken::Lo) && toks.contains(&ExToken::LoRecord),
                "{name}: a `4C 4F 11` body must decode to Lo + LoRecord"
            );
        } else {
            // **The #158 class, and the claim is now positive.** A dynamic
            // initializer (`il_dyninit_static.cpp`) is not an empty module — it
            // has a `4F 1F` function start, and a `.gl` record binds a name to
            // it — and its body opens `4C 53` where a source function opens
            // `4C 4F 11`.
            //
            // This branch used to assert the *symptom*: "decodes to 0 tokens",
            // because the model treated `4C 4F 11` as one atom and
            // `try_ex_token` returned `None` for `4C 53`. `4C` is the token and
            // `4F 11` is a separable optional record (ROADMAP §10.12), so the
            // symptom is gone and the assertion is inverted into the rule that
            // replaced it: such a body decodes, it carries the one-byte `Lo`,
            // and it carries **no** `LoRecord` — which is precisely what
            // distinguishes this class from every other measured function.
            //
            // (The precondition is `has_lo_anchored_body`, not
            // `is_empty_module`: the latter asks about `4F 1F` as well and was
            // once used to predict the behaviour of an `LO`-anchored split —
            // ROADMAP §10.11's defect, a count taken as evidence about a
            // predicate that did not produce it.)
            let toks = model.ex_tokens();
            assert!(
                !toks.is_empty(),
                "{name}: a body with no `4C 4F 11` must still decode — the bare \
                 `4C` body start is a token, not an absence"
            );
            assert!(
                toks.contains(&ExToken::Lo),
                "{name}: expected the one-byte `4C` body-start token"
            );
            assert!(
                !toks.contains(&ExToken::LoRecord),
                "{name}: this file carries no `4C 4F 11`, so no `4F 11` record \
                 may decode after a body start"
            );
            assert!(
                nfns >= 1,
                "{name}: has no LO-anchored body and is not an empty module, so it \
                 must still carry a `4F 1F` function start; got {nfns}"
            );
        }

        checked += 1;
        std::fs::remove_dir_all(&w).ok();
    }

    assert!(checked >= 15, "expected the full fixture spread, ran {checked}");
    eprintln!("K1 round-trip: {checked} fixture bundles byte-identical");
    eprintln!(
        "K2a 1:1 offsets: {} of {checked} exempt by the gate's own `{}` \
         (the workload's majority shape){}",
        bind_count_exempt.len(),
        c2_il::func::cause::BIND_COUNT,
        if bind_count_exempt.is_empty() {
            String::new()
        } else {
            format!(" — {}", bind_count_exempt.join(", "))
        }
    );
}

/// True iff `.ex` carries at least one `4C 4F 11` — the marker the codec's
/// model and `c2_il`'s census splitter both anchor bodies on.
///
/// Deliberately **not** `is_empty_module`, which asks about `4F 1F` as well and
/// therefore answers a different question (ROADMAP §10.11). Duplicated as three
/// bytes here rather than exported: this test asserts a fact about the split,
/// and reading the constant from the crate under test would let a change to it
/// move the assertion silently.
fn has_lo_anchored_body(ex: &[u8]) -> bool {
    ex.windows(3).any(|w| w == [0x4C, 0x4F, 0x11])
}
