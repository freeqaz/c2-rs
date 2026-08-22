//! `c2-harness::perf` — the latency benchmark: how fast can each side turn a
//! captured IL bundle into an `.obj`?
//!
//! Throughput is a **property** of this port, not its goal. A faster `c2`
//! speeds up every compile-in-the-loop workflow (search moves, preimage
//! checks, pass@k) at once, and that is real and worth measuring — but the
//! project's goal, decided by the owner on 2026-08-21
//! (`docs/GOAL_DECISION_2026-08-21.md`), is **perfect reproduction**, for (1)
//! understanding MSVC's internals in service of decomp and (2) parity — a
//! 100 % open-source implementation. This module's numbers may **neither
//! justify a lane nor forbid one**; `c2rs perf` is reported, never gated
//! (board #3336).
//!
//! **Ranking, added 2026-08-22** (that doc's § "AMENDED", the owner, later the
//! same day): **goal (1) is primary**, and goal (2) is a real end *and*
//! instrumental to (1) — an open port is a tweakable model of c2 that emits
//! signals about compiler state the binary cannot. One of the two named
//! consumers of those signals wants **volume**: generating aligned
//! `(IL, internal state, bytes)` triples as training data for models that
//! reverse the compiler. **That does not re-promote this module's numbers.**
//! The rule above is unchanged and deliberately symmetric — throughput may
//! neither justify a lane nor forbid one — and "a consumer would benefit"
//! is a *justification*, which is exactly the move that is barred. Recorded
//! here so a future reader does not rediscover the consumer and mistake it for
//! a reinstated thesis.
//!
//! What this module measures is the per-obj latency of the two
//! backends that produce the **same** obj from the **same** bundle:
//!
//! * the native port ([`c2_core::PortC2`]) — pure in-process Rust
//!   ([`Backend::compile_to`]: parse IL bundle → PPC select → emit COFF), and
//! * the reference ([`Toolchain::replay`]) — standalone `c2.dll` under wibo
//!   (spawn `wibo c2host c2.dll …`, the real backend on the same bundle).
//!
//! For each fixture we [`capture`](Toolchain::capture_reference) the bundle +
//! pipeline obj once, confirm the port's obj is **byte-exact** to the reference
//! (so we are timing *equivalent* output, not a shortcut), then time `R`
//! iterations of each and report median/mean latency and the speedup. Fixtures
//! outside the ported class time only the reference side and report the port as
//! [`PortPerf::NotImplemented`] — they are excluded from the speedup geomean.
//!
//! Needs the toolchain **and** `strace` (to keep the bundle) **and** mingw (to
//! build `c2host`); the CLI guards on those and skips cleanly when absent.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use c2_core::{Backend, BackendError, PortC2};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{to_wibo_path, CapturedReference, Toolchain};

/// How many timed iterations to run on each side.
#[derive(Clone, Copy, Debug)]
pub struct PerfConfig {
    /// Timed `PortC2::compile_to` iterations (cheap; run many).
    pub port_iters: usize,
    /// Timed standalone-c2 `replay` iterations (each spawns a process; few).
    pub ref_iters: usize,
}

impl Default for PerfConfig {
    fn default() -> Self {
        // The port is microseconds, so a few thousand iterations still finish
        // instantly; each replay is a wibo process (tens of ms), so keep it low.
        PerfConfig {
            port_iters: 2000,
            ref_iters: 5,
        }
    }
}

/// Port-side timing result for one fixture.
#[derive(Clone, Copy, Debug)]
pub enum PortPerf {
    /// Port emitted a byte-exact obj; here is its per-call latency.
    Match { median: Duration, mean: Duration },
    /// The bundle is outside the ported class (port returned `NotImplemented`).
    NotImplemented,
    /// Port produced an obj, but it diverged from the reference at this byte.
    Mismatch { first_offset: usize },
}

/// One fixture's benchmark row.
#[derive(Clone, Debug)]
pub struct FixturePerf {
    pub cpp: PathBuf,
    /// Reference obj size in bytes (the thing both sides produce).
    pub obj_len: usize,
    /// Whether standalone-c2 replay reproduced the pipeline obj byte-exact
    /// (the P0.1 invariant; should always hold on the fixtures).
    pub ref_exact: bool,
    pub ref_median: Duration,
    pub ref_mean: Duration,
    pub port: PortPerf,
}

impl FixturePerf {
    /// Speedup (reference median ÷ port median) when the port matched.
    pub fn speedup(&self) -> Option<f64> {
        match self.port {
            PortPerf::Match { median, .. } if median.as_secs_f64() > 0.0 => {
                Some(self.ref_median.as_secs_f64() / median.as_secs_f64())
            }
            _ => None,
        }
    }
}

/// Full report over a set of fixtures.
#[derive(Clone, Debug)]
pub struct PerfReport {
    pub rows: Vec<FixturePerf>,
    pub port_iters: usize,
    pub ref_iters: usize,
}

impl PerfReport {
    /// Geometric mean of the per-fixture speedups over the fixtures the port
    /// matched (geomean, not arithmetic — speedups are ratios). `None` if none
    /// matched.
    pub fn geomean_speedup(&self) -> Option<f64> {
        let speedups: Vec<f64> = self.rows.iter().filter_map(FixturePerf::speedup).collect();
        geomean(&speedups)
    }

    /// (matched, timed-but-mismatched, not-implemented) fixture counts.
    pub fn tally(&self) -> (usize, usize, usize) {
        let mut m = 0;
        let mut mis = 0;
        let mut ni = 0;
        for r in &self.rows {
            match r.port {
                PortPerf::Match { .. } => m += 1,
                PortPerf::Mismatch { .. } => mis += 1,
                PortPerf::NotImplemented => ni += 1,
            }
        }
        (m, mis, ni)
    }
}

/// Benchmark one fixture. Captures the bundle once, warms the replay path (which
/// also builds `c2host` and checks the P0.1 byte-exactness), then times both
/// sides. Errors only on a capture/replay I/O failure — an out-of-class port is
/// a normal [`PortPerf::NotImplemented`] result, not an error.
pub fn bench_fixture(
    tc: &Toolchain,
    cpp: &Path,
    cfg: &PerfConfig,
    work: &Path,
) -> io::Result<FixturePerf> {
    let ref_iters = cfg.ref_iters.max(1);
    let port_iters = cfg.port_iters.max(1);

    // Capture the pipeline obj + IL bundle + exact c2 argv (one real compile).
    // The fixture's declared profile, or the default (`crate::fixture_profile`).
    let captured = crate::fixture_profile::capture_fixture_reference(tc, cpp, &work.join("cap"))?;
    let obj_name = to_wibo_path(&captured.ref_obj_path);
    let bundle_dir = work.join("il");
    // Replay to the SAME `/Fo` path the reference used: MSVC embeds that path
    // string in the obj (`.debug$S` S_OBJNAME), so a different path would make
    // the replay spuriously "differ". `captured.ref_obj` is already in memory,
    // so overwriting the on-disk obj is harmless. (Same rule as `differential`.)
    let out = captured.ref_obj_path.clone();

    // Warm: one replay builds c2host + primes the FS cache, and doubles as the
    // P0.1 correctness check (replay must reproduce the pipeline obj byte-exact).
    let first = tc.replay(&captured, &bundle_dir, &out)?;
    let ref_exact = matches!(
        ObjImage::diff(&captured.ref_obj, &first),
        ObjDiff::Identical
    );

    // Time the reference side (standalone c2 under wibo).
    let mut ref_ds = Vec::with_capacity(ref_iters);
    for _ in 0..ref_iters {
        let t = Instant::now();
        let _ = tc.replay(&captured, &bundle_dir, &out)?;
        ref_ds.push(t.elapsed());
    }

    // Time the port — but only if it produces a byte-exact obj for this bundle.
    let port = PortC2::default();
    let port_perf = match port.compile_to(&captured.bundle, &obj_name) {
        Ok(o) => match ObjImage::diff(&captured.ref_obj, &o) {
            ObjDiff::Identical => {
                let mut ds = Vec::with_capacity(port_iters);
                for _ in 0..port_iters {
                    let t = Instant::now();
                    let obj = port
                        .compile_to(&captured.bundle, &obj_name)
                        .expect("in-class bundle compiled once; must compile again");
                    // Keep the optimizer from eliding the whole call.
                    std::hint::black_box(obj.len());
                    ds.push(t.elapsed());
                }
                PortPerf::Match {
                    median: median(&ds),
                    mean: mean(&ds),
                }
            }
            ObjDiff::Differs { first_offset, .. } => PortPerf::Mismatch { first_offset },
        },
        Err(BackendError::NotImplemented(_)) => PortPerf::NotImplemented,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("port error: {e}"))),
    };

    Ok(FixturePerf {
        cpp: cpp.to_path_buf(),
        obj_len: captured.ref_obj.len(),
        ref_exact,
        ref_median: median(&ref_ds),
        ref_mean: mean(&ref_ds),
        port: port_perf,
    })
}

// --- small pure stats helpers (unit-tested; no toolchain needed) -------------

/// Median of a duration sample (lower-middle element for an even count). Empty
/// → `Duration::ZERO`.
fn median(ds: &[Duration]) -> Duration {
    if ds.is_empty() {
        return Duration::ZERO;
    }
    let mut v = ds.to_vec();
    v.sort();
    v[v.len() / 2]
}

/// Arithmetic mean of a duration sample. Empty → `Duration::ZERO`.
fn mean(ds: &[Duration]) -> Duration {
    if ds.is_empty() {
        return Duration::ZERO;
    }
    ds.iter().copied().sum::<Duration>() / ds.len() as u32
}

/// Geometric mean of a set of positive ratios. `None` if empty; non-positive
/// entries are ignored.
fn geomean(xs: &[f64]) -> Option<f64> {
    let ln_sum: f64 = xs.iter().filter(|x| **x > 0.0).map(|x| x.ln()).sum();
    let n = xs.iter().filter(|x| **x > 0.0).count();
    if n == 0 {
        None
    } else {
        Some((ln_sum / n as f64).exp())
    }
}

/// Human-friendly duration formatter (ns / µs / ms / s, 3 sig-ish figures).
pub fn fmt_dur(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.2} µs", ns as f64 / 1e3)
    } else if ns < 1_000_000_000 {
        format!("{:.3} ms", ns as f64 / 1e6)
    } else {
        format!("{:.3} s", ns as f64 / 1e9)
    }
}

// --- concurrency scaling: throughput (objs/sec) vs thread count -------------
//
// The angle-H headline is not just per-obj latency but *throughput under load*:
// the port is pure in-process Rust with no shared state, so it scales across
// cores nearly linearly, while standalone c2 pays a `wibo` process spawn per obj
// and saturates far sooner. `perf-scale` measures both at a range of concurrency
// levels so the README graph can show the gap widening with parallelism.

/// Config for a [`scale_measure`] sweep.
#[derive(Clone, Debug)]
pub struct ScaleConfig {
    /// Thread counts to measure at (e.g. `[1, 2, 4, 8, 16, 32]`).
    pub concurrencies: Vec<usize>,
    /// Wall-clock budget per port measurement (cheap; short is enough).
    pub port_secs: f64,
    /// Wall-clock budget per reference measurement (process-heavy; give it more).
    pub ref_secs: f64,
}

impl Default for ScaleConfig {
    fn default() -> Self {
        ScaleConfig {
            concurrencies: vec![1, 2, 4, 8],
            port_secs: 0.5,
            ref_secs: 1.5,
        }
    }
}

/// One concurrency level's throughput, in objects per second.
#[derive(Clone, Copy, Debug)]
pub struct ScalePoint {
    pub concurrency: usize,
    pub port_ops: f64,
    pub ref_ops: f64,
}

impl ScalePoint {
    /// Port throughput ÷ reference throughput at this concurrency.
    pub fn speedup(&self) -> f64 {
        if self.ref_ops > 0.0 {
            self.port_ops / self.ref_ops
        } else {
            f64::INFINITY
        }
    }
}

/// Measure port vs reference throughput across `cfg.concurrencies` on one
/// in-class fixture. Captures once, verifies the port is byte-exact (so we are
/// scaling an *equivalent* emitter, not a shortcut), then times each side under
/// N concurrent threads. Returns the points plus the obj size in bytes.
///
/// Errors if the fixture is outside the ported class (the port must Match for a
/// fair scaling comparison — pick e.g. `mvp_add3.cpp`).
pub fn scale_measure(
    tc: &Toolchain,
    cpp: &Path,
    cfg: &ScaleConfig,
    work: &Path,
) -> io::Result<(Vec<ScalePoint>, usize)> {
    // The fixture's declared profile, or the default (`crate::fixture_profile`).
    let captured = crate::fixture_profile::capture_fixture_reference(tc, cpp, &work.join("cap"))?;
    let obj_name = to_wibo_path(&captured.ref_obj_path);

    // Warm c2host + confirm the P0.1 replay is byte-exact for this fixture.
    let warm_out = captured.ref_obj_path.clone();
    let replayed = tc.replay(&captured, &work.join("warm_il"), &warm_out)?;
    if !matches!(ObjImage::diff(&captured.ref_obj, &replayed), ObjDiff::Identical) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("standalone-c2 replay is not byte-exact for {}", cpp.display()),
        ));
    }
    // The port must be byte-exact on this fixture for the comparison to be fair.
    let port = PortC2::default();
    match port.compile_to(&captured.bundle, &obj_name) {
        Ok(o) if matches!(ObjImage::diff(&captured.ref_obj, &o), ObjDiff::Identical) => {}
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "port obj is not byte-exact for {} — pick an in-class fixture (e.g. mvp_add3.cpp)",
                    cpp.display()
                ),
            ))
        }
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "port cannot compile {} ({e}) — pick an in-class fixture (e.g. mvp_add3.cpp)",
                    cpp.display()
                ),
            ))
        }
    }

    let cap = Arc::new(captured);
    let name = Arc::new(obj_name);
    let tc_arc = Arc::new(tc.clone());

    let mut points = Vec::with_capacity(cfg.concurrencies.len());
    for &c in &cfg.concurrencies {
        let c = c.max(1);
        let port_ops = measure_port(&cap, &name, c, cfg.port_secs);
        let ref_ops = measure_ref(&tc_arc, &cap, c, cfg.ref_secs, work)?;
        points.push(ScalePoint {
            concurrency: c,
            port_ops,
            ref_ops,
        });
    }
    Ok((points, cap.ref_obj.len()))
}

/// Port throughput (objs/sec) with `concurrency` threads each compiling the
/// bundle in a tight loop until the time budget expires.
fn measure_port(
    cap: &Arc<CapturedReference>,
    obj_name: &Arc<String>,
    concurrency: usize,
    secs: f64,
) -> f64 {
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(secs);
    let handles: Vec<_> = (0..concurrency)
        .map(|_| {
            let cap = Arc::clone(cap);
            let obj_name = Arc::clone(obj_name);
            std::thread::spawn(move || {
                let port = PortC2::default();
                let mut n = 0u64;
                while Instant::now() < deadline {
                    let o = port
                        .compile_to(&cap.bundle, &obj_name)
                        .expect("in-class bundle compiles");
                    std::hint::black_box(o.len());
                    n += 1;
                }
                n
            })
        })
        .collect();
    let total: u64 = handles.into_iter().map(|h| h.join().unwrap_or(0)).sum();
    total as f64 / start.elapsed().as_secs_f64()
}

/// Reference throughput (objs/sec): `concurrency` threads each replaying the
/// bundle through standalone c2 (its own scratch dirs) until the budget expires.
fn measure_ref(
    tc: &Arc<Toolchain>,
    cap: &Arc<CapturedReference>,
    concurrency: usize,
    secs: f64,
    work: &Path,
) -> io::Result<f64> {
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(secs);
    let handles: Vec<_> = (0..concurrency)
        .map(|i| {
            let cap = Arc::clone(cap);
            let tc = Arc::clone(tc);
            // Per-thread scratch so concurrent replays never share a bundle/obj.
            let bundle_dir = work.join(format!("scale_il_c{concurrency}_{i}"));
            let out = work.join(format!("scale_out_c{concurrency}_{i}.obj"));
            std::thread::spawn(move || {
                let mut n = 0u64;
                while Instant::now() < deadline {
                    if tc.replay(&cap, &bundle_dir, &out).is_err() {
                        break;
                    }
                    n += 1;
                }
                n
            })
        })
        .collect();
    let total: u64 = handles.into_iter().map(|h| h.join().unwrap_or(0)).sum();
    Ok(total as f64 / start.elapsed().as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalepoint_speedup() {
        let p = ScalePoint {
            concurrency: 4,
            port_ops: 40_000.0,
            ref_ops: 200.0,
        };
        assert!((p.speedup() - 200.0).abs() < 1e-6);
        let zero = ScalePoint {
            concurrency: 1,
            port_ops: 1.0,
            ref_ops: 0.0,
        };
        assert!(zero.speedup().is_infinite());
    }

    #[test]
    fn median_picks_middle() {
        let ds = [
            Duration::from_millis(5),
            Duration::from_millis(1),
            Duration::from_millis(3),
        ];
        assert_eq!(median(&ds), Duration::from_millis(3));
        assert_eq!(median(&[]), Duration::ZERO);
    }

    #[test]
    fn mean_averages() {
        let ds = [Duration::from_millis(2), Duration::from_millis(4)];
        assert_eq!(mean(&ds), Duration::from_millis(3));
        assert_eq!(mean(&[]), Duration::ZERO);
    }

    #[test]
    fn geomean_of_ratios() {
        // geomean(1, 100) = 10, not the arithmetic 50.5.
        let g = geomean(&[1.0, 100.0]).unwrap();
        assert!((g - 10.0).abs() < 1e-9, "got {g}");
        assert!(geomean(&[]).is_none());
        // Non-positive entries are ignored.
        assert!((geomean(&[0.0, 4.0, 4.0]).unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn fmt_dur_scales() {
        assert_eq!(fmt_dur(Duration::from_nanos(500)), "500 ns");
        assert_eq!(fmt_dur(Duration::from_nanos(2_100)), "2.10 µs");
        assert_eq!(fmt_dur(Duration::from_micros(1_500)), "1.500 ms");
        assert_eq!(fmt_dur(Duration::from_millis(2_500)), "2.500 s");
    }

    #[test]
    fn speedup_only_when_matched() {
        let base = FixturePerf {
            cpp: PathBuf::from("x.cpp"),
            obj_len: 100,
            ref_exact: true,
            ref_median: Duration::from_millis(40),
            ref_mean: Duration::from_millis(40),
            port: PortPerf::Match {
                median: Duration::from_micros(2),
                mean: Duration::from_micros(2),
            },
        };
        assert!((base.speedup().unwrap() - 20_000.0).abs() < 1.0);

        let ni = FixturePerf {
            port: PortPerf::NotImplemented,
            ..base.clone()
        };
        assert!(ni.speedup().is_none());
    }
}
