//! Integration test for the harness. Toolchain-guarded: skips cleanly (never
//! fails) when `Toolchain::locate()` is `None`.
//!
//! Asserts:
//!   * `oracle_selftest` PASSES on the bundled fixtures (determinism + capture
//!     stability against the real toolchain);
//!   * the full `differential` reports the reference replay is **byte-exact**
//!     (P0.1 proven) AND the port is still `NotImplemented` (open T-E gate).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::{differential, oracle_selftest, DiffReport, PortStatus, SelfTestOutcome};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
        .join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-harness-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn oracle_selftest_passes_on_fixtures() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    for name in ["add3.cpp", "il_bool_materialization.cpp", "il_call_return.cpp"] {
        let w = work("selftest");
        let report = oracle_selftest(&fixture(name), &tc, &w);
        assert!(
            report.passed(),
            "oracle self-test did not pass for {name}: {:?}",
            report.outcome
        );
        // Sanity: a real capture has a non-empty .ex and a non-empty obj.
        if let SelfTestOutcome::Pass { obj_len, ex_len } = report.outcome {
            assert!(obj_len > 0 && ex_len > 0);
        }
        std::fs::remove_dir_all(&w).ok();
    }
}

#[test]
fn differential_reference_byte_exact_port_not_implemented() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    let w = work("diff");
    let port = PortC2::default();
    let report = differential(&fixture("add3.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert!(
                matches!(port, PortStatus::NotImplemented(_)),
                "expected the port to still be NotImplemented, got {port:?}"
            );
        }
        other => panic!(
            "expected ReferenceReplayByteExact (P0.1 proven) with PortNotImplemented, got {other:?}"
        ),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// MVP milestone: the native port emits a **byte-exact** `.obj` for the single
/// straight-line int add-chain function `int add3(int,int,int)`. The harness
/// threads the reference's exact `-Fo` path into the port (S_OBJNAME wiring),
/// so the whole obj — header, 5 sections, symbol + string tables — matches on
/// timestamp-normalized bytes.
#[test]
fn differential_mvp_add3_port_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    let w = work("mvp");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_add3.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_add3, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// Multi-function widening: a TU of two straight-line int functions
/// (`add2`, `add4`) is byte-exact. Exercises the multi-`.text`-symbol COFF path
/// — cumulative `Value` offsets, contiguous packing, `NumberOfSymbols = 13+N`.
#[test]
fn differential_mvp_two_multifunction_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvptwo");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_two.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_two, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W3b (wide immediates): `mvp_wide.cpp` covers constants beyond a signed
/// 16-bit field — `a+70000`/`a-70000` (`addis`+`addi`, sign-compensated) and a
/// bare wide constant (`lis`+`ori`).
#[test]
fn differential_mvp_wide_immediates_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpwide");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_wide.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_wide, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// **#35 step 2, rung 1 — Class A many-calls.** A framed body with more than one
/// call and nothing live across any of them: `n` REL24 sites in one function, the
/// callee externals in reverse first-reference order, one symbol per distinct
/// callee however many sites reference it, and the same 96-byte frame, `.pdata`
/// record and label stride the single framed call already had.
///
/// `mvp_call_seq.cpp` is nine framed functions in one TU, which is also the
/// hardest case for the label counter (`3 × 9` up front, then `+5` each under
/// `/Gy`); `mvp_call_twice.cpp` is the two-site / one-symbol minimum.
#[test]
fn differential_class_a_many_calls_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    for name in ["mvp_call_seq.cpp", "mvp_call_twice.cpp"] {
        let w = work("callseq");
        let port = PortC2::default();
        let report = differential(&fixture(name), &tc, &port, &w);
        match report {
            DiffReport::ReferenceReplayByteExact { port, .. } => {
                assert_eq!(
                    port,
                    PortStatus::Match,
                    "expected the port to be byte-exact on {name}, got {port:?}"
                );
            }
            other => panic!("expected ReferenceReplayByteExact for {name}, got {other:?}"),
        }
        std::fs::remove_dir_all(&w).ok();
    }
}

/// W4a: first relocation + external symbol. `mvp_call.cpp` is a single-function
/// tail call (`void f(){g();}`) → `b g` with an IMAGE_REL_PPC_REL24 relocation
/// to g's undefined external symbol. Proves the relocation records, the
/// section-header/aux reloc counts, and the undefined-external symbol layout.
#[test]
fn differential_mvp_call_tailcall_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpcall");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_call.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_call, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W4b2: framed non-leaf call. `mvp_framed.cpp` is `int f(int a){ return g(a)
/// + 1; }` — the call result is used, so `f` gets a 96-byte frame, a `.pdata`
/// unwind section (6 sections, 20 symbols), an ADDR32 relocation, and the
/// compiler label symbols $M2545/$M2546/$T2547. Byte-exact here proves the
/// framed prologue/epilogue, the `bl` REL24, the packed unwind word, the
/// interleaved raw+reloc file layout, and the extended symbol table.
#[test]
fn differential_mvp_framed_call_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpframed");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_framed.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_framed, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W4b2 (integer passthrough tail call): `mvp_tailret.cpp` is `int f(int a){
/// return g(a); }` — the argument is already in r3, so the reference emits a
/// bare 5-section leaf `b g` (REL24 at .text+0x0), the int analog of the void
/// tail call. Byte-exact here proves the integer tail-call lowering shares the
/// void tail call's obj layout (5 sections, 15 symbols, callee sym 14).
#[test]
fn differential_mvp_tailret_int_passthrough_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvptailret");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_tailret.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_tailret, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W4b2-vi (identity-fold tail call): `mvp_plus0.cpp` is `return g(a) + 0`. The
/// `+0` reaches the IL as a real post-op literal, but the optimizer folds it —
/// the reference obj is byte-identical to `return g(a)` (5-section leaf `b g`),
/// NOT a framed obj. Byte-exact here proves the parser routes a net-identity
/// post-op to the tail-call path, closing the W4b2-vi mis-emit leak (the old
/// framed parser would have built a `FramedCall{add_k:0}` → a spurious frame).
#[test]
fn differential_mvp_plus0_identity_fold_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpplus0");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_plus0.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_plus0, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W4b2-iv (arg-setup tail call): `mvp_argtail.cpp` is `return g(a + 1)` — the
/// `+1` is computed INTO the argument (before the `55` call-end), not a framed
/// post-op. The reference emits a 5-section leaf `addi r3,r3,1 ; b g` (REL24 at
/// .text+0x4). Byte-exact here proves the arg-setup prefix + the branch's reloc
/// offset (0x4, not 0x0). Distinct from framed `g(a)+1` (6-section .pdata obj).
#[test]
fn differential_mvp_argtail_arg_setup_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpargtail");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_argtail.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_argtail, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W4b2-i/-v (honest rejection): out-of-class call shapes the port must REFUSE,
/// not mis-emit. Each `.cpp` compiles fine under the reference (byte-exact
/// replay), but the native port has no model for its surrounding computation
/// (non-commutative / strength-reduced / wide post-ops, a second call, a second
/// statement, a two-literal post-op, or arg-setup combined with a framed post-op
/// like `g(a+1)+1`) so it must return `NotImplemented` — never a mis-emitted
/// framed obj or a bare `b g` that silently drops the computation. (The bare
/// arg-setup tail calls `g(a)`, `g(a)+0`, `g(a+1)` ARE now modeled — see the
/// int-tail-call Match tests below.) W4b2-v replaced the neighborhood-scanning
/// gates with a single positive whole-body parse (`c2_il::func::parse_segment`),
/// which accepts only the three modeled shapes and reaches the segment end, so
/// every shape below is rejected at the parser level. Same assertion shape as
/// `differential_reference_byte_exact_port_not_implemented`.
#[test]
fn differential_out_of_class_call_shapes_not_implemented() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    for name in [
        "mvp_call_submod.cpp",    // return g(a) - 1 — non-commutative post-op
        "mvp_call_mulmod.cpp",    // return g(a) * 5 — strength-reduced post-op
        "mvp_call_widemod.cpp",   // return g(a) + 70000 — wide post-op immediate
        // `mvp_call_twice.cpp` (`g(); g();`) used to be here — it is the Class A
        // many-call shape now and is graded as a Match below.
        "mvp_call_then_stmt.cpp", // g(); return a+1; — `a` is read AFTER the call,
                                  // so it must survive one: Class B, not Class A
        "mvp_call_argframed_plusk.cpp", // g(a + 1) + 1 — in-arg arith + framed op
        "mvp_call_two_framed.cpp",      // g(a) + g(a + 1) — a second call in +
        "mvp_call_plus1plus2.cpp",      // g(a) + 1 + 2 — a two-literal post-op
        // A framed call in a >8-formal function: the argument setup is a load
        // from the stack home (`lwz r3,180(r1)`), not a register move, and the
        // slot displacement is a function of the whole list's ABI footprint.
        "wfr_argreg_neg.cpp",
        // A framed function beside a comparison leaf whose label stride is 3:
        // its `$M`/`$T` numbers would come out low by 2 per neighbour, in an obj
        // that still links. The positive half (stride 1) is `wfr_cmp_stride.cpp`
        // and is graded by the mode lanes.
        "wfr_cmp_stride_neg.cpp",
        // The call-bound-to-a-local form's two drifted gates: `int z = g(b + a);
        // return z;` was a live wrong-bytes emit (c2 canonicalizes a commutative
        // argument's leaves, the port kept source order) and
        // `int z = g2(a, c); return z;` **panicked** the census. Both now refuse
        // through the single `tail_call_shape` locator.
        "il_call_bound_neg.cpp",
        // The Class A many-call neighbours: a value read after the first call
        // (Class B, one saved GPR) and a multi-argument literal list.
        "mvp_call_seq_neg.cpp",
        // W30's neighbours: a call-tail literal whose type is NOT a width-4
        // integer (`bool`, `char`, `short`, `wchar_t`, `__int64`, `float`,
        // `double`, a pointer) and one that is but whose value does not fit the
        // `li`/`addi` signed-16-bit immediate.
        "w30_callseq_tail_intlike_neg.cpp",
        // A multi-argument permutation with a cycle longer than three: c2 hoists
        // a second save into r10 and reorders the writes. Live wrong bytes until
        // the grid was measured — `il_call_multi.cpp`'s `cyc4a`/`cyc4b`.
        "il_call_multi.cpp",
    ] {
        let w = work("oocreject");
        let port = PortC2::default();
        let report = differential(&fixture(name), &tc, &port, &w);
        match report {
            DiffReport::ReferenceReplayByteExact { port, .. } => {
                assert!(
                    matches!(port, PortStatus::NotImplemented(_)),
                    "expected the port to honestly refuse {name} (NotImplemented), got {port:?}"
                );
            }
            other => panic!(
                "expected ReferenceReplayByteExact for {name} (reference compiles fine), got {other:?}"
            ),
        }
        std::fs::remove_dir_all(&w).ok();
    }
}

/// W3: literals / immediates. `mvp_lit.cpp` is a 3-function TU: `a+5` (addi),
/// `a-5` (addi with negated imm), and `return 42` (li = addi rD,r0,k). Proves
/// the operand-stack Reg/Imm model and the constant-folding into `addi`.
#[test]
fn differential_mvp_lit_immediates_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvplit");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_lit.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_lit, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W2: non-commutative int ops. `mvp_sub.cpp` is a 3-function TU mixing `-`
/// (subf, reversed operands), `*` (mullw), and `+`. Byte-exact here proves the
/// subf operand-order mapping AND the 8-byte inter-function `.text` alignment
/// (three 12-byte functions → offsets 0x0/0x10/0x20 with zero-padding between).
#[test]
fn differential_mvp_sub_noncommutative_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("mvpsub");
    let port = PortC2::default();
    let report = differential(&fixture("mvp_sub.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => {
            assert_eq!(
                port,
                PortStatus::Match,
                "expected the port to be byte-exact on mvp_sub, got {port:?}"
            );
        }
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}

/// W-UNW-1: framed functions in a MULTI-function TU, packed (`/Ox`).
///
/// The single-function framed emitter was a third whole-obj emitter with the
/// label names `$M2545/$M2546/$T2547` written out literally; these four
/// fixtures are what replaced it, and each pins something the one-function
/// shape could not express:
///
/// * `wunw_framed_pair` — one `.pdata` with **two** records, two ADDR32
///   relocations, `$T` values 0 and 8, and the framed label stride of 4;
/// * `wunw_leaf_then_framed` — the `bl` displacement rebased onto the packed
///   `.text` (`4BFFFFED`, not `4BFFFFF5`), a live wrong-bytes emit the moment
///   the single-function gate came off;
/// * `wunw_two_leaves_framed` — the counter *accumulates* per function rather
///   than being a per-TU constant one slot away;
/// * `wunw_mixed_order` — the shared `.pdata` section symbol lands inside the
///   FIRST framed function's group, not the last, and each framed function
///   introduces its own callee external inside its own group.
///
/// Every one of the four also censuses `N/N`, so the whole-TU gate cannot be
/// hiding a function whose bytes are never compared.
#[test]
fn differential_wunw_multi_function_framed_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    for name in [
        "wunw_framed_pair.cpp",
        "wunw_leaf_then_framed.cpp",
        "wunw_two_leaves_framed.cpp",
        "wunw_mixed_order.cpp",
    ] {
        let w = work(&format!("wunw_{}", name.trim_end_matches(".cpp")));
        let port = PortC2::default();
        let report = differential(&fixture(name), &tc, &port, &w);
        match report {
            DiffReport::ReferenceReplayByteExact { port, .. } => {
                assert_eq!(port, PortStatus::Match, "port not byte-exact on {name}");
            }
            other => panic!("expected ReferenceReplayByteExact for {name}, got {other:?}"),
        }
        std::fs::remove_dir_all(&w).ok();
    }
}

/// W-UNW-1 (fail closed): a floating-point leaf sharing a TU with a framed
/// function must be **refused**, not emitted.
///
/// Both functions are in class on their own and `c2rs census` grades the TU
/// 2/2, so nothing upstream of the emitter objects. The obstacle is the
/// compiler label counter: an FP leaf consumes 2 counter slots against the 1
/// every emitted class consumes, so every `$M`/`$T` number after it would be
/// one low — an obj that links, differs in six bytes, and would have been
/// invisible to a corpus with no framed multi-function TU in it.
#[test]
fn differential_wunw_float_beside_framed_refuses() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        eprintln!("SKIP: strace/mingw absent");
        return;
    }
    let w = work("wunwfloatneg");
    let port = PortC2::default();
    let report = differential(&fixture("wunw_float_neg.cpp"), &tc, &port, &w);
    match report {
        DiffReport::ReferenceReplayByteExact { port, .. } => match port {
            PortStatus::NotImplemented(_) => {}
            other => panic!("expected NotImplemented for wunw_float_neg, got {other:?}"),
        },
        other => panic!("expected ReferenceReplayByteExact, got {other:?}"),
    }
    std::fs::remove_dir_all(&w).ok();
}
