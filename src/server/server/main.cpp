#include <cstdio>
#include <base/socket.h>

int main(int argc, char *argv[])
{
    printf("OurChat Server starting...\n");
    printf("Your IP address is %s\n", socket_lib::get_self_ip());
    for(;;) {
        
    }
}
