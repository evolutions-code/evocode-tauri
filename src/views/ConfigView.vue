<template>
  <div class="config-view">
    <header class="header">
      <router-link to="/" class="back-btn">[&lt;] Back</router-link>
      <h2>Configuration</h2>
    </header>

    <div class="config-form">
      <h3>General</h3>
      <div class="form-row">
        <div class="form-group">
          <label>Provider</label>
          <div class="add-provider-row">
            <select v-model="providerId">
              <option value="">-- Select --</option>
              <option v-for="id in providerIds" :key="id" :value="id">{{ id }}</option>
            </select>
            <input v-model="newProviderName" type="text" placeholder="New provider name" class="new-provider-input" />
            <button type="button" class="btn-add" @click="addProvider" :disabled="!newProviderName">Add</button>
          </div>
        </div>
        <div class="form-group">
          <label>Model</label>
          <input v-model="model" type="text" placeholder="e.g. gpt-4.1" />
        </div>
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Context Window</label>
          <input v-model="contextWindow" type="number" placeholder="256000" />
        </div>
        <div class="form-group">
          <label>Auto Compact Limit</label>
          <input v-model="compactLimit" type="number" placeholder="220000" />
        </div>
      </div>

      <template v-if="providerId">
        <h3>Provider: {{ providerId }}</h3>
        <div class="form-group full">
          <label>Base URL</label>
          <input v-model="baseUrl" type="text" placeholder="https://api.example.com" />
        </div>
        <div class="form-group full">
          <label>API Key</label>
          <input v-model="apiKey" type="password" placeholder="Your API key" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label>Wire API</label>
            <select v-model="wireApi">
              <option value="anthropic">Anthropic</option>
              <option value="chat_completions">Chat Completions</option>
              <option value="openai">OpenAI Responses</option>
            </select>
          </div>
          <div class="form-group">
            <label>API Key Header</label>
            <input v-model="apiKeyHeader" type="text" placeholder="X-Api-Key" />
          </div>
        </div>
      </template>
    </div>

    <button class="btn-primary" @click="saveConfig">Save Config</button>
    <p v-if="message" class="message">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { readConfig, writeConfig } from '../api/bridge'

const providerId = ref('')
const newProviderName = ref('')
const model = ref('')
const contextWindow = ref('')
const compactLimit = ref('')
const wireApi = ref('anthropic')
const baseUrl = ref('')
const apiKey = ref('')
const apiKeyHeader = ref('X-Api-Key')
const providerIds = ref<string[]>([])
const message = ref('')

function addProvider() {
  const name = newProviderName.value.trim()
  if (name && !providerIds.value.includes(name)) {
    providerIds.value.push(name)
    providerId.value = name
    newProviderName.value = ''
  }
}

function parseConfig(text: string) {
  providerIds.value = []
  const lines = text.split('\n')
  let currentProvider = ''
  let inProviderSection = false

  for (const line of lines) {
    const trimmed = line.trim()

    if (trimmed.startsWith('provider = ')) {
      providerId.value = trimmed.replace('provider = ', '').replace(/"/g, '')
    } else if (trimmed.startsWith('model_context_window = ')) {
      contextWindow.value = trimmed.replace('model_context_window = ', '')
    } else if (trimmed.startsWith('model_auto_compact_token_limit = ')) {
      compactLimit.value = trimmed.replace('model_auto_compact_token_limit = ', '')
    } else if (trimmed.startsWith('[providers.')) {
      currentProvider = trimmed.replace('[providers.', '').replace(']', '')
      inProviderSection = true
      if (!providerIds.value.includes(currentProvider)) {
        providerIds.value.push(currentProvider)
      }
    } else if (trimmed.startsWith('[') && trimmed !== '[providers.' + currentProvider + ']') {
      inProviderSection = false
    } else if (inProviderSection && currentProvider === providerId.value) {
      if (trimmed.startsWith('wire_api = ')) {
        wireApi.value = trimmed.replace('wire_api = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('base_url = ')) {
        baseUrl.value = trimmed.replace('base_url = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('model = ')) {
        model.value = trimmed.replace('model = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('api_key = ')) {
        apiKey.value = trimmed.replace('api_key = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('api_key_header = ')) {
        apiKeyHeader.value = trimmed.replace('api_key_header = ', '').replace(/"/g, '')
      }
    }
  }
}

function buildConfig(): string {
  const lines: string[] = [
    '# evocode bridge config',
    '# Read by evocode-cli, not by upstream Codex directly.',
    '',
    `provider = "${providerId.value}"`,
    '',
    `model_context_window = ${contextWindow.value}`,
    `model_auto_compact_token_limit = ${compactLimit.value}`,
    '',
    `[providers.${providerId.value}]`,
    `wire_api = "${wireApi.value}"`,
    `base_url = "${baseUrl.value}"`,
    `model = "${model.value}"`,
    `api_key = "${apiKey.value}"`,
    `api_key_header = "${apiKeyHeader.value}"`,
  ]
  return lines.join('\n')
}

async function loadConfig() {
  const text = await readConfig()
  parseConfig(text)
}

async function saveConfig() {
  const content = buildConfig()
  await writeConfig(content)
  message.value = 'Config saved! Restart the bridge to apply changes.'
  setTimeout(() => { message.value = '' }, 3000)
}

onMounted(loadConfig)
</script>

<style scoped>
.config-view {
  max-width: 640px;
  margin: 0 auto;
  padding: 24px 20px;
}

.header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}

.back-btn {
  color: #888;
  text-decoration: none;
  font-size: 14px;
}

.back-btn:hover {
  color: #60a5fa;
}

.header h2 {
  font-size: 18px;
  font-weight: 500;
  color: #e0e0e0;
}

.config-form {
  background: #1a1a1a;
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 16px;
}

.config-form h3 {
  font-size: 12px;
  font-weight: 500;
  color: #666;
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group.full {
  margin-bottom: 12px;
}

.form-group label {
  font-size: 12px;
  color: #888;
}

.form-group input,
.form-group select {
  padding: 10px 12px;
  background: #0f0f0f;
  color: #e0e0e0;
  border: 1px solid #333;
  border-radius: 6px;
  font-size: 14px;
}

.add-provider-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.add-provider-row select {
  flex: 1;
  min-width: 0;
}

.new-provider-input {
  width: 120px;
}

.btn-add {
  padding: 8px 12px;
  background: #333;
  color: #e0e0e0;
  border: 1px solid #444;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-add:hover:not(:disabled) {
  background: #444;
}

.btn-add:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.form-group select {
  cursor: pointer;
}

.btn-primary {
  width: 100%;
  padding: 12px;
  background: #60a5fa;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-primary:hover {
  background: #3b82f6;
}

.message {
  margin-top: 12px;
  text-align: center;
  color: #22c55e;
  font-size: 13px;
}
</style>
