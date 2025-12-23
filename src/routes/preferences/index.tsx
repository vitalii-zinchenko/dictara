import { Navigate, createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/preferences/')({
  component: PreferencesIndex,
})

function PreferencesIndex() {
  return <Navigate to="/preferences/api-keys" />
}
