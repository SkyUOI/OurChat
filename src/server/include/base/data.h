#pragma once

#include <random>

namespace ourchat::utils {
extern std::default_random_engine random_engine;

/**
 * @brief 生成随机正整数，区间为左闭右闭
 * @param start 随机数区间开始
 * @param end 随机数区间结尾
 */
inline unsigned int random(unsigned int start, unsigned int end) {
    return (random_engine() % (end - start + 1) + start);
}

/**
 * @brief 获取数组的长度
 * @tparam T 数组类型
 * @param arr 数组
 * @return 数组的长度
 */
template <typename T> constexpr size_t sizeof_static_array(T& arr) {
    return sizeof(arr) / sizeof(arr[0]);
}
}