import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { useShortcutsConfig } from '@/hooks/useShortcutsConfig'
import { CheckCircle2, Keyboard, Mic } from 'lucide-react'

function formatShortcutKeys(keys: Array<{ label: string }>): string {
  return keys.map((k) => k.label).join(' + ')
}

export function CompleteStep() {
  const { finishOnboarding } = useOnboardingNavigation()
  const { data: shortcuts } = useShortcutsConfig()

  const pushToRecordKeys = shortcuts?.pushToRecord.keys ?? []
  const handsFreeKeys = shortcuts?.handsFree.keys ?? []
  const pushToRecordLabel = formatShortcutKeys(pushToRecordKeys)
  const handsFreeLabel = formatShortcutKeys(handsFreeKeys)

  const handleFinish = () => {
    finishOnboarding.mutate()
  }

  return (
    <StepContainer
      title="You're All Set!"
      description="Dictara is ready to use."
      onNext={handleFinish}
      nextLabel="Start Using Dictara"
      showBack={false}
      showSkip={false}
      isLoading={finishOnboarding.isPending}
    >
      <div className="space-y-6">
        <div className="flex justify-center py-4">
          <div className="w-20 h-20 rounded-full bg-green-500/20 flex items-center justify-center">
            <CheckCircle2 className="w-10 h-10 text-green-500" />
          </div>
        </div>

        <div className="text-center space-y-4">
          <p className="text-muted-foreground">
            You've completed the setup! Here's a quick reminder of how to use Dictara:
          </p>

          <div className="grid gap-4 max-w-sm mx-auto">
            <div className="flex items-start gap-3 p-3 bg-muted/50 rounded-lg text-left">
              <Keyboard className="w-5 h-5 text-muted-foreground mt-0.5 shrink-0" />
              <div>
                <p className="text-sm font-medium">Hold {pushToRecordLabel}</p>
                <p className="text-sm text-muted-foreground">
                  Hold to record, release to transcribe
                </p>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 bg-muted/50 rounded-lg text-left">
              <Mic className="w-5 h-5 text-muted-foreground mt-0.5 shrink-0" />
              <div>
                <p className="text-sm font-medium">{handsFreeLabel}</p>
                <p className="text-sm text-muted-foreground">Toggle mode: tap to start/stop</p>
              </div>
            </div>
          </div>

          <p className="text-sm text-muted-foreground pt-4">
            Look for the Dictara icon in your menu bar to access preferences.
          </p>
        </div>
      </div>
    </StepContainer>
  )
}
