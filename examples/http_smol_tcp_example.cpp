#include <iostream>
#include "interface.h"
#include <map>
#include <thread>
#include <chrono>
/*
    Destructor for us to pass to smol socket send function
    Is destructs SmolOwner functions
*/
using namespace smoltcp;

extern "C" uint8_t destruct(void *smolOwner_)
{
    //std::cout << "smol owner destruct called" << std::endl;
    SmolOwner<std::string> *smolOwner = static_cast<SmolOwner<std::string> *>(smolOwner_);
    delete smolOwner;
    return 0;
}

int main()
{
    TunSmolStack tunSmolStack("tun1", StackType::Tun);

    tunSmolStack.addIpv4Address(CIpv4Cidr{
        CIpv4Address{
            {192, 168, 69, 1}},
        24});

    tunSmolStack.addIpv6Address(CIpv6Cidr{
        CIpv6Address{
            {0xfdaa, 0, 0, 0, 0, 0, 0, 1}},
        64});
    tunSmolStack.addIpv6Address(CIpv6Cidr{
        CIpv6Address{
            {0xfe80, 0, 0, 0, 0, 0, 0, 1}},
        64});

    tunSmolStack.addDefaultV4Gateway(CIpv4Address{
        {192, 168, 69, 100}});

    tunSmolStack.addDefaultV6Gateway(CIpv6Address{
        {0xfe80, 0, 0, 0, 0, 0, 0, 0x100}});

    SmolSocket smolSocket = tunSmolStack.addSocket(SOCKET_TCP);
    uint8_t result = tunSmolStack.finalize();
    CIpEndpoint endpointNone{
        CIpEndpointType::None,
        CIpv4Address{},
        CIpv6Address{},
        0};
    enum State
    {
        Connect,
        Request,
        Response
    };
    State state = State::Connect;

    if (result == 0)
    {
        //socketLoop(tunSmolStack, handle);
        while (true)
        {
            tunSmolStack.poll();
            if (state == State::Connect)
            {
                std::cout << "connecting..." << std::endl;
                uint16_t randomOutputPort = tunSmolStack.randomOutputPort();
                tunSmolStack.connectIpv4(smolSocket, CIpv4Address{{172, 217, 28, 238}},
                                         randomOutputPort, 80);
                state = State::Request;
            }
            if (state == State::Request)
            {
                std::string httpRequestData("GET /index.html HTTP/1.1\r\nHost: www.google.com\r\nConnection: Keep-Alive\r\n\r\n");
                std::string *s = new std::string(httpRequestData);
                auto smolOwner = SmolOwner<std::string>::allocate(s);
                const uint8_t *httpRequestDataBuffer = reinterpret_cast<const uint8_t *>(s->c_str());
                tunSmolStack.send(smolSocket, httpRequestDataBuffer, httpRequestData.size(), endpointNone, smolOwner, destruct);
                state = State::Response;
            }
            if (state == State::Response)
            {
                Buffer buffer = tunSmolStack.receive(smolSocket);
                if (!buffer.empty)
                {
                    //std::cout << std::string(buffer.cBuffer.data, buffer.cBuffer.len) << std::endl;
                    fwrite(buffer.cBuffer.data, 1, buffer.cBuffer.len, stdout);
                    //std::cout << "received " << buffer.cBuffer.len << " bytes" << std::endl; 
                }
                else
                {
                }
            }
            tunSmolStack.spin(smolSocket);
            tunSmolStack.phy_wait(tunSmolStack.currentTimeMillis());
        }
    }
    else
    {
        //throw
        std::cout << "error on finalize" << std::endl;
    }

    getchar();
    return 0;
}
