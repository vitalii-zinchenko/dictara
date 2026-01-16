import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import { STEPS, type StepDefinition } from './utils'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'

interface StepperProps {
  currentStepIndex: number
}

export function Stepper({ currentStepIndex }: StepperProps) {
  const { goToStep } = useOnboardingNavigation()

  return (
    <nav className="space-y-1">
      {STEPS.map((step, index) => (
        <StepItem
          key={step.id}
          step={step}
          index={index}
          isCompleted={index < currentStepIndex}
          isCurrent={index === currentStepIndex}
          onClick={() => goToStep(step.id)}
        />
      ))}
    </nav>
  )
}

interface StepItemProps {
  step: StepDefinition
  index: number
  isCompleted: boolean
  isCurrent: boolean
  onClick: () => void
}

function StepItem({ step, index, isCompleted, isCurrent, onClick }: StepItemProps) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex items-center gap-3 py-2 px-3 rounded-md text-sm transition-colors w-full',
        'hover:bg-accent/50 cursor-pointer',
        isCurrent && 'bg-accent font-medium',
        !isCurrent && !isCompleted && 'text-muted-foreground'
      )}
    >
      <div
        className={cn(
          'w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium shrink-0',
          isCompleted && 'bg-primary text-primary-foreground',
          isCurrent && 'bg-primary text-primary-foreground',
          !isCurrent && !isCompleted && 'border border-muted-foreground text-muted-foreground'
        )}
      >
        {isCompleted ? <Check className="w-3.5 h-3.5" /> : index + 1}
      </div>
      <span className="truncate">{step.label}</span>
    </button>
  )
}
