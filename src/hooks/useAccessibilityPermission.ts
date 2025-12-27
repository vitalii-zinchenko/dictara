import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { commands } from '@/bindings'

/**
 * Hook to check accessibility permission status.
 * Provides caching, loading states, and automatic error handling.
 */
export function useAccessibilityPermission() {
  return useQuery({
    queryKey: ['accessibilityPermission'],
    queryFn: async (): Promise<boolean> => {
      return await commands.checkAccessibilityPermission()
    },
  })
}

/**
 * Hook to request accessibility permission.
 * Opens the macOS System Settings accessibility pane.
 */
export function useRequestAccessibilityPermission() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (): Promise<void> => {
      await commands.requestAccessibilityPermission()
    },
    onSuccess: () => {
      // Invalidate the permission check so it re-fetches
      queryClient.invalidateQueries({ queryKey: ['accessibilityPermission'] })
    },
  })
}
