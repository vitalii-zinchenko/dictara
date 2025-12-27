import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ExternalLink, RotateCcw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { commands } from '@/bindings'

export function About() {
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const [isRestarting, setIsRestarting] = useState(false)

  useEffect(() => {
    getVersion()
      .then((v) => {
        console.log('[About] App version loaded:', v)
        setAppVersion(v)
      })
      .catch((e: unknown) => {
        console.error('[About] Failed to load app version:', e)
      })
  }, [])

  const handleOpenGitHub = () => {
    openUrl('https://github.com/vitalii-zinchenko/dictara')
  }

  const handleRestartOnboarding = async () => {
    setIsRestarting(true)
    try {
      const result = await commands.restartOnboarding()
      if (result.status === 'error') {
        console.error('[About] Failed to restart onboarding:', result.error)
      }
    } catch (e) {
      console.error('[About] Failed to restart onboarding:', e)
    } finally {
      setIsRestarting(false)
    }
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
          onClick={handleRestartOnboarding}
          disabled={isRestarting}
          className="flex items-center gap-2 text-sm text-primary hover:underline disabled:opacity-50"
        >
          <RotateCcw className="h-4 w-4" />
          {isRestarting ? 'Restarting...' : 'Restart Onboarding'}
        </button>
      </div>
    </div>
  )
}
