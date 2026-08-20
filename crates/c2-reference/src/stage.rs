//! `c2-reference::stage` — **the stage oracle**: observe real `c2.dll`'s
//! per-function phase boundaries while it compiles, without changing what it
//! compiles to.
//!
//! # Why this exists
//!
//! The sole judge of the port is a byte compare of the finished obj. It says
//! *"differs"* and nothing about **where**, so every divergence costs a
//! whole-object byte archaeology session — which is what makes lanes here
//! expensive (a single alignment nibble cost a lane; `dag.c`'s lowering order
//! cost two). The prize this module is chasing is **divergence localization**:
//! a divergence attributable to a *pass* rather than to an obj.
//!
//! # What already existed, so nothing here is re-shipped
//!
//! The `/FAsc` + `/QXSTALLS` narration seam ([`crate::Toolchain::capture_listing_with`],
//! [`crate::cod`], `c2rs listing-scan`, boards #132/#134/#136) already makes c2
//! narrate its own output: label counter, section order, EH layout,
//! relocations by name, per-instruction issue cycles. That is an **end-state**
//! observation — after all four scheduler runs, after COLOR, after lowering —
//! and it structurally cannot separate COLOR's output from the scheduler's.
//! `/QXSTALLS`'s cycles are worse than merely late: they come from **K4**
//! (`0x10c1ce93`), which builds its *own* whole-function DAG read-only and
//! tears it down (`docs/whitebox/WB_DAGCLIENTS_FINDINGS.md` §4.4), so they are
//! a re-derivation and not the schedule the scheduler produced.
//!
//! This module observes c2 **between** passes instead, by detouring its own
//! per-function phase call sites. The machinery is in `c2host/stagetap.c`; this
//! file is the Rust seam that arms it, runs a replay through it, and parses
//! what it reported.
//!
//! # The standing bound on everything here
//!
//! **A snapshot is a development instrument.** It never gates an emit, never
//! appears in a refusal predicate, and no rule enters `crates/` on snapshot
//! equality alone. The obj byte compare against real `c2.dll` remains the sole
//! judge — which is exactly why [`TapReport`] is delivered *beside* an
//! [`ObjImage`] that the caller is expected to compare against the untapped
//! one.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use c2_core::ObjImage;

use crate::{CapturedReference, Toolchain};

/// Every site `c2host/stagetap.c` knows how to arm, in table order.
///
/// **This list is duplicated on purpose and the duplication is tested.**
/// `crates/c2-reference/tests/stage.rs::the_two_site_tables_are_one_table`
/// parses `c2host/stagetap.c`'s own table and asserts equality, the pattern
/// `docs/ARCHITECTURE_SEAMS.md` §0 used for the `..base` and `bind.rs` moves:
/// two readers of one definition, with a test standing where a shared header
/// cannot (the C side is not in the Rust workspace and never will be).
pub const STAGE_SITES: &[&str] = &[
    "sched1", "globregs", "sched2", "color", "sched3", "sched0", "region",
];

/// The six sites gated on c2's optimizer-on flag `DAT_10c2e2fc`.
///
/// `P_DAG.md` §1 and the bytes at `0x10b7dc83`/`0x10b7dcc2`/`0x10b7dd01`
/// (`cmp DWORD PTR ds:0x10c2e2fc,edi` with `edi == 0`): at `/Od` none of the
/// scheduler runs happen. This is what makes the `/Od`-vs-`/O1` null control a
/// property of the code rather than a hope.
pub const OPT_GATED_SITES: &[&str] = &["sched1", "sched2", "sched3", "sched0"];

/// What one armed run reported.
///
/// Deliberately **not** a snapshot digest type: at this revision the payload is
/// counts only, and a type that promised more than the mechanism delivers is
/// precisely the "deterministic and vacuous" failure this lane registered as
/// its most-likely-misreported outcome.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TapReport {
    /// `true` iff c2.dll loaded at its preferred base (slide 0). Recorded as a
    /// boolean, never as an address: an address in the stream would make the
    /// canonical form environment-dependent.
    pub slide_zero: bool,
    /// Sites armed, in table order.
    pub armed: Vec<String>,
    /// `(site, reason)` for every site that refused. A refusal is fail-closed:
    /// the image was **not** written.
    pub refused: Vec<(String, String)>,
    /// Hit count per armed site.
    pub hits: BTreeMap<String, u64>,
    /// Every `[stagetap]` line, verbatim, so a result table can be re-derived
    /// from the log rather than accumulated (#3231 F2).
    pub lines: Vec<String>,
    /// The payload's `TU <idx> <opcode> <cat> <flags> <cc>` rows, in order,
    /// with the leading `TU ` stripped. Empty on a counts-only run.
    pub tuples: Vec<String>,
    /// `REFUSE …` lines emitted by the bounded walk (overrun, span, arena
    /// full, implausible pointer). A non-empty list means the payload is
    /// TRUNCATED and the caller must not read absence as a terminus.
    pub walk_refusals: Vec<String>,
    /// Number of `SITE region ENTER` blocks in the payload.
    pub regions: usize,
    /// Raw tuple windows, when requested: same order as [`TapReport::tuples`].
    /// **Never** part of [`TapReport::canonical_bytes`].
    pub raw: Vec<String>,
    /// One entry per region block: `(phase, function-ordinal, tuple rows)`.
    ///
    /// The phase is the per-function site that was last entered before this
    /// region — `sched2` is immediately BEFORE the register allocator and
    /// `sched3` immediately after (`P_DAG.md` §1's order
    /// `sched1 -> globregs -> sched2 -> COLOR -> sched3`), so a pre/post-COLOR
    /// pair falls out of the region tap without ever needing the
    /// function-record → tuple-list-head offset.
    pub blocks: Vec<StageBlock>,
}

/// One scheduling region as observed at one phase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageBlock {
    /// The per-function site last entered: `sched1`/`globregs`/`sched2`/
    /// `color`/`sched3`/`sched0`.
    pub phase: String,
    /// Function ordinal within the TU (counts `sched1` entries).
    pub func: u32,
    /// `<idx> <opcode> <cat> <flags> <cc>` rows, in walk order.
    pub tuples: Vec<String>,
    /// Raw hex windows, same order, when a raw window was requested.
    pub raw: Vec<String>,
}

impl TapReport {
    /// The region blocks observed at one phase, for one function.
    pub fn blocks_at(&self, phase: &str, func: u32) -> Vec<&StageBlock> {
        self.blocks
            .iter()
            .filter(|b| b.phase == phase && b.func == func)
            .collect()
    }

    /// **The pre/post-COLOR pair.** Returns `(before, after)` as the
    /// concatenated tuple rows of every region observed at `sched2` and at
    /// `sched3` for one function.
    ///
    /// An EMPTY difference is a finding, not a green: it means the walk is
    /// reading a list COLOR does not write, and it is reported in those words.
    pub fn color_pair(&self, func: u32) -> (Vec<String>, Vec<String>) {
        let cat = |phase: &str| -> Vec<String> {
            self.blocks_at(phase, func)
                .into_iter()
                .flat_map(|b| b.tuples.iter().cloned())
                .collect()
        };
        (cat("sched2"), cat("sched3"))
    }

    /// Did the tap actually arm? **The environment control's predicate.**
    ///
    /// A run where nothing armed produces zero hits, and zero hits is
    /// indistinguishable from "the pass never ran" unless something asserts
    /// this. That is #3219/#3231's failure — a registered RED reading GREEN
    /// with a clean suite and the right exit code — in a new place.
    pub fn armed_ok(&self) -> bool {
        !self.armed.is_empty() && self.refused.is_empty()
    }

    /// Total hits over all armed sites.
    pub fn total_hits(&self) -> u64 {
        self.hits.values().copied().sum()
    }

    /// The payload's canonical bytes: the exact stream a digest is taken over.
    ///
    /// **SCHEMA RULE, and it is the whole of G2b:** no address, no pointer, no
    /// path, no PID, no timestamp and no allocation count may appear. Only
    /// walk indices and values read out of c2's own records. Without this a
    /// digest is stable only because the environment was.
    pub fn canonical_bytes(&self) -> String {
        let mut out = String::new();
        out.push_str("SCHEMA 1\n");
        out.push_str(if self.slide_zero { "SLIDE 0\n" } else { "SLIDE nonzero\n" });
        for s in &self.armed {
            out.push_str("ARMED ");
            out.push_str(s);
            out.push('\n');
        }
        for (k, v) in &self.hits {
            out.push_str("HITS ");
            out.push_str(k);
            out.push(' ');
            out.push_str(&v.to_string());
            out.push('\n');
        }
        for t in &self.tuples {
            out.push_str("TU ");
            out.push_str(t);
            out.push('\n');
        }
        for w in &self.walk_refusals {
            out.push_str(w);
            out.push('\n');
        }
        out
    }

    /// FNV-1a over [`TapReport::canonical_bytes`]. Hand-rolled: std-only, zero
    /// external crates, and nothing here needs a cryptographic digest — this
    /// compares a stream against itself across runs.
    pub fn digest(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.canonical_bytes().as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h
    }

    /// Hits at one site (0 when unarmed or never reached — the caller is
    /// expected to have checked [`TapReport::armed_ok`] first).
    pub fn hits_at(&self, site: &str) -> u64 {
        self.hits.get(site).copied().unwrap_or(0)
    }

    /// Parse c2host's stderr. Tolerant of interleaved c2 output by construction:
    /// only lines carrying the `[stagetap] ` marker are read.
    pub fn parse(stderr: &str) -> TapReport {
        let mut r = TapReport::default();
        for raw in stderr.lines() {
            let Some(rest) = raw.trim_start().strip_prefix("[stagetap] ") else {
                continue;
            };
            r.lines.push(rest.to_string());
            let mut it = rest.split_whitespace();
            match it.next() {
                Some("ARM") => {
                    if let Some(name) = it.next() {
                        r.armed.push(name.to_string());
                    }
                }
                Some("REFUSE") => {
                    // Two different refusals share the word. An ARMING refusal
                    // names a site and means nothing was patched; a WALK
                    // refusal means the payload is truncated. Conflating them
                    // would let a truncated payload read as a failed arm (or,
                    // worse, the reverse).
                    if rest.contains("walk-") || rest.contains("arena-full") {
                        r.walk_refusals.push(rest.to_string());
                    } else {
                        let name = it.next().unwrap_or("?").to_string();
                        r.refused.push((name, rest.to_string()));
                    }
                }
                Some("END") => {
                    let name = it.next().unwrap_or("?").to_string();
                    let n = it
                        .find_map(|t| t.strip_prefix("hits="))
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    r.hits.insert(name, n);
                }
                Some("TAP") => {
                    r.slide_zero = rest.contains("slide=0");
                }
                Some("RAW") => {
                    let t: Vec<&str> = rest.split_whitespace().collect();
                    if let Some(h) = t.get(2) {
                        r.raw.push((*h).to_string());
                        if let Some(b) = r.blocks.last_mut() {
                            b.raw.push((*h).to_string());
                        }
                    }
                }
                Some("TU") => {
                    let row = rest["TU ".len()..].to_string();
                    if let Some(b) = r.blocks.last_mut() {
                        b.tuples.push(row.clone());
                    }
                    r.tuples.push(row);
                }
                Some("SITE") => {
                    r.regions += 1;
                    // SITE region ENTER <phase> fn <n> r <n>
                    let t: Vec<&str> = rest.split_whitespace().collect();
                    let phase = t.get(3).copied().unwrap_or("?").to_string();
                    let func = t
                        .get(5)
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(0);
                    r.blocks.push(StageBlock { phase, func, tuples: Vec::new(), raw: Vec::new() });
                }
                _ => {}
            }
        }
        r
    }
}

impl Toolchain {
    /// **P0.1 replay with the stage tap armed.**
    ///
    /// Identical to [`Toolchain::replay`] in every respect except two
    /// environment variables, and that is the point: the neutrality claim this
    /// whole instrument stands on is *"the same command, plus arming, produces
    /// the same obj"*, and it is only a real claim if the command really is the
    /// same one. `build_replay_command` is reused unchanged.
    ///
    /// Pass an empty `taps` to get the **disarmed** control through the exact
    /// same code path — which is how the neutrality test avoids comparing two
    /// different functions and calling the result a measurement.
    pub fn replay_tapped(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
    ) -> io::Result<(ObjImage, TapReport)> {
        self.replay_tapped_with(captured, bundle_dir, out_obj, taps, false)
    }

    /// [`Toolchain::replay_tapped`] with the bounded tuple-walk payload
    /// switchable.
    ///
    /// Counts and payload are the SAME mechanism at two settings, deliberately:
    /// it means the expensive content run and the cheap neutrality sweep are
    /// not two different instruments whose agreement would have to be argued.
    /// It also means **G1 can be re-run at the full table WITH the payload**
    /// rather than extrapolated from a counts-only run — the payload is the
    /// half that touches c2's own memory, so extrapolating would be assuming
    /// the answer.
    pub fn replay_tapped_with(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        payload: bool,
    ) -> io::Result<(ObjImage, TapReport)> {
        self.replay_tapped_raw(captured, bundle_dir, out_obj, taps, payload, 0)
    }

    /// [`Toolchain::replay_tapped_with`] plus a **raw window**: `raw` bytes of
    /// every tuple, dumped as hex beside the decoded row.
    ///
    /// This is how "which fields does COLOR write?" gets ANSWERED instead of
    /// guessed — diff the `sched2` and `sched3` dumps and the offsets the
    /// register allocator touches name themselves. It is **excluded from
    /// [`TapReport::canonical_bytes`] by construction**: a raw window can
    /// contain pointers, and a digest over pointers is stable only because the
    /// allocator happened to be.
    pub fn replay_tapped_raw(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        taps: &[&str],
        payload: bool,
        raw: u32,
    ) -> io::Result<(ObjImage, TapReport)> {
        let (mut cmd, out_abs) = self.build_replay_command(captured, bundle_dir, out_obj)?;
        if raw > 0 {
            cmd.env("C2RS_STAGE_RAW", raw.to_string());
        } else {
            cmd.env_remove("C2RS_STAGE_RAW");
        }
        if payload {
            cmd.env("C2RS_STAGE_PAYLOAD", "1");
        } else {
            cmd.env_remove("C2RS_STAGE_PAYLOAD");
        }
        if taps.is_empty() {
            // Inertness, asserted at the seam and not only in C: with nothing
            // requested the variable is REMOVED, so an ambient value in the
            // caller's environment cannot silently arm a "disarmed" control.
            cmd.env_remove("C2RS_STAGE_TAPS");
        } else {
            cmd.env("C2RS_STAGE_TAPS", taps.join(","));
        }
        let output = cmd.output()?;
        if !out_abs.exists() || std::fs::metadata(&out_abs)?.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "tapped replay produced no (or an empty) obj at {} — \
                     c2 crashed or aborted mid-pass.\nstderr:\n{}",
                    out_abs.display(),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
        let report = TapReport::parse(&String::from_utf8_lossy(&output.stderr));
        Ok((ObjImage::new(std::fs::read(&out_abs)?), report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_armed_run() {
        let s = "[c2host] noise\n\
                 [stagetap] hmodule=00000018 invoke=10bebffd slide=00000000\n\
                 [stagetap] ARM sched1 site=10b7dc9f -> 10be6382 (x)\n\
                 [stagetap] armed=1 of 7 requested-list=sched1\n\
                 c2 chatter\n\
                 [stagetap] TAP 1 c80981c0 slide=0\n\
                 [stagetap] END sched1 hits=12\n";
        let r = TapReport::parse(s);
        assert!(r.armed_ok());
        assert_eq!(r.armed, vec!["sched1".to_string()]);
        assert_eq!(r.hits_at("sched1"), 12);
        assert_eq!(r.total_hits(), 12);
        assert!(r.slide_zero);
    }

    #[test]
    fn a_refusal_is_not_an_armed_run() {
        let s = "[stagetap] REFUSE sched1 site=10b7dc9f opcode=00 (expected e8)\n\
                 [stagetap] armed=0 of 7 requested-list=sched1\n";
        let r = TapReport::parse(s);
        assert!(!r.armed_ok());
        assert_eq!(r.refused.len(), 1);
        assert_eq!(r.refused[0].0, "sched1");
    }

    #[test]
    fn an_unarmed_run_reports_nothing_and_is_not_ok() {
        // The inert path: no [stagetap] line at all. `armed_ok` must be false,
        // because "no output" is exactly what an unprovisioned environment
        // produces and it must never read as a green.
        let r = TapReport::parse("[c2host] returned 0\n");
        assert!(!r.armed_ok());
        assert_eq!(r.total_hits(), 0);
        assert!(r.lines.is_empty());
    }
}
