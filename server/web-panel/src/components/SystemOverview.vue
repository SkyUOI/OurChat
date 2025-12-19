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
          <div class="chart-placeholder">
            {{ $t('chartPlaceholder') }}
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
          <div class="chart-placeholder">
            {{ $t('chartPlaceholder') }}
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
      stats: [] as Array<{ titleKey: string; value: string; icon: string; color: string }>,
      services: [] as Array<{
        name: string
        statusKey: string
        uptime: string
        cpu: string
        memory: string
      }>,
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
