// FRESH: a polymorphic class holding an array and a pointer-to-member, to see
// whether anything but the vftable trigger reaches .rdata$r.
struct Mp { Mp(); virtual void f(); int arr[4]; int Mp::*pm; };
Mp::Mp(){}
