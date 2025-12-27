import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { Mic } from 'lucide-react'

export function WelcomeStep() {
  const { goNext, skipOnboarding, isNavigating } = useOnboardingNavigation()

  return (
    <StepContainer
      title="Welcome to Dictara"
      description="Your voice-to-text companion for macOS"
      onNext={() => goNext('welcome')}
      onSkip={() => skipOnboarding.mutate()}
      nextLabel="Get Started"
      showBack={false}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <div className="space-y-6">
        <div className="flex justify-center py-6">
          <div className="w-20 h-20 rounded-full bg-primary/10 flex items-center justify-center">
            <Mic className="w-10 h-10 text-primary" />
          </div>
        </div>

        <div className="space-y-4 text-center max-w-md mx-auto">
          <p className="text-muted-foreground">
            Dictara lets you dictate text anywhere on your Mac using simple keyboard shortcuts.
          </p>

          <div className="bg-muted/50 rounded-lg p-4 text-left space-y-2">
            <p className="text-sm font-medium">What you'll set up:</p>
            <ul className="text-sm text-muted-foreground space-y-1">
              <li>1. Accessibility permission (for keyboard shortcuts)</li>
              <li>2. API keys (for speech recognition)</li>
              <li>3. Microphone access (for recording)</li>
              <li>4. Learn how to use the app</li>
            </ul>
          </div>

          <p className="text-sm text-muted-foreground">This only takes a few minutes.</p>
        </div>
      </div>
    </StepContainer>
  )
}
