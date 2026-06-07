<template>
  <div class="config-view">
    <header class="page-head fade-up">
      <div>
        <div class="eyebrow">
          <span class="dot" />
          <span>Settings</span>
        </div>
        <h1>Configuration</h1>
        <p class="muted-3">Choose a provider, fill in credentials, and save. The bridge uses this to talk to your upstream API.</p>
      </div>
      <div class="head-actions">
        <a-button @click="$router.push('/')" class="ghost">
          <template #icon><LeftOutlined /></template>
          Back to Dashboard
        </a-button>
      </div>
    </header>

    <a-alert
      v-if="msg.text"
      :type="msg.type"
      :message="msg.text"
      show-icon
      closable
      class="fade-up alert"
      @close="msg.text = ''"
    />

    <a-tabs v-model:activeKey="activeKey" class="config-tabs fade-up">
      <!-- Provider tab -->
      <a-tab-pane key="provider" tab="Provider">
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Select or create a provider</div>
              <div class="panel-sub muted-3">Pick an existing provider or add a new one. Quick start templates are available below.</div>
            </div>
            <a-tag v-if="formState.providerId" class="active-tag">Active: {{ formState.providerId }}</a-tag>
          </div>

          <a-form layout="vertical" class="form">
            <a-form-item label="Active Provider">
              <div class="provider-row">
                <a-select
                  v-model:value="formState.providerId"
                  placeholder="Choose a provider..."
                  class="provider-select"
                  @change="onProviderChange"
                >
                  <a-select-option value="">-- Choose --</a-select-option>
                  <a-select-option v-for="id in providerIds" :key="id" :value="id">{{ id }}</a-select-option>
                </a-select>
                <a-input
                  v-model:value="newProviderName"
                  placeholder="New provider name"
                  class="new-name"
                  @press-enter="addProvider"
                />
                <a-button type="primary" :disabled="!newProviderName" @click="addProvider">
                  <template #icon><PlusOutlined /></template>
                  Add
                </a-button>
              </div>
            </a-form-item>

            <div class="presets">
              <span class="presets-label">Quick start templates</span>
              <a-button size="small" class="preset" @click="applyPreset('openai_chat')">
                <span class="dot blue" /> OpenAI Chat
              </a-button>
              <a-button size="small" class="preset" @click="applyPreset('anthropic')">
                <span class="dot purple" /> Anthropic Messages
              </a-button>
            </div>
          </a-form>
        </div>
      </a-tab-pane>

      <!-- Connection tab -->
      <a-tab-pane key="connection" tab="Connection">
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Connection details</div>
              <div class="panel-sub muted-3">Tell the bridge how to reach the upstream API.</div>
            </div>
            <a-tooltip v-if="formState.providerId" title="Remove this provider">
              <a-popconfirm title="Remove this provider?" ok-text="Yes" cancel-text="No" @confirm="removeProvider">
                <a-button danger size="small">
                  <template #icon><DeleteOutlined /></template>
                  Remove provider
                </a-button>
              </a-popconfirm>
            </a-tooltip>
          </div>

          <a-empty
            v-if="!formState.providerId"
            description="Select or create a provider first."
            class="empty-block"
          />

          <a-form
            v-else
            layout="vertical"
            class="form"
            :model="formState"
          >
            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item label="Model" required>
                  <a-input v-model:value="formState.model" placeholder="e.g. MiniMax-M2.7, gpt-4.1" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item label="Wire API">
                  <a-tooltip title="Which API protocol this provider exposes: Anthropic, Chat Completions, or OpenAI Responses">
                    <a-select v-model:value="formState.wireApi">
                      <a-select-option value="anthropic">
                        <span class="opt-row"><span class="dot purple" /> Anthropic (/v1/messages)</span>
                      </a-select-option>
                      <a-select-option value="chat_completions">
                        <span class="opt-row"><span class="dot blue" /> Chat Completions (/v1/chat)</span>
                      </a-select-option>
                      <a-select-option value="openai">
                        <span class="opt-row"><span class="dot cyan" /> OpenAI Responses (/responses)</span>
                      </a-select-option>
                    </a-select>
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>

            <a-form-item label="Base URL" required>
              <a-input v-model:value="formState.baseUrl" placeholder="https://api.example.com/v1" />
            </a-form-item>

            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item label="API Key">
                  <a-input-password v-model:value="formState.apiKey" placeholder="Your API key" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item label="API Key Header">
                  <a-tooltip title="The HTTP header name for the API key, e.g. X-Api-Key or Authorization">
                    <a-input v-model:value="formState.apiKeyHeader" placeholder="X-Api-Key" />
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>
          </a-form>
        </div>
      </a-tab-pane>

      <!-- Limits tab -->
      <a-tab-pane key="limits" tab="Limits">
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Model limits</div>
              <div class="panel-sub muted-3">Control context size and auto-compaction thresholds.</div>
            </div>
          </div>

          <a-form layout="vertical" class="form">
            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item label="Context Window">
                  <a-input-number
                    v-model:value="formState.contextWindow"
                    :min="1"
                    :step="1000"
                    class="full"
                  />
                  <div class="hint muted-3">Max context size for the model.</div>
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item label="Auto Compact Limit">
                  <a-input-number
                    v-model:value="formState.compactLimit"
                    :min="1"
                    :step="1000"
                    class="full"
                  />
                  <div class="hint muted-3">Start compacting when tokens exceed this.</div>
                </a-form-item>
              </a-col>
            </a-row>
          </a-form>
        </div>
      </a-tab-pane>
    </a-tabs>

    <footer class="actions fade-up">
      <a-button
        type="primary"
        size="large"
        :loading="saving"
        :disabled="!formState.providerId"
        @click="handleSave"
      >
        <template #icon><SaveOutlined /></template>
        Save Config
      </a-button>
      <a-button
        size="large"
        :disabled="!formState.providerId"
        @click="handleSyncToCodex"
      >
        <template #icon><SyncOutlined /></template>
        Sync to Codex
      </a-button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from "vue"
import {
  LeftOutlined,
  DeleteOutlined,
  PlusOutlined,
  SaveOutlined,
  SyncOutlined,
} from "@ant-design/icons-vue"
import { readConfig, writeConfig, syncToCodex } from "../api/bridge"

interface FormState {
  providerId: string
  contextWindow: number
  compactLimit: number
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
}

const PRESETS: Record<string, Partial<FormState>> = {
  openai_chat: {
    wireApi: "chat_completions",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1",
    apiKeyHeader: "Authorization",
  },
  anthropic: {
    wireApi: "anthropic",
    baseUrl: "https://api.anthropic.com",
    model: "claude-3-5-sonnet-latest",
    apiKeyHeader: "X-Api-Key",
  },
}

const DEFAULT_CONTEXT_WINDOW = 256000
const DEFAULT_COMPACT_LIMIT = 220000

const activeKey = ref("provider")
const providerIds = ref<string[]>([])
const newProviderName = ref("")
const saving = ref(false)
const msg = reactive({ text: "", type: "success" as "success" | "error" | "warning" })

const formState = reactive<FormState>({
  providerId: "",
  contextWindow: DEFAULT_CONTEXT_WINDOW,
  compactLimit: DEFAULT_COMPACT_LIMIT,
  wireApi: "anthropic",
  baseUrl: "",
  model: "",
  apiKey: "",
  apiKeyHeader: "X-Api-Key",
})

function applyPreset(name: string) {
  const preset = PRESETS[name]
  if (!preset) return
  if (!providerIds.value.includes(name)) providerIds.value.push(name)
  formState.providerId = name
  Object.assign(formState, preset)
  activeKey.value = "connection"
}

function addProvider() {
  const name = newProviderName.value.trim()
  if (name && !providerIds.value.includes(name)) {
    providerIds.value.push(name)
    formState.providerId = name
    newProviderName.value = ""
    onProviderChange()
    activeKey.value = "connection"
  }
}

function removeProvider() {
  const idx = providerIds.value.indexOf(formState.providerId)
  if (idx > -1) providerIds.value.splice(idx, 1)
  formState.providerId = ""
  onProviderChange()
  activeKey.value = "provider"
}

function onProviderChange() {
  formState.model = ""
  formState.baseUrl = ""
  formState.apiKey = ""
  formState.wireApi = "anthropic"
  formState.apiKeyHeader = "X-Api-Key"
}

function parseConfig(text: string) {
  providerIds.value = []
  if (!text) return

  const lines = text.split("\n")
  let currentProvider = ""
  let inProviderSection = false

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith("#")) continue

    if (trimmed.startsWith("provider = ")) {
      formState.providerId = trimmed.replace("provider = ", "").replace(/"/g, "")
    } else if (trimmed.startsWith("model_context_window = ")) {
      const val = parseInt(trimmed.replace("model_context_window = ", ""))
      if (!isNaN(val)) formState.contextWindow = val
    } else if (trimmed.startsWith("model_auto_compact_token_limit = ")) {
      const val = parseInt(trimmed.replace("model_auto_compact_token_limit = ", ""))
      if (!isNaN(val)) formState.compactLimit = val
    } else if (trimmed.startsWith("[providers.")) {
      currentProvider = trimmed.replace("[providers.", "").replace("]", "")
      inProviderSection = true
      if (!providerIds.value.includes(currentProvider)) {
        providerIds.value.push(currentProvider)
      }
    } else if (trimmed.startsWith("[") && trimmed !== "[providers." + currentProvider + "]") {
      inProviderSection = false
    } else if (inProviderSection && currentProvider === formState.providerId) {
      if (trimmed.startsWith("wire_api = ")) {
        formState.wireApi = trimmed.replace("wire_api = ", "").replace(/"/g, "")
      } else if (trimmed.startsWith("base_url = ")) {
        formState.baseUrl = trimmed.replace("base_url = ", "").replace(/"/g, "")
      } else if (trimmed.startsWith("model = ")) {
        formState.model = trimmed.replace("model = ", "").replace(/"/g, "")
      } else if (trimmed.startsWith("api_key = ")) {
        formState.apiKey = trimmed.replace("api_key = ", "").replace(/"/g, "")
      } else if (trimmed.startsWith("api_key_header = ")) {
        formState.apiKeyHeader = trimmed.replace("api_key_header = ", "").replace(/"/g, "")
      }
    }
  }
}

function buildConfig(): string {
  const ctx = formState.contextWindow || DEFAULT_CONTEXT_WINDOW
  const compact = formState.compactLimit || DEFAULT_COMPACT_LIMIT
  return [
    "# evocode bridge config",
    "# Read by evocode-cli, not by upstream Codex directly.",
    "",
    `provider = "${formState.providerId}"`,
    "",
    `model_context_window = ${ctx}`,
    `model_auto_compact_token_limit = ${compact}`,
    "",
    `[providers.${formState.providerId}]`,
    `wire_api = "${formState.wireApi}"`,
    `base_url = "${formState.baseUrl}"`,
    `model = "${formState.model}"`,
    `api_key = "${formState.apiKey}"`,
    `api_key_header = "${formState.apiKeyHeader}"`,
  ].join("\n")
}

async function handleSave() {
  if (!formState.providerId) {
    msg.text = "Please select or add a provider first."
    msg.type = "error"
    return
  }
  if (!formState.model || !formState.baseUrl) {
    msg.text = "Please fill in Model and Base URL for the provider."
    msg.type = "error"
    return
  }
  saving.value = true
  try {
    await writeConfig(buildConfig())
    msg.text = "Config saved! Restart the bridge to apply changes."
    msg.type = "success"
    setTimeout(() => { msg.text = "" }, 4000)
  } catch (e: any) {
    msg.text = "Failed to save: " + (e?.message || String(e))
    msg.type = "error"
  } finally {
    saving.value = false
  }
}

async function handleSyncToCodex() {
  if (!formState.providerId) {
    msg.text = "Please select a provider first."
    msg.type = "error"
    return
  }
  saving.value = true
  try {
    await syncToCodex(
      formState.providerId,
      formState.model,
      formState.baseUrl,
      formState.apiKey,
      formState.apiKeyHeader,
      formState.wireApi,
    )
    msg.text = "Synced to .codex/config.toml!"
    msg.type = "success"
    setTimeout(() => { msg.text = "" }, 4000)
  } catch (e: any) {
    msg.text = "Failed to sync: " + (e?.message || String(e))
    msg.type = "error"
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    const text = await readConfig()
    parseConfig(text)
  } catch {}
})
</script>

<style scoped>
.config-view { display: flex; flex-direction: column; gap: 18px; }

.page-head {
  display: flex; align-items: flex-end; justify-content: space-between;
  gap: 16px; flex-wrap: wrap;
}
.eyebrow {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 4px 10px; border-radius: 999px;
  background: var(--bg-elev-3); color: var(--text-3); font-size: 12px; width: max-content;
  border: 1px solid var(--border);
}
.eyebrow .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--info); box-shadow: 0 0 8px var(--info); }

.page-head h1 { font-size: 24px; font-weight: 700; color: var(--text-1); margin: 6px 0 2px; }
.page-head p { color: var(--text-3); max-width: 60ch; }
.head-actions { display: flex; gap: 8px; }
.ghost { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.ghost:hover { border-color: var(--border-strong); }

.alert { border-radius: var(--r-md); }

.config-tabs :deep(.ant-tabs-nav) { margin-bottom: 14px; }
.config-tabs :deep(.ant-tabs-tab) { padding: 8px 4px; color: var(--text-3); }
.config-tabs :deep(.ant-tabs-tab:hover) { color: var(--text-1); }
.config-tabs :deep(.ant-tabs-tab-active .ant-tabs-tab-btn) { color: var(--text-1) !important; }
.config-tabs :deep(.ant-tabs-ink-bar) { background: linear-gradient(90deg, #4d7dff, #8b5cf6); height: 3px; border-radius: 2px; }

.panel { padding: 20px 22px; }
.panel-head {
  display: flex; align-items: flex-start; justify-content: space-between;
  gap: 12px; margin-bottom: 14px;
}
.panel-title { font-size: 15px; font-weight: 600; color: var(--text-1); }
.panel-sub { font-size: 12.5px; margin-top: 2px; }
.active-tag { border-radius: 999px; }

.form :deep(.ant-form-item-label > label) { color: var(--text-2); font-size: 12.5px; }

.provider-row { display: flex; gap: 8px; align-items: stretch; flex-wrap: wrap; }
.provider-select { flex: 1 1 220px; min-width: 200px; }
.new-name { width: 180px; }

.presets {
  display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
  margin-top: 4px; padding: 12px; border-radius: var(--r-md);
  background: var(--bg-elev-2); border: 1px dashed var(--border-strong);
}
.presets-label { font-size: 12px; color: var(--text-3); margin-right: 4px; }
.preset { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.preset:hover { border-color: var(--border-strong); color: var(--text-1); }

.opt-row { display: inline-flex; align-items: center; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.blue { background: #60a5fa; box-shadow: 0 0 8px #60a5fa; }
.dot.purple { background: #a78bfa; box-shadow: 0 0 8px #a78bfa; }
.dot.cyan { background: #22d3ee; box-shadow: 0 0 8px #22d3ee; }
.dot.green { background: var(--ok); box-shadow: 0 0 8px var(--ok); }

.empty-block { padding: 24px 0; }

.full { width: 100%; }
.hint { font-size: 11.5px; margin-top: 4px; }

.actions {
  position: sticky; bottom: 16px; z-index: 5;
  display: flex; gap: 10px; justify-content: flex-end; flex-wrap: wrap;
  padding: 12px 16px; border-radius: var(--r-lg);
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  -webkit-backdrop-filter: blur(14px) saturate(140%);
  box-shadow: var(--shadow-md);
}

@media (max-width: 720px) {
  .actions { justify-content: stretch; }
  .actions .ant-btn { flex: 1; }
}
</style>
