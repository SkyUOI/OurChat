/**
 * @file socket.cpp
 * @author your name (you@domain.com)
 * @brief socket的跨平台封装
 * @version 0.1
 * @date 2022-07-20
 * 
 * @copyright Copyright (c) 2022
 * 
 */

#include <WinSock2.h>
#include<ws2tcpip.h>
#include <cstdio>
#include <base/config.h>
#include <base/socket.h>

// todo:支持linux
namespace socket_lib {
    void init() {
        WSADATA data;
        int ret = WSAStartup(MAKEWORD(2,2), &data);
        if(ret) {
            fprintf(stderr, "Failed to init the network");
        }
    }

    SOCKET socket_() {
        SOCKET sock = socket(AF_INET, SOCK_STREAM, 0);
        if(sock == -1) {
            fprintf(stderr, "Failed to create the socket");
        }
        return sock;
    }

    char* get_self_ip() {
        PHOSTENT hostinfo;
        char hostname[255] = {0}; //主机名   
        gethostname(hostname, sizeof(hostname));
        if(((hostinfo = gethostbyname(hostname))) == NULL) //获得本地ipv4地址
        {
            errno = GetLastError();
            fprintf(stderr,"gethostbyname Error:%d\n", errno);
            exit(1);
        }
        return inet_ntoa(*(struct in_addr *) *hostinfo->h_addr_list);
    }

    void bind_(SOCKET s, char*ip) {
        sockaddr_in addr;
        addr.sin_family = AF_INET;
        addr.sin_port = htons(def::port);
        addr.sin_addr.S_un.S_addr = inet_addr(ip);
        int ret = bind(s, (sockaddr*)&addr, sizeof(addr));
        if(ret == -1) {
            fprintf(stderr, "failed to bind to the ip");
        }
    }

    void listen_(SOCKET s, int backlog) {
        int ret = listen(s, 5);
        if(ret == -1) {
            fprintf(stderr, "Failed to listen to the socket");
        }
    }

    SOCKET accept_(SOCKET s, sockaddr&addr, int&len) {
        SOCKET sockCli = accept(s, &addr, &len);
        if(sockCli == -1) {
            fprintf(stderr, "Failed to listen to the Client");
        }
        return sockCli;
    }

    void send_(SOCKET s, char* buf, int len, int flag) {
        int ret = send(s, buf, len, flag);
        if(ret == -1) {
            fprintf(stderr, "Failed to send the message");
        }
    }

    void recv_(SOCKET s, char* buf, int len, int flag) {
        int ret = recv(s, buf, len, flag);
        if(ret <= 0) {
            fprintf(stderr, "Failed to get the data from client");
        }
    }

    void close(SOCKET s) {
        closesocket(s);
    }

    void quit() {
        WSACleanup();
    }

    void connect_(SOCKET s, char*ip) {
        sockaddr_in addr;
        addr.sin_family = AF_INET;
        addr.sin_port = htons(def::port);
        addr.sin_addr.S_un.S_addr = inet_addr(ip);
        int ret = connect(s, (sockaddr*)&addr, sizeof(addr));
        if(ret == -1) {
            fprintf(stderr, "Failed to link server");
        }
    }
}
