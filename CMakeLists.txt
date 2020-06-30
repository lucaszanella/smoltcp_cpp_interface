cmake_minimum_required (VERSION 3.10)
project(libsmoltcp_cpp LANGUAGES CXX)

include_directories(src/virtual_tun)
file(GLOB libsmoltcp_cpp_sources "src/virtual_tun/interface.cpp")

add_library(libsmoltcp_cpp ${libsmoltcp_cpp_sources})

add_custom_target(
    lib_smol_tcp_rust
    COMMAND cargo build
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_dependencies(libsmoltcp_cpp lib_smol_tcp_rust)
add_library(lsmoltcp_cpp_interface_rust SHARED IMPORTED)

set_target_properties(lsmoltcp_cpp_interface_rust PROPERTIES IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libsmoltcp_cpp_interface_rust.so)

add_executable(http_smol_tcp_example examples/http_smol_tcp_example)

target_link_libraries(http_smol_tcp_example libsmoltcp_cpp lsmoltcp_cpp_interface_rust)