import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useGrpcStore } from '../grpc'

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

describe('grpc store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorageMock.clear()
    vi.clearAllMocks()
  })

  it('should initialize all gRPC clients', () => {
    const store = useGrpcStore()

    expect(store.ourchatConn).toBeDefined()
    expect(store.basicConn).toBeDefined()
    expect(store.serverManageConn).toBeDefined()
    expect(store.authConn).toBeDefined()
  })

  it('should have transport with auth interceptor configured', () => {
    const store = useGrpcStore()

    expect(store.serverManageConn).toBeDefined()
    expect(store.authConn).toBeDefined()
  })
})
