#!/usr/bin/env python3
"""Enumerate the PORT's IL-opcode decode sites, one row per (opcode, site).

Lane `w-ilarms`, board #3567-#3572.  Tooling (outside the std-only `crates/`
workspace, per CLAUDE.md).  **Reads this repository, not `c2.dll`** -- it is the
port half of the arm -> port-site map and every row it emits is marked `[src]`,
never `[R]`.

It is a CANDIDATE generator, not an oracle.  It finds every match arm in
`crates/` whose pattern is a bare 8-bit hex literal (or a `|`/`..=` set of
them), records the enclosing `fn`, and drops arms inside `#[cfg(test)]` /
`mod tests` by brace depth.  Two false-positive families are then excluded by
NAME, listed here so the exclusion is auditable rather than silent:

  * `mcall::type_class` and its siblings match on a `.sy` TYPE tag byte, not on
    an operand-stream opcode.  Same literal space, different stream.
  * width/escape discriminators (`0x80` as a VI32 wide marker) are not opcodes.

Everything else is emitted and the map curates it by reading.  A site the map
rejects is rejected in writing, with the reason.

    python3 docs/whitebox/scripts/scan_port_opcodes.py            # per opcode
    python3 docs/whitebox/scripts/scan_port_opcodes.py --tsv
    python3 docs/whitebox/scripts/scan_port_opcodes.py --coverage <ilarms.tsv>
"""

import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
CRATES = [os.path.join(ROOT, "crates", c, "src")
          for c in ("c2-il", "c2-core", "c2-obj", "c2-harness", "c2-reference")]

ARM = re.compile(
    r"^\s*(0x[0-9A-Fa-f]{2}(?:\s*(?:\||\.\.=)\s*0x[0-9A-Fa-f]{2})*)"
    r"(\s+if\s+[^=]*?)?\s*=>")
LIT = re.compile(r"0x([0-9A-Fa-f]{2})")
RANGE = re.compile(r"0x([0-9A-Fa-f]{2})\s*\.\.=\s*0x([0-9A-Fa-f]{2})")
CONT = re.compile(r"^\s*0x[0-9A-Fa-f]{2}(\s*(\||\.\.=)\s*0x[0-9A-Fa-f]{2})*\s*\|?\s*$")
CONT_NEXT = re.compile(r"^\s*\|?\s*0x[0-9A-Fa-f]{2}")
FN = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+)*fn\s+([A-Za-z0-9_]+)")

# Excluded by NAME, not silently: these match on a stream that is not the `.ex`
# operand-opcode stream, so their literals live in a different numbering.  Every
# exclusion carries its reason; the map republishes this list.
EXCLUDED_FILES = {
    "crates/c2-il/src/func/sy.rs":       "the .sy symbol stream",
    "crates/c2-il/src/func/gl.rs":       "the .gl binding stream",
    "crates/c2-il/src/func/glalias.rs":  "the .gl binding stream",
    "crates/c2-il/src/func/ininit.rs":   "the .in initializer stream",
    "crates/c2-il/src/func/inlit.rs":    "the .in initializer stream",
    "crates/c2-il/src/func/ehscope.rs":  "EH scope records, not operand opcodes",
}
EXCLUDED_FNS = {
    "from_code":         "mcall::TypeClass::from_code -- a .sy TYPE tag byte",
    "read_line_record":  "codec 0x80 is a VI32 wide marker, not an opcode",
    "shift_mask_rlwinm": "PPC rlwinm mask arithmetic, not IL",
    "opt_word_at":       "bundle option words, not the .ex body",
    "align_of_type_tag": "a TYPE tag byte",
    "db_class_size":     "an EH class-size tag",
    "gl_function_attrs": "a .gl attribute byte",
    "token_bytes":       "a .sy token width",
    "read_elements":     "a .in element tag",
    "provide_data_tu":   "a .gl COMDAT attribute byte (0xE0/0xA0/0x20)",
}
# Line-level exclusions: a literal that IS an opcode value but is being matched
# in SUB-opcode position, so crediting it to the top-level arm would be wrong.
EXCLUDED_LINES = {
    ("crates/c2-il/src/func/body/shapes/control_flow.rs", 1071):
        "the `43 42` escape sub-opcode, not top-level 0x42",
    ("crates/c2-il/src/func/body/shapes/control_flow.rs", 1072):
        "the `43 37` escape sub-opcode, not top-level 0x37",
}


# --- the gate and depth classification, read from the port source ------------
#
# GATE answers prereg limb 1 -- "does the port decode this only under a
# precondition the ARM does not impose?"  It is a property of the ENTRY POINT,
# established by reading each site's callers:
#
#   U      ungated.  `control_flow::{step,operand,walk}` is reached from
#          `census.rs:448 scan_full`, which runs on every body in the workload;
#          `codec::try_*_token` is the container codec, fenced by a byte-exact
#          round-trip; `body/mod::*_opcode_name` are census names.
#   G-adm  admission-gated -- `parse_expr_classed` has exactly one caller,
#          `body/mod.rs:2832`, on the ACCEPTING path, and `parse_segment_shape`
#          IS the admission gate (decision 13's "decode and admission are fused").
#   G-env  environment-gated -- `chain_skip_form` is the chain sink's width
#          table, "poisoned, environment-gated, off on every gate lane and every
#          default scan" (its own module doc).
#   G-shp  shape-gated -- reached only after a named body shape has matched.
#
# DEPTH is orthogonal and answers "what survives the read":
#   name   a census key or an enum discriminant; ZERO operand bytes consumed
#   width  the cursor advances by the right amount; the payload is discarded
#   field  at least one operand field is retained for a downstream consumer
GATE = {
    ("control_flow.rs", "step"): "U", ("control_flow.rs", "operand"): "U",
    ("control_flow.rs", "walk"): "U",
    ("codec.rs", "try_ex_token"): "U", ("codec.rs", "try_prefix_token"): "U",
    ("mod.rs", "expr_opcode_name"): "U", ("mod.rs", "cflow_opcode_name"): "U",
    ("bundle.rs", "bare_lo_after_prefix"): "U",
    ("expr.rs", "parse_expr_classed"): "G-adm",
    ("mod.rs", "parse_segment_shape"): "G-adm",
    ("mod.rs", "from_opcode"): "G-adm",
    ("expr.rs", "chain_skip_form"): "G-env",
    ("expr.rs", "chain_sink_step"): "G-env",
}
DEPTH = {
    ("mod.rs", "expr_opcode_name"): "name", ("mod.rs", "cflow_opcode_name"): "name",
    ("mod.rs", "from_opcode"): "name", ("mcall_tail.rs", "at"): "name",
    ("counted_accum_loop.rs", "accum_op"): "name",
    ("ptr_walk_chain_loop.rs", "chain_op_kind"): "name",
    ("control_flow.rs", "operand"): "width",
    ("expr.rs", "chain_skip_form"): "width",
}


def gate_of(rel, fn):
    return GATE.get((rel.rsplit("/", 1)[-1], fn), "G-shp")


def depth_of(rel, fn):
    return DEPTH.get((rel.rsplit("/", 1)[-1], fn), "field")


def verdict(sites):
    """The prereg's verdict, applied mechanically to a scanned opcode."""
    if not sites:
        return "ABSENT"
    gates = {gate_of(r, fn) for r, _l, fn, _g, _t in sites}
    guarded_only = all(g for _r, _l, _fn, g, _t in sites)
    if "U" not in gates or guarded_only:
        return "NARROW(gate)"
    return "MATCHED*"


def scan_file(path):
    """Yield (opcode, line_no, fn_name, guarded, text) for every opcode arm.

    Brace counting is NOT used to find the enclosing `fn`: this tree's doc
    comments contain fenced code blocks full of unbalanced braces, and a first
    cut of this scanner attributed all 63 arms in `expr.rs` to one function
    1,200 lines above them.  The enclosing `fn` is the nearest PRECEDING `fn`
    line, which is exact for a flat module and conservative for a nested one.
    Test modules are cut at the first `#[cfg(test)]`, for the same reason.
    """
    src = open(path, encoding="utf-8", errors="replace").read().splitlines()
    cut = len(src)
    for i, line in enumerate(src):
        if "#[cfg(test)]" in line:
            cut = i
            break
    fn_at = []          # (line_index, name), ascending
    for i, line in enumerate(src[:cut]):
        m = FN.match(line)
        if m:
            fn_at.append((i, m.group(1)))

    def enclosing(i):
        name = "?"
        for j, n in fn_at:
            if j <= i:
                name = n
            else:
                break
        return name

    # A match arm's pattern may WRAP: `0x05 | ... | 0x1F\n | 0x20 | ... => {`.
    # A first cut of this scanner was single-line and therefore reported the six
    # relational opcodes `1F..24` as having NO reader in `control_flow::operand`,
    # when they are on the continuation line of the arm above.  Join first.
    joined = []
    i = 0
    n = len(src[:cut])
    while i < n:
        line = src[i]
        start = i
        acc = line
        while ("=>" not in acc and CONT.match(acc)
               and i + 1 < n and CONT_NEXT.match(src[i + 1])):
            i += 1
            acc = acc.rstrip() + " " + src[i].strip()
        joined.append((start, acc))
        i += 1

    out = []
    for i, line in joined:
        stripped = line.strip()
        if stripped.startswith("//"):
            continue
        m2 = ARM.match(line)
        if not m2:
            continue
        fn_name = enclosing(i)
        if fn_name in EXCLUDED_FNS:
            continue
        pat = m2.group(1)
        ops = set()
        for a, b in RANGE.findall(pat):
            ops.update(range(int(a, 16), int(b, 16) + 1))
        # a `..=` consumes both of its endpoints; plain `|` members are
        # whatever LIT finds that a range did not already claim
        for v in LIT.findall(RANGE.sub("", pat)):
            ops.add(int(v, 16))
        for o in sorted(ops):
            out.append((o, i + 1, fn_name, bool(m2.group(2)), stripped[:110]))
    return out


def collect():
    rows = {}
    for root in CRATES:
        for dirpath, _dirs, files in os.walk(root):
            for f in sorted(files):
                if not f.endswith(".rs"):
                    continue
                p = os.path.join(dirpath, f)
                rel = os.path.relpath(p, ROOT).replace(os.sep, "/")
                if rel in EXCLUDED_FILES:
                    continue
                for op, ln, fn, guarded, text in scan_file(p):
                    if (rel, ln) in EXCLUDED_LINES:
                        continue
                    rows.setdefault(op, []).append((rel, ln, fn, guarded, text))
    return rows


def coverage(rows, tsv_path):
    """Join the port sites onto the arm table dump_ilarms.py --tsv produced."""
    arms = []
    for line in open(tsv_path, encoding="utf-8"):
        if line.startswith("arm\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 5:
            continue
        arms.append((int(f[0]), f[1], int(f[2]),
                     [int(x, 16) for x in f[3].split()] if f[3] else [],
                     f[4] == "1"))
    real = [a for a in arms if not a[4]]
    refusal = [a for a in arms if a[4]]
    n_ref_ops = sum(a[2] for a in refusal)
    covered_arms = 0
    print(f"{'arm':>3}  {'target':>10}  {'ops':>3}  {'hit':>3}  files")
    for k, tgt, n, ops, ref in arms:
        if ref:
            continue
        hit = [o for o in ops if o in rows]
        if hit:
            covered_arms += 1
        files = sorted({r for o in hit for r, _l, _f, _g, _t in rows[o]})
        short = ",".join(f.rsplit("/", 1)[-1] for f in files)
        print(f"{k:>3}  {tgt:>10}  {n:>3}  {len(hit):>3}  {short}")
    handled = sum(a[2] for a in real)
    hit_ops = sum(1 for a in real for o in a[3] if o in rows)
    ref_hit = sorted(o for a in refusal for o in a[3] if o in rows)
    print()
    print(f"arms with >= 1 port site      {covered_arms} of {len(real)} real arms")
    print(f"arms with NO port site        {len(real) - covered_arms} of {len(real)}")
    print(f"handled opcodes with a site   {hit_ops} of {handled}")
    print(f"REFUSED opcodes with a site   {len(ref_hit)} of {n_ref_ops}"
          f"   {' '.join(f'{o:#04x}' for o in ref_hit)}")
    out_of_domain = sorted(o for o in rows if o < 0x01 or o > 0xBD)
    print(f"port literals outside the dispatch domain 0x01..0xbd: "
          f"{len(out_of_domain)}  {' '.join(f'{o:#04x}' for o in out_of_domain)}")

    # V5: is the port's decode concentrated in one file?
    per_file = {}
    for k, _t, _n, ops, ref in arms:
        if ref:
            continue
        for f in {r for o in ops if o in rows for r, *_ in rows[o]}:
            per_file.setdefault(f, set()).add(k)
    print("\narms reachable from each port file (an arm may appear in several):")
    for f, ks in sorted(per_file.items(), key=lambda kv: (-len(kv[1]), kv[0])):
        print(f"  {len(ks):>3} of {len(real)}   {f}")

    # The operand class is a CROSS-CHECK on the port's own width readings:
    # opcodes sharing a class share a payload grammar, so a class the port reads
    # two different ways -- or covers only partly -- is a place the port's widths
    # were pinned per opcode from witnesses instead of from the grammar.
    by_class = {}
    for line in open(tsv_path, encoding="utf-8"):
        if line.startswith("arm\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 6 or f[4] == "1":
            continue
        ops = [int(x, 16) for x in f[3].split()]
        cls = f[5].split()
        for o, c in zip(ops, cls):
            by_class.setdefault(c, []).append(o)
    print("\noperand class (raw read of 0x10b25e48) vs port coverage, "
          "over the 95 HANDLED opcodes only:")
    split = 0
    for c in sorted(by_class):
        ops = sorted(by_class[c])
        hit = [o for o in ops if o in rows]
        mark = ""
        if hit and len(hit) != len(ops):
            split += 1
            mark = "   <-- PARTIAL: same payload grammar, read on some and not others"
        print(f"  class {c}  {len(hit)}/{len(ops)}   "
              f"{' '.join(('%02x' % o) + ('' if o in rows else '-') for o in ops)}{mark}")
    print(f"  {split} of {len(by_class)} classes are covered only partly")

    # The prereg's verdicts, applied mechanically.
    tally, wonly = {}, []
    for k, _t, _n, ops, ref in arms:
        if ref:
            continue
        for o in ops:
            v = verdict(rows.get(o, []))
            tally[v] = tally.get(v, 0) + 1
            if o in rows and all(depth_of(r, fn) != "field"
                                 for r, _l, fn, _g, _t2 in rows[o]):
                wonly.append(o)
    print("\nprereg verdicts over the 95 HANDLED opcodes:")
    for v in ("MATCHED*", "NARROW(gate)", "ABSENT"):
        print(f"  {v:<14} {tally.get(v, 0)} of 95")
    print(f"  WIDTH-or-NAME-ONLY (no operand field survives): {len(wonly)} of 95"
          f"   {' '.join('%02x' % o for o in wonly)}")

    if "--detail" in sys.argv:
        print("\nper-arm opcode detail (HIT = the port names this byte somewhere)")
        for k, tgt, _n, ops, ref in arms:
            if ref:
                continue
            print(f"arm {k:>2} {tgt}")
            for o in ops:
                if o in rows:
                    s = " ; ".join(f"{r}:{l}:{fn}[{gate_of(r, fn)},"
                                   f"{depth_of(r, fn)}]{'(guarded)' if g else ''}"
                                   for r, l, fn, g, _ in rows[o])
                    print(f"    {o:#04x} {verdict(rows[o]):<13} {s}")
                else:
                    print(f"    {o:#04x} ABSENT")


def main():
    rows = collect()
    if "--coverage" in sys.argv:
        coverage(rows, sys.argv[sys.argv.index("--coverage") + 1])
        return
    tsv = "--tsv" in sys.argv
    if tsv:
        print("opcode\tn_sites\tsites")
    for op in sorted(rows):
        sites = rows[op]
        if tsv:
            s = " ; ".join(f"{r}:{ln}:{fn}{'(guarded)' if g else ''}"
                           for r, ln, fn, g, _ in sites)
            print(f"{op:#04x}\t{len(sites)}\t{s}")
        else:
            print(f"{op:#04x}  ({len(sites)} site(s))")
            for r, ln, fn, g, text in sites:
                print(f"    {r}:{ln}  fn {fn}{'  [GUARDED]' if g else ''}")
                print(f"        {text}")
    if not tsv:
        print(f"\n{len(rows)} distinct opcode literals over "
              f"{sum(len(v) for v in rows.values())} sites")


if __name__ == "__main__":
    main()
