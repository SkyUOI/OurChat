<template>
  <div class="logs-view">
    <div class="page-header">
      <h1 class="page-title">{{ $t('logsView') }}</h1>
      <p class="page-description">{{ $t('logsViewDescription') }}</p>
    </div>

    <el-card class="logs-card card-hover" shadow="hover">
      <div class="card-header">
        <h3>{{ $t('systemLogs') }}</h3>
        <div class="actions">
          <el-select v-model="logLevel" :placeholder="$t('logLevel')" style="width: 120px">
            <el-option label="All" value="all" />
            <el-option label="Error" value="error" />
            <el-option label="Warning" value="warning" />
            <el-option label="Info" value="info" />
            <el-option label="Debug" value="debug" />
          </el-select>
          <el-button type="primary" :icon="Refresh" @click="refreshLogs" :loading="loading">
            {{ $t('refresh') }}
          </el-button>
        </div>
      </div>

      <el-table :data="logs" class="logs-table" stripe v-loading="loading">
        <el-table-column prop="timestamp" :label="$t('timestamp')" width="180" />
        <el-table-column prop="level" :label="$t('level')" width="100">
          <template #default="scope">
            <el-tag :type="getLogType(scope.row.level)" size="small">
              {{ scope.row.level }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="message" :label="$t('message')" min-width="300" />
        <el-table-column prop="source" :label="$t('source')" width="150" />
      </el-table>

      <div class="pagination-container">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[20, 50, 100]"
          :total="totalLogs"
          layout="sizes, prev, pager, next, total"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const loading = ref(false)
const logLevel = ref('all')

const logs = ref([
  {
    timestamp: '2024-01-15 10:30:45',
    level: 'INFO',
    message: 'Server started successfully',
    source: 'server',
  },
  {
    timestamp: '2024-01-15 10:30:46',
    level: 'INFO',
    message: 'Database connection established',
    source: 'database',
  },
  {
    timestamp: '2024-01-15 10:30:47',
    level: 'INFO',
    message: 'RabbitMQ connection established',
    source: 'rabbitmq',
  },
  {
    timestamp: '2024-01-15 10:31:00',
    level: 'WARNING',
    message: 'High memory usage detected: 75%',
    source: 'monitor',
  },
])

const pagination = reactive({
  page: 1,
  pageSize: 20,
})

const totalLogs = ref(4)

const getLogType = (level: string) => {
  switch (level.toUpperCase()) {
    case 'ERROR':
      return 'danger'
    case 'WARNING':
      return 'warning'
    case 'INFO':
      return 'info'
    case 'DEBUG':
      return 'success'
    default:
      return 'info'
  }
}

const refreshLogs = async () => {
  loading.value = true
  // Simulate API call
  setTimeout(() => {
    loading.value = false
    ElMessage.success(t('refreshSuccess'))
  }, 1000)
}

const handleSizeChange = (size: number) => {
  pagination.pageSize = size
  pagination.page = 1
}

const handlePageChange = (page: number) => {
  pagination.page = page
}

onMounted(() => {
  refreshLogs()
})
</script>

<style scoped>
.logs-view {
  animation: fadeInUp 0.4s ease-out;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  font-weight: 700;
  color: #1e293b;
  margin-bottom: 8px;
}

.page-description {
  font-size: 14px;
  color: #64748b;
}

.logs-card {
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.card-header h3 {
  font-size: 18px;
  font-weight: 600;
  color: #334155;
  margin: 0;
}

.actions {
  display: flex;
  gap: 12px;
}

.logs-table {
  border-radius: 12px;
  overflow: hidden;
  margin-bottom: 20px;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
}
</style>
