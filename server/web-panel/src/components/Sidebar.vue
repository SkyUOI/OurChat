<template>
  <div class="sidebar">
    <div class="logo">
      <h2>{{ $t('serverControlPanel') }}</h2>
    </div>
    <ul class="menu">
      <li v-for="item in menuItems" :key="item.id">
        <router-link :to="item.path">
          <i :class="item.icon"></i>
          <span>{{ $t(item.titleKey) }}</span>
        </router-link>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
export default {
  name: 'SidebarComponent',
  data() {
    return {
      menuItems: [
        { id: 1, titleKey: 'dashboard', path: '/', icon: 'el-icon-monitor' },
        { id: 2, titleKey: 'services', path: '/services', icon: 'el-icon-setting' },
        { id: 3, titleKey: 'monitor', path: '/monitor', icon: 'el-icon-data-line' },
        { id: 4, titleKey: 'logs', path: '/logs', icon: 'el-icon-document' },
        { id: 5, titleKey: 'users', path: '/users', icon: 'el-icon-user' },
        {
          id: 6,
          titleKey: 'sessionsManagement',
          path: '/sessions',
          icon: 'el-icon-chat-dot-round',
        },
        { id: 7, titleKey: 'config', path: '/config', icon: 'el-icon-files' },
        { id: 8, titleKey: 'announcements', path: '/announcements', icon: 'el-icon-bell' },
      ],
    }
  },
}
</script>

<style scoped>
.sidebar {
  width: 250px;
  height: 100%;
  background: linear-gradient(180deg, #1e293b 0%, #0f172a 100%);
  color: #e2e8f0;
  box-shadow: 4px 0 20px rgba(0, 0, 0, 0.15);
  z-index: 100;
  border-right: 1px solid rgba(148, 163, 184, 0.1);
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.sidebar.collapsed {
  width: 64px;
}

.logo {
  padding: 24px;
  text-align: center;
  border-bottom: 1px solid rgba(148, 163, 184, 0.15);
  transition: all 0.3s ease;
}

.logo h2 {
  font-size: 18px;
  font-weight: 700;
  background: linear-gradient(90deg, #60a5fa, #a78bfa, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  background-size: 200% 200%;
  animation: gradientShift 3s ease infinite;
}

.menu {
  list-style: none;
  padding: 16px 0;
  margin: 0;
}

.menu li {
  border-bottom: 1px solid rgba(148, 163, 184, 0.08);
}

.menu li:last-child {
  border-bottom: none;
}

.menu li a {
  display: flex;
  align-items: center;
  padding: 14px 24px;
  color: #cbd5e1;
  text-decoration: none;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.menu li a::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: linear-gradient(180deg, #3b82f6, #8b5cf6);
  transform: translateX(-100%);
  transition: transform 0.3s ease;
}

.menu li a:hover {
  background-color: rgba(51, 65, 85, 0.5);
  color: #f1f5f9;
  padding-left: 32px;
}

.menu li a:hover::before {
  transform: translateX(0);
}

.menu li a.router-link-active {
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.15), rgba(139, 92, 246, 0.15));
  color: #f1f5f9;
  box-shadow: inset 0 0 20px rgba(59, 130, 246, 0.1);
}

.menu li a.router-link-active::before {
  transform: translateX(0);
}

.menu li a i {
  margin-right: 12px;
  font-size: 18px;
  transition: transform 0.3s ease;
}

.menu li a:hover i {
  transform: scale(1.1);
}

.menu li a span {
  font-size: 14px;
  font-weight: 500;
}
</style>
