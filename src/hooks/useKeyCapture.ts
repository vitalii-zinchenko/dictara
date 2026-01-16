import { useEffect, useState, useCallback } from 'react'
import { events, commands } from '@/bindings'

export interface CapturedKey {
  keycode: number
  label: string
}

export function useKeyCapture() {
  const [isCapturing, setIsCapturing] = useState(false)
  const [pressedKeys, setPressedKeys] = useState<CapturedKey[]>([])
  const [onAutoFinish, setOnAutoFinish] = useState<((keys: CapturedKey[]) => Promise<void>) | null>(
    null
  )

  const startCapture = useCallback(
    async (autoFinishCallback?: (keys: CapturedKey[]) => Promise<void>) => {
      setPressedKeys([])
      setIsCapturing(true)
      setOnAutoFinish(() => autoFinishCallback || null)
      const result = await commands.startKeyCapture()
      if (result.status === 'error') {
        console.error('Failed to start key capture:', result.error)
        setIsCapturing(false)
      }
    },
    []
  )

  const stopCapture = useCallback(async () => {
    setIsCapturing(false)
    setOnAutoFinish(null)
    const result = await commands.stopKeyCapture()
    if (result.status === 'error') {
      console.error('Failed to stop key capture:', result.error)
    }
  }, [])

  const clearKeys = useCallback(() => {
    setPressedKeys([])
  }, [])

  // Listen to key events from backend
  useEffect(() => {
    if (!isCapturing) return

    const setupListener = async () => {
      const unlisten = await events.keyCaptureEvent.listen((event) => {
        const payload = event.payload

        if (payload.type === 'keyDown') {
          setPressedKeys((prev) => {
            // Avoid duplicates
            if (prev.some((k) => k.keycode === payload.keycode)) {
              return prev
            }
            // Max 3 keys
            if (prev.length >= 3) {
              return prev
            }
            return [...prev, { keycode: payload.keycode, label: payload.label }]
          })
        } else if (payload.type === 'keyUp') {
          // Auto-finish when any key is released (if we have captured keys)
          setPressedKeys((prev) => {
            if (prev.length > 0 && onAutoFinish) {
              // Call auto-finish callback with current keys after state update
              setTimeout(async () => {
                await onAutoFinish(prev)
              }, 0)
            }
            return prev
          })
        }
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
  }, [isCapturing, onAutoFinish])

  return {
    isCapturing,
    pressedKeys,
    startCapture,
    stopCapture,
    clearKeys,
  }
}
