#include <WinSock2.h>
#include <base/library.h>

// todo:支持linux
namespace socket_lib {
    base_c_api void init();

    base_c_api SOCKET socket_();

    /**
     * @brief 返回以太网适配器ip
     * @return char* 
     */
    base_c_api char* get_self_ip();

    base_c_api void bind_(SOCKET s, char*ip);

    base_c_api void listen_(SOCKET s, int backlog);

    base_c_api SOCKET accept_(SOCKET s, sockaddr&addr, int&len);

    base_c_api void send_(SOCKET s, char* buf, int len, int flag);

    base_c_api void recv_(SOCKET s, char* buf, int len, int flag);

    base_c_api void close(SOCKET s);

    base_c_api void quit();

    base_c_api void connect_(SOCKET s, char*ip);
}
