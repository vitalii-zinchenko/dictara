import { X } from 'lucide-react'
import type { RecordingStateChanged } from '@/bindings'

// Extract the error variant from the discriminated union
type RecordingError = Extract<RecordingStateChanged, { state: 'error' }>

interface ErrorStateProps {
  error: RecordingError
  onRetry: () => void
  onDismiss: () => void
  isRetryPending: boolean
  isDismissPending: boolean
}

export function ErrorState({
  error,
  onRetry,
  onDismiss,
  isRetryPending,
  isDismissPending,
}: ErrorStateProps) {
  return (
    <div className="flex items-center justify-between w-full h-full px-3 py-2 gap-2">
      {/* Error Message */}
      <div className="flex-1 min-w-0 overflow-hidden">
        <div className="text-red-400 text-xs font-semibold">
          {error.errorType === 'recording' ? 'Recording Failed' : 'Transcription Failed'}
        </div>
        <div className="text-gray-300 text-[10px] leading-tight line-clamp-2">
          {error.userMessage}
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-1.5 flex-shrink-0">
        {error.audioFilePath && (
          <button
            onClick={onRetry}
            disabled={isRetryPending}
            className="h-6 px-2 text-[10px] rounded bg-gray-600 hover:bg-gray-500 text-white font-medium transition-colors flex items-center disabled:opacity-50"
          >
            {isRetryPending ? '...' : 'Retry'}
          </button>
        )}
        <button
          onClick={onDismiss}
          disabled={isDismissPending}
          className="w-6 h-6 rounded bg-gray-600 hover:bg-gray-500 flex items-center justify-center transition-colors disabled:opacity-50"
        >
          <X className="w-3.5 h-3.5 text-white" strokeWidth={2.5} />
        </button>
      </div>
    </div>
  )
}
