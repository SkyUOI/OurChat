<template>
  <div class="sessions-view p-5">
    <div class="header mb-6">
      <h1 class="text-2xl font-bold">{{ $t('sessionsManagement') }}</h1>
      <div class="actions flex gap-2">
        <el-button
          type="primary"
          icon="el-icon-refresh"
          @click="refreshSessions"
          :loading="loading"
        >
          {{ $t('refresh') }}
        </el-button>
        <el-button icon="el-icon-search" @click="showSearch = !showSearch">
          {{ $t('search') }}
        </el-button>
        <el-button type="success" icon="el-icon-download" @click="exportSessions">
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
              <el-form-item :label="$t('sessionType')">
                <el-select
                  v-model="searchForm.session_type"
                  clearable
                  :placeholder="$t('selectSessionType')"
                >
                  <el-option :label="$t('sessionTypePrivate')" :value="1"></el-option>
                  <el-option :label="$t('sessionTypeGroup')" :value="2"></el-option>
                  <el-option :label="$t('sessionTypeChannel')" :value="3"></el-option>
                </el-select>
              </el-form-item>
            </el-col>
            <el-col :span="8">
              <el-form-item :label="$t('sessionName')">
                <el-input
                  v-model="searchForm.session_name"
                  :placeholder="$t('sessionNamePlaceholder')"
                  clearable
                ></el-input>
              </el-form-item>
            </el-col>
            <el-col :span="8">
              <el-form-item :label="$t('ownerId')">
                <el-input
                  v-model="searchForm.owner_id"
                  :placeholder="$t('ownerIdPlaceholder')"
                  clearable
                ></el-input>
              </el-form-item>
            </el-col>
          </el-row>
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item :label="$t('createdAfter')">
                <el-date-picker
                  v-model="searchForm.created_after"
                  type="datetime"
                  :placeholder="$t('selectDate')"
                  value-format="timestamp"
                  clearable
                ></el-date-picker>
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item :label="$t('createdBefore')">
                <el-date-picker
                  v-model="searchForm.created_before"
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

    <!-- Session statistics -->
    <div class="stats-grid mb-6 grid grid-cols-1 md:grid-cols-1 gap-4">
      <el-card shadow="hover" class="text-center">
        <div class="text-3xl font-bold text-blue-600">{{ sessionStats.totalSessions }}</div>
        <div class="text-gray-500">{{ $t('totalSessions') }}</div>
      </el-card>
    </div>

    <!-- Sessions table -->
    <el-card shadow="hover" class="mb-6">
      <div class="card-header flex justify-between items-center mb-4">
        <h3 class="text-lg font-semibold">{{ $t('sessionList') }}</h3>
        <div class="flex items-center gap-2">
          <el-pagination
            v-model:current-page="pagination.page"
            v-model:page-size="pagination.pageSize"
            :page-sizes="[10, 20, 50, 100]"
            layout="sizes, prev, pager, next"
            :total="totalSessions"
            @size-change="handleSizeChange"
            @current-change="handlePageChange"
          />
        </div>
      </div>
      <el-table v-loading="loading" :data="sessions" @selection-change="handleSelectionChange">
        <el-table-column type="selection" width="55" />
        <el-table-column prop="sessionId" :label="$t('sessionId')" width="120">
          <template #default="{ row }">
            {{ row.sessionId?.toString() }}
          </template>
        </el-table-column>
        <el-table-column prop="name" :label="$t('sessionName')" min-width="180" />
        <el-table-column prop="size" :label="$t('size')" width="100">
          <template #default="{ row }">
            {{ row.size?.toString() }}
          </template>
        </el-table-column>
        <el-table-column prop="createdTime" :label="$t('createdAt')" width="180">
          <template #default="{ row }">
            {{ formatDate(row.createdTime) }}
          </template>
        </el-table-column>
        <el-table-column prop="updatedTime" :label="$t('updatedAt')" width="180">
          <template #default="{ row }">
            {{ formatDate(row.updatedTime) }}
          </template>
        </el-table-column>
        <el-table-column :label="$t('actions')" width="280" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" icon="el-icon-view" @click="viewSession(row)">
              {{ $t('view') }}
            </el-button>
            <el-button
              type="warning"
              size="small"
              icon="el-icon-remove"
              @click="removeUserFromSession(row)"
            >
              {{ $t('removeUser') }}
            </el-button>
            <el-button type="danger" size="small" icon="el-icon-delete" @click="deleteSession(row)">
              {{ $t('delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Batch operations -->
    <div v-if="selectedSessions.length > 0" class="batch-operations mb-6 p-4 bg-gray-50 rounded-lg">
      <div class="flex items-center justify-between">
        <div class="text-gray-700">
          {{ $t('selectedCount', { count: selectedSessions.length }) }}
        </div>
        <div class="flex items-center gap-2">
          <el-select v-model="batchAction" :placeholder="$t('selectAction')" style="width: 180px">
            <el-option :label="$t('deleteSelected')" value="delete"></el-option>
          </el-select>
          <el-button
            type="primary"
            @click="executeBatchAction"
            :loading="batchLoading"
            :disabled="!batchAction || selectedSessions.length === 0 || batchLoading"
          >
            {{ $t('execute') }}
          </el-button>
          <el-button @click="clearSelection">{{ $t('clearSelection') }}</el-button>
        </div>
      </div>
    </div>

    <!-- Session details dialog -->
    <el-dialog
      v-model="sessionDetailsDialogVisible"
      :title="$t('sessionDetails')"
      width="600px"
      @close="sessionDetailsDialogVisible = false"
    >
      <div v-if="selectedSessionDetails">
        <el-descriptions :column="2" border>
          <el-descriptions-item :label="$t('sessionId')">{{
            selectedSessionDetails.sessionId?.toString()
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('sessionName')">{{
            selectedSessionDetails.name
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('size')">{{
            selectedSessionDetails.size?.toString()
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('createdAt')">{{
            formatDate(selectedSessionDetails.createdTime)
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('updatedAt')">{{
            formatDate(selectedSessionDetails.updatedTime)
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('avatarKey')">{{
            selectedSessionDetails.avatarKey || '-'
          }}</el-descriptions-item>
          <el-descriptions-item :label="$t('description')" :span="2">{{
            selectedSessionDetails.description || '-'
          }}</el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>

    <!-- Remove User Dialog -->
    <el-dialog v-model="removeUserDialogVisible" :title="$t('removeUserFromSession')" width="500px">
      <el-form :model="removeUserForm" label-width="120px">
        <el-form-item :label="$t('userId')">
          <el-input
            v-model="removeUserForm.user_id"
            :placeholder="$t('userIdPlaceholder')"
          ></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="closeRemoveUserDialog">{{ $t('cancel') }}</el-button>
          <el-button type="primary" @click="confirmRemoveUser" :loading="removeUserLoading">
            {{ $t('confirm') }}
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts" setup>
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import type { ListSessionsRequest } from '@/api/service/server_manage/session_manage/v1/session_manage'
import type { GetSessionInfoResponse } from '@/api/service/ourchat/session/get_session_info/v1/get_session_info'
import type { DeleteSessionRequest as OurChatDeleteSessionRequest } from '@/api/service/ourchat/session/delete_session/v1/delete_session'
import type { KickUserRequest } from '@/api/service/ourchat/session/kick/v1/kick'

// gRPC store
const grpcStore = useGrpcStore()

// Loading state
const loading = ref(false)

// Search
const showSearch = ref(false)
const searchForm = reactive({
  session_type: undefined as number | undefined,
  session_name: '',
  owner_id: '',
  created_after: undefined as number | undefined,
  created_before: undefined as number | undefined,
  min_members: undefined as number | undefined,
  max_members: undefined as number | undefined,
})

// Pagination
const pagination = reactive({
  page: 1,
  pageSize: 20,
})
const totalSessions = ref(0)

// Sessions list
const sessions = ref<GetSessionInfoResponse[]>([])

// Selected sessions for batch operations
const selectedSessions = ref<GetSessionInfoResponse[]>([])
const selectedSessionDetails = ref<GetSessionInfoResponse | null>(null)
const batchAction = ref('')
const batchLoading = ref(false)
const sessionDetailsDialogVisible = ref(false)

// Session statistics
const sessionStats = reactive({
  totalSessions: 0,
})

// Remove user dialog
const removeUserDialogVisible = ref(false)
const removeUserLoading = ref(false)
const selectedSession = ref<GetSessionInfoResponse | null>(null)
const removeUserForm = reactive({
  user_id: '',
})

// Helper functions
const formatDate = (timestamp: { seconds: bigint } | undefined): string => {
  if (!timestamp?.seconds) return '-'
  const date = new Date(Number(timestamp.seconds) * 1000)
  return date.toLocaleString()
}

// Note: getSessionTypeTag and getSessionTypeText removed since sessionType is no longer available

// Perform session action (single session, used for batch operations)
const performSessionAction = async (
  session: GetSessionInfoResponse,
  action: string,
  userId?: bigint,
) => {
  switch (action) {
    case 'delete':
      // Use OurChatService API for deletion (admin permission required)
      try {
        const ourchatRequest: OurChatDeleteSessionRequest = { sessionId: session.sessionId! }
        await grpcStore.ourchatConn.deleteSession(ourchatRequest)
        ElMessage.success(`Session "${session.name}" deleted successfully`)
      } catch (error) {
        console.error('Delete session failed:', error)
        ElMessage.error('Failed to delete session: admin permission required')
        throw error
      }
      break
    case 'removeUser':
      if (!userId) throw new Error('User ID required for removeUser action')
      // Use OurChatService API for kicking users (admin permission required)
      try {
        const kickRequest: KickUserRequest = { sessionId: session.sessionId!, userId }
        await grpcStore.ourchatConn.kickUser(kickRequest)
        ElMessage.success(`User removed from session "${session.name}" successfully`)
      } catch (error) {
        console.error('Kick user from session failed:', error)
        ElMessage.error('Failed to remove user from session: admin permission required')
        throw error
      }
      break
    default:
      throw new Error(`Unknown action: ${action}`)
  }
}

// Fetch sessions via gRPC
const fetchSessions = async () => {
  try {
    loading.value = true

    // Import QueryValues enum
    const { QueryValues } = await import(
      '@/api/service/ourchat/session/get_session_info/v1/get_session_info'
    )

    // Build request
    const request: ListSessionsRequest = {
      filter: {
        sessionType: searchForm.session_type,
        sessionName: searchForm.session_name || undefined,
        ownerId: searchForm.owner_id ? BigInt(searchForm.owner_id) : undefined,
        createdAfter: searchForm.created_after
          ? { seconds: BigInt(Math.floor(searchForm.created_after / 1000)), nanos: 0 }
          : undefined,
        createdBefore: searchForm.created_before
          ? { seconds: BigInt(Math.floor(searchForm.created_before / 1000)), nanos: 0 }
          : undefined,
        minMembers: searchForm.min_members,
        maxMembers: searchForm.max_members,
      },
      pagination: {
        page: pagination.page,
        pageSize: pagination.pageSize,
      },
      queryValues: [
        QueryValues.SESSION_ID,
        QueryValues.NAME,
        QueryValues.SIZE,
        QueryValues.CREATED_TIME,
        QueryValues.UPDATED_TIME,
      ],
    }

    // Try gRPC call
    const response = await grpcStore.serverManageConn.listSessions(request)
    sessions.value = response.response.sessions || []
    // Note: totalCount was removed from the response, calculate from returned sessions
    totalSessions.value = sessions.value.length

    // Update stats
    updateSessionStats()

    ElMessage.success('Sessions list updated successfully')
  } catch (error) {
    console.error('Fetch sessions error:', error)
    ElMessage.error('Failed to fetch sessions')
  } finally {
    loading.value = false
  }
}

// Update session statistics
const updateSessionStats = () => {
  // Statistics computed from currently displayed sessions (paginated)
  const displayedSessions = sessions.value
  sessionStats.totalSessions = displayedSessions.length
  // Note: sessionType is no longer available in GetSessionInfoResponse
  // Type-based statistics are not displayed
}

// Search handlers
const applySearch = () => {
  pagination.page = 1
  fetchSessions()
}

const resetSearch = () => {
  searchForm.session_type = undefined
  searchForm.session_name = ''
  searchForm.owner_id = ''
  searchForm.created_after = undefined
  searchForm.created_before = undefined
  searchForm.min_members = undefined
  searchForm.max_members = undefined
  pagination.page = 1
  fetchSessions()
}

// Pagination handlers
const handleSizeChange = (size: number) => {
  pagination.pageSize = size
  pagination.page = 1
  fetchSessions()
}

const handlePageChange = (page: number) => {
  pagination.page = page
  fetchSessions()
}

// Selection handlers
const handleSelectionChange = (selection: GetSessionInfoResponse[]) => {
  selectedSessions.value = selection
}

// Session action handlers
const viewSession = (session: GetSessionInfoResponse) => {
  selectedSessionDetails.value = session
  sessionDetailsDialogVisible.value = true
}

const removeUserFromSession = (session: GetSessionInfoResponse) => {
  selectedSession.value = session
  removeUserDialogVisible.value = true
}

const deleteSession = async (session: GetSessionInfoResponse) => {
  try {
    await ElMessageBox.confirm(
      `Are you sure to delete session "${session.name}"? This action cannot be undone.`,
      'Confirm Delete',
      { type: 'error' },
    )

    // Use OurChatService API for deletion (admin permission required)
    try {
      const ourchatRequest: OurChatDeleteSessionRequest = { sessionId: session.sessionId! }
      await grpcStore.ourchatConn.deleteSession(ourchatRequest)
      ElMessage.success(`Session "${session.name}" deleted successfully`)
      fetchSessions()
    } catch (error) {
      console.error('Delete session failed:', error)
      ElMessage.error('Failed to delete session: admin permission required')
    }
  } catch (error: unknown) {
    if (error !== 'cancel') {
      console.error('Delete session error:', error)
      ElMessage.error('Failed to delete session')
    }
  }
}

// Remove user dialog handlers
const closeRemoveUserDialog = () => {
  removeUserDialogVisible.value = false
  selectedSession.value = null
  removeUserForm.user_id = ''
}

const confirmRemoveUser = async () => {
  if (!selectedSession.value || !removeUserForm.user_id) return

  try {
    removeUserLoading.value = true
    const request: KickUserRequest = {
      sessionId: selectedSession.value.sessionId!,
      userId: BigInt(removeUserForm.user_id),
    }
    await grpcStore.ourchatConn.kickUser(request)
    ElMessage.success('User removed from session successfully')
    closeRemoveUserDialog()
    fetchSessions()
  } catch (error: unknown) {
    console.error('Remove user error:', error)
    ElMessage.error('Failed to remove user from session: admin permission required')
  } finally {
    removeUserLoading.value = false
  }
}

// Batch operations
const executeBatchAction = async () => {
  if (selectedSessions.value.length === 0 || !batchAction.value) return

  try {
    await ElMessageBox.confirm(
      `Are you sure to ${batchAction.value} ${selectedSessions.value.length} session(s)?`,
      `Confirm ${batchAction.value}`,
      { type: 'warning' },
    )

    batchLoading.value = true
    // Execute batch action by looping through selected sessions
    for (const session of selectedSessions.value) {
      await performSessionAction(session, batchAction.value)
    }
    ElMessage.success(
      `Batch ${batchAction.value} completed for ${selectedSessions.value.length} session(s)`,
    )

    // Clear selection and refresh
    selectedSessions.value = []
    batchAction.value = ''
    fetchSessions()
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
  selectedSessions.value = []
  batchAction.value = ''
}

// Refresh sessions (button handler)
const refreshSessions = () => {
  fetchSessions()
}

// Export sessions
const exportSessions = () => {
  try {
    const data = sessions.value
    if (data.length === 0) {
      ElMessage.warning('No sessions to export')
      return
    }

    // Define CSV headers
    const headers = [
      'Session ID',
      'Session Name',
      'Size',
      'Created At',
      'Updated At',
      'Description',
    ]

    // Convert data to rows
    const rows = data.map((session) => [
      session.sessionId?.toString() || '',
      session.name || '',
      session.size?.toString() || '',
      session.createdTime ? new Date(Number(session.createdTime.seconds) * 1000).toISOString() : '',
      session.updatedTime ? new Date(Number(session.updatedTime.seconds) * 1000).toISOString() : '',
      session.description || '',
    ])

    // Combine headers and rows
    const csvContent = [
      headers.join(','),
      ...rows.map((row) => row.map((field) => `"${String(field).replace(/"/g, '""')}"`).join(',')),
    ].join('\n')

    // Create download link
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    const url = URL.createObjectURL(blob)
    link.setAttribute('href', url)
    link.setAttribute('download', `sessions_export_${new Date().toISOString().split('T')[0]}.csv`)
    link.style.visibility = 'hidden'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    ElMessage.success(`Exported ${data.length} sessions successfully`)
  } catch (error) {
    console.error('Export sessions error:', error)
    ElMessage.error('Failed to export sessions')
  }
}

// Initialize on mount
onMounted(() => {
  // Load initial data
  updateSessionStats()
  fetchSessions()
})
</script>

<style scoped>
.sessions-view {
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
