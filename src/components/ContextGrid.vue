<template>
  <div class="session-grid-card glass fade-up">
    <div class="card-head">
      <div class="session-info">
        <div class="session-name">{{ session.name }}</div>
        <div class="session-model">{{ session.model }}</div>
      </div>
      <div class="session-stats">
        <span class="pct">{{ Math.min(100, Math.round(session.used / session.total * 100)) }}%</span>
      </div>
    </div>

    <div class="grid" :style="gridStyle">
      <div v-for="(cell, idx) in cells" :key="idx"
        class="cell" :class="cell.cls" :style="{ '--d': cell.delay }"
        :title="`${session.name} ${((idx + 1) * 10).toLocaleString()}K / ${(session.total * 10).toLocaleString()}K tokens`" />
    </div>

    <div class="card-foot">
      <div class="foot-line">
        <span class="foot-label">{{ t("session.context_tokens") }}</span>
        <span class="foot-value">{{ tokensUsed }} / {{ tokensTotal }} tokens</span>
      </div>
      <div class="foot-line" v-if="session.accumulated">
        <span class="foot-label">{{ t("session.total_tokens") }}</span>
        <span class="foot-value">{{ formatTokens(session.accumulated) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useLocale } from "../composables/useLocale"
import { formatTokens } from "../utils/format"

export interface SessionData {
  id: string
  name: string
  model: string
  total: number
  used: number
  total_tokens?: number
  used_tokens?: number
  accumulated?: number
  columns?: number
}

const props = withDefaults(defineProps<{ session: SessionData }>(), {})
const { t } = useLocale()

const tokensUsed = computed(() => {
  if (props.session.used_tokens != null) return formatTokens(props.session.used_tokens)
  if (props.session.total === 0) return '0'
  return formatTokens(props.session.used * 10000)
})

const tokensTotal = computed(() => {
  if (props.session.total_tokens != null) return formatTokens(props.session.total_tokens)
  return formatTokens(props.session.total * 10000)
})

const cols = computed(() => {
  if (props.session.columns) return props.session.columns
  const t = props.session.total
  if (t <= 10) return 5
  if (t <= 25) return 5
  if (t <= 50) return 10
  if (t <= 100) return 10
  if (t <= 200) return 20
  return 20
})

const cells = computed(() => {
  const n = props.session.total
  const filled = Math.min(props.session.used, n)
  const colCount = cols.value
  const arr = []
  for (let i = 0; i < n; i++) {
    const ri = Math.floor(i / colCount)
    const ci = i % colCount
    const isFilled = i < filled
    const seed = ((ri * 7 + ci * 13) % 5)
    const level = isFilled ? Math.max(1, Math.min(4, seed)) : 0
    const delay = `${(ci * 0.02 + ri * 0.008).toFixed(3)}s`
    arr.push({ filled: isFilled, level, delay, cls: isFilled ? `l${level}` : '' })
  }
  return arr
})

const gridStyle = computed(() => ({ gridTemplateColumns: `repeat(${cols.value}, 1fr)` }))
</script>

<style scoped>
.session-grid-card {
  padding: 10px 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 200px;
}
.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.session-info { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.session-name { font-weight: 600; font-size: 11px; color: var(--text-1); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.session-model { font-size: 10px; color: var(--text-4); font-family: "JetBrains Mono", monospace; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.session-stats { flex-shrink: 0; }
.session-stats .pct { font-weight: 700; font-size: 13px; color: var(--text-1); }
.grid { display: grid; gap: 1.5px; }
.cell { aspect-ratio: 1; border-radius: 1px; background: var(--bg-elev-2); border: 1px solid var(--border); transition: background 0.2s; }
.cell:hover { border-color: var(--text-3); }
.cell.l1 { background: rgba(255,255,255,0.25); border-color: rgba(255,255,255,0.15); }
.cell.l2 { background: rgba(255,255,255,0.45); border-color: rgba(255,255,255,0.25); }
.cell.l3 { background: rgba(34,211,238,0.45); border-color: rgba(34,211,238,0.25); }
.cell.l4 { background: rgba(34,211,238,0.70); border-color: rgba(34,211,238,0.40); }
:global(html[data-theme="light"]) .cell { background: var(--bg-elev-2); border-color: var(--border); }
:global(html[data-theme="light"]) .cell.l1 { background: rgba(255,255,255,0.15); border-color: rgba(255,255,255,0.10); }
:global(html[data-theme="light"]) .cell.l2 { background: rgba(255,255,255,0.30); border-color: rgba(255,255,255,0.18); }
:global(html[data-theme="light"]) .cell.l3 { background: rgba(34,211,238,0.30); border-color: rgba(34,211,238,0.18); }
:global(html[data-theme="light"]) .cell.l4 { background: rgba(34,211,238,0.55); border-color: rgba(34,211,238,0.30); }
.card-foot { display: flex; flex-direction: column; gap: 2px; align-items: flex-end; }
.foot-line { display: flex; gap: 4px; align-items: center; }
.foot-label { font-size: 9px; color: var(--text-4); text-transform: uppercase; letter-spacing: 0.03em; }
.foot-value { font-size: 10px; color: var(--text-2); font-family: "JetBrains Mono", monospace; }
</style>
