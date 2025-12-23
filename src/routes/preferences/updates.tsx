import { createFileRoute } from '@tanstack/react-router'
import { Updates } from '@/components/preferences/Updates'

export const Route = createFileRoute('/preferences/updates')({
  component: UpdatesRoute,
})

function UpdatesRoute() {
  return <Updates />
}
