#include <iostream>
#include "interface.h"

int main() {
    
    TunSmolStack tunSmolStack("tun0");
    /*
    tunSmolStack.addIpv4Address(CIpv4Cidr {
        CIpv4Address {
            {192, 168, 69, 1}
        },
        24
    });
    tunSmolStack.addIpv6Address(CIpv6Cidr {
        CIpv6Address {
            {0xfdaa, 0, 0, 0, 0, 0, 0, 1}
        },
        64
    });
    tunSmolStack.addIpv6Address(CIpv6Cidr {
        CIpv6Address {
            {0xfe80, 0, 0, 0, 0, 0, 0, 1}
        },
        64
    });

    tunSmolStack.addDefaultV4Gateway(CIpv4Address {
            {192, 168, 69, 100}
    });

    tunSmolStack.addDefaultV6Gateway(CIpv6Address {
            {0xfe80, 0, 0, 0, 0, 0, 0, 0x100}
    });
    
    SocketHandlePtr socketHandle = tunSmolStack.addSocket(SOCKET_TCP);
    tunSmolStack.spin(socketHandle);
    */
    getchar();
    return 0;
}
