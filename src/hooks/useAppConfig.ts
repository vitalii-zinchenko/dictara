import { useQuery } from '@tanstack/react-query'
import { commands, type AppConfig } from '@/bindings'

/**
 * Hook to load the app configuration using TanStack Query.
 * Provides caching, loading states, and automatic error handling.
 */
export function useAppConfig() {
  return useQuery({
    queryKey: ['appConfig'],
    queryFn: async (): Promise<AppConfig> => {
      const result = await commands.loadAppConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}
