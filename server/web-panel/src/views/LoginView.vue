<template>
  <div
    class="flex justify-center items-center min-h-screen bg-linear-to-br from-[#f5f7fa] to-[#c3cfe2] p-5"
  >
    <el-card class="w-full max-w-md rounded-lg shadow-lg">
      <div class="text-center mb-6">
        <!-- <img src="../assets/logo.png" alt="Logo" class="w-20 h-20 mx-auto mb-4" /> -->
        <h2 class="text-2xl font-bold">{{ $t('adminLogin') }}</h2>
      </div>

      <!-- Connection status indicator -->
      <div v-if="connectionStatus" class="mb-4 text-center">
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
          <el-button
            type="primary"
            native-type="submit"
            class="w-full mt-3"
            :loading="loading"
            :disabled="connectionStatus === 'failed'"
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
