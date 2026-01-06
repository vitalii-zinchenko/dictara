import { useState } from 'react'
import { error as logError } from '@tauri-apps/plugin-log'
import { relaunch } from '@tauri-apps/plugin-process'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { Button } from '@/components/ui/button'
import type { RecordingTrigger } from '@/bindings'

const TRIGGER_OPTIONS: { value: RecordingTrigger; label: string; description: string }[] = [
  { value: 'fn', label: 'Fn (Globe)', description: 'Hold the Fn/Globe key to record' },
  { value: 'control', label: 'Control', description: 'Hold the Control key to record' },
  { value: 'option', label: 'Option', description: 'Hold the Option key to record' },
  { value: 'command', label: 'Command', description: 'Hold the Command key to record' },
]

export function Hotkeys() {
  const { data: appConfig, isLoading } = useAppConfig()
  const saveConfig = useSaveAppConfig()
  const [selectedTrigger, setSelectedTrigger] = useState<RecordingTrigger | null>(null)
  const [isRestarting, setIsRestarting] = useState(false)

  const savedTrigger = appConfig?.recordingTrigger ?? 'fn'
  const currentTrigger = selectedTrigger ?? savedTrigger
  const hasChanges = selectedTrigger !== null && selectedTrigger !== savedTrigger

  const handleTriggerChange = (trigger: RecordingTrigger) => {
    setSelectedTrigger(trigger)
  }

  const handleSaveAndRestart = async () => {
    if (!selectedTrigger) return

    setIsRestarting(true)
    try {
      await saveConfig.mutateAsync({ recordingTrigger: selectedTrigger })
      await relaunch()
    } catch (e) {
      logError(`[Hotkeys] Failed to save and restart: ${e}`)
      setIsRestarting(false)
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">Loading...</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">Recording Trigger</p>
        <p className="text-sm">
          Choose which key to hold for recording. Press and hold to start, release to stop and
          transcribe.
        </p>
      </div>

      <div className="space-y-2">
        {TRIGGER_OPTIONS.map((option) => (
          <label
            key={option.value}
            className={`flex cursor-pointer items-center gap-3 rounded-lg border p-3 transition-colors ${
              currentTrigger === option.value
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-primary/50'
            } ${isRestarting ? 'pointer-events-none opacity-50' : ''}`}
          >
            <input
              type="radio"
              name="recordingTrigger"
              value={option.value}
              checked={currentTrigger === option.value}
              onChange={() => handleTriggerChange(option.value)}
              disabled={isRestarting}
              className="h-4 w-4 accent-primary"
            />
            <div className="flex-1">
              <p className="font-medium">{option.label}</p>
              <p className="text-sm text-muted-foreground">{option.description}</p>
            </div>
          </label>
        ))}
      </div>

      {saveConfig.isError && (
        <p className="text-sm text-destructive">Failed to save. Please try again.</p>
      )}

      {hasChanges && (
        <Button onClick={handleSaveAndRestart} disabled={isRestarting} className="w-full">
          {isRestarting ? 'Restarting...' : 'Save & Restart'}
        </Button>
      )}
    </div>
  )
}
