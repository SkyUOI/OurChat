#去除那些烦人的警告
add_definitions(-D _CRT_SECURE_NO_WARNINGS)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(server)

add_executable(OurChat_Client
	client/main.cpp
)
target_link_libraries(OurChat_Client ws2_32)
target_link_libraries(OurChat_server jsoncpp ws2_32 easyloggingpp wsock32)
