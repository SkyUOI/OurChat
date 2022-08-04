file(GLOB_RECURSE SRCS "*.cpp")

if("${OURCHAT_BUILD_TYPE}" STREQUAL "Test")
    add_library(OurChat_server STATIC
            ${SRCS}
            )
else()
    add_executable(OurChat_server
            ${SRCS}
            ourchat.rc
            )
endif()
