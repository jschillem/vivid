file(GLOB_RECURSE ENGINE_SOURCES "*.c")

add_library(engine SHARED ${ENGINE_SOURCES})

target_include_directories(
    engine
    PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src
    PRIVATE ${Vulkan_INCLUDE_DIRS}
)

target_compile_definitions(engine PRIVATE VEXPORT)

target_link_libraries(engine PRIVATE Vulkan::Vulkan)

if(WIN32)
    target_link_libraries(engine PRIVATE user32)
elseif(LINUX)
    target_link_libraries(engine PRIVATE xcb X11 X11-xcb xkbcommon)
    target_compile_definitions(engine PRIVATE VK_USE_PLATFORM_XCB_KHR)
endif()

# compiler flags
if(MSVC)
    target_compile_options(engine PRIVATE /W4 /WX)
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options(engine PRIVATE /Od /Zi)
    else()
        target_compile_options(engine PRIVATE /O2)
    endif()
else()
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options(engine PRIVATE -g -O0)
    else()
        target_compile_options(engine PRIVATE -O2)
    endif()
endif()
