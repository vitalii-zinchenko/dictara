import { useMutation } from '@tanstack/react-query'
import { commands } from '@/bindings'

/**
 * Hook to cancel recording.
 */
export function useCancelRecording() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.cancelRecording()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}

/**
 * Hook to stop recording.
 */
export function useStopRecording() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.stopRecording()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}

/**
 * Hook to retry transcription.
 */
export function useRetryTranscription() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.retryTranscription()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}

/**
 * Hook to dismiss error.
 */
export function useDismissError() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.dismissError()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}

/**
 * Hook to resize popup for error display.
 */
export function useResizePopupForError() {
  return useMutation({
    mutationFn: async (): Promise<void> => {
      const result = await commands.resizePopupForError()
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
  })
}
