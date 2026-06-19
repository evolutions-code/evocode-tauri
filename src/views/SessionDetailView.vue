<template>
  <div class="session-detail">
    <!-- ─── Header ─── -->
    <header class="sd-header">
      <div class="sd-header-top">
        <a-button type="text" class="sd-back-btn" @click="goBack">
          <template #icon><LeftOutlined /></template>
          {{ t("session.back") }}
        </a-button>
        <div class="sd-title-area">
          <span class="sd-title" :title="sessionTitle">{{ sessionTitle || t("session.detail") }}</span>
          <span class="sd-subtitle" v-if="sessionInfo">
            <span class="sd-model">{{ sessionInfo.model }}</span>
            <span class="sd-dot">&middot;</span>
            <span>{{ displayTokens(sessionInfo.used_tokens, sessionInfo.used) }}/{{ displayTokens(sessionInfo.total_tokens, sessionInfo.total) }}</span>
            <span class="sd-dot">&middot;</span>
            <span>{{ entries.length }} {{ t("session.entries") }}</span>
          </span>
        </div>
      </div>
    </header>

    <!-- ─── Messages ─── -->
    <section class="sd-messages" ref="messagesRef">
      <template v-for="(item, idx) in zippedEntries" :key="idx">

        <!-- User message -->
        <div v-if="item.kind === 'user'" class="msg-row msg-row-user">
          <div class="msg-stack msg-stack-user">
            <article class="msg-body msg-body-user">
              <div class="msg-card msg-card-user">
                <div class="msg-text" v-html="renderMarkdown(item.text || '')" />
              </div>
            </article>
            <span class="msg-ts">{{ formatTs(item.timestamp) }}</span>
          </div>
        </div>

        <!-- Assistant message -->
        <div v-else-if="item.kind === 'assistant'" class="msg-row msg-row-assistant">
          <div class="msg-stack msg-stack-assistant">
            <article class="msg-body msg-body-assistant">
              <div class="msg-card msg-card-assistant">
                <div class="msg-text" v-html="renderMarkdown(item.text || '')" />
              </div>
            </article>
            <span class="msg-ts">{{ formatTs(item.timestamp) }}</span>
          </div>
        </div>

        <!-- Reasoning (plain italic text, no special card - codex-mobile has no reasoning card in history) -->
        <div v-else-if="item.kind === 'reasoning'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="reasoning-text">{{ item.text }}</div>
          </div>
        </div>

        <!-- Tool call (codex-mobile cmd-row style) -->
        <div v-else-if="item.kind === 'tool_call'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="tool-block">
              <button
                type="button"
                class="cmd-row cmd-compact"
                :class="{ 'cmd-expanded': isExpanded(item.call_id || 't-' + idx) }"
                @click="toggleExpand(item.call_id || 't-' + idx)"
              >
                <span class="cmd-chevron" :class="{ 'cmd-chevron-open': isExpanded(item.call_id || 't-' + idx) }">&#9654;</span>
                <code class="cmd-label">{{ item.name || item.tool_kind || 'tool' }}</code>
                <span class="cmd-status">{{ hasOutput(item) ? 'done' : '' }}</span>
              </button>
              <div
                class="cmd-output-wrap"
                :class="{ 'cmd-output-visible': isExpanded(item.call_id || 't-' + idx) }"
              >
                <div class="cmd-output-inner">
                  <pre class="cmd-output"><code>{{ formatJsonArg(item.arguments) }}</code></pre>
                  <div v-if="item.call_id && toolOutputs.has(item.call_id)" class="cmd-output-sep">
                    <div class="tool-output-label">OUTPUT</div>
                    <pre class="cmd-output"><code>{{ toolOutputs.get(item.call_id) }}</code></pre>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Turn boundary -->
        <div v-else-if="item.kind === 'turn_boundary'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="boundary-wrap">
              <span class="boundary-line" />
              <span class="boundary-label">End of turn</span>
              <span class="boundary-line" />
            </div>
          </div>
        </div>

        <!-- Patch end -->
        <div v-else-if="item.kind === 'patch_end'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="patch-block" :class="item.success ? 'patch-ok' : 'patch-fail'">
              <span class="patch-summary">{{ item.success ? '&#x2705; Patch applied' : '&#x274C; Patch failed' }}</span>
              <div v-if="item.diffs && item.diffs.length" class="patch-files">
                <span v-for="d in item.diffs" :key="d.path" class="patch-file">{{ d.path }}</span>
              </div>
              <pre v-if="item.stdout" class="patch-stdout">{{ item.stdout.slice(0, 300) }}</pre>
            </div>
          </div>
        </div>

        <!-- Compaction -->
        <div v-else-if="item.kind === 'compaction'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <span class="compact-text">&#x1F504; Context compacted</span>
          </div>
        </div>

        <!-- Approval -->
        <div v-else-if="item.kind === 'approval'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="approval-box">&#x1F6D1; {{ item.text }}</div>
          </div>
        </div>

        <!-- File entry -->
        <div v-else-if="item.kind === 'file_entry'" class="msg-row msg-row-system">
          <div class="msg-stack msg-stack-system">
            <div class="file-chip">&#x1F4C4; {{ item.path }}</div>
          </div>
        </div>
      </template>

      <div v-if="!loading && entries.length === 0" class="sd-empty">
        <div class="empty-icon">&#x1F4AC;</div>
        <div class="empty-text">{{ t("session.empty") }}</div>
      </div>
      <div ref="scrollAnchor" class="scroll-anchor" />
    </section>

    <div v-if="loading" class="sd-loading">
      <a-spin size="small" />
      <span>Loading messages...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from "vue"
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
const expanded = ref<Set<string>>(new Set())
const messagesRef = ref<HTMLElement | null>(null)
const scrollAnchor = ref<HTMLElement | null>(null)

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

function goBack() {
  router.back()
}

function isExpanded(id: string): boolean {
  return expanded.value.has(id)
}

function toggleExpand(id: string) {
  const next = new Set(expanded.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  expanded.value = next
}

function hasOutput(item: any): boolean {
  return item.call_id && toolOutputs.value.has(item.call_id)
}

function scrollToBottom() {
  nextTick(() => {
    if (scrollAnchor.value) {
      scrollAnchor.value.scrollIntoView({ behavior: 'smooth' })
    }
  })
}

function formatTs(ts: string): string {
  if (!ts) return ""
  const d = new Date(ts)
  if (Number.isNaN(d.getTime())) return ts
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
}


function displayTokens(precise: number | undefined, cells: number | undefined): string {
  if (precise != null) return formatTokens(precise)
  if (cells == null) return "0"
  return formatTokens(cells * 10000)
}

function formatJsonArg(arg: string | undefined): string {
  if (!arg) return ""
  try {
    return JSON.stringify(JSON.parse(arg), null, 2)
  } catch {
    return arg
  }
}

function renderMarkdown(text: string): string {
  if (!text) return ""
  const escape = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
  let src = escape(text)
  src = src.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, _lang, body) => {
    const code = body.replace(/^\n/, "").replace(/\n$/, "")
    return '<pre class="md-pre"><code class="md-code-block">' + code + '</code></pre>'
  })
  src = src.replace(/`([^`\n]+)`/g, '<code class="md-code">$1</code>')
  src = src.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>")
  src = src.replace(/\*([^*\n]+)\*/g, "<em>$1</em>")
  src = src.split(/\n{2,}/).map((p) => {
    if (p.startsWith("<pre")) return p
    return '<p>' + p.replace(/\n/g, "<br>") + '</p>'
  }).join("")
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
  scrollToBottom()
})
</script>

<style scoped>
/* ─── Layout ─── */
.session-detail {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
}

/* ─── Header ─── */
.sd-header {
  flex-shrink: 0;
  padding: 12px 20px 8px;
  border-bottom: 1px solid var(--border);
}
.sd-header-top { display: flex; align-items: center; gap: 10px; }
.sd-back-btn { color: var(--text-2); padding: 4px 8px; height: auto; font-size: 13px; }
.sd-back-btn:hover { color: var(--brand-300); }
.sd-title-area { flex: 1; min-width: 0; }
.sd-title {
  display: block;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.sd-subtitle {
  display: block;
  font-size: 11px;
  color: var(--text-3);
  margin-top: 2px;
  font-family: "JetBrains Mono", monospace;
}
.sd-model { color: var(--brand-300); }
.sd-dot { margin: 0 4px; color: var(--text-4); }

/* ─── Messages Container ─── */
.sd-messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  scroll-behavior: smooth;
}
.scroll-anchor { height: 1px; }

/* ─── Message Row ─── */
.msg-row {
  position: relative;
  width: 100%;
  min-width: 0;
  max-width: min(45rem, 100%);
  margin: 0 auto;
  display: flex;
  animation: fadeIn 0.15s ease;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
.msg-row-user { justify-content: flex-end; }
.msg-row-assistant,
.msg-row-system { justify-content: flex-start; }

/* ─── Message Stack ─── */
.msg-stack { display: flex; flex-direction: column; width: 100%; min-width: 0; }
.msg-stack-user { align-items: flex-end; }
.msg-stack-assistant,
.msg-stack-system { align-items: flex-start; }

/* ─── Message Body ─── */
.msg-body { display: flex; flex-direction: column; min-width: 0; max-width: 100%; width: fit-content; }
.msg-body-user { margin-left: auto; align-items: flex-end; align-self: flex-end; }
.msg-body-assistant,
.msg-body-system { align-items: flex-start; align-self: flex-start; }

/* ─── Card (Bubble) ─── */
.msg-card { max-width: min(76ch, 100%); padding: 0; background: transparent; border: none; border-radius: 0; }
.msg-card-user {
  border-radius: 16px;
  background: rgba(107, 150, 255, 0.15);
  padding: 10px 14px;
  width: fit-content;
  max-width: min(560px, 100%);
  margin-left: auto;
  align-self: flex-end;
}
.msg-card-assistant { padding: 0; background: transparent; border: none; border-radius: 0; }

/* ─── Timestamp ─── */
.msg-ts {
  font-size: 10px;
  color: var(--text-4);
  font-family: "JetBrains Mono", monospace;
  padding: 2px 4px 0;
}
.msg-stack-user .msg-ts { text-align: right; }

/* ─── Markdown ─── */
.msg-text { font-size: 13.5px; line-height: 1.55; overflow-wrap: break-word; word-break: break-word; }
.msg-text :deep(p) { margin: 4px 0; }
.msg-text :deep(p:first-child) { margin-top: 0; }
.msg-text :deep(p:last-child) { margin-bottom: 0; }
.msg-text :deep(code) {
  background: rgba(0,0,0,0.3);
  padding: 1px 5px;
  border-radius: 4px;
  font-family: "JetBrains Mono", monospace;
  font-size: 12px;
}
.msg-text :deep(pre) {
  margin: 8px 0;
  padding: 10px;
  border-radius: 8px;
  background: rgba(0,0,0,0.25);
  border: 1px solid var(--border);
  overflow-x: auto;
  font-family: "JetBrains Mono", monospace;
  font-size: 12px;
  line-height: 1.45;
}
.msg-text :deep(pre code) { background: none; padding: 0; }
.msg-text :deep(strong) { font-weight: 600; color: var(--text-1); }
.msg-text :deep(em) { font-style: italic; }

/* ─── Reasoning (simple italic text) ─── */
.reasoning-text {
  font-style: italic;
  color: var(--text-3);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  padding: 2px 0;
}

/* ─── Tool Call (codex-mobile cmd-row style) ─── */
.tool-block { width: 100%; }
.cmd-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-elev-2);
  cursor: pointer;
  transition: all 0.15s;
  text-align: left;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-1);
}
.cmd-row:hover { background: var(--bg-elev-3); }
.cmd-row.cmd-compact { gap: 6px; padding: 5px 10px; border-radius: 8px; }
.cmd-chevron {
  font-size: 9px;
  color: var(--text-4);
  transition: transform 0.15s;
  flex-shrink: 0;
}
.cmd-chevron-open { transform: rotate(90deg); }
.cmd-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--brand-300);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}
.cmd-status {
  font-size: 11px;
  color: var(--text-4);
  flex-shrink: 0;
  max-width: 4.5rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cmd-output-wrap {
  display: none;
  margin-top: 4px;
}
.cmd-output-visible { display: block; }
.cmd-output-inner {
  padding: 8px 12px;
  border-radius: 8px;
  background: rgba(0,0,0,0.2);
  border: 1px solid var(--border);
}
.cmd-output {
  margin: 0;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-2);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 300px;
  overflow-y: auto;
}
.cmd-output-sep { margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--border); }
.tool-output-label {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-4);
  margin-bottom: 4px;
  font-family: "JetBrains Mono", monospace;
}

/* ─── Turn Boundary ─── */
.boundary-wrap {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 0;
  margin: 6px 0;
}
.boundary-line { flex: 1; height: 1px; background: var(--border); }
.boundary-label {
  font-size: 10px;
  color: var(--text-4);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  white-space: nowrap;
  padding: 0 8px;
}

/* ─── Patch ─── */
.patch-block {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  border-left: 3px solid #f472b6;
  background: rgba(244, 114, 182, 0.06);
}
.patch-block.patch-ok { border-left-color: #34d399; background: rgba(52, 211, 153, 0.06); }
.patch-summary { font-weight: 600; font-size: 12px; }
.patch-files { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px; }
.patch-file {
  font-size: 11px;
  color: #6b96ff;
  font-family: "JetBrains Mono", monospace;
  background: rgba(107, 150, 255, 0.08);
  padding: 1px 6px;
  border-radius: 4px;
}
.patch-stdout { margin: 4px 0 0; font-size: 11px; color: var(--text-3); font-family: "JetBrains Mono", monospace; white-space: pre-wrap; max-height: 120px; overflow-y: auto; }

/* ─── Compaction ─── */
.compact-text { font-size: 11px; color: var(--text-4); font-style: italic; }

/* ─── Approval ─── */
.approval-box {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.06);
  border: 1px solid rgba(251, 191, 36, 0.15);
}

/* ─── File Chip ─── */
.file-chip {
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 12px;
  color: #6b96ff;
  background: var(--bg-elev-1);
  border: 1px solid var(--border);
  font-family: "JetBrains Mono", monospace;
}

/* ─── Empty / Loading ─── */
.sd-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--text-4); gap: 8px; }
.empty-icon { font-size: 32px; }
.empty-text { font-size: 14px; }
.sd-loading {
  position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
  display: flex; align-items: center; gap: 8px; color: var(--text-3); font-size: 13px;
}
</style>