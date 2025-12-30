import { useCallback, useRef } from 'react'
import './RecordingPopup.css'
import {
  useRecordingStateMachine,
  type RecordingStateChanged,
} from './hooks/useRecordingStateMachine'
import { useAudioLevel } from './hooks/useAudioLevel'
import { useRecordingTimer } from './hooks/useRecordingTimer'
import { useResizePopupForError } from '@/hooks/useRecording'
import { RecordingState } from './states/RecordingState'
import { TranscribingState } from './states/TranscribingState'
import { ErrorState } from './states/ErrorState'

function RecordingPopup() {
  const { smoothedLevel } = useAudioLevel()
  const resizePopupForError = useResizePopupForError()

  // Refs for timer functions (to break circular dependency)
  const timerFunctionsRef = useRef<{
    startTimer: () => void
    cleanupTimer: () => void
  } | null>(null)

  // Event handler for recording state changes (side effects)
  const handleRecordingEvent = useCallback(
    (event: RecordingStateChanged) => {
      const timerFns = timerFunctionsRef.current
      if (!timerFns) return

      switch (event.state) {
        case 'started':
          timerFns.startTimer()
          break

        case 'transcribing':
        case 'stopped':
        case 'cancelled':
          timerFns.cleanupTimer()
          break

        case 'error':
          timerFns.cleanupTimer()
          resizePopupForError.mutate()
          break
      }
    },
    [resizePopupForError]
  )

  const {
    state,
    error,
    handleCancel,
    handleStop,
    handleRetry,
    handleDismiss,
    isCancelPending,
    isStopPending,
    isRetryPending,
    isDismissPending,
  } = useRecordingStateMachine(handleRecordingEvent)

  const { elapsedMs, startTimer, cleanupTimer } = useRecordingTimer(handleStop)

  // Update refs with timer functions
  timerFunctionsRef.current = { startTimer, cleanupTimer }

  return (
    <div className="w-screen h-screen rounded-2xl border-[2px] border-gray-600 bg-gray-800 overflow-hidden font-sans">
      {/* Error State */}
      {state === 'error' && error && (
        <ErrorState
          error={error}
          onRetry={handleRetry}
          onDismiss={handleDismiss}
          isRetryPending={isRetryPending}
          isDismissPending={isDismissPending}
        />
      )}

      {/* Transcribing State */}
      {state === 'transcribing' && <TranscribingState />}

      {/* Recording State */}
      {state === 'recording' && (
        <RecordingState
          elapsedMs={elapsedMs}
          smoothedLevel={smoothedLevel}
          onCancel={handleCancel}
          onStop={handleStop}
          isCancelPending={isCancelPending}
          isStopPending={isStopPending}
        />
      )}
    </div>
  )
}

export default RecordingPopup
