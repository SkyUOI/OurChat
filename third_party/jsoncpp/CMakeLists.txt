file(GLOB_RECURSE SRCS "*.cpp")
include_directories(.)
add_library(jsoncpp SHARED
        ${SRCS}
        )