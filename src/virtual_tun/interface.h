#ifndef SMOL_TCP_INTERFACE_H
#define SMOL_TCP_INTERFACE_H
#include <unordered_map>
#include <random>
#include <limits>

typedef void *TunSmolStackPtr;
typedef size_t SocketHandleKey;

static const int SOCKET_TCP = 0;
static const int SOCKET_UDP = 1;

template <typename T>
class HandleMap
{
public:
    std::unordered_map<size_t, T> map;

    T &get(size_t key)
    {
        auto it = map.find(key);
        if (it == map.end())
            throw std::invalid_argument("invalid key");
        return it->second;
    }

    size_t emplace(T t)
    {
        size_t handle = getNewHandle();
        map[handle] = t;
        return handle;
    }

private:
    size_t currentIndex = 0;
    size_t getNewHandle()
    {
        if (currentIndex < std::numeric_limits<size_t>::max())
        {
            currentIndex += 1;
            return currentIndex;
        }
        else
        {
            throw std::runtime_error("Reached handle too big, you're using too much sockets\n");
        }
    }
};

struct CIpv4Address
{
    uint8_t address[4];
};

struct CIpv6Address
{
    uint16_t address[8];
};

struct CIpv4Cidr
{
    CIpv4Address address;
    uint32_t prefix;
};

struct CIpv6Cidr
{
    CIpv6Address address;
    uint64_t prefix;
};

enum CIpEndpointType {
    None = 0,
    Ipv4 = 1, 
    Ipv6 = 2
};

struct CIpEndpoint
{
    CIpEndpointType type;
    CIpv4Address ipv4;
    CIpv6Address ipv6;
    uint16_t port;
};

extern "C" void cppDeleteArray(uint8_t *data)
{
    delete[] data;
}

extern "C" void cppDeletePointer(uint8_t *data)
{
    delete data;
}

extern "C" TunSmolStackPtr smol_stack_tun_smol_stack_new(const char *interfaceName);
extern "C" size_t smol_stack_add_socket(TunSmolStackPtr, uint8_t);
extern "C" void smol_stack_poll(TunSmolStackPtr);
extern "C" void smol_stack_spin(TunSmolStackPtr, size_t handle);
extern "C" void smol_stack_tcp_connect_ipv4(TunSmolStackPtr, CIpv4Address, uint8_t, uint8_t);
extern "C" void smol_stack_tcp_connect_ipv6(TunSmolStackPtr, CIpv6Address, uint8_t, uint8_t);
extern "C" uint8_t smol_stack_tun_smol_socket_send(TunSmolStackPtr, size_t handle, const uint8_t* data, size_t len, CIpEndpoint endpoint);
extern "C" void smol_stack_add_ipv4_address(TunSmolStackPtr, CIpv4Cidr);
extern "C" void smol_stack_add_ipv6_address(TunSmolStackPtr, CIpv6Cidr);
extern "C" void smol_stack_add_default_v4_gateway(TunSmolStackPtr, CIpv4Address);
extern "C" void smol_stack_add_default_v6_gateway(TunSmolStackPtr, CIpv6Address);
extern "C" uint8_t smol_stack_finalize(TunSmolStackPtr);

class SmolSocket
{
public:
    unsigned int id = 0;
    SocketHandleKey SocketHandleKey;
};

class TunSmolStack
{
private:
    TunSmolStackPtr tunSmolStackPtr;
    std::random_device rd;
    std::mt19937 mt{rd()};
    std::uniform_int_distribution<int> random{49152, 49152+16383};
    
public:
    
    TunSmolStack(std::string interfaceName)
    {
        tunSmolStackPtr = smol_stack_tun_smol_stack_new(interfaceName.c_str());
    }

    size_t addSocket(uint8_t socketType)
    {
        return smol_stack_add_socket(tunSmolStackPtr, socketType);
    }

    void poll()
    {
        smol_stack_poll(tunSmolStackPtr);
    }

    void spin(size_t handle)
    {
        smol_stack_spin(tunSmolStackPtr, handle);
    }

    void send(size_t handle, const uint8_t* data, size_t len, CIpEndpoint endpoint)
    {
        smol_stack_tun_smol_socket_send(tunSmolStackPtr, handle, data, len, endpoint);
    }

    void connectIpv4(CIpv4Address address, uint8_t src_port, uint8_t dst_port)
    {
        smol_stack_tcp_connect_ipv4(tunSmolStackPtr, address, src_port, dst_port);
    }

    uint16_t randomOutputPort() {
        return random(mt);
    }

    void connectIpv6(CIpv6Address address, uint8_t src_port, uint8_t dst_port)
    {
        smol_stack_tcp_connect_ipv6(tunSmolStackPtr, address, src_port, dst_port);
    }

    void addIpv4Address(CIpv4Cidr cidr)
    {
        smol_stack_add_ipv4_address(tunSmolStackPtr, cidr);
    }

    void addIpv6Address(CIpv6Cidr cidr)
    {
        smol_stack_add_ipv6_address(tunSmolStackPtr, cidr);
    }

    void addDefaultV4Gateway(CIpv4Address address)
    {
        smol_stack_add_default_v4_gateway(tunSmolStackPtr, address);
    }

    void addDefaultV6Gateway(CIpv6Address address)
    {
        smol_stack_add_default_v6_gateway(tunSmolStackPtr, address);
    }

    uint8_t finalize()
    {
        return smol_stack_finalize(tunSmolStackPtr);
    }
};

#endif //SMOL_TCP_INTERFACE_H