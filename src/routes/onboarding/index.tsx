import { createFileRoute, Navigate } from '@tanstack/react-router'
import { error as logError } from '@tauri-apps/plugin-log'
import { useOnboardingConfig } from '@/hooks/useOnboardingConfig'
import { stepToRoute } from '@/hooks/useOnboardingNavigation'

export const Route = createFileRoute('/onboarding/')({
  component: OnboardingIndex,
})

function OnboardingIndex() {
  const { data: config, isLoading, error: configError } = useOnboardingConfig()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    )
  }

  if (configError) {
    logError(`Failed to load onboarding config: ${configError}`)
  }

  // Redirect to appropriate step based on saved state
  const targetRoute = stepToRoute(config?.currentStep ?? 'welcome')
  return <Navigate to={targetRoute} />
}
