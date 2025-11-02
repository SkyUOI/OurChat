import { defineStore } from 'pinia'
import { OurChatServiceClient } from '../api/service/ourchat/v1/ourchat.client'
import { BasicServiceClient } from '../api/service/basic/v1/basic.client'
import { ServerManageServiceClient } from '../api/service/server_manage/v1/server_manage.client'
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport'

export const useGrpcStore = defineStore('grpc', () => {
  const transport = new GrpcWebFetchTransport({
    baseUrl: `/backend`,
  })
  const ourchatConn = new OurChatServiceClient(transport)
  const basicConn = new BasicServiceClient(transport)
  const serverManageConn = new ServerManageServiceClient(transport)

  return { ourchatConn, basicConn, serverManageConn }
})
