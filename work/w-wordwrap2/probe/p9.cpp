// P9 — the .bss object declared FIRST but laid out SECOND (wordwrap's own
// permutation): a small scalar declared before a large array.
unsigned int g_first;
unsigned int g_arr[146];
void S1(unsigned int x) { g_first = x; }
void S2(unsigned int x) { g_arr[0] = x; }
