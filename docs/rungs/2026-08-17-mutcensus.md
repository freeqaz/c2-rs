# MUTCENSUS — how many of `c2-il`'s refusal sites have no test that can fail on them

    Tag:       w-mutcensus
    Slug:      mutcensus
    Date:      2026-08-17
    Kind:      characterization — the question: which of `crates/c2-il`'s
               refusal/fence sites are unguarded, measured by one registered
               mutation per site against the full workspace suite
    Outcome:   instrument
    Fixtures:  none — characterization
    Census:    +0 — required-zero byte delta; every `crates/` edit in this lane
               is an applied-and-reverted mutant, and
               `git diff master..HEAD -- crates fixtures scripts` is EMPTY at
               the tip
    Record:    this file; prereg `_2026-08-17-w-mutcensus-prereg.md` (frozen at
               `58cb6803`, committed BEFORE the first mutant ran); deviations
               and corrections `work/w-mutcensus/deviations.md`; raw logs,
               runner and table generators under `work/w-mutcensus/` (tracked)

Provenance: board **#3217** — *"`#3199`'s list of unguarded surfaces was NOT
exhaustive … and NOTHING ANYWHERE ENUMERATES THE FENCES THAT HAVE NO TEST … a
mutation census over `crates/c2-il`'s refusal sites is a lane, and it is the only
thing that turns this from anecdote into a NUMBER."* This is that lane.

## 0. The answer

**X = 30 of the 63 enumerated `crates/c2-il` fence sites have NO test in the
1,648 that can fail on them.** 33 are guarded. Every one of the 63 was measured —
**0 NOT RUN, 0 INVALID** — by one registered mutation per site against
`cargo test --workspace --release --no-fail-fast`, with each run's real-`c2`
differential verified to have actually graded (§7).

Registered before any mutant ran: **X = 38**, 80 % interval **[30, 46]**.
**Observed 30 — inside the interval, at its exact lower bound.** Prereg scored
**50 hits / 14 misses** over the 64 registered colours (§6).

**Controls: 5 of 5 RED, no anomalies** — the four guards `w-guards` landed last
wave are independently confirmed to hold, reproducing their failing-test sets and
counts exactly (§5).

**But the flat X/N is the least interesting half of the answer.** The census was
commissioned off board **#3217** to count a *shape* — "guarded at one raise site
of four" — and that shape is not an anecdote: **it is the rule, and the axis it
runs along is whether a site raises a KEY or decides a GATE.**

* **Key-routing sites are almost entirely unasserted**: of the 16 key-swap
  mutations, **12 are GREEN (75 %)**. Nothing anywhere pins *which* key most of
  these sites raise.
* **Gate-removal sites are much better covered**: 16 GREEN of 36 (44 %).
* **Threshold widenings are the best covered**: 2 GREEN of 11 (18 %).

And six families are **wholly unguarded at every raise site they have** —
including the entire 4-site `callee-unresolved` key family, whose default arm
routes the key `#3209` measured rising to **1,296** bodies on the 878-TU
workload.

## 1. Populations, and where each figure was measured

| figure | value | measured |
|---|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,648 passed / 0 failed / 42 targets** | re-measured at `3835469c` (`work/w-mutcensus/baseline_test.log`); reproduced as the registered `N0` control **five** further times across four independently provisioned worktrees |
| mutation runs completed | **63 of 63** in-`N` sites + `C2` = **64 of 64** registered colours | `work/w-mutcensus/results/` (one full-suite log per run, all tracked) |
| sites measured **twice**, in independent worktrees | **54 of 63** | `work/w-mutcensus/collect.sh` — 94 duplicate pairs, **93 agree exactly**, 1 partial (§4.3) |
| 878-TU scan at the tip | `match` **25** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **845** · `fnbyte-exact` **35,734** | `work/w-mutcensus/scan_tip.log`, this worktree |
| anchored `gap-metric` keys | **394** | same scan — **identical to the briefed base at `3835469c`, 0 deltas over all 394** |

## 2. What a fence site is here, and how the 63 were enumerated

The rule is `work/w-mutcensus/enumerate.sh`, run from the repo root at
`3835469c`, plus a **bounded, published** reading step. Both are quoted from the
frozen prereg §1 rather than restated, so the frame cannot drift after the fact:

* **E1** — every `refuse("<key>")` raise site: **23**, all in
  `func/body/shapes/calls.rs`.
* **E2** — every non-test, non-doc line that **raises** one of the **19
  fence-key constants** (the 24 `pub(crate) const … : &str` in `func/body/mod.rs`
  minus the 5 dispatch-state constants and the 2 grammar-context constants).
* **E3** — every `Block::at_end(` site.
* **Reading step (bounded):** within each function containing an E1–E3 site,
  every conditional that *decides whether the raise fires* is part of the site,
  plus the resolver/gate functions those sites call (`resolve_data`,
  `resolve_data_def`, `resolve_bss_def`, `is_varargs`, `gl_extern_data_names`,
  `NAME_SEPARATORS`, `opt_word_mode`) and `IlBundle`'s TU-level admission gates
  (`functions`, `dyninit_tu`, `data_tu`).

**N for the headline is the 63 mutated `c2-il` sites.** `C2` — the `c2-core`
backstop at `codegen/calls.rs:1815` — is run as a control but is **not** in N,
because it is not a `c2-il` fence.

### 2.1 What was enumerated and deliberately NOT mutated — with counts

No silent caps: every dropped class is published with its size and its reason
(prereg §1 E5).

| dropped class | count | why |
|---|---:|---|
| grammar fail-closed `blk(` sites | **1,227** raw grep lines | one suite run per site is ≈ 5 days of wall clock, and they are a different guard class — the key is generated *from the blocking byte*, so a key-swap mutation does not exist and a removal merely moves the parse to the next blocking byte. A **sampled** census over them is a future lane |
| `blk_type(` | **6** | same class |
| `Block::refuse(` | **106** | same class (counts overlap; they include helper definitions and test uses) |
| shape-file `OptWordMode` admission predicates | **18** non-test comparison sites | budget; second-tier published-key proximity |
| `IlBundle::dyninit_tu` `return None` clauses | **12**, of which **1** mutated (`D1`) | budget — **11 dropped** |
| `IlBundle::data_tu` `return None` clauses | **14**, of which **1** mutated (`D2`) | budget — **13 dropped** |
| `IlBundle::functions()` interior gates past the three mutated | enumerated by reading | budget |
| `STORE_RUN_BIND_CALL_TAIL_RETIRED` | **0 live raise sites** | a fence key with **no fence** — test-only since #1212's correction. No mutant is possible |

**So the headline is not "all fences in `c2-il`".** It is *all 63 sites the
frozen frame enumerated*, beside a published 1,227-site class the frame
deliberately does not reach.

### 2.2 The frame already has a hole, and a peer put it there during the campaign

`w-fence163` landed `d28326b4`
(*"admit narrow string literals behind an EH-state inline fence"*) while this
campaign was running. It adds a **20th** fence-key constant —
`DATA_SYM_STRLIT_FENCED = "data-sym-strlit-fenced"` in `func/body/mod.rs` — with
**5** lines mentioning it and new deciding gates in `bind.rs`,
`bundle.rs::functions()`, `census.rs` and `gl.rs` (+240 / −13 over five `c2-il`
files).

**This lane did not re-enumerate to absorb it, and must not:** the frame and all
64 registered colours were frozen at `3835469c` before the first mutant ran, and
widening the frame afterwards would unfreeze the prereg. So the site is recorded
as one the census **necessarily misses** — and the more useful thing it
establishes is the instrument's **shelf life**:

> **One peer lane landing one fence is enough to make X/N stale.** A mutation
> census over `c2-il`'s fences is a fact about a *commit*, not about the
> repository. Re-running `enumerate.sh` is a precondition of quoting X/N against
> any later head, and *nothing in the repo enforces that* — see §9.

## 3. The table — every one of the 63 sites, registered against observed

| id | site (`crates/c2-il/src/func/…` at `3835469c`) | mutation | reg. | observed | pass/fail | failing tests |
|---|---|---|---|---|---|---|
| C1 *(control)* | `calls.rs:431` | `syms > 1` -> `syms > 2` (arity fence) | RED 0.97 | **RED** HIT | 1646/2 | gap::tests::wr1_census_key_guards::the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol<br>gap::tests::wr1_census_key_guards::the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone |
| C2 *(control)* | `c2-core calls.rs:1815` | `count() != 1` -> `> 2` (backstop) | RED 0.95 | **RED** HIT | 1647/1 | codegen::calls::tests::the_data_address_setup_refuses_the_shapes_it_has_no_capture_for |
| C3 *(control)* | `bind.rs:891` | `.then_some(name)` -> unconditional `Some(name)` | RED 0.97 | **RED** HIT | 1645/3 | gap::tests::wr1_census_key_guards::the_census_key_survives_the_round_trip_into_the_reachable_ranking<br>gap::tests::wr1_census_key_guards::the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key<br>gap::tests::wr1_census_key_guards::the_two_data_symbol_census_keys_are_not_interchangeable |
| C4 *(control)* | `census.rs:1216/1218` | swap DATA_SYM_UNRESOLVED / DATA_SYM_LINKAGE | RED 0.97 | **RED** HIT | 1646/2 | gap::tests::wr1_census_key_guards::the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key<br>gap::tests::wr1_census_key_guards::the_two_data_symbol_census_keys_are_not_interchangeable |
| C5 *(control)* | `calls.rs:430` | `false &&` on the two-sym thunk exemption | RED 0.90 | **RED** HIT | 1642/6 | func::body::wr1_dyninit::the_dynamic_initializer_thunk_decodes_to_a_three_slot_tail_call<br>func::body::wr1_dyninit::the_empty_module_scope_is_optional_and_the_module_end_is_not<br>gap::tests::wr1_census_key_guards::the_census_key_survives_the_round_trip_into_the_reachable_ranking<br>gap::tests::wr1_census_key_guards::the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key<br>gap::tests::wr1_census_key_guards::the_two_data_symbol_census_keys_are_not_interchangeable<br>gap::tests::wr1_census_key_guards::the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone |
| CS2 | `census.rs:1242` | key -> STATIC_SCAN_LOOP_OBJECT | GREEN 0.75 | **GREEN** HIT | 1648/0 | — |
| CS3 | `census.rs:1245` | key -> STORE_RUN_CALL_NO_CARRIER | GREEN 0.75 | **GREEN** HIT | 1648/0 | — |
| CS4 | `census.rs:1263` | drop `bind_key.unwrap_or` | GREEN 0.65 | **GREEN** HIT | 1648/0 | — |
| CS5 | `census.rs:1265` | key -> CALLEE_UNRESOLVED_TAIL | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| CS6 | `census.rs:1267` | key -> CALLEE_UNRESOLVED_TAIL | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| CS7 | `census.rs:1270` | key -> CALLEE_UNRESOLVED_TAIL | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| CS8 | `census.rs:1272` | default arm key -> CALLEE_UNRESOLVED_FRAMED | RED 0.80 | **GREEN** **MISS** | 1648/0 | — |
| CS9 | `census.rs:1280` | `false &&` on the opt-mode gate | RED 0.60 | **GREEN** **MISS** | 1648/0 | — |
| CS10 | `census.rs:1294` | `false &&` on ptr-walk-not-O1 | RED 0.60 | **RED** HIT | 1647/1 | the_census_and_the_port_agree_about_what_is_in_class |
| CS11 | `census.rs:1306` | `false &&` on chain-not-O1 | RED 0.60 | **RED** HIT | 1647/1 | the_census_and_the_port_agree_about_what_is_in_class |
| CS12 | `census.rs:1358` | `false &&` on callee-defined-in-tu | RED 0.90 | **RED** HIT | 1645/3 | a_callee_that_keeps_bytes_stops_the_chain<br>a_callee_this_tu_defines_is_fenced_and_its_opaque_twin_is_not<br>the_fence_yields_to_the_empty_callee_mechanism_e_already_models |
| CA2 | `calls.rs:434` | MAX_REGISTER_FORMALS + 9 (sym overflow) | GREEN 0.80 | **GREEN** HIT | 1648/0 | — |
| CA3 | `calls.rs:442` | `false &&` sym-permuted | GREEN 0.75 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA4 | `calls.rs:529` | MAX_REGISTER_FORMALS + 9 (lit slots) | GREEN 0.80 | **RED** **MISS** | 1647/1 | func::body::shapes::calls::tests::arg_site_sequence_still_refuses_the_slot_bound |
| CA5 | `calls.rs:593` | `false &&` lit-permuted | GREEN 0.70 | **RED** **MISS** | 1645/3 | func::body::shapes::calls::tests::arg_site_decides_the_literal_paths_permutation_clause<br>the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA6 | `calls.rs:693` | key nonformal -> computed (slot arm) | GREEN 0.50 | **GREEN** HIT | 1648/0 | — |
| CA7 | `calls.rs:699` | `false &&` lit-wide | GREEN 0.75 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA8 | `calls.rs:710` | key computed -> nonformal | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| CA9 | `calls.rs:732` | key lit- -> sym-classified-twice | GREEN 0.90 | **GREEN** HIT | 1648/0 | — |
| CA10 | `calls.rs:736` | key sym- -> lit-classified-twice | GREEN 0.90 | **GREEN** HIT | 1648/0 | — |
| CA11 | `calls.rs:747` | `false &&` outer-formal panic guard | RED 0.70 | **RED** HIT | 1646/2 | func::body::shapes::calls::tests::a_call_argument_from_a_formal_beyond_the_argument_count_refuses_and_does_not_panic<br>func::body::shapes::calls::tests::a_call_bound_to_a_local_gets_the_same_argument_gates_as_the_direct_form |
| CA12 | `calls.rs:759` | `false &&` duplicated source | GREEN 0.70 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA13 | `calls.rs:772` | key source-out-of-slots -> outer-formal | GREEN 0.80 | **GREEN** HIT | 1648/0 | — |
| CA14 | `calls.rs:774` | `cycles > 1` -> `> 9` | GREEN 0.70 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA15 | `calls.rs:780` | MAX_VERIFIED_PERM_CYCLE + 9 | GREEN 0.75 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA16 | `calls.rs:792` | `false &&` repeated-leaf | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| CA17 | `calls.rs:800` | `false &&` noncanonical-order (loads) | RED 0.55 | **RED** HIT | 1647/1 | func::body::shapes::calls::tests::a_call_bound_to_a_local_gets_the_same_argument_gates_as_the_direct_form |
| CA18 | `calls.rs:803` | `false &&` noncanonical-order (chain) | GREEN 0.55 | **GREEN** HIT | 1648/0 | — |
| CA19 | `calls.rs:806` | `false &&` nonformal (post) | RED 0.55 | **RED** HIT | 1647/1 | func::body::shapes::calls::tests::a_call_argument_that_is_not_a_formal_refuses_in_the_parser |
| CA20 | `calls.rs:868` | MAX_REGISTER_FORMALS + 9 (mcall chain) | GREEN 0.80 | **RED** **MISS** | 1647/1 | the_census_and_the_port_agree_about_what_is_in_class |
| CA21 | `calls.rs:878` | mcall key nonformal -> computed | RED 0.85 | **RED** HIT | 1647/1 | func::body::shapes::mcall_chain::tests::a_computed_or_nonformal_link_argument_refuses_by_name |
| CA22 | `calls.rs:883` | `false &&` mcall lit-wide | GREEN 0.80 | **RED** **MISS** | 1646/2 | the_census_and_the_port_agree_about_what_is_in_class<br>the_census_and_the_port_agree_over_the_generated_corpus |
| CA23 | `calls.rs:893` | mcall key computed -> nonformal | RED 0.85 | **RED** HIT | 1647/1 | func::body::shapes::mcall_chain::tests::a_computed_or_nonformal_link_argument_refuses_by_name |
| B2 | `bind.rs:929` | `false &&` data-def comdat/init | GREEN 0.60 | **GREEN** HIT | 1648/0 | — |
| B3 | `bind.rs:932` | `false &&` data-def thread-local | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| B4 | `bind.rs:939` | `false &&` `.in` totality | GREEN 0.65 | **GREEN** HIT | 1648/0 | — |
| B5 | `bind.rs:942` | `false &&` `.in` refs | GREEN 0.65 | **GREEN** HIT | 1648/0 | — |
| B6 | `bind.rs:946` | `false &&` size-exact | GREEN 0.60 | **GREEN** HIT | 1648/0 | — |
| B7 | `bind.rs:985` | `false &&` bss-def comdat/init | GREEN 0.60 | **GREEN** HIT | 1648/0 | — |
| B8 | `bind.rs:988` | `false &&` bss-def thread-local | GREEN 0.70 | **GREEN** HIT | 1648/0 | — |
| B9 | `bind.rs:991` | `false &&` bss-def size==0 | GREEN 0.70 | **RED** **MISS** | 1647/1 | the_cells_population_is_three_functions_one_of_which_disagrees |
| B10 | `bind.rs:862` | `false &&` varargs name gate | RED 0.65 | **RED** HIT | 1646/2 | func::bind::tests::the_varargs_gate_is_one_predicate_on_both_paths<br>func::census::tests::a_body_the_ladder_never_ran_for_reads_disp_not_run |
| G1 | `gl.rs:2188` | `|| true` on the extern-data linkage byte | RED 0.90 | **RED** HIT | 1644/4 | func::gl::data_object_tests::the_new_reader_and_the_extern_gate_admit_disjoint_linkages<br>gap::tests::wr1_census_key_guards::the_census_key_survives_the_round_trip_into_the_reachable_ranking<br>gap::tests::wr1_census_key_guards::the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key<br>gap::tests::wr1_census_key_guards::the_two_data_symbol_census_keys_are_not_interchangeable |
| G2 | `gl.rs:2198` | retain -> keep all (ambiguous names) | GREEN 0.55 | **GREEN** HIT | 1648/0 | — |
| G3 | `gl.rs:1085` | NAME_SEPARATORS drop 0x26 | RED 0.85 | **RED** HIT | 1594/54 | a_callee_that_keeps_bytes_stops_the_chain<br>a_callee_this_tu_defines_is_fenced_and_its_opaque_twin_is_not<br>a_defined_name_that_is_a_prefix_of_the_callee_fences_nothing<br>both_pool2_files_bind_their_records_before_any_body_is_looked_at<br>differential_class_a_many_calls_byte_exact<br>differential_fp_argument_registers_byte_exact<br>differential_mvp_add3_port_byte_exact<br>differential_mvp_argtail_arg_setup_byte_exact<br>differential_mvp_call_tailcall_byte_exact<br>differential_mvp_framed_call_byte_exact<br>differential_mvp_lit_immediates_byte_exact<br>differential_mvp_plus0_identity_fold_byte_exact<br>differential_mvp_sub_noncommutative_byte_exact<br>differential_mvp_tailret_int_passthrough_byte_exact<br>differential_mvp_two_multifunction_byte_exact<br>differential_mvp_wide_immediates_byte_exact<br>differential_w1274_interior_address_producer_byte_exact<br>differential_wadjust_object_receiver_byte_exact<br>differential_wbdnz_ctr_ox_accepted<br>differential_wpark_lit_permuted_pair<br>differential_wunw_multi_function_framed_byte_exact<br>every_pool_cell_binds_its_records_before_the_body_is_looked_at<br>func::bind::tests::a_26_separated_name_binds_and_the_framing_arity_does_not_move<br>func::bind::tests::a_26_that_introduces_no_plausible_name_yields_no_binding<br>func::bind::tests::per_record_binds_each_segment_to_the_record_carrying_its_offset<br>func::bind::tests::record_bytes_before_a_26_are_not_glued_onto_the_next_name<br>func::bind::tests::selective_is_per_record_when_the_records_are_one_to_one<br>func::bind::tests::selective_refuses_a_mangled_run_no_record_claims<br>func::bind::tests::selective_refuses_an_unclaimed_run_that_fits_the_inline_name_field<br>func::bind::tests::selective_refuses_a_record_offset_that_is_not_a_split_point<br>func::bind::tests::selective_refuses_even_an_exhausted_name_set_because_records_are_not_the_emit_set<br>func::bind::tests::selective_refuses_records_that_do_not_advance_through_the_segments<br>func::bind::tests::the_two_bindings_are_the_open_seam_and_are_pinned_apart<br>func::bind::tests::the_varargs_gate_is_one_predicate_on_both_paths<br>func::bundle::dyninit_tests::functions_refuses_when_a_body_c2_may_have_emitted_is_unaccounted_for<br>func::gl::tests::a_26_introduced_name_is_invisible_to_the_nul_scanner_and_visible_to_the_other<br>func::gl::tests::a_26_introduced_record_is_SEEN_by_the_scanner_and_REFUSED_by_the_gate<br>func::gl::tests::an_undecorated_record_name_is_seen_then_refused<br>func::gl::tests::a_record_name_terminated_by_26_refuses_rather_than_reading_past_it<br>func::gl::tests::gl_names_bind_to_their_own_record_not_their_position<br>func::gl::tests::gl_symbol_index_does_not_glue_a_records_own_token_onto_its_name<br>func::gl::tests::gl_symbol_index_reads_both_separator_forms<br>func::gl::tests::gl_symbol_runs_ignore_non_symbol_strings<br>func::gl::tests::only_a_plain_external_defined_record_is_exempt<br>func::gl::tests::the_fence_walk_never_grows_when_the_binding_walk_widens<br>func::gl::tests::the_relaxed_framing_reaches_the_binding_and_neither_fence<br>func::gl::tests::the_widened_walk_refuses_an_unmangled_variadic_record<br>roundtrip_all_fixtures_byte_identical<br>the_fence_yields_to_the_empty_callee_mechanism_e_already_models<br>the_free_list_tu_is_in_class_end_to_end<br>the_inline_fence_holds_one_all_exact_tu_and_the_counter_says_so<br>the_parser_hands_on_only_the_linkage_class_the_decline_bound_was_measured_on<br>the_three_pool_cells_each_move_the_verdict_by_one_construct<br>the_three_vec_cells_stop_at_three_different_gates |
| BU1 | `bundle.rs:1694` | opt_word_mode unknown -> `Some(Ox)` | RED 0.70 | **RED** HIT | 1645/3 | func::bundle::tests::fp_contract_off_is_still_the_mode_it_was_compiled_at<br>func::bundle::tests::opt_words_reports_an_unreadable_prefix_rather_than_guessing<br>func::bundle::tests::the_optimization_word_is_a_varint_not_a_fixed_escape |
| BU2 | `bundle.rs:1919` | `false &&` drectve gate | GREEN 0.60 | **RED** **MISS** | 1647/1 | func::bundle::dyninit_tests::a_bundle_without_the_three_streams_refuses |
| BU3 | `bundle.rs:1940` | `|| true` empty-module LO probe | GREEN 0.55 | **GREEN** HIT | 1648/0 | — |
| D1 | `bundle.rs:2423` | `false &&` dyninit name clause | RED 0.60 | **GREEN** **MISS** | 1648/0 | — |
| D2 | `bundle.rs:2887` | `false &&` data_tu `.in` totality | GREEN 0.55 | **GREEN** HIT | 1648/0 | — |
| L1 | `leaf_store.rs:2254` | key GROUP_SHAPE -> MULTI_PRODUCER | RED 0.65 | **RED** HIT | 1647/1 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input |
| L2 | `leaf_store.rs:2257` | key GROUP_SHAPE -> MULTI_PRODUCER | GREEN 0.55 | **GREEN** HIT | 1648/0 | — |
| L3 | `leaf_store.rs:2285` | key GROUP_SHAPE -> MULTI_PRODUCER | GREEN 0.50 | **GREEN** HIT | 1648/0 | — |
| L4 | `leaf_store.rs:2370` | `false &&` mixed-kind | RED 0.80 | **RED** HIT | 1646/2 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input<br>the_census_and_the_port_agree_about_what_is_in_class |
| L5 | `leaf_store.rs:2374` | `== 0` -> `== i32::MIN` (addr producer) | RED 0.60 | **RED** HIT | 1647/1 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input |
| L6 | `leaf_store.rs:2390` | `lits.len() > 1` -> `> 9` | RED 0.75 | **RED** HIT | 1647/1 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input |
| L7 | `leaf_store.rs:2399` | `false &&` pool bound | RED 0.55 | **RED** HIT | 1647/1 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input |
| L8 | `leaf_store.rs:2402` | MAX_SYMBOL_CROSSINGS + 9 | RED 0.75 | **RED** HIT | 1647/1 | func::body::shapes::leaf_store::tests::every_bind_gate_fires_on_a_named_input |
| L9 | `leaf_store.rs:2455` | `false &&` group-shape (2nd walk) | GREEN 0.60 | **GREEN** HIT | 1648/0 | — |

**X = 30 GREEN (unguarded) of 63 of the 63 c2-il fence sites run** — 0 NOT RUN, 0 INVALID. Prereg: 50 hits / 14 misses over the colours scored.

## 4. The per-family pattern — the shape #3217 asked to be counted

The commissioned shape — **"guarded at one raise site of four"** — reproduces,
and generalizes into a rule with a mechanism.

### 4.1 The rollups

### Rollup by mutation kind — what a GREEN means differs

| kind | what a GREEN establishes | sites run | RED | GREEN |
|---|---|---:|---:|---:|
| swap | no assertion pins WHICH key this site raises | 16 | 4 | 12 |
| remove | no fixture or unit test exercises the class the fence refuses | 36 | 20 | 16 |
| widen | nothing exercises the population between the old and new bound | 11 | 9 | 2 |

### Per-family rollup (guarded raise sites / raise sites in family)

| family | sites | RED (guarded) | GREEN (unguarded) | shape |
|---|---|---|---|---|
| leaf-store residue gates | 5 | 5 ['L4', 'L5', 'L6', 'L7', 'L8'] | 0 [] | wholly guarded (5/5) |
| callee-unresolved key family (4) | 4 | 0 [] | 4 ['CS5', 'CS6', 'CS7', 'CS8'] | **wholly unguarded** (4/4) |
| data-def / bss-def mirror | 4 | 0 [] | 4 ['B2', 'B3', 'B7', 'B8'] | **wholly unguarded** (4/4) |
| group-shape raise family (4) | 4 | 1 ['L1'] | 3 ['L2', 'L3', 'L9'] | **guarded at 1 of 4** raise sites |
| census opt/ptr-walk gates | 3 | 2 ['CS10', 'CS11'] | 1 ['CS9'] | **guarded at 2 of 3** raise sites |
| MAX_REGISTER_FORMALS threshold (3) | 3 | 2 ['CA4', 'CA20'] | 1 ['CA2'] | **guarded at 2 of 3** raise sites |
| nonformal/computed key pair | 3 | 1 ['CA19'] | 2 ['CA6', 'CA8'] | **guarded at 1 of 3** raise sites |
| call-arg source/slot | 3 | 2 ['CA11', 'CA12'] | 1 ['CA13'] | **guarded at 2 of 3** raise sites |
| call-arg op shape | 3 | 1 ['CA17'] | 2 ['CA16', 'CA18'] | **guarded at 1 of 3** raise sites |
| gl linkage/name | 3 | 2 ['G1', 'G3'] | 1 ['G2'] | **guarded at 2 of 3** raise sites |
| bundle TU gate | 3 | 2 ['BU1', 'BU2'] | 1 ['BU3'] | **guarded at 2 of 3** raise sites |
| call-arg arity | 2 | 2 ['C1', 'C5'] | 0 [] | wholly guarded (2/2) |
| store-run/static-scan key pair | 2 | 0 [] | 2 ['CS2', 'CS3'] | **wholly unguarded** (2/2) |
| call-arg permutation | 2 | 2 ['CA3', 'CA5'] | 0 [] | wholly guarded (2/2) |
| literal width | 2 | 2 ['CA7', 'CA22'] | 0 [] | wholly guarded (2/2) |
| classified-twice key pair | 2 | 0 [] | 2 ['CA9', 'CA10'] | **wholly unguarded** (2/2) |
| permutation cycles | 2 | 2 ['CA14', 'CA15'] | 0 [] | wholly guarded (2/2) |
| mcall-chain key pair | 2 | 2 ['CA21', 'CA23'] | 0 [] | wholly guarded (2/2) |
| bind .in totality | 2 | 0 [] | 2 ['B4', 'B5'] | **wholly unguarded** (2/2) |
| bind size | 2 | 1 ['B9'] | 1 ['B6'] | **guarded at 1 of 2** raise sites |
| dyninit_tu / data_tu | 2 | 0 [] | 2 ['D1', 'D2'] | **wholly unguarded** (2/2) |
| bind .gl linkage | 1 | 1 ['C3'] | 0 [] | wholly guarded (1/1) |
| data-sym key pair | 1 | 1 ['C4'] | 0 [] | wholly guarded (1/1) |
| store-run bind routing | 1 | 0 [] | 1 ['CS4'] | **wholly unguarded** (1/1) |
| census inline fence | 1 | 1 ['CS12'] | 0 [] | wholly guarded (1/1) |
| varargs | 1 | 1 ['B10'] | 0 [] | wholly guarded (1/1) |

### 4.2 The mechanism, read out of the guard that produced the first RED

**The guard tests are per-KEY witness tests, and a key with *k* raise sites
therefore contributes *k − 1* unguarded sites by construction.**

`leaf_store.rs::every_bind_gate_fires_on_a_named_input` asserts **8 witnesses
over 5 distinct keys** — one input per key, each asserting
`bind_run_ops(...) == Err(THAT_KEY)`. It is a **key-reachability** test. It says
nothing about *which* of a key's raise sites produced the key, so:

* `STORE_RUN_BIND_GROUP_SHAPE` has **four** raise sites — `leaf_store.rs:2254`,
  `:2257`, `:2285`, and `:2456` (reached through the gate at `:2455`). The single
  witness — case 5, a 4-op F2 address-valued group where `parse_simple_gpr_run`
  matches exactly three — routes through **one** of them, `:2254`. Swap the key
  there and the witness fails (`L1` RED). Swap it at any of the other three and
  **nothing anywhere notices** (`L2`, `L3`, `L9` all GREEN).

That is not a defect in how carefully those witnesses were written. It is
structural: **no per-key witness suite can distinguish a key's raise sites**, so
the guarded fraction of a *k*-site key family is bounded above by `1/k` unless
someone writes site-level witnesses. The census's whole GREEN population is
concentrated exactly where that bound bites.

**The same mechanism, one level up, explains the two rollups.** A key-swap
mutation is *only* detectable by an assertion that names the key; a gate-removal
mutation is *also* detectable by any fixture that happens to traverse the class
the gate refuses, and a threshold widening is detectable by any fixture between
the old and new bound. So coverage rises exactly with how many *independent*
mechanisms can catch the mutation — 75 % GREEN for swaps, 44 % for removals,
18 % for widenings — and that ordering is the census's most reproducible result.

### 4.3 Six families are wholly unguarded, and one is the workload's biggest key

| family | raise sites | guarded |
|---|---:|---:|
| `callee-unresolved` key family (`CS5`–`CS8`) | 4 | **0** |
| `data-def` / `bss-def` mirror (`B2`, `B3`, `B7`, `B8`) | 4 | **0** |
| `store-run` / `static-scan` key pair (`CS2`, `CS3`) | 2 | **0** |
| `classified-twice` key pair (`CA9`, `CA10`) | 2 | **0** |
| bind `.in` totality (`B4`, `B5`) | 2 | **0** |
| `dyninit_tu` / `data_tu` (`D1`, `D2`) | 2 | **0** |

**The `callee-unresolved` family is the one to read first.** All four of its
routing arms are unguarded, including `CS8`, the **default** arm
`_ => CALLEE_UNRESOLVED_TAIL` at `census.rs:1272`. That key —
`callee-unresolved-tail-call` — is the one board **#3209** measured rising to
**1,296** bodies when the multi-symbol half moved its blocker to the callee. **The
single most populous refusal key on the 878-TU workload can be swapped for a
sibling and the entire suite stays green.** `CS8` was registered **RED at 0.80**
and is one of the campaign's three registered-RED-observed-GREEN misses.

The `data-def`/`bss-def` mirror is the second worth naming: `B2`/`B7`
(comdat/init) and `B3`/`B8` (thread-local) are *mirrored* clauses on the two
resolution paths, and **neither path is guarded on either clause** — so the
mirror could silently stop being a mirror.

### 4.4 The one duplicate disagreement, and what it exposes

54 of the 63 sites were measured twice in independently provisioned worktrees.
**93 of 94 duplicate pairs agree exactly.** The one that does not is `B10`:
**RED in both runs** — the colour is robust — but with **3** failing tests in one
and **2** in the other. The extra test is
`reloc_identity::the_cells_population_is_three_functions_one_of_which_disagrees`.

Reading that test explains it, and the explanation is a finding of its own: it
begins

```rust
let rows = grade(&tc);
if rows.is_empty() {
    println!("SKIP: capture produced no graded function");
    return;                     // ← passes
}
```

**A capture that produces nothing is a silent PASS.** Under the load this
campaign ran against, that branch is reachable — which makes this test a
false-GREEN generator of exactly D6's family, *inside a test that is already
toolchain-aware*. It matters here because **`B9` is guarded by this test alone**
(RED, 1,647 / 1): a site whose only guard can silently skip is a site that can
read GREEN on a busy machine.

Bounded rather than hand-waved: the risk is confined to sites guarded *solely* by
this test, the only identified one is `B9`, and `B9` read **RED**. More generally,
**54 sites were measured twice and every GREEN among them reproduced**, which is
the positive evidence that the GREEN population is not an artifact of load.

## 5. Peer verification: the four guards `w-guards` landed last wave DO hold

**Stated plainly, as an independent finding of this lane rather than a
restatement of `w-guards`'.** The five controls were run first, from a different
session, at a different commit, in a different worktree, with recipes written
from the site text rather than from `w-guards`' patches. Four of them reproduce
`w-guards`' G1–G4 failing-test **sets and counts exactly**:

| control | site | observed | failing tests |
|---|---|---|---|
| `C1` = M1 | `calls.rs:431` arity fence | **RED 1,646 / 2** | `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` · `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` |
| `C2` = M2 | `c2-core/codegen/calls.rs:1815` backstop | **RED 1,647 / 1** | `the_data_address_setup_refuses_the_shapes_it_has_no_capture_for` (the #3199-named test) |
| `C3` = M3 | `bind.rs:891` `.gl` linkage gate | **RED 1,645 / 3** | `the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key` · `the_two_data_symbol_census_keys_are_not_interchangeable` · `the_census_key_survives_the_round_trip_into_the_reachable_ranking` |
| `C4` = M4 | `census.rs:1216/1218` data-sym key swap | **RED 1,646 / 2** | `the_data_symbol_linkage_gate_…` · `the_two_data_symbol_census_keys_are_not_interchangeable` |
| `C5` | `calls.rs:430` thunk exemption | **RED 1,642 / 6** | the thunk guard, both data-sym guards, the round-trip, and two `wr1_dyninit` decode pins |

**Controls: 5 of 5 RED. Zero control anomalies.** Prereg §2.3 registered
P(any control reads GREEN) = 0.05 and made a GREEN control a
campaign-stopping finding that outranks the census; that branch was not taken.

Two things this establishes that `w-guards`' own rung could not:

1. **The guards fire for the reason claimed, not incidentally.** `C3`'s and
   `C4`'s failing sets differ by exactly the round-trip test, and `C5` — a
   surface `w-guards` found *while building* the third guard — takes down six
   tests including both data-symbol guards, which is the interaction #3216
   predicted in advance.
2. **The probe is live.** A control that reproduces a known failing set to the
   test name is the positive check that the mutation harness can see a guarded
   site at all. Without it a table of GREENs is indistinguishable from a broken
   runner — and this campaign found exactly that failure mode in its own
   instrument (§7), so the check is not ceremonial.

Each control's colour was re-derived under the corrected rules of §7 and every
one graded **70–95 s** against real `c2` in the `census_gate` target.

## 6. Prereg scorecard

**50 hits / 14 misses over the 64 registered colours.** The headline prediction
**X = 38** against observed **30** — inside the registered 80 % interval
**[30, 46]**, at its exact lower bound.

| registration | outcome |
|---|---|
| **X = 38**, 80 % interval [30, 46] | **30 — inside, at the lower bound** |
| P(X ≥ 20) = 0.92 | **held** — 30 ≥ 20. The unguarded population *is* large; #3199's 3-of-4 was not an anomaly of four hand-picked sites |
| P(any control reads GREEN) = 0.05 | **not taken** — 5 of 5 RED |
| P(≥1 INVALID needing a recipe fix on first application) = 0.5 | **HIT** — `C3`'s `\| true` spelling was E0277 (deviations D2) |

### 6.1 The misses are not scattered — 11 of 14 are one family, in one direction

| direction | count | sites |
|---|---:|---|
| registered GREEN → observed **RED** (predicted unguarded, actually guarded) | **11** | `CA3` `CA4` `CA5` `CA7` `CA12` `CA14` `CA15` `CA20` `CA22` `B9` `BU2` |
| registered RED → observed **GREEN** (predicted guarded, actually not) | **3** | `CS8` `CS9` `D1` |

**Nine of the eleven are the `CA` call-argument block.** The prereg was
systematically pessimistic about exactly one family, and that single bias
accounts for essentially the whole 38 → 30 gap: the call-argument fences are far
better covered by the existing fixture corpus than the registration assumed,
because a `false &&` there admits shapes that real fixtures actually traverse and
the differential then grades a wrong emit.

**The three misses in the other direction are the ones that matter for the
project**, because a registered-RED-observed-GREEN is a fence someone believed
was guarded and is not: `CS8` (§4.3, the 1,296-body key), `CS9` (`census.rs:1280`,
the opt-mode gate), and `D1` (`bundle.rs:2423`, the dyninit name clause — the one
`dyninit_tu` clause of 12 that was mutated at all).

**Calibration, stated as a number rather than a vibe:** 50/64 = **78 %** of
registered colours were correct, and the error was directional, not noisy. A
future census over the dropped grammar class (§9 F3) should register the
call-argument-style families *more* guarded than intuition suggests.

## 7. The campaign's own instrument failure, found mid-run

**The registered baseline `1,648 / 0 / 42` is byte-identical with and without a
toolchain, so it cannot distinguish a run that graded against real `c2` from one
that graded nothing.** Full account and the three-layer fix:
`work/w-mutcensus/deviations.md` D6. In brief:

| run | toolchain | passed / failed / targets | `census_gate` target |
|---|---|---|---|
| session-1 baseline | present | **1,648 / 0 / 42** | **84.17 s** |
| `N0wtB` | **absent** | **1,648 / 0 / 42** | **0.00 s** |
| `N0wtC` | **absent** | **1,648 / 0 / 42** | **0.00 s** |

By design (CLAUDE.md) every toolchain-driven test prints `SKIP: toolchain absent`
**and passes**, so the totals are preserved and prereg §4.5's `targets != 42`
rule sees 42 of 42 reporting `ok`. **GREEN means "no test can fail on this
site", so in an unprovisioned worktree every site guarded only by the real-`c2`
differential reads GREEN — the error is one-directional and it inflates X, the
headline.**

It surfaced as a *contradiction between two runs of one mutant*, not by
inspection: `L4` failed
`census_gate::the_census_and_the_port_agree_about_what_is_in_class` after
171.58 s in the provisioned worktree, and passed that same test in **0.00 s** in
the sidecar. Same mutation, same commit, two different failing sets.

Fixed in three layers — worktrees provisioned via
`scripts/configure_existing_worktree.sh` (whose own hard gate is the fixture
census verdict); a **pre-flight** census probe in `run_mutants.sh` that aborts
the list rather than emitting a colour; and the **`census_gate` duration recorded
per run**, with anything under 1 s classified `INVALID`. Because the table is
*derived* from the logs by `rederive.sh`, the rule applies retroactively to every
log on disk.

**Two colours were discarded** (`CS2` read GREEN, `L4` read RED, both in
unprovisioned sidecars) and re-run from scratch. **All 8 session-1 colours
survive** re-derivation. The faulted logs are kept as `*.notoolchain*.log` and
the new rule classifies all four `INVALID` at 42 of 42 targets, which is the
check working.

> **The generalization is worth more than this lane's X.** The repo already knew
> this trap — `configure_existing_worktree.sh`'s own header says *"`cargo test`
> is green, `c2rs diff` says SKIP, and a change that mis-emits looks exactly like
> a change that is byte-exact"* — and this lane walked into it anyway, **because
> the prereg specified its probe as a pair of totals, and totals are exactly what
> the fault preserves.** A probe defined by a count cannot detect a population
> that silently left the count. That is STATUS trap 5 one level up: not a missing
> target, but a present target that measured nothing.

### 7.1 The same defect was found independently by a peer — and this is the proof the campaign's own probe was sound

Peer `w-fence163` hit this failure mode too and filed it as board **#3219**: its
registered mutants `MF1`/`MF2` read **GREEN** off a fresh `git worktree add` with
no `compilers/`, with a clean suite and the correct target count, and its
red-maker reported *"3 passed"* in **0.00 s** because cargo swallows a passing
test's `SKIP` line. **Two lanes, in one wave, walked into the same trap** — which
promotes it from this lane's incident to a property of the repo's worktree
workflow.

Because this census ran across **eight** fresh worktrees, its every GREEN is
suspect until proven otherwise. Four checks, all run at the tip:

**(1) `compilers/` present in every runner worktree.** All eight, symlinked to the
main repo. Every one was provisioned with
`scripts/configure_existing_worktree.sh`, whose own hard gate is the fixture
census verdict, and each re-verified `fixtures/cpp/w5_chain.cpp -> 4/4 functions
in class`.

**(2) A known-RED control re-run in every runner** — `C1`, whose failing set
`w-guards` pins exactly. Results in §7.2.

**(3) The executed-population check, for EVERY run rather than a sample.** This is
the layer D6 added: `run_mutants.sh` records the `census_gate` target's **duration
per run** and makes anything under 1 s `INVALID`. A skipping differential is
0.00 s; a grading one is tens of seconds. Minimum over **every** run in each
worktree:

| worktree | runs | **min** differential |
|---|---:|---:|
| `-b` | 29 | **118.71 s** |
| `-c` | 29 | **53.51 s** |
| `-d` | 29 | **91.38 s** |
| `-e` | 29 | **61.68 s** |
| `-f` | 11 | **250.18 s** |
| `-g` | 11 | **251.64 s** |
| `-h` | 10 | **142.93 s** |
| `-i` | 11 | **138.20 s** |

**No run, in any worktree, anywhere near 0.00 s.** The two runs that *were*
0.00 s are the discarded ones, and they are named in §7.

**(4) Void, not provisional.** The two colours read before provisioning
(`CS2` GREEN, `L4` RED) were **discarded and re-run from scratch**, not carried
as provisional. Their logs are **kept, not deleted** —
`CS2.notoolchain.DISCARDED.log`, `L4.notoolchain.DISCARDED.log`,
`N0wt{B,C}.notoolchain.log` — and the corrected rule classifies all four
`INVALID` at 42 of 42 targets.

**The two GREENs that stood from session 1 are sound**: `L2` (`leaf_store.rs:2257`)
and `L3` (`:2285`) were measured in the **lane checkout**, which has `compilers/`,
at differentials of **76.63 s** and **87.53 s**.

### 7.2 The control re-validation

`C1` (`calls.rs:431`, `syms > 1` → `syms > 2`) re-run in all eight runner
worktrees after the campaign. `C1` is the right probe for this because
`w-guards` pins its failing set exactly, so a runner whose captures were skipping
cannot reproduce it.

| runner worktree | colour | passed / failed | differential | failing tests |
|---|---|---|---:|---|
| `-b` | **RED** | 1,646 / 2 | 641.75 s | the G1 pair |
| `-c` | **RED** | 1,646 / 2 | 643.14 s | the G1 pair |
| `-d` | **RED** | 1,646 / 2 | 632.33 s | the G1 pair |
| `-e` | **RED** | 1,646 / 2 | 632.37 s | the G1 pair |
| `-f` | **RED** | 1,646 / 2 | 641.43 s | the G1 pair |
| `-g` | **RED** | 1,646 / 2 | 642.00 s | the G1 pair |
| `-h` | **RED** | 1,646 / 2 | 646.10 s | the G1 pair |
| `-i` | **RED** | 1,646 / 2 | 643.15 s | the G1 pair |

"the G1 pair" is, in every one of the eight,
`the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` and
`the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` — identical
sets, identical counts.

**Eight for eight, with the failing set pinned to the test name.** A worktree
whose captures were skipping could not produce this: `C1` widens the
call-argument arity fence, and both tests that catch it are capture-driven. So
**no runner's colours are void, and the GREEN population is not an artifact of a
narrowed probe.** Note also that the differential ran **632–646 s** in these runs
against **84.17 s** uncontended — the probe stayed sound under the worst load the
campaign saw, which is the condition under which a silent narrowing would have
been most likely.

**This is the check that would have caught #3219 immediately**, and it is worth
stating as the general lesson: **a mutation campaign should carry at least one
control whose failing set is pinned by NAME, re-run in every execution
environment it uses.** A control pinned only by *count* would have passed in an
unprovisioned worktree the moment the count happened to match; a control pinned
by name cannot.

## 8. Gate evidence

All figures measured in this worktree. **This lane lands no `crates/`,
`fixtures/` or `scripts/` change at all** — every source edit was an applied and
reverted mutant — so it is a revert-everything lane and the graded-tree identity
applies to it in full.

| check | result |
|---|---|
| `git diff <merge-base>..HEAD -- crates fixtures scripts` | **EMPTY** — verified after every revert and at the tip. The merge-base is `7e541a54`; the lane touches only `docs/rungs/` and `work/`. **Read against the merge-base, not against `master`**: master has since advanced to `260838d6` (peer `w-npos` merged, workload `match` 25 → 26), so a bare `master..HEAD` now shows `w-npos`'s *additions* as deletions — that is this branch being deliberately un-rebased, not a graded change by this lane. The rebase is held at the coordinator's instruction |
| graded tree identical at both ends | **YES** — established by the empty diff above over exactly the paths `gate.sh` content-hashes, and re-verified clean in all **eight** sidecar worktrees after the campaign |
| 878-TU workload scan | `match` **25** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **845** · `fnbyte-exact` **35,734** |
| anchored `gap-metric` keys | **394**, **0 deltas** against the briefed base at `3835469c` |
| `cargo test --workspace --release --no-fail-fast` | **1,648 passed / 0 failed / 42 targets**, exit 0 |
| `scripts/gate.sh --jobs 4 --require-graded` | **PASS (HATCH-RED REFUSED)** — **18/18** lanes ran and **every one graded a corpus**; **6,858** fixture-verdicts; sweep **19,460 of 19,556** generated cases graded; cross **90,424 of 90,812** cells graded, **0 mismatch** |
| `scripts/debug_lane.sh` | **18 lanes ran, 0 failed, 0 panics, 0 mismatch** — e.g. `/Ox /EHsc /GR` graded 381/381 match 150 |
| **graded tree** | **`00aeaabe2b63`**, **731 files** under `crates fixtures scripts` |
| `scripts/board_audit.sh` | all-zero: 0 cited-but-not-on-board · 0 unresolved anchors · 0 raw line anchors · 0 rows-behind-prose · 0 duplicate row numbers |
| `crates/c2-harness/tests/rung_registry.rs` | **2/2** (inside the suite row above) |
| `docs/rungs/INDEX.md` | regenerated by `scripts/gen_rung_index.sh`, never hand-edited |

**The graded-tree row is the "identical at both ends" evidence, and it is stronger
than a re-run would have been.** `gate.sh` content-hashes exactly
`crates fixtures scripts`; `git diff master..HEAD` over those paths is **empty**,
so the two ends are provably the same bytes rather than the same summary. And the
hash **`00aeaabe2b63` over 731 files** is digit-for-digit the tip hash
`w-guards` recorded, as are **6,858** fixture-verdicts, **19,556** sweep cases and
**90,424 / 90,812** cross cells — this lane's gate reproduces master's gate
exactly, which is what a revert-everything lane owes.

**The `N0` control is the strongest row here and it is worth reading as
evidence rather than ceremony.** The registered clean-tree baseline
**1,648 / 0 / 42** was reproduced **six times**: once at `3835469c` in session 1,
twice as trailing baselines after the mutant lists (`WTB` differential 118.71 s,
`WTC` 156.16 s), and in the leading baselines of the later sidecars (`WTD`
371.79 s, `WTE` 372.65 s) — **every one with a live real-`c2` differential**. It
was *also* reproduced twice with the differential at **0.00 s**, and twice more
with a live source mutation applied, which is the whole content of §7.

## 9. Found and not taken

Ranked by what the next lane should read first.

## F1 — The suite row every rung quotes has NO `--require-graded`, and `gate.sh` has one *because of exactly this failure*

`scripts/gate.sh` grew `--require-graded` / `C2RS_GATE_REQUIRE_GRADED=1` for one
reason, stated in its own header: *"the thirteenth [absence-read-as-success]
being the one `--require-graded` above was written for."* Its design is quoted
because the same design applies here — *"a POSITIVE check on a COUNT, never an
enumeration of the ways a run can be empty"*, and the demand belongs to the
**caller**, so the portable lane (entitled to be empty) is unaffected.

**Nothing equivalent exists for `cargo test --workspace`.** Measured:
`grep -rn 'REQUIRE_TOOLCHAIN\|REQUIRE_GRADED' crates scripts` returns **8 hits,
all in `scripts/gate.sh`, none under `crates/`.** Yet the workspace suite row is
quoted as evidence in essentially every rung doc in `docs/rungs/`, and §7 of this
rung shows what that row is worth in an unprovisioned worktree: **1,648 / 0 / 42
with the differential at 84.17 s and 1,648 / 0 / 42 with it at 0.00 s.**

The fix is one function and needs no new dependency: a test that reads
`C2RS_REQUIRE_TOOLCHAIN` and **fails** when it is set and `Toolchain::locate()`
is `None`. Caller states its expectation; default behaviour does not move; the
portable lane still passes.

**NOT TAKEN, and the reason is structural rather than a shortage of time.** It
lands a test under `crates/`, and this lane's success criterion is a
**required-zero byte delta** on `crates fixtures scripts`. Those are the same
commit's two halves and cannot both happen — **which is precisely the conflict
`#3217` recorded one wave ago** for the missing `cflow_emitted_modeled_keys`
printer (*"a zero-delta rung and a new printed key are the same commit's two
halves"*). **That is now twice in two waves that the instrument a lane discovered
it needed could not be landed by the lane that discovered it.** The pattern, not
the item, is the finding: a characterization lane is structurally the wrong unit
for shipping the check it just proved necessary, and the repo has no unit that
is. Note this one is **cheaper than #3217's**: it adds no `gap-metric` key, so
the anchored-key count stays at 394 and only the byte-delta rule blocks it.

## F2 — The guard tests are per-KEY witness tests, so they pin ONE raise site per key and every sibling site is invisible

This is the mechanism behind the census's headline shape, and it is readable
directly in the guard that produced this lane's first RED.
`leaf_store.rs::every_bind_gate_fires_on_a_named_input` asserts **8 witnesses
over 5 distinct keys** — one input per key, each asserting
`bind_run_ops(...) == Err(THAT_KEY)`. It is a **key-reachability** test. It says
nothing about *which* of a key's raise sites produced it, so:

* `STORE_RUN_BIND_GROUP_SHAPE` has **four** raise sites —
  `leaf_store.rs:2254`, `:2257`, `:2285`, `:2456` (the last reached through the
  gate at `:2455`). The single witness (case 5, a 4-op F2 address-valued group
  where `parse_simple_gpr_run` matches exactly three) routes through **one** of
  them. Swap the key at that one and the witness fails; swap it at any sibling
  and nothing anywhere notices.

**Generalized:** a per-key witness suite guards `min(1, sites)` of each key's
raise sites, so a key with *k* raise sites contributes *k − 1* unguarded sites by
construction — no matter how carefully the witnesses were written. The families
this lane counted are exactly the multi-raise-site keys.

The fix is a helper that asserts **site** reachability rather than **key**
reachability — e.g. each raise site carries a distinct `#[cfg(test)]`-visible
discriminant, or the witness table is required to cover each site. Sizing for
one file: `leaf_store.rs`'s **9** enumerated sites sit under **5** keys.
NOT TAKEN for the same zero-delta reason as F1.

## F3 — The 1,227-site grammar class is unmeasured, and a SAMPLED census over it is a lane

Published in §2.1 with its count. The reason it is a separate lane is not only
budget (≈ 5 days serial) but that it is a **different guard class**: the key is
generated *from the blocking byte*, so a key-swap mutation does not exist, and a
removal mutation merely moves the parse to the next blocking byte. The right
instrument is a **stride sample** with a registered colour per sampled site —
`gate.sh`'s own sweep argument, *"the sample is a STRIDE across the sorted case
list, not a prefix"*, applies unchanged: a prefix over `blk(` sites would sit
entirely inside one parser.

## F4 — Nothing re-runs this census, so X/N goes stale on the next landed fence — and one already landed during the campaign

`enumerate.sh` and `mutants.py` live under `work/w-mutcensus/`. They are tracked,
so the census is *reproducible*, but nothing *re-runs* them, and §2.2 shows the
frame going stale inside this lane's own wall-clock: peer `w-fence163`'s
`d28326b4` adds a 20th fence-key constant and new deciding gates.

The cheap standing version is **not** a re-run of the campaign (56 suite runs is
not a gate row). It is a **count**: `enumerate.sh` already prints one line per
E1–E3 site, so a gate row that compares that count against a checked-in
expectation and fails when a fence lands without the census being re-scored would
turn "X/N is a fact about a commit" into a maintained invariant. **NOT TAKEN
twice over:** the byte-delta rule (F1), and a live seam — a separate lane is
wiring `debug_lane.sh` into `scripts/gate.sh` right now and this lane was
instructed not to edit either script.

## F5 — `STORE_RUN_BIND_CALL_TAIL_RETIRED` is a fence key with zero live raise sites

Enumerated and published in §2.1. Test-only since #1212's correction, so no
mutant is possible: there is nothing to mutate. Worth a row of its own because
the *inverse* of this lane's question — **a key with no fence** — is as invisible
to every instrument as a fence with no test, and this is the only one the frame
found. Whether it should be deleted or re-armed is a decision, not a measurement,
and it is not this lane's to make.

## F6 — A differential test that silently PASSES when its capture yields nothing

`reloc_identity::the_cells_population_is_three_functions_one_of_which_disagrees`
returns early with `println!("SKIP: capture produced no graded function")` when
`grade()` comes back empty (§4.4). It is the only guard on `B9`, and it is the
sole source of the campaign's one duplicate disagreement. This is D6's family
appearing *inside* a toolchain-aware test: the absence of a graded population
reads as success. **NOT TAKEN** for the byte-delta reason of F1 — the fix is an
assertion under `crates/`.

## F7 — The enumeration went stale TWICE during this lane's own wall-clock

§2.2 records peer `w-fence163`'s `d28326b4` adding a 20th fence-key constant.
Since then peer **`w-npos` converted (`match` 25 → 26)** and lands ahead of this
lane, touching `c2-il`'s `bundle.rs`, `gl.rs`, `diag.rs`, `func/mod.rs` and
`lib.rs` — four of the five files this census enumerates sites in.

**This lane did not re-enumerate to absorb either, and must not**: the frame was
frozen at `3835469c` before the first mutant ran. Both are recorded as sites the
frame necessarily misses. **The point is no longer hypothetical — it is a
two-instance finding.** X/N is a fact about a commit, and this commit's X/N was
already stale before it was published. F4's standing count is the only thing that
would make it a maintained invariant rather than a snapshot.
