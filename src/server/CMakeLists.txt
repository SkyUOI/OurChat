#去除那些烦人的警告
add_definitions(-D _CRT_SECURE_NO_WARNINGS)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(base)
add_subdirectory(server)

target_link_libraries(serverbase ws2_32 easyloggingpp)
target_link_libraries(OurChat_server serverbase jsoncpp)
