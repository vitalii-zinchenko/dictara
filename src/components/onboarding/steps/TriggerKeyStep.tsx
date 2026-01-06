import { useEffect, useRef, useState } from 'react'
import { error as logError } from '@tauri-apps/plugin-log'
import { relaunch } from '@tauri-apps/plugin-process'
import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { TriggerKeySelector } from '@/components/TriggerKeySelector'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { AlertCircle } from 'lucide-react'
import { commands, type RecordingTrigger } from '@/bindings'

export function TriggerKeyStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()
  const { data: config, isLoading } = useAppConfig()
  const saveConfig = useSaveAppConfig()

  const [selectedTrigger, setSelectedTrigger] = useState<RecordingTrigger>('fn')
  const isInitialized = useRef(false)
  // Store the original trigger from when the app launched (before any changes)
  const originalTrigger = useRef<RecordingTrigger>('fn')

  // Sync state from config on initial load only
  useEffect(() => {
    if (config && !isInitialized.current) {
      const trigger = config.recordingTrigger ?? 'fn'
      setSelectedTrigger(trigger)
      originalTrigger.current = trigger
      isInitialized.current = true
    }
  }, [config])

  // Check if user changed from the original trigger (requires restart to take effect)
  const needsRestart = selectedTrigger !== originalTrigger.current

  const handleTriggerChange = (trigger: RecordingTrigger) => {
    const previousTrigger = selectedTrigger
    setSelectedTrigger(trigger)

    saveConfig.mutate(
      { recordingTrigger: trigger },
      {
        onError: () => {
          setSelectedTrigger(previousTrigger)
        },
      }
    )
  }

  const handleNext = () => {
    goNext('trigger_key')
  }

  const handleRestartAndNext = async () => {
    try {
      // Save the next step so we resume at fn_hold after restart
      // Note: Don't use setPendingRestart - that flag triggers special accessibility logic in setup.rs
      await commands.saveOnboardingStep('fn_hold')
      await relaunch()
    } catch (err) {
      logError(`Failed to restart app: ${err}`)
    }
  }

  if (isLoading) {
    return (
      <StepContainer
        title="Choose Your Trigger Key"
        description="Loading..."
        showBack={true}
        showSkip={true}
        onBack={() => goBack('trigger_key')}
        onSkip={() => skipOnboarding.mutate()}
      >
        <div className="text-muted-foreground">Loading configuration...</div>
      </StepContainer>
    )
  }

  return (
    <StepContainer
      title="Choose Your Trigger Key"
      description="Select which key you'll hold to start recording."
      onNext={needsRestart ? handleRestartAndNext : handleNext}
      nextLabel={needsRestart ? 'Restart & Next' : 'Next'}
      onBack={() => goBack('trigger_key')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending || saveConfig.isPending}
    >
      <div className="space-y-6">
        <p className="text-sm text-muted-foreground">
          This is the key you'll press and hold to record your voice. Release the key to stop
          recording and transcribe.
        </p>

        <TriggerKeySelector
          value={selectedTrigger}
          onChange={handleTriggerChange}
          disabled={saveConfig.isPending}
        />

        {saveConfig.isError && (
          <p className="text-sm text-destructive">Failed to save. Please try again.</p>
        )}

        {needsRestart && (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              The app needs to restart for the new trigger key to take effect.
            </AlertDescription>
          </Alert>
        )}
      </div>
    </StepContainer>
  )
}
