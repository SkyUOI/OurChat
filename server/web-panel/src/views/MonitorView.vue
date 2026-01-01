<template>
  <div class="monitor-view">
    <div class="header">
      <h1 class="text-2xl font-bold">{{ $t('resourceMonitoring') }}</h1>
      <div class="actions">
        <el-button type="primary" icon="el-icon-refresh" @click="refreshMetrics" :loading="loading">
          {{ $t('refresh') }}
        </el-button>
        <el-button icon="el-icon-download" @click="exportMetrics">
          {{ $t('export') }}
        </el-button>
      </div>
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

    <!-- Key Metrics Grid -->
    <div class="stats-grid">
      <el-card v-for="stat in keyStats" :key="stat.title" shadow="hover" class="metric-card">
        <div class="stat-item">
          <div class="icon" :style="{ backgroundColor: stat.color }">
            <i :class="stat.icon"></i>
          </div>
          <div class="info">
            <div class="value">{{ stat.value }}</div>
            <div class="title">{{ stat.title }}</div>
            <div class="trend" v-if="stat.trend">
              <i :class="stat.trend.icon" :style="{ color: stat.trend.color }"></i>
              <span :style="{ color: stat.trend.color }">{{ stat.trend.value }}</span>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <!-- Charts Row -->
    <div class="charts-row">
      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('connectionsChart') }}</h3>
          <el-tag type="primary">{{ $t('lastHour') }}</el-tag>
        </div>
        <div class="chart-container">
          <div class="chart-placeholder">
            {{ $t('chartPlaceholder') }}
          </div>
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('messageThroughput') }}</h3>
          <el-tag type="success">{{ $t('messagesPerSecond') }}</el-tag>
        </div>
        <div class="chart-container">
          <div class="chart-placeholder">
            {{ $t('chartPlaceholder') }}
          </div>
        </div>
      </el-card>
    </div>

    <!-- Detailed Metrics Table -->
    <el-card class="detailed-metrics" shadow="hover">
      <div class="card-header">
        <h3>{{ $t('detailedMetrics') }}</h3>
        <el-switch v-model="showSystemMetrics" active-text="Show System Metrics"></el-switch>
      </div>
      <el-table :data="detailedMetrics" style="width: 100%">
        <el-table-column prop="name" :label="$t('metricName')" width="200"></el-table-column>
        <el-table-column prop="value" :label="$t('value')" width="150"></el-table-column>
        <el-table-column prop="unit" :label="$t('unit')" width="100"></el-table-column>
        <el-table-column prop="description" :label="$t('description')"></el-table-column>
        <el-table-column :label="$t('status')" width="120">
          <template #default="scope">
            <el-tag
              :type="
                scope.row.status === 'healthy'
                  ? 'success'
                  : scope.row.status === 'warning'
                    ? 'warning'
                    : 'danger'
              "
            >
              {{ scope.row.status }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Service Status -->
    <div class="service-status">
      <h3>{{ $t('serviceStatus') }}</h3>
      <el-table :data="services" style="width: 100%">
        <el-table-column prop="name" :label="$t('serviceName')"></el-table-column>
        <el-table-column prop="status" :label="$t('status')">
          <template #default="scope">
            <el-tag :type="scope.row.status === 'running' ? 'success' : 'danger'">
              {{ scope.row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="connections" :label="$t('connections')"></el-table-column>
        <el-table-column prop="uptime" :label="$t('uptime')"></el-table-column>
        <el-table-column prop="cpu" :label="$t('cpuUsage')"></el-table-column>
        <el-table-column prop="memory" :label="$t('memoryUsage')"></el-table-column>
        <el-table-column :label="$t('actions')" width="120">
          <template #default="scope">
            <el-button size="small" v-if="scope.row.status === 'running'">{{
              $t('restart')
            }}</el-button>
            <el-button size="small" type="success" v-else>{{ $t('start') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import type { MonitoringMetrics } from '@/api/service/server_manage/monitoring/v1/monitoring'

// Reactive state
const loading = ref(false)
const connectionStatus = ref<'success' | 'failed' | null>(null)
const showSystemMetrics = ref(true)
const metrics = reactive<Partial<MonitoringMetrics>>({
  activeConnections: 0,
  totalUsers: 0,
  messagesPerSecond: 0,
  uptimeSeconds: BigInt(0),
  timestamp: BigInt(Math.floor(Date.now() / 1000)),
  cpuUsagePercent: 0,
  memoryUsagePercent: 0,
  diskUsagePercent: 0,
  totalSessions: 0,
  activeSessions: 0,
  databaseConnections: 0,
  redisConnections: 0,
  rabbitmqConnections: 0,
})

// gRPC store
const grpcStore = useGrpcStore()

// Computed key stats for top grid
const keyStats = computed(() => [
  {
    title: 'Active Connections',
    value: metrics.activeConnections || 0,
    icon: 'el-icon-connection',
    color: '#409EFF',
    trend: { icon: 'el-icon-top', value: '+5', color: '#67C23A' },
  },
  {
    title: 'Total Users',
    value: metrics.totalUsers || 0,
    icon: 'el-icon-user',
    color: '#67C23A',
    trend: { icon: 'el-icon-top', value: '+12', color: '#67C23A' },
  },
  {
    title: 'Message Rate',
    value: `${(metrics.messagesPerSecond || 0).toFixed(1)}/s`,
    icon: 'el-icon-chat-line-round',
    color: '#E6A23C',
    trend: { icon: 'el-icon-bottom', value: '-2.3', color: '#F56C6C' },
  },
  {
    title: 'Server Uptime',
    value: formatUptime(Number(metrics.uptimeSeconds || 0)),
    icon: 'el-icon-time',
    color: '#909399',
    trend: null,
  },
  {
    title: 'CPU Usage',
    value: `${(metrics.cpuUsagePercent || 0).toFixed(1)}%`,
    icon: 'el-icon-cpu',
    color: '#F56C6C',
    trend: { icon: 'el-icon-top', value: '+3.2%', color: '#F56C6C' },
  },
  {
    title: 'Memory Usage',
    value: `${(metrics.memoryUsagePercent || 0).toFixed(1)}%`,
    icon: 'el-icon-memory',
    color: '#8E44AD',
    trend: { icon: 'el-icon-bottom', value: '-1.5%', color: '#67C23A' },
  },
  {
    title: 'Database Connections',
    value: metrics.databaseConnections || 0,
    icon: 'el-icon-database',
    color: '#3498DB',
    trend: { icon: 'el-icon-top', value: '+2', color: '#67C23A' },
  },
  {
    title: 'Redis Connections',
    value: metrics.redisConnections || 0,
    icon: 'el-icon-data-board',
    color: '#E74C3C',
    trend: { icon: 'el-icon-bottom', value: '-1', color: '#F56C6C' },
  },
])

// Computed detailed metrics for table
const detailedMetrics = computed(() => [
  {
    name: 'Active Sessions',
    value: metrics.activeSessions || 0,
    unit: 'count',
    description: 'Currently connected user sessions',
    status: 'healthy',
  },
  {
    name: 'Total Sessions',
    value: metrics.totalSessions || 0,
    unit: 'count',
    description: 'Total sessions created',
    status: 'healthy',
  },
  {
    name: 'RabbitMQ Connections',
    value: metrics.rabbitmqConnections || 0,
    unit: 'count',
    description: 'Active RabbitMQ connections',
    status: 'healthy',
  },
  {
    name: 'Disk Usage',
    value: `${(metrics.diskUsagePercent || 0).toFixed(1)}%`,
    unit: 'percent',
    description: 'Storage disk usage',
    status: (metrics.diskUsagePercent || 0) > 90 ? 'warning' : 'healthy',
  },
  {
    name: 'Server Timestamp',
    value: new Date(Number(metrics.timestamp || 0) * 1000).toLocaleString(),
    unit: 'datetime',
    description: 'Time when metrics were collected',
    status: 'healthy',
  },
])

// Helper functions
const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

// Services data will be populated from server
const services = ref([])

// Fetch metrics via gRPC
const fetchMetrics = async () => {
  try {
    loading.value = true
    connectionStatus.value = null

    try {
      const response = await grpcStore.serverManageConn.getMonitoringMetrics({
        includeSystemMetrics: showSystemMetrics.value,
      })

      const receivedMetrics = response.response.metrics
      // Update metrics object
      Object.assign(metrics, receivedMetrics)
      connectionStatus.value = 'success'
      ElMessage.success('Metrics updated successfully')
    } catch (error: unknown) {
      console.error('gRPC error:', error)
      connectionStatus.value = 'failed'
      ElMessage.error('Failed to fetch metrics: server API not available')
    }
  } catch (error) {
    console.error('Fetch metrics error:', error)
    ElMessage.error('Failed to fetch metrics')
  } finally {
    loading.value = false
  }
}

// Refresh metrics (called by button)
const refreshMetrics = () => {
  fetchMetrics()
}

// Export metrics (mock)
const exportMetrics = () => {
  ElMessage.info('Export feature not yet implemented')
}

// Initialize on mount
onMounted(() => {
  fetchMetrics()
})
</script>

<style scoped>
.monitor-view {
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

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 30px;
}

@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
}

.metric-card {
  transition: transform 0.2s;
}

.metric-card:hover {
  transform: translateY(-2px);
}

.stat-item {
  display: flex;
  align-items: center;
}

.icon {
  width: 50px;
  height: 50px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 15px;
}

.icon i {
  font-size: 24px;
  color: white;
}

.info .value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.info .title {
  font-size: 14px;
  color: #909399;
  margin-top: 2px;
}

.info .trend {
  font-size: 12px;
  margin-top: 4px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 30px;
}

@media (max-width: 768px) {
  .charts-row {
    grid-template-columns: 1fr;
  }
}

.chart-card {
  height: 300px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.chart-container {
  height: calc(100% - 40px);
  display: flex;
  align-items: center;
}

.chart-placeholder {
  width: 100%;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #909399;
  font-size: 14px;
}

.detailed-metrics {
  margin-bottom: 30px;
}

.service-status {
  margin-top: 20px;
}
</style>
