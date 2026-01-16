import { useKeyCapture, CapturedKey } from '@/hooks/useKeyCapture'
import { PencilIcon, CommandIcon, OptionIcon, XIcon, Globe } from 'lucide-react'
import { cn } from '@/lib/utils'

interface KeyCaptureInputProps {
  value: CapturedKey[]
  onChange: (keys: CapturedKey[]) => void
  label: string
  description?: string
}

// Control key icon component (caret symbol)
const ControlIcon = ({ className }: { className?: string }) => (
  <span className={cn('font-semibold', className)}>^</span>
)

// Consistent size for all key icons
const ICON_SIZE = 'h-3.5 w-3.5'

// Map key labels to icons
const KEY_ICON_MAP: Record<string, React.ReactNode> = {
  command: <CommandIcon className={ICON_SIZE} />,
  cmd: <CommandIcon className={ICON_SIZE} />,
  option: <OptionIcon className={ICON_SIZE} />,
  opt: <OptionIcon className={ICON_SIZE} />,
  alt: <OptionIcon className={ICON_SIZE} />,
  control: <ControlIcon className={ICON_SIZE} />,
  ctrl: <ControlIcon className={ICON_SIZE} />,
  fn: <Globe className={ICON_SIZE} />,
}

const getKeyIcon = (label: string): React.ReactNode => {
  return KEY_ICON_MAP[label.toLowerCase()] ?? null
}

export function KeyCaptureInput({ value, onChange, label, description }: KeyCaptureInputProps) {
  const { isCapturing, pressedKeys, startCapture, stopCapture } = useKeyCapture()

  const handleBoxClick = async () => {
    if (!isCapturing) {
      // Pass auto-finish callback that will be called on key release with captured keys
      await startCapture(async (capturedKeys) => {
        // IMPORTANT: Call onChange FIRST to trigger optimistic update
        // before stopCapture sets isCapturing=false, preventing flicker
        onChange(capturedKeys)
        await stopCapture()
      })
    }
  }

  const handleCancel = async (e: React.MouseEvent) => {
    e.stopPropagation() // Prevent box click from triggering
    await stopCapture()
  }

  const displayKeys = isCapturing ? pressedKeys : value

  return (
    <div className="space-y-3">
      <div>
        <h3 className="text-base font-medium">{label}</h3>
        {description && <p className="text-sm text-muted-foreground mt-0.5">{description}</p>}
      </div>

      <div
        onClick={handleBoxClick}
        className={cn(
          'relative flex items-center content-center gap-2 flex-wrap h-[58px] p-3',
          'border-2 rounded-lg transition-all',
          !isCapturing && 'cursor-pointer hover:border-primary/50 hover:bg-accent/50',
          isCapturing && 'border-primary bg-accent'
        )}
      >
        {displayKeys.map((key) => {
          const icon = getKeyIcon(key.label)
          return (
            <div
              key={key.keycode}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-background border rounded-md text-sm font-medium"
            >
              {icon}
              <span>{key.label}</span>
            </div>
          )
        })}

        {!isCapturing && displayKeys.length === 0 && (
          <span className="text-sm text-muted-foreground">Click to set shortcut</span>
        )}

        {isCapturing && (
          <span className="text-sm text-muted-foreground">
            Press keys... ({pressedKeys.length}/3)
          </span>
        )}

        {/* Edit/Cancel icon on the right */}
        {!isCapturing && displayKeys.length > 0 && (
          <PencilIcon className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        )}
        {isCapturing && (
          <button
            onClick={handleCancel}
            className="absolute right-3 top-1/2 -translate-y-1/2 h-6 w-6 flex items-center justify-center rounded hover:bg-background/80 transition-colors"
          >
            <XIcon className="h-4 w-4 text-muted-foreground" />
          </button>
        )}
      </div>
    </div>
  )
}
