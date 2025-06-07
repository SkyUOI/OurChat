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

<script>
export default {
  name: 'Sidebar',
  data() {
    return {
      menuItems: [
        { id: 1, titleKey: 'dashboard', path: '/', icon: 'el-icon-monitor' },
        { id: 2, titleKey: 'services', path: '/services', icon: 'el-icon-setting' },
        { id: 3, titleKey: 'monitor', path: '/monitor', icon: 'el-icon-data-line' },
        { id: 4, titleKey: 'logs', path: '/logs', icon: 'el-icon-document' },
        { id: 5, titleKey: 'users', path: '/users', icon: 'el-icon-user' },
        { id: 6, titleKey: 'config', path: '/config', icon: 'el-icon-files' },
      ],
    }
  },
}
</script>

<style scoped>
.sidebar {
  width: 250px;
  height: 100%;
  background-color: #2c3e50;
  color: #ecf0f1;
  box-shadow: 2px 0 5px rgba(0, 0, 0, 0.1);
  z-index: 100;
}

.logo {
  padding: 20px;
  text-align: center;
  border-bottom: 1px solid #34495e;
}

.menu {
  list-style: none;
  padding: 0;
  margin: 0;
}

.menu li {
  border-bottom: 1px solid #34495e;
}

.menu li a {
  display: flex;
  align-items: center;
  padding: 15px 20px;
  color: #ecf0f1;
  text-decoration: none;
  transition: background 0.3s;
}

.menu li a:hover {
  background-color: #34495e;
}

.menu li a i {
  margin-right: 10px;
  font-size: 18px;
}

.menu li a span {
  font-size: 16px;
}
</style>
