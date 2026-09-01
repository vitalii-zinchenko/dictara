import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type OpenAIConfigStatus, type OpenAITranscriptionModel } from '@/bindings'

export const OPENAI_CONFIG_QUERY_KEY = ['openaiConfig'] as const

/**
 * Hook to load the OpenAI configuration status.
 * Returns whether a config exists (never exposes the API key).
 */
export function useOpenAIConfig() {
  return useQuery({
    queryKey: OPENAI_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<OpenAIConfigStatus | null> => {
      const result = await commands.loadOpenaiConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

interface SaveOpenAIConfigParams {
  /** Omit to keep the already-stored key (model-only update). */
  apiKey?: string
  model: OpenAITranscriptionModel
}

/**
 * Hook to save OpenAI configuration.
 *
 * Passing no `apiKey` keeps the key already in the keychain, so the model can
 * be changed without the user re-entering it.
 * Invalidates the config query on success.
 */
export function useSaveOpenAIConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveOpenAIConfigParams): Promise<void> => {
      const result = await commands.saveOpenaiConfig(params.apiKey ?? null, params.model)
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
 *
 * Validates the credential only - the backend always probes with Whisper, so
 * the result does not depend on the selected model.
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
