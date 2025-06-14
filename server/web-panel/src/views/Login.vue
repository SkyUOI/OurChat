<template>
  <div class="flex justify-center items-center min-h-screen bg-gradient-to-br from-[#f5f7fa] to-[#c3cfe2] p-5">
    <el-card class="w-full max-w-md rounded-lg shadow-lg">
      <div class="text-center mb-6">
        <img src="../../assets/logo.png" alt="Logo" class="w-20 h-20 mx-auto mb-4" />
        <h2 class="text-2xl font-bold">{{ $t('adminLogin') }}</h2>
      </div>

      <el-form
        :model="loginForm"
        :rules="loginRules"
        ref="loginFormRef"
        @submit.prevent="handleLogin"
      >
        <el-form-item prop="username">
          <el-input
            v-model="loginForm.username"
            :placeholder="$t('usernamePlaceholder')"
            prefix-icon="el-icon-user"
          ></el-input>
        </el-form-item>

        <el-form-item prop="password">
          <el-input
            v-model="loginForm.password"
            type="password"
            :placeholder="$t('passwordPlaceholder')"
            prefix-icon="el-icon-lock"
            show-password
          ></el-input>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" native-type="submit" class="w-full mt-3" :loading="loading">
            {{ $t('login') }}
          </el-button>
        </el-form-item>

        <div class="flex justify-between items-center mt-4">
          <el-checkbox v-model="rememberMe">{{ $t('rememberMe') }}</el-checkbox>
          <el-link type="primary">{{ $t('forgotPassword') }}</el-link>
        </div>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ElMessage, type ElForm } from 'element-plus'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const loginFormRef = ref<InstanceType<typeof ElForm> | null>(null)
const loading = ref(false)
const rememberMe = ref(false)
const loginForm = ref({
  username: '',
  password: '',
})
const { t } = useI18n()
const loginRules = {
  username: [{ required: true, message: t('usernameRequired'), trigger: 'blur' }],
  password: [{ required: true, message: t('passwordRequired'), trigger: 'blur' }],
}
const router = useRouter()

const handleLogin = () => {
  loginFormRef.value?.validate((valid) => {
    if (valid) {
      loading.value = true
      // 模拟登录请求
      setTimeout(() => {
        loading.value = false
        ElMessage.success(t('loginSuccess'))

        // 设置模拟token
        localStorage.setItem('token', 'mock-token-value')

        // 跳转到首页
        router.push('/')
      }, 1500)
    }
  })
}
</script>
