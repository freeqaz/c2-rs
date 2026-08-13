#!/usr/bin/env python3
"""dagorder_sim.py — reference simulator for c2's within-region list scheduler.

Implements the mechanism read by lane wb-dagorder from the flat export of
c2.dll (see WB_DAGORDER_FINDINGS.md):

  * dependence DAG per region (FUN_10b328da, dag.c band);
  * priority = (height << 13) + (fanout << 8)  [the sym-dest term is constant
    across every node of these cells and is omitted]  (FUN_10be5df6, weights
    at 0x10c3bf9c); height = 1 + max(succ.height + edge.latency);
  * ready list sorted by (priority desc, original index asc) (FUN_10be5cea);
  * cycle-driven issue (FUN_10be60c0): per cycle up to WIDTH nodes, each the
    first ready node whose earliest-start <= cycle and whose unit is free
    under the variant's constraint; successors' earliest-start updated with
    the edge latency;
  * latencies (FUN_10c1c1d4, matrix 0x10c3c1a8): ALU->ALU 2, ALU->mem
    (address) 5, ALU->store (data) 2, load->ALU 2, cmp->branch 2,
    ALU->branch 0, barrier edges 0.

The grid cells of docs/whitebox/grids/wb-dagorder/dagorder_grid.cpp are
hand-encoded below from their C source; EXPECTED is the order really emitted
by cl.exe 16.00.11886.00 at /O1 (listing + obj agree). The simulator grades
each micro-model variant (issue width x memory-unit constraint) against every
cell and prints per-variant scores. Run:  python3 dagorder_sim.py
"""

LAT_ADDR = 5   # ALU result consumed as a memory op's address
LAT_DATA = 2   # ALU result consumed as a store's data
LAT_ALU = 2    # ALU -> ALU
LAT_LOAD = 2   # load -> ALU
LAT_CMPBR = 2  # cmp -> branch
LAT_BR = 0     # anything else -> branch (barrier)


class N:
    def __init__(self, name, cls):
        self.name, self.cls = name, cls  # cls: 'alu' | 'load' | 'store' | 'cmp' | 'br'
        self.succ = []                   # (node, lat)
        self.pred = 0
        self.idx = 0
        self.height = 0
        self.start = 0


def edge(a, b, lat):
    a.succ.append((b, lat))
    b.pred += 1


def build(cellfn):
    nodes = cellfn()
    for i, n in enumerate(nodes):
        n.idx = i
    # heights (reverse topological: iterate until fixpoint, lists are tiny)
    for _ in range(len(nodes)):
        for n in nodes:
            h = 1 + max((s.height + l for s, l in n.succ), default=0)
            n.height = h
    return nodes


def fanout(n):
    return len(n.succ)


def prio(n):
    return (n.height << 13) + (fanout(n) << 8)


def schedule(nodes, width, mem_per_cycle):
    ready = [n for n in nodes if n.pred == 0]
    pending = {n: n.pred for n in nodes if n.pred > 0}
    out, cycle = [], 0
    while ready or pending:
        ready.sort(key=lambda n: (-prio(n), n.idx))
        mem_used = 0
        issued = 0
        while issued < width:
            pick = None
            for n in ready:
                if n.start > cycle:
                    continue
                if n.cls in ("load", "store") and mem_used >= mem_per_cycle:
                    continue
                pick = n
                break
            if pick is None:
                break
            ready.remove(pick)
            out.append(pick.name)
            if pick.cls in ("load", "store"):
                mem_used += 1
            issued += 1
            for s, lat in pick.succ:
                s.start = max(s.start, cycle + lat)
                pending[s] -= 1
                if pending[s] == 0:
                    del pending[s]
                    ready.append(s)
            ready.sort(key=lambda n: (-prio(n), n.idx))
        cycle += 1
        if cycle > 10000:
            raise RuntimeError("no progress")
    return out


# ---------------------------------------------------------------------------
# Cell encodings. Original (pre-scheduler) index order is the statement-major
# lowering order with the RIGHT operand's chain first inside a statement
# (established by dg_sub/dg_sub2) and, for `gO = gS + c` statements, the
# variant [hS, hD, L, A, S] within the statement (hD directly after hS is NOT
# assumed — it is placed where each variant says; the variant that reproduces
# every no-CSE cell is [hS, L, A, hD, S], see results).
# ---------------------------------------------------------------------------

def stmt(nodes, tag, order):
    """gOx = gSx + c: hS -> L -> A -> S, hD -> S."""
    hS = N("h" + tag[0], "alu")
    L = N("L" + tag[0], "load")
    A = N("A" + tag[1], "alu")
    hD = N("h" + tag[1], "alu")
    S = N("S" + tag[1], "store")
    edge(hS, L, LAT_ADDR)
    edge(L, A, LAT_LOAD)
    edge(A, S, LAT_DATA)
    edge(hD, S, LAT_ADDR)
    m = {"hS": hS, "L": L, "A": A, "hD": hD, "S": S}
    nodes.extend(m[k] for k in order)
    return m


ORDER_V = ["hS", "L", "A", "hD", "S"]   # within-statement lowering order


def cell_one():
    ns = []
    stmt(ns, "a0", ORDER_V)
    return ns


def cell_two():
    ns = []
    stmt(ns, "a0", ORDER_V)
    stmt(ns, "b1", ORDER_V)
    return ns


def cell_v1():
    ns = []
    for t in ("a0", "b1", "c2"):
        stmt(ns, t, ORDER_V)
    return ns


def cell_v4():
    ns = []
    for t in ("a0", "b1", "c2", "d3"):
        stmt(ns, t, ORDER_V)
    return ns


def cell_sub():
    # dg_o0 = dg_b - dg_c, right chain (c) first
    ns = []
    hc, Lc = N("hc", "alu"), N("Lc", "load")
    hb, Lb = N("hb", "alu"), N("Lb", "load")
    OP, h0, S = N("OP", "alu"), N("h0", "alu"), N("S0", "store")
    edge(hc, Lc, LAT_ADDR)
    edge(hb, Lb, LAT_ADDR)
    edge(Lc, OP, LAT_LOAD)
    edge(Lb, OP, LAT_LOAD)
    edge(OP, S, LAT_DATA)
    edge(h0, S, LAT_ADDR)
    ns += [hc, Lc, hb, Lb, OP, h0, S]
    return ns


def cell_chain():
    # dg_o0 = dg_b + dg_c + dg_d, REASSOCIATED to (d + c) + b (observed);
    # chains in emitted-tie order d, c, b
    ns = []
    hd, Ld = N("hd", "alu"), N("Ld", "load")
    hc, Lc = N("hc", "alu"), N("Lc", "load")
    hb, Lb = N("hb", "alu"), N("Lb", "load")
    O1, O2 = N("OP1", "alu"), N("OP2", "alu")
    h0, S = N("h0", "alu"), N("S0", "store")
    for h, L in ((hd, Ld), (hc, Lc), (hb, Lb)):
        edge(h, L, LAT_ADDR)
    edge(Ld, O1, LAT_LOAD)
    edge(Lc, O1, LAT_LOAD)
    edge(O1, O2, LAT_ALU)
    edge(Lb, O2, LAT_LOAD)
    edge(O2, S, LAT_DATA)
    edge(h0, S, LAT_ADDR)
    ns += [hd, Ld, hc, Lc, hb, Lb, O1, O2, h0, S]
    return ns


def cell_lit():
    # dg_o0 = 1; dg_o1 = 2; dg_o2 = 1  (no CSE of the literal — observed)
    ns = []
    for k in "012":
        h, li, S = N("h" + k, "alu"), N("li" + k, "alu"), N("S" + k, "store")
        edge(li, S, LAT_DATA)
        edge(h, S, LAT_ADDR)
        ns += [li, h, S]
    return ns


def cell_if():
    # dg_o0 = dg_d + 1; if (dg_a) {...}  — entry region ends at the branch
    ns = []
    m = stmt(ns, "d0", ORDER_V)
    ha, La = N("ha", "alu"), N("La", "load")
    C, B = N("CMP", "cmp"), N("BR", "br")
    edge(ha, La, LAT_ADDR)
    edge(La, C, LAT_LOAD)
    edge(C, B, LAT_CMPBR)
    # the branch is the region terminator: barrier edges (latency 0) from
    # every node with no other successor (FUN_10b3286b)
    for n in ns:
        if not n.succ:
            edge(n, B, LAT_BR)
    edge(m["S"], B, LAT_BR)
    ns += [ha, La, C, B]
    return ns


EXPECTED = {
    "one": ["ha", "h0", "La", "A0", "S0"],
    "two": ["ha", "hb", "h0", "h1", "La", "Lb", "A0", "A1", "S0", "S1"],
    "v1": ["ha", "hb", "hc", "h0", "h1", "La", "h2", "Lb", "Lc",
            "A0", "A1", "A2", "S0", "S1", "S2"],
    "v4": ["ha", "hb", "hc", "hd", "h0", "La", "h1", "Lb", "h2", "A0",
            "Lc", "Ld", "h3", "S0", "A1", "A2", "A3", "S1", "S2", "S3"],
    "sub": ["hc", "hb", "h0", "Lc", "Lb", "OP", "S0"],
    "chain": ["hd", "hc", "hb", "h0", "Ld", "Lc", "Lb", "OP1", "OP2", "S0"],
    "lit": ["h0", "h1", "h2", "li0", "li1", "li2", "S0", "S1", "S2"],
    "if": ["hd", "ha", "h0", "Ld", "La", "A0", "CMP", "S0", "BR"],
}

CELLS = {
    "one": cell_one, "two": cell_two, "v1": cell_v1, "v4": cell_v4,
    "sub": cell_sub, "chain": cell_chain, "lit": cell_lit, "if": cell_if,
}


def main():
    print(f"{'variant':28s} " + " ".join(f"{c:>5s}" for c in EXPECTED) + "  exact")
    best = []
    for width in (1, 2, 4):
        for mem in (1, 2):
            marks, exact = [], 0
            for name in EXPECTED:
                got = schedule(build(CELLS[name]), width, mem)
                ok = got == EXPECTED[name]
                marks.append("ok" if ok else "X")
                exact += ok
            tag = f"width={width} mem/cycle={mem}"
            print(f"{tag:28s} " + " ".join(f"{m:>5s}" for m in marks) + f"  {exact}/{len(EXPECTED)}")
            best.append((exact, tag))
    best.sort(reverse=True)
    print("\nbest:", best[0][1], f"{best[0][0]}/{len(EXPECTED)}")
    # print the best variant's full sequences for the misses
    for name in EXPECTED:
        got = schedule(build(CELLS[name]), 2, 1)
        if got != EXPECTED[name]:
            print(f"\n{name} (width=2 mem=1):\n  got      {got}\n  expected {EXPECTED[name]}")


if __name__ == "__main__":
    main()
