//! **The five census refusal sites `w-deadsites` proved reachable and nobody
//! had written a witness for** — lane `w-witness7`, rung
//! `docs/rungs/2026-08-18-witness7.md`.
//!
//! # The item
//!
//! Board **#3276**: of `w-mutcensus`' 26 open GREEN rows, **7** are UNGUARDED —
//! sites a first-hit probe proved an input reaches, where a mutation was still
//! invisible to all 1,648 tests. `tests/nonformal_sites.rs` guards two of them
//! (`CA6`, `CA8`). This file guards the other five:
//!
//! | row | site | what it decides | the published key |
//! |---|---|---|---|
//! | `CS3` | `census.rs:1288` | which key a `static-scan-loop`-labelled body's refusal is filed under | `static-scan-loop-object-out-of-class:eof` |
//! | `B2` | `bind.rs:974` | `resolve_data_def`'s **COMDAT and initialized** clause — *why* that body is refused | the same key |
//! | `B7` | `bind.rs:1030` | `resolve_bss_def`'s complementary **not-COMDAT and not-initialized** clause | `callee-unresolved-tail-call:eof` |
//! | `CS4` | `census.rs:1306` | whether a `store-run-bind` refusal reports `bind_run_ops`' own reason or the fallback | `store-run-bind-<reason>:eof` |
//! | `CS9` | `census.rs:1323` | the post-parse optimization-mode gate | `opt-mode-<word>` |
//!
//! # Two of these rows share ONE cell, and that is the point of the pair
//!
//! `CS3` is the census `match` arm that names the key; `B2` is one of the
//! reasons `shape_to_function` returned `None` in the first place — a
//! `BodyShape::StaticScanLoop` whose `resolve_data_def(l.array_tok)` is `None`.
//! So a single out-of-class cell binds **both** sites, in opposite directions:
//! retarget the arm and the key moves, disable the object clause and the body
//! goes **in class**. Neither mutation can be absorbed by the other.
//!
//! # Every assertion is on the PUBLISHED KEY STRING
//!
//! `w-guards`' rule — never on `STATIC_SCAN_LOOP_OBJECT`,
//! `STORE_RUN_BIND_NO_CARRIER` or `OPT_MODE`, because a guard on the constant
//! passes a mutation that renames the constant and its uses while the published
//! key moves. `w-deadsites` measured that rule in both directions (`MC2` GREEN,
//! `MC4` RED) and this file inherits it.
//!
//! # `CS3` was already RED, and by a test that never runs a compiler
//!
//! `tests/fence_site_census.rs` catches `w-mutcensus`' registered `CS3`
//! mutation — a **retarget of the arm's key** — because that moves two rows of
//! its per-key raise-site table. It reads source text only. It is therefore
//! blind to any change that leaves every count where it was and moves what a
//! *body* reports: moving the match **label** `"static-scan-loop"` to
//! `"static-scan-loop-x"` leaves `STATIC_SCAN_LOOP_OBJECT` at exactly one raise
//! site, and the body falls through to the `_` arm and reports
//! `callee-unresolved-tail-call:eof` instead. That mutation is measured GREEN on
//! master and RED with this file in the tree
//! (`docs/rungs/2026-08-18-witness7.md` §6). **A source census and a behavioural
//! witness are not substitutes for each other**, and `CS3` is the instance.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. **Load-bearing for `CS3`/`B2`**: `static_scan_loop` is an `/O1`-only
/// transcription, and at the `census` CLI's default `/Ox /GS- /c` the very same
/// source reports `expr-jump` — it never reaches the object gate at all, and
/// every assertion about that gate would be vacuous.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// `/Od`, for the one row whose subject is the optimization word itself.
const FLAGS_OD: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/Od", "/Oi", "/EHsc",
];

/// A scratch directory keyed on the tag **and** the pid — board #1045, and
/// `w-gateperf`'s `reloc_identity` race, which presented as a port defect.
fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-wit7ck-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Capture one source at `flags` and return the census keys in `.ex` order.
fn keys_at(tc: &Toolchain, tag: &str, body: &str, flags: &[&str]) -> Vec<String> {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, body).unwrap();
    let flags: Vec<String> = flags.iter().map(|s| s.to_string()).collect();
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

fn keys(tc: &Toolchain, tag: &str, body: &str) -> Vec<String> {
    keys_at(tc, tag, body, &FLAGS)
}

/// How many of this TU's rows carry `key`.
fn n(ks: &[String], key: &str) -> usize {
    ks.iter().filter(|k| k.as_str() == key).count()
}

/// The scan loop `static_scan_loop.rs` transcribes — `?NextHashPrime@@YAHH@Z`'s
/// shape — with the array's STORAGE as the one free variable. `{decl}` is the
/// only thing that moves between the three cells.
fn scan_loop(decl: &str) -> String {
    format!(
        "{decl}\
         int P(int i) {{\n\
        \x20   for (int j = 0; a[j] != 0; j++) {{\n\
        \x20       if (a[j] >= i)\n\
        \x20           return a[j];\n\
        \x20   }}\n\
        \x20   return i;\n\
         }}\n"
    )
}

/// **`CS3` + `B2` — the static-array scan loop's object gate.**
///
/// `static_scan_loop.rs`' module doc states the clause positively: *"an array
/// that is not a function-local `static` … the object must be COMDAT
/// (`gl::DATA_ATTR_COMDAT`) and **initialized**. A namespace-scope `static` is a
/// non-COMDAT `.data` placed before `.text` (`w-cfg2`'s GRID A cell `a4`, board
/// #1682), which is a different section order; an uninitialized one is a `.bss`
/// COMDAT (cell `a3`), which this lane graded no cell of."* Nothing in 1,648
/// tests could fail on either half of it.
///
/// The three cells differ in **one declaration** and nothing else:
///
/// | cell | the array | `resolve_data_def` |
/// |---|---|---|
/// | `LOCAL` | function-local `static`, initialized | admitted — the body is IN CLASS |
/// | `NS` | namespace-scope `static`, initialized | `!o.comdat` — refused at `bind.rs:974` |
/// | `BSS` | function-local `static`, **un**initialized | `!o.initialized` at the same site, and then no `.in` value either |
///
/// **`NS` is the discriminating cell for `B2` and `BSS` is not**, and this is
/// stated rather than left implicit: with `bind.rs:974` disabled, `NS` passes
/// every remaining clause (thread-local, the `.in` totality identity, the
/// interior-reference gate, `bytes.len() == o.size`) and goes in class, while
/// `BSS` has no `.in` record at all and is refused a second time by
/// `init.values.get(&tok)?` — a site outside this lane's frame. `BSS` is
/// therefore a witness for `CS3`'s key and **not** for `B2`'s clause, and a
/// reader who deletes `NS` as redundant has deleted the only `B2` row.
#[test]
fn the_static_scan_loop_object_gate_refuses_a_non_comdat_or_uninitialized_array() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    const OUT: &str = "static-scan-loop-object-out-of-class:eof";
    const IN: &str = "static-scan-loop";

    let local = keys(
        &tc,
        "ssl-local",
        &scan_loop("").replace(
            "int P(int i) {\n",
            "int P(int i) {\n    static int a[8] = { 2, 3, 5, 7, 11, 13, 17, 0 };\n",
        ),
    );
    let ns = keys(
        &tc,
        "ssl-ns",
        &scan_loop("static int a[8] = { 2, 3, 5, 7, 11, 13, 17, 0 };\n"),
    );
    let bss = keys(
        &tc,
        "ssl-bss",
        &scan_loop("").replace(
            "int P(int i) {\n",
            "int P(int i) {\n    static int a[8];\n",
        ),
    );

    assert_eq!(
        n(&local, IN), 1,
        "CONTROL — the function-local, initialized `static` array is the one \
         storage `resolve_data_def` admits, and the body must be IN CLASS under \
         `{IN}`. Without this the two negatives below are satisfied by a \
         recognizer that refuses its whole input. Keys were {local:?}"
    );
    assert_eq!(
        n(&ns, OUT), 1,
        "`CS3` + `B2` — the SAME loop over a NAMESPACE-SCOPE `static` must be \
         refused under `{OUT}`. Two sites are pinned by this one row: \
         `bind.rs:974`'s `!o.comdat` is what returns `None`, and \
         `census.rs:1288`'s `\"static-scan-loop\" => STATIC_SCAN_LOOP_OBJECT` arm \
         is what files it under this key rather than under \
         `callee-unresolved-tail-call`. This is the discriminating cell for \
         `B2`: disable that clause and this body goes in class. Keys were {ns:?}"
    );
    assert_eq!(
        n(&ns, IN), 0,
        "…and the namespace-scope cell must not be in class by any route — a \
         non-COMDAT object is placed BEFORE `.text` (GRID A cell `a4`, board \
         #1682) and no cell has graded that section order. Keys were {ns:?}"
    );
    assert_eq!(
        n(&bss, OUT), 1,
        "`CS3` — the same loop over an UNINITIALIZED function-local `static` \
         (a `.bss` COMDAT, GRID A cell `a3`) must take the same key. This row \
         witnesses the KEY, not `bind.rs:974`'s clause: with that clause \
         disabled this cell is still refused, by `init.values.get(&tok)?`, \
         because an uninitialized object has no `.in` record. Keys were {bss:?}"
    );
    assert_eq!(
        n(&bss, IN), 0,
        "…and the uninitialized cell must not be in class. Keys were {bss:?}"
    );

    // The trio must DISCRIMINATE. One in class and two refused: a recognizer
    // that answered the same way regardless of the array's storage would
    // satisfy no pair above, and this says so in one line.
    let in_class = [&local, &ns, &bss].iter().filter(|k| n(k, IN) == 1).count();
    let refused = [&local, &ns, &bss].iter().filter(|k| n(k, OUT) == 1).count();
    assert_eq!(
        (in_class, refused), (1, 2),
        "the three-cell table must split 1 in class / 2 refused — got \
         {in_class} in class, {refused} refused. The three sources differ in ONE \
         declaration, so equal verdicts mean the object gate is not being asked"
    );
}

/// **`B7` — the `.bss` resolver's clause is the exact COMPLEMENT of the `.data`
/// resolver's, and neither was guarded on it.**
///
/// `w-mutcensus` §4.3 named this pair explicitly: *"`B2`/`B7` (comdat/init) and
/// `B3`/`B8` (thread-local) are **mirrored** clauses on the two resolution
/// paths, and neither path is guarded on either clause — so the mirror could
/// silently stop being a mirror."* The test above guards `B2`; this one guards
/// `B7`, and the two are deliberately in one file so a future reader who
/// widens one resolver sees the other's row beside it.
///
/// `resolve_bss_def` requires **not COMDAT** and **not initialized** — a
/// function-local `static` with no initializer is a COMDAT `.bss` (`gl.rs`'s
/// cell `a3`, attribute `20`) placed *after* the code groups where a
/// non-COMDAT one is placed before them, and an initialized object is `.data`
/// and belongs to the other resolver.
///
/// # The key this row asserts is the DEFAULT arm's, and that is stated
///
/// `BodyShape::GlobalStoreLeaf` has no arm of its own in `census.rs`'s label
/// `match`, so its refusal falls to `_ => CALLEE_UNRESOLVED_TAIL` and publishes
/// `callee-unresolved-tail-call:eof` — a key it shares with every unnamed
/// label. That makes this row's *key* assertion weaker than the rest of this
/// file's, and the row carries its strength in the pair instead: the control is
/// in class as `global-store-leaf`, so a mutation that admits the wrong object
/// moves the witness from a refusal to that label and the row fails on both
/// assertions at once.
#[test]
fn the_bss_object_gate_refuses_an_initialized_or_comdat_destination() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    const OUT: &str = "callee-unresolved-tail-call:eof";
    const IN: &str = "global-store-leaf";

    // The one cell `resolve_bss_def` admits: a file-scope global with no
    // initializer — not COMDAT, not initialized, size 4.
    let bss = keys(&tc, "gsl-bss", "int G;\nvoid f(int v) { G = v; }\n");
    // `o.initialized` — the same object with `= 7`. One character of source.
    let init = keys(&tc, "gsl-init", "int G = 7;\nvoid f(int v) { G = v; }\n");
    // `o.comdat` — a function-local `static`, which is a COMDAT `.bss`.
    let comdat = keys(&tc, "gsl-comdat", "void f(int v) { static int s; s = v; }\n");

    assert_eq!(
        n(&bss, IN), 1,
        "CONTROL — a file-scope global with no initializer is the one \
         destination `resolve_bss_def` admits, and the store leaf must be IN \
         CLASS under `{IN}`. Keys were {bss:?}"
    );
    assert_eq!(
        n(&init, OUT), 1,
        "`B7` — the SAME body storing into an INITIALIZED global must be \
         refused. `bind.rs:1030`'s `o.comdat || o.initialized` is what returns \
         `None`: an initialized object is `.data` and belongs to \
         `resolve_data_def`, and admitting it here would emit a `.bss` for an \
         object whose bytes exist. Keys were {init:?}"
    );
    assert_eq!(
        n(&init, IN), 0,
        "…and the initialized cell must not be in class. Keys were {init:?}"
    );
    assert_eq!(
        n(&comdat, OUT), 1,
        "`B7`, the other disjunct — a function-local `static` is a COMDAT `.bss` \
         (`gl.rs` cell `a3`, attribute `20`), placed AFTER the code groups where \
         a non-COMDAT one is placed before them. Two different section orders, \
         and no cell has graded the first. Keys were {comdat:?}"
    );
    assert_eq!(
        n(&comdat, IN), 0,
        "…and the COMDAT cell must not be in class. Keys were {comdat:?}"
    );

    let in_class = [&bss, &init, &comdat].iter().filter(|k| n(k, IN) == 1).count();
    assert_eq!(
        in_class, 1,
        "exactly ONE of the three storages may be admitted — got {in_class}. \
         `B2` and `B7` are mirrored clauses (`w-mutcensus` §4.3) and this is the \
         count that fails when the mirror stops being one"
    );
}

/// **`CS4` — a `store-run-bind` refusal must report `bind_run_ops`' own reason,
/// not the fallback.**
///
/// Board **#1199** split one label's refusal into four named keys *"each with
/// its own key so each residue is separately sizeable — and one of them, the
/// mixed-kind run, is boards #836/#868 becoming a countable row on the
/// frontier's cheapest TU for the first time."* The census consults
/// `bind_refusal_key` and falls back to `STORE_RUN_BIND_NO_CARRIER` only when
/// the bind itself is fine. `w-mutcensus`' `CS4` mutation drops that routing —
/// every bind body then reports the fallback and the four residues collapse into
/// one — and it was **GREEN over 1,648 tests**.
///
/// The three cells are `w1199_bind_run_neg.cpp`'s own clause witnesses,
/// **inlined** rather than read from `fixtures/`: a peer owns that directory,
/// and a witness whose input another lane can edit is not a witness. They are
/// the three that still refuse at the workload's profile.
///
/// # What this row cannot see, said plainly
///
/// No cell here produces `store-run-bind-no-emitter-carrier`, so the fallback
/// arm itself is unwitnessed. The row therefore fails a mutation that *removes*
/// the routing (all three keys collapse) and would **not** fail one that
/// removes the fallback while keeping the routing. That second population is
/// board **#844**'s composition residue and is a lane, not a footnote.
///
/// # And a stale claim in the fixture this was drawn from
///
/// `w1199_bind_run_neg.cpp`'s header says *"one function per clause of the
/// accept boundary, and every one of them must be **0 of N in class**"*. At
/// `666fe6eb7` it censuses **1 of 4 in class** — `nf_mixed`, its
/// `store-run-bind-mixed-kind-alloc` witness, is in class at `/O1` and at the
/// `/Ox` fixture default alike. That is recorded in
/// `docs/rungs/2026-08-18-witness7.md` §10 and not fixed here: `fixtures/` is
/// not this lane's seam.
#[test]
fn each_bind_run_refusal_reports_its_own_key_and_not_the_no_carrier_fallback() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    const HDR: &str = "struct BE { BE* mNext; BE* mPrev; };\n\
                       struct H {\n\
                      \x20   H* mFreeHead;\n\
                      \x20   H* mUsedHead;\n\
                      \x20   BE mListHead;\n\
                      \x20   unsigned mSize;\n\
                      \x20   unsigned mCount;\n\
                      \x20   unsigned mA;\n\
                      \x20   unsigned mB;\n\
                      \x20   BE mSecond;\n\
                       };\n";
    const FALLBACK: &str = "store-run-bind-no-emitter-carrier:eof";

    // Two distinct interior addresses — single-kind, so `alloc::allocate`
    // answers them, and answering is not being measured (`w-midrun`).
    let addr = keys(
        &tc,
        "srb-addr",
        &format!(
            "{HDR}void nf_addrprod(H* h) {{\n\
            \x20   BE& l = h->mListHead;\n\
            \x20   BE& m = h->mSecond;\n\
            \x20   h->mFreeHead = (H*)&l;\n\
            \x20   h->mUsedHead = (H*)&m;\n\
             }}\n"
        ),
    );
    // Two distinct literals beside a bound base: at ONE producer
    // `order::store_order`'s walk provably cannot fail; at two it can.
    let multi = keys(
        &tc,
        "srb-multi",
        &format!(
            "{HDR}void nf_twoprod(H* h, BE* p) {{\n\
            \x20   h->mA = 2;\n\
            \x20   h->mB = 3;\n\
            \x20   BE& l = h->mListHead;\n\
            \x20   l.mNext = p;\n\
             }}\n"
        ),
    );
    // h, l, h, l is THREE base-symbol group boundaries, one past
    // `order::MAX_SYMBOL_CROSSINGS`.
    let cross = keys(
        &tc,
        "srb-cross",
        &format!(
            "{HDR}void nf_cross3(H* h, BE* p) {{\n\
            \x20   BE& l = h->mListHead;\n\
            \x20   h->mSize = 2;\n\
            \x20   l.mNext = p;\n\
            \x20   h->mFreeHead = h;\n\
            \x20   l.mPrev = p;\n\
             }}\n"
        ),
    );

    assert_eq!(
        n(&addr, "store-run-bind-address-producer:eof"), 1,
        "`CS4` row 1 — TWO distinct interior addresses in store value positions \
         must report `bind_run_ops`' OWN reason, not the fallback. Keys were \
         {addr:?}"
    );
    assert_eq!(
        n(&multi, "store-run-bind-multi-producer:eof"), 1,
        "`CS4` row 2 — two distinct literals beside a bound base. Keys were \
         {multi:?}"
    );
    assert_eq!(
        n(&cross, "store-run-bind-symbol-crossings:eof"), 1,
        "`CS4` row 3 — three base-symbol group boundaries, one past \
         `order::MAX_SYMBOL_CROSSINGS`. Keys were {cross:?}"
    );

    for (tag, k) in [("addr", &addr), ("multi", &multi), ("cross", &cross)] {
        assert_eq!(
            n(k, FALLBACK), 0,
            "cell `{tag}` must NOT report `{FALLBACK}`. That is exactly \
             `w-mutcensus`' `CS4` mutation — drop `bind_key.unwrap_or(…)` and \
             every bind refusal collapses onto the fallback, which was GREEN \
             over 1,648 tests. Keys were {k:?}"
        );
    }

    // Three cells, three DIFFERENT keys. Asserted as a set size because the
    // three assertions above are each satisfiable by a routing that answers
    // the same way for a reason this test happens not to look at.
    let mut named: Vec<&str> = vec![
        "store-run-bind-address-producer:eof",
        "store-run-bind-multi-producer:eof",
        "store-run-bind-symbol-crossings:eof",
    ];
    named.sort_unstable();
    named.dedup();
    let distinct = [&addr, &multi, &cross]
        .iter()
        .filter(|ks| ks.iter().any(|k| named.binary_search(&k.as_str()).is_ok()))
        .count();
    assert_eq!(
        distinct, 3,
        "the three cells must carry three DIFFERENT #1199 keys — got {distinct} \
         of 3. One key for three residues is the state #1199 replaced, and it is \
         what `CS4`'s mutation restores"
    );
}

/// **`CS9` — the post-parse optimization-mode gate.**
///
/// `census.rs:1323` is applied **last**, to an otherwise-in-class function
/// only: *"`.ex` records the optimization word per function and the port emits
/// only the two words it has been verified against; the rest — `/Od`, a
/// `#pragma optimize(\"\", off)`, an unreadable prefix — are refused."*
/// `w-mutcensus` registered its mutation **RED at 0.60** and measured **GREEN**;
/// it is one of that campaign's three registered-RED-observed-GREEN misses.
///
/// The pair is the same four lines of C++ captured at two profiles. At `/O1`
/// the body is `multiarg-tail-call`; at `/Od` it parses to exactly the same
/// shape and is refused for the word alone, which is what makes this a witness
/// for the gate rather than for the parser.
///
/// The key carries the WORD (`Block::feature` renders `OPT_MODE` from
/// `Block::aux`, `docs/OPT_MODE.md` decodes the values), and the full string is
/// asserted rather than a prefix: `opt-mode-00800005` is `/Od`'s own word at
/// this base, and a change that reached the gate with a different word would be
/// a change to what the census reports.
#[test]
fn a_body_in_class_at_an_unmodelled_optimization_word_reports_the_opt_mode_key() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    const BODY: &str = "void g2(int, int);\nvoid f(int a, int b) { g2(a, b); }\n";
    const OD_KEY: &str = "opt-mode-00800005";
    const IN: &str = "multiarg-tail-call";

    let o1 = keys_at(&tc, "optmode-o1", BODY, &FLAGS);
    let od = keys_at(&tc, "optmode-od", BODY, &FLAGS_OD);

    assert_eq!(
        n(&o1, IN), 1,
        "CONTROL — at the workload's own `/O1` this body is IN CLASS as `{IN}`. \
         Without it the negative below is satisfied by a body that never parsed. \
         Keys were {o1:?}"
    );
    assert_eq!(
        n(&od, OD_KEY), 1,
        "`CS9` — the SAME source at `/Od` must be refused by the post-parse \
         optimization-mode gate under `{OD_KEY}`. The gate runs on an \
         otherwise-in-class function only, so this key is the whole difference \
         between the two captures and nothing else about the body moved. \
         `w-mutcensus` registered this site RED at 0.60 and measured it GREEN \
         over 1,648 tests. Keys were {od:?}"
    );
    assert_eq!(
        n(&od, IN), 0,
        "…and the `/Od` capture must NOT be in class: `PortC2` refuses every \
         mode word it has not been verified against, and a census that claimed \
         this function is roadmap #44's over-claim exactly. Keys were {od:?}"
    );
    assert_ne!(
        o1, od,
        "the pair must DISCRIMINATE — one source, two profiles, and identical \
         key lists mean the optimization word is not being read at all"
    );
}
