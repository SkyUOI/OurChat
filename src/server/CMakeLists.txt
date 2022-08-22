#去除那些烦人的警告
add_definitions(-D _CRT_SECURE_NO_WARNINGS)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_VISIBILITY_PRESET hidden)
set(compile_flag " -Wextra -Wall -fdata-sections -ffunction-sections")
if ("${CMAKE_CXX_COMPILER_ID}" MATCHES "Clang" OR "${CMAKE_CXX_COMPILER_ID}" MATCHES "GNU")
	set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} ${compile_flag}")
	set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -O0 ${compile_flag}")
	set(CMAKE_CXX_FLAGS_MinSizeRel "${CMAKE_CXX_FLAGS_MinSizeRel} ${compile_flag}")
	set(CMAKE_CXX_FLAGS_RELWITHDEBINFO "${CMAKE_CXX_FLAGS_RELWITHDEBINFO} ${compile_flag}")
endif()

add_subdirectory(server)
add_subdirectory(base)
