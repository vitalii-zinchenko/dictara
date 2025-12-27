import { relaunch } from '@tauri-apps/plugin-process'
import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import {
  useAccessibilityPermission,
  useRequestAccessibilityPermission,
} from '@/hooks/useAccessibilityPermission'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { CheckCircle2, AlertCircle, Settings } from 'lucide-react'

export function AccessibilityStep() {
  const {
    data: hasPermission,
    isLoading: isChecking,
    refetch: checkPermission,
  } = useAccessibilityPermission()
  const requestPermission = useRequestAccessibilityPermission()
  const { goNext, goBack, skipOnboarding, setPendingRestart, isNavigating } =
    useOnboardingNavigation()

  const handleRestart = async () => {
    try {
      // Mark pending restart so we resume correctly after restart
      await setPendingRestart.mutateAsync(true)
      await relaunch()
    } catch (error) {
      console.error('Failed to restart app:', error)
    }
  }

  const handleNext = () => {
    if (hasPermission) {
      goNext('accessibility')
    }
  }

  if (hasPermission === undefined || isChecking) {
    return (
      <StepContainer
        title="Accessibility Permission"
        description="Checking permission status..."
        showBack={true}
        showSkip={true}
        onBack={() => goBack('accessibility')}
        onSkip={() => skipOnboarding.mutate()}
      >
        <div className="flex items-center justify-center py-12">
          <p className="text-muted-foreground">Checking...</p>
        </div>
      </StepContainer>
    )
  }

  if (hasPermission) {
    return (
      <StepContainer
        title="Accessibility Permission"
        description="Permission granted!"
        onNext={handleNext}
        onBack={() => goBack('accessibility')}
        onSkip={() => skipOnboarding.mutate()}
        isLoading={isNavigating || skipOnboarding.isPending}
      >
        <div className="space-y-6">
          <Alert className="border-green-500/50 bg-green-500/10">
            <CheckCircle2 className="h-4 w-4 text-green-500" />
            <AlertDescription className="text-green-700 dark:text-green-400">
              Dictara has accessibility access and can detect keyboard shortcuts.
            </AlertDescription>
          </Alert>

          <p className="text-sm text-muted-foreground">
            You're all set! Click Next to continue.
          </p>
        </div>
      </StepContainer>
    )
  }

  return (
    <StepContainer
      title="Accessibility Permission"
      description="Dictara needs accessibility access to detect keyboard shortcuts."
      onNext={handleRestart}
      nextLabel="Restart App"
      onBack={() => goBack('accessibility')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending || setPendingRestart.isPending}
    >
      <div className="space-y-6">
        <p className="text-sm text-muted-foreground">
          This permission allows Dictara to listen for the FN key even when other apps are focused.
          Without it, keyboard shortcuts won't work.
        </p>

        <div className="space-y-4">
          <div className="flex items-start gap-3 p-4 bg-muted/50 rounded-lg">
            <Settings className="w-5 h-5 text-muted-foreground mt-0.5 shrink-0" />
            <div className="space-y-2">
              <p className="text-sm font-medium">Step 1: Open System Settings</p>
              <p className="text-sm text-muted-foreground">
                Click the button below to open the Accessibility settings. Then find "Dictara" in
                the list and toggle it ON.
              </p>
              <Button variant="secondary" onClick={() => requestPermission.mutate()}>
                Open Accessibility Settings
              </Button>
            </div>
          </div>

          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              After enabling the permission in System Settings, click "Restart App" to continue
              setup. The app needs to restart for the permission to take effect.
            </AlertDescription>
          </Alert>
        </div>

        <Button variant="outline" size="sm" onClick={() => checkPermission()}>
          Check Again
        </Button>
      </div>
    </StepContainer>
  )
}
