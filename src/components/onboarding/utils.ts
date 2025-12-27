import type { OnboardingStep } from '@/bindings'

export interface StepDefinition {
  id: OnboardingStep
  label: string
  shortLabel: string
}

export const STEPS: StepDefinition[] = [
  { id: 'welcome', label: 'Welcome', shortLabel: 'Welcome' },
  { id: 'accessibility', label: 'Accessibility', shortLabel: 'Access.' },
  { id: 'api_keys', label: 'API Keys', shortLabel: 'API Keys' },
  { id: 'fn_hold', label: 'FN Hold', shortLabel: 'FN Hold' },
  { id: 'fn_space', label: 'FN + Space', shortLabel: 'FN+Space' },
  { id: 'complete', label: 'Complete', shortLabel: 'Done' },
]

export function getStepDefinition(step: OnboardingStep): StepDefinition | undefined {
  return STEPS.find((s) => s.id === step)
}

export function getStepIndex(step: OnboardingStep): number {
  return STEPS.findIndex((s) => s.id === step)
}
