import { error as logError } from '@tauri-apps/plugin-log'
import { useEffect, useState, useCallback, useRef } from 'react'

import {
  useCancelRecording,
  useStopRecording,
  useRetryTranscription,
  useDismissError,
} from '@/hooks/useRecording'
import { events, type RecordingStateChanged } from '@/bindings'

export type RecordingState = 'recording' | 'transcribing' | 'error'

// Extract error type from the discriminated union
export type RecordingErrorPayload = Extract<RecordingStateChanged, { state: 'error' }>

// Re-export for external use
export type { RecordingStateChanged }

// Callback type for event handling
export type RecordingEventHandler = (event: RecordingStateChanged) => void

interface UseRecordingStateMachineResult {
  state: RecordingState
  error: RecordingErrorPayload | null
  handleCancel: () => Promise<void>
  handleStop: () => Promise<void>
  handleRetry: () => Promise<void>
  handleDismiss: () => Promise<void>
  isCancelPending: boolean
  isStopPending: boolean
  isRetryPending: boolean
  isDismissPending: boolean
}

export function useRecordingStateMachine(
  onEvent?: RecordingEventHandler
): UseRecordingStateMachineResult {
  const [state, setState] = useState<RecordingState>('recording')
  const [error, setError] = useState<RecordingErrorPayload | null>(null)

  // TanStack Query mutation hooks
  const cancelRecording = useCancelRecording()
  const stopRecording = useStopRecording()
  const retryTranscription = useRetryTranscription()
  const dismissError = useDismissError()

  // Keep onEvent in a ref to avoid re-subscribing when callback changes
  const onEventRef = useRef(onEvent)
  useEffect(() => {
    onEventRef.current = onEvent
  }, [onEvent])

  // Handlers
  const handleCancel = useCallback(async () => {
    try {
      await cancelRecording.mutateAsync()
    } catch (err) {
      logError(`Failed to cancel recording: ${err}`)
    }
  }, [cancelRecording])

  const handleStop = useCallback(async () => {
    try {
      await stopRecording.mutateAsync()
    } catch (err) {
      logError(`Failed to stop recording: ${err}`)
    }
  }, [stopRecording])

  const handleRetry = useCallback(async () => {
    setError(null)
    setState('transcribing')

    try {
      await retryTranscription.mutateAsync()
    } catch (err) {
      logError(`Retry failed: ${err}`)
      // Error will be re-emitted via event
    }
  }, [retryTranscription])

  const handleDismiss = useCallback(async () => {
    try {
      await dismissError.mutateAsync()
    } catch (err) {
      logError(`Failed to dismiss: ${err}`)
    }
  }, [dismissError])

  // Set up single typesafe event listener
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await events.recordingStateChanged.listen((event) => {
        const payload = event.payload

        // Update internal state based on event
        switch (payload.state) {
          case 'started':
            setState('recording')
            setError(null)
            break

          case 'transcribing':
            setState('transcribing')
            break

          case 'stopped':
            setState('recording')
            break

          case 'cancelled':
            setState('recording')
            break

          case 'error':
            setState('error')
            setError(payload)
            break
        }

        // Notify parent component about the event (for side effects like timer, resize)
        onEventRef.current?.(payload)
      })

      return unlisten
    }

    let cleanup: (() => void) | undefined
    setupListener().then((cleanupFn) => {
      cleanup = cleanupFn
    })

    return () => {
      if (cleanup) cleanup()
    }
  }, [])

  return {
    state,
    error,
    handleCancel,
    handleStop,
    handleRetry,
    handleDismiss,
    isCancelPending: cancelRecording.isPending,
    isStopPending: stopRecording.isPending,
    isRetryPending: retryTranscription.isPending,
    isDismissPending: dismissError.isPending,
  }
}
