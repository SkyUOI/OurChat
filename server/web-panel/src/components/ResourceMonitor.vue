<template>
  <div class="resource-monitor">
    <h2 class="section-title">资源监控</h2>

    <div class="charts-row">
      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>网络流量</h3>
          <el-tag type="primary">实时</el-tag>
        </div>
        <div class="chart-container">
          <!-- 这里放置图表 -->
          <div class="chart-placeholder">
            图表数据将在此显示
          </div>
        </div>
      </el-card>

      <el-card class="chart-card" shadow="hover">
        <div class="card-header">
          <h3>磁盘I/O</h3>
          <el-tag type="warning">实时</el-tag>
        </div>
        <div class="chart-container">
          <!-- 这里放置图表 -->
          <div class="chart-placeholder">
            图表数据将在此显示
          </div>
        </div>
      </el-card>
    </div>

    <div class="disk-usage">
      <h3>磁盘使用情况</h3>
      <el-table :data="disks" style="width: 100%">
        <el-table-column prop="name" label="磁盘"></el-table-column>
        <el-table-column prop="mount" label="挂载点"></el-table-column>
        <el-table-column prop="total" label="总容量"></el-table-column>
        <el-table-column prop="used" label="已用"></el-table-column>
        <el-table-column prop="free" label="可用"></el-table-column>
        <el-table-column prop="usage" label="使用率">
          <template #default="scope">
            <el-progress
              :percentage="parseInt(scope.row.usage)"
              :color="getUsageColor(parseInt(scope.row.usage))"
            ></el-progress>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <div class="process-list">
      <h3>进程列表</h3>
      <el-table :data="processes" style="width: 100%">
        <el-table-column prop="pid" label="PID" width="80"></el-table-column>
        <el-table-column prop="name" label="进程名"></el-table-column>
        <el-table-column prop="user" label="用户" width="100"></el-table-column>
        <el-table-column prop="cpu" label="CPU%" width="100"></el-table-column>
        <el-table-column prop="memory" label="内存%" width="100"></el-table-column>
        <el-table-column prop="uptime" label="运行时间"></el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script lang="ts">
export default {
  name: 'ResourceMonitor',
  data() {
    return {
      disks: [] as Array<{name: string, mount: string, total: string, used: string, free: string, usage: string}>,
      processes: [] as Array<{pid: string, name: string, user: string, cpu: string, memory: string, uptime: string}>,
    }
  },
  methods: {
    getUsageColor(percentage: number) {
      if (percentage < 70) return '#67C23A'
      if (percentage < 90) return '#E6A23C'
      return '#F56C6C'
    },
  },
}
</script>

<style scoped>
.resource-monitor {
  padding: 20px;
}

.section-title {
  margin-bottom: 20px;
  color: #303133;
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
  flex-direction: column;
  align-items: center;
}

.chart-placeholder {
  width: 100%;
  height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #909399;
  font-size: 14px;
}




.network-stats,
.io-stats {
  display: flex;
  justify-content: space-around;
  width: 100%;
  margin-top: 15px;
  font-size: 16px;
}

.disk-usage,
.process-list {
  margin-top: 30px;
}
</style>
