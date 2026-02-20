<template>
  <div class="monitor-view">
    <div class="header">
      <h1 class="text-2xl font-bold">{{ $t('resourceMonitoring') }}</h1>
      <div class="actions">
        <el-select
          v-model="timeRange"
          style="width: 150px; margin-right: 10px"
          @change="fetchHistoricalMetrics"
        >
          <el-option :label="$t('lastHour')" :value="3600" />
          <el-option :label="$t('last6Hours')" :value="21600" />
          <el-option :label="$t('last24Hours')" :value="86400" />
          <el-option :label="$t('last7Days')" :value="604800" />
        </el-select>
        <el-button type="primary" icon="Refresh" @click="refreshMetrics" :loading="loading">
          {{ $t('refresh') }}
        </el-button>
        <el-button icon="Download" @click="exportMetrics">
          {{ $t('export') }}
        </el-button>
      </div>
    </div>

    <div v-if="connectionStatus" class="mb-4">
      <el-alert
        :title="connectionStatus === 'success' ? $t('connectionSuccess') : $t('connectionFailed')"
        :type="connectionStatus === 'success' ? 'success' : 'error'"
        :closable="false"
        show-icon
      />
    </div>

    <div class="stats-grid">
      <el-card v-for="stat in keyStats" :key="stat.title" shadow="hover" class="metric-card">
        <div class="stat-item">
          <div class="icon" :style="{ backgroundColor: stat.color }">
            <el-icon :size="24"><component :is="stat.icon" /></el-icon>
          </div>
          <div class="info">
            <div class="value">{{ stat.value }}</div>
            <div class="title">{{ stat.title }}</div>
          </div>
        </div>
      </el-card>
    </div>

    <div class="charts-row">
      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('connectionsChart') }}</h3>
        </div>
        <div class="chart-container">
          <v-chart :option="connectionsChartOption" autoresize />
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('messageThroughput') }}</h3>
        </div>
        <div class="chart-container">
          <v-chart :option="throughputChartOption" autoresize />
        </div>
      </el-card>
    </div>

    <div class="charts-row">
      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('cpuUsage') }}</h3>
        </div>
        <div class="chart-container">
          <v-chart :option="cpuChartOption" autoresize />
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('memoryUsage') }}</h3>
        </div>
        <div class="chart-container">
          <v-chart :option="memoryChartOption" autoresize />
        </div>
      </el-card>
    </div>

    <el-card class="detailed-metrics" shadow="hover">
      <div class="card-header">
        <h3>{{ $t('detailedMetrics') }}</h3>
        <el-switch
          v-model="showSystemMetrics"
          :active-text="$t('showSystemMetrics')"
          @change="fetchMetrics"
        />
      </div>
      <el-table :data="detailedMetrics" style="width: 100%">
        <el-table-column prop="name" :label="$t('metricName')" width="200" />
        <el-table-column prop="value" :label="$t('value')" width="150" />
        <el-table-column prop="unit" :label="$t('unit')" width="100" />
        <el-table-column prop="description" :label="$t('description')" />
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
              {{ $t(scope.row.status) }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import { useI18n } from 'vue-i18n'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import { Connection, User, ChatLineRound, Timer, Cpu, Memo, Coin } from '@element-plus/icons-vue'
import type {
  MonitoringMetrics,
  MetricDataPoint,
} from '@/api/service/server_manage/monitoring/v1/monitoring'
import { Timestamp } from '@/api/google/protobuf/timestamp'
import { Duration } from '@/api/google/protobuf/duration'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, LegendComponent])

const { t } = useI18n()
const loading = ref(false)
const connectionStatus = ref<'success' | 'failed' | null>(null)
const showSystemMetrics = ref(true)
const timeRange = ref(3600)
const metrics = reactive<Partial<MonitoringMetrics>>({
  activeConnections: 0,
  totalUsers: 0,
  messagesPerSecond: 0,
  uptimeSeconds: BigInt(0),
  timestamp: BigInt(0),
  cpuUsagePercent: 0,
  memoryUsagePercent: 0,
  diskUsagePercent: 0,
  totalSessions: 0,
  activeSessions: 0,
  databaseConnections: 0,
  rabbitmqConnections: 0,
})

const historicalData = ref<MetricDataPoint[]>([])
let refreshInterval: ReturnType<typeof setInterval> | null = null

const grpcStore = useGrpcStore()

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const keyStats = computed(() => [
  {
    title: t('activeConnections'),
    value: metrics.activeConnections || 0,
    icon: Connection,
    color: '#409EFF',
  },
  {
    title: t('totalUsers'),
    value: metrics.totalUsers || 0,
    icon: User,
    color: '#67C23A',
  },
  {
    title: t('messageRate'),
    value: `${(metrics.messagesPerSecond || 0).toFixed(1)}/s`,
    icon: ChatLineRound,
    color: '#E6A23C',
  },
  {
    title: t('serverUptime'),
    value: formatUptime(Number(metrics.uptimeSeconds || 0)),
    icon: Timer,
    color: '#909399',
  },
  {
    title: t('cpuUsage'),
    value: `${(metrics.cpuUsagePercent || 0).toFixed(1)}%`,
    icon: Cpu,
    color: '#F56C6C',
  },
  {
    title: t('memoryUsage'),
    value: `${(metrics.memoryUsagePercent || 0).toFixed(1)}%`,
    icon: Memo,
    color: '#8E44AD',
  },
  {
    title: t('dbConnections'),
    value: metrics.databaseConnections || 0,
    icon: Coin,
    color: '#3498DB',
  },
  {
    title: t('rabbitmqConnections'),
    value: metrics.rabbitmqConnections || 0,
    icon: Coin,
    color: '#E74C3C',
  },
])

const detailedMetrics = computed(() => [
  {
    name: t('activeSessions'),
    value: metrics.activeSessions || 0,
    unit: 'count',
    description: t('activeSessionsDesc'),
    status: 'healthy',
  },
  {
    name: t('totalSessions'),
    value: metrics.totalSessions || 0,
    unit: 'count',
    description: t('totalSessionsDesc'),
    status: 'healthy',
  },
  {
    name: t('rabbitmqConnections'),
    value: metrics.rabbitmqConnections || 0,
    unit: 'count',
    description: t('rabbitmqConnectionsDesc'),
    status: 'healthy',
  },
  {
    name: t('diskUsage'),
    value: `${(metrics.diskUsagePercent || 0).toFixed(1)}%`,
    unit: 'percent',
    description: t('diskUsageDesc'),
    status: (metrics.diskUsagePercent || 0) > 90 ? 'warning' : 'healthy',
  },
  {
    name: t('serverTimestamp'),
    value: new Date(Number(metrics.timestamp || 0) * 1000).toLocaleString(),
    unit: 'datetime',
    description: t('serverTimestampDesc'),
    status: 'healthy',
  },
])

const formatTimestamp = (ts?: { seconds: bigint; nanos: number }): string => {
  if (!ts) return ''
  return new Date(Number(ts.seconds) * 1000).toLocaleTimeString()
}

const connectionsChartOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  legend: { data: [t('activeConnections'), t('activeSessions')] },
  xAxis: {
    type: 'category',
    data: historicalData.value.map((d) => formatTimestamp(d.timestamp)),
  },
  yAxis: { type: 'value' },
  series: [
    {
      name: t('activeConnections'),
      type: 'line' as const,
      smooth: true,
      data: historicalData.value.map((d) => d.metrics?.activeConnections || 0),
    },
    {
      name: t('activeSessions'),
      type: 'line' as const,
      smooth: true,
      data: historicalData.value.map((d) => d.metrics?.activeSessions || 0),
    },
  ],
}))

const throughputChartOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  legend: { data: [t('messagesPerSecond')] },
  xAxis: {
    type: 'category',
    data: historicalData.value.map((d) => formatTimestamp(d.timestamp)),
  },
  yAxis: { type: 'value' },
  series: [
    {
      name: t('messagesPerSecond'),
      type: 'line' as const,
      smooth: true,
      areaStyle: { opacity: 0.3 },
      data: historicalData.value.map((d) => d.metrics?.messagesPerSecond || 0),
    },
  ],
}))

const cpuChartOption = computed(() => ({
  tooltip: { trigger: 'axis', formatter: '{b}<br/>{a}: {c}%' },
  legend: { data: [t('cpuUsage')] },
  xAxis: {
    type: 'category',
    data: historicalData.value.map((d) => formatTimestamp(d.timestamp)),
  },
  yAxis: { type: 'value', max: 100 },
  series: [
    {
      name: t('cpuUsage'),
      type: 'line' as const,
      smooth: true,
      itemStyle: { color: '#F56C6C' },
      areaStyle: { opacity: 0.3, color: '#F56C6C' },
      data: historicalData.value.map((d) =>
        d.metrics?.cpuUsagePercent !== undefined ? d.metrics.cpuUsagePercent.toFixed(1) : 0,
      ),
    },
  ],
}))

const memoryChartOption = computed(() => ({
  tooltip: { trigger: 'axis', formatter: '{b}<br/>{a}: {c}%' },
  legend: { data: [t('memoryUsage')] },
  xAxis: {
    type: 'category',
    data: historicalData.value.map((d) => formatTimestamp(d.timestamp)),
  },
  yAxis: { type: 'value', max: 100 },
  series: [
    {
      name: t('memoryUsage'),
      type: 'line' as const,
      smooth: true,
      itemStyle: { color: '#8E44AD' },
      areaStyle: { opacity: 0.3, color: '#8E44AD' },
      data: historicalData.value.map((d) =>
        d.metrics?.memoryUsagePercent !== undefined ? d.metrics.memoryUsagePercent.toFixed(1) : 0,
      ),
    },
  ],
}))

const fetchMetrics = async () => {
  try {
    loading.value = true
    connectionStatus.value = null

    const response = await grpcStore.serverManageConn.getMonitoringMetrics({
      includeSystemMetrics: showSystemMetrics.value,
      includeTokioMetrics: false,
    })

    const receivedMetrics = response.response.metrics
    if (receivedMetrics) {
      Object.assign(metrics, receivedMetrics)
      connectionStatus.value = 'success'
    }
  } catch (error: unknown) {
    console.error('gRPC error:', error)
    connectionStatus.value = 'failed'
    ElMessage.error(t('fetchMetricsFailed'))
  } finally {
    loading.value = false
  }
}

const fetchHistoricalMetrics = async () => {
  try {
    const now = Date.now()
    const endTime = Timestamp.fromDate(new Date(now))
    const startTime = Timestamp.fromDate(new Date(now - timeRange.value * 1000))
    const intervalSeconds = timeRange.value <= 3600 ? 60 : timeRange.value <= 86400 ? 300 : 3600
    const interval = Duration.create({ seconds: BigInt(intervalSeconds), nanos: 0 })

    const response = await grpcStore.serverManageConn.getHistoricalMetrics({
      startTime,
      endTime,
      interval,
    })

    historicalData.value = response.response.dataPoints
  } catch (error: unknown) {
    console.error('Failed to fetch historical metrics:', error)
  }
}

const refreshMetrics = async () => {
  await Promise.all([fetchMetrics(), fetchHistoricalMetrics()])
}

const exportMetrics = () => {
  const data = JSON.stringify(metrics, (_, v) => (typeof v === 'bigint' ? v.toString() : v), 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `metrics-${new Date().toISOString()}.json`
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success(t('exportSuccess'))
}

onMounted(() => {
  refreshMetrics()
  refreshInterval = setInterval(refreshMetrics, 30000)
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
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
  align-items: center;
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
  height: 350px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.card-header h3 {
  margin: 0;
  font-size: 16px;
  color: #303133;
}

.chart-container {
  height: calc(100% - 40px);
}

.detailed-metrics {
  margin-bottom: 30px;
}
</style>
