#pragma once
#include <string>

namespace ourchat {
inline const int port = 54088;

enum class client_code {
    TEXT = 0,
    EMOJI = 1,
    PICTURE = 2,
    FILE = 3,
    REGISTER = 4,
    LOGIN = 6
};

typedef unsigned int group_id_t;

typedef std::string ocid_t;
}
