int gz(int);
extern "C" void *memcpy(void *, const void *, unsigned int);
struct MI { char *pad[7]; char *pchNext; char *pchEndRead; };
extern "C" long subject(void *h, void *p, unsigned int f) {
    if (h == 0) return 5;
    if (p == 0) return 11;
    memcpy(h, p, 0x48);
    MI *ni = (MI *)h;
    if (ni->pchEndRead < ni->pchNext) { ni->pchEndRead = ni->pchNext; }
    return 0;
}
int framed(int a) { return gz(a) + 7; }
