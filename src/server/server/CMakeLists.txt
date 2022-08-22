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

if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    target_link_libraries(OurChat_server pthread)
elseif(CMAKE_SYSTEM_NAME MATCHES "Windows")
    target_link_libraries(OurChat_server ws2_32 wsock32)
endif()

target_link_libraries(OurChat_server jsoncpp easyloggingpp base hash_library)
