import { createFileRoute } from '@tanstack/react-router'
import { Hotkeys } from '@/components/preferences/Hotkeys'

export const Route = createFileRoute('/preferences/hotkeys')({
  component: HotkeysRoute,
})

function HotkeysRoute() {
  return <Hotkeys />
}
