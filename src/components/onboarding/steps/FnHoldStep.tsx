import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { useShortcutsConfig } from '@/hooks/useShortcutsConfig'
import { CheckCircle2 } from 'lucide-react'
import { useRef, useState } from 'react'
import { StepContainer } from '../StepContainer'

function formatShortcutKeys(keys: Array<{ label: string }>): string {
  return keys.map((k) => k.label).join(' + ')
}

export function FnHoldStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()
  const { data: shortcuts } = useShortcutsConfig()
  const [inputValue, setInputValue] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const pushToRecordKeys = shortcuts?.pushToRecord.keys ?? []
  const shortcutLabel = formatShortcutKeys(pushToRecordKeys)

  const handleReset = () => {
    setInputValue('')
    textareaRef.current?.focus()
  }

  const handleNext = () => {
    goNext('fn_hold')
  }

  const hasText = inputValue.trim().length > 0
  const isComplete = hasText

  return (
    <StepContainer
      title="Push to Talk"
      description={`Learn the first way to use Dictara: hold ${shortcutLabel} while speaking.`}
      onNext={handleNext}
      nextDisabled={!isComplete}
      onBack={() => goBack('fn_hold')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <div className="space-y-6">
        <div className="space-y-2">
          <p className="text-sm font-medium">How it works:</p>
          <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
            <li>Click in the text field below</li>
            <li>
              Press and hold <strong>{shortcutLabel}</strong>
            </li>
            <li>Speak</li>
            <li>
              Release <strong>{shortcutLabel}</strong> when done
            </li>
          </ol>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <p className="text-sm font-medium">
              {!hasText && (
                <>
                  Hold <strong>{shortcutLabel}</strong> and speak, then release when done
                </>
              )}
              {hasText && 'Great job!'}
            </p>
            {hasText && <CheckCircle2 className="h-5 w-5 text-green-500" />}
          </div>

          <Textarea
            ref={textareaRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            placeholder="Your dictated text will appear here..."
            className={`min-h-[100px] resize-none transition-colors ${
              hasText ? 'border-green-500 bg-green-50 dark:bg-green-950/20' : ''
            }`}
          />

          {hasText && (
            <div className="flex justify-end">
              <Button variant="outline" size="sm" onClick={handleReset}>
                Try Again
              </Button>
            </div>
          )}
        </div>

        {isComplete && (
          <p className="text-sm text-green-600 dark:text-green-400 text-center">
            You've got it! Click Next to learn the hands-free mode.
          </p>
        )}
      </div>
    </StepContainer>
  )
}
