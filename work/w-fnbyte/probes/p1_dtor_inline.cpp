// w-fnbyte probe 1 — the `tail|w1/1|port=48000000,ref=4e800020` signature.
// A destructor whose only work is a member's destructor, defined here and empty.
// The port's IL decode calls this a tail call and would emit `b ??1A@@QAA@XZ`;
// the question this probe asks real c2 is what it actually emits.
struct A { ~A() {} };
struct B { A a; ~B(); };
B::~B() {}
