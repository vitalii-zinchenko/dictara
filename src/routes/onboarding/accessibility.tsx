import { createFileRoute } from '@tanstack/react-router'
import { AccessibilityStep } from '@/components/onboarding/steps/AccessibilityStep'

export const Route = createFileRoute('/onboarding/accessibility')({
  component: AccessibilityStep,
})
