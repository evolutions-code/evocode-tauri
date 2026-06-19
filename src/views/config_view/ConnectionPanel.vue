<template>
  <div class="connection-panel">
    <div style="display:flex;align-items:center;gap:12px;flex-wrap:wrap;padding:4px 0;margin-bottom:8px;">
      <div style="display:flex;align-items:center;gap:6px;">
        <span style="font-size:13px;font-weight:500;">{{ t("config.default_model") }}:</span>
        <a-select v-model:value="defaultModelRef" style="width:280px;" :placeholder="t('config.default_model_placeholder')" allow-clear @change="onDefaultModelChange">
          <a-select-option v-for="opt in allModelOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</a-select-option>
        </a-select>
      </div>
      <a-button type="primary" :loading="syncing" @click="handleSyncAll">
        <template #icon><SyncOutlined /></template>
        {{ t("config.sync_all") }}
      </a-button>
    </div>
    <div class="glass panel">
      <div class="panel-head">
        <div style="display: inline-flex; align-items: center; gap: 10px;">
          <div class="panel-title">{{ t("config.providers") }}</div>
          <a-button size="small" type="dashed" @click="showAddModal = true">
            <template #icon><PlusOutlined /></template>
            {{ t("config.providers.add") }}
          </a-button>
        </div>
      </div>

      <a-tabs
        v-if="providerIds.length"
        type="editable-card"
        hideAdd
        :activeKey="editingId"
        @change="onTabChange"
        @edit="onTabEdit"
        class="prov-tabs"
        destroyInactiveTabPane
      >
        <a-tab-pane
          v-for="(item, key) in providers"
          :key="key"
          :tab="key"
        >
          <div class="tab-body">
            <!-- Provider config form -->
            <div class="tab-section">
              <div class="tab-section-head">
                <div class="tab-section-title">{{ t("config.form.title") }}</div>
                <a-button size="small" @click="resetForm(key)">
                  <template #icon><ReloadOutlined /></template> {{ t("config.form.reset") }}
                </a-button>
              </div>
              <a-alert v-if="connResult" class="conn-alert"
                :type="connResult.ok ? 'success' : 'error'"
                :message="connResult.message"
                :description="t('config.form.test_latency') + ': ' + connResult.latency_ms + ' ms'"
                show-icon closable @close="connResult = null" />
              <a-form layout="vertical" class="form" :model="item">
                <a-form-item :label="t('config.form.base_url')" required>
                  <a-input v-model:value="item.baseUrl" :placeholder="t('config.form.base_url_placeholder')" />
                </a-form-item>
                
                <a-row :gutter="16">
                  <a-col :span="12">                   <a-form-item :label="t('config.form.wire_api')">
                      <a-tooltip :title="t('config.wire_api.tooltip')">
                        <a-select v-model:value="item.wireApi" @change="(val: string) => onWireApiSelectChange(key, val)">
                          <a-select-option value="anthropic"><span class="opt-row"><span class="dot purple" /> {{ t("config.wire.anthropic") }}</span></a-select-option>
                          <a-select-option value="chat_completions"><span class="opt-row"><span class="dot blue" /> {{ t("config.wire.chat") }}</span></a-select-option>
                          <a-select-option value="openai"><span class="opt-row"><span class="dot cyan" /> {{ t("config.wire.openai") }}</span></a-select-option>
                        </a-select>
                      </a-tooltip>
                    </a-form-item>
                  </a-col>
                </a-row>
<a-row :gutter="16">
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.api_key')">
                      <a-input-password v-model:value="item.apiKey" :placeholder="t('config.form.api_key_placeholder')" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.api_key_header')">
                      <a-tooltip :title="t('config.form.api_key_header_tooltip')">
                        <a-input v-model:value="item.apiKeyHeader" :placeholder="t('config.form.api_key_header_placeholder')" />
                      </a-tooltip>
                    </a-form-item>
                  </a-col>
                </a-row>
              </a-form>
            </div>

                        <!-- Model list table -->
            <div class="model-list-section">
              <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px;">
                <strong style="font-size:14px;">{{ t("config.models") }}</strong>
                <span style="display:flex;gap:6px;align-items:center;">
                  <span class="muted-3" style="font-size:12px;">{{ t("config.models_count", { count: String((item.models || []).length) }) }}</span>
                  <a-button size="small" :loading="fetchingModels[key]" @click="doFetchModels(key)">
                    <template #icon><DownloadOutlined /></template>
                  </a-button>
                  <a-button size="small" @click="addModel(key)">
                    <template #icon><PlusOutlined /></template>
                  </a-button>
                </span>
              </div>
              <div v-if="!(item.models || []).length" style="padding:8px 0;">
                <div class="muted-3" style="font-size:13px;">{{ t("config.no_models") }}</div>
              </div>
              <div v-for="(model, mi) in item.models || []" :key="mi" class="model-card">
                <div class="model-card-header">
                  <a-input v-model:value="model.name" class="model-name-input" :placeholder="t('config.model_name_placeholder')" />
                  <a-button size="small" danger type="text" @click="removeModel(key, mi)" class="model-remove-btn">&times;</a-button>
                </div>
                <div class="model-card-body">
                  <div class="model-slider-section">
                    <div class="slider-label-row">
                      <div class="slider-label-group">
                        <span class="slider-label">{{ t("config.context") }}</span>
                        <div class="preset-row-inline">
                          <a-button v-for="p in LIMIT_PRESETS" :key="p.key" size="small" :type="(model.contextWindow || 256000) === p.context ? 'primary' : 'default'" @click="model.contextWindow = p.context" class="preset-btn">{{ contextLabel(p.context) }}</a-button>
                        </div>
                      </div>
                      <span class="slider-value-label">{{ contextLabel(model.contextWindow || 256000) }}</span>
                    </div>
                    <a-slider :min="16000" :max="10000000" :step="1000" :value="model.contextWindow || 256000" @change="(v: number) => { model.contextWindow = v }" class="model-slider" />
                  </div>
                  <div class="model-slider-section">
                    <div class="slider-label-row">
                      <div class="slider-label-group">
                        <span class="slider-label">{{ t("config.compact") }}</span>
                        <div class="preset-row-inline">
                          <a-button v-for="pct in [50, 60, 70, 80, 90, 95]" :key="pct" size="small" :type="Math.round((model.autoCompactTokenLimit || Math.round((model.contextWindow || 256000) * 0.8)) / (model.contextWindow || 256000) * 100) === pct ? 'primary' : 'default'" @click="model.autoCompactTokenLimit = Math.round((model.contextWindow || 256000) * pct / 100)" class="preset-btn">{{ pct }}%</a-button>
                        </div>
                      </div>
                      <span class="slider-value-label">{{ contextLabel(model.autoCompactTokenLimit) }}</span>
                    </div>
                    <a-slider :min="16000" :max="model.contextWindow || 10000000" :step="1000" :value="model.autoCompactTokenLimit || Math.round((model.contextWindow || 256000) * 0.8)" @change="(v: number) => { model.autoCompactTokenLimit = v }" class="model-slider" />
                  </div>
                  <div class="model-slider-section">
                    <div class="slider-label-row">
                      <span class="slider-label">{{ t("config.modalities") }}</span>
                    </div>
                    <a-select mode="multiple" :value="model.inputModalities || ['text']" size="small" class="modality-select"
                      @change="(val: string[]) => { model.inputModalities = val.length ? val : ['text']; }">
                      <a-select-option value="text">{{ t("config.modality_text") }}</a-select-option>
                      <a-select-option value="image">{{ t("config.modality_image") }}</a-select-option>
                    </a-select>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </a-tab-pane>
      </a-tabs>
      <div v-if="!providerIds.length" class="empty-tabs-hint">{{ t("config.form.empty") }}</div>
      </div>


     <div class="actions-bar">
     <a-select v-if="providers[editingId]?.models?.length" v-model:value="testModelId" :placeholder="t('config.test_model_placeholder')" allow-clear style="width:180px;font-size:13px;">
       <a-select-option v-for="m in providers[editingId]?.models || []" :key="m.name" :value="m.name">{{ m.name }}</a-select-option>
     </a-select>
     <a-button :loading="testingConn" :disabled="testingConn || !testModelId" @click="testConnection(editingId)">
       <template #icon><ApiOutlined /></template>
       {{ t("config.test") }}
     </a-button>
      <a-button type="primary" :loading="saving" :disabled="saving" @click="handleSave">
          <template #icon><SaveOutlined /></template>
          {{ t("config.save") }}
        </a-button>
    </div>

    <a-modal
      v-model:open="showAddModal"
      :title="t('config.providers.add_title')"
      :ok-text="t('config.providers.ok')"
      :cancel-text="t('config.providers.cancel')"
      @ok="doAddProvider"
    >
      <a-input
        v-model:value="newProviderName"
        :placeholder="t('config.providers.placeholder')"
        @input="onProviderNameInput"
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
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue"
import { useLocale } from "../../composables/useLocale"
import { syncToCodex, syncModelToCodex, saveConfig, readConfigJson, testProviderConnectivity, fetchModels as fetchModelsApi } from "../../api/bridge"
import type { ProviderConfig } from "../../api/bridge"
import { message } from "ant-design-vue"
 import { PlusOutlined, ReloadOutlined, ApiOutlined, SaveOutlined, SyncOutlined, DownloadOutlined } from "@ant-design/icons-vue"

const { t } = useLocale()

interface ModelEntry {
  name: string
  contextWindow?: number
  autoCompactTokenLimit?: number
  inputModalities?: string[]
}

interface Provider {
  wireApi: string
  baseUrl: string
  apiKey: string
  apiKeyHeader: string
  models: ModelEntry[]
}

const LIMIT_PRESETS = [
  { key: "256k", context: 128000, compact: 100000 },
  { key: "512k", context: 256000, compact: 220000 },
  { key: "1m", context: 1000000, compact: 800000 },
  { key: "10m", context: 10000000, compact: 8000000 },
]

const PRESETS = [
  { key: "anthropic", values: { wireApi: "anthropic" as const, apiKeyHeader: "X-Api-Key" as const } },
  { key: "chat_completions", values: { wireApi: "chat_completions" as const, apiKeyHeader: "Authorization" as const } },
  { key: "openai", values: { wireApi: "openai" as const, apiKeyHeader: "Authorization" as const } },
]
const providerIds = ref<string[]>([])
const providers = reactive<Record<string, Provider>>({})
const activeId = ref("")
const editingId = ref("")
const newProviderName = ref("")
const showAddModal = ref(false)
const showRemoveModal = ref(false)
const removeTarget = ref("")
const wirePresetKey = reactive<Record<string, string>>({})
const testingConn = ref(false)
const saving = ref(false)
const syncing = ref(false)
const testModelId = undefined
const defaultModelRef = ref("")
const allModelOptions = computed(() => {
  const opts: { value: string; label: string }[] = []
  for (const id of providerIds.value) {
    const p = providers[id]
    if (!p || !p.models) continue
    for (const m of p.models) {
      if (m.name) opts.push({ value: id + ":" + m.name, label: id + ":" + m.name })
    }
  }
  return opts
})
const connResult = ref<null | { ok: boolean; status: number; latency_ms: number; message: string }>(null)


const modelOptions = reactive<Record<string, { value: string }[]>>({})
const fetchingModels = reactive<Record<string, boolean>>({})

// 获取要保存的model值: 优先使用defaultModelRef，如果为空则从active provider的第一个模型派生
function getModelForSave(): string | undefined {
  console.log("[evocode] getModelForSave: defaultModelRef.value =", JSON.stringify(defaultModelRef.value))
  return defaultModelRef.value || undefined
}

// Per-provider slider rail refs
function contextLabel(n: number | undefined | null) {
  if (!n || n <= 0) return "256K"
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(n % 1_000_000 === 0 ? 0 : 1).replace(/\.0$/, "") + "M"
  if (n >= 1_000) return Math.round(n / 1_000) + "K"
  return String(n)
}

function addModel(id: string) {
  const p = providers[id]
  if (!p) return
  p.models.push({ name: 'new-model', contextWindow: 256000, autoCompactTokenLimit: 200000, inputModalities: ['text'] } as ModelEntry)
}

function removeModel(id: string, mi: number) {
  const p = providers[id]
  if (!p) return
  p.models.splice(mi, 1)
}

async function onDefaultModelChange(val: string) {
  console.log("[evocode] onDefaultModelChange: val =", JSON.stringify(val), "| defaultModelRef.value =", JSON.stringify(defaultModelRef.value))
  if (!val) {
    console.log("[evocode] onDefaultModelChange: val is empty, skipping save")
    return
  }
  const provs: Record<string, any> = {}
  for (const id of providerIds.value) {
    const p = providers[id]
    if (p) provs[id] = { base_url: p.baseUrl, wire_api: p.wireApi, models: (p.models || []).map((m: any) => ({ name: m.name, context_window: m.contextWindow, auto_compact_token_limit: m.autoCompactTokenLimit, input_modalities: m.inputModalities })), api_key: p.apiKey, api_key_header: p.apiKeyHeader }
  }
  await saveConfig({ provider: activeId.value, model: val, providers: provs })
  try { await syncModelToCodex(val) } catch {}
}

function onWireApiSelectChange(id: string, value: string) {
  const p = providers[id]
  if (!p) return
  p.wireApi = value
  const m = PRESETS.find((pr) => pr.values.wireApi === value)
  if (m) {
    wirePresetKey[id] = m.key
    p.apiKeyHeader = m.values.apiKeyHeader
  }
}

async function testConnection(id: string) {
  const p = providers[id]
  if (!p || !p.baseUrl) { message.warning(t("config.form.test_need_url")); return }
  testingConn.value = true; connResult.value = null
  try {
    const r = await testProviderConnectivity(p.baseUrl, p.apiKey || "", p.wireApi || "anthropic", p.apiKeyHeader || undefined, testModelId.value || undefined)
    connResult.value = r
  } catch (e: any) {
    connResult.value = { ok: false, status: 0, latency_ms: 0, message: t("config.form.test_error") + ": " + (e?.message || String(e)) }
  } finally {
    testingConn.value = false
  }
}

async function doFetchModels(id: string) {
  const p = providers[id]
  if (!p || !p.baseUrl) { message.warning(t("config.form.test_need_url")); return }
  fetchingModels[id] = true
  try {
    const models = await fetchModelsApi(p.baseUrl, p.apiKey || "", p.wireApi || "anthropic", p.apiKeyHeader || undefined)
    modelOptions[id] = models.map((m: string) => ({ value: m }))
    // After merge, update modelOptions with all model names
    // Store fetched models into the provider (merge, preserving existing config)
    if (p) {
      for (const m of models) {
        const existing = p.models.findIndex(e => e.name === m)
        if (existing < 0) p.models.push({ name: m } as ModelEntry)
      }
    }

    if (models.length) message.success(t("config.form.fetch_models_success", { count: String(models.length) }), 3)
    else message.info(t("config.form.fetch_models_empty"), 3)
  } catch (e: any) {
    message.error(t("config.form.fetch_models_failed") + ": " + (e?.message || String(e)), 4)
  } finally {
    fetchingModels[id] = false
  }
}

function resetForm(id: string) {
  const p = providers[id]
  if (!p) return
  p.baseUrl = ""
  p.apiKey = ""
  p.wireApi = "anthropic"
  p.apiKeyHeader = "X-Api-Key"
  const m = PRESETS.find((x) => x.values.wireApi === p.wireApi)
  if (m) wirePresetKey[id] = m.key
  modelOptions[id] = []
}

function onTabChange(key: string) {
  editingId.value = key
}

function onTabEdit(targetKey: string | MouseEvent, action: string) {
  if (action === "remove") {
    removeTarget.value = targetKey as string
    showRemoveModal.value = true
  }
}

function onProviderNameInput() {
  newProviderName.value = newProviderName.value.replace(/[^a-zA-Z0-9]/g, '')
}

function doAddProvider() {
  const name = newProviderName.value.trim()
  if (!name) return
  if (!providerIds.value.includes(name)) providerIds.value.push(name)
  if (!providers[name]) {
    providers[name] = {
      wireApi: "anthropic", baseUrl: "", apiKey: "", apiKeyHeader: "X-Api-Key",
      models: []
    }
  }
  if (!wirePresetKey[name]) wirePresetKey[name] = "anthropic"
  newProviderName.value = ""
  modelOptions[name] = []
  showAddModal.value = false
  editingId.value = name
  const m = PRESETS.find((x) => x.values.wireApi === providers[name].wireApi)
  if (m) wirePresetKey[name] = m.key
}

async function doRemoveProvider() {
  const id = removeTarget.value
  if (!id) return
  const idx = providerIds.value.indexOf(id)
  if (idx > -1) providerIds.value.splice(idx, 1)
  delete providers[id]


  delete wirePresetKey[id]
  showRemoveModal.value = false
  delete modelOptions[id]
  removeTarget.value = ""
  if (editingId.value === id) {
    editingId.value = providerIds.value[0] || ""
  }
  try {
    const modelForSave = getModelForSave()
    console.log("[evocode] handleSave: defaultModelRef.value =", JSON.stringify(defaultModelRef.value), "| modelForSave =", JSON.stringify(modelForSave))
    const provs: Record<string, ProviderConfig> = {}
    for (const sid of providerIds.value) {
      const sp = providers[sid]
      if (sp) provs[sid] = { base_url: sp.baseUrl, wire_api: sp.wireApi, models: (sp.models || []).map((m: any) => ({ name: m.name, context_window: m.contextWindow, auto_compact_token_limit: m.autoCompactTokenLimit, input_modalities: m.inputModalities })), api_key: sp.apiKey, api_key_header: sp.apiKeyHeader }
    }
    await saveConfig({ provider: activeId.value, model: getModelForSave(), providers: provs })
  } catch {}
}

// parse / build / sync
function parseConfig(cfg: any) {
  providerIds.value = []
  for (const k of Object.keys(providers)) delete providers[k]
  for (const k of Object.keys(wirePresetKey)) delete wirePresetKey[k]
  for (const k of Object.keys(modelOptions)) delete modelOptions[k]
  if (!cfg || !cfg.providers) { activeId.value = ""; editingId.value = ""; return }

  for (const id of Object.keys(cfg.providers)) {
    const raw = cfg.providers[id]
    if (!raw || !raw.base_url) continue
    providerIds.value.push(id)
    providers[id] = {
      wireApi: raw.wire_api || "anthropic",
      baseUrl: raw.base_url || "",
      apiKey: raw.api_key || "",
      apiKeyHeader: raw.api_key_header || "X-Api-Key",
      models: (raw.models || []).map((m: any) => ({
        name: m.name || "",
        contextWindow: m.context_window || undefined,
        autoCompactTokenLimit: m.auto_compact_token_limit || undefined,
        inputModalities: m.input_modalities || ["text"]
      } as ModelEntry))
    }
    if (!wirePresetKey[id]) {
      const preset = PRESETS.find((pr: any) => pr.values.wireApi === (raw.wire_api || "anthropic"))
      wirePresetKey[id] = preset ? preset.key : "anthropic"
    }
  }

  activeId.value = cfg.provider || providerIds.value[0] || ""
  editingId.value = activeId.value
  defaultModelRef.value = cfg.model || ""
}
async function handleSave() {
  saving.value = true
  try {
    const modelForSave = getModelForSave()
    console.log("[evocode] handleSave: defaultModelRef.value =", JSON.stringify(defaultModelRef.value), "| modelForSave =", JSON.stringify(modelForSave))
    const provs: Record<string, ProviderConfig> = {}
    for (const id of providerIds.value) {
      const p = providers[id]
      provs[id] = {
        base_url: p.baseUrl,
        wire_api: p.wireApi,
        models: (p.models || []).map((m: any) => ({ name: m.name, context_window: m.contextWindow, auto_compact_token_limit: m.autoCompactTokenLimit, input_modalities: m.inputModalities })),
        api_key: p.apiKey,
        api_key_header: p.apiKeyHeader
      }
    }
    await saveConfig({ provider: activeId.value, model: getModelForSave(), providers: provs })
    message.success(t("config.saved"), 3)
  } catch (e: any) {
    message.error(t("config.save_failed") + ": " + (e?.message || String(e)), 4)
  } finally {
    saving.value = false
  }
}

async function handleSyncAll() {
  if (syncing.value) return
  syncing.value = true
  try {
    const modelForSave = getModelForSave()
    console.log("[evocode] handleSyncAll: defaultModelRef.value =", JSON.stringify(defaultModelRef.value), "| modelForSave =", JSON.stringify(modelForSave))
    const provs: Record<string, any> = {}
    for (const id of providerIds.value) {
      const p = providers[id]
      if (!p) continue
      provs[id] = {
        base_url: p.baseUrl,
        wire_api: p.wireApi,
        models: (p.models || []).map((m: any) => ({ name: m.name, context_window: m.contextWindow, auto_compact_token_limit: m.autoCompactTokenLimit, input_modalities: m.inputModalities })),
        api_key: p.apiKey,
        api_key_header: p.apiKeyHeader
      }
    }
    await saveConfig({ provider: activeId.value, model: modelForSave, providers: provs })
    await syncToCodex()
    message.success(t("config.synced_all"), 3)
  } catch (e: any) {
    console.error("[evocode] handleSyncAll error:", e)
    message.error(t("config.sync.failed") + ": " + (e?.message || String(e)), 4)
  } finally {
    syncing.value = false
  }
}

defineExpose({ activeId, providerIds, parseConfig, setActive: (id: string) => { activeId.value = id } })
onMounted(async () => {
  try {
  parseConfig(await readConfigJson())
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
.active-with-sync { display: flex; justify-content: flex-end; align-items: center; gap: 6px; }
.active-with-sync .active-tag {
  height: 32px;
  line-height: 30px;
  font-size: 13px;
}
.active-with-sync .ant-select {
  height: 32px;
}
.active-with-sync .ant-btn {
  height: 32px;
}
.head-actions { display: inline-flex; gap: 8px; align-items: center; }
.empty-block { padding: 24px 0; }
.conn-alert { margin-bottom: 16px; }
.empty-tabs-hint { padding: 16px 0; color: var(--text-3); font-size: 13px; text-align: center; }
.quick-add-section { display: flex; flex-direction: column; gap: 12px; padding: 8px 0; }
.quick-add-card {
  display: flex; align-items: flex-start; gap: 14px;
  padding: 16px; border-radius: var(--r-lg);
  background: linear-gradient(135deg, rgba(255,255,255,0.10), rgba(255,255,255,0.05));
  border: 1px solid rgba(255,255,255,0.25);
}
.quick-add-icon { font-size: 28px; line-height: 1; flex-shrink: 0; margin-top: 2px; }
.quick-add-content { display: flex; flex-direction: column; gap: 6px; flex: 1; }
.quick-add-title { font-weight: 600; font-size: 14px; color: var(--text-1); }
.quick-add-desc { font-size: 12px; color: var(--text-3); }
.quick-add-content .ant-btn { align-self: flex-start; }
.add-btn-row { display: flex; gap: 8px; margin-top: 8px; }
.prov-tabs { margin-top: 4px; }
.prov-tabs :deep(.ant-tabs-nav) { margin-bottom: 0; }
.prov-tabs :deep(.ant-tabs-tab) { font-size: 13px; padding: 4px 12px; }
 .form :deep(.ant-form-item-label > label) { color: var(--text-2); font-size: 12.5px; }
 .form :deep(.ant-input-affix-wrapper) {
   background: var(--bg-elev-3) !important;
   backdrop-filter: none;
   -webkit-backdrop-filter: none;
 }
 .form :deep(.ant-input) {
   background: transparent !important;
   backdrop-filter: none;
   -webkit-backdrop-filter: none;
 }
 .form :deep(.ant-input::placeholder) {
   opacity: 0.65 !important;
   color: var(--text-3) !important;
 }
.opt-row { display: inline-flex; align-items: center; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.blue { background: #808080; box-shadow: 0 0 8px #808080; }
.dot.purple { background: #a78bfa; box-shadow: 0 0 8px #a78bfa; }
.dot.cyan { background: #22d3ee; box-shadow: 0 0 8px #22d3ee; }
.slider-block { display: flex; flex-direction: column; gap: 12px; }
.slider-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 12px; }
.slider-label { font-size: 12.5px; color: var(--text-3); margin-bottom: 2px; }
.slider-value { font-size: 22px; font-weight: 700; color: var(--text-1); }
.compact-tag { color: var(--text-2); }
.slider-rail { position: relative; height: 12px; border-radius: 999px; background: linear-gradient(90deg, var(--bg-elev-3), var(--bg-elev-2)); border: 1px solid var(--border); margin: 22px 6px 12px; cursor: pointer; user-select: none; touch-action: none; }
 .slider-fill { position: absolute; left: 0; top: 0; bottom: 0; background: linear-gradient(90deg, #4f7cff, #22d3ee); border-radius: 999px; box-shadow: 0 0 10px rgba(79, 124, 255, 0.3); transition: width .08s linear; }
.slider-fill.compact { background: linear-gradient(90deg, #34d399, #22d3ee); box-shadow: 0 0 10px rgba(52,211,153,0.35); }
.slider-thumb { position: absolute; top: 50%; width: 22px; height: 22px; border-radius: 50%; background: white; border: 3px solid #ffffff; transform: translate(-50%, -50%); box-shadow: 0 0 10px rgba(255,255,255,0.15); cursor: grab; transition: left .08s linear; z-index: 2; }
.slider-thumb:hover { box-shadow: 0 6px 18px rgba(255,255,255,0.20); }
.slider-thumb:active { cursor: grabbing; transform: translate(-50%, -50%) scale(1.08); }
.thumb-tip { position: absolute; bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); padding: 2px 8px; border-radius: 6px; background: var(--bg-elev-3); border: 1px solid var(--border-strong); color: var(--text-1); font-size: 11px; white-space: nowrap; font-weight: 600; pointer-events: none; opacity: 0; transition: opacity .15s ease; }
.slider-thumb:hover .thumb-tip, .slider-thumb:active .thumb-tip { opacity: 1; }
.slider-tick { position: absolute; top: -4px; width: 2px; height: 16px; background: var(--border-strong); transform: translateX(-50%); border-radius: 2px; }
.slider-stops { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-top: 4px; }
.slider-stop { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 10px 12px; border-radius: var(--r-md); background: var(--bg-elev-2); border: 1px solid var(--border); color: var(--text-2); cursor: pointer; transition: border-color .15s ease, color .15s ease, transform .15s ease; }
.slider-stop:hover { border-color: var(--border-strong); color: var(--text-1); transform: translateY(-1px); }
.slider-stop.active { color: var(--text-1); border-color: rgba(255,255,255,0.5); background: linear-gradient(135deg, rgba(128,128,128,0.12), rgba(128,128,128,0.06)); box-shadow: 0 0 0 1px rgba(128,128,128,0.18); }
.stop-dot { width: 8px; height: 8px; border-radius: 50%; box-shadow: 0 0 6px currentColor; }
.stop-name { font-size: 12px; }
.stop-label { font-size: 13px; font-weight: 700; color: var(--text-1); }
.tab-body { padding: 18px 0 4px; display: flex; flex-direction: column; gap: 18px; }
.tab-section { display: flex; flex-direction: column; gap: 10px; }
.tab-section-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.tab-section-title { font-size: 14px; font-weight: 600; color: var(--text-1); }
.tab-section-sub { font-size: 12px; margin-top: 1px; }
.actions-bar {
  margin-top: 14px; position: sticky; bottom: 16px; z-index: 5;
  display: flex; justify-content: flex-end; gap: 10px;
  padding: 12px 16px; border-radius: var(--r-lg);
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  box-shadow: var(--shadow-md);
}

/* Save button: gray when disabled */
.actions-bar :deep(.ant-btn-primary[disabled]) {
  background: rgba(79,124,255,0.25) !important;
  border-color: rgba(79,124,255,0.25) !important;
  color: rgba(255,255,255,0.35) !important;
}
.actions-bar :deep(.ant-btn-primary[disabled]:hover) {
  background: rgba(79,124,255,0.25) !important;
  border-color: rgba(79,124,255,0.25) !important;
}
/* ---- Model Card Styles ---- */
.model-card {
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 14px 16px;
  margin-bottom: 10px;
  background: var(--bg-elev-1);
  transition: border-color 0.2s;
}
.model-card:hover {
  border-color: var(--border-strong);
}
.model-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
}
.model-name-input {
  flex: 1;
}
.model-name-input :deep(.ant-input) {
  font-weight: 600 !important;
  font-size: 14px !important;
}
.model-remove-btn {
  flex-shrink: 0;
  font-size: 18px;
  line-height: 1;
  opacity: 0.6;
  transition: opacity 0.15s;
}
.model-remove-btn:hover {
  opacity: 1;
}
.model-card-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.model-slider-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.slider-label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}
.slider-label-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  flex: 1;
}
.preset-row-inline {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.slider-label {
  font-size: 12px;
  color: var(--text-3);
  font-weight: 500;
}
.slider-value-label {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-1);
}
.model-slider {
  margin: 0 2px;
}
.preset-row {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 2px;
}
.preset-btn {
  font-size: 11px;
  height: 24px;
  padding: 0 10px;
  border-radius: 4px;
}
.modality-select {
  width: 100%;
}
</style>
