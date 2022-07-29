#pragma once
#include <platform.h>
#ifdef WINDOWS_PLAT
#ifdef BUILD_base_DLL
#define base_api __declspec(dllexport)
#define base_c_api extern "C" __declspec(dllexport)
#else
#define base_api__declspec(dllimport)
#define base_c_api extern "C" __declspec(dllimport)
#endif
#else
#define base_api __attribute__((visibility("default")))
#define base_c_api extern "C" __attribute__((visibility("default")))
#endif
