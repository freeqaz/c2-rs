// THE FALSIFICATION CELL. DECLARED: the UNION of p1..p4's operators,
// {op:1F, op:0B, op:0A, op:38}, and nothing else. If the instrument reports a
// SUBSET it cannot see a conjunction; if it reports an operator that is in no
// component, it is desynchronising.
int p5(int a, int b) {
    if (a == b) return a & b;
    return a >> 1;
}
