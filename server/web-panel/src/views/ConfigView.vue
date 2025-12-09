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
  padding: 20px;
}
</style>
