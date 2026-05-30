export interface BridgeState {
  status: 'running' | 'stopped'
  url: string
}

export interface ConfigProvider {
  base_url: string
  wire_api: string
  model: string
  models: string[]
  api_key: string
  api_key_header: string
}
