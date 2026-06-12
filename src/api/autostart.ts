import { invoke } from '@tauri-apps/api/core'

export async function isAutostartEnabled(): Promise<boolean> {
  return invoke<boolean>('plugin:autostart|is_enabled')
}

export async function enableAutostart(): Promise<void> {
  await invoke('plugin:autostart|enable')
}

export async function disableAutostart(): Promise<void> {
  await invoke('plugin:autostart|disable')
}
