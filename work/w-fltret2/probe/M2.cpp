// w-fltret — THE DECISIVE CELL. `Timer::SplitMs` with its callees DEFINED, which
// is what the workload's `src/system/os/Timer.h` actually is.
//
// `work/w-fltret/probe/P1.cpp` declares `S::a()` and `S::f()` and never defines
// them, so c2 cannot inline them and emits `bl · bl`. Every byte this lane read
// off P1.obj is a byte from that cell. This file changes exactly one thing — the
// callees have bodies — and it is the field that decides the answer.
//
// c2 INLINES them (`docs/INLINE_PREDICATE.md`: "when c2 does not emit the call
// the IL contains"), so the reference body has no `bl` in it at all, while the
// IL still spells two calls and the reader still accepts them.
struct Timer {
    unsigned int mCycles;
    unsigned int mStart;
    int mRunning;

    void Split() {
        mCycles += mStart;
        mStart = mCycles;
    }
    float Ms() { return (float)mCycles; }
    float SplitMs() {
        Split();
        return Ms();
    }
};

float m2_call(Timer *t) {
    return t->SplitMs();
}
