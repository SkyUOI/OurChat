import { defineStore } from 'pinia'
import { OurChatServiceClient } from '../api/service/ourchat/v1/OurchatServiceClientPb'
import { computed, type Ref } from 'vue'

export var BackendIp: null | Ref<string> = null
export var BackendPort: null | Ref<string> = null

export const useGrpcStore = defineStore('grpc', () => {
  let backendConn = computed(() => {
    if (BackendIp == null || BackendPort == null) return null
    return new OurChatServiceClient(BackendIp.value + ':' + BackendPort.value)
  })
  return { backendConn }
})
