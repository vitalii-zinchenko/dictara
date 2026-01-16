import type { OnboardingStep } from '@/bindings'

export interface StepDefinition {
  id: OnboardingStep
  label: string
  shortLabel: string
}

export const STEPS: StepDefinition[] = [
  { id: 'welcome', label: 'Welcome', shortLabel: 'Welcome' },
  { id: 'accessibility', label: 'Accessibility', shortLabel: 'Access.' },
  { id: 'microphone', label: 'Microphone', shortLabel: 'Mic' },
  { id: 'api_keys', label: 'API Keys', shortLabel: 'API Keys' },
  { id: 'shortcuts', label: 'Shortcuts', shortLabel: 'Shortcuts' },
  { id: 'fn_hold', label: 'Push to Talk', shortLabel: 'Push to Talk' },
  { id: 'fn_space', label: 'Hands-Free', shortLabel: 'Hands-Free' },
  { id: 'complete', label: 'Complete', shortLabel: 'Done' },
]

export function getStepDefinition(step: OnboardingStep): StepDefinition | undefined {
  return STEPS.find((s) => s.id === step)
}

export function getStepIndex(step: OnboardingStep): number {
  return STEPS.findIndex((s) => s.id === step)
}
