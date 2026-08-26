//! **The mode-lane registry** (`scripts/lanes.txt`) — a portable lane test, no
//! toolchain needed.
//!
//! `scripts/mode_lane.sh` has always taken a mode plus arbitrary flags and has
//! always worked. Nothing *enumerated* the lanes, so the set that actually ran on
//! any given day was the set somebody remembered to type — and the four recorded
//! all through `docs/` (`/Ox`, `/O1`, `/O2`, `/Ox /Gy`) contain **no `/EH` at
//! all**, on a workload every TU of which is compiled `/EHsc`. The `/EHsc` gap
//! made the whole EH surface vacuous once: a defect over 35,964 already-in-class
//! functions survived two rungs because every standing lane compiled without
//! `/EH`, so the row collapsed onto its own control. The same measurement pass
//! found a second hole of the identical shape: **no lane had ever passed `/Oi`**,
//! though the real workload's actual profile is `/O1 /Oi /EHsc`.
//!
//! The registry closed that. This test is what makes the registry binding.
//! Until now the only thing asserting that the shipped registry still carries an
//! `/EH` lane was `scripts/gate.sh --selftest` — a shell script, run by hand,
//! guarding the property that is the entire reason the registry exists. A lane
//! deleted from `scripts/lanes.txt` in a "tidy up the gate" commit would have been
//! caught by nothing that `cargo test` runs.
//!
//! What it asserts, over `scripts/lanes.txt`:
//!
//! 1. the registry parses to a **positive** number of lanes — stated as a count
//!    that must exceed zero, never as "no errors were found". An unreadable or
//!    empty registry fails here rather than sailing through an emptied loop;
//! 2. every non-comment row becomes a lane (slug **and** at least one flag), and
//!    the lane count equals the row count — a row that fails to parse must be
//!    named, not silently dropped;
//! 3. slugs are unique — two rows under one slug means one lane's result silently
//!    overwrites the other's while the table still shows the expected row count;
//! 4. at least one lane compiles **`/EH`**, and the `/EHsc` axis is crossed over
//!    *every* base configuration, which is what the registry claims to be;
//! 5. at least one lane passes **`/Oi`**, and at least one passes **`/GR`** —
//!    and the *workload's own literal flag string* is a lane;
//! 6. `/O1` and `/O2` both appear as their own lanes — `/O2` does not exercise
//!    `OptMode::O1`, so one is not cover for the other;
//! 7. the registry has at least [`EXPECTED_LANES`] lanes. This is a **floor**:
//!    adding a lane is meant to be a one-line edit and must not break the test;
//!    deleting one is what this catches.
//!
//! ### Why an exact lane is asserted even where its verdicts are redundant
//!
//! `/O1 /EHsc` differs from `/O1` in **zero** verdict rows today. It is still in
//! the registry, and this test still requires it, because the reference obj is a
//! *different obj* — byte-compared, the `/EHsc` capture of `w27_fp_reg` is 4,662
//! bytes against 4,654. The port is reproducing genuinely different output and
//! merely arriving at the same verdict. **Verdict-identical is not redundant.**
//!
//! Contrast `/O1 /Gy`, which is *not* in the registry: it also matched `/O1` in
//! zero rows, but it was dropped because `/O1` already *implies* `/Gy`
//! (`docs/OPT_MODE.md` §3.3) — the flag is not being varied at all. The two
//! situations are indistinguishable in a verdict table and are not alike, and a
//! future reader trimming "lanes that grade nothing new" has to tell them apart.
//!
//! ### Relationship to `scripts/gate.sh --selftest`
//!
//! The gate's `shipped-registry` case is a deliberately **strictly weaker subset**
//! of this test (at least 2 lanes, at least 1 of them `/EH`), kept so that
//! `gate.sh --selftest` is self-contained on a machine with no cargo. It is a
//! smoke check, not a second definition of the rule: it cannot pass anything this
//! test rejects, so the two cannot disagree in the direction that matters. This
//! file is the binding assertion.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The lane count as of 2026-08-04: six code-shape configurations crossed with
/// the exception-handling axis (12), plus three *distinguishable* configurations
/// crossed with `/GR`, themselves crossed with `/EHsc` (6). Asserted as a
/// **floor**, not an equality — see the module doc.
///
/// It read `12` from 2026-07-31 until lane `w-gr` closed the `/GR` hole.
/// PROV[N] not load-bearing in the PROVENANCE sense — a pinned expectation over THIS PORT's own tree, asserted so a change is visible. It records the port's state, never c2's. (It is load-bearing for the test; that is a different question and the test is what guards it.) The 18 gate lanes; the same 18 `scripts/gate_identity_diff.sh` enumerates.
const EXPECTED_LANES: usize = 18;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

/// One registry row: `<slug>  <cl flags...>`.
struct Lane {
    slug: String,
    flags: Vec<String>,
}

impl Lane {
    fn flag_string(&self) -> String {
        self.flags.join(" ")
    }

    /// Does this lane compile with exception handling? `/EHsc`, `/EHa`, … all
    /// count; the registry's claim is about the `/EH` *axis*, not one spelling.
    fn has_eh(&self) -> bool {
        self.flags.iter().any(|f| f.starts_with("/EH"))
    }

    /// The lane with every `/EH…` flag removed — its partner on the other side
    /// of the exception-handling cross.
    fn without_eh(&self) -> Vec<String> {
        self.flags.iter().filter(|f| !f.starts_with("/EH")).cloned().collect()
    }
}

/// Parse `scripts/lanes.txt` by the same rule `scripts/gate.sh` uses: strip
/// `#` comments, drop blank lines, and every remaining row must yield a slug and
/// at least one flag.
///
/// Returns `(lanes, non_comment_row_count)`. The row count is returned separately
/// and on purpose: a row that fails to parse must be visible as a *discrepancy*,
/// not as a lane that quietly never existed.
fn parse_registry(text: &str) -> (Vec<Lane>, usize) {
    let mut lanes = Vec::new();
    let mut rows = 0usize;
    for line in text.lines() {
        let row = match line.split_once('#') {
            Some((before, _)) => before,
            None => line,
        };
        if row.trim().is_empty() {
            continue;
        }
        rows += 1;
        let mut fields = row.split_whitespace();
        let Some(slug) = fields.next() else { continue };
        let flags: Vec<String> = fields.map(|s| s.to_string()).collect();
        if flags.is_empty() {
            continue; // counted as a row, not produced as a lane — see caller
        }
        lanes.push(Lane { slug: slug.to_string(), flags });
    }
    (lanes, rows)
}

fn load_registry() -> Vec<Lane> {
    let path = repo_root().join("scripts/lanes.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read the lane registry at {}: {e}. The registry is the list of \
             configurations the gate runs; without it `scripts/gate.sh` grades \
             nothing.",
            path.display()
        )
    });

    let (lanes, rows) = parse_registry(&text);

    // POSITIVE first, and before anything else: the registry produced lanes, and
    // this is how many. Every assertion below is a statement about a non-empty
    // list; on an empty one they would all hold vacuously, which is the exact
    // failure mode — absence reading as success — that the registry exists to
    // close one level up.
    assert!(
        lanes.len() >= EXPECTED_LANES,
        "scripts/lanes.txt parsed {} lane(s); at least {EXPECTED_LANES} are \
         required. An empty or shortened registry is a gate that grades nothing \
         while exiting 0. Adding lanes is fine — this is a floor. If a lane was \
         deliberately retired, retire this floor in the same commit and say why.",
        lanes.len()
    );
    assert_eq!(
        lanes.len(),
        rows,
        "scripts/lanes.txt has {rows} non-comment row(s) but only {} parse as \
         lanes. A row needs a slug AND at least one flag; a row that does not \
         parse must be an error, never a lane that silently vanishes from the \
         gate.",
        lanes.len()
    );

    let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
    for lane in &lanes {
        assert!(
            seen.insert(lane.slug.as_str(), ()).is_none(),
            "duplicate lane slug `{}` in scripts/lanes.txt. Two rows under one \
             slug means one lane's result silently replaces the other's while the \
             result table still shows the expected number of rows.",
            lane.slug
        );
    }

    lanes
}

/// The property the whole registry was built for: the shipped lane list still
/// compiles `/EH`, and it compiles it over *every* base configuration.
#[test]
fn shipped_registry_crosses_the_exception_handling_axis() {
    let lanes = load_registry();

    let eh: Vec<&Lane> = lanes.iter().filter(|l| l.has_eh()).collect();
    assert!(
        !eh.is_empty(),
        "NO lane in scripts/lanes.txt compiles /EH, over {} lane(s). Every TU of \
         the dc3 workload is compiled /EHsc. This is the hole the registry was \
         written to close: a defect over 35,964 already-in-class functions \
         survived two rungs because every standing lane compiled without /EH, so \
         the row collapsed onto its own control.",
        lanes.len()
    );

    // The cross, stated in both directions. A registry could satisfy "has an /EH
    // lane" with one bolted-on row while five configurations went un-crossed, and
    // that is the shape the four historical lanes had.
    let plain: Vec<&Lane> = lanes.iter().filter(|l| !l.has_eh()).collect();
    assert!(
        !plain.is_empty(),
        "every lane compiles /EH; the axis is not being crossed, it has been moved"
    );
    for lane in &plain {
        let partner = eh.iter().find(|e| e.without_eh() == lane.flags);
        assert!(
            partner.is_some(),
            "lane `{}` ({}) has no /EH partner. The /EHsc axis is crossed over \
             ALL base configurations — that cross IS the registry. Lanes that do \
             carry a partner: {}",
            lane.slug,
            lane.flag_string(),
            eh.iter().map(|l| l.slug.as_str()).collect::<Vec<_>>().join(" ")
        );
    }
    assert_eq!(
        eh.len(),
        plain.len(),
        "the /EH cross is lopsided: {} plain lane(s) against {} /EH lane(s)",
        plain.len(),
        eh.len()
    );

    // `/O1 /EHsc` specifically. It differs from `/O1` in ZERO verdict rows and is
    // in the registry anyway, because the reference obj is a different obj (the
    // /EHsc capture of `w27_fp_reg` is 4,662 bytes against 4,654). Verdict-
    // identical is not redundant, and this assertion is where that is enforced
    // against a future pass that trims "lanes which grade nothing new".
    assert!(
        lanes.iter().any(|l| l.flags == ["/O1", "/EHsc"]),
        "no `/O1 /EHsc` lane. Its verdict rows are identical to `/O1`'s and it is \
         still not redundant: the port is reproducing a different obj and merely \
         arriving at the same verdict. Contrast `/O1 /Gy`, dropped because `/O1` \
         already implies `/Gy` — indistinguishable in a verdict table, not alike. \
         Lanes present: {}",
        lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
    );

    println!(
        "lane registry: {} lanes, {} of them compile /EH, {} plain",
        lanes.len(),
        eh.len(),
        plain.len()
    );
}

/// The flags that must be *varied* by some lane, each with the reason it is not
/// covered by another lane.
#[test]
fn shipped_registry_varies_every_flag_the_workload_depends_on() {
    let lanes = load_registry();

    // `/Oi`: until the registry existed, NO lane had ever passed it, though the
    // real workload's profile is `/O1 /Oi /EHsc`. The same "a flag no lane
    // varies" hole as `/EH`, found while measuring the registry.
    let oi: Vec<&Lane> = lanes
        .iter()
        .filter(|l| l.flags.iter().any(|f| f == "/Oi"))
        .collect();
    assert!(
        !oi.is_empty(),
        "NO lane in scripts/lanes.txt passes /Oi, over {} lane(s). The dc3 \
         workload compiles `/O1 /Oi /EHsc`; before the registry, no lane had ever \
         passed /Oi at all. Note that a `/Ox /Oi` lane does NOT close this — /Ox \
         already implies /Oi, so it varies nothing. Lanes present: {}",
        lanes.len(),
        lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
    );
    // …and it must be varied where it is not already implied. `/Ox` implies
    // `/Oi`, so only an `/O1 /Oi` lane actually exercises the flag.
    assert!(
        oi.iter().any(|l| l.flags.iter().any(|f| f == "/O1")),
        "/Oi appears only alongside /Ox, which already implies it — the flag is \
         named but not varied. The workload's profile is `/O1 /Oi /EHsc`."
    );

    // `/O1` and `/O2` are separate lanes: /O2 does not exercise `OptMode::O1`,
    // which is the mode the dc3 workload is actually built at.
    for mode in ["/O1", "/O2", "/Ox", "/Od"] {
        assert!(
            lanes.iter().any(|l| l.flags.iter().any(|f| f == mode)),
            "no lane compiles {mode}. /O2 does not exercise OptMode::O1 and /O1 \
             does not exercise /O2's; /Ox is the profile `c2rs diff` and \
             `expr_sweep.sh` hardcode; /Od is the fail-closed boundary lane. \
             Lanes present: {}",
            lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
        );
    }

    // `/Gy` — COMDAT-per-function section layout, a real axis (8 verdict rows
    // differ against `/Ox`). Only meaningful on `/Ox`, the one mode that does not
    // already imply it.
    assert!(
        lanes
            .iter()
            .any(|l| l.flags.iter().any(|f| f == "/Gy") && l.flags.iter().any(|f| f == "/Ox")),
        "no `/Ox /Gy` lane. Section layout is a real axis — it graded 8 rows \
         differently from /Ox when the registry was measured — and /Ox is the one \
         mode that does not already imply /Gy. Lanes present: {}",
        lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
    );

    // `/GR` — run-time type information, and the second instance of the exact
    // hole the registry was built to close. Every TU of the dc3 workload is
    // compiled `/GR`; for four days after the registry shipped, no lane passed
    // it. `/GR` is what mints `.rdata$r` — 24,163 sections across 676 of the
    // workload's 871 objs, and factor C's largest single step (169 -> 590).
    //
    // And it is NOT this compiler's default, which is the part worth asserting
    // rather than remembering: MSVC documents `/GR` as on by default and this
    // `cl.exe` behaves as `/GR-` without it (measured — `.rdata$r` is present
    // with `/GR` and absent both without it and with an explicit `/GR-`). So no
    // amount of new corpus *source* can reach the section; only a lane can.
    let gr: Vec<&Lane> = lanes
        .iter()
        .filter(|l| l.flags.iter().any(|f| f == "/GR"))
        .collect();
    assert!(
        !gr.is_empty(),
        "NO lane in scripts/lanes.txt passes /GR, over {} lane(s). Every TU of \
         the dc3 workload is compiled /GR and /GR is NOT this compiler's \
         default, so nothing any lane grades can contain a single `.rdata$r` \
         section — the workload has 24,163 of them across 676 objs. This is the \
         /EHsc hole one flag over, and the /EHsc one produced a live wrong emit \
         (board #263) at the workload's own flags. Lanes present: {}",
        lanes.len(),
        lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
    );
    // …and crossed, in both directions, like `/EH`. One bolted-on `/GR` row
    // beside twelve un-crossed ones is the shape the four historical lanes had.
    assert!(
        gr.iter().any(|l| l.has_eh()) && gr.iter().any(|l| !l.has_eh()),
        "the /GR lanes do not cross the /EH axis. Measured on 230 RTTI cases, \
         /EHsc moves the RTTI block's position in the section table in 43 of \
         them at every base configuration, so the cross is a real axis and not \
         symmetry for its own sake. /GR lanes: {}",
        gr.iter().map(|l| l.slug.as_str()).collect::<Vec<_>>().join(" ")
    );

    println!(
        "lane registry: {} lanes; /Oi varied by {}; /GR varied by {}",
        lanes.len(),
        oi.iter().map(|l| l.slug.as_str()).collect::<Vec<_>>().join(" "),
        gr.iter().map(|l| l.slug.as_str()).collect::<Vec<_>>().join(" ")
    );
}

/// The compile-shaping flags the dc3 workload is actually built with, taken from
/// `work/dc3-workload/flags.txt`:
///
/// ```text
/// /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I src/system/stlport /I …
/// ```
///
/// `/work` is gitignored, so the file cannot be a test dependency — but when it
/// *is* present it is compared against this constant, so the two cannot drift
/// apart silently in either direction.
/// PROV[N] not load-bearing — a MEASUREMENT PROFILE, named under [N] in DISCLOSURE: the compiler flag list this cell is captured and graded at. It selects which behaviour is observed; it is not a value read from c2. The dc3 workload's own four flags.
const WORKLOAD_PROFILE: &[&str] = &["/EHsc", "/GR", "/O1", "/Oi"];

/// The shaping flags of a `cl` command line: everything that is not `/nologo`,
/// not a `/wd####` warning suppression, not `/c`, and not an `/I <path>` pair.
/// Returned sorted, because flag order does not change what `cl` emits.
fn shaping_flags(cmdline: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip_next = false;
    for tok in cmdline.split_whitespace() {
        if skip_next {
            skip_next = false;
            continue;
        }
        match tok {
            "/I" | "-I" => {
                skip_next = true;
            }
            "/nologo" | "/c" => {}
            _ if tok.starts_with("/wd") || tok.starts_with("/I") || tok.starts_with("-I") => {}
            _ => out.push(tok.to_string()),
        }
    }
    out.sort();
    out
}

/// **The registry must contain the workload's own literal profile.**
///
/// Every hole this registry has had was the same one: a flag the *workload*
/// compiles with that no *lane* varies. `/EH` was the first (and the `/EHsc`
/// `eh-bare` slot, board #263, was a live wrong emit found at `/O1 /EHsc`);
/// `/Oi` was the second, found while measuring the registry; `/GR` was the
/// third, found four days later by reading `flags.txt` next to the lane list.
///
/// Asserting "some lane passes `/GR`" would not have caught `/Oi`, and
/// asserting "some lane passes `/Oi`" would not have caught `/GR`. Asserting
/// that the *whole* profile appears as one lane catches the next one too,
/// whatever it turns out to be — which is the only version of this test that is
/// not a list of yesterday's mistakes.
#[test]
fn the_workloads_own_profile_is_a_lane() {
    let lanes = load_registry();

    // When the workload manifest is on disk, it is the source of truth and the
    // constant is checked against it. `/work` is gitignored, so this half is
    // skipped on a clean checkout — and it says so with a count, never silently.
    let flags_path = repo_root().join("work/dc3-workload/flags.txt");
    match std::fs::read_to_string(&flags_path) {
        Ok(text) => {
            let measured = shaping_flags(&text);
            assert_eq!(
                measured,
                WORKLOAD_PROFILE,
                "the dc3 workload's shaping flags in {} are {:?}, but this test's \
                 WORKLOAD_PROFILE constant says {:?}. The workload moved and the \
                 registry does not know: update the constant AND add the lane in \
                 the same commit.",
                flags_path.display(),
                measured,
                WORKLOAD_PROFILE
            );
            println!(
                "workload profile: {:?} (read from {}, {} lanes in the registry)",
                measured,
                flags_path.display(),
                lanes.len()
            );
        }
        Err(_) => println!(
            "workload profile: {:?} (constant; work/dc3-workload/flags.txt absent \
             — gitignored, so this is expected on a clean checkout, and the \
             assertion below still runs against {} lanes)",
            WORKLOAD_PROFILE,
            lanes.len()
        ),
    }

    let matching: Vec<&Lane> = lanes
        .iter()
        .filter(|l| {
            let mut f: Vec<String> = l.flags.clone();
            f.sort();
            f == WORKLOAD_PROFILE
        })
        .collect();
    assert!(
        !matching.is_empty(),
        "NO lane in scripts/lanes.txt compiles the dc3 workload's own profile \
         {:?}, over {} lane(s). Every lane is then grading a configuration the \
         workload is not built at, and the one it IS built at is graded by \
         nothing. Both live wrong-emit families found at a workload flag in this \
         project's history — #263 at /O1 /EHsc, and the /GR gap this assertion \
         was added for — were exactly this. Lanes present: {}",
        WORKLOAD_PROFILE,
        lanes.len(),
        lanes.iter().map(|l| l.flag_string()).collect::<Vec<_>>().join(" | ")
    );

    println!(
        "the workload's own profile is graded by: {}",
        matching.iter().map(|l| l.slug.as_str()).collect::<Vec<_>>().join(" ")
    );
}

/// The `shaping_flags` parser, on the row shapes it has to survive. It is the
/// one thing between `flags.txt` and the assertion above, and a parser that
/// silently dropped `/GR` would make that assertion pass on a registry with no
/// `/GR` lane — absence reading as success, one level in.
#[test]
fn shaping_flags_drops_only_what_does_not_shape_the_obj() {
    // The real line, verbatim.
    assert_eq!(
        shaping_flags(
            "/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I src/system/stlport \
             /I src/xdk/LIBCMT /I src"
        ),
        ["/EHsc", "/GR", "/O1", "/Oi"]
    );
    // `/I` with the path attached, and the `-` spelling.
    assert_eq!(shaping_flags("/Ox /Isrc /I foo -Ibar -I baz"), ["/Ox"]);
    // A flag that shapes the obj must survive even if nobody thought of it.
    assert_eq!(shaping_flags("/O2 /Gy /GS- /Zi"), ["/GS-", "/Gy", "/O2", "/Zi"]);
    // Order does not matter; the result is sorted.
    assert_eq!(shaping_flags("/Oi /GR /O1"), shaping_flags("/O1 /GR /Oi"));
    // …and an empty command line yields nothing, which is why the caller
    // compares it against a non-empty constant rather than merely checking it.
    assert!(shaping_flags("").is_empty());
}

/// **The cross-product lane must take its modes from the registry too.**
///
/// `scripts/cross_sweep.sh` is the lane whose whole purpose is finding mis-emits
/// the hand-written corpus cannot — its record is real: mis-emit #12 was found in
/// the cross product of two individually-green branches. Until 2026-08-01 it
/// carried **its own hardcoded four modes** (packed, `/Gy`, `/O1`, `/O2`) and
/// compiled **no `/EH`, ever**, on a workload that compiles `/EHsc` on every TU
/// and has 35,964 in-class `eh-bare` functions whose markers appear only under it.
/// That was the last surviving instance of the un-enumerated-lane defect, in the
/// worst possible place, and its green read exactly like a green that had verified
/// those flags.
///
/// Converting it to `scripts/lanes.txt` is what makes it inherit every assertion
/// above for free. This test is what keeps it converted: nothing else in
/// `cargo test` would notice a "tidy up the sweep driver" commit that pasted a
/// mode tuple back into the file, and the symptom of that regression is silence —
/// the lane keeps passing, at four modes, having said nothing about `/EH`.
///
/// The two assertions are independent on purpose (a file can read the registry
/// *and* keep a private list, and a file can drop the private list without ever
/// reading the registry), and neither is guarded by a quantity that could make the
/// other unreachable.
#[test]
fn cross_product_lane_takes_its_modes_from_the_registry() {
    let path = repo_root().join("scripts/cross_sweep.py");
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read the cross-product driver at {}: {e}. It is the lane that \
             grades shape families beside each other; if it is gone, so is the only \
             instrument that finds cross-shape mis-emits.",
            path.display()
        )
    });

    assert!(
        src.contains("lanes.txt"),
        "scripts/cross_sweep.py never mentions scripts/lanes.txt, so it is not \
         taking its mode lanes from the registry. It then grades at whatever set \
         somebody typed into it — which for its whole existence was four modes \
         containing no /EH at all, on a 100%-/EHsc workload."
    );

    // A private mode table is the actual regression, and it is checked separately
    // from "reads the registry" because a file can do both.
    for forbidden in ["MODES = (", "MODES = [", "MODES=(", "MODES=["] {
        assert!(
            !src.contains(forbidden),
            "scripts/cross_sweep.py defines its own mode table (`{forbidden}`). The \
             lane list is data in scripts/lanes.txt and a second copy in the sweep \
             driver is the same one-rule-two-implementations shape docs/GAPS.md §6 \
             keeps recording — with the extra property that this copy is the one \
             that historically had no /EH lane."
        );
    }

    println!(
        "scripts/cross_sweep.py: reads scripts/lanes.txt, defines no private mode table"
    );
}

/// The parser this test uses is the one thing between the file on disk and every
/// assertion above, so it is exercised directly on rows shaped like the failures
/// it has to catch. A table test proves completeness only over the list it was
/// written from — these are the row shapes, not a claim that no other exists.
#[test]
fn registry_parser_counts_rows_it_cannot_turn_into_lanes() {
    let (lanes, rows) = parse_registry("A /O1\nB /O1 /EHsc\n");
    assert_eq!((lanes.len(), rows), (2, 2));
    assert_eq!(lanes[1].flags, ["/O1", "/EHsc"]);

    // Comments and blanks are not rows.
    let (lanes, rows) = parse_registry("# header\n\nA /O1  # trailing\n\n");
    assert_eq!((lanes.len(), rows), (1, 1));
    assert_eq!(lanes[0].flags, ["/O1"]);

    // A slug with no flags is a ROW that is not a LANE — the discrepancy the
    // count comparison in `load_registry` reports. It must not silently vanish.
    let (lanes, rows) = parse_registry("A /O1\nBroken\nC /Ox\n");
    assert_eq!(rows, 3, "a slug-only row is still a row");
    assert_eq!(lanes.len(), 2, "…and it is not a lane");

    // An empty registry parses to zero lanes — and `load_registry` is what turns
    // that into a failure rather than a gate over nothing.
    assert_eq!(parse_registry("").0.len(), 0);
    assert_eq!(parse_registry("# only comments\n").0.len(), 0);

    // `/EHa` counts on the /EH axis; `/EHsc` is not the only spelling.
    let (lanes, _) = parse_registry("A /O1 /EHa\n");
    assert!(lanes[0].has_eh());
    assert_eq!(lanes[0].without_eh(), ["/O1"]);
}
