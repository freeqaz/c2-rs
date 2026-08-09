struct Q { char *a; char *b; };
void wpool_two_members(Q *q, void *v) {
    q->a = (char *)v;
    q->b = (char *)v;
}
