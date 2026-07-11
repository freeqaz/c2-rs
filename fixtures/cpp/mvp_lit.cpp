// Literal / immediate fixture (W3): integer constants folded into addi. add
// with a positive immediate, subtract with an immediate (folded to addi with
// negated imm), and a bare constant return (li = addi rD,r0,k). Straight-line,
// 16-bit immediates only. See docs/CODEGEN_PPC_MVP.md.
int addk(int a) {
    return a + 5;
}

int subk(int a) {
    return a - 5;
}

int konst() {
    return 42;
}
