import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { commands, type OnboardingStep } from '@/bindings'

const STEP_ORDER: OnboardingStep[] = [
  'welcome',
  'accessibility',
  'api_keys',
  'fn_hold',
  'fn_space',
  'complete',
]

const STEP_ROUTES: Record<OnboardingStep, string> = {
  welcome: '/onboarding/welcome',
  accessibility: '/onboarding/accessibility',
  api_keys: '/onboarding/api-keys',
  fn_hold: '/onboarding/fn-hold',
  fn_space: '/onboarding/fn-space',
  complete: '/onboarding/complete',
}

export function stepToRoute(step: OnboardingStep): string {
  return STEP_ROUTES[step]
}

export function getNextStep(currentStep: OnboardingStep): OnboardingStep | null {
  const currentIndex = STEP_ORDER.indexOf(currentStep)
  if (currentIndex === -1 || currentIndex >= STEP_ORDER.length - 1) {
    return null
  }
  return STEP_ORDER[currentIndex + 1]
}

export function getPreviousStep(currentStep: OnboardingStep): OnboardingStep | null {
  const currentIndex = STEP_ORDER.indexOf(currentStep)
  if (currentIndex <= 0) {
    return null
  }
  return STEP_ORDER[currentIndex - 1]
}

export function getStepIndex(step: OnboardingStep): number {
  return STEP_ORDER.indexOf(step)
}

export function useOnboardingNavigation() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const saveStep = useMutation({
    mutationFn: async (step: OnboardingStep): Promise<void> => {
      const result = await commands.saveOnboardingStep(step)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['onboardingConfig'] })
    },
  })

  const goToStep = async (step: OnboardingStep) => {
    await saveStep.mutateAsync(step)
    navigate({ to: stepToRoute(step) })
  }

  const goNext = async (currentStep: OnboardingStep) => {
    const nextStep = getNextStep(currentStep)
    if (nextStep) {
      await goToStep(nextStep)
    }
  }

  const goBack = async (currentStep: OnboardingStep) => {
    const prevStep = getPreviousStep(currentStep)
    if (prevStep) {
      await goToStep(prevStep)
    }
  }

  const finishOnboarding = useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.finishOnboarding()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      // Window is closed by the backend
    },
  })

  const skipOnboarding = useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.skipOnboarding()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
      // Window is closed by the backend
    },
  })

  const setPendingRestart = useMutation({
    mutationFn: async (pending: boolean): Promise<void> => {
      const result = await commands.setPendingRestart(pending)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })

  return {
    goToStep,
    goNext,
    goBack,
    finishOnboarding,
    skipOnboarding,
    setPendingRestart,
    isNavigating: saveStep.isPending,
  }
}

/**
 * Hook to restart the onboarding flow.
 * Can be used outside of the onboarding context (e.g., from preferences).
 */
export function useRestartOnboarding() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.restartOnboarding()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}
