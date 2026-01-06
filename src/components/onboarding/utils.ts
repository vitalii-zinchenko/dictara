import type { OnboardingStep, RecordingTrigger } from '@/bindings'
import type { KeyboardKey } from './KeyboardVisual'

export interface StepDefinition {
  id: OnboardingStep
  label: string
  shortLabel: string
}

/** Get display name for a trigger key (e.g., "Fn", "Control", "Option", "Command") */
export function getTriggerDisplayName(trigger: RecordingTrigger): string {
  switch (trigger) {
    case 'fn':
      return 'Fn'
    case 'control':
      return 'Control'
    case 'option':
      return 'Option'
    case 'command':
      return 'Command'
  }
}

/** Get keyboard key identifier for KeyboardVisual component */
export function getTriggerKeyboardKey(trigger: RecordingTrigger): KeyboardKey {
  return trigger // They happen to match in our case
}

export const STEPS: StepDefinition[] = [
  { id: 'welcome', label: 'Welcome', shortLabel: 'Welcome' },
  { id: 'accessibility', label: 'Accessibility', shortLabel: 'Access.' },
  { id: 'api_keys', label: 'API Keys', shortLabel: 'API Keys' },
  { id: 'trigger_key', label: 'Trigger Key', shortLabel: 'Trigger' },
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
