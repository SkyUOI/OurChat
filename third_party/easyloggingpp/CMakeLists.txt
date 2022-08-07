cmake_minimum_required(VERSION 3.16)

project(Easyloggingpp CXX)

macro(require_cpp11)
        if (${CMAKE_MAJOR_VERSION}.${CMAKE_MINOR_VERSION} GREATER 3.0)
                # CMake 3.1 has built-in CXX standard checks.
                message("-- Setting C++11")
                set(CMAKE_CXX_STANDARD 11)
                set(CMAKE_CXX_STANDARD_REQUIRED on)
        else()
                if (CMAKE_CXX_COMPILER_ID MATCHES "GCC")
                    message ("-- GNU CXX (-std=c++11)")
                    list(APPEND CMAKE_CXX_FLAGS "-std=c++11")
                elseif (CMAKE_CXX_COMPILER_ID MATCHES "Clang")
                    message ("-- CLang CXX (-std=c++11)")
                    list(APPEND CMAKE_CXX_FLAGS "-std=c++11")
                else()
                    message ("-- Easylogging++ requires C++11. Your compiler does not support it.")
                endif()
        endif()
endmacro()

option(build_static_lib "Build easyloggingpp as a static library" ON)
option(lib_utc_datetime "Build library with UTC date/time logging" OFF)

set(ELPP_MAJOR_VERSION "9")
set(ELPP_MINOR_VERSION "96")
set(ELPP_PATCH_VERSION "7")
set(ELPP_VERSION_STRING "${ELPP_MAJOR_VERSION}.${ELPP_MINOR_VERSION}.${ELPP_PATCH_VERSION}")

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

configure_file(cmake/easyloggingpp.pc.cmakein
                easyloggingpp.pc @ONLY)

include(CheckIncludeFileCXX)
check_include_file_cxx("execinfo.h" HAVE_EXECINFO)
if (HAVE_EXECINFO)
	add_definitions(-DHAVE_EXECINFO)
endif()

if (lib_utc_datetime)
        add_definitions(-DELPP_UTC_DATETIME)
endif()

require_cpp11()
add_library(easyloggingpp STATIC easylogging++.cc)
set_property(TARGET easyloggingpp PROPERTY POSITION_INDEPENDENT_CODE ON)
            
