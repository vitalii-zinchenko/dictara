import { useState, useEffect, useRef } from 'react'
import { StepContainer } from '../StepContainer'
import { KeyboardVisual } from '../KeyboardVisual'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { events } from '@/bindings'
import { CheckCircle2 } from 'lucide-react'

type TrainingPhase = 'waiting' | 'recording' | 'success'

export function FnSpaceStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()
  const [phase, setPhase] = useState<TrainingPhase>('waiting')
  const [inputValue, setInputValue] = useState('')
  const [isFocused, setIsFocused] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const unlistenRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await events.recordingStateChanged.listen((event) => {
        const payload = event.payload
        switch (payload.state) {
          case 'started':
            setPhase('recording')
            break
          case 'stopped':
            setPhase('success')
            break
          case 'cancelled':
            setPhase('waiting')
            break
          case 'error':
            setPhase('waiting')
            break
        }
      })
      unlistenRef.current = unlisten
    }

    setupListener()

    return () => {
      if (unlistenRef.current) {
        unlistenRef.current()
      }
    }
  }, [])

  const handleReset = () => {
    setPhase('waiting')
    setInputValue('')
    textareaRef.current?.focus()
  }

  const handleNext = () => {
    goNext('fn_space')
  }

  const hasText = inputValue.trim().length > 0
  const isComplete = hasText

  return (
    <StepContainer
      title="FN + Space Toggle Mode"
      description="Learn the second way to use Dictara: toggle recording on and off."
      onNext={handleNext}
      nextDisabled={!isComplete}
      onBack={() => goBack('fn_space')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <div className="space-y-6">
        <div className="space-y-2">
          <p className="text-sm font-medium">How it works:</p>
          <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
            <li>Click in the text field below</li>
            <li>Press FN + Space to start recording</li>
            <li>Speak your text (hands-free!)</li>
            <li>Press FN again to stop</li>
          </ol>
        </div>

        <div className="flex justify-center py-4">
          <KeyboardVisual
            highlightedKeys={['fn', 'space']}
            pressedKeys={phase === 'recording' ? ['fn'] : []}
          />
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <p className="text-sm font-medium">
              {!isFocused && !hasText && 'Click in the field below, then press FN + Space'}
              {isFocused && phase === 'waiting' && !hasText && 'Now press FN + Space to start...'}
              {phase === 'recording' && 'Recording... Press FN to stop'}
              {hasText && phase !== 'recording' && 'Perfect!'}
            </p>
            {hasText && <CheckCircle2 className="h-5 w-5 text-green-500" />}
          </div>

          <Textarea
            ref={textareaRef}
            value={inputValue}
            onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setInputValue(e.target.value)}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            placeholder="Your dictated text will appear here..."
            className={`min-h-[100px] resize-none transition-colors ${
              isFocused && !hasText
                ? 'ring-2 ring-primary ring-offset-2'
                : hasText
                  ? 'border-green-500 bg-green-50 dark:bg-green-950/20'
                  : ''
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
            You're all set! Click Next to complete setup.
          </p>
        )}
      </div>
    </StepContainer>
  )
}
