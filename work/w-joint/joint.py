#!/usr/bin/env python3
"""joint.py — the JOINT DATA+CODE fixpoint, and the root rules that feed it.

w-skip proved, through real `c2.dll`, that an initializer contributes roots only
when the owning DATA symbol is itself emitted (10/10 against 0/10, with
`+0x20 = 0x1c01` in both arms, so it is not a flag).  That makes the emit set a
least fixpoint over TWO sorts of symbol, and the code half alone cannot express
it:

    CODE nodes  U   names with a gate-clean tag-0x0E `.gl` record   (w-refs)
    DATA nodes  W   names of kind-1 `.gl` records that own an `in`
                    initializer record                              (w-skip)

    EDGES
      cc   f -> RGL(f)      the per-symbol reference list, function targets
                            only (w-skip T-e: `0x10b27f3c` keeps an edge only
                            for a tag-0x0E target), so there is NO code->data
                            edge and the data half cannot be reached from code
      dc   d -> f           an `02` initializer node of d naming a function
      dd   d -> d'          an `02` node of d naming another data symbol
                            (`_CTA4` -> `_CT` -> `??_R0` -> `??_7type_info`)

    ROOTS
      Rc = Seed             { f : flags4c & 0x20 }                  (w-roots)
      Rd = ???              THE FITTED PARAMETER OF THIS LANE

Because there is no cc-edge into the data half, **Rd carries the entire data
side**: whatever Rd does not name can only be reached by a dd-edge from
something Rd did name.  That is why Rd is where the fitting is, why it is
declared here in one place, and why every variant is a named member of one
enumeration rather than a tweak.

    RD VARIANTS — what varies across the fitted parameter
      ORACLE   Rd = D(t), the extended truth's defined-symbol set.  NOT a
               model: a CEILING on what any joint fixpoint can do, and the
               question "given a perfect data half, is the code half solved?"
      ALL      Rd = every owner.  This is exactly w-mark's unfiltered reading,
               reproduced here as the fixpoint's degenerate case.
      SC<k>    Rd = owners whose kind-1 storage-class nibble is in k.
      F20<m,v> Rd = owners with (f20 & m) == v.
      TAG<t>   Rd = owners whose `.gl` record tag is in t.
      NONE     Rd = {}.  The floor: the fixpoint degenerates to `P_RGL`.

`ORACLE` is reported as a ceiling and never as a model.  The others have
parameters and are graded as models.

stdlib only.  Reads no c2 output except the extended truth, which is the thing
being conditioned on and is labelled as such everywhere it is used.
"""


def closure(seed, edges, U, skip=()):
    """w-roots'/w-refs'/w-skip's operator, imported by value so the incumbents
    reproduce to the digit."""
    seen = set(x for x in seed if x in U)
    stack = list(seen)
    while stack:
        a = stack.pop()
        for f in edges.get(a, ()):
            if f not in seen and f in U and f not in skip:
                seen.add(f)
                stack.append(f)
    return seen


def owner_nodes(inrecs, syms, idx):
    """-> ({owner_name: set(target_name)}, stats).

    An `in` record's owner token is resolved through the kind-1/kind-4 `.gl`
    header decode (w-skip's `glowner`); its `02` nodes' tokens through the name
    index (w-refs/w-emit's `il.gl_symbol_index`).  Both blind spots are counted
    and reported: an owner this decoder cannot name is contributed to the
    `UNBOUND` bucket rather than silently dropped, because a decoder's blind
    spot must never be allowed to look like a filter.
    """
    own = {}
    st = {"rec": 0, "owner_unbound": 0, "node": 0, "node_unbound": 0,
          "unbound_targets": set()}
    for (_tag, _fl, otok, toks) in inrecs:
        st["rec"] += 1
        orec = syms.get(otok)
        oname = orec["name"] if orec else None
        if oname is None:
            st["owner_unbound"] += 1
        bucket = own.setdefault(oname, set())
        for t in toks:
            st["node"] += 1
            nm = idx.get(t)
            if nm is None:
                st["node_unbound"] += 1
                continue
            bucket.add(nm)
    return own, st


def data_fixpoint(own, Rd, U):
    """Least fixpoint over the DATA half, then the code names it marks.

    `own` maps a data symbol to every name its initializer nodes reference.
    Starting from `Rd`, a data symbol that is emitted marks everything its
    initializer names; a *data* name so marked is itself emitted and is walked
    in turn (the dd-edge).  Returns (emitted_data, marked_code).

    Unordered on purpose: w-skip §2 showed `0x10b98e26` has exactly one caller
    chain, runs before the compile loop, and reads no codegen-mutated field, so
    a fixpoint iterated to convergence is sufficient FOR THIS CHANNEL.  The
    ordering requirement w-mark found at `0x10b7f1e5` lives in the compile loop
    and in `0x10b3389b`/`0x10b9aa26`, which this lane does not model.
    """
    live = set(d for d in Rd if d in own)
    stack = list(live)
    code = set()
    while stack:
        d = stack.pop()
        for t in own[d]:
            if t in U:
                code.add(t)
            elif t in own and t not in live:
                live.add(t)
                stack.append(t)
    return live, code


def rd_oracle(own, D):
    return set(d for d in own if d is not None and d in D)


def rd_all(own):
    return set(d for d in own if d is not None)


def rd_flag(own, syms_by_name, mask, value):
    out = set()
    for d in own:
        r = syms_by_name.get(d)
        if r is not None and (r["f20"] & mask) == value:
            out.add(d)
    return out


def rd_sc(own, syms_by_name, classes):
    out = set()
    for d in own:
        r = syms_by_name.get(d)
        if r is not None and r["sc"] in classes:
            out.add(d)
    return out


def rd_tag(own, syms_by_name, tags):
    out = set()
    for d in own:
        r = syms_by_name.get(d)
        if r is not None and r["tag"] in tags:
            out.add(d)
    return out
