import { cn } from '@/lib/utils'

interface KeyboardVisualProps {
  highlightedKeys: ('fn' | 'space')[]
  pressedKeys?: ('fn' | 'space')[]
}

export function KeyboardVisual({ highlightedKeys, pressedKeys = [] }: KeyboardVisualProps) {
  const isHighlighted = (key: 'fn' | 'space') => highlightedKeys.includes(key)
  const isPressed = (key: 'fn' | 'space') => pressedKeys.includes(key)

  return (
    <div className="bg-gray-800 p-4 rounded-lg inline-block">
      {/* Simplified keyboard layout - bottom row with FN and Space */}
      <div className="flex gap-1 items-center">
        <Key
          label="fn"
          isHighlighted={isHighlighted('fn')}
          isPressed={isPressed('fn')}
          className="w-10"
        />
        <Key label="^" className="w-10" />
        <Key label="opt" className="w-10" />
        <Key label="cmd" className="w-12" />
        <Key
          label=""
          isHighlighted={isHighlighted('space')}
          isPressed={isPressed('space')}
          className="w-40"
        />
        <Key label="cmd" className="w-12" />
        <Key label="opt" className="w-10" />
      </div>
    </div>
  )
}

interface KeyProps {
  label: string
  className?: string
  isHighlighted?: boolean
  isPressed?: boolean
}

function Key({ label, className, isHighlighted, isPressed }: KeyProps) {
  return (
    <div
      className={cn(
        'bg-gray-700 text-gray-300 px-2 py-2 rounded text-xs font-mono text-center h-8 flex items-center justify-center transition-all',
        isHighlighted && 'ring-2 ring-primary bg-gray-600',
        isPressed && 'bg-primary text-primary-foreground scale-95',
        className
      )}
    >
      {label}
    </div>
  )
}
