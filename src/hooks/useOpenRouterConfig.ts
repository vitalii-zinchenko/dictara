import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type OpenRouterConfigStatus } from '@/bindings'

export const OPENROUTER_CONFIG_QUERY_KEY = ['openrouterConfig'] as const

/**
 * Hook to load the OpenRouter configuration status.
 * Returns whether a config exists and the model name (never exposes the API key).
 */
export function useOpenRouterConfig() {
  return useQuery({
    queryKey: OPENROUTER_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<OpenRouterConfigStatus | null> => {
      const result = await commands.loadOpenrouterConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

interface SaveOpenRouterConfigParams {
  apiKey: string
  model: string
}

/**
 * Hook to save OpenRouter configuration.
 * Invalidates the config query on success.
 */
export function useSaveOpenRouterConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveOpenRouterConfigParams): Promise<void> => {
      const result = await commands.saveOpenrouterConfig(params.apiKey, params.model)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OPENROUTER_CONFIG_QUERY_KEY })
    },
  })
}

interface TestOpenRouterConfigParams {
  apiKey: string
  model: string
}

/**
 * Hook to test OpenRouter config key and model validity.
 */
export function useTestOpenRouterConfig() {
  return useMutation({
    mutationFn: async (params: TestOpenRouterConfigParams): Promise<boolean> => {
      const result = await commands.testOpenrouterConfig(params.apiKey, params.model)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

/**
 * Hook to delete OpenRouter configuration.
 * Invalidates the config query on success.
 */
export function useDeleteOpenRouterConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.deleteOpenrouterConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: OPENROUTER_CONFIG_QUERY_KEY })
    },
  })
}
