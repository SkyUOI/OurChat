<template>
  <div class="announcements-view p-5">
    <div class="header mb-6">
      <h1 class="text-2xl font-bold">{{ $t('announcements') }}</h1>
    </div>

    <!-- Publish Announcement Form -->
    <el-card shadow="hover" class="mb-6">
      <h3 class="text-lg font-semibold mb-4">{{ $t('publishAnnouncement') }}</h3>
      <el-form :model="announcementForm" label-width="120px">
        <el-form-item :label="$t('announcementTitle')">
          <el-input
            v-model="announcementForm.title"
            :placeholder="$t('announcementTitlePlaceholder')"
            clearable
          ></el-input>
        </el-form-item>
        <el-form-item :label="$t('announcementContent')">
          <el-input
            v-model="announcementForm.content"
            type="textarea"
            :rows="5"
            :placeholder="$t('announcementContentPlaceholder')"
          ></el-input>
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            @click="publishAnnouncement"
            :loading="publishing"
            icon="el-icon-bell"
          >
            {{ $t('publish') }}
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- Announcements List -->
    <el-card shadow="hover">
      <template #header>
        <div class="flex justify-between items-center">
          <h3 class="text-lg font-semibold">{{ $t('announcements') }}</h3>
        </div>
      </template>
      <el-empty
        v-if="announcements.length === 0"
        :description="$t('announcements') + ': None'"
      ></el-empty>
      <div v-else>
        <el-card
          v-for="item in announcements"
          :key="String(item.id)"
          shadow="hover"
          class="mb-4 announcement-item"
        >
          <div class="announcement-header">
            <span class="announcement-title">{{ item.title }}</span>
            <span class="announcement-time">{{ formatTime(item.createdAt) }}</span>
          </div>
          <div class="announcement-content">{{ item.content }}</div>
        </el-card>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'
import { FetchMsgsRequest } from '@/api/service/ourchat/msg_delivery/v1/msg_delivery'
import { Timestamp } from '@/api/google/protobuf/timestamp'

// gRPC store
const grpcStore = useGrpcStore()

// States
const publishing = ref(false)

// Announcement form
const announcementForm = reactive({
  title: '',
  content: '',
})

// Announcements list
interface AnnouncementItem {
  id: bigint
  title: string
  content: string
  createdAt?: { seconds?: bigint; nanos?: number }
  publisherId?: bigint
}

const announcements = ref<AnnouncementItem[]>([])

// Format timestamp
const formatTime = (timestamp?: { seconds?: bigint; nanos?: number }) => {
  if (!timestamp?.seconds) return '-'
  const date = new Date(Number(timestamp.seconds) * 1000)
  return date.toLocaleString()
}

// Fetch announcements
const fetchAnnouncements = async () => {
  try {
    // Create request with announcementOnly = true, fetch from epoch (all announcements)
    const request = FetchMsgsRequest.create({
      time: Timestamp.create({ seconds: BigInt(0), nanos: 0 }),
      announcementOnly: true,
    })

    const call = grpcStore.ourchatConn.fetchMsgs(request)

    // Consume the server streaming response
    for await (const response of call.responses) {
      if (response.respondEventType.oneofKind === 'announcementResponse') {
        const ann = response.respondEventType.announcementResponse
        announcements.value.unshift({
          id: ann.id,
          title: ann.announcement?.title || '',
          content: ann.announcement?.content || '',
          createdAt: ann.createdAt,
          publisherId: ann.announcement?.publisherId,
        })
      }
    }
  } catch (error: unknown) {
    console.error('Fetch announcements error:', error)
    ElMessage.error('Failed to fetch announcements')
  }
}

// Publish announcement
const publishAnnouncement = async () => {
  if (!announcementForm.title || !announcementForm.content) {
    ElMessage.warning('Please fill in both title and content')
    return
  }

  publishing.value = true
  try {
    const request: Record<string, unknown> = {
      announcement: {
        title: announcementForm.title,
        content: announcementForm.content,
        publisherId: BigInt(localStorage.getItem('userId') || '0'),
      },
    }
    await grpcStore.serverManageConn.publishAnnouncement(request as never)

    ElMessage.success('Announcement published successfully')

    // Clear form
    announcementForm.title = ''
    announcementForm.content = ''
  } catch (error: unknown) {
    console.error('Publish announcement error:', error)
    ElMessage.error('Failed to publish announcement')
  } finally {
    publishing.value = false
  }
}

onMounted(() => {
  fetchAnnouncements()
})
</script>

<style scoped>
.announcements-view {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.announcement-item {
  margin-bottom: 12px;
}

.announcement-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.announcement-title {
  font-weight: bold;
  font-size: 16px;
}

.announcement-time {
  color: #909399;
  font-size: 12px;
}

.announcement-content {
  color: #606266;
  white-space: pre-wrap;
}
</style>
