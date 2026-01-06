import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { commands, events, type ModelInfo, type LocalModelConfig } from '@/bindings'

export const MODELS_QUERY_KEY = ['models'] as const
export const LOCAL_MODEL_CONFIG_QUERY_KEY = ['localModelConfig'] as const
export const LOADED_MODEL_QUERY_KEY = ['loadedModel'] as const

/**
 * Hook to get all available models with their current status.
 * Automatically refreshes when download/load state changes.
 * Note: Progress events are handled separately in ModelSelector via local state,
 * so we only invalidate on complete/error events to avoid excessive re-fetching.
 */
export function useAvailableModels() {
  const queryClient = useQueryClient()

  // Listen for state change events (using discriminated unions)
  useEffect(() => {
    // Download state changes - invalidate on complete or error
    const unlistenDownload = events.modelDownloadStateChanged.listen((event) => {
      if (event.payload.state === 'complete' || event.payload.state === 'error') {
        queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
      }
    })

    // Loading state changes - invalidate on any state change
    const unlistenLoading = events.modelLoadingStateChanged.listen(() => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: LOADED_MODEL_QUERY_KEY })
    })

    return () => {
      unlistenDownload.then((fn) => fn())
      unlistenLoading.then((fn) => fn())
    }
  }, [queryClient])

  return useQuery({
    queryKey: MODELS_QUERY_KEY,
    queryFn: async (): Promise<ModelInfo[]> => {
      return await commands.getAvailableModels()
    },
  })
}

/**
 * Hook to download a model.
 * Invalidates models query on success.
 */
export function useDownloadModel() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (modelName: string): Promise<void> => {
      const result = await commands.downloadModel(modelName)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
    },
  })
}

/**
 * Hook to cancel an ongoing model download.
 */
export function useCancelModelDownload() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (modelName: string): Promise<void> => {
      const result = await commands.cancelModelDownload(modelName)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
    },
  })
}

/**
 * Hook to delete a downloaded model.
 * Invalidates models query on success.
 */
export function useDeleteModel() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (modelName: string): Promise<void> => {
      const result = await commands.deleteModel(modelName)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
    },
  })
}

/**
 * Hook to load a model into memory for transcription.
 * Invalidates models and loaded model queries on success.
 */
export function useLoadModel() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (modelName: string): Promise<void> => {
      const result = await commands.loadModel(modelName)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: LOADED_MODEL_QUERY_KEY })
    },
  })
}

/**
 * Hook to unload the currently loaded model.
 */
export function useUnloadModel() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      await commands.unloadModel()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: MODELS_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: LOADED_MODEL_QUERY_KEY })
    },
  })
}

/**
 * Hook to get the name of the currently loaded model.
 */
export function useLoadedModel() {
  return useQuery({
    queryKey: LOADED_MODEL_QUERY_KEY,
    queryFn: async (): Promise<string | null> => {
      return await commands.getLoadedModel()
    },
  })
}

/**
 * Hook to load local model configuration (selected model).
 */
export function useLocalModelConfig() {
  return useQuery({
    queryKey: LOCAL_MODEL_CONFIG_QUERY_KEY,
    queryFn: async (): Promise<LocalModelConfig | null> => {
      const result = await commands.loadLocalModelConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}

/**
 * Hook to save local model configuration (selected model).
 * Invalidates the config query on success.
 */
export function useSaveLocalModelConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (modelName: string): Promise<void> => {
      const result = await commands.saveLocalModelConfig(modelName)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: LOCAL_MODEL_CONFIG_QUERY_KEY })
    },
  })
}

/**
 * Hook to delete local model configuration.
 * Invalidates the config query on success.
 */
export function useDeleteLocalModelConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.deleteLocalModelConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: LOCAL_MODEL_CONFIG_QUERY_KEY })
    },
  })
}
