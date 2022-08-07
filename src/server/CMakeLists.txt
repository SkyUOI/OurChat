#去除那些烦人的警告
add_definitions(-D _CRT_SECURE_NO_WARNINGS)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(server)
add_subdirectory(base)

add_executable(OurChat_Client
	client/main.cpp
)
if(CMAKE_SYSTEM_NAME MATCHES "Linux")
	target_link_libraries(OurChat_Client pthread)
	target_link_libraries(OurChat_server pthread)
elseif(CMAKE_SYSTEM_NAME MATCHES "Windows")
	target_link_libraries(OurChat_Client ws2_32)
	target_link_libraries(OurChat_server ws2_32 wsock32)
endif()

target_link_libraries(OurChat_Client base)
target_link_libraries(OurChat_server jsoncpp easyloggingpp base)
