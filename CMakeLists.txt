cmake_minimum_required(VERSION 3.16)

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
link_directories(third_party/boost/lib)

include_directories(src/server/include)
include_directories(third_party/jsoncpp)
include_directories(third_party/boost)
include_directories(third_party/hash-library)
include_directories(third_party/easyloggingpp)

add_subdirectory(third_party)
add_subdirectory(src/server)
add_subdirectory(tests/server/testclient)

if("${OURCHAT_BUILD_TYPE}" STREQUAL "Test")
    add_subdirectory(tests/server/unittest)
    add_subdirectory(third_party/googletest)
    message("using gtest")
    target_link_libraries(unittest OurChat_server gtest)
    if(CMAKE_SYSTEM_NAME MATCHES "Linux")
        target_link_libraries(unittest pthread)
    endif()
endif()
