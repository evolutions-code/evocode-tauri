<template>
  <div class="session-detail">
    <div class="session-header">
      <section class="page-header">
        <a-button type="text" class="back-btn" @click="goBack()">
          <template #icon><LeftOutlined /></template>
          {{ t("session.back") }}
        </a-button>
        <div class="page-title">
          <span class="bar" />
          <span :title="sessionTitle">{{ sessionTitle || t("session.detail") }}</span>
        </div>
        <div class="header-spacer" />
        <a-segmented
          v-model:value="viewMode"
          :options="viewOptions"
          size="small"
        />
      </section>

      <section class="session-meta" v-if="sessionInfo">
        <div class="meta-item">
          <span class="meta-label">{{ t("session.model") }}</span>
          <span class="meta-value">{{ sessionInfo.model }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">{{ t("session.tokens") }}</span>
          <span class="meta-value">{{ displayTokens(sessionInfo.used_tokens, sessionInfo.used) }} / {{ displayTokens(sessionInfo.total_tokens, sessionInfo.total) }}</span>
        </div>
        <div class="meta-item" v-if="sessionInfo.accumulated">
          <span class="meta-label">{{ t("session.total_tokens") }}</span>
          <span class="meta-value">{{ formatTokens(sessionInfo.accumulated) }}</span>
        </div>
        <div class="meta-item" v-if="entries.length">
          <span class="meta-label">{{ t("session.entries") || "Entries" }}</span>
          <span class="meta-value">{{ entries.length }}</span>
        </div>
      </section>
    </div>

    <section v-if="viewMode === 'chat'" class="thread">
      <template v-for="(item, _idx) in zippedEntries" :key="_idx">
        <div class="entry-card" :class="['entry-' + (item as any).kind, (item as any).kind === 'user' ? 'msg-right' : (item as any).kind === 'assistant' ? 'msg-left' : '']">
          <div class="entry-head" @click="toggleTool((item as any).call_id ?? `e-${_idx}`)" style="cursor:pointer">
            <span class="entry-kind" :class="'kind-' + (item as any).kind">{{ (item as any).kind }}</span>
            <span class="entry-name" v-if="(item as any).name">{{ (item as any).name }}</span>
            <span class="entry-callid" v-if="(item as any).call_id">{{ shortId((item as any).call_id) }}</span>
            <span class="collapse-arrow">{{ expandedTools.has((item as any).call_id ?? `e-${_idx}`) ? '▾' : '▸' }}</span>
            <span class="ts" v-if="(item as any).timestamp">{{ formatTs((item as any).timestamp) }}</span>
          </div>
          <div v-if="expandedTools.has((item as any).call_id ?? `e-${_idx}`) || (item as any).kind === 'user' || (item as any).kind === 'assistant'" class="entry-body">
            <div v-if="(item as any).kind === 'user' || (item as any).kind === 'assistant'" class="md" v-html="renderMarkdown((item as any).text || '')" />
            <div v-else-if="(item as any).kind === 'reasoning'" class="reasoning-text">{{ (item as any).text }}</div>
            <pre v-else class="entry-json"><code>{{ formatEntry(item as any) }}</code></pre>
          </div>
          <div v-if="expandedTools.has((item as any).call_id ?? `e-${_idx}`) && (item as any).call_id && toolOutputs.has((item as any).call_id)" class="entry-output">
            <div class="output-label">OUTPUT</div>
            <pre><code class="output-content">{{ toolOutputs.get((item as any).call_id) }}</code></pre>
          </div>
        </div>
      </template>
      <div v-if="!loading && entries.length === 0" class="empty-state">
        <div class="empty-title">No entries</div>
      </div>
    </section>

    <section v-else class="raw-pane">
      <pre class="raw-content">{{ rawContent }}</pre>
    </section>
  </div>
</template>
<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { LeftOutlined } from "@ant-design/icons-vue"
import { useLocale } from "../composables/useLocale"
import { formatTokens } from "../utils/format"
import { getSessionContent, getSessions } from "../api/bridge"
import type { SessionInfo, SessionEntry } from "../api/bridge"

const { t } = useLocale()
const route = useRoute()
const router = useRouter()

const sessionId = route.params.id as string
const sessionInfo = ref<SessionInfo | null>(null)
const sessionTitle = ref("")
const entries = ref<SessionEntry[]>([])
const loading = ref(false)
const viewMode = ref<"chat" | "raw">("chat")
const viewOptions = [
  { label: "Chat", value: "chat" },
  { label: "Raw", value: "raw" },
]
const rawContent = ref("")
const expandedTools = ref<Set<string>>(new Set())

function goBack() {
  router.back()
}

function toggleTool(id: string) {
  const next = new Set(expandedTools.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  expandedTools.value = next
}

function displayTokens(precise: number | undefined, cells: number | undefined): string {
  if (precise != null) return formatTokens(precise)
  if (cells == null) return "0"
  return formatTokens(cells * 10000)
}

const toolOutputs = computed(() => {
  const m = new Map<string, string>()
  for (const e of entries.value) {
    if (e.kind === "tool_output") m.set((e as any).call_id, (e as any).output)
  }
  return m
})

const zippedEntries = computed(() =>
  entries.value.filter(e => e.kind !== 'tool_output')
)

function formatTs(ts: string): string {
  if (!ts) return ""
  const d = new Date(ts)
  if (Number.isNaN(d.getTime())) return ts
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })
}

function shortId(id: string): string {
  if (!id) return ""
  return id.length > 16 ? id.slice(0, 8) + "…" + id.slice(-4) : id
}

function formatEntry(item: any): string {
  if (!item) return ""
  var obj: any = {}
  for (var k in item) {
    if (k === "kind" || k === "timestamp") continue
    var v = item[k]
    if (k === "arguments" && typeof v === "string") {
      try { v = JSON.parse(v) } catch(e) {}
    }
    obj[k] = v
  }
  return JSON.stringify(obj, null, 2)
}

function renderMarkdown(text: string): string {
  if (!text) return ""
  const escape = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
  let src = escape(text)
  src = src.replace(/```([\s\S]*?)```/g, (_, body) => `<pre class="md-pre"><code>${body.replace(/^\n/, "").replace(/\n$/, "")}</code></pre>`)
  src = src.replace(/`([^`\n]+)`/g, "<code class=\"md-code\">$1</code>")
  src = src.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>")
  src = src.split(/\n{2,}/).map((p) => { if (p.startsWith("<pre")) return p; return `<p>${p.replace(/\n/g, "<br>")}</p>` }).join("")
  return src
}

onMounted(async () => {
  try {
    const allSessions = await getSessions(0, 1000)
    const found = allSessions.sessions.find(s => s.id === sessionId)
    if (found) {
      sessionInfo.value = found
      sessionTitle.value = found.name
    }
  } catch {}
  loading.value = true
  try {
    entries.value = await getSessionContent(sessionId)
  } catch (e: any) {
    console.error("Failed to load session:", e)
    entries.value = []
  } finally {
    loading.value = false
  }
  rawContent.value = entries.value.map((e) => JSON.stringify(e)).join("\n")
})
</script>
<style scoped>
.session-detail { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; gap: 0; }
.session-header { flex-shrink: 0; display: flex; flex-direction: column; gap: 10px; padding-bottom: 10px; }
.back-btn { color: var(--text-2); }
.back-btn:hover { color: var(--brand-300); }
.header-spacer { flex: 1; }
.session-meta {
  display: flex; gap: 24px; flex-wrap: wrap;
  padding: 12px 16px;
  background: var(--bg-elev-2);
  border-radius: var(--r-lg);
  border: 1px solid var(--border);
}
.meta-item { display: flex; gap: 8px; align-items: center; }
.meta-label { font-size: 12px; color: var(--text-3); }
.meta-value { font-size: 12px; color: var(--text-1); font-family: "JetBrains Mono", monospace; }
.thread { flex: 1; overflow-y: auto; min-height: 0; padding: 4px 0; }
.entry-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elev-2);
  overflow: hidden;
  max-width: 100%;
}
.msg-right { align-self: flex-end; max-width: 85%; }
.msg-left { align-self: flex-start; max-width: 85%; }
.entry-card.entry-tool_call {
  border-color: rgba(79, 124, 255, 0.3);
  border-left: 3px solid #4f7cff;
}
.entry-card.entry-reasoning {
  border-color: rgba(251, 191, 36, 0.2);
  border-left: 3px solid #fbbf24;
}
.entry-card.entry-turn_boundary {
  border: none;
  background: transparent;
}
.entry-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  border-bottom: 1px solid var(--border);
  background: rgba(255,255,255,0.02);
}
.entry-kind {
  display: inline-block;
  padding: 1px 8px;
  border-radius: 4px;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.kind-tool_call { background: rgba(79,124,255,0.2); color: #6b96ff; }
.kind-reasoning { background: rgba(251,191,36,0.2); color: #fbbf24; }
.kind-user { background: rgba(52,211,153,0.2); color: #34d399; }
.kind-assistant { background: rgba(255,255,255,0.12); color: #f5f7fb; }
.kind-turn_boundary { background: transparent; color: var(--text-4); font-weight: 400; }
.kind-patch_end { background: rgba(236,72,153,0.2); color: #ec4899; }
.entry-name { font-weight: 600; color: var(--text-1); }
.entry-callid { color: var(--text-4); font-size: 10px; }
.collapse-arrow { color: var(--text-4); font-size: 10px; }
.entry-body { padding: 8px 10px; font-size: 12px; color: var(--text-2); }
.entry-body :deep(p) { margin: 4px 0; }
.reasoning-text { font-style: italic; color: var(--text-4); line-height: 1.5; white-space: pre-wrap; }
.entry-json {
  margin: 0;
  background: rgba(0,0,0,0.25);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-2);
  max-height: 400px;
  overflow-y: auto;
}
.entry-output {
  border-top: 1px solid var(--border);
  padding: 6px 10px;
}
.output-label {
  font-size: 9px;
  text-transform: uppercase;
  color: var(--text-4);
  letter-spacing: 0.06em;
  margin-bottom: 4px;
  font-family: "JetBrains Mono", monospace;
}
.entry-output pre {
  margin: 0;
  background: rgba(0,0,0,0.2);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 8px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-2);
  max-height: 240px;
  overflow-y: auto;
}
.ts { font-size: 10px; color: var(--text-4); font-family: "JetBrains Mono", monospace; margin-left: auto; }
.raw-pane { flex: 1; min-height: 0; overflow: auto; }
.raw-content {
  margin: 0;
  padding: 16px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  font-family: "JetBrains Mono", monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-2);
}
.thread > * + * { margin-top: 8px; }
.empty-state { padding: 40px 20px; text-align: center; color: var(--text-4); }
.empty-title { font-size: 14px; }
</style>