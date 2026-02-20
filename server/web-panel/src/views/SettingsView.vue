<template>
  <div class="settings-view p-5">
    <div class="header mb-6">
      <h1 class="text-2xl font-bold">{{ $t('settings') }}</h1>
    </div>

    <el-row :gutter="20">
      <!-- Profile Settings -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>{{ $t('profile') }}</span>
            </div>
          </template>

          <el-form :model="profileForm" label-width="100px">
            <el-form-item :label="$t('userName')">
              <el-input
                v-model="profileForm.userName"
                :placeholder="$t('userNamePlaceholder')"
                clearable
              ></el-input>
            </el-form-item>

            <el-form-item label="Status">
              <el-input
                v-model="profileForm.status"
                placeholder="Set your status"
                clearable
              ></el-input>
            </el-form-item>

            <el-form-item label="Avatar Key">
              <el-input
                v-model="profileForm.avatarKey"
                placeholder="Avatar key (optional)"
                clearable
              ></el-input>
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                @click="saveProfile"
                :loading="savingProfile"
                icon="el-icon-check"
              >
                {{ $t('save') }}
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <!-- Account Info -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Account Information</span>
            </div>
          </template>

          <el-descriptions :column="1" border>
            <el-descriptions-item label="OCID">{{ currentOcid || '-' }}</el-descriptions-item>
            <el-descriptions-item label="User ID">{{ currentUserId || '-' }}</el-descriptions-item>
            <el-descriptions-item label="Email">{{ userEmail || '-' }}</el-descriptions-item>
            <el-descriptions-item label="Roles">
              <el-tag v-for="role in userRoles" :key="role.id" type="primary" class="mr-2">
                {{ role.name }}
              </el-tag>
              <span v-if="userRoles.length === 0">-</span>
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>

    <!-- Security Section -->
    <el-row :gutter="20" class="mt-4">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Security</span>
            </div>
          </template>

          <el-form label-width="150px">
            <el-form-item label="Change Password">
              <el-button type="warning" icon="el-icon-lock">Change Password</el-button>
              <span class="ml-3 text-gray-500 text-sm">Not implemented yet</span>
            </el-form-item>

            <el-form-item label="Two-Factor Auth">
              <el-button type="info" icon="el-icon-key">Enable 2FA</el-button>
              <span class="ml-3 text-gray-500 text-sm">Not implemented yet</span>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import type { SetSelfInfoRequest } from '@/api/service/ourchat/set_account_info/v1/set_account_info'

// gRPC store
const grpcStore = useGrpcStore()

// Loading states
const savingProfile = ref(false)

// Current user info
const currentOcid = ref('')
const currentUserId = ref('')
const userEmail = ref('')
const userRoles = ref<{ id: number; name: string; description: string }[]>([])

// Profile form
const profileForm = reactive({
  userName: '',
  status: '',
  avatarKey: '',
})

// Load current user info
const loadUserInfo = async () => {
  const ocid = localStorage.getItem('userOcid')
  const userId = localStorage.getItem('userId')

  currentOcid.value = ocid || ''
  currentUserId.value = userId || ''

  if (!ocid) return

  try {
    // Fetch account info
    const { GetAccountInfoRequest, QueryValues } = await import(
      '@/api/service/ourchat/get_account_info/v1/get_account_info'
    )
    const request = GetAccountInfoRequest.create({
      requestValues: [
        QueryValues.USER_NAME,
        QueryValues.STATUS,
        QueryValues.AVATAR_KEY,
        QueryValues.EMAIL,
      ],
    })
    const response = await grpcStore.ourchatConn.getAccountInfo(request)

    profileForm.userName = response.response.userName || ''
    profileForm.status = response.response.status || ''
    profileForm.avatarKey = response.response.avatarKey || ''
    userEmail.value = response.response.email || ''
  } catch (error) {
    console.error('Failed to fetch account info:', error)
  }

  // Fetch roles
  if (userId) {
    try {
      const roleIdsRes = await grpcStore.serverManageConn.listUserServerRoles({
        userId: BigInt(userId),
      })
      const roleIds = roleIdsRes.response.roleIds || []

      const rolesRes = await grpcStore.serverManageConn.listServerRoles({})
      const allRoles = rolesRes.response.roles || []

      const roleIdStrings = roleIds.map((id) => id.toString())
      userRoles.value = allRoles
        .filter((r) => roleIdStrings.includes(r.id.toString()))
        .map((r) => ({ id: Number(r.id), name: r.name, description: r.description || '' }))
    } catch (error) {
      console.error('Failed to fetch roles:', error)
    }
  }
}

// Save profile
const saveProfile = async () => {
  savingProfile.value = true
  try {
    const request: SetSelfInfoRequest = {
      userName: profileForm.userName || undefined,
      userDefinedStatus: profileForm.status || undefined,
      avatarKey: profileForm.avatarKey || undefined,
    }

    await grpcStore.ourchatConn.setSelfInfo(request)
    ElMessage.success('Profile updated successfully')
  } catch (error: unknown) {
    console.error('Failed to update profile:', error)
    ElMessage.error('Failed to update profile')
  } finally {
    savingProfile.value = false
  }
}

onMounted(() => {
  loadUserInfo()
})
</script>

<style scoped>
.settings-view {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header {
  font-weight: bold;
  font-size: 16px;
}

.mt-4 {
  margin-top: 16px;
}

.text-gray-500 {
  color: #909399;
}

.text-sm {
  font-size: 14px;
}
</style>
