<template>
  <div class="quick-ref">
    <h3>Quick Reference</h3>
    <div class="endpoints">
      <div class="endpoint">
        <span class="method">POST</span>
        <code>/v1/chat/completions</code>
      </div>
      <div class="endpoint">
        <span class="method">POST</span>
        <code>/v1/messages</code>
      </div>
      <div class="endpoint">
        <span class="method">POST</span>
        <code>/responses</code>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-ref {
  background: #1a1a1a;
  border-radius: 10px;
  padding: 16px;
  margin-top: 20px;
}

h3 {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 12px;
  color: #888;
}

.endpoints {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.endpoint {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
}

.method {
  color: #60a5fa;
  font-family: monospace;
  font-size: 12px;
}

code {
  color: #e0e0e0;
  font-family: monospace;
}
</style>
