import { createFileRoute } from '@tanstack/react-router'
import { ApiKeys } from '@/components/preferences/ApiKeys'

export const Route = createFileRoute('/preferences/api-keys')({
  component: ApiKeysRoute,
})

function ApiKeysRoute() {
  return <ApiKeys />
}
