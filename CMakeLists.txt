cmake_minimum_required (VERSION 3.10)
project(libsmoltcp_cpp LANGUAGES CXX)

include_directories(src/virtual_tun)
file(GLOB LIBSMOLTCP_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/src/virtual_tun/interface.cpp" 
"${CMAKE_CURRENT_SOURCE_DIR}/src/virtual_tun/utils.cpp")

add_library(lsmoltcp_cpp_interface_rust STATIC IMPORTED)
set_target_properties(lsmoltcp_cpp_interface_rust PROPERTIES IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libsmoltcp_cpp_interface_rust.a)

add_library(smoltcp_cpp ${LIBSMOLTCP_SOURCES})
add_library(smoltcp_cpp_static STATIC ${LIBSMOLTCP_SOURCES})

#TODO: enhance this to run on the right folder
add_custom_target(
    lib_smol_tcp_rust
    COMMAND cargo build
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

add_dependencies(smoltcp_cpp lib_smol_tcp_rust)
set_target_properties(smoltcp_cpp PROPERTIES
            CXX_STANDARD 17
            CXX_EXTENSIONS OFF)
target_link_libraries(smoltcp_cpp INTERFACE lsmoltcp_cpp_interface_rust -ldl)

add_dependencies(smoltcp_cpp_static lib_smol_tcp_rust)
set_target_properties(smoltcp_cpp_static PROPERTIES
            CXX_STANDARD 17
            CXX_EXTENSIONS OFF)
target_link_libraries(smoltcp_cpp_static INTERFACE lsmoltcp_cpp_interface_rust -ldl)


add_executable(http_smol_tcp_tun_example ${CMAKE_CURRENT_SOURCE_DIR}/examples/http_smol_tcp_tun_example)
add_dependencies(http_smol_tcp_tun_example smoltcp_cpp lsmoltcp_cpp_interface_rust)
set_target_properties(http_smol_tcp_tun_example PROPERTIES
            CXX_STANDARD 17
            CXX_EXTENSIONS OFF)
target_link_libraries(http_smol_tcp_tun_example smoltcp_cpp lsmoltcp_cpp_interface_rust dl pthread)


add_executable(http_smol_tcp_virtual_tun_example ${CMAKE_CURRENT_SOURCE_DIR}/examples/http_smol_tcp_virtual_tun_example)
add_dependencies(http_smol_tcp_virtual_tun_example smoltcp_cpp lsmoltcp_cpp_interface_rust)
set_target_properties(http_smol_tcp_virtual_tun_example PROPERTIES
            CXX_STANDARD 17
            CXX_EXTENSIONS OFF)
target_link_libraries(http_smol_tcp_virtual_tun_example smoltcp_cpp lsmoltcp_cpp_interface_rust dl pthread)