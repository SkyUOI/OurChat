file(GLOB_RECURSE SRCS "*.cpp")
add_library(serverbase SHARED
    ${SRCS}
)
target_compile_definitions(serverbase PRIVATE BUILD_base_DLL)
