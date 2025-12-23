import { createFileRoute, Outlet } from '@tanstack/react-router'
import { PreferencesLayout } from '@/components/preferences/PreferencesLayout'

export const Route = createFileRoute('/preferences')({
  component: PreferencesRoute,
})

function PreferencesRoute() {
  return (
    <PreferencesLayout>
      <Outlet />
    </PreferencesLayout>
  )
}
