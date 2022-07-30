cmake_minimum_required(VERSION 3.23)

# ccache的使用
find_program(CCACHE_PROGRAM ccache)
if(CCACHE_PROGRAM)
    message("using ccache")
    set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE ccache)
    set_property(GLOBAL PROPERTY RULE_LAUNCH_LINK ccache)
endif()

project(OurChat CXX)

set(EXECUTABLE_OUTPUT_PATH ${PROJECT_SOURCE_DIR}/bin)
set(LIBRARY_OUTPUT_PATH ${PROJECT_SOURCE_DIR}/bin)
link_directories(${PROJECT_SOURCE_DIR}/bin)

include_directories(src/server/include)
include_directories(third_party)
include_directories(third_party/jsoncpp)

add_subdirectory(third_party)
add_subdirectory(src/server)

