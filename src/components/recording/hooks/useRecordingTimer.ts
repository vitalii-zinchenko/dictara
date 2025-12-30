import { useCallback, useEffect, useRef, useState } from 'react'

// Maximum recording duration (10 minutes). Change to 10000 for 10-second testing
const MAX_RECORDING_DURATION_MS = 10 * 60 * 1000

interface UseRecordingTimerResult {
  elapsedMs: number
  startTimer: () => void
  cleanupTimer: () => void
}

export function useRecordingTimer(onTimeout: () => void): UseRecordingTimerResult {
  const [elapsedMs, setElapsedMs] = useState(0)
  const startTimeRef = useRef<number | undefined>(undefined)
  const timerIntervalRef = useRef<NodeJS.Timeout | undefined>(undefined)
  const onTimeoutRef = useRef(onTimeout)

  // Keep the ref updated with the latest callback
  useEffect(() => {
    onTimeoutRef.current = onTimeout
  }, [onTimeout])

  const cleanupTimer = useCallback(() => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current)
      timerIntervalRef.current = undefined
    }
    startTimeRef.current = undefined
    setElapsedMs(0)
  }, [])

  const startTimer = useCallback(() => {
    // Prevent restarting if already running
    if (timerIntervalRef.current) {
      return
    }

    // Set start time
    startTimeRef.current = Date.now()

    // Initialize display to max duration
    setElapsedMs(MAX_RECORDING_DURATION_MS)

    // Start countdown interval
    timerIntervalRef.current = setInterval(() => {
      if (startTimeRef.current === undefined) return

      const elapsed = Date.now() - startTimeRef.current
      const remaining = Math.max(0, MAX_RECORDING_DURATION_MS - elapsed)
      setElapsedMs(remaining)

      if (remaining <= 0) {
        if (timerIntervalRef.current) {
          clearInterval(timerIntervalRef.current)
          timerIntervalRef.current = undefined
        }
        startTimeRef.current = undefined
        onTimeoutRef.current()
      }
    }, 1000)
  }, [])

  return { elapsedMs, startTimer, cleanupTimer }
}
