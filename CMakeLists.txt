cmake_minimum_required(VERSION 3.25)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

message(STATUS "Build type: ${CMAKE_BUILD_TYPE}")

project(VividGameEngine C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# set C standards
set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)

# vulkan package
find_package(Vulkan REQUIRED)

if(
    (CMAKE_BUILD_TYPE STREQUAL "Debug")
    OR (CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
)
    add_compile_definitions(_DEBUG)
    if(CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
        add_compile_definitions(VRELWITHDEBINFO VRELEASE)
    endif()
elseif(CMAKE_BUILD_TYPE STREQUAL "Release")
    add_compile_definitions(VRELEASE)
endif()

if(MSVC)
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS)
endif()

# engine library
add_subdirectory(engine)

# testbed executable
add_subdirectory(testbed)

if(CMAKE_EXPORT_COMPILE_COMMANDS)
    add_custom_target(
        copy_compile_commands
        ALL
        COMMAND
            ${CMAKE_COMMAND} -E copy_if_different
            ${CMAKE_BINARY_DIR}/compile_commands.json
            ${CMAKE_SOURCE_DIR}/compile_commands.json
        DEPENDS ${CMAKE_BINARY_DIR}/compile_commands.json
    )
endif()
