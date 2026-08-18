#!/bin/bash
# mutants.sh — the colours registered in the prereg §6, run against the shipped
# decode. One mutation at a time, restored after each.
#
# #3219 / #3231: a colour taken in an environment whose EXECUTED-TEST COUNT and
# DIFFERENTIAL DURATION were not asserted is VOID, not provisional. Both are
# printed for every run below, and `C2RS_REQUIRE_TOOLCHAIN=1` is set, so an
# unprovisioned worktree fails hard instead of reading GREEN in 7 seconds.
set -u
cd "$(dirname "$0")/../.." || exit 1
F=crates/c2-il/src/func/gl.rs
OUT=work/w-glattrs/mutants
mkdir -p "$OUT"
cp "$F" "$OUT/gl.rs.orig"

run() {
    local id="$1"
    local log="$OUT/$id.log"
    ( time C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast ) \
        > "$log" 2>&1
    local rc=$?
    local passed failed targets skips census
    passed=$(grep -oE '^test result: (ok|FAILED)\. [0-9]+ passed; [0-9]+ failed' "$log" \
             | awk '{p+=$4} END {print p+0}')
    failed=$(grep -oE '^test result: (ok|FAILED)\. [0-9]+ passed; [0-9]+ failed' "$log" \
             | awk '{f+=$6} END {print f+0}')
    targets=$(grep -c '^test result:' "$log")
    skips=$(grep -c 'SKIP: toolchain absent' "$log")
    # The DIFFERENTIAL's duration, asserted rather than the exit code (#3231):
    # a run with no toolchain is byte-identical in every printed count and
    # differs only here.
    census=$(awk '/Running tests.census_gate/{f=1} f&&/^test result:/{print $NF; exit}' "$log")
    printf '%-6s rc=%-3s passed=%-5s failed=%-3s targets=%-3s SKIP=%-3s census_gate=%s  %s\n' \
        "$id" "$rc" "$passed" "$failed" "$targets" "$skips" "${census:-NONE}" \
        "$( [ "$failed" -eq 0 ] && [ "$rc" -eq 0 ] && echo GREEN || echo RED )"
}

restore() { cp "$OUT/gl.rs.orig" "$F"; }

echo "== CONTROL — the shipped tree"
run control

echo "== M1 — the escape consumes 5 bytes (SRCPOS's width, the wrong reader)"
sed -i 's/^const GL_SIZE_ESCAPE_PAYLOAD: usize = 2;$/const GL_SIZE_ESCAPE_PAYLOAD: usize = 4;/' "$F"
grep -q 'GL_SIZE_ESCAPE_PAYLOAD: usize = 4;' "$F" || { echo "M1 NOT APPLIED"; exit 1; }
run M1
restore

echo "== M3 — 0x81..=0xff decoded as a 3-byte escape instead of refused"
python3 - <<'PY'
p = "crates/c2-il/src/func/gl.rs"
s = open(p).read()
old = """        match *gl.get(q)? {
            0x80 => q += 1 + GL_SIZE_ESCAPE_PAYLOAD,
            b if b < 0x80 => q += 1,
            _ => return None,
        }"""
new = """        match *gl.get(q)? {
            b if b >= 0x80 => q += 1 + GL_SIZE_ESCAPE_PAYLOAD,
            _ => q += 1,
        }"""
assert old in s
open(p, "w").write(s.replace(old, new, 1))
PY
grep -q 'b if b >= 0x80' "$F" || { echo "M3 NOT APPLIED"; exit 1; }
run M3
restore

echo "== M4 — the escape arm deleted, i.e. the incumbent restored"
python3 - <<'PY'
p = "crates/c2-il/src/func/gl.rs"
s = open(p).read()
old = """        match *gl.get(q)? {
            0x80 => q += 1 + GL_SIZE_ESCAPE_PAYLOAD,
            b if b < 0x80 => q += 1,
            _ => return None,
        }"""
new = """        if *gl.get(q)? >= 0x80 {
            return None;
        }
        q += 1;
        let _ = GL_SIZE_ESCAPE_PAYLOAD;"""
assert old in s
open(p, "w").write(s.replace(old, new, 1))
PY
grep -q 'if \*gl.get(q)? >= 0x80' "$F" || { echo "M4 NOT APPLIED"; exit 1; }
run M4
restore

echo "== M2 — big-endian payload: NOT EXPRESSIBLE."
echo "   The shipped reader never READS the payload; it steps over it"
echo "   (\`q += 1 + GL_SIZE_ESCAPE_PAYLOAD\`). There is no byte order in"
echo "   crates/ to mutate. Registered GREEN, and the reason is stronger than"
echo "   'the tests cannot see it': the claim lives in GRID-A and the docs."

git diff --stat -- "$F"
echo "(empty diff above == the tree is restored)"
