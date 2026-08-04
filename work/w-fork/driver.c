/* lane w-fork — A/B benchmark driver.
 *
 * One host-native client so both arms pay the same driver overhead.
 *
 *   driver floor  <n> <wibo>                       -- `wibo --version` n times
 *   driver spawn  <corpus> <wibo> <c2host> <c2dll> -- today's path: one
 *                                                     wibo process per obj
 *   driver fork   <corpus> <socket> <c2dll>                -- fork-server client
 *
 * Every mode prints a POSITIVE count of objs produced; producing nothing is
 * reported as a failure, never as speed.
 */
#define _GNU_SOURCE
#include <dirent.h>
#include <fcntl.h>
#include <sys/resource.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define MAXARGS 256
#define MAXCASES 20000

static char *cases[MAXCASES];
static int ncases;

static int cmpstr(const void *a, const void *b) { return strcmp(*(char **)a, *(char **)b); }

static void load_cases(const char *corpus) {
    DIR *d = opendir(corpus);
    if (!d) { perror(corpus); exit(1); }
    struct dirent *e;
    while ((e = readdir(d))) {
        if (e->d_name[0] == '.') continue;
        char p[4096];
        snprintf(p, sizeof p, "%s/%s/argv.txt", corpus, e->d_name);
        struct stat st;
        if (stat(p, &st) != 0) continue;
        char dir[4096];
        snprintf(dir, sizeof dir, "%s/%s", corpus, e->d_name);
        if (ncases >= MAXCASES) break;
        cases[ncases++] = strdup(dir);
    }
    closedir(d);
    qsort(cases, ncases, sizeof(char *), cmpstr);
}

/* Read <case>/argv.txt into argv[] (one token per line). Returns count. */
static int read_argv(const char *dir, char **out, int max, char *buf, size_t buflen) {
    char p[4096];
    snprintf(p, sizeof p, "%s/argv.txt", dir);
    FILE *f = fopen(p, "rb");
    if (!f) return -1;
    size_t n = fread(buf, 1, buflen - 1, f);
    fclose(f);
    buf[n] = 0;
    int c = 0;
    char *s = buf;
    while (*s && c < max) {
        char *nl = strchr(s, '\n');
        if (nl) *nl = 0;
        if (*s) out[c++] = s;
        if (!nl) break;
        s = nl + 1;
    }
    return c;
}

static double now(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec * 1e-9;
}

static int produced(const char *dir, const char *name) {
    char p[4096];
    struct stat st;
    snprintf(p, sizeof p, "%s/%s", dir, name);
    return stat(p, &st) == 0 && st.st_size > 0;
}

static void rename_out(const char *dir, const char *suffix) {
    char a[4096], b[4096];
    snprintf(a, sizeof a, "%s/out.obj", dir);
    snprintf(b, sizeof b, "%s/%s.obj", dir, suffix);
    rename(a, b);
}

static void report(const char *tag, int n, int total, double wall) {
    struct rusage ru;
    getrusage(RUSAGE_CHILDREN, &ru);
    double cpu = ru.ru_utime.tv_sec + ru.ru_utime.tv_usec * 1e-6 + ru.ru_stime.tv_sec +
                 ru.ru_stime.tv_usec * 1e-6;
    printf("%s: produced %d of %d   wall %.3f s   %.3f ms/obj   child-cpu %.3f s   %.3f ms-cpu/obj\n",
           tag, n, total, wall, wall * 1000.0 / (total ? total : 1), cpu,
           cpu * 1000.0 / (total ? total : 1));
    if (n == 0) { fprintf(stderr, "%s: PRODUCED NOTHING — failure, not speed\n", tag); exit(1); }
    if (n != total) fprintf(stderr, "%s: WARNING %d of %d cases produced no obj\n", tag, total - n, total);
}

static int mode_floor(int argc, char **argv) {
    if (argc < 4) return 2;
    int n = atoi(argv[2]);
    const char *wibo = argv[3];
    /* extra args after the wibo path replace the default `--version` */
    char *full[MAXARGS];
    int k = 0;
    full[k++] = (char *)wibo;
    if (argc > 4) { for (int i = 4; i < argc && k < MAXARGS - 1; i++) full[k++] = argv[i]; }
    else full[k++] = (char *)"--version";
    full[k] = NULL;
    double t0 = now();
    for (int i = 0; i < n; i++) {
        pid_t pid = fork();
        if (pid == 0) {
            int devnull = open("/dev/null", O_WRONLY);
            dup2(devnull, 1); dup2(devnull, 2);
            execv(wibo, full);
            _exit(127);
        }
        int st; waitpid(pid, &st, 0);
    }
    double wall = now() - t0;
    struct rusage ru; getrusage(RUSAGE_CHILDREN, &ru);
    double cpu = ru.ru_utime.tv_sec + ru.ru_utime.tv_usec * 1e-6 + ru.ru_stime.tv_sec +
                 ru.ru_stime.tv_usec * 1e-6;
    printf("floor: %d spawns of `wibo --version`   wall %.3f s   %.3f ms each   child-cpu %.3f s   %.3f ms-cpu each\n",
           n, wall, wall * 1000.0 / n, cpu, cpu * 1000.0 / n);
    return 0;
}

static int mode_spawn(int argc, char **argv) {
    if (argc < 6) return 2;
    load_cases(argv[2]);
    const char *wibo = argv[3], *host = argv[4], *dll = argv[5];
    const char *suffix = argc > 6 ? argv[6] : "spawn";
    int n = 0;
    double t0 = now();
    for (int i = 0; i < ncases; i++) {
        char buf[65536];
        char *a[MAXARGS];
        int c = read_argv(cases[i], a, MAXARGS - 8, buf, sizeof buf);
        if (c < 0) continue;
        char *full[MAXARGS];
        int k = 0;
        full[k++] = (char *)wibo;
        full[k++] = (char *)host;
        full[k++] = (char *)dll;
        full[k++] = (char *)dll;
        for (int j = 0; j < c; j++) full[k++] = a[j];
        full[k] = NULL;
        char outp[4096];
        snprintf(outp, sizeof outp, "%s/out.obj", cases[i]);
        unlink(outp);
        pid_t pid = fork();
        if (pid == 0) {
            if (chdir(cases[i]) != 0) _exit(126);
            int devnull = open("/dev/null", O_WRONLY);
            dup2(devnull, 1); dup2(devnull, 2);
            execv(wibo, full);
            _exit(127);
        }
        int st; waitpid(pid, &st, 0);
        if (produced(cases[i], "out.obj")) { rename_out(cases[i], suffix); n++; }
    }
    double wall = now() - t0;
    report("spawn", n, ncases, wall);
    return 0;
}

/* Request wire format, all little-endian:
 *   u32 total_len
 *   u32 argc
 *   cwd NUL
 *   argv[0] NUL ... argv[argc-1] NUL
 * Reply: i32 child exit status (as returned by waitpid, already WEXITSTATUS'd
 *        or 128+sig).                                                        */
static int mode_fork(int argc, char **argv) {
    if (argc < 5) return 2;
    load_cases(argv[2]);
    const char *sockpath = argv[3];
    const char *dll = argv[4];
    const char *suffix = argc > 5 ? argv[5] : "fork";
    int n = 0;
    double t0 = now();
    for (int i = 0; i < ncases; i++) {
        char buf[65536];
        char *a[MAXARGS];
        int c = read_argv(cases[i], a, MAXARGS, buf, sizeof buf);
        if (c < 0) continue;
        char outp[4096];
        snprintf(outp, sizeof outp, "%s/out.obj", cases[i]);
        unlink(outp);

        char pay[131072];
        size_t off = 8;
        size_t l = strlen(cases[i]) + 1;
        memcpy(pay + off, cases[i], l); off += l;
        /* argv[0] is the backend's own module path, exactly as c2host passes it */
        l = strlen(dll) + 1; memcpy(pay + off, dll, l); off += l;
        for (int j = 0; j < c; j++) { l = strlen(a[j]) + 1; memcpy(pay + off, a[j], l); off += l; }
        unsigned int tl = (unsigned int)off, ac = (unsigned int)(c + 1);
        memcpy(pay, &tl, 4); memcpy(pay + 4, &ac, 4);

        int fd = socket(AF_UNIX, SOCK_STREAM, 0);
        struct sockaddr_un sa; memset(&sa, 0, sizeof sa);
        sa.sun_family = AF_UNIX;
        snprintf(sa.sun_path, sizeof sa.sun_path, "%s", sockpath);
        if (connect(fd, (struct sockaddr *)&sa, sizeof sa) != 0) {
            perror("connect"); exit(1);
        }
        size_t sent = 0;
        while (sent < off) {
            ssize_t w = write(fd, pay + sent, off - sent);
            if (w <= 0) { perror("write"); exit(1); }
            sent += (size_t)w;
        }
        int status = -1;
        size_t got = 0;
        while (got < 4) {
            ssize_t r = read(fd, (char *)&status + got, 4 - got);
            if (r <= 0) break;
            got += (size_t)r;
        }
        close(fd);
        if (got != 4) { fprintf(stderr, "fork: short reply on case %s\n", cases[i]); }
        if (produced(cases[i], "out.obj")) { rename_out(cases[i], suffix); n++; }
    }
    double wall = now() - t0;
    /* Explicit shutdown so the server prints its own rusage accounting. */
    {
        int fd = socket(AF_UNIX, SOCK_STREAM, 0);
        struct sockaddr_un sa; memset(&sa, 0, sizeof sa);
        sa.sun_family = AF_UNIX;
        snprintf(sa.sun_path, sizeof sa.sun_path, "%s", sockpath);
        if (connect(fd, (struct sockaddr *)&sa, sizeof sa) == 0) {
            unsigned int hdr[2] = {8u, 0u};
            (void)!write(fd, hdr, sizeof hdr);
            int st; (void)!read(fd, &st, 4);
        }
        close(fd);
    }
    printf("fork: produced %d of %d   wall %.3f s   %.3f ms/obj\n", n, ncases, wall,
           wall * 1000.0 / (ncases ? ncases : 1));
    if (n == 0) { fprintf(stderr, "fork: PRODUCED NOTHING — failure, not speed\n"); exit(1); }
    if (n != ncases) fprintf(stderr, "fork: WARNING %d of %d cases produced no obj\n", ncases - n, ncases);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: driver floor|spawn|fork ...\n");
        return 2;
    }
    if (!strcmp(argv[1], "floor")) return mode_floor(argc, argv);
    if (!strcmp(argv[1], "spawn")) return mode_spawn(argc, argv);
    if (!strcmp(argv[1], "fork")) return mode_fork(argc, argv);
    fprintf(stderr, "unknown mode %s\n", argv[1]);
    return 2;
}
