struct __declspec(align(16)) A{int a;};
static A g;
A* p = &g;
