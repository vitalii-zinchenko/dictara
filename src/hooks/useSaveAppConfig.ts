import { useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import type { Provider } from '@/bindings'

interface SaveAppConfigParams {
  activeProvider: Provider | null
}

export function useSaveAppConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAppConfigParams): Promise<void> => {
      await invoke('save_app_config', { activeProvider: params.activeProvider })
    },
    onSuccess: () => {
      // Invalidate the appConfig query to refetch fresh data
      queryClient.invalidateQueries({ queryKey: ['appConfig'] })
    },
  })
}
