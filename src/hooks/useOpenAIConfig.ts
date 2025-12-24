import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type OpenAIConfig } from '@/bindings'

export const OPENAI_CONFIG_QUERY_KEY = ['openaiConfig'] as const

/**
 * Hook to load the OpenAI configuration.
 * Provides caching, loading states, and automatic error handling.
 */
export function useOpenAIConfig() {
  return useQuery({
    queryKey: OPENAI_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<OpenAIConfig | null> => {
      const result = await commands.loadOpenaiConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

interface SaveOpenAIConfigParams {
  apiKey: string
}

/**
 * Hook to save OpenAI configuration.
 * Invalidates the config query on success.
 */
export function useSaveOpenAIConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveOpenAIConfigParams): Promise<void> => {
      const result = await commands.saveOpenaiConfig(params.apiKey)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OPENAI_CONFIG_QUERY_KEY })
    },
  })
}

interface TestOpenAIConfigParams {
  apiKey: string
}

/**
 * Hook to test OpenAI API key validity.
 */
export function useTestOpenAIConfig() {
  return useMutation({
    mutationFn: async (params: TestOpenAIConfigParams): Promise<boolean> => {
      const result = await commands.testOpenaiConfig(params.apiKey)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

/**
 * Hook to delete OpenAI configuration.
 * Invalidates the config query on success.
 */
export function useDeleteOpenAIConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.deleteOpenaiConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OPENAI_CONFIG_QUERY_KEY })
    },
  })
}
