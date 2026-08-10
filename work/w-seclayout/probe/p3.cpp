struct P { virtual ~P(); };
struct Q : P { virtual ~Q() {} };
struct R : Q { R(); ~R(); };
R::R() {}
R::~R() {}
