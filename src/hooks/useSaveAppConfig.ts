import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type Provider, type RecordingTrigger, type AudioFormat } from '@/bindings'

interface SaveAppConfigParams {
  activeProvider?: Provider | null
  recordingTrigger?: RecordingTrigger
  audioFormat?: AudioFormat | null
}

export function useSaveAppConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAppConfigParams): Promise<void> => {
      const result = await commands.saveAppConfig(
        params.activeProvider ?? null,
        params.recordingTrigger ?? null,
        params.audioFormat ?? null
      )
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      // Invalidate the appConfig query to refetch fresh data
      queryClient.invalidateQueries({ queryKey: ['appConfig'] })
    },
  })
}
