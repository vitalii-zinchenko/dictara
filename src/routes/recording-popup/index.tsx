import { createFileRoute } from '@tanstack/react-router'
import RecordingPopup from '@/components/recording/RecordingPopup'

export const Route = createFileRoute('/recording-popup/')({
  component: RecordingPopupIndex,
})

function RecordingPopupIndex() {
  return <RecordingPopup />
}
