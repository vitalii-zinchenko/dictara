import { useMutation } from '@tanstack/react-query'
import { commands } from '@/bindings'

interface CheckForUpdatesParams {
  showNoUpdateMessage?: boolean
}

/**
 * Hook to check for application updates.
 */
export function useCheckForUpdates() {
  return useMutation({
    mutationFn: async (params: CheckForUpdatesParams = {}): Promise<boolean> => {
      const result = await commands.checkForUpdates(params.showNoUpdateMessage ?? true)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}
