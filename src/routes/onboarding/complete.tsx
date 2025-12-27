import { createFileRoute } from '@tanstack/react-router'
import { CompleteStep } from '@/components/onboarding/steps/CompleteStep'

export const Route = createFileRoute('/onboarding/complete')({
  component: CompleteStep,
})
