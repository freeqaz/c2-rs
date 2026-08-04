#pragma section(".mysec", read, write)
static int cand(int x) { return x*3+1; }
__declspec(allocate(".mysec")) int (*g_p)(int) = &cand;
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
