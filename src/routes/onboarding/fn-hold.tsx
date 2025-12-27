import { createFileRoute } from '@tanstack/react-router'
import { FnHoldStep } from '@/components/onboarding/steps/FnHoldStep'

export const Route = createFileRoute('/onboarding/fn-hold')({
  component: FnHoldStep,
})
