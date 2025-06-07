<template>
  <div class="config-manager">
    <h2 class="section-title">配置管理</h2>

    <div class="config-tabs">
      <el-tabs v-model="activeTab" type="card">
        <el-tab-pane label="服务器配置" name="server">
          <div class="config-editor">
            <el-input
              type="textarea"
              :autosize="{ minRows: 15 }"
              v-model="serverConfig"
              placeholder="服务器配置内容"
            ></el-input>
          </div>
        </el-tab-pane>

        <el-tab-pane label="应用配置" name="app">
          <div class="config-editor">
            <el-input
              type="textarea"
              :autosize="{ minRows: 15 }"
              v-model="appConfig"
              placeholder="应用配置内容"
            ></el-input>
          </div>
        </el-tab-pane>

        <el-tab-pane label="数据库配置" name="database">
          <div class="config-editor">
            <el-input
              type="textarea"
              :autosize="{ minRows: 15 }"
              v-model="dbConfig"
              placeholder="数据库配置内容"
            ></el-input>
          </div>
        </el-tab-pane>
      </el-tabs>

      <div class="actions">
        <el-button type="primary" icon="el-icon-download">保存配置</el-button>
        <el-button type="success" icon="el-icon-refresh">重新加载</el-button>
        <el-button type="warning" icon="el-icon-view">查看历史</el-button>
      </div>
    </div>

    <div class="config-history">
      <h3>配置历史记录</h3>
      <el-table :data="history" style="width: 100%">
        <el-table-column prop="time" label="时间" width="180"></el-table-column>
        <el-table-column prop="user" label="操作人" width="120"></el-table-column>
        <el-table-column prop="type" label="配置类型" width="120"></el-table-column>
        <el-table-column prop="description" label="描述"></el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="scope">
            <el-button size="small">查看</el-button>
            <el-button size="small" type="danger">回滚</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script>
export default {
  name: 'ConfigManager',
  data() {
    return {
      activeTab: 'server',
      serverConfig: '',
      appConfig: '',
      dbConfig: '',
      history: [
        {
          time: '2025-06-05 14:30:22',
          user: 'admin',
          type: '服务器配置',
          description: '修改了端口设置',
        },
        {
          time: '2025-06-04 09:15:47',
          user: 'admin',
          type: '应用配置',
          description: '更新了API地址',
        },
        {
          time: '2025-06-03 16:20:33',
          user: 'admin',
          type: '数据库配置',
          description: '调整了连接池大小',
        },
        {
          time: '2025-06-02 11:05:18',
          user: 'admin',
          type: '服务器配置',
          description: '添加了SSL配置',
        },
      ],
    }
  },
}
</script>

<style scoped>
.config-manager {
  padding: 20px;
}

.section-title {
  margin-bottom: 20px;
  color: #303133;
}

.config-tabs {
  margin-bottom: 30px;
}

.config-editor {
  margin-top: 15px;
}

.actions {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.config-history {
  margin-top: 30px;
}
</style>
