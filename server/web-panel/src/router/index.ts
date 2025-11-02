import { createRouter, createWebHistory, type RouteLocationNormalized } from 'vue-router'
import Dashboard from '@/views/DashboardView.vue'

// Check if token exists
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

// Add route guarantee
router.beforeEach((to: RouteLocationNormalized, _from: RouteLocationNormalized) => {
  // Without token, redirect to Login page
  if (to.name !== 'Login' && !hasToken()) {
    return { name: 'Login' }
  }
})

export default router
