//! **The standing SITE COUNT for the `callee-unresolved` key family** — lane
//! `w-calleeguard`.
//!
//! `crates/c2-harness/src/gap/tests.rs`' `callee_unresolved_arms` module lands
//! one witness per raise site of this family, driven through the public
//! `IlBundle::census_functions()` and asserting on the published key string.
//! **A witness table covers the sites that existed when it was written, and
//! nothing makes a new arm add a row.** That is the standing form of
//! `w-mutcensus` F4 — *"nothing re-runs this census, so X/N goes stale on the
//! next landed fence, and one already landed during the campaign"* — scoped to
//! one family and made cheap.
//!
//! This test is the missing half. It reads `c2-il`'s own source (the crate this
//! lane does not own, and does not edit) and asserts the **shape of the
//! dispatch**, so that:
//!
//! * a **fifth** `callee-unresolved-*` arm cannot land without this failing and
//!   naming the witness table that must grow a row;
//! * a raise site cannot be **deleted** and leave a witness asserting a key
//!   nothing raises;
//! * a constant cannot be raised at **two** sites without the count saying so —
//!   which is the precondition that makes a per-key witness equal a per-site
//!   witness for this family (`w-mutcensus` F2's mechanism does **not** apply
//!   here because k = 1 four times over, and this test is what keeps that true).
//!
//! It is a **count, not a status** (`docs/STATUS.md` trap 5's mitigation), and
//! every assertion names what moved rather than reporting that something did.
//!
//! Needs no toolchain and never skips: it reads a tracked source file.

use std::path::PathBuf;

/// The file the arms live in. Named as a path rather than a line number
/// deliberately — `w-mutcensus`' own table went stale on two peer merges in one
/// wave, and a line number is the first thing to rot.
/// PROV[N] not load-bearing — a path into THIS repo's own source, so a site census can be taken over it. Nothing derived from c2.
const CENSUS_RS: &str = "crates/c2-il/src/func/census.rs";
/// Where the four key strings are declared.
/// PROV[N] not load-bearing — the second such path into this repo's own source.
const BODY_MOD_RS: &str = "crates/c2-il/src/func/body/mod.rs";

/// The arms of the `match label` that routes a parsed-but-unbuildable body to a
/// blocking key, **in source order**, each named by the pattern text this test
/// requires to be present exactly once.
/// PROV[N] not load-bearing — source-text patterns this test greps for in this repo's own `crates/`. A structural assertion about the port, not a value from c2.
const ARM_PATTERNS: [&str; 7] = [
    "\"store-run-call\" =>",
    "\"static-scan-loop\" =>",
    "\"store-run-bind\" =>",
    "\"framed-call\" =>",
    "l if l.starts_with(\"call-sequence\") =>",
    "l if l.starts_with(\"empty-dtor\") =>",
    "_ =>",
];

/// The family: the constant raised, and the key string it must be declared as.
/// **The string is the published thing** — `scan.rs` concatenates
/// `FnVerdict::key()` into `emit-cflow-modeled-key|{}` — so a rename of the
/// constant alone is invisible downstream and a change to the *string* is not.
/// PROV[N] not load-bearing — this port's own census key family, paired with the sites that raise it. Same class as `func::diag::cause`.
const FAMILY: [(&str, &str); 4] = [
    ("CALLEE_UNRESOLVED_FRAMED", "callee-unresolved-framed-call"),
    ("CALLEE_UNRESOLVED_SEQ", "callee-unresolved-call-sequence"),
    ("CALLEE_UNRESOLVED_DTOR", "callee-unresolved-dtor-delegation"),
    ("CALLEE_UNRESOLVED_TAIL", "callee-unresolved-tail-call"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("reading {}: {e}", p.display()))
}

/// The body of the `match label { … }` that routes the family, by brace
/// matching from the `match` keyword — not by line number, and not by a
/// hand-copied excerpt that would go stale silently.
fn match_label_block(src: &str) -> &str {
    let head = "match label {";
    let at = src
        .find(head)
        .expect("`match label {` — the dispatch this family's four raise sites are arms of. \
                 If it was renamed or restructured, the witness table in \
                 `gap::tests::wr1_census_key_guards::callee_unresolved_arms` must be re-derived \
                 against the new shape before this test is adjusted");
    let open = at + head.len() - 1;
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open + 1..i];
                }
            }
            _ => {}
        }
    }
    panic!("`match label {{` is unbalanced in {CENSUS_RS}");
}

#[test]
fn the_callee_unresolved_family_still_has_exactly_four_raise_sites() {
    let src = read(CENSUS_RS);
    let block = match_label_block(&src);

    // (1) Each constant is raised EXACTLY ONCE. This is the property that makes
    //     a per-key witness a per-site witness for this family; `w-mutcensus`
    //     F2's `k - 1` mechanism bites the moment it stops holding.
    for (constant, key) in FAMILY {
        let n = block.matches(constant).count();
        assert_eq!(
            n, 1,
            "`{constant}` (the key `{key}`) must be raised at EXACTLY ONE site in \
             `{CENSUS_RS}`'s `match label`; it is raised at {n}. At 0 the witness \
             in `callee_unresolved_arms` asserts a key nothing raises. At 2 or \
             more the family has acquired `w-mutcensus` F2's shape — a per-key \
             witness now pins ONE of the sites and the siblings are unguarded by \
             construction — and the witness table must grow one row per site"
        );
    }

    // (2) The family's total site count, asserted as a number.
    let total = block.matches("CALLEE_UNRESOLVED_").count();
    assert_eq!(
        total, 4,
        "the `callee-unresolved` family must have exactly 4 raise sites in this \
         dispatch; found {total}. `w-mutcensus` measured all four GREEN — nothing \
         in the suite could fail on any of them — and a FIFTH landing without a \
         witness would be unguarded on arrival, which is exactly the staleness \
         F4 records (its own frame went stale on two peer merges inside one \
         lane's wall clock)"
    );

    // (3) The arm SET, named. A new arm anywhere in this dispatch changes what
    //     the default arm catches, so the default-arm witness has to be
    //     re-derived even when the family's own four are untouched.
    for pat in ARM_PATTERNS {
        let n = block.matches(pat).count();
        assert_eq!(
            n, 1,
            "the arm `{pat}` must appear exactly once in `match label`; found {n}. \
             The dispatch's arm set is part of what the DEFAULT arm means: \
             `callee-unresolved-tail-call` is whatever no earlier arm claimed, so \
             adding, removing or reordering an arm can move bodies out of board \
             #3209's 1,296 without touching one line of the family"
        );
    }
    let arms = block.matches("=>").count();
    assert_eq!(
        arms, ARM_PATTERNS.len(),
        "`match label` must have exactly {} arms; it has {arms}. This is the \
         standing count `w-mutcensus` F4 asks for, scoped to one dispatch: it \
         fails when a fence lands without the witness table being re-scored, \
         which is the only thing that turns X/N from a fact about a commit into \
         a maintained invariant",
        ARM_PATTERNS.len()
    );

    // (4) …and the constants really do carry the key STRINGS the witnesses
    //     assert. Without this the two files could drift apart: the witness
    //     would pin a string, this test would count a constant, and a rename of
    //     the constant's VALUE would move the published key past both.
    let decls = read(BODY_MOD_RS);
    for (constant, key) in FAMILY {
        let want = format!("const {constant}: &str = \"{key}\";");
        assert!(
            decls.contains(&want),
            "`{BODY_MOD_RS}` must declare `{want}` — the witness table asserts \
             the STRING `{key}` and this test counts the CONSTANT `{constant}`, \
             so the binding between them is what makes the two halves one guard. \
             `scan.rs` publishes the string, not the constant"
        );
    }
}
