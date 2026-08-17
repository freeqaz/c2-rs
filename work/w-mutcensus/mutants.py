#!/usr/bin/env python3
"""w-mutcensus mutation patcher — one registered mutation per fence site.

Every spec is (file, 1-based line, old substring, new substring). `apply`
asserts the target line contains `old` exactly once and rewrites it; `revert`
asserts the line contains `new` and restores `old`. A spec that does not match
ABORTS (non-zero) — a vacuous patch must fail loudly (w-guards' rule).

Line numbers and text are pinned at master 3835469c. Any drift = abort.

Usage: mutants.py apply <id> | revert <id> | check <id> | list
"""
import sys, os

IL = "crates/c2-il/src/func"
CALLS = f"{IL}/body/shapes/calls.rs"
LEAF = f"{IL}/body/shapes/leaf_store.rs"
CENSUS = f"{IL}/census.rs"
BIND = f"{IL}/bind.rs"
GL = f"{IL}/gl.rs"
BUNDLE = f"{IL}/bundle.rs"
CORE_CALLS = "crates/c2-core/src/codegen/calls.rs"

SPECS = {
    # ---- controls (w-guards' four surfaces + M2 backstop), registered RED ----
    "C1": [(CALLS, 431, "if syms > 1 && !two_sym_thunk {",
                        "if syms > 2 && !two_sym_thunk {")],
    "C2": [(CORE_CALLS, 1815, ".count() != 1 {", ".count() > 2 {")],
    "C3": [(BIND, 890, ".contains(&name)", ".contains(&name) | true")],
    "C4": [(CENSUS, 1216, "DATA_SYM_UNRESOLVED", "DATA_SYM_LINKAGE"),
           (CENSUS, 1218, "DATA_SYM_LINKAGE", "DATA_SYM_UNRESOLVED")],
    "C5": [(CALLS, 430, "let two_sym_thunk = syms == 2 &&",
                        "let two_sym_thunk = false && syms == 2 &&")],
    # ---- census.rs post-parse cluster ----
    "CS2": [(CENSUS, 1242, '"store-run-call" => STORE_RUN_CALL_NO_CARRIER,',
                           '"store-run-call" => STATIC_SCAN_LOOP_OBJECT,')],
    "CS3": [(CENSUS, 1245, '"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,',
                           '"static-scan-loop" => STORE_RUN_CALL_NO_CARRIER,')],
    "CS4": [(CENSUS, 1263, "bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)",
                           "STORE_RUN_BIND_NO_CARRIER")],
    "CS5": [(CENSUS, 1265, '"framed-call" => CALLEE_UNRESOLVED_FRAMED,',
                           '"framed-call" => CALLEE_UNRESOLVED_TAIL,')],
    "CS6": [(CENSUS, 1267, "CALLEE_UNRESOLVED_SEQ", "CALLEE_UNRESOLVED_TAIL")],
    "CS7": [(CENSUS, 1270, "CALLEE_UNRESOLVED_DTOR", "CALLEE_UNRESOLVED_TAIL")],
    "CS8": [(CENSUS, 1272, "_ => CALLEE_UNRESOLVED_TAIL,",
                           "_ => CALLEE_UNRESOLVED_FRAMED,")],
    "CS9": [(CENSUS, 1280, "Some(f) if opt_word_mode(opt_word).is_none() => {",
                           "Some(f) if false && opt_word_mode(opt_word).is_none() => {")],
    "CS10": [(CENSUS, 1294, "if f.ptr_walk_loop.is_some()",
                            "if false && f.ptr_walk_loop.is_some()")],
    "CS11": [(CENSUS, 1306, "if f.ptr_walk_chain_loop.is_some()",
                            "if false && f.ptr_walk_chain_loop.is_some()")],
    "CS12": [(CENSUS, 1358, "if callee_defined_here(&f, defined).is_some()",
                            "if false && callee_defined_here(&f, defined).is_some()")],
    # ---- calls.rs call-argument fence family ----
    "CA2": [(CALLS, 434, "> MAX_REGISTER_FORMALS {", "> MAX_REGISTER_FORMALS + 9 {")],
    "CA3": [(CALLS, 442, "if !in_place {", "if false && !in_place {")],
    "CA4": [(CALLS, 529, "> MAX_REGISTER_FORMALS {", "> MAX_REGISTER_FORMALS + 9 {")],
    "CA5": [(CALLS, 593, "if !in_place &&", "if false && !in_place &&")],
    "CA6": [(CALLS, 693, 'refuse("call-arg-nonformal")', 'refuse("call-arg-computed")')],
    "CA7": [(CALLS, 699, "if !(LI_IMM_MIN..=LI_IMM_MAX).contains(k) {",
                         "if false && !(LI_IMM_MIN..=LI_IMM_MAX).contains(k) {")],
    "CA8": [(CALLS, 710, 'refuse("call-arg-computed")', 'refuse("call-arg-nonformal")')],
    "CA9": [(CALLS, 732, 'refuse("call-arg-lit-classified-twice")',
                         'refuse("call-arg-sym-classified-twice")')],
    "CA10": [(CALLS, 736, 'refuse("call-arg-sym-classified-twice")',
                          'refuse("call-arg-lit-classified-twice")')],
    "CA11": [(CALLS, 747, "if arg_sources.iter().any(",
                          "if false && arg_sources.iter().any(")],
    "CA12": [(CALLS, 759, "if arg_sources[..i].contains(s) {",
                          "if false && arg_sources[..i].contains(s) {")],
    "CA13": [(CALLS, 772, 'refuse("call-arg-source-out-of-slots")',
                          'refuse("call-arg-outer-formal")')],
    "CA14": [(CALLS, 774, "if cycles > 1 {", "if cycles > 9 {")],
    "CA15": [(CALLS, 780, "> MAX_VERIFIED_PERM_CYCLE {", "> MAX_VERIFIED_PERM_CYCLE + 9 {")],
    "CA16": [(CALLS, 792, "if has_repeated_leaf(&arg_ops) {",
                          "if false && has_repeated_leaf(&arg_ops) {")],
    "CA17": [(CALLS, 800, "if n_loads > 1 &&", "if false && n_loads > 1 &&")],
    "CA18": [(CALLS, 803, "if !additive_chain_canonical(&arg_ops) {",
                          "if false && !additive_chain_canonical(&arg_ops) {")],
    "CA19": [(CALLS, 806, "if !arg_loads_are_formals(&arg_ops, &params) {",
                          "if false && !arg_loads_are_formals(&arg_ops, &params) {")],
    "CA20": [(CALLS, 868, "> MAX_REGISTER_FORMALS {", "> MAX_REGISTER_FORMALS + 9 {")],
    "CA21": [(CALLS, 878, 'refuse("mcall-chain-link-arg-nonformal")',
                          'refuse("mcall-chain-link-arg-computed")')],
    "CA22": [(CALLS, 883, "if !(-0x8000..=0x7FFF).contains(k) {",
                          "if false && !(-0x8000..=0x7FFF).contains(k) {")],
    "CA23": [(CALLS, 893, 'refuse("mcall-chain-link-arg-computed")',
                          'refuse("mcall-chain-link-arg-nonformal")')],
    # ---- bind.rs resolution gates ----
    "B2": [(BIND, 929, "if !o.comdat || !o.initialized {",
                       "if false && (!o.comdat || !o.initialized) {")],
    "B3": [(BIND, 932, "if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {",
                       "if false && o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {")],
    "B4": [(BIND, 939, "if init.accepted + init.residue.len() != init.records {",
                       "if false && init.accepted + init.residue.len() != init.records {")],
    "B5": [(BIND, 942, "if !init.refs.get(&tok)",
                       "if false && !init.refs.get(&tok)")],
    "B6": [(BIND, 946, "if bytes.len() != o.size as usize {",
                       "if false && bytes.len() != o.size as usize {")],
    "B7": [(BIND, 985, "if o.comdat || o.initialized {",
                       "if false && (o.comdat || o.initialized) {")],
    "B8": [(BIND, 988, "if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {",
                       "if false && o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {")],
    "B9": [(BIND, 991, "if o.size == 0 {", "if false && o.size == 0 {")],
    "B10": [(BIND, 862, "self.paired && self.names",
                        "false && self.paired && self.names")],
    # ---- gl.rs name/linkage fences ----
    "G1": [(GL, 2188, "== Some(LINKAGE_UNDEF_EXTERN);",
                      "== Some(LINKAGE_UNDEF_EXTERN) || true;")],
    "G2": [(GL, 2198, "out.retain(|n| !bad.contains(n));",
                      "out.retain(|n| { let _ = bad.contains(n); true });")],
    "G3": [(GL, 1085, "const NAME_SEPARATORS: [u8; 2] = [0x00, 0x26];",
                      "const NAME_SEPARATORS: [u8; 2] = [0x00, 0x00];")],
    # ---- bundle.rs TU-level gates ----
    "BU1": [(BUNDLE, 1694, "_ => None,", "_ => Some(OptWordMode::Ox),")],
    "BU2": [(BUNDLE, 1919, "if !drectve_is_boilerplate(gl) {",
                           "if false && !drectve_is_boilerplate(gl) {")],
    "BU3": [(BUNDLE, 1940, "return if find_subslice(ex, &LO_MARKER).is_none() {",
                           "return if find_subslice(ex, &LO_MARKER).is_none() || true {")],
    "D1": [(BUNDLE, 2423, "if !is_dynamic_initializer_name(&thunk_name) {",
                          "if false && !is_dynamic_initializer_name(&thunk_name) {")],
    "D2": [(BUNDLE, 2887, "if init.accepted + init.residue.len() != init.records {",
                          "if false && init.accepted + init.residue.len() != init.records {")],
    # ---- leaf_store.rs bind_run_ops fences ----
    "L1": [(LEAF, 2254, "return Err(STORE_RUN_BIND_GROUP_SHAPE);",
                        "return Err(STORE_RUN_BIND_MULTI_PRODUCER);")],
    "L2": [(LEAF, 2257, "return Err(STORE_RUN_BIND_GROUP_SHAPE);",
                        "return Err(STORE_RUN_BIND_MULTI_PRODUCER);")],
    "L3": [(LEAF, 2285, "_ => return Err(STORE_RUN_BIND_GROUP_SHAPE),",
                        "_ => return Err(STORE_RUN_BIND_MULTI_PRODUCER),")],
    "L4": [(LEAF, 2370, "if !served {", "if false && !served {")],
    "L5": [(LEAF, 2374, "|| addrs[0].1 == 0 {", "|| addrs[0].1 == i32::MIN {")],
    "L6": [(LEAF, 2390, "if lits.len() > 1 {", "if lits.len() > 9 {")],
    "L7": [(LEAF, 2399, "if (!lits.is_empty()", "if false && (!lits.is_empty()")],
    "L8": [(LEAF, 2402, "> MAX_SYMBOL_CROSSINGS {", "> MAX_SYMBOL_CROSSINGS + 9 {")],
    "L9": [(LEAF, 2455, "if !matches!(b, IlOp::Load(_)) {",
                        "if false && !matches!(b, IlOp::Load(_)) {")],
}


def edit(path, line, frm, to):
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    if line > len(lines):
        sys.exit(f"ABORT: {path}:{line} beyond EOF ({len(lines)} lines)")
    text = lines[line - 1]
    n = text.count(frm)
    if n != 1:
        sys.exit(f"ABORT: {path}:{line} contains {n} occurrences of {frm!r} "
                 f"(need exactly 1). Line is: {text!r}")
    lines[line - 1] = text.replace(frm, to)
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)
    print(f"  {path}:{line}: {frm!r} -> {to!r}")


def main():
    if len(sys.argv) == 2 and sys.argv[1] == "list":
        for k in SPECS:
            print(k)
        return
    if len(sys.argv) != 3 or sys.argv[1] not in ("apply", "revert", "check"):
        sys.exit(__doc__)
    verb, mid = sys.argv[1], sys.argv[2]
    if mid not in SPECS:
        sys.exit(f"ABORT: unknown mutant id {mid!r}")
    for (path, line, old, new) in SPECS[mid]:
        if not os.path.exists(path):
            sys.exit(f"ABORT: {path} not found (run from repo root)")
        if verb == "apply":
            edit(path, line, old, new)
        elif verb == "revert":
            edit(path, line, new, old)
        else:  # check — verify pristine
            with open(path, encoding="utf-8") as f:
                text = f.readlines()[line - 1]
            if text.count(old) != 1:
                sys.exit(f"ABORT: {path}:{line} not pristine: {text!r}")
    if verb == "check":
        print(f"{mid}: pristine")


if __name__ == "__main__":
    main()
