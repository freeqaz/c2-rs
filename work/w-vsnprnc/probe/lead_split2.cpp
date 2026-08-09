// lead_split2 — TWO functions of one spelling, so the label STRIDE is readable
// IN-TU and SEED-FREE: the difference between the two $M/$T triples.
typedef unsigned int size_t_;
typedef int (*output_fn_t)(void);
extern "C" int *_errno(void);
int helper_long_aa(output_fn_t, char *, size_t_, const char *, void *, void *);
int outfn_long_aa(void);
extern "C" void _invalid_parameter_noinfo(void);
typedef unsigned int size_t_;
typedef int (*output_fn_t)(void);
extern "C" int *_errno(void);
int helper_long_bb(output_fn_t, char *, size_t_, const char *, void *, void *);
int outfn_long_bb(void);
extern "C" void _invalid_parameter_noinfo(void);
int shape_function_aa(char *buffer, size_t_ sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) { *_errno() = 22; _invalid_parameter_noinfo(); return -1; }
    result = helper_long_aa(outfn_long_aa, buffer, sizeInBytes, format, locale, argptr);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *_errno() = 34; _invalid_parameter_noinfo(); return -1;
}
int shape_function_bb(char *buffer, size_t_ sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) { *_errno() = 22; _invalid_parameter_noinfo(); return -1; }
    result = helper_long_bb(outfn_long_bb, buffer, sizeInBytes, format, locale, argptr);
    if (result < 0) { *buffer = 0; }
    if (result != -2) { return result; }
    *_errno() = 34; _invalid_parameter_noinfo(); return -1;
}
