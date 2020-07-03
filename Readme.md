# SmolTCP C and C++ interface

This is a project aimed towards making the great Rust SmolTCP library acessible through C++ (and incidentally through C).

I created a simple C interface and I compile and link with SmolTCP's library. Support for TUN, TAP and VirtualTun are almost ready.

The idea of this project is to integrate with OpenVPN3's C++ library in order to use it as a true library, without relying on the system's TUN/TAP interface.

# Compilation 

## With docker:

```
git clone https://github.com/lucaszanella/smoltcp_cpp_interface
cd smoltcp_cpp_interface/docker
./build.sh
./run.sh
```

Note that if you simply clone this project and open it on VSCode, then if you 
have the `ms-vscode-remote.remote-containers` extension and docker w/out sudo, 
then you'll be prompted with a dialog to open this in a container. It'll then 
simply build the entire docker container and enter into it so you can develop
or build :)  

## Without docker:

Install all dependencies listed in the Dockerfile, then:

```
git clone https://github.com/lucaszanella/smoltcp_cpp_interface
cd smoltcp_cpp_interface
mkdir build
cd build 
cmake ../
```

