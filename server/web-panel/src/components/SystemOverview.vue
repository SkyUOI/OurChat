<template>
  <div class="system-overview">
    <h2 class="section-title">{{ $t('systemOverview') }}</h2>

    <div class="stats-grid">
      <el-card v-for="stat in stats" :key="stat.titleKey" shadow="hover">
        <div class="stat-item">
          <div class="icon" :style="{ backgroundColor: stat.color }">
            <i :class="stat.icon"></i>
          </div>
          <div class="info">
            <div class="value">{{ stat.value }}</div>
            <div class="title">{{ $t(stat.titleKey) }}</div>
          </div>
        </div>
      </el-card>
    </div>

    <div class="charts-row">
      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('cpuUsage') }}</h3>
          <el-tag type="success">{{ $t('realtime') }}</el-tag>
        </div>
        <div class="chart-container">
          <!-- Chart will be placed here -->
          <div class="chart-placeholder">
            {{ cpuUsage !== null ? cpuUsage + '%' : $t('chartPlaceholder') }}
          </div>
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('memoryUsage') }}</h3>
          <el-tag type="info">{{ $t('last24Hours') }}</el-tag>
        </div>
        <div class="chart-container">
          <!-- Chart will be placed here -->
          <div class="chart-placeholder">
            {{ memoryUsage !== null ? memoryUsage + '%' : $t('chartPlaceholder') }}
          </div>
        </div>
      </el-card>
    </div>

    <div class="service-status">
      <h3>{{ $t('serviceStatus') }}</h3>
      <el-table :data="services" style="width: 100%">
        <el-table-column prop="name" :label="$t('serviceName')"></el-table-column>
        <el-table-column prop="status" :label="$t('status')">
          <template #default="scope">
            <el-tag :type="scope.row.statusKey === 'running' ? 'success' : 'danger'">
              {{ $t(scope.row.statusKey) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="uptime" :label="$t('uptime')"></el-table-column>
        <el-table-column prop="cpu" :label="$t('cpuUsage')"></el-table-column>
        <el-table-column prop="memory" :label="$t('memoryUsage')"></el-table-column>
        <el-table-column :label="$t('actions')" width="120">
          <template #default="scope">
            <el-button size="small" v-if="scope.row.statusKey === 'running'">{{
              $t('restart')
            }}</el-button>
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

export default {
  name: 'SystemOverview',
  setup() {
    const grpcStore = useGrpcStore()
    const metrics = ref<MonitoringMetrics | null>(null)
    const loading = ref(false)
    let refreshInterval: ReturnType<typeof setInterval> | null = null

    // Computed stats
    const stats = computed(() => [
      {
        titleKey: 'activeConnections',
        value: metrics.value?.activeConnections?.toString() || '0',
        icon: 'el-icon-connection',
        color: '#409EFF',
      },
      {
        titleKey: 'activeSessions',
        value: metrics.value?.activeSessions?.toString() || '0',
        icon: 'el-icon-chat-dot-round',
        color: '#67C23A',
      },
      {
        titleKey: 'totalUsers',
        value: metrics.value?.totalUsers?.toString() || '0',
        icon: 'el-icon-user',
        color: '#E6A23C',
      },
      {
        titleKey: 'messageRate',
        value: (metrics.value?.messagesPerSecond || 0) + ' msg/s',
        icon: 'el-icon-s-data',
        color: '#909399',
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
      services,
      loading,
    }
  },
}
</script>

<style scoped>
.section-title {
  margin-bottom: 20px;
  color: #303133;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 30px;
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
}

.charts-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 30px;
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

.service-status {
  margin-top: 20px;
}
</style>
