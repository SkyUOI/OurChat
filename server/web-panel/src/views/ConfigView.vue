<template>
  <div class="config-view">
    <h1 class="text-2xl font-bold mb-6">{{ $t('configManagement') }}</h1>
    <ConfigManager />
  </div>
</template>

<script setup lang="ts">
import ConfigManager from '@/components/ConfigManager.vue'
</script>

<style scoped>
.config-view {
  padding: 24px;
  animation: fadeInUp 0.4s ease-out;
}

.config-view h1 {
  font-size: 28px;
  font-weight: 700;
  color: #1e293b;
  margin-bottom: 24px;
}
</style>
