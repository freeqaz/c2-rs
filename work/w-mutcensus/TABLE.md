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
