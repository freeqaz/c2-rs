#!/usr/bin/env python3
"""w-mutcensus publication table.

Reads results/summary.tsv and emits (a) the full per-site markdown table with
file:line at 3835469c, the mutation, the registered colour and the observed
colour, and (b) the per-family rollups this census was commissioned to produce
(a "raise family" = the set of sites that raise the SAME census key, or the
same threshold constant, or mirror each other across data/bss).

Site text and registered colours mirror the frozen prereg
(docs/rungs/_2026-08-17-w-mutcensus-prereg.md §2) exactly. Read-only.
"""
import csv, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))

# id -> (file:line at 3835469c, mutation, registered colour, P, family)
SITES = [
    # ---- controls; C2 is the only out-of-crate site and is NOT in N ----
    ("C1",  "calls.rs:431",       "`syms > 1` -> `syms > 2` (arity fence)",           "RED",   .97, "call-arg arity"),
    ("C2",  "c2-core calls.rs:1815", "`count() != 1` -> `> 2` (backstop)",            "RED",   .95, "(control, out of N)"),
    ("C3",  "bind.rs:891",        "`.then_some(name)` -> unconditional `Some(name)`", "RED",   .97, "bind .gl linkage"),
    ("C4",  "census.rs:1216/1218","swap DATA_SYM_UNRESOLVED / DATA_SYM_LINKAGE",      "RED",   .97, "data-sym key pair"),
    ("C5",  "calls.rs:430",       "`false &&` on the two-sym thunk exemption",        "RED",   .90, "call-arg arity"),
    # ---- census.rs post-parse routing + gates ----
    ("CS2", "census.rs:1242", "key -> STATIC_SCAN_LOOP_OBJECT",             "GREEN", .75, "store-run/static-scan key pair"),
    ("CS3", "census.rs:1245", "key -> STORE_RUN_CALL_NO_CARRIER",           "GREEN", .75, "store-run/static-scan key pair"),
    ("CS4", "census.rs:1263", "drop `bind_key.unwrap_or`",                  "GREEN", .65, "store-run bind routing"),
    ("CS5", "census.rs:1265", "key -> CALLEE_UNRESOLVED_TAIL",              "GREEN", .70, "callee-unresolved key family (4)"),
    ("CS6", "census.rs:1267", "key -> CALLEE_UNRESOLVED_TAIL",              "GREEN", .70, "callee-unresolved key family (4)"),
    ("CS7", "census.rs:1270", "key -> CALLEE_UNRESOLVED_TAIL",              "GREEN", .70, "callee-unresolved key family (4)"),
    ("CS8", "census.rs:1272", "default arm key -> CALLEE_UNRESOLVED_FRAMED","RED",   .80, "callee-unresolved key family (4)"),
    ("CS9", "census.rs:1280", "`false &&` on the opt-mode gate",            "RED",   .60, "census opt/ptr-walk gates"),
    ("CS10","census.rs:1294", "`false &&` on ptr-walk-not-O1",              "RED",   .60, "census opt/ptr-walk gates"),
    ("CS11","census.rs:1306", "`false &&` on chain-not-O1",                 "RED",   .60, "census opt/ptr-walk gates"),
    ("CS12","census.rs:1358", "`false &&` on callee-defined-in-tu",         "RED",   .90, "census inline fence"),
    # ---- calls.rs call-argument fence family ----
    ("CA2", "calls.rs:434", "MAX_REGISTER_FORMALS + 9 (sym overflow)",      "GREEN", .80, "MAX_REGISTER_FORMALS threshold (3)"),
    ("CA3", "calls.rs:442", "`false &&` sym-permuted",                      "GREEN", .75, "call-arg permutation"),
    ("CA4", "calls.rs:529", "MAX_REGISTER_FORMALS + 9 (lit slots)",         "GREEN", .80, "MAX_REGISTER_FORMALS threshold (3)"),
    ("CA5", "calls.rs:593", "`false &&` lit-permuted",                      "GREEN", .70, "call-arg permutation"),
    ("CA6", "calls.rs:693", "key nonformal -> computed (slot arm)",         "GREEN", .50, "nonformal/computed key pair"),
    ("CA7", "calls.rs:699", "`false &&` lit-wide",                          "GREEN", .75, "literal width"),
    ("CA8", "calls.rs:710", "key computed -> nonformal",                    "GREEN", .70, "nonformal/computed key pair"),
    ("CA9", "calls.rs:732", "key lit- -> sym-classified-twice",             "GREEN", .90, "classified-twice key pair"),
    ("CA10","calls.rs:736", "key sym- -> lit-classified-twice",             "GREEN", .90, "classified-twice key pair"),
    ("CA11","calls.rs:747", "`false &&` outer-formal panic guard",          "RED",   .70, "call-arg source/slot"),
    ("CA12","calls.rs:759", "`false &&` duplicated source",                 "GREEN", .70, "call-arg source/slot"),
    ("CA13","calls.rs:772", "key source-out-of-slots -> outer-formal",      "GREEN", .80, "call-arg source/slot"),
    ("CA14","calls.rs:774", "`cycles > 1` -> `> 9`",                        "GREEN", .70, "permutation cycles"),
    ("CA15","calls.rs:780", "MAX_VERIFIED_PERM_CYCLE + 9",                  "GREEN", .75, "permutation cycles"),
    ("CA16","calls.rs:792", "`false &&` repeated-leaf",                     "GREEN", .70, "call-arg op shape"),
    ("CA17","calls.rs:800", "`false &&` noncanonical-order (loads)",        "RED",   .55, "call-arg op shape"),
    ("CA18","calls.rs:803", "`false &&` noncanonical-order (chain)",        "GREEN", .55, "call-arg op shape"),
    ("CA19","calls.rs:806", "`false &&` nonformal (post)",                  "RED",   .55, "nonformal/computed key pair"),
    ("CA20","calls.rs:868", "MAX_REGISTER_FORMALS + 9 (mcall chain)",       "GREEN", .80, "MAX_REGISTER_FORMALS threshold (3)"),
    ("CA21","calls.rs:878", "mcall key nonformal -> computed",              "RED",   .85, "mcall-chain key pair"),
    ("CA22","calls.rs:883", "`false &&` mcall lit-wide",                    "GREEN", .80, "literal width"),
    ("CA23","calls.rs:893", "mcall key computed -> nonformal",              "RED",   .85, "mcall-chain key pair"),
    # ---- bind.rs resolution gates ----
    ("B2",  "bind.rs:929", "`false &&` data-def comdat/init",               "GREEN", .60, "data-def / bss-def mirror"),
    ("B3",  "bind.rs:932", "`false &&` data-def thread-local",              "GREEN", .70, "data-def / bss-def mirror"),
    ("B4",  "bind.rs:939", "`false &&` `.in` totality",                     "GREEN", .65, "bind .in totality"),
    ("B5",  "bind.rs:942", "`false &&` `.in` refs",                         "GREEN", .65, "bind .in totality"),
    ("B6",  "bind.rs:946", "`false &&` size-exact",                         "GREEN", .60, "bind size"),
    ("B7",  "bind.rs:985", "`false &&` bss-def comdat/init",                "GREEN", .60, "data-def / bss-def mirror"),
    ("B8",  "bind.rs:988", "`false &&` bss-def thread-local",               "GREEN", .70, "data-def / bss-def mirror"),
    ("B9",  "bind.rs:991", "`false &&` bss-def size==0",                    "GREEN", .70, "bind size"),
    ("B10", "bind.rs:862", "`false &&` varargs name gate",                  "RED",   .65, "varargs"),
    # ---- gl.rs ----
    ("G1",  "gl.rs:2188", "`|| true` on the extern-data linkage byte",      "RED",   .90, "gl linkage/name"),
    ("G2",  "gl.rs:2198", "retain -> keep all (ambiguous names)",           "GREEN", .55, "gl linkage/name"),
    ("G3",  "gl.rs:1085", "NAME_SEPARATORS drop 0x26",                      "RED",   .85, "gl linkage/name"),
    # ---- bundle.rs TU gates ----
    ("BU1", "bundle.rs:1694", "opt_word_mode unknown -> `Some(Ox)`",        "RED",   .70, "bundle TU gate"),
    ("BU2", "bundle.rs:1919", "`false &&` drectve gate",                    "GREEN", .60, "bundle TU gate"),
    ("BU3", "bundle.rs:1940", "`|| true` empty-module LO probe",            "GREEN", .55, "bundle TU gate"),
    ("D1",  "bundle.rs:2423", "`false &&` dyninit name clause",             "RED",   .60, "dyninit_tu / data_tu"),
    ("D2",  "bundle.rs:2887", "`false &&` data_tu `.in` totality",          "GREEN", .55, "dyninit_tu / data_tu"),
    # ---- leaf_store.rs bind_run_ops ----
    ("L1",  "leaf_store.rs:2254", "key GROUP_SHAPE -> MULTI_PRODUCER",      "RED",   .65, "group-shape raise family (4)"),
    ("L2",  "leaf_store.rs:2257", "key GROUP_SHAPE -> MULTI_PRODUCER",      "GREEN", .55, "group-shape raise family (4)"),
    ("L3",  "leaf_store.rs:2285", "key GROUP_SHAPE -> MULTI_PRODUCER",      "GREEN", .50, "group-shape raise family (4)"),
    ("L4",  "leaf_store.rs:2370", "`false &&` mixed-kind",                  "RED",   .80, "leaf-store residue gates"),
    ("L5",  "leaf_store.rs:2374", "`== 0` -> `== i32::MIN` (addr producer)","RED",   .60, "leaf-store residue gates"),
    ("L6",  "leaf_store.rs:2390", "`lits.len() > 1` -> `> 9`",              "RED",   .75, "leaf-store residue gates"),
    ("L7",  "leaf_store.rs:2399", "`false &&` pool bound",                  "RED",   .55, "leaf-store residue gates"),
    ("L8",  "leaf_store.rs:2402", "MAX_SYMBOL_CROSSINGS + 9",               "RED",   .75, "leaf-store residue gates"),
    ("L9",  "leaf_store.rs:2455", "`false &&` group-shape (2nd walk)",      "GREEN", .60, "group-shape raise family (4)"),
]
CONTROLS = {"C1", "C2", "C3", "C4", "C5"}
OUT_OF_N = {"C2"}          # the c2-core backstop: a control, not a c2-il fence site


def load():
    rows = {}
    path = os.path.join(HERE, "results", "summary.tsv")
    with open(path) as f:
        for r in csv.reader(f, delimiter="\t"):
            if r and not r[0].endswith(".aborted") and not r[0].startswith("N0"):
                rows[r[0]] = r          # last write wins: a rerun supersedes
    return rows


def main():
    rows = load()
    print("| id | site (`crates/c2-il/src/func/…` at `3835469c`) | mutation | reg. | observed | pass/fail | failing tests |")
    print("|---|---|---|---|---|---|---|")
    hits = misses = 0
    greens = reds = notrun = invalid = 0
    fam = {}
    for (mid, site, mut, reg, p, family) in SITES:
        r = rows.get(mid)
        obs = r[1] if r else "NOT RUN"
        counts = f"{r[2]}/{r[3]}" if r else "—"
        fails = (r[5] if r and len(r) > 5 else "").replace(";", "<br>") or "—"
        if obs in ("NOT RUN", "INVALID"):
            mark = ""
            notrun += obs == "NOT RUN"
            invalid += obs == "INVALID"
        else:
            mark = " HIT" if obs == reg else " **MISS**"
            hits += obs == reg
            misses += obs != reg
            if mid not in OUT_OF_N:
                greens += obs == "GREEN"
                reds += obs == "RED"
        if mid not in OUT_OF_N:
            fam.setdefault(family, []).append((mid, obs))
        tag = " *(control)*" if mid in CONTROLS else ""
        print(f"| {mid}{tag} | `{site}` | {mut} | {reg} {p:.2f} | **{obs}**{mark} | {counts} | {fails} |")

    print(f"\n**X = {greens} GREEN (unguarded) of {greens+reds} of the 63 c2-il "
          f"fence sites run** — {notrun} NOT RUN, {invalid} INVALID. "
          f"Prereg: {hits} hits / {misses} misses over the colours scored.")

    print("\n### Per-family rollup (guarded raise sites / raise sites in family)\n")
    print("| family | sites | RED (guarded) | GREEN (unguarded) | shape |")
    print("|---|---|---|---|---|")
    for family, members in sorted(fam.items(), key=lambda kv: -len(kv[1])):
        run = [(m, o) for m, o in members if o in ("RED", "GREEN")]
        r_ = [m for m, o in run if o == "RED"]
        g_ = [m for m, o in run if o == "GREEN"]
        if not run:
            shape = "not run"
        elif not r_:
            shape = f"**wholly unguarded** ({len(g_)}/{len(members)})"
        elif not g_:
            shape = f"wholly guarded ({len(r_)}/{len(members)})"
        else:
            shape = f"**guarded at {len(r_)} of {len(members)}** raise sites"
        print(f"| {family} | {len(members)} | {len(r_)} {r_} | {len(g_)} {g_} | {shape} |")


if __name__ == "__main__":
    main()
