#!/usr/bin/env python3
"""`w-witness7` — the probe/mutant patcher.

EXACT STRING replacement with a **uniqueness assertion**, never by line number:
`w-mutcensus`' own enumeration went stale twice inside one lane's wall clock,
and `w-deadsites` re-located every row by text for the same reason.

    patch.py list                 print the table
    patch.py apply ID [ID ...]    apply the named patches
    patch.py revert               git checkout every file the table touches
"""

import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
S = "crates/c2-il/src/func/"

CALLS = S + "body/shapes/calls.rs"
MCMP = S + "body/shapes/mcall_cmp.rs"
MTAIL = S + "body/shapes/mcall_tail.rs"
CENSUS = S + "census.rs"
BIND = S + "bind.rs"

# id -> (path, old, new)
P = {}


def pat(pid, path, old, new):
    P[pid] = (path, old, new)


# ---------------------------------------------------------------------------
# SITE IDENTIFICATION — the five raise sites of `call-arg-nonformal`, each
# rekeyed to a UNIQUE sentinel so a census run says which site produced the key.
# Applied together, as one patch: the whole point is that the five answers are
# read off one run and cannot be confused with each other.
# ---------------------------------------------------------------------------
pat("SID1", CALLS,
    'None => return Err(refuse("call-arg-nonformal")),',
    'None => return Err(refuse("wit7-site1")),')

pat("SID2", CALLS,
    "    if !arg_loads_are_formals(&arg_ops, &params) {\n"
    '        return Err(refuse("call-arg-nonformal"));\n'
    "    }\n",
    "    if !arg_loads_are_formals(&arg_ops, &params) {\n"
    '        return Err(refuse("wit7-site2"));\n'
    "    }\n")

pat("SID3", CALLS,
    'return Err(Block::refuse(seg, *p, "call-arg-nonformal"));',
    'return Err(Block::refuse(seg, *p, "wit7-site3"));')

pat("SID4", MCMP,
    'return Err(Some(Block::refuse(seg, p, "call-arg-nonformal")));',
    'return Err(Some(Block::refuse(seg, p, "wit7-site4")));')

pat("SID5", MTAIL,
    'return Err(Some(Block::refuse(seg, p, "call-arg-nonformal")));',
    'return Err(Some(Block::refuse(seg, p, "wit7-site5")));')

# ---------------------------------------------------------------------------
# THE MUTANTS — `w-mutcensus`' own registered mutations, replayed at this base.
# ---------------------------------------------------------------------------

# `C1` — the NAMED CONTROL (`docs/rungs/README.md` probe rule 1). NOT a site.
pat("C1", CALLS,
    "if syms > 1 && !two_sym_thunk {",
    "if syms > 2 && !two_sym_thunk {")

# `CS3` — `w-mutcensus`' own mutation: retarget the arm's KEY.
pat("M-CS3", CENSUS,
    '"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,',
    '"static-scan-loop" => STORE_RUN_CALL_NO_CARRIER,')

# `CS3B` — THE RETARGET A SOURCE-TEXT FENCE CENSUS CANNOT SEE. The match LABEL
# moves, so the arm is never selected and the body falls to the `_` arm; the
# constant `STATIC_SCAN_LOOP_OBJECT` and its ONE raise site do not move, so
# `tests/fence_site_census.rs`' per-key table is byte-identical.
pat("M-CS3B", CENSUS,
    '"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,',
    '"static-scan-loop-x" => STATIC_SCAN_LOOP_OBJECT,')

# `CS3C` — THE MUTATION NEITHER SOURCE-TEXT TEST CAN SEE.
#
# `M-CS3B` was registered GREEN at base and is RED — caught by
# `callee_unresolved_sites.rs`' `ARM_PATTERNS`, which counts the literal
# `"static-scan-loop" =>` in the `match label` block. So the arm's TEXT is
# guarded twice over (that test and `fence_site_census.rs`) and **neither runs a
# compiler**. This mutation moves the label at its PRODUCER instead
# (`census.rs:951`, `FnVerdict::InClass("static-scan-loop")`), leaving the
# `match label` block byte-identical and every constant at its one raise site.
# Both source-text tests stay green; the arm is simply never selected and every
# static-scan-loop body falls to `_ => CALLEE_UNRESOLVED_TAIL`.
pat("M-CS3C", CENSUS,
    'FnVerdict::InClass("static-scan-loop")',
    'FnVerdict::InClass("static-scan-loop2")')

# `CS4` — drop the bind-refusal routing; every bind body reports the fallback.
pat("M-CS4", CENSUS,
    "bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)",
    "{ let _ = bind_key; STORE_RUN_BIND_NO_CARRIER }")

# `CS9` — `false &&` on the opt-mode gate.
pat("M-CS9", CENSUS,
    "Some(f) if opt_word_mode(opt_word).is_none() => {",
    "Some(f) if false && opt_word_mode(opt_word).is_none() => {")

# `CA6` — the slot arm's key, nonformal -> computed.
pat("M-CA6", CALLS,
    'None => return Err(refuse("call-arg-nonformal")),',
    'None => return Err(refuse("call-arg-computed")),')

# `CA8` — computed -> nonformal.
pat("M-CA8", CALLS,
    '_ => return Err(refuse("call-arg-computed")),',
    '_ => return Err(refuse("call-arg-nonformal")),')

# `B2` — `false &&` on `resolve_data_def`'s comdat/initialized gate.
pat("M-B2", BIND,
    "        if !o.comdat || !o.initialized {\n"
    "            return None;\n"
    "        }\n"
    "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
    "            return None;\n"
    "        }\n"
    "        let init = super::ininit::in_scalar_initializers(self.inb);\n",
    "        if false && (!o.comdat || !o.initialized) {\n"
    "            return None;\n"
    "        }\n"
    "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
    "            return None;\n"
    "        }\n"
    "        let init = super::ininit::in_scalar_initializers(self.inb);\n")

# `B7` — `false &&` on `resolve_bss_def`'s comdat/initialized gate.
pat("M-B7", BIND,
    "        if o.comdat || o.initialized {\n"
    "            return None;\n"
    "        }\n",
    "        if false && (o.comdat || o.initialized) {\n"
    "            return None;\n"
    "        }\n")

# `B9` — board **#3281**. `false &&` on `resolve_bss_def`'s `o.size == 0`, the
# site `w-deadsites` measured UNREACHED and `w-mutcensus` scored RED.
pat("M-B9", BIND,
    "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
    "            return None;\n"
    "        }\n"
    "        if o.size == 0 {\n"
    "            return None;\n"
    "        }\n",
    "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
    "            return None;\n"
    "        }\n"
    "        if false && o.size == 0 {\n"
    "            return None;\n"
    "        }\n")

FILES = sorted({v[0] for v in P.values()})


def read(p):
    with open(os.path.join(ROOT, p), encoding="utf8") as fh:
        return fh.read()


def write(p, t):
    with open(os.path.join(ROOT, p), "w", encoding="utf8") as fh:
        fh.write(t)


def dirty():
    out = subprocess.run(
        ["git", "status", "--porcelain", "--", "crates"],
        cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    return out


def revert():
    subprocess.run(["git", "checkout", "--"] + FILES, cwd=ROOT, check=True)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    cmd = sys.argv[1]
    if cmd == "list":
        for k, (p, o, _) in sorted(P.items()):
            print(f"{k:10s} {p}  ({len(read(p).split(o)) - 1} match)")
        return 0
    if cmd == "revert":
        revert()
        print("reverted:", " ".join(FILES))
        return 0
    if cmd == "apply":
        d = dirty()
        if d:
            print("REFUSING: crates/ is dirty:\n" + d, file=sys.stderr)
            return 1
        for pid in sys.argv[2:]:
            if pid not in P:
                print(f"unknown patch {pid}", file=sys.stderr)
                return 1
            path, old, new = P[pid]
            t = read(path)
            n = t.count(old)
            if n != 1:
                print(f"{pid}: locator matches {n} times in {path}, expected 1",
                      file=sys.stderr)
                revert()
                return 1
            write(path, t.replace(old, new))
            print(f"applied {pid} -> {path}")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
