import { createFileRoute, Outlet } from '@tanstack/react-router'
import { OnboardingLayout } from '@/components/onboarding/OnboardingLayout'

export const Route = createFileRoute('/onboarding')({
  component: OnboardingRoute,
})

function OnboardingRoute() {
  return (
    <OnboardingLayout>
      <Outlet />
    </OnboardingLayout>
  )
}
