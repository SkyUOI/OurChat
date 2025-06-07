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
          <div class="mock-chart">
            <div class="chart-line" style="height: 30%"></div>
            <div class="chart-line" style="height: 70%"></div>
            <div class="chart-line" style="height: 45%"></div>
            <div class="chart-line" style="height: 60%"></div>
            <div class="chart-line" style="height: 85%"></div>
          </div>
          <div class="network-stats">
            <div>上传: 12.4 Mbps</div>
            <div>下载: 24.8 Mbps</div>
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
          <div class="mock-chart">
            <div class="chart-bar" style="height: 40%"></div>
            <div class="chart-bar" style="height: 75%"></div>
            <div class="chart-bar" style="height: 55%"></div>
            <div class="chart-bar" style="height: 90%"></div>
            <div class="chart-bar" style="height: 65%"></div>
          </div>
          <div class="io-stats">
            <div>读取: 120 IOPS</div>
            <div>写入: 85 IOPS</div>
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

<script>
export default {
  name: 'ResourceMonitor',
  data() {
    return {
      disks: [
        { name: '/dev/sda1', mount: '/', total: '120GB', used: '85GB', free: '35GB', usage: '71' },
        {
          name: '/dev/sdb1',
          mount: '/data',
          total: '500GB',
          used: '320GB',
          free: '180GB',
          usage: '64',
        },
        {
          name: '/dev/sdc1',
          mount: '/backup',
          total: '1TB',
          used: '250GB',
          free: '750GB',
          usage: '25',
        },
      ],
      processes: [
        {
          pid: '1234',
          name: 'nginx',
          user: 'www-data',
          cpu: '12.5',
          memory: '3.2',
          uptime: '12:34:56',
        },
        {
          pid: '5678',
          name: 'mysql',
          user: 'mysql',
          cpu: '8.2',
          memory: '15.7',
          uptime: '1 day 2:30',
        },
        { pid: '9012', name: 'redis', user: 'redis', cpu: '2.1', memory: '1.8', uptime: '5:20:10' },
        { pid: '3456', name: 'node', user: 'app', cpu: '4.5', memory: '8.3', uptime: '3:45:22' },
        { pid: '7890', name: 'python', user: 'app', cpu: '1.8', memory: '2.1', uptime: '2:15:33' },
      ],
    }
  },
  methods: {
    getUsageColor(percentage) {
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

.mock-chart {
  width: 100%;
  height: 180px;
  display: flex;
  align-items: flex-end;
  justify-content: space-around;
  padding: 0 20px;
}

.chart-line {
  width: 30px;
  background: linear-gradient(to top, #e6a23c, #f0c78a);
  border-radius: 4px 4px 0 0;
}

.chart-bar {
  width: 30px;
  background: linear-gradient(to top, #e6a23c, #f0c78a);
  border-radius: 4px 4px 0 0;
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
