<template>
  <div class="about-view">
    <section class="hero glass fade-up">
      <div class="hero-content">
        <div class="eyebrow">
          <span class="dot" />
          <span>{{ t("hero.eyebrow") }}</span>
        </div>
        <h1>
         <span class="gradient-text">evocode</span><br />
        </h1>
        <p class="lead" v-html="t('hero.lead', { url: '<code class=\'mono\'>http://127.0.0.1:17761</code>', name: '<span class=\'gradient-text\'>evocode</span>', c: '<code class=\'mono\'>/v1/chat/completions</code>', m: '<code class=\'mono\'>/v1/messages</code>', r: '<code class=\'mono\'>/responses</code>' })" />

        <div class="hero-stats">
          <div class="stat">
            <div class="num mono">17761</div>
            <div class="lbl">{{ t("hero.default_port") }}</div>
          </div>
          <div class="sep" />
          <div class="stat">
            <div class="num">v<span class="mono">{{ currentVersion || '0.0.0' }}</span></div>
            <div class="lbl">{{ t("hero.bridge_version") }}</div>
          </div>
        </div>
      </div>

      <div class="hero-art" aria-hidden="true">
        <div class="orb orb-a" />
        <div class="orb orb-b" />
        <div class="orb orb-c" />
        <div class="grid-bg" />
        <div class="card-stack">
          <div class="mini-card">
            <span class="kv-key">model_provider</span>
            <code class="mono">"anthropic"</code>
          </div>
          <div class="mini-card">
            <span class="kv-key">base_url</span>
            <code class="mono">http://127.0.0.1:17761</code>
          </div>
          <div class="mini-card">
            <span class="kv-key">wire_api</span>
            <code class="mono">"responses"</code>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"

import { useLocale } from "../composables/useLocale"
import { getAppVersion } from "../api/bridge"

const { t } = useLocale()
const currentVersion = ref("")

onMounted(async () => {
  try {
    currentVersion.value = await getAppVersion()
  } catch {}
})
</script>

<style scoped>
.about-view {
  max-width: 900px;
  margin: 0 auto;
}

.hero {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 20px;
  padding: 28px 28px;
  border-radius: var(--r-xl);
}
.hero-content { display: flex; flex-direction: column; gap: 14px; }
.eyebrow {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 4px 10px; border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-3); font-size: 12px; width: max-content;
  border: 1px solid var(--border);
}
.eyebrow .dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--ok); box-shadow: 0 0 8px var(--ok);
}
.hero h1 {
  font-size: clamp(26px, 3.6vw, 36px);
  line-height: 1.15; font-weight: 700; margin: 4px 0 0;
  color: var(--text-1);
}
.lead { color: var(--text-3); max-width: 58ch; }
.lead code { color: var(--brand-300); background: var(--bg-elev-3); padding: 1px 6px; border-radius: 6px; font-size: 12.5px; }

.hero-stats {
  display: inline-flex; align-items: center; gap: 18px;
  margin-top: 6px; padding: 10px 14px; border-radius: var(--r-md);
  background: var(--bg-elev-2); border: 1px solid var(--border); width: max-content;
}
.stat { display: flex; flex-direction: column; align-items: flex-start; }
.stat .num { font-size: 18px; font-weight: 700; color: var(--text-1); }
.stat .lbl { font-size: 11px; color: var(--text-3); text-transform: uppercase; letter-spacing: .8px; }
.sep { width: 1px; height: 24px; background: var(--border); }

.hero-art {
  position: relative; min-height: 220px; border-radius: var(--r-lg);
  background: linear-gradient(135deg, rgba(77,125,255,0.12), rgba(139,92,246,0.08));
  border: 1px solid var(--border); overflow: hidden;
}
.grid-bg {
  position: absolute; inset: 0;
  background-image:
    linear-gradient(rgba(255,255,255,0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.05) 1px, transparent 1px);
  background-size: 22px 22px;
  mask-image: radial-gradient(circle at 60% 50%, black 0%, transparent 70%);
}
.orb { position: absolute; border-radius: 50%; filter: blur(40px); opacity: .6; }
.orb-a { width: 180px; height: 180px; background: #4d7dff; top: -40px; right: -20px; }
.orb-b { width: 140px; height: 140px; background: #8b5cf6; bottom: -30px; left: 10px; }
.orb-c { width: 100px; height: 100px; background: #22d3ee; top: 30%; left: 40%; opacity: .35; }

.card-stack {
  position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center;
  gap: 10px; padding: 18px 22px;
}
.mini-card {
  display: flex; align-items: center; justify-content: space-between; gap: 12px;
  padding: 10px 12px; border-radius: 10px;
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(10px);
  animation: float 6s ease-in-out infinite;
}
.mini-card:nth-child(2) { margin-left: 28px; animation-delay: .6s; }
.mini-card:nth-child(3) { margin-left: 14px; animation-delay: 1.2s; }
.mini-card .kv-key {
  font-size: 11px; color: var(--text-3);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  text-transform: uppercase; letter-spacing: .8px;
}
.mini-card code { color: var(--text-1); font-size: 12.5px; }
@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}

@media (max-width: 780px) {
  .hero { grid-template-columns: 1fr; }
  .hero-art { min-height: 180px; }
}
</style>

