@echo off

REM set a variable BUILD_TYPE passed as an argument
set BUILD_TYPE=%1

if not exist "build" mkdir build
cd build
cmake .. --DCMAKE_BUILD_TYPE=%BUILD_TYPE%
cmake --build .

