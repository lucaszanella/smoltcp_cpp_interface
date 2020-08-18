#ifndef SMOL_TCP_INTERFACE_H
#define SMOL_TCP_INTERFACE_H
#include <unordered_map>
#include <random>
#include <limits>
#include <queue>
#include <chrono>
#include <iostream>
#include <memory>
#include <optional>
#include <utility>
#include "utils.h"

typedef void *SmolStackPtr;
typedef size_t SocketHandle;

static const int SOCKET_TCP = 0;
static const int SOCKET_UDP = 1;

namespace smoltcp
{
    using namespace std::chrono;

    //It does NOT own the data
    struct CBuffer
    {
        uint8_t *data;
        size_t len;
    };

    struct NoDeleter
    {
        void operator()(uint8_t *b) { std::cout << "not going to delete Buffer" << std::endl; }
    };

    //BIG TODO: does this delete everything or just the pointed value?
    struct Buffer
    {
    public:
        using Ptr = std::shared_ptr<Buffer>;
        std::unique_ptr<uint8_t[]> data;
        size_t len = 0;
        Buffer(CBuffer cBuffer)
        {
            data = std::unique_ptr<uint8_t[]>(cBuffer.data);
            len = cBuffer.len;
        }
        Buffer(bool empty)
        {
            data = std::unique_ptr<uint8_t[]>(nullptr);
            this->len = 0;
            this->empty = empty;
        }

        uint8_t *getData()
        {
            return data.get();
        }
        bool empty = false;
    };

    extern "C" uint8_t *cpp_allocate_buffer(size_t size);

    class Instant
    {
    public:
        static milliseconds now()
        {
            return duration_cast<milliseconds>(system_clock::now().time_since_epoch());
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

    struct CIpAddress
    {
        uint8_t isIpv4 = 0;
        CIpv4Address ipv4Address;
        CIpv6Address ipv6Address;
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

    enum CIpEndpointType
    {
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

    extern "C" void cppDeleteArray(uint8_t *data);
    extern "C" void cppDeletePointer(uint8_t *data);
    extern "C" uint8_t *cpp_allocate_buffer(size_t size);
    extern "C" uint8_t *cpp_allocate_buffer_zero_terminated(size_t size);

    extern "C" SmolStackPtr smol_stack_smol_stack_new_virtual_tun(const char *interfaceName);
    extern "C" SmolStackPtr smol_stack_smol_stack_new_tun(const char *interfaceName);
    extern "C" SmolStackPtr smol_stack_smol_stack_new_tap(const char *interfaceName);
    extern "C" uint8_t smol_stack_add_socket(SmolStackPtr, uint8_t socketType, SocketHandle socketHandle);
    extern "C" void smol_stack_poll(SmolStackPtr);
    extern "C" void smol_stack_phy_wait(SmolStackPtr, int64_t timestamp);
    extern "C" void smol_stack_spin(SmolStackPtr, SocketHandle socketHandle);
    extern "C" void smol_stack_spin_all(SmolStackPtr);
    extern "C" uint8_t smol_stack_tcp_connect(SmolStackPtr, SocketHandle socketHandle, CIpAddress, uint16_t src_port, uint16_t dst_port);
    extern "C" uint8_t smol_stack_tcp_connect_ipv4(SmolStackPtr, SocketHandle socketHandle, CIpv4Address, uint16_t src_port, uint16_t dst_port);
    extern "C" uint8_t smol_stack_tcp_connect_ipv6(SmolStackPtr, SocketHandle socketHandle, CIpv6Address, uint16_t src_port, uint16_t dst_port);
    extern "C" uint8_t smol_stack_smol_socket_send(SmolStackPtr, SocketHandle socketHandle, const uint8_t *data, size_t len, CIpEndpoint endpoint, void *, uint8_t (*)(void *));
    extern "C" uint8_t smol_stack_smol_socket_send_copy(SmolStackPtr, SocketHandle socketHandle, const uint8_t *data, size_t len, CIpEndpoint endpoint);
    extern "C" uint8_t smol_stack_smol_socket_receive(SmolStackPtr, SocketHandle socketHandle, CBuffer *cbuffer, uint8_t *(*)(size_t));
    extern "C" uint8_t smol_stack_smol_socket_receive_wait(SmolStackPtr, SocketHandle socketHandle, CBuffer *cbuffer, uint8_t *(*)(size_t), CIpAddress *address);
    extern "C" uint8_t smol_stack_smol_socket_may_send(SmolStackPtr, SocketHandle socketHandle);
    extern "C" void smol_stack_add_ipv4_address(SmolStackPtr, CIpv4Cidr);
    extern "C" void smol_stack_add_ipv6_address(SmolStackPtr, CIpv6Cidr);
    extern "C" void smol_stack_add_default_v4_gateway(SmolStackPtr, CIpv4Address);
    extern "C" void smol_stack_add_default_v6_gateway(SmolStackPtr, CIpv6Address);
    extern "C" uint8_t smol_stack_finalize(SmolStackPtr);
    extern "C" uint8_t smol_stack_virtual_tun_send(SmolStackPtr, const uint8_t *data, size_t len);
    extern "C" uint8_t smol_stack_virtual_tun_receive_wait(SmolStackPtr, CBuffer *cbuffer, uint8_t *(*)(size_t));
    extern "C" uint8_t smol_stack_virtual_tun_receive_instantly(SmolStackPtr, CBuffer *cbuffer, uint8_t *(*)(size_t));
    extern "C" void smol_stack_destroy(void *);

    class RustSlice
    {
    public:
        uint8_t *data;
        size_t len;

        RustSlice(uint8_t *data, size_t len) : data(data), len(len) {}

        ~RustSlice()
        {
            //destroy on Rust
        }
    };

    class SmolSocket
    {
    public:
        //Handle is just a number that represents this socket
        //on Rust side.
        SocketHandle handle;
        //TODO: put limit on packets size
        std::queue<RustSlice> packets;
    };

    template <typename T>
    class SmolOwner
    {
    private:
        /*
            SmolOwner owns the pointer to this type and
            it's responsible for deleting it when it's 
            destructed
        */
        T *t;
        SmolOwner(T *t)
        {
            this->t = t;
        }

    public:
        //Prevents SmolOwner to be created on stack
        static SmolOwner *allocate(T *t)
        {
            return new SmolOwner(t);
        }

        ~SmolOwner()
        {
            std::cout << "~SmolOwner called" << std::endl;
            delete t;
        }
    };

    class TunSmolStack
    {
        public:
        using Ptr = std::shared_ptr<TunSmolStack>;
    private:
        SmolStackPtr smolStackPtr;
        std::random_device rd;
        std::mt19937 mt{rd()};
        std::uniform_int_distribution<int> random{49152, 49152 + 16383};
        size_t currentHandle = 0;
        std::unordered_map<size_t, SmolSocket> smolSocketHandles;

    public:
        enum StackType
        {
            VirtualTun,
            Tun,
            Tap
        };

        TunSmolStack(std::string interfaceName, StackType stackType)
        {
            if (stackType == StackType::VirtualTun)
            {
                smolStackPtr = smol_stack_smol_stack_new_virtual_tun(interfaceName.c_str());
            }
            else if (stackType == StackType::Tun)
            {
                smolStackPtr = smol_stack_smol_stack_new_tun(interfaceName.c_str());
            }
            else if (stackType == StackType::Tap)
            {
                smolStackPtr = smol_stack_smol_stack_new_tap(interfaceName.c_str());
            }
        }

        size_t getNewHandle()
        {
            if (currentHandle < std::numeric_limits<size_t>::max())
            {
                currentHandle += 1;
                return currentHandle;
            }
            else
            {
                throw std::runtime_error("Reached handle too big, you're using too much sockets\n");
            }
        }

        SmolSocket addSocket(uint8_t socketType)
        {
            size_t handle = getNewHandle();
            uint8_t result = smol_stack_add_socket(smolStackPtr, socketType, handle);
            SmolSocket smolSocket;
            smolSocket.handle = handle;
            smolSocketHandles[handle] = smolSocket;
            return smolSocket;
        }

        void poll()
        {
            smol_stack_poll(smolStackPtr);
        }

        void spin(SmolSocket smolSocket)
        {
            smol_stack_spin(smolStackPtr, smolSocket.handle);
        }

        void spinAll()
        {
            smol_stack_spin_all(smolStackPtr);
        }

        /*
            On the act of send, we specify the handle for the socket, the pointer do the data,
            which is the most important type, and its lenght. For UDP and IGMP sockets we also
            have to pass an endpoint (TCP does not need since we call connect before sending).
            Then, we pass a pointer to `SmolOwner`, which is a class that owns the object that
            owns `uint8_t* data`. We also pass the destructor function, which is the function 
            that accepts the `SmolOwner` pointer and deletes it. This function is supposed to
            be called from Rust when it does not need the data `uint8_t* data` anymore.
        */
        template <typename T>
        void send(SmolSocket smolSocket, const uint8_t *data, size_t len, CIpEndpoint endpoint, SmolOwner<T> *pointerToSmolOwner, uint8_t (*smolOwnerDestructor)(void *))
        {
            smol_stack_smol_socket_send(smolStackPtr, smolSocket.handle, data, len, endpoint, static_cast<void *>(pointerToSmolOwner), smolOwnerDestructor);
        }

        bool send_copy(SmolSocket smolSocket, const uint8_t *data, size_t len, CIpEndpoint endpoint)
        {
            uint8_t r = smol_stack_smol_socket_send_copy(smolStackPtr, smolSocket.handle, data, len, endpoint);
            if (r == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        //TCP only, no endpoint
        bool send_copy(SmolSocket smolSocket, const uint8_t *data, size_t len)
        {
            CIpEndpoint endpointNone{
                CIpEndpointType::None,
                CIpv4Address{},
                CIpv6Address{},
                0};

            uint8_t r = smol_stack_smol_socket_send_copy(smolStackPtr, smolSocket.handle, data, len, endpointNone);
            if (r == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        std::optional<std::pair<std::shared_ptr<Buffer>, CIpAddress>> receive(SmolSocket smolSocket)
        {
            CBuffer cbuffer;
            CIpAddress address;

            uint8_t r = smol_stack_smol_socket_receive(smolStackPtr, smolSocket.handle, &cbuffer, &cpp_allocate_buffer);
            if (r == 0)
            {
                auto buffer = std::make_shared<Buffer>(cbuffer);
                auto pair = std::make_pair(buffer, address);
                return std::optional<decltype(pair)>(pair);
            }
            else
            {
                return std::nullopt;
            }
        }

        std::optional<std::pair<std::shared_ptr<Buffer>, CIpAddress>> receiveWait(SmolSocket smolSocket)
        {
            //std::cout << "receiveWait" << std::endl;    
            CBuffer cbuffer;
            CIpAddress address;

            uint8_t r = smol_stack_smol_socket_receive_wait(smolStackPtr, smolSocket.handle, &cbuffer, &cpp_allocate_buffer, &address);
            if (r == 0)
            {
                //printBufferBeggining(cbuffer.data, cbuffer.len, 5);
                //std::cout << "...";
                //printBufferEnd(cbuffer.data, cbuffer.len, 5);

                auto buffer = std::make_shared<Buffer>(cbuffer);
                auto pair = std::make_pair(buffer, address);
                return std::optional<decltype(pair)>(pair);
            }
            else
            {
                return std::nullopt;
            }
        }

         /*
            Use your own custom allocator. Might be useful specially for ZLMediaKit which requires a buffer terminated with a \0
        */
        std::optional<std::pair<std::shared_ptr<Buffer>, CIpAddress>> receiveWait(SmolSocket smolSocket, uint8_t *(*custom_allocator)(size_t))
        {
            CBuffer cbuffer;
            CIpAddress address;

            uint8_t r = smol_stack_smol_socket_receive_wait(smolStackPtr, smolSocket.handle, &cbuffer, custom_allocator, &address);
            if (r == 0)
            {
                //std::cout << "#(" << cbuffer.len << ") - ";
                //Utils::compactBufferPrint(cbuffer.data, cbuffer.len, 5);
                auto buffer = std::make_shared<Buffer>(cbuffer);
                auto pair = std::make_pair(buffer, address);
                return std::optional<decltype(pair)>(pair);
            }
            else
            {
                return std::nullopt;
            }
        }

        bool maySend(SmolSocket smolSocket)
        {
            uint8_t r = smol_stack_smol_socket_may_send(smolStackPtr, smolSocket.handle);
            if (r == 0)
                return true;
            else
                return false;
        }

        bool connect(SmolSocket smolSocket, CIpAddress address, uint16_t src_port, uint16_t dst_port)
        {
            uint8_t r = smol_stack_tcp_connect(smolStackPtr, smolSocket.handle, address, src_port, dst_port);
            if (r == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        bool connectIpv4(SmolSocket smolSocket, CIpv4Address address, uint16_t src_port, uint16_t dst_port)
        {
            uint8_t r = smol_stack_tcp_connect_ipv4(smolStackPtr, smolSocket.handle, address, src_port, dst_port);
            if (r == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        uint16_t randomOutputPort()
        {
            return random(mt);
        }

        bool connectIpv6(SmolSocket smolSocket, CIpv6Address address, uint16_t src_port, uint16_t dst_port)
        {
            uint8_t r = smol_stack_tcp_connect_ipv6(smolStackPtr, smolSocket.handle, address, src_port, dst_port);
            if (r == 0)
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        void addIpv4Address(CIpv4Cidr cidr)
        {
            smol_stack_add_ipv4_address(smolStackPtr, cidr);
        }

        void addIpv6Address(CIpv6Cidr cidr)
        {
            smol_stack_add_ipv6_address(smolStackPtr, cidr);
        }

        void addDefaultV4Gateway(CIpv4Address address)
        {
            smol_stack_add_default_v4_gateway(smolStackPtr, address);
        }

        void addDefaultV6Gateway(CIpv6Address address)
        {
            smol_stack_add_default_v6_gateway(smolStackPtr, address);
        }

        void phy_wait(int64_t timestamp)
        {
            smol_stack_phy_wait(smolStackPtr, timestamp);
        }

        int64_t currentTimeMillis()
        {
            return Instant::now().count();
        }

        uint8_t finalize()
        {
            return smol_stack_finalize(smolStackPtr);
        }

        void virtualTunSend(const uint8_t *data, size_t len)
        {
            smol_stack_virtual_tun_send(smolStackPtr, data, len);
        }

        std::optional<std::shared_ptr<Buffer>> virtualTunReceiveWait()
        {
            CBuffer cbuffer;

            uint8_t r = smol_stack_virtual_tun_receive_wait(smolStackPtr, &cbuffer, &cpp_allocate_buffer);
            if (r == 0)
            {
                auto buffer = std::make_shared<Buffer>(cbuffer);
                return buffer;
            }
            else
            {
                return std::nullopt;
            }
        }

        std::optional<std::shared_ptr<Buffer>> virtualTunReceiveInstantly()
        {
            CBuffer cbuffer;

            uint8_t r = smol_stack_virtual_tun_receive_instantly(smolStackPtr, &cbuffer, &cpp_allocate_buffer);
            if (r == 0)
            {
                auto buffer = std::make_shared<Buffer>(cbuffer);
                return buffer;
            }
            else
            {
                return std::nullopt;
            }
        }

        /*
            Smoltcp's thread is responsible for calling the callback
            back with the data once it's ready
        */
        /*
        template<class ReadHandler>
        Buffer virtualTunReceiveCallback(SmolSocket smolSocket, ReadHandler &&handler)
        {
            CBuffer cbuffer;

            uint8_t r = smol_stack_virtual_tun_receive_callback(smolStackPtr, &cbuffer, &cpp_allocate_buffer);
            if (r == 0)
            {
                auto buffer = Buffer(cbuffer);
                return buffer;
            }
            else
            {
                auto buffer = Buffer(true);
                return buffer;
            }
        }
        */

        ~TunSmolStack()
        {
            std::cout << "TunSmolStack destruction" << std::endl;
            smol_stack_destroy(smolStackPtr);
        }
    };
} //namespace smoltcp
#endif //SMOL_TCP_INTERFACE_H