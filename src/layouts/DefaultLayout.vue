<template>
  <a-layout class="app-shell">
    <a-layout-sider
      v-model:collapsed="collapsed"
      :trigger="null"
      collapsible
      :width="224"
      :collapsedWidth="72"
      class="app-sider"
    >
      <div class="brand" :class="{ collapsed }">
        <img :src="logo" alt="evocode logo" class="brand-logo" />
        <transition name="brand-fade">
          <span v-if="!collapsed" class="brand-text gradient-text">evocode</span>
        </transition>
      </div>

      <a-menu
        mode="inline"
        :selected-keys="[route.path]"
        class="app-menu"
        @click="onMenuClick"
      >
        <a-menu-item key="/">
          <template #icon><HomeOutlined /></template>
          <span>{{ t("dashboard") }}</span>
        </a-menu-item>
        <a-menu-item key="/logs">
          <template #icon><CodeOutlined /></template>
          <span>{{ t("logs.title") }}</span>
        </a-menu-item>
        <a-menu-item key="/prompts">
          <template #icon><FileTextOutlined /></template>
          <span>{{ t("prompts.title") }}</span>
        </a-menu-item>
        <a-menu-item key="/config">
          <template #icon><SettingOutlined /></template>
          <span>{{ t("configuration") }}</span>
        </a-menu-item>
        <a-menu-item key="/about">
          <template #icon><InfoCircleOutlined /></template>
          <span>{{ t("about") }}</span>
        </a-menu-item>
      </a-menu>

      <div class="sider-footer">
        <div class="version-pill" v-if="!collapsed">
          <span class="dot ok" />
          <span>v{{ currentVersion || '0.0.0' }}</span>
        </div>
        <a-button
          v-if="!collapsed"
          type="text"
          size="small"
          class="update-btn"
          :loading="checkingUpdate"
          @click="handleCheckUpdate"
        >
          <template #icon><DownloadOutlined v-if="!updateAvailable" /><ArrowDownOutlined v-else style="color: var(--ok)" /></template>
          {{ updateText }}
        </a-button>
        <a-tooltip :title="collapsed ? t('expand') : t('collapse')">
          <a-button
            type="text"
            class="collapse-btn"
            @click="collapsed = !collapsed"
          >
            <MenuUnfoldOutlined v-if="collapsed" />
            <MenuFoldOutlined v-else />
          </a-button>
        </a-tooltip>
      </div>
    </a-layout-sider>

    <a-layout class="app-main">
      <a-layout-header class="app-header">
        <div class="header-left">
          <a-breadcrumb class="crumbs">
            <a-breadcrumb-item>
              <router-link to="/">{{ t("home") }}</router-link>
            </a-breadcrumb-item>
            <a-breadcrumb-item v-if="route.path !== '/'">
              {{ currentTitle }}
            </a-breadcrumb-item>
          </a-breadcrumb>
        </div>
        <div class="header-right">
<a-tooltip :title="locale === 'en' ? '\u4e2d\u6587' : 'English'">
            <a-button type="text" class="icon-btn lang-btn" @click="localeToggle">
              {{ locale === 'en' ? 'EN' : '\u4e2d\u6587' }}
            </a-button>
          </a-tooltip>
        </div>
      </a-layout-header>

      <a-layout-content class="app-content">
        <div class="content-inner fade-up">
          <slot />
        </div>
      </a-layout-content>
    </a-layout>

  <!-- Update checking modal -->
  <a-modal
    v-model:open="showUpdateLoadingModal"
    :footer="null"
    :closable="true"
    width="320px"
    class="update-modal"
    :centered="true"
    :maskClosable="false"
    :bodyStyle="{ padding: 0 }"
    destroyOnClose
    >
    <div class="update-body update-loading-body">
      <div class="update-title">{{ t('update.checking_title') }}</div>
      <div class="update-loading-spinner">
        <a-spin size="large" />
      </div>
    </div>
  </a-modal>

  <!-- Update found modal -->
  <a-modal
    v-model:open="showUpdateModal"
    :footer="null"
    :closable="true"
    width="360px"
    class="update-modal"
    :centered="true"
    :maskClosable="false"
    :bodyStyle="{ padding: 0 }"
    destroyOnClose
    >
    <div class="update-body">
      <div class="update-title">{{ t('update.found') }}</div>
      <div class="update-info">
        <span class="update-label">{{ t('update.modal_version') }}</span>
        <span class="update-version">v{{ latestVersion }}</span>
      </div>
      <div class="update-current">{{ t('update.modal_current') }} v{{ currentVersion }}</div>
      <div class="update-btns">
        <a-button type="primary" class="btn-update" @click="downloadUpdate">{{ t('update.download') }}</a-button>
      </div>
    </div>
  </a-modal>
  </a-layout>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import logo from '../assets/logo.png'
import {
  HomeOutlined,
  SettingOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  DownloadOutlined,
  ArrowDownOutlined,
  InfoCircleOutlined,
  CodeOutlined,
} from '@ant-design/icons-vue'
import { useLocale } from '../composables/useLocale'
import { message } from 'ant-design-vue'
import { getAppVersion } from '../api/bridge'
import { checkUpdate } from '../api/check_update'
import { openUrl } from '@tauri-apps/plugin-opener'

const route = useRoute()
const router = useRouter()
const collapsed = ref(false)
const currentVersion = ref('')
const updateAvailable = ref(false)
const updateUrl = ref('')
const latestVersion = ref('')
const checkingUpdate = ref(false)
const showUpdateModal = ref(false)
const showUpdateLoadingModal = ref(false)

const { locale, t, toggle: localeToggle } = useLocale()

const titleMap: Record<string, string> = {
  '/': t('dashboard'),
  '/config': t('configuration'),
}
const currentTitle = computed(() => titleMap[route.path] || 'Page')

const updateText = computed(() => {
  if (checkingUpdate.value) return ''
  if (updateAvailable.value) return 'v' + latestVersion.value
  return t('update.check')
})

async function handleCheckUpdate() {
  checkingUpdate.value = true
  try {
    const result = await checkUpdate()
    if (result.hasUpdate) {
      latestVersion.value = result.latestVersion
      updateUrl.value = result.releaseUrl
      updateAvailable.value = true
      showUpdateModal.value = true
    } else {
      latestVersion.value = ''
      updateAvailable.value = false
      message.success(t('update.up_to_date'), 2)
    }
  } catch {
    message.error(t('update.error'), 3)
  } finally {
    checkingUpdate.value = false
  }
}
async function downloadUpdate() {
  try {
    if (!updateUrl.value) { message.error("No update URL available"); return }
    await openUrl(updateUrl.value)
  } catch (e) {
    console.error('Failed to open URL:', e)
  }
  showUpdateModal.value = false
}

function onMenuClick({ key }: { key: string }) {
  if (key !== route.path) router.push(key)
}

onMounted(async () => {
  try {
    currentVersion.value = await getAppVersion()
  } catch {}
  // auto-check update on first app launch only (not on page refresh)
  if (sessionStorage.getItem('_update_checked')) return
  sessionStorage.setItem('_update_checked', '1')
  showUpdateLoadingModal.value = true
  try {
    const result = await checkUpdate()
    showUpdateLoadingModal.value = false
    if (result.hasUpdate) {
      latestVersion.value = result.latestVersion
      updateUrl.value = result.releaseUrl
      updateAvailable.value = true
      showUpdateModal.value = true
    }
  } catch {
    showUpdateLoadingModal.value = false
    // silent fail for auto-check
  }
})
</script>

<style scoped>
.app-shell {
  min-height: 100vh;
  background: transparent;
}

.app-sider {
  background: var(--bg-elev-1) !important;
  border-right: 1px solid var(--border);
  position: sticky;
  top: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  box-shadow: 1px 0 0 var(--border);
}

.app-sider :deep(.ant-layout-sider-children) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 18px 18px 16px;
  font-weight: 700;
  font-size: 18px;
  letter-spacing: 0.3px;
  height: var(--header-h);
  box-sizing: border-box;
}
.brand.collapsed { padding: 18px 0 16px; justify-content: center; }

.brand-logo {
  width: 32px;
  height: 32px;
  border-radius: 9px;
  display: block;
  object-fit: cover;
  flex-shrink: 0;
  box-shadow: var(--shadow-glow);
}

.brand-text {
  font-size: 17px;
  font-weight: 700;
  white-space: nowrap;
}

.brand-fade-enter-active,
.brand-fade-leave-active {
  transition: opacity .2s ease;
}
.brand-fade-enter-from,
.brand-fade-leave-to {
  opacity: 0;
}

.app-menu {
  background: transparent !important;
  border-inline-end: none !important;
  padding: 8px 10px;
  flex: 1;
}
.app-menu :deep(.ant-menu-item) {
  border-radius: 10px;
  margin: 4px 0;
  height: 40px;
  line-height: 40px;
  color: var(--text-2);
}
.app-menu :deep(.ant-menu-item:hover) {
  color: var(--text-1);
  background: var(--bg-elev-3) !important;
}
.app-menu :deep(.ant-menu-item-selected) {
  background: rgba(255,255,255,0.08) !important;
  color: var(--text-1) !important;
  border: 1px solid rgba(255,255,255,0.15);
  box-shadow: inset 0 0 0 1px rgba(255,255,255,0.08);
}
.app-menu :deep(.ant-menu-item-selected .anticon) { color: var(--brand-300); }

.sider-footer {
  border-top: 1px solid var(--border);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: stretch;
}
.version-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-3);
  font-size: 12px;
  width: max-content;
  margin: 0 auto;
}
.version-pill .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); box-shadow: 0 0 8px var(--ok); }
.update-btn {
  width: 100%;
  color: var(--text-3);
  font-size: 12px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.update-btn:hover { color: var(--text-1); background: var(--bg-elev-3); }
.collapse-btn {
  width: 100%;
  color: var(--text-3);
  display: flex;
  justify-content: center;
}

.app-main {
  background: transparent;
}

.app-header {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 28px;
  height: var(--header-h);
  background: var(--bg-glass);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  -webkit-backdrop-filter: blur(14px) saturate(140%);
}

.crumbs :deep(.ant-breadcrumb-link),
.crumbs :deep(.ant-breadcrumb-separator) {
  color: var(--text-3);
}
.crumbs :deep(a) { color: var(--text-2); }
.crumbs :deep(a:hover) { color: var(--brand-300); }

.lang-btn { font-weight: 600; font-size: 13px; letter-spacing: 0.5px; width: auto; padding: 0 8px; }

.app-content {
  padding: 24px 28px 40px;
  min-height: calc(100vh - var(--header-h));
  height: calc(100vh - var(--header-h));
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
}
.content-inner {
  max-width: 1080px;
  margin: 0 auto;
  width: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  
}

@media (max-width: 720px) {
  .app-header { padding: 0 16px; }
  .app-content { padding: 16px 14px 32px; }
  .app-sider { position: fixed; z-index: 20; }
}
</style>

<style>
/* ── Update modal (global overrides) ── */
.update-modal .ant-modal-content {
  border-radius: 12px !important;
  overflow: hidden;
}
.update-modal .ant-modal-header {
  display: none !important;
}
.update-modal .ant-modal-body {
  padding: 0 !important;
}

.update-body {
  padding: 24px 24px 20px;
}
.update-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-1);
  margin-bottom: 16px;
}

.update-loading-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 24px;
  padding-top: 32px;
  padding-bottom: 28px;
}
.update-loading-spinner {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 48px;
}
.update-loading-body .update-title {
  margin-bottom: 0;
}
.update-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}
.update-label {
  font-size: 14px;
  color: var(--text-2);
}
.update-version {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-1);
}
.update-current {
  font-size: 13px;
  color: var(--text-3);
  margin-bottom: 20px;
}
.update-btns {
  display: flex;
  gap: 10px;
}
.btn-update {
  flex: 1;
  height: 36px !important;
  border-radius: 8px !important;
  font-size: 14px !important;
}
.btn-skip {
  flex: 1;
  height: 36px !important;
  border-radius: 8px !important;
  font-size: 14px !important;
  color: var(--text-3) !important;
  border-color: var(--border) !important;
}
.btn-skip:hover {
  color: var(--text-1) !important;
  border-color: var(--text-3) !important;
}
</style>

