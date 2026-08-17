#!/bin/sh
# w-mutcensus enumeration rule — reproducible fence-site census over crates/c2-il/src.
# Run from the repo root. Output: one line per candidate site, tab-separated:
#   class<TAB>file:line<TAB>text
# Classes:
#   E1  call-argument fences: `refuse("<key>")` raise sites (calls.rs family)
#   E2  named-key fence raises: a non-test, non-comment line that RAISES one of the
#       19 fence-key constants (Block/Err/match-arm), excluding const defs, imports,
#       doc comments, and #[cfg(test)] regions
#   E3  post-parse gate raises: `Block::at_end(` sites
#   E5x grammar fail-closed sites (counted, EXCLUDED from mutation): blk( / blk_type( /
#       Block::refuse( — the parser's per-byte refusals
# The predicate gates that DECIDE whether each raise fires (arity thresholds,
# linkage .contains, separator sets, opt_word_mode, thunk exemption) are added by
# reading the functions the E1-E3 sites live in; each is listed in the prereg with
# its own file:line. That reading step is bounded: only functions containing an
# E1-E3 site, plus the resolver/gate functions they call (resolve, resolve_data,
# resolve_data_def, opt_word_mode, gl_extern_data_names, NAME_SEPARATORS users).
SRC=crates/c2-il/src
CONSTS='OPT_MODE|PTR_WALK_LOOP_NOT_O1|PTR_WALK_CHAIN_LOOP_NOT_O1|CALLEE_UNRESOLVED_TAIL|CALLEE_DEFINED_IN_TU|STORE_RUN_CALL_NO_CARRIER|STATIC_SCAN_LOOP_OBJECT|STORE_RUN_BIND_NO_CARRIER|STORE_RUN_BIND_MIXED_KIND|STORE_RUN_BIND_ADDR_PRODUCER|STORE_RUN_BIND_MULTI_PRODUCER|STORE_RUN_BIND_SYMBOL_CROSSINGS|STORE_RUN_BIND_CALL_TAIL_RETIRED|STORE_RUN_BIND_GROUP_SHAPE|CALLEE_UNRESOLVED_DTOR|CALLEE_UNRESOLVED_FRAMED|CALLEE_UNRESOLVED_SEQ|DATA_SYM_UNRESOLVED|DATA_SYM_LINKAGE'
grep -rn 'refuse("' $SRC --include='*.rs' | grep -v '^\s*//' | sed 's/^/E1\t/' | sed 's/:\([0-9]*\):/:\1\t/'
grep -rnE "\b($CONSTS)\b" $SRC --include='*.rs' \
  | grep -vE 'pub\(crate\) const|^\S+:[0-9]+: *//|use (super|crate)|^\S+:[0-9]+:\s*\*' \
  | grep -vE 'tests?\.rs|mod tests' \
  | grep -E 'Err\(|Block|=> *[A-Z_]+,?$|Some\(|\b(if|return|unwrap_or)\b' \
  | sed 's/^/E2\t/' | sed 's/:\([0-9]*\):/:\1\t/'
grep -rn 'Block::at_end(' $SRC --include='*.rs' | grep -v '^\s*//' | sed 's/^/E3\t/' | sed 's/:\([0-9]*\):/:\1\t/'
echo "E5-counts (excluded from mutation, grammar fail-closed):"
printf 'E5 blk( sites: %s\n'        "$(grep -rn -F 'blk(' $SRC --include='*.rs' | wc -l)"
printf 'E5 blk_type( sites: %s\n'   "$(grep -rn -F 'blk_type(' $SRC --include='*.rs' | wc -l)"
printf 'E5 Block::refuse( sites: %s\n' "$(grep -rn -F 'Block::refuse(' $SRC --include='*.rs' | wc -l)"
