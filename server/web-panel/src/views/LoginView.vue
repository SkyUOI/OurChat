<template>
  <div class="login-container">
    <div class="background-decoration">
      <div class="circle circle-1"></div>
      <div class="circle circle-2"></div>
      <div class="circle circle-3"></div>
    </div>

    <el-card class="login-card card-hover">
      <div class="text-center mb-8 logo-animation">
        <h2 class="text-3xl font-bold gradient-text">{{ $t('serverControlPanel') }}</h2>
        <p class="text-slate-500 mt-2">{{ $t('adminLogin') }}</p>
      </div>

      <!-- Connection status indicator -->
      <div v-if="connectionStatus" class="mb-4">
        <el-alert
          :title="connectionStatus === 'success' ? $t('connectionSuccess') : $t('connectionFailed')"
          :type="connectionStatus === 'success' ? 'success' : 'error'"
          :closable="false"
          show-icon
        />
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
            size="large"
          ></el-input>
        </el-form-item>

        <el-form-item prop="password">
          <el-input
            v-model="loginForm.password"
            type="password"
            :placeholder="$t('passwordPlaceholder')"
            prefix-icon="el-icon-lock"
            show-password
            size="large"
          ></el-input>
        </el-form-item>

        <el-form-item>
          <el-button
            type="primary"
            native-type="submit"
            class="w-full mt-3 login-btn"
            :loading="loading"
            :disabled="connectionStatus === 'failed'"
            size="large"
          >
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
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import * as basicPb from '../api/service/basic/v1/basic'
import { AuthRequest } from '../api/service/auth/authorize/v1/authorize'
import { useGrpcStore } from '../stores/grpc'

const loginFormRef = ref<InstanceType<typeof ElForm> | null>(null)
const loading = ref(false)
const rememberMe = ref(false)
const connectionStatus = ref<'success' | 'failed' | null>(null)

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
const grpcStore = useGrpcStore()

onMounted(async () => {
  await checkConnection()
})

const checkConnection = async () => {
  try {
    const client = grpcStore.basicConn
    const request = basicPb.GetServerInfoRequest.create()
    await client.getServerInfo(request, {})
    connectionStatus.value = 'success'
  } catch (error) {
    console.error('Connection test failed:', error)
    connectionStatus.value = 'failed'
    ElMessage.error(t('connectionFailed'))
  }
}

const handleLogin = async () => {
  const valid = await loginFormRef.value?.validate().catch(() => false)
  if (!valid) return

  loading.value = true

  try {
    const client = grpcStore.authConn
    const request = AuthRequest.create({
      account: {
        oneofKind: 'email',
        email: loginForm.value.username,
      },
      password: loginForm.value.password,
    })

    const response = await client.auth(request, {})
    const authResponse = response.response

    localStorage.setItem('token', authResponse.token)
    localStorage.setItem('userId', authResponse.id.toString())
    localStorage.setItem('userOcid', authResponse.ocid)

    ElMessage.success(t('loginSuccess'))
    router.push('/')
  } catch (error) {
    console.error('Login failed:', error)
    ElMessage.error(t('loginFailed'))
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #f8fafc 0%, #dbeafe 50%, #f3e8ff 100%);
  padding: 20px;
  position: relative;
  overflow: hidden;
}

.background-decoration .circle {
  position: absolute;
  border-radius: 50%;
  opacity: 0.15;
  animation: float 20s infinite ease-in-out;
}

.circle-1 {
  width: 400px;
  height: 400px;
  background: linear-gradient(135deg, #3b82f6, #60a5fa);
  top: -100px;
  left: -100px;
  animation-delay: 0s;
}

.circle-2 {
  width: 320px;
  height: 320px;
  background: linear-gradient(135deg, #8b5cf6, #a78bfa);
  top: 50%;
  right: -80px;
  animation-delay: 5s;
}

.circle-3 {
  width: 240px;
  height: 240px;
  background: linear-gradient(135deg, #3b82f6, #60a5fa);
  bottom: 40px;
  left: 33%;
  animation-delay: 10s;
}

@keyframes float {
  0%,
  100% {
    transform: translate(0, 0) scale(1);
  }
  25% {
    transform: translate(50px, -50px) scale(1.1);
  }
  50% {
    transform: translate(-30px, 30px) scale(0.9);
  }
  75% {
    transform: translate(40px, 20px) scale(1.05);
  }
}

.login-card {
  width: 100%;
  max-width: 420px;
  border-radius: 20px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  border: none;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  position: relative;
  z-index: 10;
  padding: 8px;
}

.logo-animation {
  animation: fadeInDown 0.6s ease-out;
}

@keyframes fadeInDown {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.gradient-text {
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.login-btn {
  height: 44px;
  font-size: 15px;
  font-weight: 500;
  transition: all 0.2s ease;
  background: linear-gradient(90deg, #3b82f6, #6366f1);
  border: none;
}

.login-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(59, 130, 246, 0.35);
}

.login-btn:active {
  transform: translateY(0);
}
</style>
