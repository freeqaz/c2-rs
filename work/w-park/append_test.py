#!/usr/bin/env python3
"""Append W-PARK's differential test to the fixture-gate test file."""
TEST = '''
/// **W-PARK's pair, graded by the oracle at the default `/Ox` profile.**
///
/// Board **#1920**. Unlike every mode-fenced class beside it, this one is
/// **NOT** mode-split: `wpark_lit_permuted.cpp` is 5/5 in class and a
/// byte-exact `match` at `/O1` **and** at `/Ox`, over two objs that are not the
/// same obj — five COMDAT `.text` sections sharing one epilogue at `/O1`, one
/// packed 444-byte `.text` with the epilogue duplicated per arm at `/Ox`. The
/// port models both sides.
///
/// So the assertion here is `Match` rather than `NotImplemented`, and that
/// asymmetry with `differential_wjson_utf8_copy_refuses_outside_its_mode` is
/// the point: the first draft of the fixture header asserted the `/Ox` refusal
/// from a neighbouring module's doc, and the measurement said otherwise. A mode
/// split *in* a class is not a licence to infer a refusal *of* it.
///
/// `wpark_lit_permuted_neg.cpp` refuses in the READER at every mode. Its
/// **seven** cells decline on **seven distinct clauses**, checked per cell with
/// a reverted probe patch (`work/w-park/decline_probe.md`) — three of them were
/// rewritten after the probe read a key other than the one they were written
/// for, which is how this lane found that the permutation fence runs *before*
/// the `callseq-multiarg-lit-*` fence.
#[test]
fn differential_wpark_lit_permuted_pair() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    for (name, want_match) in
        [("wpark_lit_permuted.cpp", true), ("wpark_lit_permuted_neg.cpp", false)]
    {
        let w = work("wpark");
        let port = PortC2::default();
        let report = differential(&fixture(name), &tc, &port, &w);
        match report {
            DiffReport::ReferenceReplayByteExact { port, .. } => match (&port, want_match) {
                (PortStatus::Match, true) => {}
                (PortStatus::NotImplemented(_), false) => {}
                (other, _) => panic!("unexpected port status for {name} at /Ox: {other:?}"),
            },
            other => panic!("expected ReferenceReplayByteExact for {name}, got {other:?}"),
        }
        std::fs::remove_dir_all(&w).ok();
    }
}
'''
p = "crates/c2-harness/tests/differential.rs"
s = open(p).read()
assert "differential_wpark_lit_permuted_pair" not in s
open(p, "w").write(s.rstrip("\n") + "\n" + TEST)
print("appended")
