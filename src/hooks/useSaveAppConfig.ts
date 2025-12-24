import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type Provider } from '@/bindings'

interface SaveAppConfigParams {
  activeProvider: Provider | null
}

export function useSaveAppConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAppConfigParams): Promise<void> => {
      const result = await commands.saveAppConfig(params.activeProvider)
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
