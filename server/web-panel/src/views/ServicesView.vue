<template>
  <div class="services-view">
    <div class="page-header">
      <h1 class="page-title">{{ $t('servicesManagement') }}</h1>
      <p class="page-description">{{ $t('servicesManagementDescription') }}</p>
    </div>

    <el-card class="services-card card-hover" shadow="hover">
      <div class="card-header">
        <h3>{{ $t('serviceStatus') }}</h3>
        <el-button type="primary" :icon="Refresh" @click="refreshServices" :loading="loading">
          {{ $t('refresh') }}
        </el-button>
      </div>

      <el-table :data="services" class="service-table" stripe v-loading="loading">
        <el-table-column prop="name" :label="$t('serviceName')" />
        <el-table-column prop="status" :label="$t('status')" width="120">
          <template #default="scope">
            <el-tag :type="scope.row.status === 'running' ? 'success' : 'danger'">
              {{ $t(scope.row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="uptime" :label="$t('uptime')" />
        <el-table-column :label="$t('actions')" width="200">
          <template #default="scope">
            <el-button
              size="small"
              v-if="scope.row.status === 'running'"
              @click="restartService(scope.row)"
            >
              {{ $t('restart') }}
            </el-button>
            <el-button size="small" type="success" v-else @click="startService(scope.row)">
              {{ $t('start') }}
            </el-button>
            <el-button size="small" type="danger" @click="stopService(scope.row)">
              {{ $t('stop') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const loading = ref(false)

const services = ref([
  { name: 'OurChat Server', status: 'running', uptime: '5d 12h' },
  { name: 'PostgreSQL', status: 'running', uptime: '5d 12h' },
  { name: 'RabbitMQ', status: 'running', uptime: '5d 12h' },
])

const refreshServices = async () => {
  loading.value = true
  // Simulate API call
  setTimeout(() => {
    loading.value = false
    ElMessage.success(t('refreshSuccess'))
  }, 1000)
}

const restartService = (service: any) => {
  ElMessage.info(`${t('restarting')} ${service.name}...`)
}

const startService = (service: any) => {
  ElMessage.info(`${t('starting')} ${service.name}...`)
}

const stopService = (service: any) => {
  ElMessage.info(`${t('stopping')} ${service.name}...`)
}

onMounted(() => {
  refreshServices()
})
</script>

<style scoped>
.services-view {
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

.services-card {
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

.service-table {
  border-radius: 12px;
  overflow: hidden;
}
</style>
