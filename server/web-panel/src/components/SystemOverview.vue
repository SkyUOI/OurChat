<template>
  <div class="system-overview">
    <h2 class="section-title">{{ $t('systemOverview') }}</h2>

    <div class="stats-grid">
      <div
        v-for="stat in stats"
        :key="stat.titleKey"
        class="stat-card card-hover"
        :class="`card-${stat.color}`"
      >
        <div class="icon">
          <i :class="stat.icon"></i>
        </div>
        <div class="info">
          <div class="value">{{ stat.value }}</div>
          <div class="title">{{ $t(stat.titleKey) }}</div>
        </div>
      </div>
    </div>

    <div class="charts-row">
      <el-card class="chart-card card-hover" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('cpuUsage') }}</h3>
          <el-tag type="success" effect="dark">{{ $t('realtime') }}</el-tag>
        </div>
        <div class="chart-container">
          <v-chart
            v-if="cpuUsage !== null"
            :option="cpuChartOption"
            autoresize
            class="echarts-instance"
          />
          <div v-else class="chart-placeholder">{{ $t('chartPlaceholder') }}</div>
        </div>
      </el-card>

      <el-card class="chart-card card-hover" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('memoryUsage') }}</h3>
          <el-tag type="info">{{ $t('last24Hours') }}</el-tag>
        </div>
        <div class="chart-container">
          <v-chart
            v-if="memoryUsage !== null"
            :option="memoryChartOption"
            autoresize
            class="echarts-instance"
          />
          <div v-else class="chart-placeholder">{{ $t('chartPlaceholder') }}</div>
        </div>
      </el-card>
    </div>

    <div class="service-status">
      <h3>{{ $t('serviceStatus') }}</h3>
      <el-table :data="services" class="service-table" stripe>
        <el-table-column prop="name" :label="$t('serviceName')" />
        <el-table-column prop="status" :label="$t('status')">
          <template #default="scope">
            <el-tag :type="scope.row.statusKey === 'running' ? 'success' : 'danger'">
              {{ $t(scope.row.statusKey) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="uptime" :label="$t('uptime')" />
        <el-table-column prop="cpu" :label="$t('cpuUsage')" />
        <el-table-column prop="memory" :label="$t('memoryUsage')" />
        <el-table-column :label="$t('actions')" width="120">
          <template #default="scope">
            <el-button size="small" v-if="scope.row.statusKey === 'running'">
              {{ $t('restart') }}
            </el-button>
            <el-button size="small" type="success" v-else>{{ $t('start') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useGrpcStore } from '../stores/grpc'
import type { MonitoringMetrics } from '../api/service/server_manage/monitoring/v1/monitoring'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { GaugeChart, LineChart } from 'echarts/charts'
import { CanvasRenderer } from 'echarts/renderers'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
} from 'echarts/components'

// Register ECharts components
use([
  CanvasRenderer,
  GaugeChart,
  LineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
])

export default {
  name: 'SystemOverview',
  components: {
    VChart,
  },
  setup() {
    const grpcStore = useGrpcStore()
    const metrics = ref<MonitoringMetrics | null>(null)
    const loading = ref(false)
    let refreshInterval: ReturnType<typeof setInterval> | null = null

    // Computed stats with color classes
    const stats = computed(() => [
      {
        titleKey: 'activeConnections',
        value: metrics.value?.activeConnections?.toString() || '0',
        icon: 'el-icon-connection',
        color: 'blue',
      },
      {
        titleKey: 'activeSessions',
        value: metrics.value?.activeSessions?.toString() || '0',
        icon: 'el-icon-chat-dot-round',
        color: 'green',
      },
      {
        titleKey: 'totalUsers',
        value: metrics.value?.totalUsers?.toString() || '0',
        icon: 'el-icon-user',
        color: 'orange',
      },
      {
        titleKey: 'messageRate',
        value: (metrics.value?.messagesPerSecond || 0) + ' msg/s',
        icon: 'el-icon-s-data',
        color: 'purple',
      },
    ])

    // CPU and memory usage
    const cpuUsage = computed(() => {
      if (!metrics.value?.cpuUsagePercent) return null
      return Number(metrics.value.cpuUsagePercent).toFixed(1)
    })

    const memoryUsage = computed(() => {
      if (!metrics.value?.memoryUsagePercent) return null
      return Number(metrics.value.memoryUsagePercent).toFixed(1)
    })

    // CPU Gauge Chart Option
    const cpuChartOption = computed(() => ({
      series: [
        {
          type: 'gauge',
          startAngle: 180,
          endAngle: 0,
          min: 0,
          max: 100,
          splitNumber: 8,
          axisLine: {
            lineStyle: {
              width: 16,
              color: [
                [0.25, '#3b82f6'],
                [0.5, '#8b5cf6'],
                [0.75, '#f59e0b'],
                [1, '#ef4444'],
              ],
            },
          },
          pointer: {
            icon: 'path://M12.8,0.7l12,40.1H0.7L12.8,0.7z',
            length: '12%',
            width: 20,
            offsetCenter: [0, '-60%'],
            itemStyle: { color: 'auto' },
          },
          axisTick: {
            length: 8,
            lineStyle: { color: 'auto', width: 2 },
          },
          splitLine: {
            length: 16,
            lineStyle: { color: 'auto', width: 4 },
          },
          axisLabel: {
            color: '#64748b',
            fontSize: 12,
            distance: -50,
          },
          title: {
            offsetCenter: [0, '-15%'],
            fontSize: 16,
            color: '#64748b',
          },
          detail: {
            fontSize: 36,
            offsetCenter: [0, '10%'],
            valueAnimation: true,
            formatter: '{value}%',
            color: '#1e293b',
          },
          data: [{ value: cpuUsage.value ? parseFloat(cpuUsage.value) : 0, name: 'CPU' }],
        },
      ],
    }))

    // Memory Line Chart Option
    const memoryChartOption = computed(() => ({
      tooltip: {
        trigger: 'axis',
        formatter: '{b}<br />Memory: {c}%',
      },
      grid: {
        left: '5%',
        right: '5%',
        bottom: '10%',
        top: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['00:00', '04:00', '08:00', '12:00', '16:00', '20:00', 'Now'],
        axisLine: { lineStyle: { color: '#cbd5e1' } },
        axisLabel: { color: '#64748b' },
      },
      yAxis: {
        type: 'value',
        max: 100,
        axisLabel: {
          formatter: '{value}%',
          color: '#64748b',
        },
        splitLine: { lineStyle: { color: '#e2e8f0' } },
      },
      series: [
        {
          name: 'Memory',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          lineStyle: {
            width: 3,
            color: '#8b5cf6',
          },
          itemStyle: {
            color: '#8b5cf6',
          },
          areaStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 0,
              y2: 1,
              colorStops: [
                { offset: 0, color: 'rgba(139, 92, 246, 0.4)' },
                { offset: 1, color: 'rgba(139, 92, 246, 0)' },
              ],
            },
          },
          data: [30, 35, 32, 40, 45, 42, memoryUsage.value ? parseFloat(memoryUsage.value) : 0],
        },
      ],
    }))

    // Services list
    const services = computed(() => [
      {
        name: 'OurChat Server',
        statusKey: metrics.value ? 'running' : 'stopped',
        uptime: formatUptime(metrics.value?.uptimeSeconds),
        cpu: cpuUsage.value !== null ? cpuUsage.value + '%' : '-',
        memory: memoryUsage.value !== null ? memoryUsage.value + '%' : '-',
      },
      {
        name: 'PostgreSQL',
        statusKey: (metrics.value?.databaseConnections || 0) > 0 ? 'running' : 'stopped',
        uptime: '-',
        cpu: '-',
        memory: '-',
      },
      {
        name: 'RabbitMQ',
        statusKey: (metrics.value?.rabbitmqConnections || 0) > 0 ? 'running' : 'stopped',
        uptime: '-',
        cpu: '-',
        memory: '-',
      },
    ])

    const formatUptime = (seconds: bigint | undefined): string => {
      if (!seconds) return '-'
      const secs = Number(seconds)
      const days = Math.floor(secs / 86400)
      const hours = Math.floor((secs % 86400) / 3600)
      const mins = Math.floor((secs % 3600) / 60)
      if (days > 0) return `${days}d ${hours}h`
      if (hours > 0) return `${hours}h ${mins}m`
      return `${mins}m`
    }

    const fetchMetrics = async () => {
      try {
        loading.value = true
        const response = await grpcStore.serverManageConn.getMonitoringMetrics({
          includeSystemMetrics: true,
          includeTokioMetrics: false,
        })
        metrics.value = response.response.metrics || null
      } catch (error) {
        console.error('Failed to fetch metrics:', error)
      } finally {
        loading.value = false
      }
    }

    onMounted(() => {
      fetchMetrics()
      refreshInterval = setInterval(fetchMetrics, 30000)
    })

    onUnmounted(() => {
      if (refreshInterval) {
        clearInterval(refreshInterval)
      }
    })

    return {
      stats,
      cpuUsage,
      memoryUsage,
      cpuChartOption,
      memoryChartOption,
      services,
      loading,
    }
  },
}
</script>

<style scoped>
.system-overview {
  animation: fadeInUp 0.4s ease-out;
}

.section-title {
  margin-bottom: 24px;
  font-size: 24px;
  font-weight: 700;
  color: #1e293b;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
  margin-bottom: 32px;
}

@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

.stat-card {
  background: white;
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  border: 1px solid #e2e8f0;
  transition:
    transform 0.2s ease-out,
    box-shadow 0.2s ease-out;
}

.stat-card.card-blue {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  border: none;
  color: white;
}

.stat-card.card-green {
  background: linear-gradient(135deg, #10b981, #059669);
  border: none;
  color: white;
}

.stat-card.card-orange {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  border: none;
  color: white;
}

.stat-card.card-purple {
  background: linear-gradient(135deg, #8b5cf6, #7c3aed);
  border: none;
  color: white;
}

.stat-card .icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(8px);
}

.stat-card .icon i {
  font-size: 24px;
  color: white;
}

.info .value {
  font-size: 24px;
  font-weight: 700;
  color: inherit;
}

.info .title {
  font-size: 13px;
  font-weight: 500;
  opacity: 0.9;
  margin-top: 2px;
}

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  margin-bottom: 32px;
}

@media (max-width: 768px) {
  .charts-row {
    grid-template-columns: 1fr;
  }
}

.chart-card {
  height: 320px;
  border: 1px solid #e2e8f0;
  border-radius: 16px;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.card-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: #334155;
}

.chart-container {
  height: calc(100% - 48px);
  display: flex;
  align-items: center;
}

.echarts-instance {
  width: 100%;
  height: 100%;
}

.chart-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  font-size: 14px;
}

.service-status {
  margin-top: 24px;
}

.service-status h3 {
  font-size: 18px;
  font-weight: 600;
  color: #334155;
  margin-bottom: 16px;
}

.service-table {
  border-radius: 12px;
  overflow: hidden;
}
</style>
