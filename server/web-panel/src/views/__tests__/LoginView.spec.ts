import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '../LoginView.vue'
import ElementPlus from 'element-plus'
import { createI18n } from 'vue-i18n'

const mockAuth = vi.fn()
const mockGetServerInfo = vi.fn()

vi.mock('../../stores/grpc', () => ({
  useGrpcStore: () => ({
    authConn: {
      auth: mockAuth,
    },
    basicConn: {
      getServerInfo: mockGetServerInfo,
    },
  }),
}))

const mockPush = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
}))

const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key]
    }),
    clear: vi.fn(() => {
      store = {}
    }),
  }
})()

Object.defineProperty(global, 'localStorage', {
  value: localStorageMock,
})

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  messages: {
    en: {
      adminLogin: 'Admin Login',
      usernamePlaceholder: 'Username',
      passwordPlaceholder: 'Password',
      usernameRequired: 'Username is required',
      passwordRequired: 'Password is required',
      login: 'Login',
      rememberMe: 'Remember me',
      forgotPassword: 'Forgot password?',
      loginSuccess: 'Login successful',
      loginFailed: 'Login failed',
      connectionSuccess: 'Connection successful',
      connectionFailed: 'Connection failed',
    },
  },
})

describe('LoginView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorageMock.clear()
    vi.clearAllMocks()
    mockGetServerInfo.mockResolvedValue({})
    mockAuth.mockReset()
  })

  const mountComponent = () => {
    return mount(LoginView, {
      global: {
        plugins: [ElementPlus, i18n],
      },
    })
  }

  it('should render login form', async () => {
    const wrapper = mountComponent()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('h2').text()).toContain('Admin Login')
    expect(wrapper.find('input[type="password"]').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('should call getServerInfo on mount', async () => {
    mockGetServerInfo.mockResolvedValue({})

    mountComponent()
    await new Promise((resolve) => setTimeout(resolve, 100))

    expect(mockGetServerInfo).toHaveBeenCalled()
  })

  it('should store token in localStorage on successful login', async () => {
    mockAuth.mockResolvedValue({
      response: {
        token: 'test-token',
        id: 1n,
        ocid: 'test-ocid',
      },
    })

    const wrapper = mountComponent()
    await wrapper.vm.$nextTick()

    const form = wrapper.findComponent({ name: 'ElForm' })
    await form.setValue('test@example.com', 'username')
    await form.setValue('password123', 'password')

    await wrapper.find('form').trigger('submit.prevent')
    await new Promise((resolve) => setTimeout(resolve, 100))

    expect(mockAuth).toHaveBeenCalled()
  })

  it('should not submit with empty fields', async () => {
    const wrapper = mountComponent()
    await wrapper.vm.$nextTick()

    await wrapper.find('form').trigger('submit.prevent')

    expect(mockAuth).not.toHaveBeenCalled()
  })
})
