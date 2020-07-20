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