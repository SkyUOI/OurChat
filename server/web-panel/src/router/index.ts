import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import Dashboard from '@/views/DashboardView.vue'

// 检查token是否存在
const hasToken = (): boolean => {
  return !!localStorage.getItem('token')
}

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/LoginView.vue'),
  },
  {
    path: '/',
    name: 'Dashboard',
    component: Dashboard,
  },
  {
    path: '/services',
    name: 'Services',
    component: () => import('@/views/ServicesView.vue'),
  },
  {
    path: '/monitor',
    name: 'Monitor',
    component: () => import('@/views/MonitorView.vue'),
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('@/views/LogsView.vue'),
  },
  {
    path: '/users',
    name: 'Users',
    component: () => import('@/views/UsersView.vue'),
  },
  {
    path: '/config',
    name: 'Config',
    component: () => import('@/views/ConfigView.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

// 添加路由守卫
router.beforeEach((to: RouteLocationNormalized, from: RouteLocationNormalized) => {
  // 如果目标路由不是登录页且没有token，则重定向到登录页
  if (to.name !== 'Login' && !hasToken()) {
    return { name: 'Login' }
  }
})

export default router
