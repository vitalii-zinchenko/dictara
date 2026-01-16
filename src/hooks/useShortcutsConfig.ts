import { useQuery } from '@tanstack/react-query'
import { commands, type ShortcutsConfig } from '@/bindings'

/**
 * Hook to load the shortcuts configuration using TanStack Query.
 * Provides caching, loading states, and automatic error handling.
 */
export function useShortcutsConfig() {
  return useQuery({
    queryKey: ['shortcutsConfig'],
    queryFn: async (): Promise<ShortcutsConfig> => {
      const result = await commands.loadShortcutsConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}
