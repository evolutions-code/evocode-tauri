import { ref, watchEffect, readonly } from 'vue'

export type ThemeMode = 'dark' | 'light'

const STORAGE_KEY = 'evocode.theme'
const DEFAULT_THEME: ThemeMode = 'dark'

function readInitial(): ThemeMode {
  if (typeof window === 'undefined') return DEFAULT_THEME
  const stored = window.localStorage.getItem(STORAGE_KEY) as ThemeMode | null
  if (stored === 'dark' || stored === 'light') return stored
  return DEFAULT_THEME
}

const theme = ref<ThemeMode>(readInitial())

function applyTheme(mode: ThemeMode) {
  if (typeof document === 'undefined') return
  document.documentElement.setAttribute('data-theme', mode)
}

watchEffect(() => {
  applyTheme(theme.value)
  if (typeof window !== 'undefined') {
    window.localStorage.setItem(STORAGE_KEY, theme.value)
  }
})

export function useTheme() {
  function toggle() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }
  function set(mode: ThemeMode) {
    theme.value = mode
  }
  return {
    theme: readonly(theme),
    toggle,
    set,
  }
}
