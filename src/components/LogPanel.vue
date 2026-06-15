<template>
  <div class="log-card glass fade-up">
    <div class="log-head">
      <div class="title">
        <span class="bar" />
        <span>{{ t("logs.title") }}</span>
        <a-tag :color="statusColor" class="status-tag">{{ statusLabel }}</a-tag>
      </div>
      <div class="actions">
        <a-tooltip :title="t('logs.refresh')">
          <a-button type="text" size="small" class="icon-btn" @click="pollStatus">
            <ReloadOutlined />
          </a-button>
        </a-tooltip>
        <a-tooltip :title="t('logs.clear')">
          <a-button type="text" size="small" class="icon-btn" @click="logLines = []">
            <ClearOutlined />
          </a-button>
        </a-tooltip>
        <a-tooltip :title="t('logs.auto_scroll')">
          <a-switch v-model:checked="autoScroll" size="small" />
        </a-tooltip>
      </div>
    </div>

    <div class="log-body" ref="bodyRef" @scroll="onScroll">
      <div v-if="logLines.length === 0" class="empty-state">
        <DatabaseOutlined class="empty-icon" />
        <div class="empty-title">{{ t("logs.empty_title") }}</div>
        <div class="empty-sub">{{ t("logs.empty_sub", { status: bridgeRunning ? t("logs.running") : t("logs.stopped") }) }}</div>
      </div>
      <div
        v-for="(line, i) in logLines"
        :key="i"
        class="log-line"
        :class="lineClass(line)"
      >
        <span class="line-no">{{ String(i + 1).padStart(3, "0") }}</span>
        <span class="line-content">{{ line }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue"
import { useLocale } from "../composables/useLocale"
import {
  ReloadOutlined,
  ClearOutlined,
  DatabaseOutlined,
} from "@ant-design/icons-vue"
import { getBridgeStatus, getBridgeLogsTail, getBridgeLogsRange } from "../api/bridge"

const { t } = useLocale()

const PAGE_SIZE = 200

const props = defineProps<{
  bridgeRunning: boolean
}>()

type Status = "starting" | "running" | "stopped" | "error"
const status = ref<Status>(props.bridgeRunning ? "starting" : "stopped")
const logLines = ref<string[]>([])
const autoScroll = ref(true)
const bodyRef = ref<HTMLElement | null>(null)
/** File line index of the first element in `logLines` (0 = top of file) */
const loadedFromLine = ref(0)
/** Total lines currently in the log file */
const totalLines = ref(0)
const loadingMore = ref(false)

const statusLabel = computed(() => {
  switch (status.value) {
    case "starting": return t("logs.label_starting")
    case "running": return t("logs.label_running")
    case "error": return t("logs.label_error")
    default: return t("logs.label_stopped")
  }
})

const statusColor = computed(() => {
  switch (status.value) {
    case "running": return "success"
    case "starting": return "processing"
    case "error": return "error"
    default: return "default"
  }
})

function lineClass(line: string) {
  const l = line.toLowerCase()
  if (l.includes("error") || l.includes("fail")) return "err"
  if (l.includes("warn")) return "warn"
  if (l.includes("request") || l.includes("POST") || l.includes("GET")) return "info"
  return ""
}

let statusInterval: ReturnType<typeof setInterval> | null = null

/// Fetch new lines appended since our last known total, then append to buffer
async function pollStatus() {
  try {
    const s = await getBridgeStatus()
    status.value = s === "running" ? "running" : "stopped"
    if (s !== "running") return

    const el = bodyRef.value
    const wasAtBottom = el && (el.scrollHeight - el.scrollTop - el.clientHeight) < 50

    if (totalLines.value === 0) {
      // First load: fetch last PAGE_SIZE lines
      const result = await getBridgeLogsTail(PAGE_SIZE)
      totalLines.value = result.total_lines
      loadedFromLine.value = Math.max(0, result.total_lines - result.lines.length)
      logLines.value = result.lines
    } else {
      // Fetch only newly appended lines
      const range = await getBridgeLogsRange(totalLines.value, 100)
      if (range.lines.length > 0) {
        logLines.value = [...logLines.value, ...range.lines]
        totalLines.value = range.total_lines
      }
    }

    // Auto-scroll to bottom only if user was already at bottom
    if (autoScroll.value && wasAtBottom) {
      await nextTick()
      if (el) el.scrollTop = el.scrollHeight
    }
  } catch {
    status.value = "error"
  }
}

/// Scroll-up pagination: when user scrolls near top, load previous page
async function onScroll() {
  const el = bodyRef.value
  if (!el || loadingMore.value || loadedFromLine.value === 0) return

  if (el.scrollTop < 100) {
    loadingMore.value = true
    const prevScrollHeight = el.scrollHeight
    const newStart = loadedFromLine.value >= PAGE_SIZE
      ? loadedFromLine.value - PAGE_SIZE
      : 0
    const actualCount = loadedFromLine.value - newStart

    if (actualCount > 0) {
      const result = await getBridgeLogsRange(newStart, actualCount + PAGE_SIZE)
      const newLines = result.lines.slice(0, actualCount)
      loadedFromLine.value = newStart
      logLines.value = [...newLines, ...logLines.value]

      await nextTick()
      // Preserve visual scroll position after prepending
      el.scrollTop = el.scrollHeight - prevScrollHeight
    }
    loadingMore.value = false
  }
}

function startPolling() {
  status.value = "starting"
  logLines.value = []
  loadedFromLine.value = 0
  totalLines.value = 0
  pollStatus()
  statusInterval = setInterval(pollStatus, 1000)
}

function stopPolling() {
  if (statusInterval) { clearInterval(statusInterval); statusInterval = null }
  status.value = "stopped"
  logLines.value = []
  loadedFromLine.value = 0
  totalLines.value = 0
}

watch(() => props.bridgeRunning, (running) => {
  if (running) startPolling()
  else stopPolling()
})

onMounted(() => {
  if (props.bridgeRunning) startPolling()
})
onUnmounted(stopPolling)
</script></script>

<style scoped>
.log-card { padding: 0; overflow: hidden; flex: 1; display: flex; flex-direction: column; }

.log-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(180deg, rgba(255,255,255,0.02), transparent);
}
.title { display: inline-flex; align-items: center; gap: 10px; color: var(--text-1); font-weight: 600; }
.title .bar {
  width: 3px; height: 14px; border-radius: 2px;
  background: linear-gradient(180deg, var(--brand-400), var(--brand-700));
}
.status-tag { margin-left: 4px; }

.actions { display: inline-flex; align-items: center; gap: 8px; }
.log-body {
  flex: 1;
  overflow-y: auto;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.7;
  background: linear-gradient(180deg, var(--bg-elev-1), var(--bg-elev-2));
  padding: 6px 0;
}

.log-line {
  display: flex; gap: 12px; padding: 1px 14px;
  color: var(--text-2); white-space: pre-wrap; word-break: break-all;
  transition: background .12s ease;
}
.log-line:hover { background: rgba(255,255,255,0.03); }
.line-no { color: var(--text-4); user-select: none; min-width: 36px; }
.log-line.info { color: #cfe1ff; }
.log-line.warn { color: #fde68a; background: rgba(251,191,36,0.04); }
.log-line.err  { color: #fecaca; background: rgba(248,113,113,0.06); }
</style>
