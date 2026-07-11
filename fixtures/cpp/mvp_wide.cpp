// W3b (wide immediates): constants beyond a signed 16-bit field. reg+wide and
// reg-wide use addis+addi (sign-compensated); a bare wide constant uses the
// lis+ori idiom (addis+ori). See docs/CODEGEN_PPC_MVP.md.
int addw(int a) {
    return a + 70000;
}

int subw(int a) {
    return a - 70000;
}

int kw() {
    return 70000;
}
