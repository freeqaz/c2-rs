#!/usr/bin/env python3
"""keymap.py — board #3509: map Phase 1's ten constructs to census keys and
publish each one's TU-denominated reach.

NOT A RANKING. Output is ordered C1..C10 and, within a construct, by key NAME.

The mapping is a READ. Every edge below is the inverse of a renderer in
`crates/c2-il/src/func/body/mod.rs` (`Block::feature`, `expr_opcode_name`,
`cflow_opcode_name`) joined to the arm in
`crates/c2-il/src/func/body/shapes/control_flow.rs` (`Scan::off_class`) that
gives the construct its NAME. Nothing here is inferred from a count.
"""
import json, sys, re, collections

SCAN = sys.argv[1] if len(sys.argv) > 1 else 'work/w-keymap/scan.jsonl'

# ---- inverse of expr_opcode_name (mod.rs:1816) -------------------------------
EXPR_NAME = {
    'cmp-eq': 0x1F, 'cmp-ne': 0x20, 'cmp-le': 0x21, 'cmp-lt': 0x22,
    'cmp-ge': 0x23, 'cmp-gt': 0x24, 'not': 0x1A, 'or-or': 0x1B,
    'and-and': 0x1C, 'shl': 0x09, 'shr': 0x0A, 'bit-and': 0x0B,
    'bit-or': 0x0C, 'bit-xor': 0x0D, 'convert': 0x2C,
    'intrinsic-call': 0x40, 'class-descriptor': 0x66, 'ternary': 0x43,
    'call-in-expr': 0x26,
}
# ---- inverse of cflow_opcode_name (mod.rs:1790) ------------------------------
CFLOW_NAME = {'label': 0x29, 'brfalse': 0x38, 'brtrue': 0x39, 'jump': 0x3A,
              'switch-dispatch': 0x3B, 'switch-table': 0x3C, 'switch-case': 0x3D}

# ---- the FOUR type-block ctxs the census route uses (grep of blk_type) -------
# expr.rs:1989 expr-load-type · expr.rs:2025 expr-lit-type ·
# expr-convert-target · assign-store-type
TYPE_CTX_OPCODE = {
    'expr-load-type':      0xB9,   # LOAD  `B9 <tok> <TYPE>`
    'expr-lit-type':       0x33,   # LIT   `33 <TYPE> <payload>`
    'expr-convert-target': 0x2C,   # CONVERT `2C <TYPE> <varint>`
    'assign-store-type':   0x32,   # STORE `32 <TYPE>`
}

HEX = re.compile(r'-0x([0-9A-F]{2})$')
TYPED_OP = re.compile(r'^expr-op-0x([0-9A-F]{2})-(?:[0-9A-F]{4}|notype)$')
TYPE_TAIL = re.compile(r'^(.*)-([0-9A-F]{4})$')


def head_byte(key):
    """The opcode the key's FIRST blocker sits on, or None when the key names
    no byte at all. Exactly inverts Block::feature's dispatch order."""
    if key.startswith('opt-mode-'):
        return None, 'opt-mode'                     # feature() arm 1
    if key.startswith('expr-intrinsic-') or key.startswith('call-intrinsic-'):
        return 0x40, 'intrinsic-selector'           # arm 2
    m = TYPED_OP.match(key)
    if m:
        return int(m.group(1), 16), 'typed-op'      # arm 3 (EXPR_TYPED_OP)
    if key.startswith('expr-call-in-expr-'):
        return 0x26, 'mcall'                        # arm 4 (CALL_IN_EXPR)
    m = TYPE_TAIL.match(key)
    if m and m.group(1) in TYPE_CTX_OPCODE:
        return TYPE_CTX_OPCODE[m.group(1)], 'type-block'   # arm 5 (aux != 0)
    if key.endswith(':eof') or key.endswith(':mid'):
        return None, 'byteless'                     # arm 6
    if key.startswith('expr-'):                     # arm 7 (ctx == "expr")
        n = key[len('expr-'):]
        if n in EXPR_NAME:
            return EXPR_NAME[n], 'expr-named'
        if n in CFLOW_NAME:
            return CFLOW_NAME[n], 'expr-named-cflow'
    if '-cflow-' in key:                            # arm 8
        n = key.rsplit('-cflow-', 1)[1]
        if n in CFLOW_NAME:
            return CFLOW_NAME[n], 'cflow'
    m = HEX.search(key)
    if m:
        return int(m.group(1), 16), 'ctx-hex'       # arm 9 (fall-through)
    return None, 'UNCLASSIFIED'


# ---- Phase 1's ten, each from the off_class arm that NAMES it ---------------
# control_flow.rs line numbers are the `s.off_class("<reason>")` call site.
PHASE1 = [
    ('C1',  'off-add',        {0x27},                          'control_flow.rs:941'),
    ('C2',  'intrinsic',      {0x40},                          'control_flow.rs:1057'),
    ('C3',  'bind',           {0x99},                          'control_flow.rs:1168'),
    ('C4',  'load-type',      {0xB9},                          'control_flow.rs:823'),
    ('C5',  'temp',           {0x9B},                          'control_flow.rs:1174'),
    ('C6',  'lit-type',       {0x33},                          'control_flow.rs:833'),
    ('C7',  'compare',        {0x1F,0x20,0x21,0x22,0x23,0x24}, 'control_flow.rs:885'),
    ('C8',  'bitwise',        {0x0B,0x0C,0x0D,0x0E},           'control_flow.rs:885'),
    ('C9',  'materialize-64', {0x64},                          'control_flow.rs:1157'),
    ('C10', 'virtual-slot',   {0x67},                          'control_flow.rs:1114'),
]
# C4/C6 are TYPE PREDICATES, not bare opcodes: a `B9`/`33` whose TYPE is inside
# int4/ptr4 is IN CLASS and raises nothing. So their key set is the type-block
# family at that ctx, and a bare `*-0xB9` / `*-0x33` hex key is a DIFFERENT
# refusal (a malformed/short token), which is recorded separately below.
TYPE_PREDICATE = {'C4': 'expr-load-type', 'C6': 'expr-lit-type'}

rows = [json.loads(l) for l in open(SCAN) if l.strip()]
prov = [r for r in rows if r.get('record') == 'provenance'][0]
data = [r for r in rows if r.get('record') != 'provenance']

bodies = collections.Counter()
key_tus = collections.Counter()
tu_keyset = {}
for r in data:
    fb = r['fn_blockers']
    tu_keyset[r['src']] = set(fb)
    for k, v in fb.items():
        bodies[k] += v
        key_tus[k] += 1

TOT_BODIES = sum(bodies.values())
TOT_KEYS = len(bodies)
TOT_TUS = len(data)
matching = [r['src'] for r in data if r['class'] == 'match']
nonmatch = [r['src'] for r in data if r['class'] != 'match' and tu_keyset[r['src']]]

# classify every key once
cls = {k: head_byte(k) for k in bodies}
unclassified = sorted(k for k in cls if cls[k][1] == 'UNCLASSIFIED')

construct_keys = {}
for cid, reason, ops, site in PHASE1:
    if cid in TYPE_PREDICATE:
        pre = TYPE_PREDICATE[cid] + '-'
        ks = {k for k in bodies if k.startswith(pre) and TYPE_TAIL.match(k)}
    else:
        ks = {k for k in bodies if cls[k][0] in ops and cls[k][1] in
              ('ctx-hex', 'expr-named', 'typed-op', 'intrinsic-selector')}
    construct_keys[cid] = ks

UNION = set().union(*construct_keys.values())

print(f"# workload {prov['workload_head'][:9]} dirty={prov['workload_dirty']} · "
      f"c2rs {prov['c2rs_head'][:9]} · binary {prov['binary_sha'][:12]}")
print(f"# DENOMINATORS: TUs {TOT_TUS} · match {len(matching)} · "
      f"TUs with >=1 blocked body {len(nonmatch)} · "
      f"blocked bodies {TOT_BODIES} · distinct blocker keys {TOT_KEYS}")
print()

print("== A. MAPPING (C1..C10, keys by NAME within a construct) ==")
for cid, reason, ops, site in PHASE1:
    ks = sorted(construct_keys[cid])
    b = sum(bodies[k] for k in ks)
    t = len({s for s in tu_keyset if any(k in tu_keyset[s] for k in ks)})
    ops_s = '/'.join(f'0x{o:02X}' for o in sorted(ops))
    print(f"\n{cid} {reason}  opcode(s) {ops_s}  [{site}]")
    print(f"  keys {len(ks)} · heads {b} of {TOT_BODIES} blocked bodies "
          f"({100*b/TOT_BODIES:.2f}%) · appears in {t} of {TOT_TUS} TUs")
    if not ks:
        print("  *** ZERO census keys — this construct heads NO blocked body ***")
    for k in ks:
        print(f"    {k}\t{bodies[k]}\t{key_tus[k]}")

print("\n== B. UNION ==")
ub = sum(bodies[k] for k in UNION)
print(f"Phase 1 union: {len(UNION)} of {TOT_KEYS} keys · heads {ub} of "
      f"{TOT_BODIES} blocked bodies ({100*ub/TOT_BODIES:.2f}%)")

print("\n== C. TU-DENOMINATED REACH (the number #3509 asks for) ==")
inside = [s for s in nonmatch if tu_keyset[s] <= UNION]
print(f"TUs whose ENTIRE blocked-body head-key set lies inside the union: "
      f"{len(inside)} of {len(nonmatch)}")
if inside:
    for s in sorted(inside):
        print(f"   {s}  floor {len(tu_keyset[s])}")

# per-construct: TUs entirely inside ONE construct
print("\nTUs entirely inside a SINGLE construct:")
for cid, reason, ops, site in PHASE1:
    ks = construct_keys[cid]
    n = sum(1 for s in nonmatch if tu_keyset[s] <= ks)
    print(f"   {cid} {reason}: {n} of {len(nonmatch)}")

# relaxed: allow every byteless / complete-* key as free
FREE = {k for k in bodies if cls[k][1] in ('byteless', 'opt-mode')}
inside2 = [s for s in nonmatch if tu_keyset[s] <= (UNION | FREE)]
print(f"\nRELAXED (union + all {len(FREE)} byteless/:eof/:mid/opt-mode keys free): "
      f"{len(inside2)} of {len(nonmatch)}")
if inside2:
    for s in sorted(inside2):
        print(f"   {s}  floor {len(tu_keyset[s])}")

print("\n== D. RESIDUAL FLOOR after deleting every Phase-1 key ==")
def stats(vals):
    v = sorted(vals); n = len(v)
    return (v[0], v[n//10], v[n//2], v[(9*n)//10], v[-1])
full = [len(tu_keyset[s]) for s in nonmatch]
resid = [len(tu_keyset[s] - UNION) for s in nonmatch]
resid2 = [len(tu_keyset[s] - UNION - FREE) for s in nonmatch]
for label, v in (('floor (all keys)', full), ('residual, minus Phase 1', resid),
                 ('residual, minus Phase 1 and byteless', resid2)):
    mn, p10, med, p90, mx = stats(v)
    print(f"  {label:38s} min {mn:4d} p10 {p10:4d} median {med:4d} "
          f"p90 {p90:4d} max {mx:4d} · >50: {sum(1 for x in v if x>50)} of {len(v)}")

print("\n== E. POSITIVE CONTROL (K4) ==")
# The instrument must be able to print something other than 0. Feed it, per TU,
# that TU's OWN floor set as the "granted" set: every TU must come back inside.
ctrl = sum(1 for s in nonmatch if tu_keyset[s] <= tu_keyset[s])
print(f"  each TU against its OWN floor set: {ctrl} of {len(nonmatch)} inside "
      f"(must equal {len(nonmatch)})")
# and a middle magnitude: the union of the 20 smallest-floor TUs' key sets
small = sorted(nonmatch, key=lambda s: len(tu_keyset[s]))[:20]
G = set().union(*[tu_keyset[s] for s in small])
mid = sum(1 for s in nonmatch if tu_keyset[s] <= G)
print(f"  against the union of the 20 smallest floors ({len(G)} keys): {mid} inside")

print("\n== F. THE TUs AT SMALL FLOOR, PRINTED ENTIRE (never ranked by mass) ==")
for s in sorted(nonmatch, key=lambda x: (len(tu_keyset[x]), x))[:16]:
    ks = sorted(tu_keyset[s])
    tag = {k: next((c for c,_,_,_ in PHASE1 if k in construct_keys[c]), '--') for k in ks}
    print(f"  {s}  floor {len(ks)}")
    for k in ks:
        print(f"      [{tag[k]}] {k}\t{sum(1 for _ in ())or bodies[k]}")

print("\n== G. KEYS NOT CLASSIFIED BY THE READ ==")
print(f"  UNCLASSIFIED: {len(unclassified)}")
for k in unclassified:
    print(f"    {k}\t{bodies[k]}")
byclass = collections.Counter(cls[k][1] for k in bodies)
print("  key population by renderer arm (K5: denominators):")
for a, n in sorted(byclass.items()):
    bb = sum(bodies[k] for k in bodies if cls[k][1] == a)
    print(f"    {a:20s} keys {n:4d}  bodies {bb:8d} ({100*bb/TOT_BODIES:5.2f}%)")

print("\n== H. NON-PHASE-1 opcodes that DO head blocked bodies ==")
# every distinct head byte outside the ten, with the off_class reason if any
P1OPS = set().union(*[o for _,_,o,_ in PHASE1])
byte_bodies = collections.Counter()
for k in bodies:
    b, arm = cls[k]
    if b is not None and arm in ('ctx-hex','expr-named','typed-op','intrinsic-selector'):
        byte_bodies[b] += bodies[k]
for b in sorted(byte_bodies):
    if b in P1OPS: continue
    print(f"    0x{b:02X}\t{byte_bodies[b]}")
