//! **The rung registry** (`docs/ARCHITECTURE_SEAMS.md` §3.4, §5.1) — a portable
//! lane test, no toolchain needed.
//!
//! On 2026-07-30 nine rungs landed in parallel worktrees and the collisions that
//! git could *not* flag were all name allocations: the tag `W23` claimed twice
//! by two agents each picking "the next free number", and the ROADMAP section
//! letters §6e/§6f/§6g/§6i each claimed twice the same way. Small integers and
//! section letters allocated concurrently collide silently. Filenames collide as
//! add/add conflicts, loudly — so a rung's identity became its **slug**, which is
//! its `docs/rungs/` filename, and this test is what makes that claim binding.
//!
//! What it asserts, over `docs/rungs/*.md`:
//!
//! 1. every rung doc carries the machine-read header block (`Tag:`, `Slug:`,
//!    `Fixtures:`) that `_TEMPLATE.md` defines;
//! 2. **no two rung docs declare the same `Tag:`** — the W23 collision;
//! 3. the declared `Slug:` equals the filename's slug — a doc copy-pasted from
//!    a sibling and half-edited is the realistic way a slug claim goes wrong;
//! 4. every fixture a rung names **exists** in `fixtures/cpp` (§5.1: per-rung
//!    fixtures are the merge safety net that makes one-line dispatch merges
//!    safe, so a rung doc naming a fixture that was never landed is a hole);
//! 5. every rung names **at least one** fixture;
//! 6. no fixture prefix (`w25`, `wunw`, …) is claimed by two rung docs;
//! 7. `INDEX.md` equals what `scripts/gen_rung_index.sh` generates — the index
//!    is generated, and this is what keeps it from drifting. The test *runs the
//!    script* rather than reimplementing it, so the generation rule keeps
//!    exactly one locator.
//!
//! What it deliberately does **not** assert: that every `wNN` prefix in
//! `fixtures/cpp` is claimed by some rung doc. The historical rungs' write-ups
//! live in `docs/ROADMAP.md` §6a–§6m and are staying there (§9.6 — freeze and
//! fork, don't migrate); backfilling the rest is §6 step 4. Claiming full
//! coverage before that is done would be a green light that means nothing, so
//! the unclaimed prefixes are printed instead.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

/// A rung doc's machine-read header: the indented `Key: value` block at the top.
struct Rung {
    file: String,
    slug_from_name: String,
    fields: BTreeMap<String, String>,
}

impl Rung {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|s| s.as_str())
    }
}

/// `2026-07-30-store-leaf.md` → `store-leaf`. Returns `None` if the name is not
/// in `YYYY-MM-DD-<slug>.md` form, which is itself a failure.
fn slug_of(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".md")?;
    let mut parts = stem.splitn(4, '-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    let slug = parts.next()?;
    let dated = y.len() == 4
        && m.len() == 2
        && d.len() == 2
        && y.bytes().chain(m.bytes()).chain(d.bytes()).all(|b| b.is_ascii_digit());
    if !dated || slug.is_empty() {
        return None;
    }
    Some(slug.to_string())
}

/// `w25_store_leaf.cpp` → `w25`. The prefix is the rung's fixture claim.
fn fixture_prefix(name: &str) -> String {
    name.split('_').next().unwrap_or(name).to_string()
}

fn load_rungs(dir: &Path) -> Vec<Rung> {
    let mut out = Vec::new();
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md"))
        .filter(|n| !n.starts_with('_') && n != "INDEX.md" && n != "README.md")
        .collect();
    names.sort();
    for name in names {
        let text = std::fs::read_to_string(dir.join(&name)).expect("rung doc readable");
        let mut fields = BTreeMap::new();
        for line in text.lines() {
            // The header block is indented four spaces; stop at the first
            // non-indented, non-blank line after it has started.
            let Some(rest) = line.strip_prefix("    ") else {
                if !fields.is_empty() && !line.trim().is_empty() {
                    break;
                }
                continue;
            };
            let Some((k, v)) = rest.split_once(':') else { continue };
            if k.is_empty() || k.contains(char::is_whitespace) {
                continue;
            }
            fields.insert(k.trim().to_string(), v.trim().to_string());
        }
        let slug_from_name = slug_of(&name).unwrap_or_else(|| {
            panic!(
                "rung doc {name} is not named YYYY-MM-DD-<slug>.md — the filename \
                 IS the slug claim (docs/rungs/README.md)"
            )
        });
        out.push(Rung { file: name, slug_from_name, fields });
    }
    out
}

#[test]
fn rung_docs_claim_their_tag_slug_and_fixtures_exactly_once() {
    let root = repo_root();
    let rungs = load_rungs(&root.join("docs/rungs"));
    assert!(
        !rungs.is_empty(),
        "docs/rungs/ has no rung docs — the registry test would be vacuous"
    );

    let fixtures_dir = root.join("fixtures/cpp");

    let mut tags: BTreeMap<String, String> = BTreeMap::new();
    let mut slugs: BTreeMap<String, String> = BTreeMap::new();
    let mut prefixes: BTreeMap<String, String> = BTreeMap::new();

    for rung in &rungs {
        let file = &rung.file;

        let tag = rung
            .field("Tag")
            .unwrap_or_else(|| panic!("{file}: no `Tag:` in the header block"));
        let slug = rung
            .field("Slug")
            .unwrap_or_else(|| panic!("{file}: no `Slug:` in the header block"));
        let fixtures = rung
            .field("Fixtures")
            .unwrap_or_else(|| panic!("{file}: no `Fixtures:` in the header block"));

        assert!(!tag.is_empty(), "{file}: empty `Tag:`");
        if let Some(other) = tags.insert(tag.to_string(), file.clone()) {
            panic!(
                "tag {tag} claimed twice: {other} and {file}. Tags are assigned by \
                 the merge funnel, one serial actor allocating from one sequence \
                 (docs/ARCHITECTURE_SEAMS.md §3.4) — this is the W23 collision."
            );
        }

        assert_eq!(
            slug, rung.slug_from_name,
            "{file}: declared `Slug: {slug}` but the filename claims \
             `{}` — the filename is the claim",
            rung.slug_from_name
        );
        if let Some(other) = slugs.insert(slug.to_string(), file.clone()) {
            panic!("slug {slug} claimed twice: {other} and {file}");
        }

        // **The instrument-rung exception, kept narrow on purpose.** The rule
        // below exists because a widening rung with no fixture cannot be shown
        // to admit what it claims. An *instrument* rung admits nothing — it
        // ships a measurement, not an accepted class — so the rule's own reason
        // does not reach it, and demanding a fixture would produce a fake one:
        // a fixture proves nothing unless the census says N/N, and there is no
        // N to say.
        //
        // Two conditions, both textual so this stays portable, and both of
        // which a real widening rung would have to LIE about to satisfy:
        //   * `Fixtures: none — <reason>`, the reason mandatory; and
        //   * a `Census:` line carrying `+0`.
        // A rung that widened the class and still wrote `+0` is caught by the
        // gate, which is the check that owns that claim. Each condition has its
        // own failure message, because a shared one would let an early guard
        // make the later assertion unreachable — the lane-registry defect
        // (ROADMAP §9.18.8), where seven mutations all tripped one count and
        // the assertions behind it never ran.
        if let Some(reason) = fixtures.strip_prefix("none") {
            let reason = reason.trim_start_matches([' ', '\u{2014}', '-']).trim();
            assert!(
                !reason.is_empty(),
                "{file}: `Fixtures: none` needs a reason on the same line, e.g. \
                 `none — this rung ships an instrument, not an accepted class`. \
                 Bare `none` is indistinguishable from a widening rung that \
                 forgot its fixtures, which is what this rule exists to catch."
            );
            let census = rung.field("Census").unwrap_or("");
            assert!(
                census.contains("+0"),
                "{file}: `Fixtures: none` is only for a rung that admits nothing, \
                 so its `Census:` must record `+0`; got `{census}`. A rung that \
                 moved the census admitted a class, and a class with no fixture \
                 cannot be shown to be what it claims."
            );
            continue;
        }

        let named: Vec<&str> = fixtures.split_whitespace().collect();
        assert!(
            !named.is_empty(),
            "{file}: names no fixture. Every rung lands a positive and a negative \
             fixture graded N/N and 0/N — that is what makes the one-line dispatch \
             merges safe (docs/ARCHITECTURE_SEAMS.md §5.1)."
        );
        for fixture in &named {
            assert!(
                fixtures_dir.join(fixture).is_file(),
                "{file}: names fixture {fixture}, which does not exist in \
                 fixtures/cpp"
            );
        }

        let prefix = fixture_prefix(named[0]);
        for fixture in &named {
            assert_eq!(
                fixture_prefix(fixture),
                prefix,
                "{file}: fixtures span more than one prefix ({prefix} and {}) — \
                 a rung's fixture prefix is part of its identity",
                fixture_prefix(fixture)
            );
        }
        if let Some(other) = prefixes.insert(prefix.clone(), file.clone()) {
            panic!("fixture prefix {prefix} claimed twice: {other} and {file}");
        }
    }

    // Reported, not asserted — see the module doc.
    let mut unclaimed: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&fixtures_dir) {
        let mut seen: Vec<String> = Vec::new();
        for e in entries.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".cpp") {
                continue;
            }
            let p = fixture_prefix(&name);
            if !seen.contains(&p) {
                seen.push(p);
            }
        }
        seen.sort();
        for p in seen {
            if !prefixes.contains_key(&p) && !p.ends_with(".cpp") {
                unclaimed.push(p);
            }
        }
    }
    println!(
        "rung registry: {} rungs, {} tags, {} fixture prefixes claimed; \
         not yet claimed (ARCHITECTURE_SEAMS §6 step 4): {}",
        rungs.len(),
        tags.len(),
        prefixes.len(),
        if unclaimed.is_empty() { "none".to_string() } else { unclaimed.join(" ") }
    );
}

#[test]
fn rung_index_is_generated_and_current() {
    let root = repo_root();
    let script = root.join("scripts/gen_rung_index.sh");
    let index = root.join("docs/rungs/INDEX.md");
    assert!(script.is_file(), "scripts/gen_rung_index.sh is missing");

    let out = match Command::new("/bin/sh").arg(&script).arg("-").output() {
        Ok(out) => out,
        // Degrade cleanly rather than failing the portable lane on a machine
        // without a POSIX shell (CLAUDE.md: never panic when a tool is absent).
        Err(e) => {
            println!("SKIP: cannot run /bin/sh ({e})");
            return;
        }
    };
    assert!(
        out.status.success(),
        "gen_rung_index.sh failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let generated = String::from_utf8_lossy(&out.stdout).into_owned();
    let on_disk = std::fs::read_to_string(&index).expect("docs/rungs/INDEX.md readable");
    assert_eq!(
        generated, on_disk,
        "docs/rungs/INDEX.md is stale — it is GENERATED. Run \
         `scripts/gen_rung_index.sh`."
    );
}

// ===========================================================================
// **THE FAIL-AXIS REQUIREMENT** — board **#3744**, lane `w-doctrine`, wave 17.
//
// `docs/rungs/README.md`'s construct-rung definition grades on a **required-zero
// byte delta**. Board `#3723` measured that this passes a real emit widening:
// `w-regsel`'s control C6 opened the caller's allowed register set to `r0..r31`
// and 471 of 475 crate tests, every encoder row and both gate ends stayed green.
//
// The executable half of the answer is `c2_core::surface` — a registry of
// decision surfaces whose domains run past what the corpus exercises. It can
// only cover surfaces somebody registered. **This is the half that reaches an
// UNREGISTERED surface**: a construct rung has to name, in its header, the axis
// on which it can fail with every byte identical, and for a rung over an allowed
// set, a candidate set or a refusal boundary that axis is the refusal domain.
//
// **This check is weaker than it looks and the weakness is stated on purpose.**
// It verifies that a field is present and non-empty. It cannot verify that the
// axis was measured, or that the thing named is really an axis. Presence is not
// measurement. It ships because it is the only part of this that binds a FUTURE
// lane over a surface `c2_core::surface::SURFACES` does not hold, and because
// `#3679`/`#3689` are this repo's standing finding that an unenforced rule is a
// paragraph — a paragraph in `README.md` is what was already there.
//
// **Scope, declared rather than discovered (the `#3684` objection).** The
// population is construct rungs dated on or after [`FAIL_AXIS_CUTOFF`]. Every
// construct rung in the tree predates it, so the tree run grades **zero** docs
// today — which is exactly the state `#3470` says is green-and-says-nothing, and
// is why the detector has its own two-directional self-test below rather than
// relying on the tree ever entering the population. Blast radius: a rung doc
// only exists in the worktree of the lane writing it until it merges, and the
// merge funnel (`#3687`) runs `cargo test --workspace`, so a doc that fails this
// cannot reach master and cannot redden a peer's tree.
// ===========================================================================

/// Construct rungs dated on or after this must name a fail axis. Dated records
/// before it stay exactly as written (`docs/rungs/README.md`; the same
/// grandfathering `#3689`'s ceiling uses, and for the same reason).
///
/// PROV[N] a policy cutoff date; reaches no emitted byte.
const FAIL_AXIS_CUTOFF: &str = "2026-08-28";

/// The indented `Key: value` header block at the top of a rung doc, as raw
/// lines. Deliberately a **second, more tolerant reader** than [`load_rungs`]'s:
/// that one drops any key containing whitespace, which silently discards
/// `Fail axis:` — the very field this checks for. Widening the shared parser
/// would change what every existing assertion sees; this does not.
fn header_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut started = false;
    for line in text.lines() {
        match line.strip_prefix("    ") {
            Some(rest) => {
                started = true;
                out.push(rest);
            }
            None => {
                if started && !line.trim().is_empty() {
                    break;
                }
            }
        }
    }
    out
}

/// The value of a header field whose key may contain spaces or hyphens, matched
/// case-insensitively. Continuation lines (an indented line with no `Key:` of
/// its own) count toward the value, so a multi-line axis is not read as empty.
fn header_field(text: &str, key: &str) -> Option<String> {
    let want = key.to_ascii_lowercase().replace(['-', ' '], "");
    let lines = header_lines(text);
    let mut value: Option<String> = None;
    for line in lines {
        let Some((k, v)) = line.split_once(':') else {
            if let Some(acc) = value.as_mut() {
                acc.push(' ');
                acc.push_str(line.trim());
            }
            continue;
        };
        let k_norm = k.trim().to_ascii_lowercase().replace(['-', ' '], "");
        if k_norm.chars().all(|c| c.is_ascii_alphanumeric()) && !k_norm.is_empty() {
            if k_norm == want {
                value = Some(v.trim().to_string());
            } else if value.is_some() {
                break;
            }
        } else if let Some(acc) = value.as_mut() {
            acc.push(' ');
            acc.push_str(line.trim());
        }
    }
    value
}

/// Does this doc declare itself a construct rung?
fn declares_construct_rung(text: &str) -> bool {
    header_field(text, "Kind")
        .map(|k| k.to_ascii_lowercase().contains("construct rung"))
        .unwrap_or(false)
}

/// The one predicate, so the tree run and the self-test cannot drift apart.
/// `Some(reason)` is a violation.
fn fail_axis_violation(file_name: &str, text: &str) -> Option<String> {
    if !declares_construct_rung(text) {
        return None;
    }
    let date = file_name.get(..10).unwrap_or("");
    if date < FAIL_AXIS_CUTOFF {
        return None;
    }
    match header_field(text, "Fail axis") {
        None => Some(format!(
            "{file_name} declares `Kind: construct rung` and carries no `Fail axis:` \
             header field"
        )),
        Some(v) if v.trim().is_empty() => {
            Some(format!("{file_name} carries an EMPTY `Fail axis:` field"))
        }
        Some(_) => None,
    }
}

/// **The tree run.** Board `#3744`.
#[test]
fn construct_rungs_from_the_cutoff_name_an_axis_they_can_fail_on() {
    let root = repo_root();
    let dir = root.join("docs/rungs");
    let mut examined = 0usize;
    let mut construct = 0usize;
    let mut in_population = 0usize;
    let mut violations: Vec<String> = Vec::new();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .expect("docs/rungs readable")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        // `_`-prefixed files are PREREGS, not rungs, and `load_rungs` excludes
        // them for the same reason: the requirement is on the rung's own
        // header. (Their names also sort ABOVE the cutoff because `_` > `2`,
        // which is how this exclusion was discovered rather than assumed.)
        .filter(|n| n.ends_with(".md") && !n.starts_with('_') && n != "INDEX.md" && n != "README.md")
        .collect();
    names.sort();
    for name in &names {
        let text = std::fs::read_to_string(dir.join(name)).expect("rung doc readable");
        examined += 1;
        if declares_construct_rung(&text) {
            construct += 1;
            if name.get(..10).unwrap_or("") >= FAIL_AXIS_CUTOFF {
                in_population += 1;
            }
        }
        if let Some(v) = fail_axis_violation(name, &text) {
            violations.push(v);
        }
    }
    // Denominators, printed rather than asserted, because the population being
    // zero is the EXPECTED state at the cutoff and asserting it nonzero would
    // make this test fail for being new. What is asserted instead is that the
    // detector works, immediately below.
    println!(
        "fail-axis: {examined} rung docs examined, {construct} declare a construct rung, \
         {in_population} dated at or after {FAIL_AXIS_CUTOFF} (the graded population), \
         {} violation(s)",
        violations.len()
    );
    assert!(examined > 100, "only {examined} rung docs found — the scan is not reading docs/rungs");
    assert!(construct > 5, "only {construct} construct rungs found — the Kind: parser has stopped matching");
    assert!(
        violations.is_empty(),
        "A CONSTRUCT RUNG WITH NO FAIL AXIS (board #3744, #3723).\n\n\
         A required-zero byte delta is silent about everything that is not a \
         byte, and #3723 measured that it is silent about a real emit widening \
         too. Name, in the rung header, the axis on which this rung can fail \
         with every byte identical -- for a rung over an allowed set, a \
         candidate set or a refusal boundary that is the REFUSAL DOMAIN, and \
         c2_core::surface is where it goes so it is enumerated and not just \
         asserted.\n\n{}",
        violations.join("\n")
    );
}

/// **The control.** `#3336`: a check never watched failing is decoration — and
/// this one grades zero docs today, so without this it would be decoration by
/// construction.
#[test]
fn the_fail_axis_detector_fires_and_stays_quiet_in_the_right_places() {
    let with_axis = "# x\n\n    Tag:        w-x\n    Kind:       construct rung\n    Fail axis:  the refusal domain, tabulated over a 63-cell grid\n\n---\n";
    let without = "# x\n\n    Tag:        w-x\n    Kind:       construct rung\n    Census:     +0\n\n---\n";
    let empty = "# x\n\n    Tag:        w-x\n    Kind:       construct rung\n    Fail axis:\n\n---\n";
    let hyphen = "# x\n\n    Kind:       construct rung\n    Fail-axis:  throughput\n\n---\n";
    let not_construct = "# x\n\n    Tag:        w-x\n    Kind:       characterization\n\n---\n";

    // Fires.
    assert!(fail_axis_violation("2026-08-28-w-x.md", without).is_some(), "a construct rung with no Fail axis must be caught");
    assert!(fail_axis_violation("2026-08-28-w-x.md", empty).is_some(), "an EMPTY Fail axis must be caught");
    assert!(fail_axis_violation("2099-01-01-w-x.md", without).is_some(), "a later date is still in the population");

    // Stays quiet.
    assert!(fail_axis_violation("2026-08-28-w-x.md", with_axis).is_none(), "a named axis must pass");
    assert!(fail_axis_violation("2026-08-28-w-x.md", hyphen).is_none(), "`Fail-axis:` is the same field");
    assert!(fail_axis_violation("2026-08-28-w-x.md", not_construct).is_none(), "only construct rungs are graded");
    assert!(fail_axis_violation("2026-08-27-w-x.md", without).is_none(), "docs before the cutoff are grandfathered");

    // And the field reader itself, since a reader that returns None for
    // everything would make the whole check silently vacuous in one direction.
    assert_eq!(header_field(with_axis, "Kind").as_deref(), Some("construct rung"));
    assert_eq!(header_field(without, "Fail axis"), None);
    assert!(header_field(with_axis, "Fail axis").unwrap().contains("63-cell"));
}
