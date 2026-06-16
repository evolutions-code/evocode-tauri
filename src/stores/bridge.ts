import { ref } from 'vue'
import { getBridgePort } from '../api/bridge'

export function useBridgeStore() {
  const status = ref<'running' | 'stopped'>('stopped')
  const url = ref('http://127.0.0.1:17761')

  const setStatus = (s: 'running' | 'stopped') => {
    status.value = s
  }

  async function loadUrl() {
    try {
      const port = await getBridgePort()
      url.value = 'http://127.0.0.1:' + port
    } catch { /* keep default */ }
  }

  return { status, url, setStatus, loadUrl }
}
