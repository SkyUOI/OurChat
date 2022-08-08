#include <base/server_def.h>
#include <cstdlib>
#include <cstdio>

namespace ourchat{
    const int port = 54088;

    void noreach_internal(size_t line, const char* funcname, const char* filename) {
        fprintf(stderr,
            "Fatal error in function \"%s\" file %s line "
            "%zu",
            funcname, filename, line);
        exit(EXIT_FAILURE);
    }
}
