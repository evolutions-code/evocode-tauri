<template>
  <div class="default-layout">
    <header class="toolbar">
      <div class="toolbar-left">
        <span class="brand">evocode</span>
      </div>
      <div class="toolbar-right">
        <router-link to="/config" class="settings-btn">
          <SettingOutlined />
        </router-link>
      </div>
    </header>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { SettingOutlined } from '@ant-design/icons-vue'
</script>

<style scoped>
.default-layout {
  min-height: 100vh;
  background: #0f0f0f;
  color: #e0e0e0;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid #222;
  background: #111;
  position: sticky;
  top: 0;
  z-index: 100;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.brand {
  font-size: 16px;
  font-weight: 600;
  color: #fff;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  color: #888;
  border-radius: 6px;
  transition: color 0.2s, background 0.2s;
  text-decoration: none;
}

.settings-btn:hover {
  color: #fff;
  background: #222;
}
</style>
