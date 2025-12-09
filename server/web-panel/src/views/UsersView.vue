<template>
  <div class="users-view p-5">
    <div class="header mb-6">
      <h1 class="text-2xl font-bold">{{ $t('usersManagement') }}</h1>
      <div class="actions flex gap-2">
        <el-button type="primary" icon="el-icon-refresh" @click="refreshUsers" :loading="loading">
          {{ $t('refresh') }}
        </el-button>
        <el-button icon="el-icon-search" @click="showSearch = !showSearch">
          {{ $t('search') }}
        </el-button>
        <el-button type="success" icon="el-icon-download" @click="exportUsers">
          {{ $t('export') }}
        </el-button>
      </div>
    </div>

    <!-- Search/filter panel -->
    <el-collapse-transition>
      <div v-if="showSearch" class="search-panel mb-6 p-4 bg-gray-50 rounded-lg">
        <el-form :model="searchForm" label-width="120px">
          <el-row :gutter="20">
            <el-col :span="8">
              <el-form-item :label="$t('userStatus')">
                <el-select v-model="searchForm.account_status" clearable :placeholder="$t('selectStatus')">
                  <el-option :label="$t('statusActive')" :value="0"></el-option>
                  <el-option :label="$t('statusDeleted')" :value="1"></el-option>
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="8">
              <el-form-item :label="$t('email')">
                <el-input v-model="searchForm.email" :placeholder="$t('emailPlaceholder')" clearable></el-input>
              </el-form-item>
            </el-col>
            <el-col :span="8">
              <el-form-item :label="$t('userName')">
                <el-input v-model="searchForm.user_name" :placeholder="$t('userNamePlaceholder')" clearable></el-input>
              </el-form-item>
            </el-col>
          </el-row>
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item :label="$t('registeredAfter')">
                <el-date-picker
                  v-model="searchForm.registered_after"
                  type="datetime"
                  :placeholder="$t('selectDate')"
                  value-format="timestamp"
                  clearable
                ></el-date-picker>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item :label="$t('registeredBefore')">
                <el-date-picker
                  v-model="searchForm.registered_before"
                  type="datetime"
                  :placeholder="$t('selectDate')"
                  value-format="timestamp"
                  clearable
                ></el-date-picker>
              </el-form-item>
            </el-col>
          </el-row>
          <div class="text-right">
            <el-button @click="resetSearch">{{ $t('reset') }}</el-button>
            <el-button type="primary" @click="applySearch">{{ $t('search') }}</el-button>
          </div>
        </el-form>
      </div>
    </el-collapse-transition>

    <!-- User statistics -->
    <div class="stats-grid mb-6 grid grid-cols-1 md:grid-cols-4 gap-4">
      <el-card shadow="hover" class="text-center">
        <div class="text-3xl font-bold text-blue-600">{{ userStats.totalUsers }}</div>
        <div class="text-gray-500">{{ $t('totalUsers') }}</div>
      </el-card>
      <el-card shadow="hover" class="text-center">
        <div class="text-3xl font-bold text-green-600">{{ userStats.activeUsers }}</div>
        <div class="text-gray-500">{{ $t('activeUsers') }}</div>
      </el-card>
      <el-card shadow="hover" class="text-center">
        <div class="text-3xl font-bold text-purple-600">{{ userStats.bannedUsers }}</div>
        <div class="text-gray-500">{{ $t('bannedUsers') }}</div>
      </el-card>
      <el-card shadow="hover" class="text-center">
        <div class="text-3xl font-bold text-orange-600">{{ userStats.onlineUsers }}</div>
        <div class="text-gray-500">{{ $t('onlineUsers') }}</div>
      </el-card>
    </div>

    <!-- Users table -->
    <el-card shadow="hover" class="mb-6">
      <div class="card-header flex justify-between items-center mb-4">
        <h3 class="text-lg font-semibold">{{ $t('userList') }}</h3>
        <div class="flex items-center gap-2">
          <el-pagination
            v-model:current-page="pagination.page"
            v-model:page-size="pagination.pageSize"
            :page-sizes="[10, 20, 50, 100]"
            :total="totalUsers"
            layout="sizes, prev, pager, next, total"
            @size-change="handleSizeChange"
            @current-change="handlePageChange"
          ></el-pagination>
        </div>
      </div>

      <el-table :data="users" v-loading="loading" style="width: 100%">
        <el-table-column prop="id" :label="$t('userId')" width="100" sortable></el-table-column>
        <el-table-column prop="ocid" :label="$t('ocid')" width="150"></el-table-column>
        <el-table-column prop="email" :label="$t('email')" width="200"></el-table-column>
        <el-table-column prop="userName" :label="$t('userName')" width="150"></el-table-column>
        <el-table-column prop="accountStatus" :label="$t('status')" width="120">
          <template #default="scope">
            <el-tag :type="getStatusType(scope.row.accountStatus)" size="small">
              {{ getStatusText(scope.row.accountStatus) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="registerTime" :label="$t('registerTime')" width="180">
          <template #default="scope">
            {{ formatDate(scope.row.registerTime?.seconds) }}
          </template>
        </el-table-column>
        <el-table-column prop="sessionsCount" :label="$t('sessionsCount')" width="120" sortable></el-table-column>
        <el-table-column :label="$t('actions')" width="300" fixed="right">
          <template #default="scope">
            <div class="flex flex-wrap gap-1">
              <el-button size="small" icon="el-icon-view" @click="viewUser(scope.row)">{{ $t('view') }}</el-button>
              <el-button
                size="small"
                type="warning"
                icon="el-icon-warning"
                @click="banUser(scope.row)"
                v-if="scope.row.accountStatus === 0"
                >{{ $t('ban') }}</el-button
              >
              <el-button
                size="small"
                type="success"
                icon="el-icon-check"
                @click="unbanUser(scope.row)"
                v-else-if="scope.row.accountStatus === 2"
                >{{ $t('unban') }}</el-button
              >
              <el-button
                size="small"
                type="danger"
                icon="el-icon-delete"
                @click="deleteUser(scope.row)"
                >{{ $t('delete') }}</el-button
              >
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Batch operations -->
    <el-card shadow="hover" class="mb-6">
      <div class="card-header mb-4">
        <h3 class="text-lg font-semibold">{{ $t('batchOperations') }}</h3>
      </div>
      <div class="flex flex-wrap gap-2">
        <el-select v-model="batchAction" :placeholder="$t('selectAction')" style="width: 200px">
          <el-option :label="$t('banSelected')" value="ban"></el-option>
          <el-option :label="$t('unbanSelected')" value="unban"></el-option>
          <el-option :label="$t('muteSelected')" value="mute"></el-option>
          <el-option :label="$t('deleteSelected')" value="delete"></el-option>
          <el-option :label="$t('forceLogoutSelected')" value="forceLogout"></el-option>
        </el-select>
        <el-button type="primary" @click="executeBatchAction" :disabled="!batchAction || selectedUsers.length === 0 || batchLoading" :loading="batchLoading">
          {{ $t('execute') }}
        </el-button>
        <el-button @click="clearSelection">{{ $t('clearSelection') }}</el-button>
        <span class="text-gray-500 ml-2">
          {{ $t('selectedCount', { count: selectedUsers.length }) }}
        </span>
      </div>
    </el-card>

    <!-- Role management dialog -->
    <el-dialog
      v-model="roleDialogVisible"
      :title="$t('manageRoles')"
      width="500px"
      @close="closeRoleDialog"
    >
      <div v-if="selectedUser">
        <h4 class="mb-2">{{ $t('currentRoles') }}: {{ selectedUser.userName }}</h4>
        <el-tag
          v-for="role in userRoles"
          :key="role"
          closable
          @close="removeRole(role)"
          class="mr-2 mb-2"
        >
          {{ role }}
        </el-tag>
        <div class="mt-4">
          <el-select v-model="newRole" :placeholder="$t('selectRole')" style="width: 300px">
            <el-option v-for="role in availableRoles" :key="role.id" :label="role.name" :value="role.id"></el-option>
          </el-select>
          <el-button type="primary" @click="assignRole" class="ml-2">{{ $t('assign') }}</el-button>
        </div>
      </div>
    </el-dialog>

    <!-- User details dialog -->
    <el-dialog
      v-model="userDetailsDialogVisible"
      :title="$t('userDetails')"
      width="600px"
      @close="userDetailsDialogVisible = false"
    >
      <div v-if="selectedUser">
        <el-descriptions :column="2" border>
          <el-descriptions-item :label="$t('userId')">{{ selectedUser.id }}</el-descriptions-item>
          <el-descriptions-item :label="$t('ocid')">{{ selectedUser.ocid }}</el-descriptions-item>
          <el-descriptions-item :label="$t('email')">{{ selectedUser.email }}</el-descriptions-item>
          <el-descriptions-item :label="$t('userName')">{{ selectedUser.userName }}</el-descriptions-item>
          <el-descriptions-item :label="$t('status')">
            <el-tag :type="getStatusType(selectedUser.accountStatus)" size="small">
              {{ getStatusText(selectedUser.accountStatus) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="$t('registerTime')">{{ formatDate(selectedUser.registerTime?.seconds) }}</el-descriptions-item>
          <el-descriptions-item :label="$t('sessionsCount')">{{ selectedUser.sessionsCount }}</el-descriptions-item>
          <el-descriptions-item :label="$t('lastSeen')">{{ formatDate(selectedUser.lastSeen?.seconds) }}</el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>

    <!-- Ban/Mute dialog -->
    <el-dialog
      v-model="actionDialogVisible"
      :title="actionDialogTitle"
      width="400px"
      @close="closeActionDialog"
    >
      <el-form :model="actionForm" label-width="80px">
        <el-form-item :label="$t('reason')">
          <el-input
            v-model="actionForm.reason"
            type="textarea"
            :rows="3"
            :placeholder="$t('reasonPlaceholder')"
          ></el-input>
        </el-form-item>
        <el-form-item :label="$t('duration')">
          <el-select v-model="actionForm.duration_unit" style="width: 100px" class="mr-2">
            <el-option :label="$t('hours')" value="hours"></el-option>
            <el-option :label="$t('days')" value="days"></el-option>
            <el-option :label="$t('weeks')" value="weeks"></el-option>
            <el-option :label="$t('permanent')" value="permanent"></el-option>
          </el-select>
          <el-input-number
            v-model="actionForm.duration_value"
            :min="1"
            :max="365"
            :disabled="actionForm.duration_unit === 'permanent'"
            style="width: 120px"
          ></el-input-number>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="closeActionDialog">{{ $t('cancel') }}</el-button>
          <el-button type="primary" @click="confirmAction">{{ $t('confirm') }}</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import type { UserInfo, ListUsersRequest, BanUserRequest, AssignServerRoleRequest } from '@/api/service/server_manage/user_manage/v1/user_manage'

// Reactive state
const loading = ref(false)
const showSearch = ref(false)
const roleDialogVisible = ref(false)
const userDetailsDialogVisible = ref(false)
const actionDialogVisible = ref(false)
const actionDialogTitle = ref('')
const selectedUser = ref<UserInfo | null>(null)
const selectedUsers = ref<UserInfo[]>([])
const batchAction = ref('')
const batchLoading = ref(false)
const newRole = ref('')

// Pagination
const pagination = reactive({
  page: 1,
  pageSize: 20
})
const totalUsers = ref(0)

// Search form
const searchForm = reactive({
  account_status: undefined as number | undefined,
  email: '',
  user_name: '',
  registered_after: undefined as number | undefined,
  registered_before: undefined as number | undefined
})

// Action form
const actionForm = reactive({
  reason: '',
  duration_unit: 'days',
  duration_value: 7
})

// User statistics
const userStats = reactive({
  totalUsers: 0,
  activeUsers: 0,
  bannedUsers: 0,
  onlineUsers: 0
})



// Users list
const users = ref<UserInfo[]>([])

// Available roles (to be fetched from server)
const availableRoles = ref<{id: number, name: string}[]>([])

// User roles (to be fetched from server)
const userRoles = ref<string[]>([])

// gRPC store
const grpcStore = useGrpcStore()

// Helper functions
const formatDate = (timestamp: number | bigint | undefined): string => {
  if (!timestamp) return '-'
  const date = new Date(Number(timestamp) * 1000)
  return date.toLocaleString()
}

const getStatusType = (status: number): string => {
  switch (status) {
    case 0: return 'success' // Active
    case 1: return 'info'    // Deleted
    case 2: return 'warning' // Banned
    default: return 'default'
  }
}

const getStatusText = (status: number): string => {
  switch (status) {
    case 0: return 'Active'
    case 1: return 'Deleted'
    case 2: return 'Banned'
    default: return 'Unknown'
  }
}


// Perform user action (single user, used for batch operations)
const performUserAction = async (user: UserInfo, action: string, reason?: string, durationUnit?: string, durationValue?: number) => {
  try {
    switch (action) {
      case 'ban':
        const banRequest: BanUserRequest = {
          userId: user.id,
          reason: reason || undefined,
          duration: durationUnit === 'permanent' ? undefined : {
            seconds: BigInt((durationValue || 1) *
              (durationUnit === 'hours' ? 3600 :
               durationUnit === 'days' ? 86400 : 604800)),
            nanos: 0
          }
        }
        await grpcStore.serverManageConn.banUser(banRequest)
        ElMessage.success(`User ${user.userName} banned successfully`)
        break
      case 'unban':
        await grpcStore.serverManageConn.unbanUser({ userId: user.id })
        ElMessage.success(`User ${user.userName} unbanned successfully`)
        break
      case 'delete':
        await grpcStore.serverManageConn.deleteAccount({ userId: user.id })
        ElMessage.success(`User ${user.userName} deleted successfully`)
        break
      case 'forceLogout':
        // ForceLogoutUser API not implemented yet
        ElMessage.info(`Force logout for user ${user.userName} - Not implemented yet`)
        break
      default:
        throw new Error(`Unknown action: ${action}`)
    }
  } catch (error: unknown) {
    console.error(`${action} user error:`, error)
    ElMessage.error(`Failed to ${action} user: server API not available`)
  }
}

// Fetch users via gRPC
const fetchUsers = async () => {
  try {
    loading.value = true

    // Build request
    const request: ListUsersRequest = {
      filter: {
        accountStatus: searchForm.account_status,
        email: searchForm.email || undefined,
        userName: searchForm.user_name || undefined,
        registeredAfter: searchForm.registered_after ? { seconds: BigInt(Math.floor(searchForm.registered_after / 1000)), nanos: 0 } : undefined,
        registeredBefore: searchForm.registered_before ? { seconds: BigInt(Math.floor(searchForm.registered_before / 1000)), nanos: 0 } : undefined
      },
      pagination: {
        page: pagination.page,
        pageSize: pagination.pageSize
      }
    }

    // Try gRPC call
    const response = await grpcStore.serverManageConn.listUsers(request)
    users.value = response.response.users || []
    totalUsers.value = response.response.totalCount || 0

    // Update stats
    updateUserStats()

    ElMessage.success('Users list updated successfully')
  } catch (error) {
    console.error('Fetch users error:', error)
    ElMessage.error('Failed to fetch users')
  } finally {
    loading.value = false
  }
}

// Update user statistics
const updateUserStats = () => {
  // Statistics computed from currently displayed users (paginated)
  const displayedUsers = users.value
  userStats.totalUsers = displayedUsers.length
  userStats.activeUsers = displayedUsers.filter(u => u.accountStatus === 0).length
  userStats.bannedUsers = displayedUsers.filter(u => u.accountStatus === 2).length
  userStats.onlineUsers = 0 // Online count requires separate API
}

// Search handlers
const applySearch = () => {
  pagination.page = 1
  fetchUsers()
}

const resetSearch = () => {
  searchForm.account_status = undefined
  searchForm.email = ''
  searchForm.user_name = ''
  searchForm.registered_after = undefined
  searchForm.registered_before = undefined
  pagination.page = 1
  fetchUsers()
}

// Pagination handlers
const handleSizeChange = (size: number) => {
  pagination.pageSize = size
  pagination.page = 1
  fetchUsers()
}

const handlePageChange = (page: number) => {
  pagination.page = page
  fetchUsers()
}

// User action handlers
const viewUser = (user: UserInfo) => {
  selectedUser.value = user
  userDetailsDialogVisible.value = true
}

const banUser = (user: UserInfo) => {
  selectedUser.value = user
  actionDialogTitle.value = 'Ban User'
  actionDialogVisible.value = true
}

const unbanUser = async (user: UserInfo) => {
  try {
    await ElMessageBox.confirm(`Are you sure to unban user ${user.userName}?`, 'Confirm Unban', { type: 'warning' })

    const request: BanUserRequest = { userId: user.id }
    await grpcStore.serverManageConn.unbanUser(request)
    ElMessage.success(`User ${user.userName} unbanned successfully`)
    fetchUsers()
  } catch (error: unknown) {
    if (error !== 'cancel') {
      console.error('Unban user error:', error)
      ElMessage.error('Failed to unban user: server API not available')
    }
  }
}

const deleteUser = async (user: UserInfo) => {
  try {
    await ElMessageBox.confirm(`Are you sure to delete user ${user.userName}? This action cannot be undone.`, 'Confirm Delete', { type: 'error' })

    // Use existing DeleteAccount API
    await grpcStore.serverManageConn.deleteAccount({ userId: user.id })
    ElMessage.success(`User ${user.userName} deleted successfully`)
    fetchUsers()
  } catch (error: unknown) {
    if (error !== 'cancel') {
      console.error('Delete user error:', error)
      ElMessage.error('Failed to delete user')
    }
  }
}

const closeRoleDialog = () => {
  roleDialogVisible.value = false
  selectedUser.value = null
  userRoles.value = []
  newRole.value = ''
}

const assignRole = async () => {
  if (!newRole.value || !selectedUser.value) return

  try {
    const request: AssignServerRoleRequest = {
      userId: selectedUser.value.id,
      roleId: BigInt(newRole.value)
    }
    await grpcStore.serverManageConn.assignServerRole(request)
    ElMessage.success('Role assigned successfully')
    newRole.value = ''
  } catch (error: unknown) {
    console.error('Assign role error:', error)
    ElMessage.error('Failed to assign role: server API not available')
  }
}

const removeRole = async (role: string) => {
  if (!selectedUser.value) return

  try {
    // TODO: Call RemoveServerRole RPC when implemented
    // For now, update local UI state
    userRoles.value = userRoles.value.filter(r => r !== role)
    ElMessage.success(`Role ${role} removed from local UI`)
  } catch (error) {
    console.error('Remove role error:', error)
    ElMessage.error('Failed to remove role')
  }
}

// Action dialog handlers
const closeActionDialog = () => {
  actionDialogVisible.value = false
  selectedUser.value = null
  actionForm.reason = ''
  actionForm.duration_unit = 'days'
  actionForm.duration_value = 7
}

const confirmAction = async () => {
  if (!selectedUser.value) return

  try {
    if (actionDialogTitle.value.includes('Ban')) {
      const request: BanUserRequest = {
        userId: selectedUser.value.id,
        reason: actionForm.reason || undefined,
        duration: actionForm.duration_unit === 'permanent' ? undefined : {
          seconds: BigInt(actionForm.duration_value *
            (actionForm.duration_unit === 'hours' ? 3600 :
             actionForm.duration_unit === 'days' ? 86400 : 604800)),
          nanos: 0
        }
      }
      await grpcStore.serverManageConn.banUser(request)
      ElMessage.success(`User ${selectedUser.value.userName} banned successfully`)
    }

    closeActionDialog()
    fetchUsers()
  } catch (error: unknown) {
    console.error('Action error:', error)
    ElMessage.error('Failed to execute action: server API not available')
  }
}

// Batch operations
const executeBatchAction = async () => {
  if (selectedUsers.value.length === 0 || !batchAction.value) return

  try {
    const userNames = selectedUsers.value.map(u => u.userName).join(', ')
    await ElMessageBox.confirm(
      `Are you sure to ${batchAction.value} ${selectedUsers.value.length} user(s) (${userNames})?`,
      `Confirm ${batchAction.value}`,
      { type: 'warning' }
    )

    batchLoading.value = true
    // Execute batch action by looping through selected users
    // Use default parameters for ban/mute (permanent, no reason)
    for (const user of selectedUsers.value) {
      await performUserAction(user, batchAction.value, '', 'permanent', 1)
    }
    ElMessage.success(`Batch ${batchAction.value} completed for ${selectedUsers.value.length} user(s)`)

    // Clear selection and refresh
    selectedUsers.value = []
    batchAction.value = ''
    fetchUsers()
  } catch (error: unknown) {
    if (error !== 'cancel') {
      console.error('Batch action error:', error)
      ElMessage.error('Failed to execute batch action')
    }
  } finally {
    batchLoading.value = false
  }
}

const clearSelection = () => {
  selectedUsers.value = []
  batchAction.value = ''
}

// Refresh users (button handler)
const refreshUsers = () => {
  fetchUsers()
}

// Export users
const exportUsers = () => {
  try {
    const data = users.value
    if (data.length === 0) {
      ElMessage.warning('No users to export')
      return
    }

    // Define CSV headers
    const headers = ['ID', 'OCID', 'Email', 'Username', 'Account Status', 'Register Time', 'Sessions Count']

    // Convert data to rows
    const rows = data.map(user => [
      user.id,
      user.ocid || '',
      user.email || '',
      user.userName || '',
      user.accountStatus !== undefined ? user.accountStatus : '',
      user.registerTime ? new Date(Number(user.registerTime.seconds) * 1000).toISOString() : '',
      user.sessionsCount || ''
    ])

    // Combine headers and rows
    const csvContent = [
      headers.join(','),
      ...rows.map(row => row.map(field => `"${String(field).replace(/"/g, '""')}"`).join(','))
    ].join('\n')

    // Create download link
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    const url = URL.createObjectURL(blob)
    link.setAttribute('href', url)
    link.setAttribute('download', `users_export_${new Date().toISOString().split('T')[0]}.csv`)
    link.style.visibility = 'hidden'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    ElMessage.success(`Exported ${data.length} users successfully`)
  } catch (error) {
    console.error('Export users error:', error)
    ElMessage.error('Failed to export users')
  }
}

// Initialize on mount
onMounted(() => {
  // Load initial data
  updateUserStats()
  fetchUsers()
})
</script>

<style scoped>
.users-view {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.actions {
  display: flex;
  gap: 10px;
}

.search-panel {
  background-color: #f9fafb;
  border-radius: 8px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(1, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

@media (min-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
</style>
