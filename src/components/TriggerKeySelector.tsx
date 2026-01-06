import type { RecordingTrigger } from '@/bindings'

const TRIGGER_OPTIONS: { value: RecordingTrigger; label: string; description: string }[] = [
  { value: 'fn', label: 'Fn (Globe)', description: 'Hold the Fn/Globe key to record' },
  { value: 'control', label: 'Control', description: 'Hold the Control key to record' },
  { value: 'option', label: 'Option', description: 'Hold the Option key to record' },
  { value: 'command', label: 'Command', description: 'Hold the Command key to record' },
]

interface TriggerKeySelectorProps {
  value: RecordingTrigger
  onChange: (trigger: RecordingTrigger) => void
  disabled?: boolean
}

export function TriggerKeySelector({ value, onChange, disabled }: TriggerKeySelectorProps) {
  return (
    <div className="space-y-2">
      {TRIGGER_OPTIONS.map((option) => (
        <label
          key={option.value}
          className={`flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors ${
            value === option.value
              ? 'border-primary bg-primary/5'
              : 'border-border hover:border-primary/50'
          } ${disabled ? 'pointer-events-none opacity-50' : ''}`}
        >
          <input
            type="radio"
            name="recordingTrigger"
            value={option.value}
            checked={value === option.value}
            onChange={() => onChange(option.value)}
            disabled={disabled}
            className="h-4 w-4 accent-primary"
          />
          <div className="flex-1">
            <p className="font-medium">{option.label}</p>
            <p className="text-sm text-muted-foreground">{option.description}</p>
          </div>
        </label>
      ))}
    </div>
  )
}

export { TRIGGER_OPTIONS }
