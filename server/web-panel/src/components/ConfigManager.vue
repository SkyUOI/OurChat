<template>
  <div class="config-manager">
    <h2 class="section-title">{{ $t('configManagement') }}</h2>

    <div class="config-tabs">
      <el-tabs v-model="activeTab" type="card">
        <!-- Server Configuration (ourchat.toml) -->
        <el-tab-pane :label="$t('configTabs.server')" name="server">
          <el-form
            ref="serverFormRef"
            :model="serverConfig"
            :rules="serverRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item prop="auto_clean_duration" :label="$t('configFields.autoCleanDuration')">
              <el-input v-model="serverConfig.auto_clean_duration" placeholder="0 0 * * *" />
              <span class="field-description">{{
                $t('configDescriptions.autoCleanDuration')
              }}</span>
            </el-form-item>

            <el-form-item prop="user_files_limit" :label="$t('configFields.userFilesLimit')">
              <el-input v-model="serverConfig.user_files_limit" placeholder="100MiB" />
              <span class="field-description">{{ $t('configDescriptions.userFilesLimit') }}</span>
            </el-form-item>

            <el-form-item
              prop="friends_number_limit"
              :label="$t('configFields.friendsNumberLimit')"
            >
              <el-input-number v-model="serverConfig.friends_number_limit" :min="1" :max="100000" />
              <span class="field-description">{{
                $t('configDescriptions.friendsNumberLimit')
              }}</span>
            </el-form-item>

            <el-form-item
              prop="verification_expire_time"
              :label="$t('configFields.verificationExpireTime')"
            >
              <el-input v-model="serverConfig.verification_expire_time" placeholder="3d" />
              <span class="field-description">{{
                $t('configDescriptions.verificationExpireTime')
              }}</span>
            </el-form-item>

            <el-form-item prop="files_storage_path" :label="$t('configFields.filesStoragePath')">
              <el-input v-model="serverConfig.files_storage_path" placeholder="files_storage/" />
              <span class="field-description">{{ $t('configDescriptions.filesStoragePath') }}</span>
            </el-form-item>

            <el-form-item prop="files_save_time" :label="$t('configFields.filesSaveTime')">
              <el-input v-model="serverConfig.files_save_time" placeholder="10d" />
              <span class="field-description">{{ $t('configDescriptions.filesSaveTime') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.singleInstance')">
              <el-switch v-model="serverConfig.single_instance" />
              <span class="field-description">{{ $t('configDescriptions.singleInstance') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.leaderNode')">
              <el-switch
                v-model="serverConfig.leader_node"
                :disabled="!serverConfig.single_instance"
              />
              <span class="field-description">{{ $t('configDescriptions.leaderNode') }}</span>
            </el-form-item>

            <el-form-item prop="log_clean_duration" :label="$t('configFields.logCleanDuration')">
              <el-input v-model="serverConfig.log_clean_duration" placeholder="30d" />
              <span class="field-description">{{ $t('configDescriptions.logCleanDuration') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.requireEmailVerification')">
              <el-switch v-model="serverConfig.require_email_verification" />
              <span class="field-description">{{
                $t('configDescriptions.requireEmailVerification')
              }}</span>
            </el-form-item>

            <el-form-item prop="unregister_policy" :label="$t('configFields.unregisterPolicy')">
              <el-select v-model="serverConfig.unregister_policy" placeholder="Select policy">
                <el-option label="disable" value="disable" />
                <el-option label="delete" value="delete" />
              </el-select>
              <span class="field-description">{{ $t('configDescriptions.unregisterPolicy') }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- HTTP Configuration (http.toml) -->
        <el-tab-pane :label="$t('configTabs.http')" name="http">
          <el-form
            ref="httpFormRef"
            :model="httpConfig"
            :rules="httpRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item prop="ip" :label="$t('configFields.httpIp')">
              <el-input v-model="httpConfig.ip" placeholder="0.0.0.0" />
              <span class="field-description">{{ $t('configDescriptions.httpIp') }}</span>
            </el-form-item>

            <el-form-item prop="port" :label="$t('configFields.httpPort')">
              <el-input-number v-model="httpConfig.port" :min="1" :max="65535" />
              <span class="field-description">{{ $t('configDescriptions.httpPort') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.enableMatrix')">
              <el-switch v-model="httpConfig.enable_matrix" />
              <span class="field-description">{{ $t('configDescriptions.enableMatrix') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.runMigration')">
              <el-switch v-model="httpConfig.run_migration" />
              <span class="field-description">{{ $t('configDescriptions.runMigration') }}</span>
            </el-form-item>

            <el-divider>{{ $t('configSections.rateLimit') }}</el-divider>

            <el-form-item :label="$t('configFields.rateLimitEnable')">
              <el-switch v-model="httpConfig.rate_limit_enable" />
              <span class="field-description">{{ $t('configDescriptions.rateLimitEnable') }}</span>
            </el-form-item>

            <el-form-item
              prop="num_of_burst_requests"
              :label="$t('configFields.numOfBurstRequests')"
            >
              <el-input-number v-model="httpConfig.num_of_burst_requests" :min="1" :max="1000" />
              <span class="field-description">{{
                $t('configDescriptions.numOfBurstRequests')
              }}</span>
            </el-form-item>

            <el-form-item prop="replenish_duration" :label="$t('configFields.replenishDuration')">
              <el-input v-model="httpConfig.replenish_duration" placeholder="500ms" />
              <span class="field-description">{{
                $t('configDescriptions.replenishDuration')
              }}</span>
            </el-form-item>

            <el-divider>{{ $t('configSections.tls') }}</el-divider>

            <el-form-item :label="$t('configFields.tlsEnable')">
              <el-switch v-model="httpConfig.tls_enable" />
              <span class="field-description">{{ $t('configDescriptions.tlsEnable') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.clientCertificateRequired')">
              <el-switch
                v-model="httpConfig.client_certificate_required"
                :disabled="!httpConfig.tls_enable"
              />
              <span class="field-description">{{
                $t('configDescriptions.clientCertificateRequired')
              }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- Database Configuration (database.toml) -->
        <el-tab-pane :label="$t('configTabs.database')" name="database">
          <el-form
            ref="databaseFormRef"
            :model="databaseConfig"
            :rules="databaseRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item :label="$t('configFields.dbHost')">
              <el-input v-model="databaseConfig.host" placeholder="db" />
              <span class="field-description">{{ $t('configDescriptions.dbHost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.dbPort')">
              <el-input-number v-model="databaseConfig.port" :min="1" :max="65535" />
              <span class="field-description">{{ $t('configDescriptions.dbPort') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.dbUser')">
              <el-input v-model="databaseConfig.user" placeholder="postgres" />
              <span class="field-description">{{ $t('configDescriptions.dbUser') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.dbPassword')">
              <el-input
                v-model="databaseConfig.passwd"
                type="password"
                show-password
                placeholder="123456"
              />
              <span class="field-description">{{ $t('configDescriptions.dbPassword') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.dbName')">
              <el-input v-model="databaseConfig.db" placeholder="OurChat" />
              <span class="field-description">{{ $t('configDescriptions.dbName') }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- Redis Configuration (redis.toml) -->
        <el-tab-pane :label="$t('configTabs.redis')" name="redis">
          <el-form
            ref="redisFormRef"
            :model="redisConfig"
            :rules="redisRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item :label="$t('configFields.redisHost')">
              <el-input v-model="redisConfig.host" placeholder="redis" />
              <span class="field-description">{{ $t('configDescriptions.redisHost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.redisPort')">
              <el-input-number v-model="redisConfig.port" :min="1" :max="65535" />
              <span class="field-description">{{ $t('configDescriptions.redisPort') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.redisUser')">
              <el-input v-model="redisConfig.user" placeholder="default" />
              <span class="field-description">{{ $t('configDescriptions.redisUser') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.redisPassword')">
              <el-input
                v-model="redisConfig.passwd"
                type="password"
                show-password
                placeholder="123456"
              />
              <span class="field-description">{{ $t('configDescriptions.redisPassword') }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- RabbitMQ Configuration (rabbitmq.toml) -->
        <el-tab-pane :label="$t('configTabs.rabbitmq')" name="rabbitmq">
          <el-form
            ref="rabbitmqFormRef"
            :model="rabbitmqConfig"
            :rules="rabbitmqRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item :label="$t('configFields.rabbitmqHost')">
              <el-input v-model="rabbitmqConfig.host" placeholder="mq" />
              <span class="field-description">{{ $t('configDescriptions.rabbitmqHost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.rabbitmqPort')">
              <el-input-number v-model="rabbitmqConfig.port" :min="1" :max="65535" />
              <span class="field-description">{{ $t('configDescriptions.rabbitmqPort') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.rabbitmqUser')">
              <el-input v-model="rabbitmqConfig.user" placeholder="guest" />
              <span class="field-description">{{ $t('configDescriptions.rabbitmqUser') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.rabbitmqPassword')">
              <el-input
                v-model="rabbitmqConfig.passwd"
                type="password"
                show-password
                placeholder="123456"
              />
              <span class="field-description">{{ $t('configDescriptions.rabbitmqPassword') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.rabbitmqVhost')">
              <el-input v-model="rabbitmqConfig.vhost" placeholder="/" />
              <span class="field-description">{{ $t('configDescriptions.rabbitmqVhost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.rabbitmqManagePort')">
              <el-input-number v-model="rabbitmqConfig.manage_port" :min="1" :max="65535" />
              <span class="field-description">{{
                $t('configDescriptions.rabbitmqManagePort')
              }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- Password Hash Configuration -->
        <el-tab-pane :label="$t('configTabs.passwordHash')" name="passwordHash">
          <el-form
            ref="passwordHashFormRef"
            :model="passwordHashConfig"
            :rules="passwordHashRules"
            label-width="180px"
            class="config-form"
          >
            <el-form-item :label="$t('configFields.mCost')">
              <el-input-number v-model="passwordHashConfig.m_cost" :min="8" :max="1000000" />
              <span class="field-description">{{ $t('configDescriptions.mCost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.tCost')">
              <el-input-number v-model="passwordHashConfig.t_cost" :min="1" :max="100" />
              <span class="field-description">{{ $t('configDescriptions.tCost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.pCost')">
              <el-input-number v-model="passwordHashConfig.p_cost" :min="1" :max="100" />
              <span class="field-description">{{ $t('configDescriptions.pCost') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.outputLen')">
              <el-input-number v-model="passwordHashConfig.output_len" :min="16" :max="64" />
              <span class="field-description">{{ $t('configDescriptions.outputLen') }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- Advanced Configuration -->
        <el-tab-pane :label="$t('configTabs.advanced')" name="advanced">
          <el-form
            ref="advancedFormRef"
            :model="advancedConfig"
            :rules="advancedRules"
            label-width="180px"
            class="config-form"
          >
            <el-divider>{{ $t('configSections.debug') }}</el-divider>

            <el-form-item :label="$t('configFields.debugConsole')">
              <el-switch v-model="advancedConfig.debug_console" />
              <span class="field-description">{{ $t('configDescriptions.debugConsole') }}</span>
            </el-form-item>

            <el-form-item :label="$t('configFields.debugConsolePort')">
              <el-input-number v-model="advancedConfig.debug_console_port" :min="1" :max="65535" />
              <span class="field-description">{{ $t('configDescriptions.debugConsolePort') }}</span>
            </el-form-item>

            <el-divider>{{ $t('configSections.voip') }}</el-divider>

            <el-form-item :label="$t('configFields.emptyRoomKeepDuration')">
              <el-input v-model="advancedConfig.empty_room_keep_duration" placeholder="1h" />
              <span class="field-description">{{
                $t('configDescriptions.emptyRoomKeepDuration')
              }}</span>
            </el-form-item>

            <el-divider>{{ $t('configSections.oauth') }}</el-divider>

            <el-form-item :label="$t('configFields.oauthEnable')">
              <el-switch v-model="advancedConfig.oauth_enable" />
              <span class="field-description">{{ $t('configDescriptions.oauthEnable') }}</span>
            </el-form-item>

            <el-form-item
              :label="$t('configFields.githubClientId')"
              v-if="advancedConfig.oauth_enable"
            >
              <el-input v-model="advancedConfig.github_client_id" placeholder="" />
              <span class="field-description">{{ $t('configDescriptions.githubClientId') }}</span>
            </el-form-item>

            <el-form-item
              :label="$t('configFields.githubClientSecret')"
              v-if="advancedConfig.oauth_enable"
            >
              <el-input
                v-model="advancedConfig.github_client_secret"
                type="password"
                show-password
                placeholder=""
              />
              <span class="field-description">{{
                $t('configDescriptions.githubClientSecret')
              }}</span>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>

      <div class="actions">
        <el-button type="primary" icon="el-icon-download" @click="saveConfig">{{
          $t('configActions.save')
        }}</el-button>
        <el-button type="success" icon="el-icon-refresh" @click="loadConfig">{{
          $t('configActions.reload')
        }}</el-button>
        <el-button type="warning" icon="el-icon-view" @click="showHistory = !showHistory">{{
          $t('configActions.viewHistory')
        }}</el-button>
      </div>
    </div>

    <div class="config-history" v-if="showHistory">
      <h3>{{ $t('configHistory.title') }}</h3>
      <el-table :data="history" style="width: 100%">
        <el-table-column
          prop="time"
          :label="$t('configHistory.time')"
          width="180"
        ></el-table-column>
        <el-table-column
          prop="user"
          :label="$t('configHistory.user')"
          width="120"
        ></el-table-column>
        <el-table-column
          prop="type"
          :label="$t('configHistory.type')"
          width="120"
        ></el-table-column>
        <el-table-column
          prop="description"
          :label="$t('configHistory.description')"
        ></el-table-column>
        <el-table-column :label="$t('configHistory.actions')" width="150">
          <template #default="{ row }">
            <el-button size="small" @click="viewHistory(row)">{{
              $t('configHistory.view')
            }}</el-button>
            <el-button size="small" type="danger" @click="rollbackHistory(row)">{{
              $t('configHistory.rollback')
            }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { ElMessage, ElMessageBox, type FormInstance, type FormRules } from 'element-plus'
import { useGrpcStore } from '@/stores/grpc'

// Active tab
const activeTab = ref('server')
const showHistory = ref(false)

// Form refs
const serverFormRef = ref<FormInstance>()
const httpFormRef = ref<FormInstance>()
const databaseFormRef = ref<FormInstance>()
const redisFormRef = ref<FormInstance>()
const rabbitmqFormRef = ref<FormInstance>()
const passwordHashFormRef = ref<FormInstance>()
const advancedFormRef = ref<FormInstance>()

// gRPC store
const grpcStore = useGrpcStore()

// Note: ConfigType was removed from the proto - now using raw TOML content
// Config sections are managed client-side for display purposes

// Validation rules
const serverRules: FormRules = {
  auto_clean_duration: [
    { required: true, message: 'Auto clean duration is required', trigger: 'blur' },
    { pattern: /^(\S+\s+){4}\S+$/, message: 'Must be a valid cron expression', trigger: 'blur' },
  ],
  user_files_limit: [
    { required: true, message: 'User files limit is required', trigger: 'blur' },
    { pattern: /^\d+(\.\d+)?[KMG]?i?B$/, message: 'Must be like 100MiB, 1GiB', trigger: 'blur' },
  ],
  friends_number_limit: [
    { required: true, message: 'Friends number limit is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 100000,
      message: 'Must be between 1 and 100000',
      trigger: 'blur',
    },
  ],
  verification_expire_time: [
    { required: true, message: 'Verification expire time is required', trigger: 'blur' },
    { pattern: /^\d+[dhms]$/, message: 'Must be like 3d, 24h, 30m, 60s', trigger: 'blur' },
  ],
  files_storage_path: [
    { required: true, message: 'Files storage path is required', trigger: 'blur' },
  ],
  files_save_time: [
    { required: true, message: 'Files save time is required', trigger: 'blur' },
    { pattern: /^\d+[dhms]$/, message: 'Must be like 10d, 24h', trigger: 'blur' },
  ],
  log_clean_duration: [
    { required: true, message: 'Log clean duration is required', trigger: 'blur' },
    { pattern: /^\d+[dhms]$/, message: 'Must be like 30d, 7d', trigger: 'blur' },
  ],
  unregister_policy: [
    { required: true, message: 'Unregister policy is required', trigger: 'change' },
  ],
}

const httpRules: FormRules = {
  ip: [
    { required: true, message: 'IP address is required', trigger: 'blur' },
    {
      pattern: /^(?:\d{1,3}\.){3}\d{1,3}$|^0\.0\.0\.0$/,
      message: 'Must be a valid IP address',
      trigger: 'blur',
    },
  ],
  port: [
    { required: true, message: 'Port is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
  num_of_burst_requests: [
    { type: 'number', min: 1, max: 1000, message: 'Must be between 1 and 1000', trigger: 'blur' },
  ],
  replenish_duration: [
    { pattern: /^\d+(ms|s|m|h)$/, message: 'Must be like 500ms, 1s, 5m, 1h', trigger: 'blur' },
  ],
}

const databaseRules: FormRules = {
  host: [{ required: true, message: 'Database host is required', trigger: 'blur' }],
  port: [
    { required: true, message: 'Database port is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
  user: [{ required: true, message: 'Database user is required', trigger: 'blur' }],
  passwd: [{ required: true, message: 'Database password is required', trigger: 'blur' }],
  db: [{ required: true, message: 'Database name is required', trigger: 'blur' }],
}

const redisRules: FormRules = {
  host: [{ required: true, message: 'Redis host is required', trigger: 'blur' }],
  port: [
    { required: true, message: 'Redis port is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
}

const rabbitmqRules: FormRules = {
  host: [{ required: true, message: 'RabbitMQ host is required', trigger: 'blur' }],
  port: [
    { required: true, message: 'RabbitMQ port is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
  manage_port: [
    { required: true, message: 'RabbitMQ management port is required', trigger: 'blur' },
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
}

const passwordHashRules: FormRules = {
  m_cost: [
    { required: true, message: 'Memory cost is required', trigger: 'blur' },
    {
      type: 'number',
      min: 8,
      max: 1000000,
      message: 'Must be between 8 and 1000000',
      trigger: 'blur',
    },
  ],
  t_cost: [
    { required: true, message: 'Time cost is required', trigger: 'blur' },
    { type: 'number', min: 1, max: 100, message: 'Must be between 1 and 100', trigger: 'blur' },
  ],
  p_cost: [
    { required: true, message: 'Parallelism cost is required', trigger: 'blur' },
    { type: 'number', min: 1, max: 100, message: 'Must be between 1 and 100', trigger: 'blur' },
  ],
  output_len: [
    { required: true, message: 'Output length is required', trigger: 'blur' },
    {
      type: 'number',
      min: 16,
      max: 64,
      message: 'Must be between 16 and 64 bytes',
      trigger: 'blur',
    },
  ],
}

// Configuration models
const serverConfig = reactive({
  auto_clean_duration: '0 0 * * *',
  user_files_limit: '100MiB',
  friends_number_limit: 5000,
  verification_expire_time: '3d',
  files_storage_path: 'files_storage/',
  files_save_time: '10d',
  single_instance: true,
  leader_node: true,
  log_clean_duration: '30d',
  require_email_verification: false,
  unregister_policy: 'disable',
})

const httpConfig = reactive({
  ip: '0.0.0.0',
  port: 7777,
  enable_matrix: false,
  run_migration: false,
  rate_limit_enable: true,
  num_of_burst_requests: 16,
  replenish_duration: '500ms',
  tls_enable: false,
  client_certificate_required: false,
})

const databaseConfig = reactive({
  host: 'db',
  port: 5432,
  user: 'postgres',
  passwd: '123456',
  db: 'OurChat',
})

const redisConfig = reactive({
  host: 'redis',
  port: 6379,
  user: 'default',
  passwd: '123456',
})

const rabbitmqConfig = reactive({
  host: 'mq',
  port: 5672,
  user: 'guest',
  passwd: '123456',
  vhost: '/',
  manage_port: 15672,
})

const passwordHashConfig = reactive({
  m_cost: 19456,
  t_cost: 2,
  p_cost: 1,
  output_len: 32,
})

const advancedConfig = reactive({
  debug_console: true,
  debug_console_port: 7776,
  empty_room_keep_duration: '1h',
  oauth_enable: false,
  github_client_id: '',
  github_client_secret: '',
})

// Advanced configuration rules
const advancedRules = computed<FormRules>(() => ({
  debug_console_port: [
    {
      type: 'number',
      min: 1,
      max: 65535,
      message: 'Port must be between 1 and 65535',
      trigger: 'blur',
    },
  ],
  empty_room_keep_duration: [
    { pattern: /^\d+[dhms]$/, message: 'Must be like 1h, 30m, 24h', trigger: 'blur' },
  ],
  github_client_id: [
    {
      required: advancedConfig.oauth_enable,
      message: 'GitHub Client ID is required when OAuth is enabled',
      trigger: 'blur',
    },
  ],
  github_client_secret: [
    {
      required: advancedConfig.oauth_enable,
      message: 'GitHub Client Secret is required when OAuth is enabled',
      trigger: 'blur',
    },
  ],
}))

type HistoryItem = { time: string; user: string; type: string; description: string }
// History data will be populated from server
const history = ref<HistoryItem[]>([])

// Methods
const saveConfig = async () => {
  try {
    // Validate active form
    const formRefs: Record<string, FormInstance | undefined> = {
      server: serverFormRef.value,
      http: httpFormRef.value,
      database: databaseFormRef.value,
      redis: redisFormRef.value,
      rabbitmq: rabbitmqFormRef.value,
      passwordHash: passwordHashFormRef.value,
      advanced: advancedFormRef.value,
    }

    const activeFormRef = formRefs[activeTab.value]
    if (activeFormRef) {
      try {
        await activeFormRef.validate()
      } catch {
        ElMessage.error('Please fix validation errors before saving')
        return
      }
    }

    // Get the config data based on active tab
    let configData: Record<string, unknown> = {}
    switch (activeTab.value) {
      case 'server':
        configData = serverConfig
        break
      case 'http':
        configData = httpConfig
        break
      case 'database':
        configData = databaseConfig
        break
      case 'redis':
        configData = redisConfig
        break
      case 'rabbitmq':
        configData = rabbitmqConfig
        break
      case 'passwordHash':
        configData = passwordHashConfig
        break
      case 'advanced':
        configData = advancedConfig
        break
    }

    // Convert to string (JSON for now, should be TOML when server implements it)
    const content = JSON.stringify(configData, null, 2)

    try {
      const response = await grpcStore.serverManageConn.setConfig({
        content,
      })

      if (response.response.success) {
        ElMessage.success('Configuration saved successfully')
        // Add to history
        history.value.unshift({
          time: new Date().toLocaleString(),
          user: 'admin',
          type: activeTab.value,
          description: 'Configuration saved via gRPC',
        })
      } else {
        ElMessage.error(`Failed to save configuration: ${response.response.message}`)
      }
    } catch (error: unknown) {
      console.error('gRPC error:', error)
      const message = error instanceof Error ? error.message : String(error)
      const displayMessage = message || 'Server error'
      ElMessage.error(`Failed to save configuration: ${displayMessage}`)
    }
  } catch (error) {
    ElMessage.error('Failed to save configuration')
    console.error('Save error:', error)
  }
}

const loadConfig = async () => {
  try {
    try {
      const response = await grpcStore.serverManageConn.getConfig({})

      const content = response.response.content
      // Try to parse as JSON (for now, should be TOML when server implements it)
      try {
        const parsed = JSON.parse(content)
        // Update the appropriate config object based on active tab
        switch (activeTab.value) {
          case 'server':
            Object.assign(serverConfig, parsed)
            break
          case 'http':
            Object.assign(httpConfig, parsed)
            break
          case 'database':
            Object.assign(databaseConfig, parsed)
            break
          case 'redis':
            Object.assign(redisConfig, parsed)
            break
          case 'rabbitmq':
            Object.assign(rabbitmqConfig, parsed)
            break
          case 'passwordHash':
            Object.assign(passwordHashConfig, parsed)
            break
          case 'advanced':
            Object.assign(advancedConfig, parsed)
            break
        }
        ElMessage.success('Configuration loaded from server')
      } catch (parseError) {
        console.error('Failed to parse config content:', parseError)
        ElMessage.warning(
          'Received config but could not parse it. Server may be sending TOML format.',
        )
      }
    } catch (error: unknown) {
      console.error('gRPC error:', error)
      const message = error instanceof Error ? error.message : String(error)
      const displayMessage = message || 'Server error'
      ElMessage.error(`Failed to load configuration: ${displayMessage}`)
    }
  } catch (error) {
    console.error('Load config error:', error)
    ElMessage.error('Failed to load configuration')
  }
}

const viewHistory = (item: HistoryItem) => {
  ElMessage.info(`Viewing history: ${item.description}`)
}

const rollbackHistory = (item: HistoryItem) => {
  ElMessageBox.confirm(
    `Are you sure you want to rollback to this configuration from ${item.time}?`,
    'Confirm Rollback',
    {
      confirmButtonText: 'Rollback',
      cancelButtonText: 'Cancel',
      type: 'warning',
    },
  )
    .then(() => {
      ElMessage.success('Configuration rolled back successfully')
    })
    .catch(() => {
      // Cancel
    })
}
</script>

<style scoped>
.config-manager {
  padding: 20px;
}

.section-title {
  margin-bottom: 20px;
  color: #303133;
  font-size: 1.5rem;
}

.config-tabs {
  margin-bottom: 30px;
}

.config-form {
  margin-top: 20px;
  max-width: 800px;
}

.field-description {
  display: block;
  margin-top: 4px;
  font-size: 0.85rem;
  color: #909399;
  line-height: 1.4;
}

.actions {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.config-history {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid #e4e7ed;
}
</style>
