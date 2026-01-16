import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type ShortcutsConfig } from '@/bindings'

/**
 * Hook to save the shortcuts configuration using TanStack Mutation.
 * Uses optimistic updates to prevent UI flickering.
 */
export function useSaveShortcutsConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (config: ShortcutsConfig): Promise<void> => {
      const result = await commands.saveShortcutsConfig(config)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onMutate: async (newConfig) => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['shortcutsConfig'] })

      // Snapshot the previous value
      const previousConfig = queryClient.getQueryData<ShortcutsConfig>(['shortcutsConfig'])

      // Optimistically update to the new value
      queryClient.setQueryData(['shortcutsConfig'], newConfig)

      // Return context with the previous value
      return { previousConfig }
    },
    onError: (_err, _newConfig, context) => {
      // If mutation fails, rollback to the previous value
      if (context?.previousConfig) {
        queryClient.setQueryData(['shortcutsConfig'], context.previousConfig)
      }
    },
    onSettled: () => {
      // Always refetch after error or success to ensure we're in sync with server
      queryClient.invalidateQueries({ queryKey: ['shortcutsConfig'] })
    },
  })
}
