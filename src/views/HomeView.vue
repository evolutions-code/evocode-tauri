<template>
  <div class="home">
    <header class="header">
      <h1>evocode</h1>
      <span class="version">v0.1.0</span>
    </header>

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

.header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  margin-bottom: 24px;
}

.header h1 {
  font-size: 28px;
  font-weight: 600;
  color: #fff;
}

.version {
  font-size: 13px;
  color: #888;
}

.actions {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin: 20px 0;
}

.action-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 20px 16px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 10px;
  text-align: left;
  text-decoration: none;
  color: inherit;
  transition: border-color 0.2s, background 0.2s;
  cursor: pointer;
  font-family: inherit;
}

.action-card:hover {
  border-color: #60a5fa;
  background: #252525;
}

.action-icon {
  font-size: 18px;
  color: #60a5fa;
}

.action-title {
  font-size: 15px;
  font-weight: 500;
  color: #e0e0e0;
}

.action-desc {
  font-size: 12px;
  color: #888;
}
</style>
