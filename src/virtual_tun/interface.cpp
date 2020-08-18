#include <iostream>
#include "interface.h"

extern "C" void cppDeleteArray(uint8_t *data)
{
    delete[] data;
}

extern "C" void cppDeletePointer(uint8_t *data)
{
    delete data;
}

extern "C" uint8_t *cpp_allocate_buffer(size_t size)
{
    uint8_t *buffer = new uint8_t[size];
    return buffer;
}

//Useful in ZLMediaKit which requires buffer terminated in \0
extern "C" uint8_t *cpp_allocate_buffer_zero_terminated(size_t size)
{
    //std::cout << "allocating buffer with size " << size +1 << std::endl;
    uint8_t *buffer = new uint8_t[size+1];
    buffer[size] = '\0';
    return buffer;
}