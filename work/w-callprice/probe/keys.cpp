// w-callprice — KEY PROBE (scratch, never a fixture).
//
// Reproduces, in the smallest source that can, the constructs the workload's
// three largest EMITTED keys were sampled at, so the reading of each key is a
// measurement (does this source shape mint that key?) and not an inference from
// a hex window.
//
//   K1  expr-call-in-expr-recv-object-then-call-recv-object-more   5,608 emitted
//       dc3 `src/system/utl/MakeString.h:67` — a local FormatString, a chain of
//       `operator<<`, and `Str()`.
//   K2  expr-call-in-expr-recv-load-then-bit-and-and-branch-more   4,290 emitted
//   K3  expr-call-in-expr-recv-load-then-intrinsic-call            2,865 emitted

// ---- K1 ---------------------------------------------------------------------
class FS {
    char *mFmt;
    char mBuf[0x100];

public:
    FS(const char *);
    FS &operator<<(const char *);
    FS &operator<<(const int &);
    const char *Str();
};

template <class T1, class T2>
const char *wcp_MakeString(const char *c, const T1 &t1, const T2 &t2) {
    FS fs(c);
    fs << t1 << t2;
    return fs.Str();
}

const char *wcp_k1_user(const char (&a)[22], const int &b) {
    return wcp_MakeString("%s%d", a, b);
}

// ---- K2 — a member call guarded by a bit test --------------------------------
struct O {
    unsigned int mFlags;
    void Poll();
    void Sync(int);
    O *Next();
    float Level();
    unsigned int GetFlags();
};

void wcp_k2(O *o) {
    if (o->mFlags & 4)
        o->Poll();
}

// the member call is the LEFT operand of the bit test — `-then-bit-and-and-branch`
void wcp_k2b(O *o) {
    if (o->GetFlags() & 4)
        o->Poll();
}

// ---- K3 — a member call whose argument is a class-layout intrinsic ------------
struct Base {
    virtual ~Base();
};
struct Derived : Base {
    void Take(Base *);
};

void wcp_k3(Derived *d) { d->Take(d); }

// ---- the chained receiver in a later statement of a sequence (R2) ------------
void wcp_chain_seq(O *o) {
    o->Poll();
    o->Next()->Poll();
}

// ---- the value tail (msc-result-not-discarded) --------------------------------
float wcp_value_tail(O *o) {
    o->Poll();
    return o->Level();
}
