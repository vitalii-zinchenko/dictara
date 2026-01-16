import { KeyCaptureInput } from '@/components/shortcuts/KeyCaptureInput'
import { Button } from '@/components/ui/button'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { useShortcutsConfig } from '@/hooks/useShortcutsConfig'
import { useSaveShortcutsConfig } from '@/hooks/useSaveShortcutsConfig'
import { useResetShortcutsConfig } from '@/hooks/useResetShortcutsConfig'

export function ShortcutsConfiguration() {
  const { data: config } = useShortcutsConfig()
  const saveMutation = useSaveShortcutsConfig()
  const resetMutation = useResetShortcutsConfig()

  if (!config) {
    return <div>Loading...</div>
  }

  const handlePushToRecordChange = (keys: Array<{ keycode: number; label: string }>) => {
    saveMutation.mutate({
      ...config,
      pushToRecord: { keys },
    })
  }

  const handleHandsFreeChange = (keys: Array<{ keycode: number; label: string }>) => {
    saveMutation.mutate({
      ...config,
      handsFree: { keys },
    })
  }

  const handleReset = () => {
    resetMutation.mutate()
  }

  return (
    <div className="space-y-6">
      <KeyCaptureInput
        label="Push to Record"
        description="Hold to record, release to stop"
        value={config.pushToRecord.keys}
        onChange={handlePushToRecordChange}
      />

      <KeyCaptureInput
        label="Hands-free"
        description="Press to toggle (start/stop)"
        value={config.handsFree.keys}
        onChange={handleHandsFreeChange}
      />

      {saveMutation.isError && (
        <Alert variant="destructive">
          <AlertDescription>
            {saveMutation.error?.message || 'Failed to save shortcuts'}
          </AlertDescription>
        </Alert>
      )}

      <div className="flex justify-end pt-4">
        <Button onClick={handleReset} variant="outline" disabled={resetMutation.isPending}>
          Reset to Defaults
        </Button>
      </div>
    </div>
  )
}
