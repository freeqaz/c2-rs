// w-5c probe — the SMALLEST `5C`-bearing TU whose functions the PORT accepts,
// captured fresh at THIS master.
//
// `docs/EH_RECORDS.md` §7.2 measured the generated destructor of a class with
// one destructible member as census key `empty-dtor-member` (**in class**) and
// EH class `eh-bare`, with no `__ehfuncinfo$` in the obj. The committed fixture
// `fixtures/cpp/w15_dtor_member.cpp` transcribes that body in its own header and
// the transcription contains `5c 86 41 74 11` — so the token under test sits
// inside a shape the port has emitted byte-exact since that fixture landed.
//
// The member's destructor is DECLARED and NOT DEFINED, for `w15`'s own stated
// reason: c2 may inline a callee it can also see, and the point here is a real
// call whose statement carries the live-state marker.
//
// The grade to read off this file is `Port=Match` — the port emitting an obj
// byte-exact against real `c2.dll` for a TU every one of whose bodies contains a
// `5C`. Three offsets so the `5C` is not pinned beside one receiver form.

struct Mem { ~Mem(); int x; };

struct At0  { ~At0();  Mem m; };
struct At4  { ~At4();  int pad; Mem m; };
struct At8  { ~At8();  double d; Mem m; };

At0::~At0() {}
At4::~At4() {}
At8::~At8() {}
