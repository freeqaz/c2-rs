#!/usr/bin/env python3
"""Load the `scripts/sweep.d/` fragments — **one locator**, two consumers.

`scripts/expr_sweep.sh` grades the generated cases one at a time against the
real toolchain; `scripts/cross_sweep.py` grades the *cross product* of the
shape families those cases turn out to exercise. Both need to enumerate the
fragments and run their `cases(emit)` hooks, and this module is the only place
that knows how. A second copy of this loop is exactly the "one rule, two
implementations" shape `docs/GAPS.md` §6 keeps recording (instances #9, #10),
so there is one.

The fragment contract is unchanged and is stated in `scripts/expr_sweep.sh`:

    def cases(emit):
        emit("int f(int a) { return a + 1; }\\n")   # one .cpp case

`emit` is supplied by the *loader*, never by the fragment, so a fragment can
neither see nor rewind another fragment's counter — the `n`-shadowing trap that
silently overwrote 1,233 already-written cases stays unrepresentable.

Run directly, this module is the generator half of the sweep driver:

    python3 scripts/sweep_gen.py <outdir> <fragment-dir>

writing `<stem>-%04d.cpp` per case, printing a per-fragment count, and failing
if any fragment emits zero cases or if what it wrote to disk does not equal
what it counted.
"""

import os
import sys


def fragment_files(frag_dir, only=""):
    """`(all, selected)` fragment filenames, sorted. `only` is a substring filter."""
    names = sorted(
        f for f in os.listdir(frag_dir) if f.endswith(".py") and not f.startswith("_")
    )
    if not names:
        raise SystemExit("no sweep fragments in %s" % frag_dir)
    selected = [f for f in names if only in f]
    if only and not selected:
        raise SystemExit("C2RS_SWEEP_ONLY=%r matched no fragment" % only)
    return names, selected


def fragment_cases(frag_dir, name):
    """`(stem, [source, ...])` for one fragment, in emission order."""
    stem = name[:-3]
    path = os.path.join(frag_dir, name)
    collected = []

    # The loader owns the accumulator: a fragment is handed `emit` and nothing
    # else, so it cannot reach another fragment's namespace.
    def emit(src, _collected=collected):
        _collected.append(src)

    ns = {"__name__": "sweep_" + stem.replace("-", "_"), "__file__": path}
    with open(path) as fh:
        exec(compile(fh.read(), path, "exec"), ns)
    if "cases" not in ns:
        raise SystemExit("fragment %s defines no cases(emit)" % name)
    ns["cases"](emit)
    return stem, collected


def load_all(frag_dir, only=""):
    """`[(stem, [source, ...]), ...]` for every selected fragment."""
    _, selected = fragment_files(frag_dir, only)
    return [fragment_cases(frag_dir, name) for name in selected]


def write_cases(out, frag_dir, only="", quiet=False):
    """Write every selected fragment's cases as `<stem>-%04d.cpp`. Returns the total.

    Fails on a fragment that emitted zero cases (the observable symptom of the
    counter bug) and on a printed/on-disk mismatch (a case silently overwritten
    by a name collision would otherwise be invisible).
    """
    all_names, selected = fragment_files(frag_dir, only)
    if only and not quiet:
        print(
            "C2RS_SWEEP_ONLY=%r: %d of %d fragments — THE TOTAL BELOW IS PARTIAL"
            % (only, len(selected), len(all_names))
        )

    total = 0
    empty = []
    written = set()
    for name in selected:
        stem, srcs = fragment_cases(frag_dir, name)
        for i, src in enumerate(srcs, 1):
            path = os.path.join(out, "%s-%04d.cpp" % (stem, i))
            if path in written:
                raise SystemExit("two cases claim %s" % path)
            written.add(path)
            with open(path, "w") as fh:
                fh.write(src)
        if not quiet:
            print("  fragment %-26s %5d cases" % (stem, len(srcs)))
        if not srcs:
            empty.append(stem)
        total += len(srcs)

    if empty:
        raise SystemExit(
            "FRAGMENT EMITTED ZERO CASES: %s — a silent generator drop is a hard "
            "error here (docs/ARCHITECTURE_SEAMS.md §2.4)" % ", ".join(empty)
        )

    # printed == generated == on disk. The generator counts what it emitted; this
    # counts what survived, so a name collision that overwrote a case (the 1,233-case
    # bug's actual damage) fails here instead of reporting a smaller green sweep.
    on_disk = len([f for f in os.listdir(out) if f.endswith(".cpp")])
    if on_disk != total:
        raise SystemExit(
            "GENERATED %d CASES BUT %d .cpp ARE ON DISK in %s — a case was "
            "overwritten or a stale case survived" % (total, on_disk, out)
        )
    if not quiet:
        print("  %d fragments, %d cases total (%d .cpp on disk)"
              % (len(selected), total, on_disk))
    return total


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: sweep_gen.py <outdir> <fragment-dir>")
    write_cases(sys.argv[1], sys.argv[2], os.environ.get("C2RS_SWEEP_ONLY", ""))
