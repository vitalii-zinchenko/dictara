import { useState } from 'react'
import { error as logError } from '@tauri-apps/plugin-log'
import { relaunch } from '@tauri-apps/plugin-process'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { Button } from '@/components/ui/button'
import { TriggerKeySelector } from '@/components/TriggerKeySelector'
import type { RecordingTrigger } from '@/bindings'

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

      <TriggerKeySelector
        value={currentTrigger}
        onChange={handleTriggerChange}
        disabled={isRestarting}
      />

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
