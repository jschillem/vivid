#!/bin/bash

# determine based on argument what release type to run (default is Debug)
BUILD_TYPE=$1

mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=$BUILD_TYPE
cmake --build .

./testbed/testbed


