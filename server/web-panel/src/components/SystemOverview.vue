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
          <!-- 这里放置图表 -->
          <div class="mock-chart">
            <div class="chart-bar" style="height: 30%"></div>
            <div class="chart-bar" style="height: 70%"></div>
            <div class="chart-bar" style="height: 45%"></div>
            <div class="chart-bar" style="height: 60%"></div>
            <div class="chart-bar" style="height: 85%"></div>
          </div>
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>{{ $t('memoryUsage') }}</h3>
          <el-tag type="info">{{ $t('last24Hours') }}</el-tag>
        </div>
        <div class="chart-container">
          <!-- 这里放置图表 -->
          <div class="mock-chart pie">
            <div class="pie-slice" style="--value: 65"></div>
          </div>
          <div class="memory-stats">
            <div>{{ $t('used') }}: 8.2GB <span class="usage">(65%)</span></div>
            <div>{{ $t('free') }}: 4.4GB <span class="free">(35%)</span></div>
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
export default {
  name: 'SystemOverview',
  data() {
    return {
      stats: [
        { titleKey: 'onlineUsers', value: '128', icon: 'el-icon-user', color: '#409EFF' },
        { titleKey: 'serviceCount', value: '12', icon: 'el-icon-cpu', color: '#67C23A' },
        { titleKey: 'alerts', value: '3', icon: 'el-icon-warning', color: '#E6A23C' },
        { titleKey: 'diskUsage', value: '78%', icon: 'el-icon-hard-drive', color: '#F56C6C' },
      ],
      services: [
        {
          name: 'Web服务器',
          statusKey: 'running',
          uptime: '12天 3小时',
          cpu: '12%',
          memory: '256MB',
        },
        {
          name: '数据库服务',
          statusKey: 'running',
          uptime: '8天 7小时',
          cpu: '8%',
          memory: '512MB',
        },
        { name: '缓存服务', statusKey: 'running', uptime: '5天 2小时', cpu: '5%', memory: '128MB' },
        { name: '消息队列', statusKey: 'stopped', uptime: '-', cpu: '0%', memory: '0MB' },
        { name: '日志服务', statusKey: 'running', uptime: '2天 9小时', cpu: '3%', memory: '64MB' },
      ],
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

.mock-chart {
  width: 100%;
  height: 200px;
  display: flex;
  align-items: flex-end;
  justify-content: space-around;
  padding: 0 20px;
}

.mock-chart.pie {
  align-items: center;
  justify-content: center;
  position: relative;
}

.pie-slice {
  width: 150px;
  height: 150px;
  border-radius: 50%;
  background: conic-gradient(#67c23a calc(var(--value) * 1%), #ebeef5 0);
}

.chart-bar {
  width: 30px;
  background: linear-gradient(to top, #409eff, #79bbff);
  border-radius: 4px 4px 0 0;
}

.memory-stats {
  margin-left: 30px;
  font-size: 16px;
}

.memory-stats div {
  margin-bottom: 10px;
}

.usage {
  color: #67c23a;
  font-weight: bold;
}

.free {
  color: #909399;
}

.service-status {
  margin-top: 20px;
}
</style>
