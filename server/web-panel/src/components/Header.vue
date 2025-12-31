<template>
  <div class="header">
    <div class="left">
      <el-icon class="collapse-icon" @click="toggleSidebar">
        <expand v-if="sidebarCollapsed" />
        <fold v-else />
      </el-icon>
      <el-breadcrumb separator="/">
        <el-breadcrumb-item :to="{ path: '/' }">{{ $t('dashboard') }}</el-breadcrumb-item>
        <el-breadcrumb-item v-for="item in breadcrumb" :key="item.path">
          {{ $t(item.titleKey) }}
        </el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <div class="right">
      <el-dropdown trigger="click" @command="changeLanguage">
        <el-button type="text" class="language-switch">
          {{ currentLanguage === 'zh' ? $t('chinese') : 'English' }}
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="zh">{{ $t('chinese') }}</el-dropdown-item>
            <el-dropdown-item command="en">English</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-dropdown trigger="click">
        <div class="user-info">
          <el-avatar
            size="small"
            src="https://cube.elemecdn.com/3/7c/3ea6beec64369c2642b92c6726f1epng.png"
          ></el-avatar>
          <span class="username">{{ $t('admin') }}</span>
        </div>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item><i class="el-icon-user"></i>{{ $t('profile') }}</el-dropdown-item>
            <el-dropdown-item><i class="el-icon-setting"></i>{{ $t('settings') }}</el-dropdown-item>
            <el-dropdown-item divided
              ><i class="el-icon-switch-button"></i>{{ $t('logout') }}</el-dropdown-item
            >
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
  </div>
</template>

<script lang="ts">
import { Fold, Expand } from '@element-plus/icons-vue'

export default {
  name: 'HeaderComponent',
  components: {
    Fold,
    Expand,
  },
  data() {
    return {
      sidebarCollapsed: false,
      breadcrumb: [{ titleKey: 'systemOverview', path: '/' }],
      currentLanguage: 'en', // Default language
    }
  },
  methods: {
    toggleSidebar() {
      this.sidebarCollapsed = !this.sidebarCollapsed
      this.$emit('toggle-sidebar', this.sidebarCollapsed)
    },
    changeLanguage(lang: string) {
      this.currentLanguage = lang
      this.$i18n.locale = lang
    },
  },
}
</script>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  height: 60px;
  background-color: #fff;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
  z-index: 99;
}

.left {
  display: flex;
  align-items: center;
}

.collapse-icon {
  font-size: 20px;
  margin-right: 20px;
  cursor: pointer;
  color: #606266;
}

.right {
  display: flex;
  align-items: center;
}

.user-info {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.username {
  margin-left: 8px;
  font-size: 14px;
  color: #606266;
}
</style>
