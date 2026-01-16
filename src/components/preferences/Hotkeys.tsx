import { ShortcutsConfiguration } from '@/components/preferences/ShortcutsConfig'

export function Hotkeys() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold">Keyboard Shortcuts</h2>
        <p className="text-sm text-muted-foreground mt-1">Configure your recording shortcuts</p>
      </div>

      <ShortcutsConfiguration />
    </div>
  )
}
