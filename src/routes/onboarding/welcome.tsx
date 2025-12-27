import { createFileRoute } from '@tanstack/react-router'
import { WelcomeStep } from '@/components/onboarding/steps/WelcomeStep'

export const Route = createFileRoute('/onboarding/welcome')({
  component: WelcomeStep,
})
