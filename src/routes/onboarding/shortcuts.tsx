import { createFileRoute } from '@tanstack/react-router'
import { ShortcutsStep } from '@/components/onboarding/steps/ShortcutsStep'

export const Route = createFileRoute('/onboarding/shortcuts')({
  component: ShortcutsStep,
})
