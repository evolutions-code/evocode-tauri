<template>
  <div class="logs-page">
    <section class="page-header">
      <div class="page-title">
        <span class="bar" />
        <span>{{ t("logs.title") }}</span>
      </div>
    </section>
    <section class="log-section">
      <LogPanel :bridge-running="bridgeStatus === 'running'" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useLocale } from "../composables/useLocale"
import { getBridgeStatus } from "../api/bridge"
import LogPanel from "../components/LogPanel.vue"

const { t } = useLocale()
const bridgeStatus = ref("stopped")

onMounted(async () => {
  try {
    bridgeStatus.value = await getBridgeStatus()
  } catch {}
})
</script>

<style scoped>
.logs-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
  flex: 1;
  min-height: 0;
}
.log-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
