#!/bin/bash
#
# counterfactual.sh — prove the two halves of the PORT_WRITER_SECTIONS guard by
# BREAKING the tree in each of the two ways that inflate factor C, and showing
# which test catches which.
#
#   BREAK 1  a name in the constant with no `Section` literal behind it
#            -> caught by `the_writer_vocabulary_is_every_section_name_…`
#               (this is `w-rdata` §5's measurement, re-run here as a control)
#   BREAK 2  a `Section { name: ".rdata$r" }` literal in an emitter that NOTHING
#            CALLS, plus the constant
#            -> the vocabulary test goes GREEN. Board #301's hole. Caught only by
#               `every_production_emitter_has_a_lib_rs_caller`, added by w-rtti.
#
# Refuses to start on a dirty `crates/`, so the final restore check cannot be
# vacuous, and asserts `git status --porcelain crates/` is empty at the end.
set -uo pipefail
cd "$(dirname "$0")/../.."

[ -z "$(git status --porcelain crates/)" ] || {
    echo "REFUSING: crates/ is dirty before the counterfactual"
    exit 1
}

FN=crates/c2-core/src/coff/function.rs

run() {
    # $1 = label. Prints one PASS/FAIL line per test, from a COUNT.
    for t in the_writer_vocabulary_is_every_section_name_this_file_emits \
             every_production_emitter_has_a_lib_rs_caller; do
        out="$(cargo test --release -q -p c2-core -- --exact "coff::tests::tests::$t" 2>&1)"
        if echo "$out" | grep -q "^test result: ok. 1 passed"; then
            echo "  $1  $t  ok"
        else
            echo "  $1  $t  FAILED"
        fi
    done
}

echo "### BASE"
run BASE

echo "### BREAK 1 — the constant only, no Section literal"
python3 - "$FN" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
a = 'pub const PORT_WRITER_SECTIONS: [&str; 10] = [\n    ".drectve",'
b = 'pub const PORT_WRITER_SECTIONS: [&str; 11] = [\n    ".rdata$r",\n    ".drectve",'
assert a in s
open(p, "w").write(s.replace(a, b, 1))
PY
run BREAK1
git checkout -- "$FN"

echo "### BREAK 2 — the constant PLUS an uncalled emitter that builds the Section"
python3 - "$FN" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
a = 'pub const PORT_WRITER_SECTIONS: [&str; 10] = [\n    ".drectve",'
b = 'pub const PORT_WRITER_SECTIONS: [&str; 11] = [\n    ".rdata$r",\n    ".drectve",'
assert a in s
s = s.replace(a, b, 1)
s += '''
/// COUNTERFACTUAL ONLY — an emitter with a real `Section` literal and no caller.
pub fn emit_rtti_obj_counterfactual(obj_name: &str) -> Vec<u8> {
    let mut sections = shell_sections(obj_name);
    sections.push(Section {
        name: ".rdata$r",
        characteristics: 0x4030_1040,
        raw: std::borrow::Cow::Borrowed(&[]),
        checksum: 0,
        selection: 2,
        assoc: 0,
        uninit_size: None,
    });
    Vec::new()
}
'''
open(p, "w").write(s)
PY
run BREAK2
git checkout -- "$FN"

echo "### RESTORE"
echo "  git status --porcelain crates/ = $(git status --porcelain crates/ | wc -l) path(s)"
run RESTORE
