// FRESH: __declspec(dllexport) — does it change Selection or the COMDAT?
struct __declspec(dllexport) Ex { Ex(); virtual void f(); int e; };
Ex::Ex(){}
