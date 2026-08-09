/* wb-label — LABEL_COUNTER.md §4.2.2's byte-identical triple, one TU.
   All three loop bodies emit the same 24 bytes at /O1 and charge +1, +3, +1;
   p_mulli emits 8 branch-free bytes and charges +2. */
int ga(int);
int a0(int a){ return ga(a)+1; }
int p_dowhile(int a){ int r=0; do { r=r+a; a=a-1; } while (a); return r; }
int a1(int a){ return ga(a)+2; }
int p_forever(int a){ int r=0; for(;;){ r=r+a; a=a-1; if(!a) break; } return r; }
int a2(int a){ return ga(a)+3; }
int p_goto(int a){ int r=0; top: r=r+a; a=a-1; if (a) goto top; return r; }
int a3(int a){ return ga(a)+4; }
int p_mulli(int a){ int r=0; for (int i=0;i<10;i++) r=r+a; return r; }
int a4(int a){ return ga(a)+5; }
