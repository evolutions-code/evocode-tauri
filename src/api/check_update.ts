import { invoke } from '@tauri-apps/api/core'

export interface CheckUpdateResult {
  hasUpdate: boolean
  latestVersion: string
  currentVersion: string
  releaseUrl: string
}

export async function checkUpdate(): Promise<CheckUpdateResult> {
  const raw = await invoke<string>('check_update')
  const parts = raw.split('__')
  if (parts[0] === 'update_available') {
    return {
      hasUpdate: true,
      latestVersion: parts[1],
      currentVersion: parts[2],
      releaseUrl: parts[3],
    }
  }
  return {
    hasUpdate: false,
    latestVersion: '',
    currentVersion: parts[1] ?? '',
    releaseUrl: '',
  }
}
