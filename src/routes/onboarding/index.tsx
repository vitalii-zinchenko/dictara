import { createFileRoute, Navigate } from '@tanstack/react-router'
import { useOnboardingConfig } from '@/hooks/useOnboardingConfig'
import { stepToRoute } from '@/hooks/useOnboardingNavigation'

export const Route = createFileRoute('/onboarding/')({
  component: OnboardingIndex,
})

function OnboardingIndex() {
  const { data: config, isLoading, error } = useOnboardingConfig()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    )
  }

  if (error) {
    console.error('Failed to load onboarding config:', error)
  }

  // Redirect to appropriate step based on saved state
  const targetRoute = stepToRoute(config?.current_step ?? 'welcome')
  return <Navigate to={targetRoute} />
}
