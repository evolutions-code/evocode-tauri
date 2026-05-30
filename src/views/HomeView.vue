<template>
  <div class="home">
    <BridgeStatus :status="bridgeStatus" :loading="loading" @toggle="toggleBridge" />

    <LogPanel :bridge-running="bridgeStatus === 'running'" />

    <QuickRef />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { startBridge, stopBridge, getBridgeStatus } from '../api/bridge'
import BridgeStatus from '../components/BridgeStatus.vue'
import QuickRef from '../components/QuickRef.vue'
import LogPanel from '../components/LogPanel.vue'

const bridgeStatus = ref('stopped')
const loading = ref(false)

async function updateStatus() {
  bridgeStatus.value = await getBridgeStatus()
}

async function toggleBridge() {
  loading.value = true
  try {
    if (bridgeStatus.value === 'running') {
      await stopBridge()
    } else {
      await startBridge()
    }
    await updateStatus()
  } finally {
    loading.value = false
  }
}

onMounted(updateStatus)
</script>

<style scoped>
.home {
  max-width: 640px;
  margin: 0 auto;
  padding: 24px 20px;
}
</style>
