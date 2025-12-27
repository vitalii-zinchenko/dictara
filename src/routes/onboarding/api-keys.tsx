import { createFileRoute } from '@tanstack/react-router'
import { ApiKeysStep } from '@/components/onboarding/steps/ApiKeysStep'

export const Route = createFileRoute('/onboarding/api-keys')({
  component: ApiKeysStep,
})
