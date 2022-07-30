file(GLOB_RECURSE SRCS "*.cpp")
add_executable(OurChar_server
        ${SRCS}
        ourchat.rc
        )
