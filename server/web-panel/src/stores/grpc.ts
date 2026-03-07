import { defineStore } from 'pinia'
import { OurChatServiceClient } from '../api/service/ourchat/v1/ourchat.client'
import { BasicServiceClient } from '../api/service/basic/v1/basic.client'
import { ServerManageServiceClient } from '../api/service/server_manage/v1/server_manage.client'
import { AuthServiceClient } from '../api/service/auth/v1/auth.client'
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport'
import type { RpcInterceptor } from '@protobuf-ts/runtime-rpc'

const getToken = (): string | null => localStorage.getItem('token')

const authInterceptor: RpcInterceptor = {
  interceptUnary(next, method, input, options) {
    const token = getToken()
    if (token) {
      options = {
        ...options,
        meta: {
          ...options.meta,
          Authorization: `Bearer ${token}`,
        },
      }
    }
    return next(method, input, options)
  },
  interceptServerStreaming(next, method, input, options) {
    const token = getToken()
    if (token) {
      options = {
        ...options,
        meta: {
          ...options.meta,
          Authorization: `Bearer ${token}`,
        },
      }
    }
    return next(method, input, options)
  },
}
export const useGrpcStore = defineStore('grpc', () => {
  const transport = new GrpcWebFetchTransport({
    baseUrl: `/backend`,
    interceptors: [authInterceptor],
  })
  const ourchatConn = new OurChatServiceClient(transport)
  const basicConn = new BasicServiceClient(transport)
  const serverManageConn = new ServerManageServiceClient(transport)
  const authConn = new AuthServiceClient(transport)
  return {
    ourchatConn,
    basicConn,
    serverManageConn,
    authConn,
  }
})
