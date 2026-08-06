//! **INSTRUMENT 2 — the production `.in` reader's own cursor**, over a directory
//! of captured IL bundles.
//!
//! `work/w-tag02/scan.py` is instrument 1: a forward record parser written from
//! the grammar, in another language, importing nothing from this crate. This is
//! instrument 2: the same streams read by the code that ships. The `w-divsplit`
//! discipline is that a decoded grammar is believed only when two independent
//! instruments agree, and **neither may be the other's witness** — so this file
//! deliberately reports raw counts and asserts nothing about them. The
//! comparison is done outside, by `work/w-tag02/two_instruments.py`.
//!
//! Set `C2RS_IN_PROBE` to a directory of `<cell>/*.in` bundles and run:
//!
//! ```sh
//! C2RS_IN_PROBE=work/w-tag02/il cargo test -p c2-il --test in_init_probe -- --nocapture
//! ```
//!
//! Without the variable it prints `SKIP` and passes, so the portable lane and
//! `gate.sh` are unaffected — the same degrade-cleanly rule the CLI follows.

use std::path::Path;

/// Load one captured bundle directory (a `_CL_*` quintet) into an
/// [`c2_il::IlBundle`].
fn bundle_of(dir: &Path) -> Option<c2_il::IlBundle> {
    let mut bundle = c2_il::IlBundle::new("probe");
    let mut any = false;
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        let Some(ext) = p.extension().and_then(|s| s.to_str()) else { continue };
        if c2_il::IL_SUFFIXES.contains(&ext) {
            bundle.set(ext, std::fs::read(&p).ok()?);
            any = true;
        }
    }
    any.then_some(bundle)
}

#[test]
fn in_init_probe() {
    let Some(root) = std::env::var_os("C2RS_IN_PROBE") else {
        println!("SKIP: set C2RS_IN_PROBE to a directory of captured IL bundles");
        return;
    };
    let root = Path::new(&root);
    let mut cells: Vec<std::path::PathBuf> = match std::fs::read_dir(root) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_dir()).collect(),
        Err(e) => {
            println!("SKIP: {}: {e}", root.display());
            return;
        }
    };
    cells.sort();
    for dir in cells {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let Some(b) = bundle_of(&dir) else {
            println!("{name}\tNO-BUNDLE");
            continue;
        };
        let Some(r) = b.in_init_report() else {
            println!("{name}\tNO-IN");
            continue;
        };
        let residue: Vec<String> =
            r.residue_by_reason.iter().map(|(k, n)| format!("{k}={n}")).collect();
        // `unanchored`, `failclosed` and `notoken` are board **#961** — the
        // denominator `records` is silent about. They are printed here beside it
        // and never added into it, so a reconciliation can grade the anchor
        // scan's reach against a sequential parse of the same stream.
        println!(
            "{name}\trecords={} elements={} values={} conflicts={} residue={} symrefs={} \
             records_with_symrefs={} unanchored={} failclosed={} notoken={} [{}]",
            r.records,
            r.elements,
            r.values,
            r.conflicts,
            r.residue,
            r.sym_refs,
            r.records_with_sym_refs,
            r.unanchored,
            r.fail_closed,
            r.no_token,
            residue.join(" "),
        );
        // The object-level view: what `data_tu` makes of the same bundle. A
        // reader widening that moves the report and not this line has changed
        // nothing a consumer can use.
        match b.data_tu() {
            Some(t) => println!(
                "{name}\tdata_tu=ACCEPT objects={} in_census={:?}",
                t.objects.len(),
                t.in_census
            ),
            None => println!("{name}\tdata_tu=REFUSE"),
        }
    }
}
