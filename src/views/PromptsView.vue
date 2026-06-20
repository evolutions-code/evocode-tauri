<template>
  <div class="prompts-page">
    <div class="page-header">
      <h2 class="page-title">
        <FileTextOutlined />
        <span class="bar" />
        {{ t("prompts.title") }}
      </h2>
      <div class="page-actions">
        <a-tag v-if="agentsExists" color="green" class="status-tag">
          <CheckCircleOutlined /> {{ t("prompts.agents_exists") }}
        </a-tag>
        <a-tag v-else color="default" class="status-tag">
          {{ t("prompts.agents_missing") }}
        </a-tag>
      </div>
    </div>

    <div class="prompts-layout">
      <!-- LEFT: Prompt file list -->
      <div class="prompts-sidebar">
        <div class="sidebar-header">
          <span class="sidebar-title">{{ t("prompts.presets") }}</span>
          <a-button type="primary" size="small" @click="handleNewPrompt">
            <template #icon><PlusOutlined /></template>
            {{ t("prompts.new") }}
          </a-button>
        </div>

        <div v-if="promptFiles.length === 0" class="empty-state" style="min-height:120px">
          <div class="empty-icon"><InboxOutlined /></div>
          <div class="empty-title">{{ t("prompts.no_presets") }}</div>
          <div class="empty-sub">{{ t("prompts.no_presets_hint") }}</div>
        </div>

        <a-menu v-else mode="inline" class="presets-menu" :selected-keys="[selectedName]">
          <a-menu-item
            v-for="f in promptFiles"
            :key="f.name"
            @click="selectPrompt(f)"
          >
            <div class="preset-item">
              <div class="preset-info">
                <span class="preset-name">{{ f.name }}</span>
                <span class="preset-date">{{ formatDate(f.updated_at) }}</span>
              </div>
              <a-button
                type="text"
                size="small"
                class="preset-delete-btn"
                danger
                @click.stop="handleDeletePrompt(f)"
              >
                <template #icon><DeleteOutlined /></template>
              </a-button>
            </div>
          </a-menu-item>
        </a-menu>
      </div>

      <!-- RIGHT: Editor -->
      <div class="prompts-editor">
        <div v-if="!selectedName" class="empty-state">
          <div class="empty-icon"><EditOutlined /></div>
          <div class="empty-title">{{ t("prompts.select_hint") }}</div>
          <div class="empty-sub">{{ t("prompts.select_hint_desc") }}</div>
        </div>

        <template v-else>
          <div class="editor-header">
            <a-input
              v-model:value="editingName"
              :placeholder="t('prompts.name_placeholder')"
              class="name-input"
              size="large"
            />
            <div class="editor-actions">
              <a-button @click="handleSave" type="primary" :loading="saving" :disabled="!editingName.trim()">
                <template #icon><SaveOutlined /></template>
                {{ t("prompts.save") }}
              </a-button>
              <a-button @click="handleActivate" type="primary" :loading="activating" :disabled="!editingContent.trim()">
                <template #icon><SendOutlined /></template>
                {{ t("prompts.activate") }}
              </a-button>
            </div>
          </div>

          <div class="editor-body">
            <div class="editor-label">
              {{ t("prompts.content") }}
              <span class="editor-label-hint">{{ editingContent.length }} {{ t("prompts.chars") }}</span>
            </div>
            <a-textarea
              v-model:value="editingContent"
              class="content-textarea mono"
              :placeholder="t('prompts.content_placeholder')"
              :rows="20"
            />
          </div>
        </template>
      </div>
    </div>

    <!-- New prompt name modal -->
    <a-modal
      v-model:open="showNewModal"
      :title="t('prompts.new')"
      :ok-text="t('prompts.save')"
      :cancel-text="t('prompts.delete_cancel')"
      @ok="confirmNewPrompt"
      :okButtonProps="{ disabled: !newPromptName.trim() }"
    >
      <a-input
        v-model:value="newPromptName"
        :placeholder="t('prompts.name_placeholder')"
        @pressEnter="confirmNewPrompt"
      />
    </a-modal>

    <!-- Delete confirm modal -->
    <a-modal
      v-model:open="showDeleteModal"
      :title="t('prompts.delete_title')"
      :ok-text="t('prompts.delete_ok')"
      :cancel-text="t('prompts.delete_cancel')"
      @ok="confirmDelete"
      :okButtonProps="{ danger: true }"
    >
      <p>{{ t("prompts.delete_confirm", { name: deletingName }) }}</p>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { useLocale } from '../composables/useLocale'
import {
  FileTextOutlined,
  PlusOutlined,
  DeleteOutlined,
  SaveOutlined,
  SendOutlined,
  EditOutlined,
  InboxOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons-vue'
import {
  listPromptFiles,
  readCorePrompt,
  readPromptFile,
  writePromptFile,
  deletePromptFile,
  readAgentsFile,
  writeAgentsFile,
  type PromptFile,
} from '../api/prompts'
import { syncToCodex } from '../api/bridge'

const { t } = useLocale()

const promptFiles = ref<PromptFile[]>([])
const selectedName = ref<string>('')
const editingName = ref<string>('')
const editingContent = ref<string>('')
const agentsContent = ref<string>('')
const agentsExists = ref(false)
const saving = ref(false)
const activating = ref(false)

// New prompt modal
const showNewModal = ref(false)
const newPromptName = ref('')

// Delete modal
const showDeleteModal = ref(false)
const deletingName = ref('')

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr)
    const month = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    const hour = String(d.getHours()).padStart(2, '0')
    const min = String(d.getMinutes()).padStart(2, '0')
    return `${month}-${day} ${hour}:${min}`
  } catch {
    return dateStr
  }
}

async function loadData() {
  try {
    const [files, agents] = await Promise.all([
      listPromptFiles(),
      readAgentsFile(),
    ])
    promptFiles.value = files
    agentsContent.value = agents
    agentsExists.value = agents.length > 0
  } catch (err) {
    console.error('Failed to load prompt data:', err)
  }
}

async function selectPrompt(f: PromptFile) {
  selectedName.value = f.name
  editingName.value = f.name
  editingContent.value = ''
  try {
    const content = await readPromptFile(f.name)
    editingContent.value = content
  } catch (err) {
    console.error('Failed to read prompt file:', err)
    editingContent.value = ''
  }
}

function handleNewPrompt() {
  newPromptName.value = ''
  showNewModal.value = true
}

async function confirmNewPrompt() {
  const name = newPromptName.value.trim()
  if (!name) return

  // Check if already exists
  if (promptFiles.value.some(f => f.name === name)) {
    const existing = promptFiles.value.find(f => f.name === name)
    if (existing) selectPrompt(existing)
    showNewModal.value = false
    return
  }

  const now = new Date().toISOString()
  try {
    // Create file with empty content, then pre-fill editor with core prompt
    const corePrompt = await readCorePrompt()
    await writePromptFile(name, corePrompt)
    promptFiles.value.unshift({ name, updated_at: now })
    selectedName.value = name
    editingName.value = name
    editingContent.value = corePrompt
    showNewModal.value = false
  } catch (err) {
    console.error('Failed to create prompt:', err)
  }
}

async function handleSave() {
  const name = editingName.value.trim()
  if (!name) return

  saving.value = true
  try {
    if (name !== selectedName.value) {
      await deletePromptFile(selectedName.value)
    }
    await writePromptFile(name, editingContent.value)
    selectedName.value = name
    promptFiles.value = await listPromptFiles()
    message.success(t('prompts.saved_ok'))
  } catch (err) {
    console.error('Failed to save prompt:', err)
    message.error(t('prompts.save_fail'))
  } finally {
    saving.value = false
  }
}

async function handleActivate() {
  if (!editingContent.value.trim()) return
  activating.value = true
  try {
    // Write to AGENTS.md, then apply to Codex model catalog
    await writeAgentsFile(editingContent.value)
    agentsContent.value = editingContent.value
    agentsExists.value = true
    await syncToCodex()
    message.success(t('prompts.activated_ok'))
  } catch (err) {
    console.error('Failed to activate prompt:', err)
    message.error(t('prompts.activated_fail'))
  } finally {
    activating.value = false
  }
}

function handleDeletePrompt(f: PromptFile) {
  deletingName.value = f.name
  showDeleteModal.value = true
}

async function confirmDelete() {
  try {
    await deletePromptFile(deletingName.value)
    promptFiles.value = promptFiles.value.filter(f => f.name !== deletingName.value)
    if (selectedName.value === deletingName.value) {
      selectedName.value = ''
      editingName.value = ''
      editingContent.value = ''
    }
  } catch (err) {
    console.error('Failed to delete prompt:', err)
  } finally {
    showDeleteModal.value = false
    deletingName.value = ''
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.prompts-page {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.prompts-layout {
  flex: 1;
  display: flex;
  gap: 16px;
  min-height: 0;
  overflow: hidden;
}

.prompts-sidebar {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-elev-2);
  border-radius: var(--r-md);
  border: 1px solid var(--border);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
}

.sidebar-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.presets-menu {
  flex: 1;
  overflow-y: auto;
  padding: 6px 8px;
  background: transparent !important;
  border-inline-end: none !important;
}

.presets-menu :deep(.ant-menu-item) {
  height: auto !important;
  line-height: normal !important;
  padding: 8px 10px !important;
  border-radius: 8px;
  margin: 2px 0;
  white-space: normal;
  color: var(--text-2);
}

.presets-menu :deep(.ant-menu-item:hover) {
  background: var(--bg-elev-3) !important;
  color: var(--text-1);
}

.presets-menu :deep(.ant-menu-item-selected) {
  background: rgba(79, 124, 255, 0.12) !important;
  color: var(--text-1) !important;
}

.preset-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
}

.preset-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.preset-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-date {
  font-size: 11px;
  color: var(--text-4);
}

.preset-delete-btn {
  opacity: 0;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.preset-item:hover .preset-delete-btn {
  opacity: 1;
}

.prompts-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-elev-2);
  border-radius: var(--r-md);
  border: 1px solid var(--border);
  overflow: hidden;
}

.editor-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}

.name-input {
  flex: 1;
  min-width: 180px;
}

.name-input :deep(.ant-input) {
  font-weight: 600;
  font-size: 15px;
  background: transparent !important;
  border: none !important;
  box-shadow: none !important;
  padding: 0;
}

.name-input :deep(.ant-input:hover),
.name-input :deep(.ant-input:focus) {
  background: var(--bg-elev-3) !important;
  padding: 4px 8px;
  border-radius: 6px;
}

.editor-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.editor-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow: hidden;
}

.editor-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-3);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.editor-label-hint {
  font-weight: 400;
  color: var(--text-4);
}

.content-textarea {
  flex: 1;
  resize: none;
  border-radius: 8px;
  background: var(--bg-elev-1) !important;
  border: 1px solid var(--border) !important;
  color: var(--text-1) !important;
  font-size: 13px;
  line-height: 1.6;
  padding: 12px 14px;
}

.content-textarea:focus {
  border-color: var(--brand-300) !important;
  box-shadow: 0 0 0 2px rgba(79, 124, 255, 0.15) !important;
}

.page-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.status-tag {
  border-radius: 999px;
  font-size: 12px;
  padding: 2px 12px;
}
</style>
