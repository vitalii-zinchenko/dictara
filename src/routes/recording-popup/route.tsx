import { createFileRoute, Outlet } from '@tanstack/react-router'
import { RecordingPopupLayout } from '@/components/recording/RecordingPopupLayout'

export const Route = createFileRoute('/recording-popup')({
  component: RecordingPopupRoute,
})

function RecordingPopupRoute() {
  return (
    <RecordingPopupLayout>
      <Outlet />
    </RecordingPopupLayout>
  )
}
