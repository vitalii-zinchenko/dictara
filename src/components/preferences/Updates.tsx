import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import { Button } from '../ui/button'

export function Updates() {
  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false)

  const handleCheckForUpdates = async () => {
    setIsCheckingUpdates(true)
    try {
      await invoke('check_for_updates', { showNoUpdateMessage: true })
    } catch (e) {
      console.error('[Updates] Failed to check for updates:', e)
    } finally {
      setIsCheckingUpdates(false)
    }
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">Software Updates</p>
        <p className="text-sm">Check for new versions of Dictara.</p>
      </div>

      <Button
        variant="outline"
        onClick={handleCheckForUpdates}
        disabled={isCheckingUpdates}
      >
        {isCheckingUpdates ? 'Checking...' : 'Check for Updates'}
      </Button>
    </div>
  )
}
