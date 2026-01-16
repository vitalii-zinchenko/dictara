import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type ShortcutsConfig } from '@/bindings'

/**
 * Hook to reset the shortcuts configuration to defaults using TanStack Mutation.
 * Automatically invalidates the shortcuts config query on success.
 */
export function useResetShortcutsConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<ShortcutsConfig> => {
      const result = await commands.resetShortcutsConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
    onSuccess: () => {
      // Invalidate the shortcutsConfig query to refetch fresh data
      queryClient.invalidateQueries({ queryKey: ['shortcutsConfig'] })
    },
  })
}
