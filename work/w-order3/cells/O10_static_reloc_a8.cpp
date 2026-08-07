struct __declspec(align(8)) A{int a;};
static A g;
A* p = &g;
