// FRESH: virtual destructors on base AND derived — the ??_G / ??_E shape the
// spec's §8 says it does NOT price.
struct Vd0 { Vd0(); virtual ~Vd0(); virtual int f(); int a; };
struct Vd1 : Vd0 { Vd1(); virtual ~Vd1(); virtual int f(); int b; };
Vd0::~Vd0(){}
Vd1::~Vd1(){}
