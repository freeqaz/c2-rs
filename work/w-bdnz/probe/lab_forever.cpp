// Two SEPARATING controls in one file is not possible, so this slot carries the
// one that separates the loop from its LOCALS: a straight-line leaf with the
// same two `int` locals the counted loop declares and no loop at all.
int gz(int);
int lead(int n, int k) { int s = 0; int i = k; s -= i; return s; }
int z9(int a) { return gz(a) + 7; }
