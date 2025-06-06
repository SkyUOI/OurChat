<template>
  <div class="dashboard">
    <SystemOverview />
  </div>
</template>

<script>
import SystemOverview from '@/components/SystemOverview.vue'

export default {
  name: 'Dashboard',
  components: {
    SystemOverview
  }
}
</script>

<style scoped>
.dashboard {
  padding: 20px;
}
</style>
