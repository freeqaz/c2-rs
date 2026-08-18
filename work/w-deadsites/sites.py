#!/usr/bin/env python3
"""`w-deadsites` — the site table and the patcher.

One table, two modes:

  * `probe`  — insert a behaviour-preserving `deadprobe::hit(ix, "id")` on the
               branch `w-mutcensus`' mutation removed.
  * `panic`  — replace that branch's effect with `panic!("w-deadsites <id>")`,
               which is board #3246's named method.

Both are applied by EXACT STRING replacement with a uniqueness assertion, never
by line number: `w-mutcensus`' own enumeration went stale twice inside one
lane's wall clock, and this lane re-located every one of its 30 rows by text.

Usage:
    sites.py probe                 apply the probe to every site
    sites.py panic ID [ID ...]     apply panic!() to the named sites
    sites.py revert                git checkout the touched files
    sites.py list                  print the table
"""

import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
F = "crates/c2-il/src/func/"

# (id, index, file, old, probe-new, panic-new)
#
# `probe-new` inserts a call and changes nothing else. `panic-new` replaces the
# refusal itself, so a run that completes proves the branch was never taken.
SITES = []


def site(sid, ix, path, old, new, panic):
    SITES.append((sid, ix, path, old, new, panic))


# ---------------------------------------------------------------- census.rs --
CENSUS = F + "census.rs"

site("CS2", 0, CENSUS,
     '"store-run-call" => STORE_RUN_CALL_NO_CARRIER,',
     '"store-run-call" => { crate::deadprobe::hit(0, "CS2"); STORE_RUN_CALL_NO_CARRIER }',
     '"store-run-call" => panic!("w-deadsites CS2"),')

site("CS3", 1, CENSUS,
     '"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT,',
     '"static-scan-loop" => { crate::deadprobe::hit(1, "CS3"); STATIC_SCAN_LOOP_OBJECT }',
     '"static-scan-loop" => panic!("w-deadsites CS3"),')

site("CS4", 2, CENSUS,
     "bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)",
     '{ if bind_key.is_some() { crate::deadprobe::hit(2, "CS4"); } '
     "bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER) }",
     '{ if bind_key.is_some() { panic!("w-deadsites CS4"); } '
     "bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER) }")

site("CS5", 3, CENSUS,
     '"framed-call" => CALLEE_UNRESOLVED_FRAMED,',
     '"framed-call" => { crate::deadprobe::hit(3, "CS5"); CALLEE_UNRESOLVED_FRAMED }',
     '"framed-call" => panic!("w-deadsites CS5"),')

site("CS6", 4, CENSUS,
     'l if l.starts_with("call-sequence") => {\n'
     "                                            CALLEE_UNRESOLVED_SEQ\n",
     'l if l.starts_with("call-sequence") => {\n'
     '                                            crate::deadprobe::hit(4, "CS6");\n'
     "                                            CALLEE_UNRESOLVED_SEQ\n",
     'l if l.starts_with("call-sequence") => {\n'
     '                                            panic!("w-deadsites CS6")\n')

site("CS7", 5, CENSUS,
     'l if l.starts_with("empty-dtor") => {\n'
     "                                            CALLEE_UNRESOLVED_DTOR\n",
     'l if l.starts_with("empty-dtor") => {\n'
     '                                            crate::deadprobe::hit(5, "CS7");\n'
     "                                            CALLEE_UNRESOLVED_DTOR\n",
     'l if l.starts_with("empty-dtor") => {\n'
     '                                            panic!("w-deadsites CS7")\n')

site("CS8", 6, CENSUS,
     "_ => CALLEE_UNRESOLVED_TAIL,",
     '_ => { crate::deadprobe::hit(6, "CS8"); CALLEE_UNRESOLVED_TAIL }',
     '_ => panic!("w-deadsites CS8"),')

site("CS9", 7, CENSUS,
     "Some(f) if opt_word_mode(opt_word).is_none() => {\n"
     "                                    let _ = f;\n",
     "Some(f) if opt_word_mode(opt_word).is_none() => {\n"
     "                                    let _ = f;\n"
     '                                    crate::deadprobe::hit(7, "CS9");\n',
     "Some(f) if opt_word_mode(opt_word).is_none() => {\n"
     "                                    let _ = f;\n"
     '                                    panic!("w-deadsites CS9");\n')

site("X4", 30, CENSUS,
     "FnVerdict::Blocked(Block::at_end(seg, CALLEE_DEFINED_IN_TU))",
     '{ crate::deadprobe::hit(30, "X4"); '
     "FnVerdict::Blocked(Block::at_end(seg, CALLEE_DEFINED_IN_TU)) }",
     'panic!("w-deadsites X4")')

# ----------------------------------------------------------------- calls.rs --
CALLS = F + "body/shapes/calls.rs"

site("CA2", 8, CALLS,
     'return Err(refuse("call-arg-sym-overflow"));',
     '{ crate::deadprobe::hit(8, "CA2"); return Err(refuse("call-arg-sym-overflow")); }',
     'panic!("w-deadsites CA2");')

site("CA6", 9, CALLS,
     'None => return Err(refuse("call-arg-nonformal")),',
     'None => { crate::deadprobe::hit(9, "CA6"); return Err(refuse("call-arg-nonformal")); }',
     'None => panic!("w-deadsites CA6"),')

site("CA8", 10, CALLS,
     '_ => return Err(refuse("call-arg-computed")),',
     '_ => { crate::deadprobe::hit(10, "CA8"); return Err(refuse("call-arg-computed")); }',
     '_ => panic!("w-deadsites CA8"),')

site("CA9", 11, CALLS,
     'SlotArg::Lit(_) => return Err(refuse("call-arg-lit-classified-twice")),',
     'SlotArg::Lit(_) => { crate::deadprobe::hit(11, "CA9"); '
     'return Err(refuse("call-arg-lit-classified-twice")); }',
     'SlotArg::Lit(_) => panic!("w-deadsites CA9"),')

site("CA10", 12, CALLS,
     'SlotArg::SymAddr(_) => return Err(refuse("call-arg-sym-classified-twice")),',
     'SlotArg::SymAddr(_) => { crate::deadprobe::hit(12, "CA10"); '
     'return Err(refuse("call-arg-sym-classified-twice")); }',
     'SlotArg::SymAddr(_) => panic!("w-deadsites CA10"),')

site("CA13", 13, CALLS,
     'return Err(refuse("call-arg-source-out-of-slots"));',
     '{ crate::deadprobe::hit(13, "CA13"); '
     'return Err(refuse("call-arg-source-out-of-slots")); }',
     'panic!("w-deadsites CA13");')

site("CA16", 14, CALLS,
     'return Err(refuse("call-arg-repeated-leaf"));',
     '{ crate::deadprobe::hit(14, "CA16"); return Err(refuse("call-arg-repeated-leaf")); }',
     'panic!("w-deadsites CA16");')

site("CA18", 15, CALLS,
     "if !additive_chain_canonical(&arg_ops) {\n"
     '        return Err(refuse("call-arg-noncanonical-order"));\n',
     "if !additive_chain_canonical(&arg_ops) {\n"
     '        crate::deadprobe::hit(15, "CA18");\n'
     '        return Err(refuse("call-arg-noncanonical-order"));\n',
     "if !additive_chain_canonical(&arg_ops) {\n"
     '        panic!("w-deadsites CA18");\n')

site("X2", 32, CALLS,
     'return Err(refuse("call-arg-outer-formal"));',
     '{ crate::deadprobe::hit(32, "X2"); return Err(refuse("call-arg-outer-formal")); }',
     'panic!("w-deadsites X2");')

# ------------------------------------------------------------------ bind.rs --
BIND = F + "bind.rs"

site("B2B3", 16, BIND,
     "        if !o.comdat || !o.initialized {\n"
     "            return None;\n"
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     "            return None;\n"
     "        }\n",
     "        if !o.comdat || !o.initialized {\n"
     '            crate::deadprobe::hit(16, "B2");\n'
     "            return None;\n"
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     '            crate::deadprobe::hit(17, "B3");\n'
     "            return None;\n"
     "        }\n",
     "        if !o.comdat || !o.initialized {\n"
     '            panic!("w-deadsites B2");\n'
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     '            panic!("w-deadsites B3");\n'
     "        }\n")

site("B4", 18, BIND,
     "        if init.accepted + init.residue.len() != init.records {\n"
     "            return None;\n"
     "        }\n",
     "        if init.accepted + init.residue.len() != init.records {\n"
     '            crate::deadprobe::hit(18, "B4");\n'
     "            return None;\n"
     "        }\n",
     "        if init.accepted + init.residue.len() != init.records {\n"
     '            panic!("w-deadsites B4");\n'
     "        }\n")

site("B5", 19, BIND,
     "        if !init.refs.get(&tok).map(|r| r.is_empty()).unwrap_or(true) {\n"
     "            return None;\n"
     "        }\n",
     "        if !init.refs.get(&tok).map(|r| r.is_empty()).unwrap_or(true) {\n"
     '            crate::deadprobe::hit(19, "B5");\n'
     "            return None;\n"
     "        }\n",
     "        if !init.refs.get(&tok).map(|r| r.is_empty()).unwrap_or(true) {\n"
     '            panic!("w-deadsites B5");\n'
     "        }\n")

site("B6", 20, BIND,
     "        if bytes.len() != o.size as usize {\n"
     "            return None;\n"
     "        }\n",
     "        if bytes.len() != o.size as usize {\n"
     '            crate::deadprobe::hit(20, "B6");\n'
     "            return None;\n"
     "        }\n",
     "        if bytes.len() != o.size as usize {\n"
     '            panic!("w-deadsites B6");\n'
     "        }\n")

site("B7B8X3", 21, BIND,
     "        if o.comdat || o.initialized {\n"
     "            return None;\n"
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     "            return None;\n"
     "        }\n"
     "        if o.size == 0 {\n"
     "            return None;\n"
     "        }\n",
     "        if o.comdat || o.initialized {\n"
     '            crate::deadprobe::hit(21, "B7");\n'
     "            return None;\n"
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     '            crate::deadprobe::hit(22, "B8");\n'
     "            return None;\n"
     "        }\n"
     "        if o.size == 0 {\n"
     '            crate::deadprobe::hit(33, "X3");\n'
     "            return None;\n"
     "        }\n",
     "        if o.comdat || o.initialized {\n"
     '            panic!("w-deadsites B7");\n'
     "        }\n"
     "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
     '            panic!("w-deadsites B8");\n'
     "        }\n"
     "        if o.size == 0 {\n"
     '            panic!("w-deadsites X3");\n'
     "        }\n")

# ---------------------------------------------------------------- bundle.rs --
BUNDLE = F + "bundle.rs"

site("BU3", 23, BUNDLE,
     "            return if find_subslice(ex, &LO_MARKER).is_none() {\n"
     "                Some(Vec::new())\n"
     "            } else {\n"
     "                None\n"
     "            };\n",
     "            return if find_subslice(ex, &LO_MARKER).is_none() {\n"
     "                Some(Vec::new())\n"
     "            } else {\n"
     '                crate::deadprobe::hit(23, "BU3");\n'
     "                None\n"
     "            };\n",
     "            return if find_subslice(ex, &LO_MARKER).is_none() {\n"
     "                Some(Vec::new())\n"
     "            } else {\n"
     '                panic!("w-deadsites BU3")\n'
     "            };\n")

site("D1", 24, BUNDLE,
     "        if !is_dynamic_initializer_name(&thunk_name) {\n"
     "            return None;\n"
     "        }\n",
     "        if !is_dynamic_initializer_name(&thunk_name) {\n"
     '            crate::deadprobe::hit(24, "D1");\n'
     "            return None;\n"
     "        }\n",
     "        if !is_dynamic_initializer_name(&thunk_name) {\n"
     '            panic!("w-deadsites D1");\n'
     "        }\n")

site("D2", 25, BUNDLE,
     "        // it.\n"
     "        if init.accepted + init.residue.len() != init.records {\n"
     "            return None;\n"
     "        }\n",
     "        // it.\n"
     "        if init.accepted + init.residue.len() != init.records {\n"
     '            crate::deadprobe::hit(25, "D2");\n'
     "            return None;\n"
     "        }\n",
     "        // it.\n"
     "        if init.accepted + init.residue.len() != init.records {\n"
     '            panic!("w-deadsites D2");\n'
     "        }\n")

# -------------------------------------------------------------------- gl.rs --
GL = F + "gl.rs"

site("G2", 26, GL,
     "    out.retain(|n| !bad.contains(n));\n",
     "    if out.iter().any(|n| bad.contains(n)) {\n"
     '        crate::deadprobe::hit(26, "G2");\n'
     "    }\n"
     "    out.retain(|n| !bad.contains(n));\n",
     "    if out.iter().any(|n| bad.contains(n)) {\n"
     '        panic!("w-deadsites G2");\n'
     "    }\n"
     "    out.retain(|n| !bad.contains(n));\n")

# ------------------------------------------------------------- leaf_store.rs --
LEAF = F + "body/shapes/leaf_store.rs"

site("X1", 31, LEAF,
     "        let [b, v, IlOp::StoreInd { .. }, tail @ ..] = walk else {\n"
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        };\n",
     "        let [b, v, IlOp::StoreInd { .. }, tail @ ..] = walk else {\n"
     '            crate::deadprobe::hit(31, "X1");\n'
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        };\n",
     "        let [b, v, IlOp::StoreInd { .. }, tail @ ..] = walk else {\n"
     '            panic!("w-deadsites X1");\n'
     "        };\n")

site("L2", 27, LEAF,
     "        let IlOp::Load(base_tok) = b else {\n"
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        };\n",
     "        let IlOp::Load(base_tok) = b else {\n"
     '            crate::deadprobe::hit(27, "L2");\n'
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        };\n",
     "        let IlOp::Load(base_tok) = b else {\n"
     '            panic!("w-deadsites L2");\n'
     "        };\n")

site("L3", 28, LEAF,
     "            _ => return Err(STORE_RUN_BIND_GROUP_SHAPE),\n",
     '            _ => { crate::deadprobe::hit(28, "L3"); '
     "return Err(STORE_RUN_BIND_GROUP_SHAPE); }\n",
     '            _ => panic!("w-deadsites L3"),\n')

site("L9", 29, LEAF,
     "        if !matches!(b, IlOp::Load(_)) {\n"
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        }\n",
     "        if !matches!(b, IlOp::Load(_)) {\n"
     '            crate::deadprobe::hit(29, "L9");\n'
     "            return Err(STORE_RUN_BIND_GROUP_SHAPE);\n"
     "        }\n",
     "        if !matches!(b, IlOp::Load(_)) {\n"
     '            panic!("w-deadsites L9");\n'
     "        }\n")

# --------------------------------------------------------------------------
# PANIC-MODE OVERRIDES.
#
# Three sites share a patch hunk with a site that FIRED (`B2` with `B3`; `B7`
# with `B8` and `X3`), so the grouped replacement above cannot panic one without
# panicking the other. These entries locate the quiet member alone, by a longer
# unique context, and are used in `panic` mode instead of the group.
#   id -> (path, old, new)
PANIC_OVERRIDE = {
    "B3": (BIND,
           "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
           "            return None;\n"
           "        }\n"
           "        let init = super::ininit::in_scalar_initializers(self.inb);\n",
           "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
           '            panic!("w-deadsites B3");\n'
           "        }\n"
           "        let init = super::ininit::in_scalar_initializers(self.inb);\n"),
    "B8X3": (BIND,
             "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
             "            return None;\n"
             "        }\n"
             "        if o.size == 0 {\n"
             "            return None;\n"
             "        }\n",
             "        if o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0 {\n"
             '            panic!("w-deadsites B8");\n'
             "        }\n"
             "        if o.size == 0 {\n"
             '            panic!("w-deadsites X3");\n'
             "        }\n"),
}

# The named control (`docs/rungs/README.md` probe rule 1) — NOT a probe site.
# `w-guards` pins its failing set to the test name and this lane reproduces it
# before the first probe and after the last.
CONTROL_C1 = (CALLS, "if syms > 1 && !two_sym_thunk {", "if syms > 2 && !two_sym_thunk {")

FILES = sorted({s[2] for s in SITES})


def read(p):
    with open(os.path.join(ROOT, p), encoding="utf-8") as fh:
        return fh.read()


def write(p, t):
    with open(os.path.join(ROOT, p), "w", encoding="utf-8") as fh:
        fh.write(t)


def install_module(on):
    lib = os.path.join(ROOT, "crates/c2-il/src/lib.rs")
    with open(lib, encoding="utf-8") as fh:
        t = fh.read()
    tgt = os.path.join(ROOT, "crates/c2-il/src/deadprobe.rs")
    if on:
        src = os.path.join(os.path.dirname(__file__), "deadprobe.rs")
        with open(src, encoding="utf-8") as fh:
            body = fh.read()
        with open(tgt, "w", encoding="utf-8") as fh:
            fh.write(body)
        if "mod deadprobe;" not in t:
            # APPENDED, never prepended: `lib.rs` opens with a crate-level `//!`
            # block and an item in front of it is `error[E0753]`.
            with open(lib, "w", encoding="utf-8") as fh:
                fh.write(t + "\npub(crate) mod deadprobe;\n")
    else:
        if os.path.exists(tgt):
            os.remove(tgt)


def apply(mode, only=None):
    dirty = subprocess.run(["git", "-C", ROOT, "status", "--porcelain", "--", "crates"],
                           capture_output=True, text=True).stdout.strip()
    if dirty:
        sys.exit("REFUSING: crates/ is dirty:\n" + dirty)
    if mode == "probe":
        install_module(True)
    texts = {p: read(p) for p in FILES}
    applied = []
    todo = []
    for sid, ix, path, old, new, panic in SITES:
        if only is not None and sid not in only:
            continue
        todo.append((sid, path, old, new if mode == "probe" else panic))
    if mode == "panic" and only is not None:
        for sid in sorted(only):
            if sid in PANIC_OVERRIDE:
                path, old, new = PANIC_OVERRIDE[sid]
                todo.append((sid, path, old, new))
    for sid, path, old, repl in todo:
        t = texts[path]
        n = t.count(old)
        if n != 1:
            sys.exit(f"REFUSING: site {sid} matched {n} times in {path} (need exactly 1)")
        texts[path] = t.replace(old, repl)
        applied.append(sid)
    for p, t in texts.items():
        write(p, t)
    print(f"applied {mode} to {len(applied)} sites: {' '.join(applied)}")


def revert():
    install_module(False)
    subprocess.run(["git", "-C", ROOT, "checkout", "--"] + FILES +
                   ["crates/c2-il/src/lib.rs"], check=True)
    left = subprocess.run(["git", "-C", ROOT, "status", "--porcelain", "--", "crates/c2-il"],
                          capture_output=True, text=True).stdout.strip()
    print("reverted; crates/c2-il status:", left if left else "CLEAN")


def control(on):
    path, old, new = CONTROL_C1
    t = read(path)
    a, b = (old, new) if on else (new, old)
    if t.count(a) != 1:
        sys.exit(f"REFUSING: control C1 matched {t.count(a)} times")
    write(path, t.replace(a, b))
    print("control C1", "applied" if on else "reverted")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    if cmd == "probe":
        apply("probe")
    elif cmd == "panic":
        apply("panic", only=set(sys.argv[2:]))
    elif cmd == "revert":
        revert()
    elif cmd == "control-on":
        control(True)
    elif cmd == "control-off":
        control(False)
    else:
        for sid, ix, path, *_ in SITES:
            print(f"{sid:6} ix={ix:<3} {path}")
