#! /bin/bash

set -euxo pipefail

apt-get update

apt-get install -y cmake
apt-get install -y clang-7
apt-get install -y libclang-7-dev

ln -s /usr/bin/clang-10 /usr/bin/clang
ln -s /usr/bin/clang++-10 /usr/bin/clang++

apt-get install -y libudev-dev
