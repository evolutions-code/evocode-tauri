<template>
  <div class="general-panel">
    <div class="setting-row">
      <div class="setting-meta">
        <div class="setting-name">{{ t("config.autostart.title") }}</div>
        <div class="setting-desc muted-3">{{ t("config.autostart.desc") }}</div>
      </div>
      <a-switch :checked="autostartEnabled" :loading="autostartLoading" @change="onAutostartChange" />
    </div>
    <a-divider class="row-divider" />
    <div class="setting-row">
      <div class="setting-meta">
        <div class="setting-name">{{ t("config.bridge_port.title") }}</div>
        <div class="setting-desc muted-3">{{ t("config.bridge_port.desc") }}</div>
      </div>
      <div class="setting-control">
        <a-input-number
          v-model:value="bridgePort"
          :min="1024"
          :max="65535"
          :disabled="portLoading"
          style="width: 120px"
        />
        <a-button type="primary" @click="savePort" :loading="portLoading" style="margin-left: 8px">
          {{ t("config.save") }}
        </a-button>
      </div>
    </div>
    <a-divider class="row-divider" />
    <div class="setting-row">
      <div class="setting-meta">
        <div class="setting-name">{{ t("config.configdir.title") }}</div>
        <div class="setting-desc muted-3">{{ configDirHint }}</div>
      </div>
      <a-button @click="openConfigDir" :loading="openingDir">
        <template #icon><FolderOpenOutlined /></template>
        {{ t("config.configdir.open") }}
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useLocale } from "../../composables/useLocale"
import { message } from "ant-design-vue"
import { enable, isEnabled, disable } from "@tauri-apps/plugin-autostart"
import { openConfigDir as openConfigDirApi, getBridgePort, setBridgePort } from "../../api/bridge"
import { FolderOpenOutlined } from "@ant-design/icons-vue"

const { t } = useLocale()
const autostartEnabled = ref(false)
const autostartLoading = ref(false)
const openingDir = ref(false)
const configDirHint = ref("")
const bridgePort = ref(17761)
const portLoading = ref(false)

onMounted(() => {
  buildConfigDirHint()
  loadAutostartStatus()
  loadPort()
})

function buildConfigDirHint() {
  const sep = navigator?.platform?.toLowerCase().includes("win") ? "\\" : "/"
  configDirHint.value = "~" + sep + ".evocode" + sep + "config.toml"
}

async function loadAutostartStatus() {
  try { autostartEnabled.value = await isEnabled() } catch { autostartEnabled.value = false }
}

async function onAutostartChange(checked: boolean) {
  autostartLoading.value = true
  try {
    if (checked) { await enable(); message.success(t("config.autostart.enabled"), 3) }
    else { await disable(); message.success(t("config.autostart.disabled"), 3) }
    autostartEnabled.value = checked
  } catch (e: any) { message.error(t("config.autostart.error") + ": " + (e?.message || String(e)), 4) }
  finally { autostartLoading.value = false }
}

async function loadPort() {
  try { bridgePort.value = await getBridgePort() } catch { bridgePort.value = 17761 }
}

async function savePort() {
  const port = bridgePort.value
  if (port < 1024 || port > 65535) {
    message.error(t("config.bridge_port.invalid"), 3)
    return
  }
  portLoading.value = true
  try {
    await setBridgePort(port)
    message.success(t("config.bridge_port.saved"), 3)
  } catch (e: any) {
    const msg = e?.message || String(e)
    if (msg.includes("running")) {
      message.warning(t("config.bridge_port.running_warn"), 4)
    } else {
      message.error(t("config.bridge_port.error") + ": " + msg, 4)
    }
    loadPort()
  } finally {
    portLoading.value = false
  }
}

async function openConfigDir() {
  openingDir.value = true
  try { const path = await openConfigDirApi(); message.success(t("config.configdir.opened") + ": " + path, 3) }
  catch (e: any) { message.error(t("config.configdir.error") + ": " + (e?.message || String(e)), 4) }
  finally { openingDir.value = false }
}
</script>

<style scoped>
.general-panel { display: flex; flex-direction: column; gap: 4px; }
.setting-row { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 14px 4px; }
.setting-meta { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.setting-name { font-size: 14px; font-weight: 600; color: var(--text-1); }
.setting-desc { font-size: 12.5px; color: var(--text-3); }
.row-divider { margin: 4px 0; border-color: var(--border); }
.setting-control { display: flex; align-items: center; gap: 4px; }
</style>
