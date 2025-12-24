import { Button } from '../ui/button'
import { useCheckForUpdates } from '@/hooks/useCheckForUpdates'

export function Updates() {
  const checkForUpdates = useCheckForUpdates()

  const handleCheckForUpdates = async () => {
    try {
      await checkForUpdates.mutateAsync({ showNoUpdateMessage: true })
    } catch (e) {
      console.error('[Updates] Failed to check for updates:', e)
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
        disabled={checkForUpdates.isPending}
      >
        {checkForUpdates.isPending ? 'Checking...' : 'Check for Updates'}
      </Button>
    </div>
  )
}
