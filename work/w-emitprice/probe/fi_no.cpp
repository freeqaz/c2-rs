typedef unsigned int wf2f_usz;
typedef int (*wf2f_outfn_t)(void);

int *wf2f_lasterr(void);
void wf2f_report(void);
int wf2f_helper(wf2f_outfn_t, char *, wf2f_usz, const char *, void *, void *);
int wf2f_outfn(void);

              int wf2f_big(char *buffer, wf2f_usz sizeInBytes, const char *format, void *locale,
                    void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = wf2f_lasterr();
        err_val = 22;
    } else {
        result = wf2f_helper(wf2f_outfn, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) {
            buffer[0] = 0;
        }
        if (result != -2) {
            return result;
        }
        err_ptr = wf2f_lasterr();
        err_val = 34;
    }
    *err_ptr = err_val;
    wf2f_report();
    return -1;
}

int wf2f_wrap(char *buffer, wf2f_usz sizeInBytes, const char *format, void *argptr) {
    return wf2f_big(buffer, sizeInBytes, format, 0, argptr);
}
