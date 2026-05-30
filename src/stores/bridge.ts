import { ref } from 'vue'

export function useBridgeStore() {
  const status = ref<'running' | 'stopped'>('stopped')
  const url = ref('http://127.0.0.1:17761')

  const setStatus = (s: 'running' | 'stopped') => {
    status.value = s
  }

  return { status, url, setStatus }
}
