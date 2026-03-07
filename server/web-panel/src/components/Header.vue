<template>
  <div class="header">
    <div class="left">
      <el-icon class="collapse-icon" @click="toggleSidebar">
        <expand v-if="sidebarCollapsed" />
        <fold v-else />
      </el-icon>
      <el-breadcrumb separator="/">
        <el-breadcrumb-item :to="{ path: '/' }">{{ $t('dashboard') }}</el-breadcrumb-item>
        <el-breadcrumb-item v-for="item in breadcrumb" :key="item.path">
          {{ $t(item.titleKey) }}
        </el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <div class="right">
      <el-tag :type="serverStatus === 1 ? 'success' : 'danger'" class="mr-3" effect="dark">
        {{ serverStatus === 1 ? 'Online' : 'Maintenance' }}
      </el-tag>
      <el-dropdown
        trigger="click"
        @command="(cmd: string) => setServerStatus(cmd === 'normal' ? 1 : 2)"
      >
        <el-button type="text" class="language-switch">
          {{ $t('serverStatus') }}
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="normal" :disabled="serverStatus === 1">
              <el-tag type="success" size="small">Normal</el-tag>
            </el-dropdown-item>
            <el-dropdown-item command="maintain" :disabled="serverStatus === 2">
              <el-tag type="danger" size="small">Maintenance</el-tag>
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-dropdown trigger="click" @command="changeLanguage">
        <el-button type="text" class="language-switch">
          {{ currentLanguage === 'zh' ? $t('chinese') : 'English' }}
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="zh">{{ $t('chinese') }}</el-dropdown-item>
            <el-dropdown-item command="en">English</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <el-dropdown trigger="click" @command="handleUserMenu">
        <div class="user-info">
          <el-avatar
            size="small"
            src="https://cube.elemecdn.com/3/7c/3ea6beec64369c2642b92c6726f1epng.png"
          ></el-avatar>
          <span class="username">{{ userName || $t('admin') }}</span>
        </div>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="profile"
              ><i class="el-icon-user"></i>{{ $t('profile') }}</el-dropdown-item
            >
            <el-dropdown-item command="settings"
              ><i class="el-icon-setting"></i>{{ $t('settings') }}</el-dropdown-item
            >
            <el-dropdown-item divided @click="handleLogout"
              ><i class="el-icon-switch-button"></i>{{ $t('logout') }}</el-dropdown-item
            >
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>

    <el-dialog v-model="profileDialogVisible" title="Profile" width="500px">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="Username">{{ userName || '-' }}</el-descriptions-item>
        <el-descriptions-item label="Email">{{ userEmail || '-' }}</el-descriptions-item>
        <el-descriptions-item label="OCID">{{ userOcid || '-' }}</el-descriptions-item>
        <el-descriptions-item label="Roles">
          <el-tag v-for="role in userRoles" :key="role.id" type="primary" class="mr-2">
            {{ role.name }}
          </el-tag>
          <span v-if="userRoles.length === 0">-</span>
        </el-descriptions-item>
        <el-descriptions-item label="Permissions">
          <el-tag v-for="perm in userPermissions" :key="perm.id" type="success" class="mr-2 mb-2">
            {{ perm.name }}
          </el-tag>
          <span v-if="userPermissions.length === 0">-</span>
        </el-descriptions-item>
      </el-descriptions>
      <template #footer>
        <el-button @click="profileDialogVisible = false">{{ $t('close') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script lang="ts">
import { Fold, Expand } from '@element-plus/icons-vue'
import * as getAccountInfoPb from '../api/service/ourchat/get_account_info/v1/get_account_info'
import { ElMessage } from 'element-plus'
import { useGrpcStore } from '../stores/grpc'
import type { ServerStatus } from '../api/service/server_manage/set_server_status/v1/set_server_status'

export default {
  name: 'HeaderComponent',
  components: {
    Fold,
    Expand,
  },
  data() {
    return {
      sidebarCollapsed: false,
      breadcrumb: [{ titleKey: 'systemOverview', path: '/' }],
      currentLanguage: 'en',
      userName: '',
      userEmail: '',
      userOcid: '',
      userRoles: [] as { id: number; name: string; description: string }[],
      userPermissions: [] as { id: number; name: string; description: string }[],
      profileDialogVisible: false,
      serverStatus: 1 as number, // 1 = NORMAL, 2 = MAINTAINING
      serverStatusLoading: false,
    }
  },
  async mounted() {
    await this.fetchUserInfo()
  },
  methods: {
    toggleSidebar() {
      this.sidebarCollapsed = !this.sidebarCollapsed
      this.$emit('toggle-sidebar', this.sidebarCollapsed)
    },
    changeLanguage(lang: string) {
      this.currentLanguage = lang
      this.$i18n.locale = lang
    },
    handleUserMenu(command: string) {
      if (command === 'profile') {
        this.showProfile()
      } else if (command === 'settings') {
        this.$router.push('/settings')
      }
    },
    handleLogout() {
      localStorage.removeItem('token')
      localStorage.removeItem('userId')
      localStorage.removeItem('userOcid')
      this.$router.push('/login')
    },
    async showProfile() {
      this.profileDialogVisible = true
      await this.fetchProfileData()
    },
    async fetchUserInfo() {
      const ocid = localStorage.getItem('userOcid')
      if (!ocid) {
        return
      }

      try {
        const grpcStore = useGrpcStore()
        const request = getAccountInfoPb.GetAccountInfoRequest.create({
          requestValues: [getAccountInfoPb.QueryValues.USER_NAME],
        })
        const response = await grpcStore.ourchatConn.getAccountInfo(request)
        this.userName = response.response.userName || ''
        this.userEmail = response.response.email || ''
        this.userOcid = ocid
      } catch (error) {
        console.error('Failed to fetch user info:', error)
      }
    },
    async fetchProfileData() {
      const userId = localStorage.getItem('userId')
      if (!userId) {
        return
      }

      try {
        const grpcStore = useGrpcStore()

        const roleIdsRes = await grpcStore.serverManageConn.listUserServerRoles({
          userId: BigInt(userId),
        })
        const roleIds = roleIdsRes.response.roleIds
        console.log('User role IDs:', roleIds)

        const rolesRes = await grpcStore.serverManageConn.listServerRoles({})
        const allRoles = rolesRes.response.roles
        console.log('All roles:', allRoles)

        // Convert to strings for reliable comparison (BigInt.includes can be unreliable)
        const roleIdStrings = roleIds.map((id) => id.toString())
        this.userRoles = allRoles
          .filter((r) => roleIdStrings.includes(r.id.toString()))
          .map((r) => ({ id: Number(r.id), name: r.name, description: r.description || '' }))
        console.log('Matched user roles:', this.userRoles)

        this.userPermissions = []
        for (const role of this.userRoles) {
          const permRes = await grpcStore.serverManageConn.listServerRolePermissions({
            roleId: BigInt(role.id),
          })
          for (const p of permRes.response.permissions) {
            if (!this.userPermissions.find((x) => x.id === Number(p.id))) {
              this.userPermissions.push({
                id: Number(p.id),
                name: p.name,
                description: p.description || '',
              })
            }
          }
        }
      } catch (error) {
        console.error('Failed to fetch profile data:', error)
      }
    },
    async setServerStatus(status: number) {
      this.serverStatusLoading = true
      try {
        const grpcStore = useGrpcStore()
        await grpcStore.serverManageConn.setServerStatus({
          serverStatus: status as ServerStatus,
          reason: status === 2 ? 'Server maintenance mode enabled' : 'Server normal mode',
        })
        this.serverStatus = status
        ElMessage.success(
          status === 2 ? 'Server is now in maintenance mode' : 'Server is now in normal mode',
        )
      } catch (error) {
        console.error('Failed to set server status:', error)
        ElMessage.error('Failed to set server status')
      } finally {
        this.serverStatusLoading = false
      }
    },
  },
}
</script>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  height: 60px;
  background-color: #fff;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.1);
  z-index: 99;
}

.left {
  display: flex;
  align-items: center;
}

.collapse-icon {
  font-size: 20px;
  margin-right: 20px;
  cursor: pointer;
  color: #606266;
}

.right {
  display: flex;
  align-items: center;
}

.user-info {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.username {
  margin-left: 8px;
  font-size: 14px;
  color: #606266;
}
</style>
