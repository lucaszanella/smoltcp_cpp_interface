FROM ubuntu:bionic

WORKDIR /home/project

RUN export DEBIAN_FRONTEND=noninteractive && apt-get update\
    && apt-get install -y build-essential curl autoconf git m4 iproute2 clang wireshark wget

RUN curl --proto '=https' --tlsv1.2 -o rust.sh https://sh.rustup.rs\
    && /bin/bash rust.sh -y

RUN wget https://github.com/Kitware/CMake/releases/download/v3.18.0-rc2/cmake-3.18.0-rc2-Linux-x86_64.sh && ls

RUN mkdir -p /usr/local/cmake \
&& chmod +x cmake-3.18.0-rc2-Linux-x86_64.sh \
&& ./cmake-3.18.0-rc2-Linux-x86_64.sh --skip-license --prefix=/usr/local/cmake

ENV PATH="/root/.cargo/bin:/usr/local/cmake/bin:${PATH}"
