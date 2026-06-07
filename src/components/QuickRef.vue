<template>
  <div class="quick-ref glass fade-up">
    <div class="head">
      <div class="title">
        <span class="bar" />
        <span>Quick Reference</span>
      </div>
      <a-tag class="tag">3 endpoints</a-tag>
    </div>
    <div class="grid">
      <div
        v-for="ep in endpoints"
        :key="ep.path"
        class="ep"
      >
        <a-tag class="method" :color="ep.color">{{ ep.method }}</a-tag>
        <code class="mono path">{{ ep.path }}</code>
        <a-tooltip title="Copy">
          <CopyOutlined class="copy" @click="copy(ep.path)" />
        </a-tooltip>
        <div class="desc muted-3">{{ ep.desc }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { CopyOutlined } from "@ant-design/icons-vue"

const endpoints = [
  { method: "POST", path: "/v1/chat/completions", color: "blue", desc: "OpenAI Chat Completions" },
  { method: "POST", path: "/v1/messages",         color: "purple", desc: "Anthropic Messages" },
  { method: "POST", path: "/responses",          color: "cyan", desc: "OpenAI Responses" },
]

function copy(text: string) {
  navigator.clipboard?.writeText(text).catch(() => {})
}
</script>

<style scoped>
.quick-ref { padding: 16px 18px; }

.head {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 12px;
}
.title { display: inline-flex; align-items: center; gap: 10px; font-weight: 600; color: var(--text-1); }
.title .bar {
  width: 3px; height: 14px; border-radius: 2px;
  background: linear-gradient(180deg, #8b5cf6, #4d7dff);
}
.tag { border-radius: 999px; }

.grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 8px;
}

.ep {
  display: grid;
  grid-template-columns: auto 1fr auto;
  grid-template-rows: auto auto;
  align-items: center;
  column-gap: 10px;
  row-gap: 2px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--bg-elev-2);
  transition: border-color .15s ease, transform .15s ease;
}
.ep:hover { border-color: var(--border-strong); transform: translateY(-1px); }
.ep .method { grid-row: 1; grid-column: 1; }
.ep .path { grid-row: 1; grid-column: 2; color: var(--text-1); font-size: 13px; }
.ep .copy { grid-row: 1; grid-column: 3; color: var(--text-3); cursor: pointer; padding: 4px; }
.ep .copy:hover { color: var(--brand-300); }
.ep .desc { grid-row: 2; grid-column: 2 / span 2; font-size: 12px; }

@media (min-width: 720px) {
  .grid { grid-template-columns: 1fr 1fr 1fr; }
}
</style>
