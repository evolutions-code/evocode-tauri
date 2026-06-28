<template>
  <div class="bridge-card glass fade-up" v-if="ready">
    <div class="ring" :class="status">
      <span class="core" />
      <span class="pulse" />
      <template v-if="status === 'running'">
        <span v-for="i in 8" :key="i" class="particle" :style="{ '--d': `${(i - 1) * 0.3}s` }" />
      </template>
    </div>
    <div class="meta">
      <div class="label">{{ statusLabel }}</div>
      <div class="sub">{{ status === 'running' ? t('bridge.serving') : status === 'starting' ? t('bridge.booting') : t('bridge.idle') }}</div>
      <div class="provider">
        <span class="dot" :class="status" />
        <span>{{ status }}</span>
        <span v-if="status === 'running' && latency !== null" class="stat-item">
          <span class="sep" />
          <span class="stat-dot" :class="latencyClass" />
          <span>{{ latency }}ms</span>
        </span>
        <span v-if="status === 'running'" class="stat-item">
          <span class="sep" />
          <span>⏱ {{ uptimeDisplay }}</span>
        </span>
        <span class="sep" />
        <span class="mono">{{ props.provider || '-' }}</span>
      </div>
    </div>
    <div class="url mono">
      <span class="url-label">{{ t("bridge.url") }}</span>
      <code>http://127.0.0.1:{{ port }}</code>
      <a-tag v-if="status === 'running'" color="green" class="listening-tag">
        <span class="listening-dot" />{{ t("bridge.listening") }}
      </a-tag>
      <a-tooltip :title="t('bridge.copy')">
        <CopyOutlined class="copy" @click="copyUrl" />
      </a-tooltip>
    </div>
    <div v-if="status === 'running'" class="activity-wave">
      <span v-for="i in 12" :key="i" class="wave-bar" :style="{ '--i': i }" />
    </div>
    <div class="btn-row">
      <a-button
        class="toggle"
        :type="status === 'running' ? 'default' : 'primary'"
        :danger="status === 'running'"
        :loading="loading"
        size="large"
        @click="$emit('toggle')"
      >
        <template #icon>
          <PoweroffOutlined v-if="status !== 'running'" />
          <PauseOutlined v-else />
        </template>
        {{ status === 'running' ? t('bridge.stop') : t('bridge.start') }}
      </a-button>
      <a-button
        class="logs-btn"
        type="default"
        size="large"
        @click="goToLogs"
      >
        <template #icon><CodeOutlined /></template>
        {{ t("logs.title") }}
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useLocale } from '../composables/useLocale'
import { getBridgePort, pingBridge } from '../api/bridge'
import { PoweroffOutlined, PauseOutlined, CopyOutlined, CodeOutlined } from '@ant-design/icons-vue'

const port = ref(17761)
const ready = ref(false)
const { t } = useLocale()
const router = useRouter()

const props = defineProps<{
  status: string
  loading: boolean
  provider?: string
}>()

defineEmits<{
  toggle: []
}>()

// ─── Uptime ───
const startedAt = ref<number | null>(null)
const uptimeSeconds = ref(0)
let uptimeTimer: ReturnType<typeof setInterval> | null = null

// ─── Health check ───
const latency = ref<number | null>(null)
let pingTimer: ReturnType<typeof setInterval> | null = null

const statusLabel = computed(() => {
  switch (props.status) {
    case 'running': return t('bridge.online')
    case 'starting': return t('bridge.starting')
    case 'error': return 'Error'
    default: return t('bridge.offline')
  }
})

const latencyClass = computed(() => {
  if (latency.value === null) return ''
  if (latency.value < 50) return 'latency-fast'
  if (latency.value < 200) return 'latency-ok'
  return 'latency-slow'
})

const uptimeDisplay = computed(() => {
  const s = uptimeSeconds.value
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`
})

function copyUrl() {
  navigator.clipboard?.writeText('http://127.0.0.1:' + port.value).catch(() => {})
}

function goToLogs() {
  router.push('/logs')
}

// ─── Uptime tracking ───
function startUptime() {
  startedAt.value = Date.now()
  uptimeSeconds.value = 0
  uptimeTimer = setInterval(() => {
    if (startedAt.value) {
      uptimeSeconds.value = Math.floor((Date.now() - startedAt.value) / 1000)
    }
  }, 1000)
}

function stopUptime() {
  if (uptimeTimer) { clearInterval(uptimeTimer); uptimeTimer = null }
  startedAt.value = null
  uptimeSeconds.value = 0
}

// ─── Health check ───
async function doPing() {
  try {
    latency.value = await pingBridge()
  } catch {
    latency.value = null
  }
}

function startPing() {
  doPing()
  pingTimer = setInterval(doPing, 5000)
}

function stopPing() {
  if (pingTimer) { clearInterval(pingTimer); pingTimer = null }
  latency.value = null
}

// Watch status changes
watch(() => props.status, (newStatus, oldStatus) => {
  if (newStatus === 'running' && oldStatus !== 'running') {
    startUptime(); startPing()
  } else if (newStatus !== 'running' && oldStatus === 'running') {
    stopUptime(); stopPing()
  }
})

onMounted(async () => {
  try { port.value = await getBridgePort() } catch { /* keep default 17761 */ }
  if (props.status === 'running') { startUptime(); startPing() }
  ready.value = true
})

onUnmounted(() => { stopUptime(); stopPing() })
</script>

<style scoped>
.bridge-card {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-areas:
    "ring meta"
    "url url"
    "wave wave"
    "toggle toggle";
  align-items: center;
  gap: 14px 20px;
  padding: 20px 22px;
  border-radius: var(--r-lg);
  position: relative;
  overflow: hidden;
}

/* ═══════ Ring with Data Particles ═══════ */
.ring {
  grid-area: ring;
  position: relative;
  width: 60px;
  height: 60px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: rgba(248, 113, 113, 0.12);
  border: 2px solid rgba(248, 113, 113, 0.3);
  flex-shrink: 0;
}
.ring .core {
  width: 14px; height: 14px; border-radius: 50%;
  background: var(--err);
  box-shadow: 0 0 12px var(--err);
  z-index: 2;
}
.ring .pulse {
  position: absolute; inset: 0; border-radius: 50%;
  border: 2px solid var(--err);
  opacity: .5;
  animation: pulse 2s ease-out infinite;
}
.ring .particle {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  z-index: 1;
  animation: particle-orbit 2.4s linear infinite;
  animation-delay: var(--d, 0s);
}
.ring .particle::before {
  content: '';
  position: absolute;
  top: -3px;
  left: 50%;
  width: 5px;
  height: 5px;
  margin-left: -2.5px;
  border-radius: 50%;
  background: var(--ok);
  box-shadow: 0 0 6px var(--ok);
  animation: particle-fade 2.4s ease-in-out infinite;
  animation-delay: var(--d, 0s);
}
@keyframes particle-orbit {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
@keyframes particle-fade {
  0%, 100% { opacity: 0; }
  5%, 45% { opacity: 1; }
}
.ring.running {
  background: rgba(52, 211, 153, 0.12);
  border-color: rgba(52, 211, 153, 0.35);
}
.ring.running .core { background: var(--ok); box-shadow: 0 0 14px var(--ok); }
.ring.running .pulse { border-color: var(--ok); }
.ring.starting {
  background: rgba(251, 191, 36, 0.12);
  border-color: rgba(251, 191, 36, 0.35);
}
.ring.starting .core { background: var(--warn); box-shadow: 0 0 12px var(--warn); }
.ring.starting .pulse { border-color: var(--warn); }
@keyframes pulse {
  0% { transform: scale(0.85); opacity: .7; }
  100% { transform: scale(1.6); opacity: 0; }
}

/* ═══════ Meta ═══════ */
.meta { grid-area: meta; min-width: 0; }
.meta .label { font-size: 16px; font-weight: 600; color: var(--text-1); }
.meta .sub { font-size: 12px; color: var(--text-3); margin-top: 2px; }
.meta .provider {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-3);
}
.meta .provider .dot { width: 6px; height: 6px; border-radius: 50%; display: inline-block; }
.meta .provider .dot.running { background: var(--ok); }
.meta .provider .dot.stopped { background: var(--text-4); }
.meta .provider .sep { width: 1px; height: 12px; background: var(--border); display: inline-block; }
.meta .provider .mono { font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace; color: var(--text-2); }
.meta .stat-item { display: inline-flex; align-items: center; gap: 4px; }
.stat-dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--text-4);
}
.stat-dot.latency-fast { background: var(--ok); box-shadow: 0 0 6px var(--ok); }
.stat-dot.latency-ok { background: var(--warn); box-shadow: 0 0 6px var(--warn); }
.stat-dot.latency-slow { background: var(--err); box-shadow: 0 0 6px var(--err); }

/* ═══════ URL ═══════ */
.url {
  grid-area: url;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-2);
  font-size: 12px;
  white-space: nowrap;
  justify-self: start;
}
.url .url-label { color: var(--text-3); font-size: 10px; text-transform: uppercase; letter-spacing: .8px; }
.url code { color: var(--brand-300); }
.url .copy { color: var(--text-3); cursor: pointer; padding: 2px; }
.url .copy:hover { color: var(--brand-300); }
.listening-tag {
  display: inline-flex !important;
  align-items: center;
  gap: 4px;
  font-size: 10px !important;
  line-height: 1;
  padding: 0 7px !important;
  border-radius: 999px !important;
  background: rgba(52, 211, 153, 0.15) !important;
  border: 1px solid rgba(52, 211, 153, 0.3) !important;
  color: var(--ok) !important;
  height: 20px !important;
  margin-inline-end: 0 !important;
}
.listening-dot {
  width: 5px; height: 5px; border-radius: 50%;
  background: var(--ok);
  animation: blink 1.2s ease-in-out infinite;
}
@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

/* ═══════ Activity Wave ═══════ */
.activity-wave {
  grid-area: wave;
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 26px;
  padding: 4px 0;
}
.wave-bar {
  width: 5px;
  border-radius: 3px;
  background: linear-gradient(180deg, var(--ok) 0%, rgba(52, 211, 153, 0.25) 100%);
  animation: wave-bar 1.6s ease-in-out infinite;
  animation-delay: calc(var(--i) * 0.08s);
}
@keyframes wave-bar {
  0%, 100% { height: 3px; opacity: 0.35; }
  30% { height: 18px; opacity: 1; }
  60% { height: 8px; opacity: 0.7; }
}

/* ═══════ Buttons ═══════ */
.btn-row {
  grid-area: toggle;
  display: flex;
  gap: 10px;
}
.btn-row .toggle { flex: 1; }
.btn-row .logs-btn { flex-shrink: 0; }

/* ═══════ Responsive ═══════ */
@media (max-width: 640px) {
  .bridge-card {
    grid-template-columns: 1fr;
    grid-template-areas:
      "ring"
      "meta"
      "url"
      "wave"
      "toggle";
    text-align: center;
    justify-items: center;
  }
  .url { font-size: 11px; justify-self: center; }
}
</style>
