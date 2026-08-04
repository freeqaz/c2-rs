extern void v0(); extern void v1(); extern void v2(); extern void v3(); extern void v4();
// join length 1, 2, 3 — does the /Ox duplication have a size threshold?
void j1(int a) { if (a != 0) v0(); else v1(); v2(); }
void j2(int a) { if (a != 0) v0(); else v1(); v2(); v3(); }
void j3(int a) { if (a != 0) v0(); else v1(); v2(); v3(); v4(); }
// and the ONE-ARMED guard at the same join lengths, as the control
void k1(int a) { if (a != 0) v0(); v2(); }
void k2(int a) { if (a != 0) v0(); v2(); v3(); }
void k3(int a) { if (a != 0) v0(); v2(); v3(); v4(); }
