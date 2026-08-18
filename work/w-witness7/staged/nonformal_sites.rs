//! **`call-arg-nonformal` has FIVE raise sites, and this is one witness per
//! site** — lane `w-witness7`, rung `docs/rungs/2026-08-18-witness7.md`.
//!
//! # The item
//!
//! `w-deadsites` (board **#3276**) partitioned `w-mutcensus`' 26 open GREEN
//! census rows and found **7** of them UNGUARDED — sites its probe proved an
//! input reaches, where nobody had written a witness. `CA6`
//! (`calls.rs:693`, the argument-slot path's `call-arg-nonformal`) and `CA8`
//! (`calls.rs:710`, `call-arg-computed`) are two of the seven. This file is
//! their witness, and it takes the form `w-calleeguard`'s **P13** asked for and
//! `w-deadsites` closed on a `k = 2` family: **one row per RAISE SITE**, at the
//! largest `k` in the crate.
//!
//! # Why a per-site table rather than one cell per key
//!
//! `w-mutcensus` **F2**: *a key with `k` raise sites contributes `k − 1`
//! unguarded sites by construction* — one cell proves the key can be produced
//! and says nothing about which of the `k` sites produced it. Parsing
//! `crates/c2-il/src` for `refuse("call-arg-nonformal")` and
//! `Block::refuse(…, "call-arg-nonformal")` gives **five** raise sites, the
//! largest `k` of any refuse-literal key in the crate:
//!
//! | row | site | the production it is inside | the control's in-class label |
//! |---|---|---|---|
//! | 1 | `calls.rs:693` | `tail_call_shape`, the **slot** path (≥ 2 arguments) — `w-mutcensus`' `CA6` | `multiarg-tail-call` |
//! | 2 | `calls.rs:807` | `tail_call_shape`, the **single-argument operand** path | `int-tail-call` |
//! | 3 | `calls.rs:1749` | the **framed** post-op path of a FREE function (`g(x) + k`) | `framed-call` |
//! | 4 | `mcall_cmp.rs:246` | the member-call **comparison** production's receivers | `call-sequence-cmp-eq` |
//! | 5 | `mcall_tail.rs:673` | the member-call **framed** production's receiver | `framed-call` |
//!
//! # The site → cell map is MEASURED, not read
//!
//! Rows 3 and 5 have the same control label (`framed-call`) and the same key,
//! so reading the source cannot say which of the two productions a member-call
//! cell routes through. It was settled by experiment instead: all five raise
//! sites were rekeyed **to five distinct sentinels in one patch**
//! (`work/w-witness7/patch.py`, ids `SID1`–`SID5`) and each cell below was
//! censused against that tree. The result is one-to-one —
//! `work/w-witness7/logs/SID.census.log`:
//!
//! ```text
//!   s1 -> wit7-site1:eof     s2 -> wit7-site2:eof     s3 -> wit7-site3:eof
//!   s4 -> wit7-site4:eof     s5 -> wit7-site5:eof
//! ```
//!
//! **Each cell reaches exactly one raise site and no cell reaches two.** That
//! is the property that makes each row below a *site* witness rather than a
//! fifth copy of a key witness, and it is a measurement rather than an argument
//! about the source.
//!
//! # Every assertion is on the PUBLISHED KEY STRING
//!
//! `w-guards`' rule: a guard on the constant passes a mutation that renames the
//! constant and its uses while the published key moves. These keys are string
//! literals at the raise site rather than named constants, so the rule takes its
//! other form here — **assert what `FnVerdict::key()` emits**, never
//! `Block::ctx` and never a substring.
//!
//! # What this file cannot see, stated because a table is a kind of count
//!
//! It covers the five raise sites that existed when it was written. **A SIXTH
//! raise site of `call-arg-nonformal` adds no row and turns nothing red here.**
//! `tests/fence_site_census.rs` is the standing count that would — and its
//! `EXPECTED_REFUSE_SITES` is a single integer over the whole `refuse("…")`
//! population, so it sees a site *added* and is blind to one *retargeted* from
//! this key to another. The two together still cannot see a sixth site that
//! replaces a fifth.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use — the flags the 878-TU scan is taken at.
///
/// **Load-bearing.** At the `census` CLI's default `/Ox /GS- /c` the member-call
/// cells (rows 4 and 5) do not reach their productions at all, and at `/Od`
/// every cell here reports `opt-mode-00800005` instead of its own key — which is
/// the neighbouring gate this lane also guards
/// (`tests/census_key_routing.rs`).
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// The one key all five raise sites publish. Spelled once: a test that re-types
/// the string its subject produces is a test of two spellings.
const NONFORMAL: &str = "call-arg-nonformal:eof";

/// `CA8`'s key — the *other* refusal of the same argument-slot loop, and the
/// one `w-mutcensus`' `M-CA6`/`M-CA8` mutations swap with it.
const COMPUTED: &str = "call-arg-computed:eof";

/// A scratch directory keyed on the tag **and** the pid — board #1045 (four
/// parallel tests sharing one PID-keyed directory raced their captures and
/// fabricated a finding) and `w-gateperf`'s `reloc_identity` race, which
/// presented as a port defect for an hour.
fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-wit7nf-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Capture one source and return the census keys in `.ex` order.
///
/// Keyed by position rather than by name: the census reports a name only when
/// the `.gl` name count equals the segment count, and a name-keyed lookup would
/// silently skip a cell that does not pair.
fn keys(tc: &Toolchain, tag: &str, body: &str) -> Vec<String> {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, body).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc
        .capture_reference_with(&src, &dir, &flags, None)
        .unwrap_or_else(|e| panic!("cell `{tag}`: capture failed: {e}"));
    let census = cap.bundle.function_census().unwrap_or_else(|| {
        panic!(
            "cell `{tag}`: no census at all — a cell that stopped producing rows \
             grades nothing and must fail loudly rather than pass"
        )
    });
    assert!(
        !census.is_empty(),
        "cell `{tag}`: the census is EMPTY. Every assertion below would be \
         vacuous (`docs/STATUS.md` trap 5)"
    );
    census.iter().map(|c| c.verdict.key()).collect()
}

/// How many of this TU's rows carry `key`. Counted, so a cell that grew a body
/// cannot silently satisfy a `contains`.
fn n(ks: &[String], key: &str) -> usize {
    ks.iter().filter(|k| k.as_str() == key).count()
}

// ---------------------------------------------------------------------------
// The cells. Each witness differs from its control in **one** fact: whether the
// value in the argument (or receiver) position is one of this function's own
// formals or a file-scope global. Everything else is byte-identical source.
// ---------------------------------------------------------------------------

/// ROW 1 — `calls.rs:693`. Two arguments, so `tail_call_shape` takes the SLOT
/// path; slot 0 is `[Load(gi)]` and `gi` is not in `params`.
const S1: &str = "int gi;\nvoid g2(int, int);\nvoid f(int a) { g2(gi, a); }\n";
const S1_CTL: &str = "void g2(int, int);\nvoid f(int a, int b) { g2(a, b); }\n";

/// ROW 2 — `calls.rs:807`. ONE argument, so the slot path is skipped entirely
/// and the single-argument operand path asks `arg_loads_are_formals`.
const S2: &str = "int gi;\nint g1(int);\nint f(int a) { return g1(gi); }\n";
const S2_CTL: &str = "int g1(int);\nint f(int a) { return g1(a); }\n";

/// ROW 3 — `calls.rs:1749`. A non-zero post-op makes it a FRAMED call, whose
/// passthrough argument must still be a formal.
const S3: &str = "int gi;\nint g1(int);\nint f(int a) { return g1(gi) + 1; }\n";
const S3_CTL: &str = "int g1(int);\nint f(int a) { return g1(a) + 1; }\n";

/// ROW 4 — `mcall_cmp.rs:246`. Two nullary member calls compared; the SECOND
/// receiver is a global rather than a formal.
const S4: &str = "struct S { int m(); };\nS* gp;\nbool f(S* p) { return p->m() == gp->m(); }\n";
const S4_CTL: &str = "struct S { int m(); };\nbool f(S* p, S* q) { return p->m() == q->m(); }\n";

/// ROW 5 — `mcall_tail.rs:673`. A framed MEMBER call whose receiver is a global.
const S5: &str = "struct S { int m(); };\nS* gp;\nint f(S* p) { return gp->m() + 1; }\n";
const S5_CTL: &str = "struct S { int m(); };\nint f(S* p) { return p->m() + 1; }\n";

/// **The witness table — one row per raise site of `call-arg-nonformal`.**
///
/// Each row asserts three things: the witness carries the published key, the
/// control does **not**, and the control is in class under the label that names
/// **which production it went through**. That third assertion is what pins the
/// row to its site: rows 1, 2, 3 and 4 have four different control labels, so a
/// cell cannot be silently rerouted into a neighbouring production without the
/// control saying so.
///
/// Rows 3 and 5 share the label `framed-call`, which is exactly why the
/// site → cell map was established by the `SID1`–`SID5` sentinel patch (this
/// file's header) rather than by reading. Under that patch row 3's cell reports
/// `wit7-site3` and row 5's reports `wit7-site5`, so the two productions are
/// distinguished by measurement even though their in-class labels are not.
///
/// **What each mutation does to this test**, and it is one row each:
///
/// * `M-CA6` — `calls.rs:693`'s key `call-arg-nonformal` → `call-arg-computed`
///   (`w-mutcensus`' own registered `CA6` mutation, measured GREEN over 1,648
///   tests): **ROW 1** fails and rows 2–5 still pass.
/// * a mutation at `calls.rs:807`, `:1749`, `mcall_cmp.rs:246` or
///   `mcall_tail.rs:673`: **that row alone** fails.
#[test]
fn each_of_the_five_raise_sites_of_call_arg_nonformal_has_its_own_witness() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let s1 = keys(&tc, "s1", S1);
    let s1c = keys(&tc, "s1c", S1_CTL);
    let s2 = keys(&tc, "s2", S2);
    let s2c = keys(&tc, "s2c", S2_CTL);
    let s3 = keys(&tc, "s3", S3);
    let s3c = keys(&tc, "s3c", S3_CTL);
    let s4 = keys(&tc, "s4", S4);
    let s4c = keys(&tc, "s4c", S4_CTL);
    let s5 = keys(&tc, "s5", S5);
    let s5c = keys(&tc, "s5c", S5_CTL);

    // ---- ROW 1 — calls.rs:693, the argument-SLOT path (`w-mutcensus`' CA6) --
    assert_eq!(
        n(&s1, NONFORMAL), 1,
        "ROW 1 — `calls.rs:693`. A TWO-argument tail call whose slot 0 is a load \
         of a file-scope global must take `{NONFORMAL}`: `tail_call_shape` puts \
         2+ arguments on the SLOT path, and the `[IlOp::Load(t)]` arm asks \
         `params.iter().position(|q| q == t)`, which is `None` for `gi`. This is \
         `w-deadsites`' UNGUARDED row `CA6`. Keys were {s1:?}"
    );
    assert_eq!(
        n(&s1c, "multiarg-tail-call"), 1,
        "ROW 1's CONTROL — the identical call with both arguments FORMAL must be \
         in class as `multiarg-tail-call`, which is the label that says the cell \
         went through the slot path at all. Without it the row above is \
         satisfied by any refusal anywhere. Keys were {s1c:?}"
    );
    assert_eq!(
        n(&s1c, NONFORMAL), 0,
        "ROW 1's CONTROL must not carry the key. Keys were {s1c:?}"
    );

    // ---- ROW 2 — calls.rs:807, the single-argument OPERAND path -------------
    assert_eq!(
        n(&s2, NONFORMAL), 1,
        "ROW 2 — `calls.rs:807`. ONE argument, so the slot path is skipped \
         entirely (`args.len() > 1 || matches!(args[0], [SymAddr])` is false) \
         and the operand path's `arg_loads_are_formals` raises the same key from \
         a different site. Keys were {s2:?}"
    );
    assert_eq!(
        n(&s2c, "int-tail-call"), 1,
        "ROW 2's CONTROL — the identical single-argument call with the FORMAL in \
         place is in class as `int-tail-call`, a label rows 1, 3 and 4 never \
         produce. That is what rules the other four sites out for this row. \
         Keys were {s2c:?}"
    );
    assert_eq!(
        n(&s2c, NONFORMAL), 0,
        "ROW 2's CONTROL must not carry the key. Keys were {s2c:?}"
    );

    // ---- ROW 3 — calls.rs:1749, the FRAMED free-function post-op ------------
    assert_eq!(
        n(&s3, NONFORMAL), 1,
        "ROW 3 — `calls.rs:1749`. A non-zero post-op routes the body to the \
         6-section FRAMED production, whose bare passthrough argument must still \
         be a formal (`Block::refuse(seg, *p, \"call-arg-nonformal\")` — the only \
         one of the five raised through `Block::refuse` rather than the local \
         `refuse` closure). Measured to be this site and not `:807` by the \
         `SID3` sentinel. Keys were {s3:?}"
    );
    assert_eq!(
        n(&s3c, "framed-call"), 1,
        "ROW 3's CONTROL — the same body with the formal in place is in class as \
         `framed-call`, which is what says the cell reached the framed \
         production rather than the tail one. Keys were {s3c:?}"
    );
    assert_eq!(
        n(&s3c, NONFORMAL), 0,
        "ROW 3's CONTROL must not carry the key. Keys were {s3c:?}"
    );

    // ---- ROW 4 — mcall_cmp.rs:246, the member-call COMPARISON receivers -----
    assert_eq!(
        n(&s4, NONFORMAL), 1,
        "ROW 4 — `mcall_cmp.rs:246`. Two nullary member calls compared; the \
         emission is a register MOVE of each receiver, so both receivers must be \
         this function's own formals and a global is a load. This site is in a \
         different FILE from rows 1–3 and nothing in `calls.rs` can satisfy it. \
         Keys were {s4:?}"
    );
    assert_eq!(
        n(&s4c, "call-sequence-cmp-eq"), 1,
        "ROW 4's CONTROL — both receivers formal is in class as \
         `call-sequence-cmp-eq`, a label no other row produces. Keys were {s4c:?}"
    );
    assert_eq!(
        n(&s4c, NONFORMAL), 0,
        "ROW 4's CONTROL must not carry the key. Keys were {s4c:?}"
    );

    // ---- ROW 5 — mcall_tail.rs:673, the framed MEMBER call's receiver -------
    assert_eq!(
        n(&s5, NONFORMAL), 1,
        "ROW 5 — `mcall_tail.rs:673`. The member-call framed production's own \
         receiver check. Its in-class label (`framed-call`) is the SAME as row \
         3's, which is why the site → cell map here is a measurement: under the \
         `SID1`–`SID5` sentinel patch this cell reports `wit7-site5` and row 3's \
         reports `wit7-site3` \
         (`work/w-witness7/logs/SID.census.log`). Keys were {s5:?}"
    );
    assert_eq!(
        n(&s5c, "framed-call"), 1,
        "ROW 5's CONTROL — the framed member call on a FORMAL receiver is in \
         class. Keys were {s5c:?}"
    );
    assert_eq!(
        n(&s5c, NONFORMAL), 0,
        "ROW 5's CONTROL must not carry the key. Keys were {s5c:?}"
    );

    // ---- the table is a DISCRIMINATION, asserted as a count -----------------
    //
    // Ten cells, five carrying the key and five in class. A collapse — every
    // cell refusing, or every cell in class — satisfies no row above, and this
    // says so in one line rather than leaving it to be inferred from fifteen
    // assertions.
    let witnesses = [&s1, &s2, &s3, &s4, &s5]
        .iter()
        .filter(|k| n(k, NONFORMAL) == 1)
        .count();
    let controls = [&s1c, &s2c, &s3c, &s4c, &s5c]
        .iter()
        .filter(|k| n(k, NONFORMAL) == 0)
        .count();
    assert_eq!(
        (witnesses, controls), (5, 5),
        "the ten-cell table must split 5 witnesses / 5 controls — got \
         {witnesses} witnesses, {controls} controls. One witness per raise site \
         of `call-arg-nonformal`, and one control per witness that rules the \
         other four sites out"
    );
}

/// **`CA8` — `calls.rs:710`, and the pair that makes it a witness rather than a
/// restatement of row 1.**
///
/// `w-mutcensus`' registered mutations for these two sites are each other:
/// `CA6` retargets `calls.rs:693` from `call-arg-nonformal` to
/// `call-arg-computed`, and `CA8` retargets `calls.rs:710` the other way. Both
/// were measured **GREEN** over 1,648 tests — nothing in the suite could tell
/// the two refusals of one loop apart.
///
/// The two cells differ in one character of source: `g2(gi, a)` against
/// `g2(a + 1, b)`. Both are two-argument tail calls, both take the slot path,
/// and they part company on which arm of the `match ops.as_slice()` they land
/// in — `[IlOp::Load(t)]` with a non-formal token, against the `_` catch-all for
/// an operand stream that has to be computed into a register.
///
/// Asserting the two keys **and their difference** is what catches either
/// retarget: after `M-CA6` both cells report `call-arg-computed`, after `M-CA8`
/// both report `call-arg-nonformal`, and the `assert_ne!` fires in both
/// directions even if a future reader deletes one of the two positive rows.
#[test]
fn the_slot_paths_two_refusals_are_distinct_keys_and_neither_answers_for_the_other() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    let nonformal = keys(&tc, "ca6", S1);
    let computed = keys(
        &tc,
        "ca8",
        "void g2(int, int);\nvoid f(int a, int b) { g2(a + 1, b); }\n",
    );

    assert_eq!(
        n(&nonformal, NONFORMAL), 1,
        "`CA6` (`calls.rs:693`) — a non-formal LOAD in an argument slot is \
         `{NONFORMAL}`. Keys were {nonformal:?}"
    );
    assert_eq!(
        n(&computed, COMPUTED), 1,
        "`CA8` (`calls.rs:710`) — a COMPUTED operand stream in an argument slot \
         is `{COMPUTED}`, the `_` arm of the same `match`. It is a different \
         population from the row above and a different rung would pay for it: \
         `wla_lit_call_arg.cpp` records `call-arg-computed` at **5,537** \
         functions on the 878-TU workload, the largest single argument-shape \
         row. Keys were {computed:?}"
    );
    assert_eq!(
        n(&nonformal, COMPUTED), 0,
        "the non-formal cell must not report the computed key — that is \
         `w-mutcensus`' `CA6` mutation exactly, and it was GREEN over 1,648 \
         tests. Keys were {nonformal:?}"
    );
    assert_eq!(
        n(&computed, NONFORMAL), 0,
        "the computed cell must not report the non-formal key — that is \
         `w-mutcensus`' `CA8` mutation exactly. Keys were {computed:?}"
    );
    assert_ne!(
        nonformal, computed,
        "the pair must DISCRIMINATE: two two-argument tail calls that differ \
         only in one argument's op stream must not report identical key lists, \
         or the slot loop is answering the same way regardless of which arm it \
         took"
    );
}
