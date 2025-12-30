import { getVersion } from '@tauri-apps/api/app'
import { error as logError } from '@tauri-apps/plugin-log'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ExternalLink, RotateCcw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useRestartOnboarding } from '@/hooks/useOnboardingNavigation'

export function About() {
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const restartOnboarding = useRestartOnboarding()

  useEffect(() => {
    getVersion()
      .then((v) => setAppVersion(v))
      .catch((e: unknown) => {
        logError(`[About] Failed to load app version: ${e}`)
      })
  }, [])

  const handleOpenGitHub = () => {
    openUrl('https://github.com/vitalii-zinchenko/dictara')
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">Version</p>
        <p className="text-2xl font-semibold">{appVersion ? `v${appVersion}` : 'Loading...'}</p>
      </div>

      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">Source Code</p>
        <button
          type="button"
          onClick={handleOpenGitHub}
          className="flex items-center gap-2 text-sm text-primary hover:underline"
        >
          <ExternalLink className="h-4 w-4" />
          github.com/vitalii-zinchenko/dictara
        </button>
      </div>

      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">Setup</p>
        <button
          type="button"
          onClick={() => restartOnboarding.mutate()}
          disabled={restartOnboarding.isPending}
          className="flex items-center gap-2 text-sm text-primary hover:underline disabled:opacity-50"
        >
          <RotateCcw className="h-4 w-4" />
          {restartOnboarding.isPending ? 'Restarting...' : 'Restart Onboarding'}
        </button>
      </div>
    </div>
  )
}
