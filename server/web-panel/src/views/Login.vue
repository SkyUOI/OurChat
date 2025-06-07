<template>
  <div class="login-container">
    <el-card class="login-card">
      <div class="login-header">
        <img src="../../assets/logo.png" alt="Logo" class="logo" />
        <h2>{{ $t('adminLogin') }}</h2>
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
          <el-button type="primary" native-type="submit" class="login-btn" :loading="loading">
            {{ $t('login') }}
          </el-button>
        </el-form-item>

        <div class="login-footer">
          <el-checkbox v-model="rememberMe">{{ $t('rememberMe') }}</el-checkbox>
          <el-link type="primary">{{ $t('forgotPassword') }}</el-link>
        </div>
      </el-form>
    </el-card>

    <div class="copyright">© 2025 OurChat Server Control Panel. {{ $t('allRightsReserved') }}</div>
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

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 400px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.login-header {
  text-align: center;
  margin-bottom: 24px;
}

.logo {
  width: 80px;
  height: 80px;
  margin-bottom: 16px;
}

.login-btn {
  width: 100%;
  margin-top: 10px;
}

.login-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
}

.copyright {
  text-align: center;
  margin-top: 24px;
  color: #666;
  font-size: 12px;
}
</style>
