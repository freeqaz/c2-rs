// x_deref_split — store form `*buffer = 0`, error arms `split`.
typedef unsigned int size_t_;
typedef int (*output_fn_t)(void);
extern "C" int *_errno(void);
int vsnprintf_helper_long(output_fn_t, char *, size_t_, const char *, void *, void *);
int output_s_l_long(void);
extern "C" void _invalid_parameter_noinfo(void);

int shape_x_deref_split(char *buffer, size_t_ sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        *_errno() = 22; _invalid_parameter_noinfo(); return -1;
    }
    result = vsnprintf_helper_long(output_s_l_long, buffer, sizeInBytes, format, locale, argptr);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *_errno() = 34; _invalid_parameter_noinfo(); return -1;
}
