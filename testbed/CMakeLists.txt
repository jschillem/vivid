file(GLOB_RECURSE TESTBED_SOURCES "*.c")

add_executable(testbed ${TESTBED_SOURCES})

target_include_directories(
    testbed
    PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src $(CMAKE_SOURCE_DIR)/engine/src
)

target_link_libraries(testbed PRIVATE engine)

# compiler flags
if(MSVC)
    target_compile_options(testbed PRIVATE /W4)
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options(testbed PRIVATE /Od /Zi)
    else()
        target_compile_options(testbed PRIVATE /O2)
    endif()
else()
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options(testbed PRIVATE -g -O0)
    else()
        target_compile_options(testbed PRIVATE -O2)
    endif()
endif()
