file(GLOB_RECURSE SRCS "*.cpp")

add_library(base STATIC
    ${SRCS}
)
target_link_libraries(base libmysql)
