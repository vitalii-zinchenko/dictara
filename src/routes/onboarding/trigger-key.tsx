import { createFileRoute } from '@tanstack/react-router'
import { TriggerKeyStep } from '@/components/onboarding/steps/TriggerKeyStep'

export const Route = createFileRoute('/onboarding/trigger-key')({
  component: TriggerKeyStep,
})
