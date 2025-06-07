import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import Dashboard from '@/views/Dashboard.vue'

// 检查token是否存在
const hasToken = (): boolean => {
  return !!localStorage.getItem('token')
}

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
  },
  {
    path: '/',
    name: 'Dashboard',
    component: Dashboard,
  },
  {
    path: '/services',
    name: 'Services',
    component: () => import('@/views/Services.vue'),
  },
  {
    path: '/monitor',
    name: 'Monitor',
    component: () => import('@/views/Monitor.vue'),
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('@/views/Logs.vue'),
  },
  {
    path: '/users',
    name: 'Users',
    component: () => import('@/views/Users.vue'),
  },
  {
    path: '/config',
    name: 'Config',
    component: () => import('@/views/Config.vue'),
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
