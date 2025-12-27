import { useQuery } from '@tanstack/react-query'
import { commands, type OnboardingConfig } from '@/bindings'

/**
 * Hook to load the onboarding configuration using TanStack Query.
 * Provides caching, loading states, and automatic error handling.
 */
export function useOnboardingConfig() {
  return useQuery({
    queryKey: ['onboardingConfig'],
    queryFn: async (): Promise<OnboardingConfig> => {
      const result = await commands.loadOnboardingConfig()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      return result.data
    },
  })
}
