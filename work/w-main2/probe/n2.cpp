class App { int _pad; public: App(int, char **); ~App(); void Run(); };
class Log { int _pad; public: Log(); ~Log(); };
int main(int argc, char **argv) { App app(argc, argv); Log log; app.Run(); }
