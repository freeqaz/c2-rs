#!/usr/bin/env python3
"""w-fencea — the two mutants that fire the CLOSURE of the admission set."""
import os, subprocess, sys
R = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
F = os.path.join(R, "crates/c2-core/src/codegen/labels.rs")

def run(name, old, new, want):
    src = open(F).read()
    assert old in src, name
    open(F, "w").write(src.replace(old, new, 1))
    print("=== MUTANT %s ===" % name)
    p = subprocess.run(["cargo", "test", "-p", "c2-core", "--release", "--lib",
                        "codegen::labels"], cwd=R, capture_output=True, text=True)
    out = p.stdout + p.stderr
    if "error[" in out or "\nerror:" in out:
        print("  BUILD-RED — the exhaustive match refuses it at compile time")
        for line in out.splitlines():
            if line.startswith("error["):
                print("   ", line.strip())
    elif "%s ... FAILED" % want in out:
        print("  RED: %s" % want)
    else:
        print("  GREEN — the guard did not fire")
    open(F, "w").write(src)

run("M8-ALL-is-reordered",
    "        ChargedClass::PtrWalkModLoop,\n        ChargedClass::XteaEncryptLoop,",
    "        ChargedClass::XteaEncryptLoop,\n        ChargedClass::PtrWalkModLoop,",
    "the_admitted_class_list_is_complete")
run("M9-a-fourth-class-with-no-evidence",
    "pub enum ChargedClass {\n",
    "pub enum ChargedClass {\n    /// a class nobody graded\n    Bogus,\n",
    "every_admitted_class_has_a_registered_control_flow_surcharge")
