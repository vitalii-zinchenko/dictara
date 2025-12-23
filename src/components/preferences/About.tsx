import { getVersion } from '@tauri-apps/api/app'
import { useEffect, useState } from 'react'

export function About() {
  const [appVersion, setAppVersion] = useState<string | null>(null)

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

  return (
    <div className="space-y-2">
      <p className="text-sm text-muted-foreground">Version</p>
      <p className="text-2xl font-semibold">{appVersion ? `v${appVersion}` : 'Loading...'}</p>
    </div>
  )
}

