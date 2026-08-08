// w-5c probe — the EH LIVE-STATE marker `5C`, with the STATE as the only
// field that moves across the rows, plus the two classes a grid could wrongly
// exclude (PREREG §3): a 4-byte TYPE and an ESCAPED state varint.
//
// Compiled at the workload's own flags. Graded by `c2rs diff` against real
// `c2.dll` under wibo — this file is a CAPTURE, not a reading.

struct MemA { int a; MemA(); ~MemA(); };
struct MemB { int b0; int b1; MemB(); ~MemB(); };

int  g(int);
void h();

// R1 — one object live, one statement. The minimal `5C`.
void one_local() { MemA s; }

// R2 — two objects live. Two `5C`s, two different states, nothing else moves.
void two_locals() { MemA s; MemB t; }

// R3 — an ORDINARY function, not a generated destructor: EH_RECORDS §7.1's own
// witness that `5C` is not a ctor/dtor token.
int userfn(int a) { MemA s; g(a); return a + 1; }

// R4 — the object is live ACROSS a call, so the `5C` is followed by more
// statements rather than by the function tail.
int across(int a) { MemA s; h(); h(); return g(a); }

// R5 — a generated destructor's SUB-OBJECT statement (ctor_dtor.rs's spelling).
struct HasMem { MemA m; int k; };
void use_hasmem() { HasMem q; q.k = 1; }

// R6 — the ESCAPED state varint. The state is `2n+1` (EH_RECORDS §7.1 shows
// `80 01 01 00 00` = 257 = 2*128+1), so a body needs >= 64 live objects before
// the field leaves the single-byte range. 100 of them, so the last states are
// unambiguously escaped.
void many_locals() {
    MemA s0;
    MemA s1;
    MemA s2;
    MemA s3;
    MemA s4;
    MemA s5;
    MemA s6;
    MemA s7;
    MemA s8;
    MemA s9;
    MemA s10;
    MemA s11;
    MemA s12;
    MemA s13;
    MemA s14;
    MemA s15;
    MemA s16;
    MemA s17;
    MemA s18;
    MemA s19;
    MemA s20;
    MemA s21;
    MemA s22;
    MemA s23;
    MemA s24;
    MemA s25;
    MemA s26;
    MemA s27;
    MemA s28;
    MemA s29;
    MemA s30;
    MemA s31;
    MemA s32;
    MemA s33;
    MemA s34;
    MemA s35;
    MemA s36;
    MemA s37;
    MemA s38;
    MemA s39;
    MemA s40;
    MemA s41;
    MemA s42;
    MemA s43;
    MemA s44;
    MemA s45;
    MemA s46;
    MemA s47;
    MemA s48;
    MemA s49;
    MemA s50;
    MemA s51;
    MemA s52;
    MemA s53;
    MemA s54;
    MemA s55;
    MemA s56;
    MemA s57;
    MemA s58;
    MemA s59;
    MemA s60;
    MemA s61;
    MemA s62;
    MemA s63;
    MemA s64;
    MemA s65;
    MemA s66;
    MemA s67;
    MemA s68;
    MemA s69;
    MemA s70;
    MemA s71;
    MemA s72;
    MemA s73;
    MemA s74;
    MemA s75;
    MemA s76;
    MemA s77;
    MemA s78;
    MemA s79;
    MemA s80;
    MemA s81;
    MemA s82;
    MemA s83;
    MemA s84;
    MemA s85;
    MemA s86;
    MemA s87;
    MemA s88;
    MemA s89;
    MemA s90;
    MemA s91;
    MemA s92;
    MemA s93;
    MemA s94;
    MemA s95;
    MemA s96;
    MemA s97;
    MemA s98;
    MemA s99;
}

MemA::MemA() { }
MemA::~MemA() { }
MemB::MemB() { }
MemB::~MemB() { }
