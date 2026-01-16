import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { ShortcutsConfiguration } from '@/components/preferences/ShortcutsConfig'

export function ShortcutsStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()

  const handleNext = () => {
    goNext('shortcuts')
  }

  return (
    <StepContainer
      title="Configure Your Shortcuts"
      description="Set up keyboard shortcuts for recording."
      onNext={handleNext}
      onBack={() => goBack('shortcuts')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <div className="space-y-6">
        <p className="text-sm text-muted-foreground">
          Configure shortcuts for push-to-talk and hands-free recording. Changes take effect
          immediately - no restart required.
        </p>

        <ShortcutsConfiguration />
      </div>
    </StepContainer>
  )
}
