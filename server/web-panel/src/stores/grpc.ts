import { defineStore } from 'pinia'
import { OurChatServiceClient } from '../api/service/ourchat/v1/OurchatServiceClientPb'
import { computed, ref, type Ref } from 'vue'

export const BackendIp: Ref<string> = ref("localhost")
export const BackendPort: Ref<number> = ref(7777)

export const useGrpcStore = defineStore('grpc', () => {
  const backendConn = computed(() => {
    return new OurChatServiceClient(BackendIp.value + ':' + BackendPort.value)
  })
  return { backendConn }
})
