file(GLOB_RECURSE SRCS "*.cpp")
add_library(base SHARED
    ${SRCS}
)
target_compile_definitions(base PRIVATE BUILD_base_DLL)
