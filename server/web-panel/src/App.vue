<script setup lang="ts">
import { RouterView } from 'vue-router'
import Header from '@/components/Header.vue'
import Sidebar from '@/components/Sidebar.vue'
</script>

<template>
  <div class="app-container">
    <Sidebar />
    <div class="main-container">
      <Header />
      <div class="content">
        <RouterView />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  background-color: #f5f7fa;
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}
</style>
