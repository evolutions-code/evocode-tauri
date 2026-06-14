<template>
  <a-modal
    v-model:open="visible"
    title="导入会话"
    :footer="null"
    :width="640"
    @cancel="handleCancel"
  >
    <div class="import-dialog">
      <!-- Step 1: choose source -->
      <div class="step">
        <div class="step-label">导入来源</div>
        <a-radio-group v-model:value="source">
          <a-radio-button
            v-for="s in sources"
            :key="s"
            :value="s"
            @click="loadSessions(s)"
          >
            {{ sourceLabel(s) }}
          </a-radio-button>
        </a-radio-group>
      </div>

      <!-- Step 2: select sessions -->
      <div v-if="sessionList.length" class="step">
        <div class="step-head">
          <div class="step-label">选择要导入的会话</div>
          <a-checkbox :checked="allSelected" @change="toggleAll">
            全选 ({{ sessionList.length }})
          </a-checkbox>
        </div>
        <div class="session-list">
          <div
            v-for="s in sessionList"
            :key="s.id"
            class="session-item"
            :class="{ selected: selectedIds.has(s.id) }"
            @click="toggleSession(s.id)"
          >
            <a-checkbox :checked="selectedIds.has(s.id)" />
            <div class="session-info">
              <div class="session-title">{{ s.title }}</div>
              <div class="session-meta">
                <span class="session-model" v-if="s.model">{{ s.model }}</span>
                <span class="session-tokens">{{ formatTokens(s.token_count) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else-if="loaded" class="empty-state">
        {{ source ? '没有可导入的会话' : '请先选择导入来源' }}
      </div>

      <!-- Actions -->
      <div class="actions-bar">
        <a-button @click="handleCancel">取消</a-button>
        <a-button
          type="primary"
          :disabled="selectedIds.size === 0"
          :loading="importing"
          @click="handleImport"
        >
          导入选中的 {{ selectedIds.size }} 个会话
        </a-button>
      </div>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import { message } from "ant-design-vue"
import { listImportSources, listImportableSessions, importSessions } from "../api/bridge"
import type { ImportableSession } from "../api/bridge"

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: "close"): void; (e: "imported"): void }>()
const visible = computed({
  get: () => props.open,
  set: (v: boolean) => { if (!v) emit("close") },
})

const sources = ref<string[]>([])
const source = ref("")
const sessionList = ref<ImportableSession[]>([])
const selectedIds = ref<Set<string>>(new Set())
const importing = ref(false)
const loaded = ref(false)

const allSelected = computed(() => sessionList.value.length > 0 && selectedIds.value.size === sessionList.value.length)

function sourceLabel(s: string): string {
  const map: Record<string, string> = {
    opencode: "OpenCode CLI",
    claude_code: "Claude Code",
  }
  return map[s] || s
}

function toggleAll() {
  if (allSelected.value) {
    selectedIds.value = new Set()
  } else {
    selectedIds.value = new Set(sessionList.value.map(s => s.id))
  }
}

function toggleSession(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

function formatTokens(n: string | number): string {
  n = typeof n === 'string' ? parseInt(n) || 0 : n
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M"
  if (n >= 1_000) return (n / 1_000).toFixed(0) + "K"
  return String(n)
}

async function loadSources() {
  try {
    sources.value = await listImportSources()
  } catch {
    // Tauri backend not available (e.g. running in browser)
  }
}

async function loadSessions(src: string) {
  source.value = src
  loaded.value = false
  selectedIds.value = new Set()
  try {
    sessionList.value = await listImportableSessions(src)
  } catch (e: any) {
    message.error("加载会话列表失败: " + (e?.message || String(e)), 3)
    sessionList.value = []
  } finally {
    loaded.value = true
  }
}

async function handleImport() {
  if (selectedIds.value.size === 0) return
  importing.value = true
  try {
    const result = await importSessions(source.value, Array.from(selectedIds.value))
    if (result.imported > 0) {
      message.success(`成功导入 ${result.imported} 个会话`, 3)
    }
    if (result.failed > 0) {
      message.warning(`${result.failed} 个会话导入失败`, 4)
    }
    emit("imported")
    handleCancel()
  } catch (e: any) {
    message.error("导入失败: " + (e?.message || String(e)), 4)
  } finally {
    importing.value = false
  }
}

function handleCancel() {
  source.value = ""
  sessionList.value = []
  selectedIds.value = new Set()
  loaded.value = false
  emit("close")
}

loadSources().catch(() => { /* ignore outside Tauri */ })
</script>

<style scoped>
.import-dialog { display: flex; flex-direction: column; gap: 16px; }
.step { display: flex; flex-direction: column; gap: 8px; }
.step-label { font-size: 13px; font-weight: 600; color: var(--text-1); }
.step-head { display: flex; align-items: center; justify-content: space-between; }
.session-list {
  display: flex; flex-direction: column; gap: 4px;
  max-height: 360px; overflow-y: auto;
  border: 1px solid var(--border); border-radius: var(--r-md);
  padding: 4px;
}
.session-item {
  display: flex; align-items: center; gap: 10px;
  padding: 10px 12px; border-radius: var(--r-sm);
  cursor: pointer; transition: background .12s;
}
.session-item:hover { background: var(--bg-elev-3); }
.session-item.selected { background: rgba(77,125,255,0.10); }
.session-info { min-width: 0; flex: 1; }
.session-title {
  font-size: 13px; font-weight: 500; color: var(--text-1);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.session-meta { display: flex; gap: 10px; margin-top: 2px; }
.session-model { font-size: 11px; color: var(--brand-300); }
.session-tokens { font-size: 11px; color: var(--text-4); font-family: monospace; }
.empty-state { text-align: center; color: var(--text-4); padding: 32px 0; }
.actions-bar {
  display: flex; justify-content: flex-end; gap: 10px;
  padding-top: 8px; border-top: 1px solid var(--border);
}
</style>
