<template>
  <div class="connection-panel">
    <div class="glass panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">{{ t("config.providers") }}</div>
        </div>
        <div class="active-with-sync">
          <a-tag v-if="activeId" class="active-tag">{{ t("config.providers.active") }} {{ activeId }}</a-tag>
          <a-select
            v-if="providerIds.length"
            v-model:value="activeId"
            style="width: 240px;"
            size="small"
            :loading="syncing"
            @change="handleSyncToCodex">
            <a-select-option v-for="id in providerIds" :key="id" :value="id">
              {{ id }}
            </a-select-option>
          </a-select>
          <a-tag v-else style="font-weight: 400;">{{ t("config.sync.no_providers") }}</a-tag>
        </div>
      </div>

      <a-tabs
        v-if="providerIds.length"
        type="editable-card"
        hideAdd
        v-model:activeKey="editingId"
        @change="onTabChange"
        @edit="onTabEdit"
        class="prov-tabs"
        size="small"
      >
        <a-tab-pane
          v-for="id in providerIds"
          :key="id"
          :tab="id"
          :closable="providerIds.length > 1"
        />
      </a-tabs>
      <div v-else class="empty-tabs-hint">{{ t("config.sync.no_providers") }}</div>


    </div>

    <a-modal
      v-model:open="showAddModal"
      :title="t('config.providers.add')"
      :ok-text="t('config.providers.ok')"
      :cancel-text="t('config.providers.cancel')"
      @ok="doAddProvider"
    >
      <a-input
        v-model:value="newProviderName"
        :placeholder="t('config.providers.placeholder')"
        @press-enter="doAddProvider"
      />
    </a-modal>

    <a-modal
      v-model:open="showRemoveModal"
      :title="t('config.providers.remove_title')"
      :ok-text="t('config.providers.ok')"
      :cancel-text="t('config.providers.cancel')"
      @ok="doRemoveProvider"
    >
      <p>{{ removeTarget ? t("config.providers.remove_confirm", { name: removeTarget }) : "" }}</p>
    </a-modal>
    <div class="glass panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">{{ t("config.wire_api") }}</div>
          <div class="panel-sub muted-3">{{ t("config.wire_api.desc") }}</div>
        </div>
      </div>
      <a-segmented v-model:value="activePresetKey" :options="wireOptions" block @change="onWireApiChange" />
    </div>

    <div class="glass panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">{{ t("config.form.title") }}</div>
          <div class="panel-sub muted-3">{{ t("config.form.desc") }} <code class="mono">[providers.{{ activeId || "..." }}]</code>.</div>
        </div>
        <div class="head-actions">
          <a-button size="small" :disabled="!editingId" @click="resetForm">
            <template #icon><ReloadOutlined /></template> Reset
          </a-button>
        </div>
      </div>
      <a-alert v-if="connResult" class="conn-alert"
        :type="connResult.ok ? 'success' : 'error'"
        :message="connResult.message"
        :description="t('config.form.test_latency') + ': ' + connResult.latency_ms + ' ms'"
        show-icon closable @close="connResult = null" />
      <a-empty v-if="!editingId" :description="t('config.form.empty')" class="empty-block" />
      <a-form v-else layout="vertical" class="form" :model="formState">
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item :label="t('config.form.model')" required>
              <a-input v-model:value="formState.model" :placeholder="t('config.form.model_placeholder')" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item :label="t('config.form.wire_api')">
              <a-tooltip :title="t('config.wire_api.tooltip')">
                <a-select v-model:value="formState.wireApi" @change="onWireApiSelectChange">
                  <a-select-option value="anthropic"><span class="opt-row"><span class="dot purple" /> {{ t("config.wire.anthropic") }}</span></a-select-option>
                  <a-select-option value="chat_completions"><span class="opt-row"><span class="dot blue" /> {{ t("config.wire.chat") }}</span></a-select-option>
                  <a-select-option value="openai"><span class="opt-row"><span class="dot cyan" /> {{ t("config.wire.openai") }}</span></a-select-option>
                </a-select>
              </a-tooltip>
            </a-form-item>
          </a-col>
        </a-row>
        <a-form-item :label="t('config.form.base_url')" required>
          <a-input v-model:value="formState.baseUrl" :placeholder="t('config.form.base_url_placeholder')" />
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item :label="t('config.form.api_key')">
              <a-input-password v-model:value="formState.apiKey" :placeholder="t('config.form.api_key_placeholder')" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item :label="t('config.form.api_key_header')">
              <a-tooltip :title="t('config.form.api_key_header_tooltip')">
                <a-input v-model:value="formState.apiKeyHeader" :placeholder="t('config.form.api_key_header_placeholder')" />
              </a-tooltip>
            </a-form-item>
          </a-col>
        </a-row>
      </a-form>
    </div>

    <div v-if="editingId" class="glass panel">
      <div class="panel-head">
        <div>
          <div class="panel-title">{{ t("config.limits.title") }}</div>
          <div class="panel-sub muted-3">{{ t("config.limits.desc") }}</div>
        </div>
        <a-button size="small" :loading="testingConn" @click="testConnection">
          <template #icon><ApiOutlined /></template>
          {{ t("config.form.test") }}
        </a-button>
      </div>

      <div class="slider-block">
        <div class="slider-head">
          <div>
            <div class="slider-label">{{ t("config.limits.context_window") }}</div>
            <div class="slider-value">{{ contextLabel(ctxValue) }}</div>
          </div>
        </div>
        <div ref="ctxRail" class="slider-rail"
          @mousedown="(e: any) => startDrag(e, 'context')"
          @touchstart.prevent="(e: any) => startDrag(e, 'context')">
          <div class="slider-fill" :style="{ width: ctxPercent + '%' }" />
          <div class="slider-thumb" :style="{ left: ctxPercent + '%' }"><span class="thumb-tip">{{ contextLabel(ctxValue) }}</span></div>
          <div v-for="(_, i) in LIMIT_PRESETS" :key="i" class="slider-tick" :style="{ left: tickLeft(i) + '%' }" />
        </div>
        <div class="slider-stops">
          <div v-for="p in LIMIT_PRESETS" :key="p.key" class="slider-stop"
            :class="{ active: ctxValue === p.context }"
            @click="applyLimitPreset(p.key)">
            <div class="stop-dot" :style="{ color: p.key === 'compact' ? '#34d399' : '#4d7dff' }" />
            <div class="stop-name">{{ t(p.labelKey) }}</div>
            <div class="stop-label">{{ contextLabel(p.context) }}</div>
          </div>
        </div>
      </div>

      <a-divider style="margin: 8px 0;" />

      <div class="slider-block">
        <div class="slider-head">
          <div>
            <div class="slider-label">
              {{ t("config.limits.auto_compact") }}
              <a-tooltip :title="t('config.limits.auto_compact_tip')"><InfoCircleOutlined class="compact-tag" /></a-tooltip>
            </div>
            <div class="slider-value">{{ contextLabel(compactTokens) }}</div>
          </div>
          <div class="slider-label" style="margin-bottom: 2px;">{{ compactRatio }}%</div>
        </div>
        <div ref="ratioRail" class="slider-rail"
          @mousedown="(e: any) => startDrag(e, 'ratio')"
          @touchstart.prevent="(e: any) => startDrag(e, 'ratio')">
          <div class="slider-fill compact" :style="{ width: compactRatio + '%' }" />
          <div class="slider-thumb" :style="{ left: compactRatio + '%' }">
            <span class="thumb-tip">{{ compactRatio }}%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>



<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted } from "vue"
import { useLocale } from "../../composables/useLocale"
import { writeConfig, syncToCodex, readConfig } from "../../api/bridge"
import { message } from "ant-design-vue"
import { PlusOutlined, ReloadOutlined, ApiOutlined, InfoCircleOutlined } from "@ant-design/icons-vue"
import { testProviderConnectivity } from "../../api/bridge"

const { t } = useLocale()

interface Provider {
  providerId: string
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
  modelContextWindow: number
  modelAutoCompactLimit: number
}

const CONTEXT_MIN = 16_000
const CONTEXT_MAX = 2_000_000
const DEFAULT_CONTEXT_WINDOW = 128_000
const DEFAULT_COMPACT_LIMIT = 100_000

const LIMIT_PRESETS = [
  { key: "256k", context: 128_000, compact: 100_000, labelKey: "config.limits.preset_256k" as const },
  { key: "512k", context: 256_000, compact: 220_000, labelKey: "config.limits.preset_512k" as const },
  { key: "1m", context: 1_000_000, compact: 800_000, labelKey: "config.limits.preset_1m" as const },
  { key: "compact", context: 200_000, compact: 40_000, labelKey: "config.limits.preset_compact" as const },
]

const PRESETS = [
  { key: "anthropic", values: { wireApi: "anthropic" as const, apiKeyHeader: "X-Api-Key" as const } },
  { key: "chat_completions", values: { wireApi: "chat_completions" as const, apiKeyHeader: "Authorization" as const } },
  { key: "openai", values: { wireApi: "openai" as const, apiKeyHeader: "Authorization" as const } },
]

const wireOptions = PRESETS.map((p) => ({ value: p.key, label: t("config.wire." + p.values.wireApi) }))

const providerIds = ref<string[]>([])
const providers = reactive<Record<string, Provider>>({})
const activeId = ref("")
const editingId = ref("")
const newProviderName = ref("")
const showAddModal = ref(false)
const showRemoveModal = ref(false)
const removeTarget = ref("")
const activePresetKey = ref("anthropic")
const testingConn = ref(false)
const syncing = ref(false)
const connResult = ref<null | { ok: boolean; status: number; latency_ms: number; message: string }>(null)

const formState = reactive<Provider>({
  providerId: "", wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key",
  modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
})

const ctxValue = computed(() => formState.modelContextWindow || DEFAULT_CONTEXT_WINDOW)
const compactRatio = computed(() => {
  if (!formState.modelContextWindow) return 0
  return Math.round((formState.modelAutoCompactLimit / formState.modelContextWindow) * 1000) / 10
})
const compactTokens = computed(() => Math.round(ctxValue.value * compactRatio.value / 100))
const ctxPercent = computed(() => {
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, ctxValue.value))
  return Math.round(((v - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
})

function contextLabel(n: number) {
  if (!n) return "0"
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(n % 1_000_000 === 0 ? 0 : 1).replace(/\.0$/, "") + "M"
  if (n >= 1_000) return Math.round(n / 1_000) + "K"
  return String(n)
}

function tickLeft(i: number) {
  const p = LIMIT_PRESETS[i]
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, p.context))
  return Math.round(((v - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
}

const ctxRail = ref<HTMLElement | null>(null)
const ratioRail = ref<HTMLElement | null>(null)
let dragging: "context" | "ratio" | null = null
let onMove: ((e: MouseEvent | TouchEvent) => void) | null = null
let onUp: (() => void) | null = null

function railPercent(rail: HTMLElement, clientX: number): number {
  const rect = rail.getBoundingClientRect()
  return Math.min(100, Math.max(0, ((clientX - rect.left) / rect.width) * 100))
}

function applyDrag(target: "context" | "ratio", clientX: number) {
  if (target === "context") {
    const rail = ctxRail.value; if (!rail) return
    const pct = railPercent(rail, clientX)
    let snapped: number | null = null
    for (const p of LIMIT_PRESETS) {
      if (Math.abs(((p.context - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100 - pct) < 6) { snapped = p.context; break }
    }
    if (snapped == null) setContextWindow(CONTEXT_MIN + (CONTEXT_MAX - CONTEXT_MIN) * (pct / 100))
    else setContextWindow(snapped)
  } else {
    const rail = ratioRail.value; if (!rail) return
    setCompactRatio(railPercent(rail, clientX))
  }
}

function setContextWindow(value: number) {
  const next = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, Math.round(value)))
  formState.modelContextWindow = next
  formState.modelAutoCompactLimit = Math.round(next * 0.8)
}

function setCompactRatio(percent: number) {
  formState.modelAutoCompactLimit = Math.round(formState.modelContextWindow * Math.min(100, Math.max(0, percent)) / 100)
}

function startDrag(e: MouseEvent | TouchEvent, target: "context" | "ratio") {
  const clientX = "touches" in e ? e.touches[0].clientX : (e as MouseEvent).clientX
  applyDrag(target, clientX)
  dragging = target
  onMove = (ev: MouseEvent | TouchEvent) => {
    if (!dragging) return
    applyDrag(dragging, "touches" in ev ? ev.touches[0].clientX : (ev as MouseEvent).clientX)
  }
  onUp = () => { dragging = null; if (onMove) { window.removeEventListener("mousemove", onMove as any); window.removeEventListener("touchmove", onMove as any) } if (onUp) { window.removeEventListener("mouseup", onUp); window.removeEventListener("touchend", onUp) } onMove = null; onUp = null }
  window.addEventListener("mousemove", onMove as any); window.addEventListener("touchmove", onMove as any, { passive: true } as any)
  window.addEventListener("mouseup", onUp); window.addEventListener("touchend", onUp)
}
onUnmounted(() => { dragging = null; if (onMove) { window.removeEventListener("mousemove", onMove as any); window.removeEventListener("touchmove", onMove as any) } if (onUp) { window.removeEventListener("mouseup", onUp); window.removeEventListener("touchend", onUp) } })

function applyLimitPreset(key: string) {
  const p = LIMIT_PRESETS.find((x) => x.key === key)
  if (!p) return
  formState.modelContextWindow = p.context; formState.modelAutoCompactLimit = p.compact
}

function snapshotEditing() {
  if (!editingId.value) return
  providers[editingId.value] = { ...formState, providerId: editingId.value }
}

function loadForm(id: string) {
  if (!id || !providers[id]) {
    Object.assign(formState, { providerId: "", wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT })
    return
  }
  Object.assign(formState, providers[id])
  const m = PRESETS.find((p) => p.values.wireApi === formState.wireApi)
  if (m) activePresetKey.value = m.key
}

function setEditing(id: string) {
  snapshotEditing(); editingId.value = id; loadForm(id)
}

function onTabChange(key: string) {
  setEditing(key)
}

function onTabEdit(targetKey: string | MouseEvent, action: string) {
  if (action === "remove") {
    removeTarget.value = targetKey as string
    showRemoveModal.value = true
  }
}

function doAddProvider() {
  const name = newProviderName.value.trim()
  if (!name) return
  if (!providerIds.value.includes(name)) providerIds.value.push(name)
  if (!providers[name]) providers[name] = { providerId: name, wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT }
  newProviderName.value = ""
  showAddModal.value = false
  setEditing(name)
}

function doRemoveProvider() {
  const id = removeTarget.value
  if (!id) return
  const idx = providerIds.value.indexOf(id)
  if (idx > -1) providerIds.value.splice(idx, 1)
  delete providers[id]
  showRemoveModal.value = false
  removeTarget.value = ""
  if (editingId.value === id) {
    editingId.value = providerIds.value[0] || ""
    loadForm(editingId.value)
  }
}

function resetForm() {
  Object.assign(formState, { model: "", baseUrl: "", apiKey: "", wireApi: "anthropic", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT })
  if (editingId.value) providers[editingId.value] = { ...formState, providerId: editingId.value }
  const m = PRESETS.find((p) => p.values.wireApi === formState.wireApi)
  if (m) activePresetKey.value = m.key
}

function onWireApiChange(key: string | number) {
  const preset = PRESETS.find((p) => p.key === key)
  if (!preset) return
  formState.wireApi = preset.values.wireApi; formState.apiKeyHeader = preset.values.apiKeyHeader
  snapshotEditing()
}

function onWireApiSelectChange(value: string) {
  formState.wireApi = value
  const m = PRESETS.find((p) => p.values.wireApi === value)
  if (m) { activePresetKey.value = m.key; formState.apiKeyHeader = m.values.apiKeyHeader }
}

async function testConnection() {
  if (!formState.baseUrl) { message.warning(t("config.form.test_need_url")); return }
  testingConn.value = true; connResult.value = null
  try {
    const r = await testProviderConnectivity(formState.baseUrl, formState.apiKey || "", formState.wireApi || "anthropic", formState.apiKeyHeader || undefined)
    connResult.value = r
  } catch (e: any) { connResult.value = { ok: false, status: 0, latency_ms: 0, message: t("config.form.test_error") + ": " + (e?.message || String(e)) } }
  finally { testingConn.value = false }
}

function parseConfig(text: string) {
  providerIds.value = []
  for (const k of Object.keys(providers)) delete providers[k]
  if (!text) { activeId.value = ""; editingId.value = ""; return }

  const lines = text.split("\n")
  let cur = "", inProv = false, activeFromTop = ""

  for (const line of lines) {
    const t = line.trim()
    if (!t || t.startsWith("#")) continue
    if (t.startsWith("provider = ")) { activeFromTop = t.replace("provider = ", "").replace(/"/g, "") }
    else if (t.startsWith("[providers.")) {
      cur = t.replace("[providers.", "").replace("]", ""); inProv = true
      if (!providerIds.value.includes(cur)) providerIds.value.push(cur)
      if (!providers[cur]) providers[cur] = { providerId: cur, wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT }
    } else if (t.startsWith("[")) { inProv = false }
    else if (inProv) {
      const p = providers[cur]; if (!p) continue
      if (t.startsWith("wire_api = ")) p.wireApi = t.replace("wire_api = ", "").replace(/"/g, "")
      else if (t.startsWith("base_url = ")) p.baseUrl = t.replace("base_url = ", "").replace(/"/g, "")
      else if (t.startsWith("model = ")) p.model = t.replace("model = ", "").replace(/"/g, "")
      else if (t.startsWith("api_key = ")) p.apiKey = t.replace("api_key = ", "").replace(/"/g, "")
      else if (t.startsWith("api_key_header = ")) p.apiKeyHeader = t.replace("api_key_header = ", "").replace(/"/g, "")
      else if (t.startsWith("model_context_window = ")) { const v = parseInt(t.replace("model_context_window = ", "")); if (!isNaN(v)) p.modelContextWindow = v }
      else if (t.startsWith("model_auto_compact_token_limit = ")) { const v = parseInt(t.replace("model_auto_compact_token_limit = ", "")); if (!isNaN(v)) p.modelAutoCompactLimit = v }
    }
  }

  activeId.value = activeFromTop && providers[activeFromTop] ? activeFromTop : (providerIds.value[0] || "")
  editingId.value = activeId.value; loadForm(editingId.value)
}

function buildConfig(): string {
  snapshotEditing()
  const active = activeId.value
  const blocks: string[] = ["# evocode bridge config", "# Read by evocode-cli, not by upstream Codex directly.", "", "provider = \"" + active + "\"", ""]
  for (const id of providerIds.value) {
    const p = providers[id] || { providerId: id, wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT }
    blocks.push("[providers." + id + "]", "wire_api = \"" + p.wireApi + "\"", "base_url = \"" + p.baseUrl + "\"", "model = \"" + p.model + "\"", "api_key = \"" + p.apiKey + "\"", "api_key_header = \"" + p.apiKeyHeader + "\"", "model_context_window = " + (p.modelContextWindow || DEFAULT_CONTEXT_WINDOW), "model_auto_compact_token_limit = " + (p.modelAutoCompactLimit || DEFAULT_COMPACT_LIMIT), "")
  }
  return blocks.join("\n").replace(/\n+$/, "\n")
}

async function handleSyncToCodex(providerId: string) {
  if (!providerId || syncing.value) return
  syncing.value = true
  try {
    const cfg = buildConfig()
    await writeConfig(cfg)
    await syncToCodex()
    message.success(t("config.sync.done") + " " + providerId, 3)
  } catch (e: any) {
    message.error(t("config.sync.failed") + ": " + (e?.message || String(e)), 4)
  } finally {
    syncing.value = false
  }
}

defineExpose({ activeId, providerIds, buildConfig, parseConfig, setActive: (id: string) => { activeId.value = id } })

onMounted(async () => {
  try {
    parseConfig(await readConfig())
  } catch (e) {
    console.error("Failed to load config in ConnectionPanel", e)
  }
})
</script>

<style scoped>
.connection-panel { display: flex; flex-direction: column; gap: 14px; }
.panel { padding: 20px 22px; display: flex; flex-direction: column; gap: 14px; }
.panel-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.panel-title { font-size: 15px; font-weight: 600; color: var(--text-1); }
.panel-sub { font-size: 12.5px; margin-top: 2px; }
.active-tag { border-radius: 999px; }
.active-with-sync { display: inline-flex; align-items: center; gap: 6px; }
.head-actions { display: inline-flex; gap: 8px; align-items: center; }
.empty-block { padding: 24px 0; }
.conn-alert { margin-bottom: 16px; }
.empty-tabs-hint { padding: 16px 0; color: var(--text-3); font-size: 13px; text-align: center; }
.add-btn-row { display: flex; gap: 8px; }
.prov-tabs { margin-top: 4px; }
.prov-tabs :deep(.ant-tabs-nav) { margin-bottom: 0; }
.prov-tabs :deep(.ant-tabs-tab) { font-size: 13px; padding: 4px 12px; }
.form :deep(.ant-form-item-label > label) { color: var(--text-2); font-size: 12.5px; }
.opt-row { display: inline-flex; align-items: center; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.blue { background: #60a5fa; box-shadow: 0 0 8px #60a5fa; }
.dot.purple { background: #a78bfa; box-shadow: 0 0 8px #a78bfa; }
.dot.cyan { background: #22d3ee; box-shadow: 0 0 8px #22d3ee; }
.slider-block { display: flex; flex-direction: column; gap: 12px; }
.slider-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 12px; }
.slider-label { font-size: 12.5px; color: var(--text-3); margin-bottom: 2px; }
.slider-value { font-size: 22px; font-weight: 700; color: var(--text-1); }
.compact-tag { color: var(--text-2); }
.slider-rail { position: relative; height: 12px; border-radius: 999px; background: linear-gradient(90deg, var(--bg-elev-3), var(--bg-elev-2)); border: 1px solid var(--border); margin: 22px 6px 12px; cursor: pointer; user-select: none; touch-action: none; }
.slider-fill { position: absolute; left: 0; top: 0; bottom: 0; background: linear-gradient(90deg, #4d7dff, #8b5cf6); border-radius: 999px; box-shadow: 0 0 12px rgba(77,125,255,0.45); transition: width .08s linear; }
.slider-fill.compact { background: linear-gradient(90deg, #34d399, #22d3ee); box-shadow: 0 0 10px rgba(52,211,153,0.35); }
.slider-thumb { position: absolute; top: 50%; width: 22px; height: 22px; border-radius: 50%; background: white; border: 3px solid #4d7dff; transform: translate(-50%, -50%); box-shadow: 0 0 14px rgba(77,125,255,0.55); cursor: grab; transition: left .08s linear; z-index: 2; }
.slider-thumb:hover { box-shadow: 0 6px 18px rgba(77,125,255,0.65); }
.slider-thumb:active { cursor: grabbing; transform: translate(-50%, -50%) scale(1.08); }
.thumb-tip { position: absolute; bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); padding: 2px 8px; border-radius: 6px; background: var(--bg-elev-3); border: 1px solid var(--border-strong); color: var(--text-1); font-size: 11px; white-space: nowrap; font-weight: 600; pointer-events: none; opacity: 0; transition: opacity .15s ease; }
.slider-thumb:hover .thumb-tip, .slider-thumb:active .thumb-tip { opacity: 1; }
.slider-tick { position: absolute; top: -4px; width: 2px; height: 16px; background: var(--border-strong); transform: translateX(-50%); border-radius: 2px; }
.slider-stops { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-top: 4px; }
.slider-stop { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 10px 12px; border-radius: var(--r-md); background: var(--bg-elev-2); border: 1px solid var(--border); color: var(--text-2); cursor: pointer; transition: border-color .15s ease, color .15s ease, transform .15s ease; }
.slider-stop:hover { border-color: var(--border-strong); color: var(--text-1); transform: translateY(-1px); }
.slider-stop.active { color: var(--text-1); border-color: rgba(77,125,255,0.5); background: linear-gradient(135deg, rgba(77,125,255,0.12), rgba(139,92,246,0.06)); box-shadow: 0 0 0 1px rgba(77,125,255,0.18); }
.stop-dot { width: 8px; height: 8px; border-radius: 50%; box-shadow: 0 0 6px currentColor; }
.stop-name { font-size: 12px; }
.stop-label { font-size: 13px; font-weight: 700; color: var(--text-1); }
</style>



