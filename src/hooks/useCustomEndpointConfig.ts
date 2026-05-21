import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type CustomEndpointConfigStatus } from '@/bindings'

export const CUSTOM_ENDPOINT_CONFIG_QUERY_KEY = ['customEndpointConfig'] as const

/**
 * Hook to load the Custom Endpoint configuration status.
 * Returns whether a config exists, the base URL, the model name, and if it has an API key.
 */
export function useCustomEndpointConfig() {
  return useQuery({
    queryKey: CUSTOM_ENDPOINT_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<CustomEndpointConfigStatus | null> => {
      const result = await commands.loadCustomEndpointConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

interface SaveCustomEndpointConfigParams {
  apiKey: string | null
  baseUrl: string
  model: string
}

/**
 * Hook to save Custom Endpoint configuration.
 * Invalidates the config query on success.
 */
export function useSaveCustomEndpointConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveCustomEndpointConfigParams): Promise<void> => {
      const result = await commands.saveCustomEndpointConfig(
        params.apiKey,
        params.baseUrl,
        params.model
      )
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CUSTOM_ENDPOINT_CONFIG_QUERY_KEY })
    },
  })
}

interface TestCustomEndpointConfigParams {
  apiKey: string | null
  baseUrl: string
  model: string
}

/**
 * Hook to test Custom Endpoint credentials validity.
 */
export function useTestCustomEndpointConfig() {
  return useMutation({
    mutationFn: async (params: TestCustomEndpointConfigParams): Promise<boolean> => {
      const result = await commands.testCustomEndpointConfig(
        params.apiKey,
        params.baseUrl,
        params.model
      )
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

/**
 * Hook to delete Custom Endpoint configuration.
 * Invalidates the config query on success.
 */
export function useDeleteCustomEndpointConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.deleteCustomEndpointConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CUSTOM_ENDPOINT_CONFIG_QUERY_KEY })
    },
  })
}
