#include <iostream>

void printBuffer(uint8_t* data, size_t len) {
    for (size_t i = 0; i<len; i++) {
        std::cout << data[i];
    }
    std::cout << std::endl;
}