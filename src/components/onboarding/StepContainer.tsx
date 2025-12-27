import { Button } from '@/components/ui/button'
import type { ReactNode } from 'react'

interface StepContainerProps {
  title: string
  description?: string
  children: ReactNode
  onNext?: () => void
  onBack?: () => void
  onSkip?: () => void
  nextLabel?: string
  nextDisabled?: boolean
  isLoading?: boolean
  showBack?: boolean
  showSkip?: boolean
}

export function StepContainer({
  title,
  description,
  children,
  onNext,
  onBack,
  onSkip,
  nextLabel = 'Next',
  nextDisabled = false,
  isLoading = false,
  showBack = true,
  showSkip = true,
}: StepContainerProps) {
  return (
    <div className="flex flex-col h-full min-h-0">
      {/* Header */}
      <div className="shrink-0 mb-6">
        <h2 className="text-xl font-semibold">{title}</h2>
        {description && <p className="text-sm text-muted-foreground mt-1">{description}</p>}
      </div>

      {/* Content - scrollable area */}
      <div className="flex-1 min-h-0 overflow-y-auto p-1">{children}</div>

      {/* Footer with navigation - fixed at bottom */}
      <div className="shrink-0 flex justify-between items-center pt-6 mt-6 border-t">
        <div className="flex gap-2">
          {showBack && onBack && (
            <Button variant="outline" onClick={onBack} disabled={isLoading}>
              Back
            </Button>
          )}
          {showSkip && onSkip && (
            <Button variant="ghost" onClick={onSkip} disabled={isLoading}>
              Skip Setup
            </Button>
          )}
        </div>
        {onNext && (
          <Button onClick={onNext} disabled={nextDisabled || isLoading}>
            {isLoading ? 'Please wait...' : nextLabel}
          </Button>
        )}
      </div>
    </div>
  )
}
