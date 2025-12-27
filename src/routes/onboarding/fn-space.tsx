import { createFileRoute } from '@tanstack/react-router'
import { FnSpaceStep } from '@/components/onboarding/steps/FnSpaceStep'

export const Route = createFileRoute('/onboarding/fn-space')({
  component: FnSpaceStep,
})
