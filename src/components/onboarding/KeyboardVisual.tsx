import { cn } from '@/lib/utils'

export type KeyboardKey = 'fn' | 'control' | 'option' | 'command' | 'space'

interface KeyboardVisualProps {
  highlightedKeys: KeyboardKey[]
  pressedKeys?: KeyboardKey[]
}

export function KeyboardVisual({ highlightedKeys, pressedKeys = [] }: KeyboardVisualProps) {
  const isHighlighted = (key: KeyboardKey) => highlightedKeys.includes(key)
  const isPressed = (key: KeyboardKey) => pressedKeys.includes(key)

  return (
    <div className="bg-gray-100 p-4 rounded-sm inline-block max-w-full">
      {/* Simplified keyboard layout - bottom row with modifier keys and Space */}
      <div className="flex gap-1 items-center">
        <Key
          label="fn"
          isHighlighted={isHighlighted('fn')}
          isPressed={isPressed('fn')}
          className="w-10"
        />
        <Key
          label="^"
          isHighlighted={isHighlighted('control')}
          isPressed={isPressed('control')}
          className="w-10"
        />
        <Key
          label="opt"
          isHighlighted={isHighlighted('option')}
          isPressed={isPressed('option')}
          className="w-10"
        />
        <Key
          label="cmd"
          isHighlighted={isHighlighted('command')}
          isPressed={isPressed('command')}
          className="w-12"
        />
        <Key
          label=""
          isHighlighted={isHighlighted('space')}
          isPressed={isPressed('space')}
          className="w-40"
        />
        <Key
          label="cmd"
          isHighlighted={isHighlighted('command')}
          isPressed={isPressed('command')}
          className="w-12"
        />
        <Key
          label="opt"
          isHighlighted={isHighlighted('option')}
          isPressed={isPressed('option')}
          className="w-10"
        />
      </div>
    </div>
  )
}

interface KeyProps {
  label: string
  className?: string
  isHighlighted?: boolean
  isPressed?: boolean
  variant?: 'light' | 'dark'
}

function Key({ label, className, isHighlighted, isPressed, variant = 'light' }: KeyProps) {
  return (
    <div
      className={cn(
        'px-2 py-2 rounded text-xs font-mono text-center h-8 flex items-center justify-center transition-all border shadow-sm',
        variant === 'light' && 'bg-white text-gray-700 border-gray-300',
        variant === 'dark' && 'bg-gray-500 text-white border-gray-600',
        isHighlighted && 'ring-2 ring-primary',
        isPressed && 'bg-primary text-primary-foreground scale-95',
        className
      )}
    >
      {label}
    </div>
  )
}
