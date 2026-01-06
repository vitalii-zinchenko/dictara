export type Provider = 'open_ai' | 'azure_open_ai' | 'local' | null

export interface AppConfig {
  active_provider: Provider
}

export interface OpenAIConfig {
  api_key: string
}

export interface AzureOpenAIConfig {
  api_key: string
  endpoint: string
}

export interface ProviderFormState {
  testResult: 'success' | 'error' | null
  saveSuccess: boolean
  errorMessage: string | null
  isTesting: boolean
  isLoading: boolean
}
