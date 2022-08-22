#include <server/server.h>

using boost::asio::ip::tcp;

int main() {
    printf("Ourchat server starting...");
    ourchat::server server;
    return 0;
}
