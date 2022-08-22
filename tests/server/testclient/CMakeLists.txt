file(GLOB_RECURSE SRCS "*.cpp")

add_executable(TestClient
        ${SRCS}
        )
if(CMAKE_SYSTEM_NAME MATCHES "Linux")
    target_link_libraries(TestClient pthread)
elseif(CMAKE_SYSTEM_NAME MATCHES "Windows")
    target_link_libraries(TestClient ws2_32)
endif()
target_link_libraries(TestClient base jsoncpp)
