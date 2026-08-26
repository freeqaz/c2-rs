//! **Per-fixture compile profiles** — the flags `oracle_selftest` compiles a
//! fixture at, declared in the fixture itself.
//!
//! ## The defect this exists for
//!
//! `all_fixtures()` hands every `fixtures/cpp/*.cpp` to `oracle_selftest`, which
//! compiled all of them at [`CAPTURE_IL_DEFAULT_FLAGS`] (`/Ox /GS- /c`). There
//! was no per-fixture profile and no way to declare one. That is fine right up
//! until a fixture's *class* is a construct `cl.exe` refuses to compile, and on
//! 2026-08-09 exactly that landed: `wmain_no_return.cpp` and
//! `wmain_no_return_neg.cpp` exist to pin **a non-`void` function with no
//! `return` statement**, which this compiler reports as
//!
//! ```text
//! error C4716: 'no_return_call' : must return a value
//! ```
//!
//! — an *error*, so no obj, so `compile #1 failed`, so `c2rs selftest` exited
//! non-zero and `scripts/status.sh` rendered a named gate row RED plus two
//! dependent rows `NO-RESULT`. One fixture silently assumed to be compilable at
//! the one universal profile took a whole gate row down.
//!
//! **Measured, W-OXFIX, X360 `cl.exe` 16.00.11886.00 under wibo** — the cell
//! grid that decides which flag is responsible:
//!
//! ```text
//!   /Ox /GS- /c                          error C4716   no obj
//!   /Ox /c                               error C4716   no obj
//!   /O1 /c                               error C4716   no obj
//!   /O1 /GS- /c                          error C4716   no obj
//!   /Od /c                               error C4716   no obj
//!   /c                                   error C4716   no obj
//!   /Ox /GS- /EHsc /c                    error C4716   no obj
//!   /Ox /GS- /GR /c                      error C4716   no obj
//!   /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc        (the WORKLOAD's own
//!                                        error C4716    profile) no obj
//!   /Ox /GS- /w14716 /c                warning C4716   obj, 1394 B
//!   /Ox /GS- /wd4716 /c                    (silent)    obj, 1394 B  — byte-
//!                                                      identical to /w14716
//! ```
//!
//! **No optimization, EH, `/GS` or `/GR` flag promotes or demotes C4716.** It is
//! an error at every optimization level this toolchain has, including `/Od` and a
//! bare `/c`, and it is an error at the 878-TU workload's own profile. The only
//! thing that moves it is an explicit warning-level override, and neither profile
//! carries one. That refutes the shape the defect was first read as ("`/Ox`
//! promotes it, the workload is fine") — see `docs/rungs/2026-08-09-w-oxfix.md`.
//!
//! ## The declaration
//!
//! One line, in the fixture, machine-read:
//!
//! ```text
//! // c2rs-profile: /Ox /GS- /w14716 /c  # <why this fixture cannot use the default>
//! ```
//!
//! * **flags before the `#`** — the *complete* profile, not a delta. A delta
//!   would silently re-derive itself if the default moved; a full list is what an
//!   auditor can read against the default and against
//!   [`CAPTURE_IL_DEFAULT_FLAGS`] without running anything.
//! * **a reason after the `#`, mandatory and non-empty.** This is deliberately
//!   the same rule the rung registry puts on `Fixtures: none — <reason>`
//!   (`crates/c2-harness/tests/rung_registry.rs`): an opt-out that does not have
//!   to justify itself becomes a skip list, and a skip list is this project's
//!   "absence read as success" shape (ROADMAP §9.18.8). A profile marker with no
//!   reason is a hard error, not a warning.
//! * **at most one per file.** Two markers is a hard error rather than
//!   last-wins — the realistic way this goes wrong is a fixture copy-pasted from
//!   a sibling and half-edited, and last-wins would compile at a profile nobody
//!   chose.
//! * **`/c` is mandatory.** Without it `cl.exe` links, which is not what any
//!   fixture wants; catching it here beats catching it as a link error.
//!
//! A fixture that declares **nothing** gets [`CAPTURE_IL_DEFAULT_FLAGS`],
//! unchanged, byte-for-byte. That is the whole corpus but the declaring few, and
//! `fixture_profiles.rs` asserts it by count.
//!
//! ## What this is NOT
//!
//! **It is not a skip list, and there is no way to make it one.** A fixture that
//! cannot compile at its resolved profile — declared or default — is a *loud
//! named failure* carrying the fixture, the profile, and where the profile came
//! from ([`FixtureProfile::compile_failure`]). There is no verdict in this module
//! that drops a fixture from the corpus, and adding one would reintroduce exactly
//! the failure mode the module exists to close, one level quieter.

use std::fmt;
use std::path::{Path, PathBuf};

use c2_reference::CAPTURE_IL_DEFAULT_FLAGS;

/// The marker a fixture declares its profile with. Matched on the **trimmed**
/// line, so leading indentation is allowed and a marker inside a block comment
/// or a string is not (it must start the line).
/// PROV[N] not load-bearing — the in-source marker this repo's OWN fixture convention uses to carry a per-fixture flag list. A c2-rs convention, invisible to c2 (it is a C++ comment).
pub const PROFILE_MARKER: &str = "// c2rs-profile:";

/// Separator between the flag list and the mandatory reason.
/// PROV[N] not load-bearing — the separator inside that same c2-rs-only marker.
pub const PROFILE_REASON_SEP: char = '#';

/// The compile profile `oracle_selftest` will use for one fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureProfile {
    /// The flags handed to `cl.exe`, in order.
    pub flags: Vec<String>,
    /// The declared reason. Empty exactly when [`Self::declared`] is false.
    pub reason: String,
    /// 1-based line of the marker; 0 when the profile is the default.
    pub line: usize,
    /// `true` when the fixture declared this profile; `false` for the default.
    pub declared: bool,
}

impl FixtureProfile {
    /// [`CAPTURE_IL_DEFAULT_FLAGS`], as every non-declaring fixture gets it.
    pub fn default_profile() -> Self {
        FixtureProfile {
            flags: CAPTURE_IL_DEFAULT_FLAGS.iter().map(|s| s.to_string()).collect(),
            reason: String::new(),
            line: 0,
            declared: false,
        }
    }

    /// The flags as one space-separated string, for messages.
    pub fn flags_str(&self) -> String {
        self.flags.join(" ")
    }

    /// Where this profile came from, for messages.
    pub fn origin(&self, cpp: &Path) -> String {
        if self.declared {
            format!("declared at {}:{} — `{PROFILE_MARKER}`", cpp.display(), self.line)
        } else {
            format!(
                "the DEFAULT — {} has no `{PROFILE_MARKER}` line",
                cpp.display()
            )
        }
    }

    /// **The loud named failure.** A fixture that cannot compile at its resolved
    /// profile produces this, never a skip.
    ///
    /// The first line is self-contained on purpose: `selftest_row` renders only
    /// `first_line(msg)`, and the whole point of this lane is that the one-line
    /// form must name the fixture AND the profile. The tail carries the signpost
    /// — for a non-declaring fixture, how to declare one — so the next person to
    /// meet this hits an instruction rather than a puzzle.
    pub fn compile_failure(&self, cpp: &Path, stage: &str, err: &dyn fmt::Display) -> String {
        let name = cpp
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| cpp.display().to_string());
        let which = if self.declared { "its DECLARED" } else { "the DEFAULT" };
        let mut msg = format!(
            "fixture {name} does not compile at {which} profile [{}] ({})",
            self.flags_str(),
            self.origin(cpp),
        );
        if self.declared {
            msg.push_str(&format!("\n  declared reason: {}", self.reason));
        } else {
            msg.push_str(&format!(
                "\n  If this fixture's CLASS cannot be compiled at the default, declare a \
                 profile in the fixture itself:\n    {PROFILE_MARKER} <flags…> \
                 {PROFILE_REASON_SEP} <why>\n  See fixtures/README.md, \
                 \"Per-fixture compile profiles\". Do NOT add a skip list."
            ));
        }
        msg.push_str(&format!("\n  {stage}: {err}"));
        msg
    }
}

/// Everything that can be wrong with a declaration. Each is a hard failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    /// The fixture could not be read.
    Io { path: PathBuf, msg: String },
    /// Two `// c2rs-profile:` lines. Never last-wins.
    Duplicate { path: PathBuf, first: usize, second: usize },
    /// Marker present, no flags before the `#`.
    NoFlags { path: PathBuf, line: usize },
    /// Marker present, no `#` or an empty reason after it.
    NoReason { path: PathBuf, line: usize },
    /// Flags present but no `/c` — the profile would link.
    MissingSlashC { path: PathBuf, line: usize, flags: String },
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::Io { path, msg } => {
                write!(f, "cannot read fixture {}: {msg}", path.display())
            }
            ProfileError::Duplicate { path, first, second } => write!(
                f,
                "fixture {} declares `{PROFILE_MARKER}` twice (lines {first} and {second}) — \
                 a fixture has exactly one profile. Last-wins would compile it at a profile \
                 nobody chose, which is how a half-edited copy-paste ships.",
                path.display()
            ),
            ProfileError::NoFlags { path, line } => write!(
                f,
                "fixture {}:{line} declares `{PROFILE_MARKER}` with no flags before the \
                 `{PROFILE_REASON_SEP}`. An empty profile is not the default profile — say \
                 which flags you mean, in full.",
                path.display()
            ),
            ProfileError::NoReason { path, line } => write!(
                f,
                "fixture {}:{line} declares `{PROFILE_MARKER}` with no reason. Write \
                 `{PROFILE_MARKER} <flags…> {PROFILE_REASON_SEP} <why the default does not \
                 work>`. The reason is mandatory for the same reason the rung registry \
                 requires one on `Fixtures: none — <reason>`: an unjustified opt-out is a \
                 skip list.",
                path.display()
            ),
            ProfileError::MissingSlashC { path, line, flags } => write!(
                f,
                "fixture {}:{line} declares profile [{flags}], which has no `/c` — cl.exe \
                 would try to LINK. Every fixture profile is a compile-only profile.",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Parse a fixture's declaration out of its **text**. `path` is used only for
/// error messages, so this is callable from a test with no file on disk.
///
/// Returns `Ok(None)` when the fixture declares nothing — the overwhelmingly
/// common case, and the one that must keep resolving to the default.
pub fn parse_profile(path: &Path, text: &str) -> Result<Option<FixtureProfile>, ProfileError> {
    let mut found: Option<(usize, &str)> = None;
    for (i, line) in text.lines().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix(PROFILE_MARKER) else {
            continue;
        };
        let lineno = i + 1;
        if let Some((first, _)) = found {
            return Err(ProfileError::Duplicate {
                path: path.to_path_buf(),
                first,
                second: lineno,
            });
        }
        found = Some((lineno, rest));
    }
    let Some((line, rest)) = found else {
        return Ok(None);
    };

    let (flags_part, reason) = match rest.split_once(PROFILE_REASON_SEP) {
        Some((f, r)) => (f, r.trim()),
        None => {
            return Err(ProfileError::NoReason {
                path: path.to_path_buf(),
                line,
            })
        }
    };
    if reason.is_empty() {
        return Err(ProfileError::NoReason {
            path: path.to_path_buf(),
            line,
        });
    }
    let flags: Vec<String> = flags_part.split_whitespace().map(|s| s.to_string()).collect();
    if flags.is_empty() {
        return Err(ProfileError::NoFlags {
            path: path.to_path_buf(),
            line,
        });
    }
    if !flags.iter().any(|f| f == "/c") {
        return Err(ProfileError::MissingSlashC {
            path: path.to_path_buf(),
            line,
            flags: flags.join(" "),
        });
    }
    Ok(Some(FixtureProfile {
        flags,
        reason: reason.to_string(),
        line,
        declared: true,
    }))
}

/// Capture a **fixture's** pipeline reference (obj + IL bundle + c2 argv) at the
/// profile the fixture declares, or at [`CAPTURE_IL_DEFAULT_FLAGS`].
///
/// The one seam every fixture-corpus consumer of `capture_reference` goes
/// through — `differential` (`c2rs diff`), `perf::bench_fixture` and
/// `perf::scale_measure`. Without it those three hardcode `/Ox /GS- /c` and a
/// profile-declaring fixture reports a bare `replay produced no (or an empty)
/// obj`, which is what made `c2rs perf` exit non-zero and put
/// `scripts/status.sh`'s two perf rows at `NO-RESULT` even after `selftest` was
/// green. A malformed declaration is surfaced as an `InvalidData` error carrying
/// the full [`ProfileError`] text rather than being swallowed.
pub fn capture_fixture_reference(
    tc: &c2_reference::Toolchain,
    cpp: &Path,
    work_dir: &Path,
) -> std::io::Result<c2_reference::CapturedReference> {
    let profile = resolve_profile(cpp)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    tc.capture_reference_flags(cpp, work_dir, &profile.flags)
}

/// Read `cpp` and resolve the profile `oracle_selftest` will compile it at:
/// the declared one, or [`FixtureProfile::default_profile`].
///
/// A malformed declaration is an **error**, not a fallback to the default — a
/// typo'd marker that silently reverted to `/Ox` is the same class of bug as the
/// one this module closes.
pub fn resolve_profile(cpp: &Path) -> Result<FixtureProfile, ProfileError> {
    let text = std::fs::read_to_string(cpp).map_err(|e| ProfileError::Io {
        path: cpp.to_path_buf(),
        msg: e.to_string(),
    })?;
    Ok(parse_profile(cpp, &text)?.unwrap_or_else(FixtureProfile::default_profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> PathBuf {
        PathBuf::from("fixtures/cpp/x.cpp")
    }

    #[test]
    fn no_marker_resolves_to_the_default_unchanged() {
        let text = "int f(int a) { return a; }\n// an ordinary comment\n";
        assert_eq!(parse_profile(&p(), text), Ok(None));
        let d = FixtureProfile::default_profile();
        assert_eq!(d.flags, CAPTURE_IL_DEFAULT_FLAGS.to_vec());
        assert!(!d.declared);
        assert_eq!(d.line, 0);
    }

    #[test]
    fn a_declaration_parses_flags_reason_and_line() {
        let text = "// head\n// c2rs-profile: /Ox /GS- /w14716 /c  # C4716 is an error\nint f();\n";
        let got = parse_profile(&p(), text).unwrap().unwrap();
        assert_eq!(got.flags, ["/Ox", "/GS-", "/w14716", "/c"]);
        assert_eq!(got.reason, "C4716 is an error");
        assert_eq!(got.line, 2);
        assert!(got.declared);
    }

    #[test]
    fn indented_markers_are_read_and_mid_line_ones_are_not() {
        let indented = "    // c2rs-profile: /Ox /c # why\n";
        assert!(parse_profile(&p(), indented).unwrap().is_some());
        // Not at the start of the (trimmed) line — a mention in prose, not a
        // declaration. This is what keeps the module doc from declaring one.
        let prose = "// see `// c2rs-profile: /Ox /c # why` for the syntax\n";
        assert_eq!(parse_profile(&p(), prose), Ok(None));
    }

    #[test]
    fn two_markers_is_an_error_not_last_wins() {
        let text = "// c2rs-profile: /Ox /c # a\nint f();\n// c2rs-profile: /O1 /c # b\n";
        let err = parse_profile(&p(), text).unwrap_err();
        assert_eq!(err, ProfileError::Duplicate { path: p(), first: 1, second: 3 });
        assert!(err.to_string().contains("twice"));
    }

    #[test]
    fn a_reason_is_mandatory_with_and_without_the_separator() {
        for text in ["// c2rs-profile: /Ox /c\n", "// c2rs-profile: /Ox /c #   \n"] {
            let err = parse_profile(&p(), text).unwrap_err();
            assert_eq!(err, ProfileError::NoReason { path: p(), line: 1 });
        }
    }

    #[test]
    fn flags_are_mandatory() {
        let err = parse_profile(&p(), "// c2rs-profile:   # a reason\n").unwrap_err();
        assert_eq!(err, ProfileError::NoFlags { path: p(), line: 1 });
    }

    #[test]
    fn a_profile_without_slash_c_is_refused() {
        let err = parse_profile(&p(), "// c2rs-profile: /Ox /GS- # a reason\n").unwrap_err();
        assert_eq!(
            err,
            ProfileError::MissingSlashC { path: p(), line: 1, flags: "/Ox /GS-".into() }
        );
    }

    #[test]
    fn the_loud_failure_names_the_fixture_and_the_profile_on_its_first_line() {
        let cpp = PathBuf::from("fixtures/cpp/wmain_no_return.cpp");
        let declared = FixtureProfile {
            flags: ["/Ox", "/GS-", "/w14716", "/c"].iter().map(|s| s.to_string()).collect(),
            reason: "C4716".into(),
            line: 12,
            declared: true,
        };
        let msg = declared.compile_failure(&cpp, "compile #1", &"boom");
        let first = msg.lines().next().unwrap();
        assert!(first.contains("wmain_no_return.cpp"), "{first}");
        assert!(first.contains("/Ox /GS- /w14716 /c"), "{first}");
        assert!(first.contains("DECLARED"), "{first}");

        // The non-declaring form must carry the signpost, and must NOT suggest
        // skipping — that is the failure shape this module exists to refuse.
        let msg = FixtureProfile::default_profile().compile_failure(&cpp, "compile #1", &"boom");
        assert!(msg.lines().next().unwrap().contains("DEFAULT"));
        assert!(msg.contains(PROFILE_MARKER));
        assert!(msg.contains("Do NOT add a skip list"));
    }
}
