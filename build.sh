cargo build
clang++ -shared -fPIC -o libsmoltcp_cpp_interface_cpp.so -L target/debug/ src/virtual_tun/interface.cpp -lstdc++ -lsmoltcp_cpp_interface_rust -pthread -ldl
clang++ -o smoltcp_httpclient_tun_example -L . lib_smol_tcp/*.cpp -L target/debug/ -I src/virtual_tun -lsmoltcp_cpp_interface_cpp -lsmoltcp_cpp_interface_rust
clang++ -o smoltcp_httpclient_virtual_tun_example -L . lib_smol_tcp/*.cpp -L target/debug/ -I src/virtual_tun -lsmoltcp_cpp_interface_cpp -lsmoltcp_cpp_interface_rust
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$PWD:$PWD/target/debug