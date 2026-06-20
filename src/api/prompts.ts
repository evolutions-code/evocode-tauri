import { invoke } from '@tauri-apps/api/core'

export interface PromptFile {
  name: string
  updated_at: string
}

export async function readAgentsFile(): Promise<string> {
  return invoke<string>('read_agents_file')
}

export async function writeAgentsFile(content: string): Promise<void> {
  return invoke<void>('write_agents_file', { content })
}

export async function listPromptFiles(): Promise<PromptFile[]> {
  return invoke<PromptFile[]>('list_prompt_files')
}

export async function readPromptFile(name: string): Promise<string> {
  return invoke<string>('read_prompt_file', { name })
}

export async function writePromptFile(name: string, content: string): Promise<void> {
  return invoke<void>('write_prompt_file', { name, content })
}

export async function readCorePrompt(): Promise<string> {
  return invoke<string>('read_core_prompt')
}

export async function deletePromptFile(name: string): Promise<void> {
  return invoke<void>('delete_prompt_file', { name })
}