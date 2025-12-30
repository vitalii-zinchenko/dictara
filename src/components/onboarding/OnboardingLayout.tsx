import type { ReactNode } from 'react'
import { Stepper } from './Stepper'
import { useOnboardingConfig } from '@/hooks/useOnboardingConfig'
import { getStepIndex } from './utils'

interface OnboardingLayoutProps {
  children: ReactNode
}

export function OnboardingLayout({ children }: OnboardingLayoutProps) {
  const { data: config, isLoading } = useOnboardingConfig()

  const currentStepIndex = config ? getStepIndex(config.currentStep) : 0

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <p className="text-muted-foreground">Loading...</p>
      </div>
    )
  }

  return (
    <div className="h-screen bg-background flex flex-col overflow-hidden">
      {/* Header */}
      <header className="shrink-0 border-b px-6 py-4">
        <h1 className="text-lg font-semibold">Welcome to Dictara</h1>
      </header>

      <div className="flex flex-1 min-h-0">
        {/* Stepper sidebar */}
        <aside className="w-48 border-r p-4 shrink-0 overflow-y-auto">
          <Stepper currentStepIndex={currentStepIndex} />
        </aside>

        {/* Step content - StepContainer handles its own scrolling */}
        <main className="flex-1 min-h-0 overflow-hidden p-6">{children}</main>
      </div>
    </div>
  )
}
