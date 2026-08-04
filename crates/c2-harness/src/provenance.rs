//! `c2-harness::provenance` — **what produced this number**, recorded on every
//! scan (roadmap #46, #48).
//!
//! # Why this exists
//!
//! A scan's headline is a ratio over a corpus, computed by a binary, driven by a
//! loader, against a compiler. Four moving parts, and the report used to name
//! none of them. Two instrument failures in one day came out of that gap:
//!
//! * **The corpus moved mid-session and the denominator matched anyway.** The
//!   guard in place was "`fn_total` is unchanged", and it held across a real
//!   change of the workload tree — so it is *proven insufficient*, not merely
//!   weak. The tree's own git HEAD is the identity; a count of functions is not.
//! * **A stale sibling wibo (`1.0.1-7` against the known-good `1.0.1-23`)
//!   silently turned the replay column `36 checked / 0 diverged` into `36/30`**
//!   while the census and the mismatch count stayed byte-identical. A fake
//!   correctness alarm on the oracle seam, from a binary nothing in the report
//!   named.
//!
//! So: every scan prints, and records in its JSONL, the workload tree's HEAD
//! (plus a dirty flag), the c2-rs HEAD, the resolved toolchain paths, and the
//! wibo version — with a loud warning when wibo is older than
//! [`c2_reference::WIBO_KNOWN_GOOD`].
//!
//! # Fail-soft, always
//!
//! `git` may be absent, the workload may not be a checkout, wibo may not answer
//! `--version`. Every field degrades to `unknown` and nothing here can fail a
//! run: the CLAUDE.md hard constraint is that the CLI degrades cleanly, and an
//! instrument that can abort the measurement is worse than one that admits it
//! does not know.

use std::path::{Path, PathBuf};
use std::process::Command;

use c2_reference::Toolchain;

use crate::jstr;

/// Repo root = this crate's `CARGO_MANIFEST_DIR` joined `../..`
/// (`.../c2-rs/crates/c2-harness` → `.../c2-rs`). Mirrors `c2-reference`'s own
/// resolution so nothing absolute is baked into source.
pub fn repo_root() -> PathBuf {
    tidy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

/// The **main** repository root — the same directory from every linked worktree.
///
/// # Why this exists
///
/// `repo_root()` is `CARGO_MANIFEST_DIR`, which is resolved **at compile time**,
/// so a binary built inside `.claude/worktrees/<lane>` reports *that worktree*
/// as the repo root. For provenance that is exactly right — the reader wants to
/// know which tree was measured. For anything meant to be **shared** between
/// lanes it is exactly wrong, and one thing is: the capture cache. Every lane
/// that built its own binary minted its own `work/capture-cache`, and on
/// 2026-08-04 there were 50 of them holding 3,996,458 entries — three of which
/// were independent, byte-for-byte-equivalent copies of the same 530k-entry
/// sweep, stored separately only because `cache-root` is part of the cache key
/// and their roots differed.
///
/// The documented workaround was to pass `C2RS_GAP_CACHE=<main-repo>/…`
/// verbatim from a worktree (docs/ROADMAP.md, docs/rungs/…w-vgl-prereg). That
/// works and still overrides this; it just has to be remembered once per lane,
/// which is why 50 caches exist.
///
/// # Why it is resolved here and not exported from a shell profile
///
/// A shared cache root makes two lanes compute the *same* key for the same
/// work, which is the point — and which also makes concurrent same-key captures
/// possible for the first time. Those are guarded by the `O_EXCL` lockfile in
/// `capture_cache`, and a lockfile only guards binaries that **have** it. An
/// environment variable would have pointed every already-built lane binary,
/// lock or no lock, at the shared root on its next run. Resolving it in code
/// ties the sharing and the guard to the same build: a lane on an old binary
/// keeps its own root and its old behaviour, and picks up both together when it
/// rebuilds. The rollout is monotone rather than a flag day.
///
/// # How
///
/// In a linked worktree `.git` is a *file* holding `gitdir: <main>/.git/worktrees/<name>`.
/// Three parents up from that is the main repo. Every failure — `.git` is a
/// directory (we are already in the main repo), no git at all, an unexpected
/// layout — falls back to `repo_root()`, so this can only ever collapse
/// worktrees onto their parent and never point somewhere unrelated.
pub fn main_repo_root() -> PathBuf {
    let root = repo_root();
    // `read_to_string` on a directory fails, which is the main-repo case.
    let Ok(s) = std::fs::read_to_string(root.join(".git")) else {
        return root;
    };
    let Some(rest) = s.trim().strip_prefix("gitdir:") else {
        return root;
    };
    let gitdir = Path::new(rest.trim());
    let gitdir = if gitdir.is_absolute() {
        gitdir.to_path_buf()
    } else {
        root.join(gitdir)
    };
    // <main>/.git/worktrees/<name> -> <main>
    match gitdir.parent().and_then(Path::parent).and_then(Path::parent) {
        // Confirm the destination really is a repo before redirecting to it.
        Some(main) if main.join(".git").is_dir() => tidy(main),
        _ => root,
    }
}

/// Collapse `a/b/../..` for display. A provenance line whose whole job is to
/// let a reader recognize which tree they measured is worth resolving; falls
/// back to the raw path when the target does not exist.
fn tidy(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

/// Identity of a git working tree: the commit, and whether the tree is dirty.
///
/// `head` is `"unknown"` when `git` is absent or the directory is not a
/// checkout; `dirty` is `None` in the same case (tri-state on purpose — "clean"
/// and "we could not tell" are different facts and a scan report must not
/// conflate them).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitInfo {
    pub head: String,
    pub dirty: Option<bool>,
}

impl GitInfo {
    /// The all-unknown value, used whenever git cannot answer.
    pub fn unknown() -> GitInfo {
        GitInfo {
            head: "unknown".to_string(),
            dirty: None,
        }
    }

    /// Interrogate `dir` with `git rev-parse` + `git status --porcelain`.
    /// Never fails: any error becomes [`GitInfo::unknown`].
    pub fn probe(dir: &Path) -> GitInfo {
        let head = match run_git(dir, &["rev-parse", "HEAD"]) {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => return GitInfo::unknown(),
        };
        // `-uno`: untracked files are build scratch in every tree we scan, and
        // they cannot change a compile. Tracked modifications can.
        let dirty = run_git(dir, &["status", "--porcelain", "-uno"])
            .map(|s| !s.trim().is_empty());
        GitInfo { head, dirty }
    }

    /// Short (12-char) commit, or `unknown`.
    pub fn short(&self) -> String {
        if self.head == "unknown" {
            self.head.clone()
        } else {
            self.head.chars().take(12).collect()
        }
    }

    /// `clean` / `DIRTY` / `dirty-unknown`, for the printed header.
    pub fn dirty_label(&self) -> &'static str {
        match self.dirty {
            Some(true) => "DIRTY",
            Some(false) => "clean",
            None => "dirty-unknown",
        }
    }

    /// A single token combining both facts — this is what a cache key wants,
    /// because "same commit, dirty" is not the same input as "same commit,
    /// clean".
    pub fn token(&self) -> String {
        format!("{}+{}", self.head, self.dirty_label())
    }
}

/// Run `git -C dir <args>` and return stdout, or `None` on any failure.
fn run_git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The running executable's path and content digest — see [`Provenance::binary`].
///
/// Reads the whole binary (a few MB) once per invocation, which is noise beside a
/// scan that spawns `cl.exe` under wibo hundreds of times.
fn binary_identity() -> Option<(PathBuf, String)> {
    let p = std::env::current_exe().ok()?;
    let bytes = std::fs::read(&p).ok()?;
    Some((tidy(&p), crate::capture_cache::digest128(&bytes)))
}

/// Everything a scan must name about its own inputs.
#[derive(Clone, Debug)]
pub struct Provenance {
    /// The c2-rs checkout this binary was built from.
    pub c2rs_dir: PathBuf,
    pub c2rs: GitInfo,
    /// The workload tree the TUs were compiled in (`--cwd`), when given.
    pub workload_dir: Option<PathBuf>,
    pub workload: GitInfo,
    /// Resolved loader + compiler binaries, verbatim.
    pub wibo: PathBuf,
    pub wibo_version: Option<String>,
    /// `Some(true)` = older than [`c2_reference::WIBO_KNOWN_GOOD`]; `None` = unknown.
    pub wibo_stale: Option<bool>,
    pub cl_exe: PathBuf,
    pub c2_dll: PathBuf,
    pub c1xx_dll: PathBuf,
    pub strace: Option<PathBuf>,
    pub mingw: Option<PathBuf>,
    /// **The running binary's own content digest**, and its path.
    ///
    /// The git fields above are TREE identity, and tree identity does not answer
    /// *"did these two runs grade the same code"* — the same question
    /// `scripts/harness_bin.sh` exists to answer for the sweep lanes, which pin a
    /// run-private copy and print its sha. `c2rs gap` had no such answer: it is
    /// the command that produces the census figure this project publishes, and it
    /// was the one instrument with no binary identity at all. A five-hour-stale
    /// binary once produced 47 phantom mismatches from exactly this shape, and
    /// the dangerous direction is the other one — a false GREEN.
    ///
    /// `None` when `current_exe()` or the read fails; fail-soft like every other
    /// field here.
    pub binary: Option<(PathBuf, String)>,
}

impl Provenance {
    /// Collect it. Runs `git` twice per tree and `wibo --version` once; all of
    /// them fail soft.
    pub fn collect(tc: &Toolchain, workload_dir: Option<&Path>) -> Provenance {
        let c2rs_dir = repo_root();
        let wibo_version = tc.wibo_version();
        let wibo_stale = wibo_version
            .as_deref()
            .and_then(c2_reference::parse_wibo_version)
            .and_then(|have| {
                c2_reference::parse_wibo_version(c2_reference::WIBO_KNOWN_GOOD)
                    .map(|want| have < want)
            });
        Provenance {
            c2rs: GitInfo::probe(&c2rs_dir),
            c2rs_dir,
            workload: workload_dir.map(GitInfo::probe).unwrap_or_else(GitInfo::unknown),
            workload_dir: workload_dir.map(tidy),
            wibo: tidy(&tc.wibo),
            wibo_version,
            wibo_stale,
            cl_exe: tidy(&tc.cl_exe),
            c2_dll: tidy(&tc.c2_dll),
            c1xx_dll: tidy(&tc.c1xx_dll),
            strace: tc.strace.as_deref().map(tidy),
            mingw: tc.mingw.as_deref().map(tidy),
            binary: binary_identity(),
        }
    }

    /// The wibo staleness warning, when one is warranted. Printed loudly, never
    /// fatal: toolchain location is env-driven by design (CLAUDE.md), so a scan
    /// against an odd loader is a legitimate thing to want.
    pub fn wibo_warning(&self) -> Option<String> {
        match self.wibo_stale {
            Some(true) => Some(format!(
                "WARNING: wibo {} is OLDER than the known-good {} — this loader loses \
                 `_CL_*` bundles (no WIBO_KEEP_TEMP) and turns the replay column into a \
                 FAKE divergence alarm while the census and mismatch count stay \
                 byte-identical. Point C2RS_WIBO at a >= {} build before believing any \
                 replay number below.",
                self.wibo_version
                    .as_deref()
                    .map(|v| v.trim_start_matches("wibo ").trim())
                    .unwrap_or("?"),
                c2_reference::WIBO_KNOWN_GOOD,
                c2_reference::WIBO_KNOWN_GOOD,
            )),
            None => Some(format!(
                "WARNING: could not read a version from {} — wibo staleness UNKNOWN (the \
                 known-good is {}).",
                self.wibo.display(),
                c2_reference::WIBO_KNOWN_GOOD,
            )),
            Some(false) => None,
        }
    }

    /// The human header block, one field per line, printed by `gap` and
    /// `selftest` before any measurement.
    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str("provenance:\n");
        s.push_str(&format!(
            "  c2-rs      {} ({})  {}\n",
            self.c2rs.short(),
            self.c2rs.dirty_label(),
            self.c2rs_dir.display()
        ));
        // Binary identity, immediately under tree identity, because the two are
        // different claims and the tree one is the weaker.
        s.push_str(&match &self.binary {
            Some((p, d)) => format!("  binary     {}  {}\n", &d[..16], p.display()),
            None => "  binary     (unreadable — this run has NO binary identity)\n".to_string(),
        });
        match &self.workload_dir {
            Some(d) => s.push_str(&format!(
                "  workload   {} ({})  {}\n",
                self.workload.short(),
                self.workload.dirty_label(),
                d.display()
            )),
            None => s.push_str("  workload   (none — fixtures)\n"),
        }
        s.push_str(&format!(
            "  wibo       {}  {}\n",
            self.wibo_version.as_deref().unwrap_or("version unknown"),
            self.wibo.display()
        ));
        s.push_str(&format!("  cl.exe     {}\n", self.cl_exe.display()));
        s.push_str(&format!("  c2.dll     {}\n", self.c2_dll.display()));
        s.push_str(&format!("  c1xx.dll   {}\n", self.c1xx_dll.display()));
        s.push_str(&format!(
            "  strace     {}\n",
            self.strace
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(absent)".to_string())
        ));
        s.push_str(&format!(
            "  mingw      {}\n",
            self.mingw
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(absent)".to_string())
        ));
        if let Some(w) = self.wibo_warning() {
            s.push_str(&format!("  {w}\n"));
        }
        s
    }

    /// The JSONL provenance record — written as the **first line** of a scan's
    /// JSONL, tagged `"record":"provenance"` so a consumer can skip it with
    /// `if r.get("record"): continue`. Per-TU rows are unchanged and carry no
    /// `record` field, so two scans' rows stay byte-comparable.
    pub fn to_json(&self, extra: &[(&str, String)]) -> String {
        let mut s = String::from("{\"record\":\"provenance\"");
        let mut put = |k: &str, v: String| {
            s.push(',');
            s.push_str(&jstr(k));
            s.push(':');
            s.push_str(&v);
        };
        put("c2rs_dir", jstr(&self.c2rs_dir.display().to_string()));
        put("c2rs_head", jstr(&self.c2rs.head));
        put("c2rs_dirty", opt_bool(self.c2rs.dirty));
        put(
            "binary_sha",
            match &self.binary {
                Some((_, d)) => jstr(d),
                None => "null".to_string(),
            },
        );
        put(
            "binary_path",
            match &self.binary {
                Some((p, _)) => jstr(&p.display().to_string()),
                None => "null".to_string(),
            },
        );
        put(
            "workload_dir",
            match &self.workload_dir {
                Some(d) => jstr(&d.display().to_string()),
                None => "null".to_string(),
            },
        );
        put("workload_head", jstr(&self.workload.head));
        put("workload_dirty", opt_bool(self.workload.dirty));
        put("wibo", jstr(&self.wibo.display().to_string()));
        put(
            "wibo_version",
            match &self.wibo_version {
                Some(v) => jstr(v),
                None => "null".to_string(),
            },
        );
        put("wibo_stale", opt_bool(self.wibo_stale));
        put("wibo_known_good", jstr(c2_reference::WIBO_KNOWN_GOOD));
        put("cl_exe", jstr(&self.cl_exe.display().to_string()));
        put("c2_dll", jstr(&self.c2_dll.display().to_string()));
        put("c1xx_dll", jstr(&self.c1xx_dll.display().to_string()));
        put(
            "strace",
            match &self.strace {
                Some(p) => jstr(&p.display().to_string()),
                None => "null".to_string(),
            },
        );
        put(
            "mingw",
            match &self.mingw {
                Some(p) => jstr(&p.display().to_string()),
                None => "null".to_string(),
            },
        );
        for (k, v) in extra {
            put(k, v.clone());
        }
        s.push('}');
        s
    }
}

fn opt_bool(b: Option<bool>) -> String {
    match b {
        Some(v) => v.to_string(),
        None => "null".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_info_unknown_is_tri_state() {
        let u = GitInfo::unknown();
        assert_eq!(u.short(), "unknown");
        assert_eq!(u.dirty_label(), "dirty-unknown");
        // "we could not tell" must never render as "clean".
        assert_ne!(u.dirty_label(), GitInfo { head: "x".into(), dirty: Some(false) }.dirty_label());
    }

    #[test]
    fn git_probe_on_a_non_repo_degrades_to_unknown() {
        // /proc is never a checkout; the probe must not panic or fail.
        assert_eq!(GitInfo::probe(Path::new("/proc")), GitInfo::unknown());
    }

    /// Every worktree of one repo must resolve to ONE cache root.
    ///
    /// The assert that matters is the second: this test is compiled inside
    /// whichever tree it is run from, so when that tree is a linked worktree
    /// `main_repo_root()` must differ from `repo_root()` — and when it is the
    /// main repo they must agree. Both directions are checked here rather than
    /// only the one the current checkout happens to exercise, because the whole
    /// defect was that the worktree case silently did something reasonable.
    #[test]
    fn every_worktree_resolves_to_one_main_repo_root() {
        let root = repo_root();
        let main = main_repo_root();

        // Whatever the answer, it must be a real repository — never a path
        // invented by a mis-parse.
        assert!(
            main.join(".git").exists(),
            "main_repo_root() must name a repository, got {}",
            main.display()
        );

        let dotgit = root.join(".git");
        if dotgit.is_dir() {
            // Main repo: unchanged behaviour, which is the compatibility claim.
            assert_eq!(root, main, "in the main repo the two must agree");
        } else if dotgit.is_file() {
            // Linked worktree: it must collapse, and it must collapse UPWARD —
            // the worktree has to live under the main repo it points at.
            assert_ne!(
                root, main,
                "a linked worktree must not resolve to itself, or every lane \
                 mints its own capture cache again"
            );
            assert!(
                root.starts_with(&main),
                "{} is not under {}",
                root.display(),
                main.display()
            );
        }
    }

    #[test]
    fn git_token_separates_clean_from_dirty_at_the_same_commit() {
        let clean = GitInfo { head: "abc".into(), dirty: Some(false) };
        let dirty = GitInfo { head: "abc".into(), dirty: Some(true) };
        assert_ne!(clean.token(), dirty.token());
    }
}
