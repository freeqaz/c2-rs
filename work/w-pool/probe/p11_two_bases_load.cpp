struct P { char *mFree; };
void wpool_two_bases_load(P *p, void **v) {
    *v = p->mFree;
    *v = 0;
}
