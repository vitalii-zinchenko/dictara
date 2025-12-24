import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type AzureOpenAIConfig } from '@/bindings'

export const AZURE_OPENAI_CONFIG_QUERY_KEY = ['azureOpenaiConfig'] as const

/**
 * Hook to load the Azure OpenAI configuration.
 * Provides caching, loading states, and automatic error handling.
 */
export function useAzureOpenAIConfig() {
  return useQuery({
    queryKey: AZURE_OPENAI_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<AzureOpenAIConfig | null> => {
      const result = await commands.loadAzureOpenaiConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

interface SaveAzureOpenAIConfigParams {
  apiKey: string
  endpoint: string
}

/**
 * Hook to save Azure OpenAI configuration.
 * Invalidates the config query on success.
 */
export function useSaveAzureOpenAIConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAzureOpenAIConfigParams): Promise<void> => {
      const result = await commands.saveAzureOpenaiConfig(params.apiKey, params.endpoint)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: AZURE_OPENAI_CONFIG_QUERY_KEY })
    },
  })
}

interface TestAzureOpenAIConfigParams {
  apiKey: string
  endpoint: string
}

/**
 * Hook to test Azure OpenAI configuration validity.
 */
export function useTestAzureOpenAIConfig() {
  return useMutation({
    mutationFn: async (params: TestAzureOpenAIConfigParams): Promise<boolean> => {
      const result = await commands.testAzureOpenaiConfig(params.apiKey, params.endpoint)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

/**
 * Hook to delete Azure OpenAI configuration.
 * Invalidates the config query on success.
 */
export function useDeleteAzureOpenAIConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.deleteAzureOpenaiConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: AZURE_OPENAI_CONFIG_QUERY_KEY })
    },
  })
}
