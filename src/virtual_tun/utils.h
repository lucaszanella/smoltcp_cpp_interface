#ifndef SMOLTCP_CPP_INTERFACE_UTILS_H
#define SMOLTCP_CPP_INTERFACE_UTILS_H
#include <iostream>
namespace smoltcp
{

    class Utils
    {
    public:
        static void printBuffer(const uint8_t *data, size_t quantity)
        {
            for (int i = 0; i < quantity; i++)
            {
                std::cout << std::hex << (int)(uint8_t)data[i] << " " << std::flush;
            }
        }

        static void printBufferBeggining(const uint8_t *data, size_t len, size_t quantity)
        {
            if (quantity == 0)
                quantity = len;
            if (quantity>len) {
                std::cout << "buffer too small" << std::endl;
                return;
            } 
            for (int i = 0; i < quantity; i++)
            {
                std::cout << std::hex << (int)(uint8_t)data[i] << " " << std::flush;
            }
        }

        static void printBufferEnd(const uint8_t *data, size_t len, size_t quantity)
        {
            if (quantity == 0)
                quantity = len;
            if (quantity>len) {
                std::cout << "buffer too small" << std::endl;
                return;
            }
            for (int i = quantity; i > 0; i--)
            {
                std::cout << std::hex << (int)(uint8_t)data[len - i] << " " << std::flush;
            }
        }

        static void compactBufferPrint(const uint8_t *data, size_t len, size_t quantity)
        {
            Utils::printBufferBeggining(data, len, quantity);
            std::cout << "...";
            Utils::printBufferEnd(data, len, quantity);
            std::cout << std::endl;
        }
    };
} // namespace smoltcp
#endif //SMOLTCP_CPP_INTERFACE_UTILS_H